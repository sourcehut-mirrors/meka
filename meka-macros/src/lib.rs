use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, LitStr, Token, braced,
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
    pub name: Option<LitStr>,
    pub map: Option<Vec<(LitStr, Expr)>>,
}

impl Parse for MekaInclude {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Handle empty input: meka_include!()
        if input.is_empty() {
            return Ok(MekaInclude {
                name: None,
                map: None,
            });
        }

        // Use lookahead to determine what we're parsing first.
        let lookahead = input.lookahead1();

        if lookahead.peek(LitStr) {
            // First argument is a string literal.
            let name = input.parse::<LitStr>()?;

            if input.is_empty() {
                // Case: meka_include!("some-string")
                Ok(MekaInclude {
                    name: Some(name),
                    map: None,
                })
            } else {
                // Case: meka_include!("some-string", {key => value, ...})
                input.parse::<Token![,]>()?;
                let map = parse_function_map(input)?;
                Ok(MekaInclude {
                    name: Some(name),
                    map: Some(map),
                })
            }
        } else if lookahead.peek(syn::token::Brace) {
            // Case: meka_include!({key => value, ...})
            let map = parse_function_map(input)?;
            Ok(MekaInclude {
                name: None,
                map: Some(map),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Parse a braced map of "string_key" => function_value pairs
fn parse_function_map(input: ParseStream) -> syn::Result<Vec<(LitStr, Expr)>> {
    let content;
    braced!(content in input);

    let mut pairs = Vec::new();

    while !content.is_empty() {
        // Parse key as string literal
        let key = content.parse::<LitStr>()?;
        content.parse::<Token![=>]>()?;

        // Parse value as expression (should be a function pointer)
        let value = content.parse::<Expr>()?;

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
        let tokens = match (self.name, self.map) {
            (Some(name), Some(map)) => {
                // Both name and map present
                // Generate a HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>>
                let map_entries = map.iter().map(|(key, value)| {
                    let key_str = &key.value();
                    quote! {
                        let _: fn(&mlua::Lua, mlua::Table, &std::primitive::str) -> mlua::Result<mlua::Function> = #value;
                        map.insert(std::borrow::Cow::from(#key_str), #value);
                    }
                });

                quote! {
                    {
                        let name = #name;
                        let mut map: std::collections::HashMap<std::borrow::Cow<'static, std::primitive::str>, fn(&mlua::Lua, mlua::Table, &std::primitive::str) -> mlua::Result<mlua::Function>> = std::collections::HashMap::new();
                        #(#map_entries)*

                        // TODO: Replace with your actual logic
                        println!("Name: {}, Map has {} entries", name, map.len());
                        for (k, _) in &map {
                            println!("  Key: {}", k);
                        }
                    }
                }
            }
            (Some(name), None) => {
                // Only name present
                quote! {
                    {
                        let name = #name;
                        // TODO: Replace with your actual logic
                        println!("Name only: {}", name);
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

#[cfg(test)]
mod inline_tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn empty_parse_works() {
        let input = quote! {};
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.name.is_none());
        assert!(parsed.map.is_none());
    }

    #[test]
    fn string_only_works() {
        let input = quote! { "test" };
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.name.is_some());
        assert_eq!(parsed.name.unwrap().value(), "test");
        assert!(parsed.map.is_none());
    }

    #[test]
    fn map_only_works() {
        let input = quote! { {"key1" => func1, "key2" => func2} };
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.name.is_none());
        assert!(parsed.map.is_some());
        assert_eq!(parsed.map.unwrap().len(), 2);
    }

    #[test]
    fn string_and_map_works() {
        let input = quote! { "test", {"key1" => func1, "key2" => func2} };
        let parsed: MekaInclude = parse2(input).unwrap();
        assert!(parsed.name.is_some());
        assert_eq!(parsed.name.unwrap().value(), "test");
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
