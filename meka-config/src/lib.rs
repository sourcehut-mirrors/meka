use mlua::{Function, Lua, LuaOptions, StdLib, Table};
use mlua_module_manifest::{Manifest, Module};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::From;
use std::error;
use std::fmt;
use std::path::Path;
use std::result::Result;

/// Error message designed for running `table.get(key)` on `mlua::Table` `table` verified to
/// contain key `key`.
const TABLE_GET_EXPECT: &str = "Unexpectedly couldn't get key from pre-checked table";

#[derive(Debug)]
pub enum ConfigInitError {
    FennelCompileError(fennel_compile::Error),
    FennelMountError(fennel_mount::Error),
    FennelSearcherError(fennel_searcher::Error),
    Lua(mlua::Error),
}

impl fmt::Display for ConfigInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ConfigInitError::FennelCompileError(error) => format!("{}", error),
            ConfigInitError::FennelMountError(error) => format!("{}", error),
            ConfigInitError::FennelSearcherError(error) => format!("{}", error),
            ConfigInitError::Lua(error) => format!("{}", error),
        };
        write!(f, "{}", res)
    }
}

impl From<fennel_compile::Error> for ConfigInitError {
    fn from(error: fennel_compile::Error) -> Self {
        ConfigInitError::FennelCompileError(error)
    }
}

impl From<fennel_mount::Error> for ConfigInitError {
    fn from(error: fennel_mount::Error) -> Self {
        ConfigInitError::FennelMountError(error)
    }
}

impl From<fennel_searcher::Error> for ConfigInitError {
    fn from(error: fennel_searcher::Error) -> Self {
        ConfigInitError::FennelSearcherError(error)
    }
}

impl From<mlua::Error> for ConfigInitError {
    fn from(error: mlua::Error) -> Self {
        ConfigInitError::Lua(error)
    }
}

impl error::Error for ConfigInitError {}

pub type ConfigInitResult<A> = Result<A, ConfigInitError>;

pub struct Config(HashMap<String, Manifest>);

impl Config {
    pub fn new(module: Module, env: Option<Env>) -> ConfigInitResult<Self> {
        let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

        // Set up Lua environment: modify `package.path` and `package.cpath` to prevent loading
        // Lua and C modules from system paths.
        modify_paths(&lua)?;

        // Set up "standard library": enable importing Fennel, `fennel_src::loader` and `meka`.
        setup_standard_library(&lua)?;

        // If `env` exists, enable importing libraries therein.
        setup_env_library(&lua)?;

        // Set up Lua environment: add Fennel searcher to `package.loaders` to enable importing
        // local Fennel modules.
        insert_fennel_searcher(&lua)?;

        // Determine whether the config module is written in Fennel or Lua.

        // Fennel requires 1) adding macro searcher to `mlua::Lua` to enable using our Fennel
        // macros, and 2) prepending an `import-macros` line to the config module so that end
        // users don't have to.

        // Evaluate the config module and check the return value. It should be a `Manifest`
        // `mlua::Userdata` or an `mlua::Table` containing `Manifest` `mlua::Userdata`s indexed
        // by string keys.

        // Collect the `Manifest`(s) into a `HashMap`.
        let mut map: HashMap<String, Manifest> = HashMap::new();

        todo!()
    }

    /// Modify `package.path` and `package.cpath` to prevent loading Lua and C modules from
    /// system paths.
    fn modify_paths(lua: &Lua) -> ConfigInitResult<()> {
        let globals: Table = lua.globals();

        let package: Table = globals.get("package").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't get Lua package table".to_string(),
            )
        })?;

        let (package_path, package_cpath) = mlua_utils::extract_non_system_lua_paths(&lua)?;

        package.set("path", package_path).map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't set Lua package.path".to_string(),
            )
        })?;

        package.set("cpath", package_cpath).map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't set Lua package.cpath".to_string(),
            )
        })?;

        Ok(())
    }

    fn setup_standard_library(lua: &Lua) -> ConfigInitResult<()> {
        todo!()
    }

    fn setup_env_library(lua: &Lua) -> ConfigInitResult<()> {
        todo!()
    }

    /// Insert Fennel's searcher function in `package.searchers` (or `package.loaders`).
    ///
    /// Requires: Fennel library is available for import
    fn insert_fennel_searcher(lua: &Lua) -> ConfigInitResult<()> {
        let globals: Table = lua.globals();

        let require: Function = globals.get("require").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't get require function".to_string(),
            )
        })?;

        let fennel: Table = require.call("fennel").map_err(|_| {
            mlua::Error::RuntimeError("meka-config new function couldn't import Fennel".to_string())
        })?;

        let fennel_make_searcher: Function = fennel.get("make-searcher").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't get fennel.make-searcher function".to_string(),
            )
        })?;

        let fennel_searcher: Function = fennel_make_searcher.call().map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function called fennel.make-searcher and got error".to_string(),
            )
        })?;

        let package: Table = globals.get("package").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't get Lua package table".to_string(),
            )
        })?;

        let package_loaders: Table = if package.contains_key("loaders") {
            package.get("loaders").expect(TABLE_GET_EXPECT)
        } else if package.contains_key("searchers") {
            package.get("searchers").expect(TABLE_GET_EXPECT)
        } else {
            return Err(mlua::Error::RuntimeError("meka-config new function couldn't find either Lua package.loaders or package.searchers table".to_string())?);
        };

        let package_loaders_len = package_loaders.len().map_err(|_| {
            mlua::Error::RuntimeError("meka-config new function couldn't get length of Lua package.loaders (or package.searchers) table".to_string())
        })?;

        package_loaders
            .set(package_loaders_len + 1, fennel_searcher)
            .map_err(|_| {
                mlua::Error::RuntimeError("meka-config new function couldn't append Fennel searcher to package.loaders (or package.searchers) table".to_string())
            })?;

        Ok(())
    }
}

/// `Env` is a `HashMap` of Lua loader functions indexed by name.
///
/// Each Lua loader function must return an `mlua::Function` which, when called, returns an
/// `mlua::Table` with a `__call` metamethod defined. Calling said `mlua::Table` must return
/// an `mlua_module_manifest::Manifest`. The idea is to enable Rust crates to export complete
/// Lua modules. We map those exported Lua modules to names which can be `require`d within a
/// Meka config.
pub type Env = HashMap<&'static str, fn(&Lua, Table, &str) -> mlua::Result<Function>>;
