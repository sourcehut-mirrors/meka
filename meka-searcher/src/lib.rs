use fennel_searcher::AddSearcher as _;
use io_cat::CatKind;
use meka_module_manifest::CompiledNamedTextManifest;
use meka_types::{CatCow, CatCowMap};
use mlua::Lua;
use mlua_module_manifest::{
    Manifest, Module, ModuleFileType, ModuleNamedFile, ModuleNamedText, Name,
};
use mlua_searcher::AddSearcher as _;
use optional_collections::InsertOrInit;
use quote::{ToTokens, quote};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::From;
use std::error;
use std::fmt;
use std::result::Result;

pub mod prelude {
    pub use crate::{
        AddMekaSearcher, AddMekaSearcherError, AddMekaSearcherResult, ComptimeEmbedded,
        MekaSearcher, RuntimeRead,
    };
}

#[derive(Debug)]
pub enum AddMekaSearcherError {
    FennelSearcherError(fennel_searcher::Error),
    LuaSearcherError(mlua_searcher::Error),
}

impl fmt::Display for AddMekaSearcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            AddMekaSearcherError::FennelSearcherError(error) => format!("{}", error),
            AddMekaSearcherError::LuaSearcherError(error) => format!("{}", error),
        };
        write!(f, "{}", res)
    }
}

impl From<fennel_searcher::Error> for AddMekaSearcherError {
    fn from(error: fennel_searcher::Error) -> Self {
        AddMekaSearcherError::FennelSearcherError(error)
    }
}

impl From<mlua_searcher::Error> for AddMekaSearcherError {
    fn from(error: mlua_searcher::Error) -> Self {
        AddMekaSearcherError::LuaSearcherError(error)
    }
}

impl error::Error for AddMekaSearcherError {}

pub type AddMekaSearcherResult<A> = Result<A, AddMekaSearcherError>;

/// Pre-categorized Fennel, Fennel macro and Lua modules paths/text indexed by name, with
/// modules content resolved at comptime or runtime, enum variant depending.
///
/// The enum variants are named after the stage at which the modules contained therein might
/// plausibly be available for reading.
#[derive(Clone, Debug)]
pub enum MekaSearcher {
    /// Modules contained herein may be available for reading at comptime.
    ComptimeEmbedded(ComptimeEmbedded),
    /// Modules contained herein may be available for reading at runtime.
    RuntimeRead(RuntimeRead),
}

impl From<CompiledNamedTextManifest> for MekaSearcher {
    fn from(manifest: CompiledNamedTextManifest) -> Self {
        MekaSearcher::ComptimeEmbedded(ComptimeEmbedded::from(manifest))
    }
}

impl From<Manifest> for MekaSearcher {
    fn from(manifest: Manifest) -> Self {
        MekaSearcher::RuntimeRead(RuntimeRead::from(manifest))
    }
}

impl ToTokens for MekaSearcher {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expanded = match self {
            MekaSearcher::ComptimeEmbedded(comptime_embedded) => {
                quote! {
                    ::meka::MekaSearcher::ComptimeEmbedded(#comptime_embedded)
                }
            }
            MekaSearcher::RuntimeRead(runtime_read) => {
                quote! {
                    ::meka::MekaSearcher::RuntimeRead(#runtime_read)
                }
            }
        };
        tokens.extend(expanded);
    }
}

/// Pre-categorized Fennel macro and Lua modules text indexed by name, with modules content
/// resolved at comptime.
#[derive(Clone, Debug)]
pub struct ComptimeEmbedded {
    /// For use with `mlua::Lua.add_searcher_fnl_macros()`.
    pub fnl_macros: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,

