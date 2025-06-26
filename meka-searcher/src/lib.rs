use fennel_searcher::AddSearcher as _;
use meka_module_manifest::CompiledNamedTextManifest;
use meka_types::CatCowMap;
use mlua::Lua;
use mlua_module_manifest::{ModuleFileType, NamedTextManifest};
use mlua_searcher::AddSearcher as _;
use optional_collections::InsertOrInit;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::From;
use std::error;
use std::fmt;

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

impl From<NamedTextManifest> for MekaSearcher {
    fn from(manifest: NamedTextManifest) -> Self {
        MekaSearcher::RuntimeRead(RuntimeRead::from(manifest))
    }
}

/// Pre-categorized Fennel macro and Lua modules text indexed by name, with modules content
/// resolved at comptime.
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

/// Pre-categorized Fennel, Fennel macro and Lua modules paths/text indexed by name, with
/// modules content resolved at runtime.
pub struct RuntimeRead {
    /// For use with `mlua::Lua.add_cat_searcher_fnl()`.
    pub fnl: Option<CatCowMap>,

    /// For use with `mlua::Lua.add_cat_searcher_fnl_macros()`.
    pub fnl_macros: Option<CatCowMap>,

    /// For use with `mlua::Lua.add_cat_searcher()`.
    pub lua: Option<CatCowMap>,
}

impl From<NamedTextManifest> for RuntimeRead {
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
                if let Some(fnl_macros) = fnl_macros {
                    self.add_searcher_fnl_macros(fnl_macros)?;
                }
                if let Some(lua) = lua {
                    self.add_searcher(lua)?;
                }
            }
            MekaSearcher::RuntimeRead(RuntimeRead {
                fnl,
                fnl_macros,
                lua,
            }) => {
                if let Some(fnl) = fnl {
                    self.add_cat_searcher_fnl(fnl)?;
                }
                if let Some(fnl_macros) = fnl_macros {
                    self.add_cat_searcher_fnl_macros(fnl_macros)?;
                }
                if let Some(lua) = lua {
                    self.add_cat_searcher(lua)?;
                }
            }
        }
        Ok(())
    }
}
