use meka_config::{Config, Env};
use meka_module_manifest::CompiledNamedTextManifest;
use meka_types::CatCowMap;
use mlua::{Function, Lua, Table};
use mlua_module_manifest::{Manifest, Module, ModuleFile, ModuleFileType, NamedTextManifest};
use optional_collections::InsertOrInit;
use proc_macro::TokenStream;
use quote::quote;
use std::borrow::Cow;
use std::boxed::Box;
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::vec::Vec;
use syn::{
    Ident, LitStr, Token, braced,
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
    pub map: Option<Vec<(LitStr, Ident)>>,
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
fn parse_function_map(input: ParseStream) -> syn::Result<Vec<(LitStr, Ident)>> {
    let content;
    braced!(content in input);

    let mut pairs = Vec::new();

    while !content.is_empty() {
        // Parse key as string literal
        let key = content.parse::<LitStr>()?;
        content.parse::<Token![=>]>()?;

        // Parse value as identifier (function name)
        let value = content.parse::<Ident>()?;

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
                let mut env: Env = HashMap::with_capacity(map.len());
                let map_entries = map.iter().map(|(key, value)| {
                    let key_str = &key.value();
                    let fn_ptr: fn(&Lua, Table, &str) -> mlua::Result<Function> = value.value();
                    env.insert(Cow::from(key_str), fn_ptr);
                });

                let runtime_root =
                    meka_utils::runtime_root().expect("Sorry, couldn't get $CARGO_MANIFEST_DIR");

                let path_fnl = runtime_root.join("manifest.fnl");
                let path_init_fnl = runtime_root.join("manifest").join("init.fnl");
                let path_lua = runtime_root.join("manifest.lua");
                let path_init_lua = runtime_root.join("manifest").join("init.lua");

                let module = if path_fnl.is_file() {
                    Module::File(
                        ModuleFile::new(path_fnl, Some(ModuleFileType::Fennel)).expect(&format!(
                            "Sorry, couldn't instantiate Module from path {:?}",
                            path_fnl
                        )),
                    )
                } else if path_init_fnl.is_file() {
                    Module::File(
                        ModuleFile::new(path_init_fnl, Some(ModuleFileType::Fennel)).expect(
                            &format!(
                                "Sorry, couldn't instantiate Module from path {:?}",
                                path_init_fnl
                            ),
                        ),
                    )
                } else if path_lua.is_file() {
                    Module::File(ModuleFile::new(path_lua, Some(ModuleFileType::Lua)).expect(
                        &format!(
                            "Sorry, couldn't instantiate Module from path {:?}",
                            path_lua
                        ),
                    ))
                } else if path_init_lua.is_file() {
                    Module::File(
                        ModuleFile::new(path_init_lua, Some(ModuleFileType::Lua)).expect(&format!(
                            "Sorry, couldn't instantiate Module from path {:?}",
                            path_init_lua
                        )),
                    )
                } else {
                    panic!("Sorry, couldn't find Meka manifest in $CARGO_MANIFEST_DIR");
                };

                let config: HashMap<String, Manifest> = Config::new(module, Some(env))
                    .expect("Sorry, couldn't instantiate Config")
                    .0;

                let key = key.value();

                let manifest = if let Some(manifest) = config.get(&key) {
                    CompiledNamedTextManifest::try_from(*manifest)
                        .expect("Sorry, couldn't convert Manifest into CompiledNamedTextManifest")
                } else {
                    panic!("Sorry, couldn't find key {} in Meka manifest", key);
                };

                let include = Include::from(manifest);

                quote! {
                    {
                        let key = #key;
                        let mut map: std::collections::HashMap<std::borrow::Cow<'static, std::primitive::str>, fn(&mlua::Lua, mlua::Table, &std::primitive::str) -> mlua::Result<mlua::Function>> = std::collections::HashMap::new();
                        #(#map_entries)*

                        // TODO: Replace with your actual logic
                        println!("Key: {}, Map has {} entries", key, map.len());
                        for (k, _) in &map {
                            println!("  Key: {}", k);
                        }
                    }
                }
            }
            (Some(key), None) => {
                // Only key present
                quote! {
                    {
                        let key = #key;
                        // TODO: Replace with your actual logic
                        println!("Key only: {}", key);
                    }
                }
            }
            (None, Some(map)) => {
                // Only map present
                let map_entries = map.iter().map(|(key, value)| {
                    let key_str = &key.value();
                    quote! {
                        let _: fn(&mlua::Lua, mlua::Table, &std::primitive::str) -> mlua::Result<mlua::Function> = #value;
                        map.insert(std::borrow::Cow::from(#key_str), #value);
                    }
                });

                quote! {
                    {
                        let mut map: std::collections::HashMap<std::borrow::Cow<'static, std::primitive::str>, fn(&mlua::Lua, mlua::Table, &std::primitive::str) -> mlua::Result<mlua::Function>> = std::collections::HashMap::new();
                        #(#map_entries)*

                        // TODO: Replace with your actual logic
                        println!("Map only with {} entries", map.len());
                        for (k, _) in &map {
                            println!("  Key: {}", k);
                        }
                    }
                }
            }
            (None, None) => {
                // Empty macro call
                quote! {
                    {
                        // TODO: Replace with your actual logic
                        println!("Empty meka_include call");
                    }
                }
            }
        };

        Ok(tokens.into())
    }
}

struct Include {
    /// For use with `mlua::Lua.add_searcher_fnl_macros()`.
    pub fnl_macros: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,

    /// For use with `mlua::Lua.add_searcher()`.
    pub lua: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
}

impl From<CompiledNamedTextManifest> for Include {
    fn from(manifest: CompiledNamedTextManifest) -> Self {
        let mut fnl_macros: Option<HashMap<Cow<'static, str>, Cow<'static, str>>> = None;
        let mut lua: Option<HashMap<Cow<'static, str>, Cow<'static, str>>> = None;
        for module in manifest.modules.into_iter() {
            match module.file_type {
                // Fennel has already been AOT-compiled to Lua.
                ModuleFileType::Fennel | ModuleFileType::Lua => {
                    lua.insert_or_init(module.name, module.text);
                }
                ModuleFileType::FennelMacros => {
                    fnl_macros.insert_or_init(module.name, module.text);
                }
            }
        }
        Self { fnl_macros, lua }
    }
}

struct Load {
    /// For use with `mlua::Lua.add_cat_searcher_fnl()`.
    pub fnl: Option<CatCowMap>,

    /// For use with `mlua::Lua.add_cat_searcher_fnl_macros()`.
    pub fnl_macros: Option<CatCowMap>,

    /// For use with `mlua::Lua.add_cat_searcher()`.
    pub lua: Option<CatCowMap>,
}

impl From<NamedTextManifest> for Load {
    fn from(manifest: NamedTextManifest) -> Self {
        let mut fnl: Option<CatCowMap> = None;
        let mut fnl_macros: Option<CatCowMap> = None;
        let mut lua: Option<CatCowMap> = None;
        for module in manifest.modules.into_iter() {
            match module.file_type {
                ModuleFileType::Fennel => {
                    fnl.insert_or_init(module.name, Box::new(module.text));
                }
                ModuleFileType::FennelMacros => {
                    fnl_macros.insert_or_init(module.name, Box::new(module.text));
                }
                ModuleFileType::Lua => {
                    lua.insert_or_init(module.name, Box::new(module.text));
                }
            }
        }
        Self {
            fnl,
            fnl_macros,
            lua,
        }
    }
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