    /// For use with `mlua::Lua.add_searcher()`.
    pub lua: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
}

impl From<CompiledNamedTextManifest> for ComptimeEmbedded {
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

impl ToTokens for ComptimeEmbedded {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fnl_macros_tokens = to_tokens_for_optional_cowmap(&self.fnl_macros);
        let lua_tokens = to_tokens_for_optional_cowmap(&self.lua);
        let expanded = quote! {
            ::meka::ComptimeEmbedded {
                fnl_macros: #fnl_macros_tokens,
                lua: #lua_tokens,
            }
        };
        tokens.extend(expanded);
    }
}

fn to_tokens_for_optional_cowmap(
    cowmap: &Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
) -> proc_macro2::TokenStream {
    match cowmap {
        None => quote! { None },
        Some(map) => {
            let entries = map.iter().map(|(key, value)| {
                let key_str = key.as_ref();
                let value_str = value.as_ref();
                quote! {
                    (::std::borrow::Cow::from(#key_str), ::std::borrow::Cow::from(#value_str))
                }
            });
            quote! {
                Some(::std::collections::HashMap::from([#(#entries),*]))
            }
        }
    }
}

/// Pre-categorized Fennel, Fennel macro and Lua modules paths/text indexed by name, with
/// modules content resolved at runtime.
#[derive(Clone, Debug)]
pub struct RuntimeRead {
    /// For use with `mlua::Lua.add_cat_searcher_fnl()`.
    pub fnl: Option<CatCow>,

    /// For use with `mlua::Lua.add_cat_searcher_fnl_macros()`.
    pub fnl_macros: Option<CatCow>,

    /// For use with `mlua::Lua.add_cat_searcher()`.
    pub lua: Option<CatCow>,
}

impl From<Manifest> for RuntimeRead {
    fn from(manifest: Manifest) -> Self {
        let mut fnl: Option<CatCowMap> = None;
        let mut fnl_macros: Option<CatCowMap> = None;
        let mut lua: Option<CatCowMap> = None;
        for module in manifest.modules.into_iter() {
            match module {
                Module::File(module_file) => {
                    let name = module_file.name();
                    match module_file.file_type {
                        ModuleFileType::Fennel => {
                            fnl.insert_or_init(name, CatKind::from_path(module_file.path));
                        }
                        ModuleFileType::FennelMacros => {
                            fnl_macros.insert_or_init(name, CatKind::from_path(module_file.path));
                        }
                        ModuleFileType::Lua => {
                            lua.insert_or_init(name, CatKind::from_path(module_file.path));
                        }
                    }
                }
                Module::NamedFile(ModuleNamedFile {
                    name,
                    path,
                    file_type,
                }) => match file_type {
                    ModuleFileType::Fennel => {
                        fnl.insert_or_init(name, CatKind::from_path(path));
                    }
                    ModuleFileType::FennelMacros => {
                        fnl_macros.insert_or_init(name, CatKind::from_path(path));
                    }
                    ModuleFileType::Lua => {
                        lua.insert_or_init(name, CatKind::from_path(path));
                    }
                },
                Module::NamedText(ModuleNamedText {
                    name,
                    text,
                    file_type,
                }) => match file_type {
                    ModuleFileType::Fennel => {
                        fnl.insert_or_init(name, CatKind::from_str(text));
                    }
                    ModuleFileType::FennelMacros => {
                        fnl_macros.insert_or_init(name, CatKind::from_str(text));
                    }
                    ModuleFileType::Lua => {
                        lua.insert_or_init(name, CatKind::from_str(text));
                    }
                },
            }
        }
        let fnl = if let Some(fnl) = fnl {
            Some(CatCow(fnl))
        } else {
            None
        };
        let fnl_macros = if let Some(fnl_macros) = fnl_macros {
            Some(CatCow(fnl_macros))
        } else {
            None
        };
        let lua = if let Some(lua) = lua {
            Some(CatCow(lua))
        } else {
            None
        };
        Self {
            fnl,
            fnl_macros,
            lua,
        }
    }
}

impl ToTokens for RuntimeRead {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fnl_tokens = to_tokens_for_optional_catcow(&self.fnl);
        let fnl_macros_tokens = to_tokens_for_optional_catcow(&self.fnl_macros);
        let lua_tokens = to_tokens_for_optional_catcow(&self.lua);
        let expanded = quote! {
            ::meka::RuntimeRead {
                fnl: #fnl_tokens,
                fnl_macros: #fnl_macros_tokens,
                lua: #lua_tokens,
            }
        };
        tokens.extend(expanded);
    }
}

fn to_tokens_for_optional_catcow(catcow: &Option<CatCow>) -> proc_macro2::TokenStream {
    match catcow {
        None => quote! { None },
        Some(catcow) => {
            let entries = catcow.0.iter().map(|(key, cat_kind)| {
                let key_str = key.as_ref();
                quote! {
                    (::std::borrow::Cow::from(#key_str), #cat_kind)
                }
            });
            quote! {
                Some(::meka::CatCow(::meka::CatCowMap::from([#(#entries),*])))
            }
        }
    }
}

/// Extend `mlua::Lua` to support `require`ing Fennel, Fennel macro and Lua modules provided
/// directly in the form of text or indirectly in the form of paths, and to support importing
/// said modules by name.
pub trait AddMekaSearcher {
    /// Add a `HashMap` of Fennel modules indexed by module name to Lua's `package.searchers` table
    /// table in an `mlua::Lua`, with lookup functionality provided by the `mlua_searcher::PolySearcher`
    /// struct, and Fennel-to-Lua compilation done on-the-fly with `fennel-compile`.
    ///
    /// Add a `HashMap` of Fennel macro modules indexed by module name to Fennel's
    /// `fennel.macro-searchers` table in an `mlua::Lua`, with lookup functionality provided
    /// by the `fennel_searcher::MacroSearcher` or `fennel_searcher::MacroCatSearcher` struct.
    ///
    /// Add a `HashMap` of Lua modules indexed by module name to Lua's `package.searchers` table
    /// in an `mlua::Lua`, with lookup functionality provided by the `mlua_searcher::Searcher`
    /// or `mlua_searcher::CatSearcher` struct.
    fn add_meka_searcher(&self, meka_searcher: MekaSearcher) -> AddMekaSearcherResult<()>;
}

impl AddMekaSearcher for Lua {
    fn add_meka_searcher(&self, meka_searcher: MekaSearcher) -> AddMekaSearcherResult<()> {
        match meka_searcher {
            MekaSearcher::ComptimeEmbedded(ComptimeEmbedded { fnl_macros, lua }) => {
                if let Some(lua) = lua {
                    self.add_searcher(lua)?;
                }
                if let Some(fnl_macros) = fnl_macros {
                    self.add_searcher_fnl_macros(fnl_macros)?;
                }
            }
            MekaSearcher::RuntimeRead(RuntimeRead {
                fnl,
                fnl_macros,
                lua,
            }) => {
                if let Some(lua) = lua {
                    self.add_cat_searcher(lua)?;
                }
                if let Some(fnl) = fnl {
                    self.add_cat_searcher_fnl(fnl)?;
                }
                if let Some(fnl_macros) = fnl_macros {
                    self.add_cat_searcher_fnl_macros(fnl_macros)?;
                }
            }
        }
        Ok(())
    }
}
