use mlua::{Function, Lua, LuaOptions, StdLib, Table};
use mlua_module_manifest::{Manifest, Module};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::From;
use std::error;
use std::fmt;
use std::path::Path;
use std::result::Result;

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

/// Modify `package.path` and `package.cpath` to prevent loading Lua and C modules from system
/// paths.
fn config_new_modify_paths(lua: &Lua) -> ConfigInitResult<()> {
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

impl Config {
    pub fn new(module: Module, env: Option<Env>) -> ConfigInitResult<Self> {
        let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

        // Set up Lua environment: modify `package.path` and `package.cpath` to prevent loading
        // Lua and C modules from system paths.
        config_new_modify_paths(&lua)?;

        // Set up Lua environment: add Fennel searcher to `package.loaders` to enable importing
        // local Fennel modules.

        // Set up "standard library": enable importing `fennel_src::loader` and `meka`.

        // If `env` exists, enable importing libraries therein.

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
}

/// `Env` is a `HashMap` of Lua loader functions indexed by name.
///
/// Each Lua loader function must return an `mlua::Function` which, when called, returns an
/// `mlua::Table` with a `__call` metamethod defined. Calling said `mlua::Table` must return
/// an `mlua_module_manifest::Manifest`. The idea is to enable Rust crates to export complete
/// Lua modules. We map those exported Lua modules to names which can be `require`d within a
/// Meka config.
pub type Env = HashMap<&'static str, fn(&Lua, Table, &str) -> mlua::Result<Function>>;
