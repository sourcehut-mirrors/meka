use meka_config::{Config, LoaderRegistry};
use meka_module_manifest::CompiledNamedTextManifest;
use meka_searcher::MekaSearcher;
use mlua::{Function, Lua, Table};
use mlua_module_manifest::{Manifest, Module, ModuleFile, ModuleFileType, NamedTextManifest};
use optional_collections::InsertOrInit;
use proc_macro::TokenStream;
use quote::quote;
use std::borrow::Cow;
use std::boxed::Box;
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::path::PathBuf;
use std::vec::Vec;
use syn::{
    LitStr, Path, Token, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[proc_macro]
pub fn meka_include(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as MekaInclude);
    parsed
        .expand()
        .unwrap_or_else(|err| err.to_compile_error())
        // Convert `proc_macro2::TokenStream` to `proc_macro::TokenStream`.
        .into()
}

struct MekaInclude {
    pub key: Option<LitStr>,
    pub map: Option<Vec<(LitStr, Path)>>,
}

impl Parse for MekaInclude {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Handle empty input: meka_include!()
        if input.is_empty() {
            return Ok(MekaInclude {
                key: None,
                map: None,
            });
        }

        // Use lookahead to determine what we're parsing first.
        let lookahead = input.lookahead1();

        if lookahead.peek(LitStr) {
            // First argument is a string literal.
            let key = input.parse::<LitStr>()?;

            if input.is_empty() {
                // Case: meka_include!("some-string")
                Ok(MekaInclude {
                    key: Some(key),
                    map: None,
                })
            } else {
                // Case: meka_include!("some-string", {key => value, ...})
                input.parse::<Token![,]>()?;
                let map = parse_function_map(input)?;
                Ok(MekaInclude {
                    key: Some(key),
                    map: Some(map),
                })
            }
        } else if lookahead.peek(syn::token::Brace) {
            // Case: meka_include!({key => value, ...})
            let map = parse_function_map(input)?;
            Ok(MekaInclude {
                key: None,
                map: Some(map),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Parse a braced map of "string_key" => function_ident pairs
fn parse_function_map(input: ParseStream) -> syn::Result<Vec<(LitStr, Path)>> {
    let content;
    braced!(content in input);

    let mut pairs = Vec::new();

    while !content.is_empty() {
        // Parse key as string literal
        let key = content.parse::<LitStr>()?;
        content.parse::<Token![=>]>()?;

        // Parse value as identifier (function name)
        let value = content.parse::<Path>()?;

        pairs.push((key, value));

        // Handle optional comma (including trailing comma)
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        } else {
            break;
        }
    }

    Ok(pairs)
}

impl MekaInclude {
    /// Returns `proc_macro2::TokenStream` for testability.
    fn expand(self) -> syn::Result<proc_macro2::TokenStream> {
        let tokens = match (self.key, self.map) {
            (Some(key), Some(map)) => {
                // Both key and map present
                let tokens = config_new_with_map(map);
                let (_, path_str, _) = selected_path();
                quote! {{
                    #tokens
                    let key: &str = #key.as_ref();
                    let manifest = if let Some(manifest) = config.get(key) {
                        ::meka_module_manifest::CompiledNamedTextManifest::try_from((*manifest).clone())
                            .expect("Sorry, couldn't convert Manifest into CompiledNamedTextManifest")
                    } else {
                        panic!("Sorry, couldn't find key {} in Meka manifest at {}", key, #path_str);
                    };
                    ::meka_searcher::MekaSearcher::from(manifest)
                }}
            }
            (Some(key), None) => {
                // Only key present
                let config = config_new_without_map();
                let (_, path_str, _) = selected_path();
                let key = key.value();
                let key: &str = key.as_ref();
                let manifest = if let Some(manifest) = config.get(key) {
                    CompiledNamedTextManifest::try_from((*manifest).clone())
                        .expect("Sorry, couldn't convert Manifest into CompiledNamedTextManifest")
                } else {
                    panic!(
                        "Sorry, couldn't find key {} in Meka manifest at {}",
                        key, path_str
                    );
                };
                let searcher = MekaSearcher::from(manifest);
                quote! { #searcher }
            }
            (None, Some(map)) => {
                // Only map present
                let tokens = config_new_with_map(map);
                let (_, path_str, _) = selected_path();
                quote! {{
                    #tokens
                    let manifest = if let Some(manifest) = config.get("") {
                        ::meka_module_manifest::CompiledNamedTextManifest::try_from((*manifest).clone())
                            .expect("Sorry, couldn't convert Manifest into CompiledNamedTextManifest")
                    } else {
                        panic!("Sorry, couldn't find Meka manifest at {}", #path_str);
                    };
                    ::meka_searcher::MekaSearcher::from(manifest)
                }}
            }
            (None, None) => {
                // Empty macro call
                let config = config_new_without_map();
                let (_, path_str, _) = selected_path();
                let manifest = if let Some(manifest) = config.get("") {
                    CompiledNamedTextManifest::try_from((*manifest).clone())
                        .expect("Sorry, couldn't convert Manifest into CompiledNamedTextManifest")
                } else {
                    panic!("Sorry, couldn't find Meka manifest at {}", path_str);
                };
                let searcher = MekaSearcher::from(manifest);
                quote! { #searcher }
            }
        };

        Ok(tokens.into())
    }
}

fn config_new_with_map(map: Vec<(LitStr, Path)>) -> proc_macro2::TokenStream {
    let map_entries = map_entries(map);
    let map_entries_len = map_entries.len();
    let module = module_from_path();
    quote! {
        let mut loader_registry = ::meka_config::LoaderRegistry::with_capacity(#map_entries_len);
        #(#map_entries)*
        let config: ::std::collections::HashMap<::std::string::String, ::mlua_module_manifest::Manifest> = ::meka_config::Config::new(#module, Some(loader_registry))
            .expect("Sorry, couldn't instantiate Config")
            .0;
    }
}

fn map_entries(map: Vec<(LitStr, Path)>) -> Vec<proc_macro2::TokenStream> {
    let map_entries = map.iter().map(|(key, value)| {
        quote! {
            let _: fn(&::mlua::Lua, ::mlua::Table, &::std::primitive::str) -> ::mlua::Result<::mlua::Function> = #value;
            loader_registry.insert(::std::borrow::Cow::from(#key), #value);
        }
    }).collect();
    map_entries
}

fn module_from_path() -> Module {
    let (path, path_str, file_type) = selected_path();
    let module = ModuleFile::new(path.clone(), Some(file_type)).expect(&format!(
        "Sorry, couldn't instantiate Module from path {}",
        path_str
    ));
    Module::File(module)
}

fn selected_path() -> (PathBuf, String, ModuleFileType) {
    let runtime_root =
        ::meka_utils::runtime_root().expect("Sorry, couldn't get $CARGO_MANIFEST_DIR");

    let path_fnl = runtime_root.join("manifest.fnl");
    let path_init_fnl = runtime_root.join("manifest").join("init.fnl");
    let path_lua = runtime_root.join("manifest.lua");
    let path_init_lua = runtime_root.join("manifest").join("init.lua");

    // For improved error messages.
    let path_fnl_str = path_fnl.to_string_lossy().into_owned();
    let path_init_fnl_str = path_init_fnl.to_string_lossy().into_owned();
    let path_lua_str = path_lua.to_string_lossy().into_owned();
    let path_init_lua_str = path_init_lua.to_string_lossy().into_owned();

    if path_fnl.is_file() {
        (path_fnl, path_fnl_str, ModuleFileType::Fennel)
    } else if path_init_fnl.is_file() {
        (path_init_fnl, path_init_fnl_str, ModuleFileType::Fennel)
    } else if path_lua.is_file() {
        (path_lua, path_lua_str, ModuleFileType::Lua)
    } else if path_init_lua.is_file() {
        (path_init_lua, path_init_lua_str, ModuleFileType::Lua)
    } else {
        panic!("Sorry, couldn't find Meka manifest in $CARGO_MANIFEST_DIR");
    }
}

fn config_new_without_map() -> HashMap<String, Manifest> {
    let module = module_from_path();
    Config::new(module, None)
        .expect("Sorry, couldn't instantiate Config")
        .0
}

#[cfg(test)]
mod inline_tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn empty_parse_works() {
        let input = quote! {};
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.key.is_none());
        assert!(parsed.map.is_none());
    }

    #[test]
    fn string_only_works() {
        let input = quote! { "test" };
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.key.is_some());
        assert_eq!(parsed.key.unwrap().value(), "test");
        assert!(parsed.map.is_none());
    }

    #[test]
    fn map_only_works() {
        let input = quote! { {"key1" => func1, "key2" => func2} };
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.key.is_none());
        assert!(parsed.map.is_some());
        assert_eq!(parsed.map.unwrap().len(), 2);
    }

    #[test]
    fn string_and_map_works() {
        let input = quote! { "test", {"key1" => func1, "key2" => func2} };
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.key.is_some());
        assert_eq!(parsed.key.unwrap().value(), "test");
        assert!(parsed.map.is_some());
        assert_eq!(parsed.map.unwrap().len(), 2);
    }

    #[test]
    #[ignore = "requires separate testcrate for manifest and associated modules"]
    fn expand_empty_works() {
        let input = quote! {};
        let parsed: MekaInclude = parse2(input).unwrap();
        let expanded = parsed.expand().unwrap();
        // Works because we're using `proc_macro2::TokenStream`.
        let expanded_string = expanded.to_string();
        assert!(!expanded_string.is_empty());
        assert!(expanded_string.contains("Empty meka_include call"));
    }

    #[test]
    fn invalid_syntax_fails() {
        let input = quote! { 123 }; // Invalid: not a string or map
        let result: Result<MekaInclude, _> = parse2(input);
        assert!(result.is_err());
    }
}
