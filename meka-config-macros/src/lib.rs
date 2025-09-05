use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::result::Result;
use std::string::String;

#[derive(Debug)]
enum LoaderRegistryError {
    IoError(std::io::Error),
    TomlError(toml::de::Error),
    MissingEnvCargoManifestDir,
    MissingMetadata,
    // key, reason
    InvalidLoaderRegistry(String, String),
    // key, reason
    InvalidLoader(String, String),
}

impl fmt::Display for LoaderRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderRegistryError::IoError(e) => write!(f, "IO error: {}", e),
            LoaderRegistryError::TomlError(e) => write!(f, "TOML parsing error: {}", e),
            LoaderRegistryError::MissingEnvCargoManifestDir => {
                write!(f, "No $CARGO_MANIFEST_DIR found")
            }
            LoaderRegistryError::MissingMetadata => {
                write!(f, "No package.metadata.meka section found")
            }
            LoaderRegistryError::InvalidLoaderRegistry(key, reason) => {
                write!(f, "Invalid loader registry '{}': {}", key, reason)
            }
            LoaderRegistryError::InvalidLoader(key, reason) => {
                write!(f, "Invalid loader '{}': {}", key, reason)
            }
        }
    }
}

#[proc_macro]
pub fn loader_registry_from_cargo_manifest(_input: TokenStream) -> TokenStream {
    match get_loaders_from_cargo_toml() {
        Ok(loaders) => generate_loader_registry_tokens(loaders),
        Err(LoaderRegistryError::MissingEnvCargoManifestDir)
        | Err(LoaderRegistryError::MissingMetadata) => {
            // Generate empty map for missing metadata
            generate_empty_loader_registry()
        }
        Err(e) => {
            // Generate compile error for other issues
            let error_msg = e.to_string();
            let expanded = quote! { compile_error!(#error_msg); };
            TokenStream::from(expanded)
        }
    }
}

fn generate_empty_loader_registry() -> TokenStream {
    let expanded = quote! {
        ::std::collections::HashMap::<
            ::std::borrow::Cow<'static, ::std::primitive::str>,
            fn(&::mlua::Lua, ::mlua::Table, &::std::primitive::str) -> ::mlua::Result<::mlua::Function>
        >::new()
    };
    TokenStream::from(expanded)
}

fn generate_loader_registry_tokens(loaders: HashMap<String, String>) -> TokenStream {
    if loaders.is_empty() {
        return generate_empty_loader_registry();
    }

    let entries = loaders.iter().map(|(name, path)| {
        let path_tokens: proc_macro2::TokenStream = path.parse()
            .unwrap_or_else(|_| {
                // This will cause a compile error if the path is invalid
                quote! { compile_error!(concat!("Invalid loader path: ", #path)) }
            });

        quote! {
            let _: fn(&::mlua::Lua, ::mlua::Table, &::std::primitive::str) -> ::mlua::Result<::mlua::Function> = #path_tokens;
            map.insert(::std::borrow::Cow::from(#name), #path_tokens);
        }
    });

    let entries_len = entries.len();

    let expanded = quote! {
        {
            let mut map: ::std::collections::HashMap<
                ::std::borrow::Cow<'static, ::std::primitive::str>,
                fn(&::mlua::Lua, ::mlua::Table, &::std::primitive::str) -> ::mlua::Result<::mlua::Function>
            > = ::std::collections::HashMap::with_capacity(#entries_len);
            #(#entries)*
            map
        }
    };

    TokenStream::from(expanded)
}

fn get_loaders_from_cargo_toml() -> Result<HashMap<String, String>, LoaderRegistryError> {
    let cargo_toml: toml::Value = {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| LoaderRegistryError::MissingEnvCargoManifestDir)?;
        let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");
        let cargo_toml_content =
            fs::read_to_string(&cargo_toml_path).map_err(LoaderRegistryError::IoError)?;
        toml::from_str(&cargo_toml_content).map_err(LoaderRegistryError::TomlError)?
    };

    let metadata_table = {
        let package = cargo_toml
            .get("package")
            .ok_or(LoaderRegistryError::MissingMetadata)?;
        let metadata = package
            .get("metadata")
            .ok_or(LoaderRegistryError::MissingMetadata)?;
        let meka = metadata
            .get("meka")
            .ok_or(LoaderRegistryError::MissingMetadata)?;
        meka.as_table()
            .ok_or(LoaderRegistryError::InvalidLoaderRegistry(
                "package.metadata.meka".to_string(),
                "package.metadata.meka must be a table".to_string(),
            ))?
    };

    let loaders_table = {
        let loaders = metadata_table
            .get("loaders")
            .ok_or(LoaderRegistryError::MissingMetadata)?;
        loaders.as_table().ok_or_else(|| {
            LoaderRegistryError::InvalidLoaderRegistry(
                "loaders".to_string(),
                "loaders must be a table".to_string(),
            )
        })?
    };

    loaders_table
        .iter()
        .map(|(name, loader_path)| parse_loader_entry(name, loader_path))
        .collect::<Result<HashMap<String, String>, LoaderRegistryError>>()
}

fn parse_loader_entry(
    name: &str,
    loader_path: &toml::Value,
) -> Result<(String, String), LoaderRegistryError> {
    let loader_path = loader_path.as_str().ok_or_else(|| {
        LoaderRegistryError::InvalidLoader(
            name.to_string(),
            "loader path must be a string".to_string(),
        )
    })?;

    validate_loader_path(name, loader_path)?;

    Ok((name.to_string(), loader_path.to_string()))
}

fn validate_loader_path(name: &str, loader_path: &str) -> Result<(), LoaderRegistryError> {
    if loader_path.is_empty() {
        return Err(LoaderRegistryError::InvalidLoader(
            name.to_string(),
            "loader path cannot be empty".to_string(),
        ));
    }

    if !loader_path.contains("::") {
        return Err(LoaderRegistryError::InvalidLoader(
            name.to_string(),
            "loader path must contain '::' (e.g., 'crate::function')".to_string(),
        ));
    }

    Ok(())
}

#[proc_macro]
pub fn loader_paths_from_cargo_manifest(_input: TokenStream) -> TokenStream {
    match get_loaders_from_cargo_toml() {
        Ok(loaders) => generate_loader_paths_tokens(loaders),
        Err(LoaderRegistryError::MissingEnvCargoManifestDir)
        | Err(LoaderRegistryError::MissingMetadata) => generate_empty_loader_paths(),
        Err(e) => {
            let error_msg = e.to_string();
            let expanded = quote! { compile_error!(#error_msg); };
            TokenStream::from(expanded)
        }
    }
}

fn generate_loader_paths_tokens(loaders: HashMap<String, String>) -> TokenStream {
    if loaders.is_empty() {
        return generate_empty_loader_paths();
    }

    let entries = loaders.iter().map(|(name, path)| {
        quote! {
            (#name.to_string(), #path.to_string())
        }
    });

    let expanded = quote! {
        vec![#(#entries),*]
    };

    TokenStream::from(expanded)
}

fn generate_empty_loader_paths() -> TokenStream {
    let expanded = quote! {
        ::std::vec::Vec::<(::std::string::String, ::std::string::String)>::new()
    };
    TokenStream::from(expanded)
}
