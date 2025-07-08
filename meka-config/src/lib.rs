use fennel_compile::Compile;
use fennel_mount::Mount;
use fennel_searcher::AddSearcher as _;
use meka_config_macros::loader_registry_from_cargo_manifest;
use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value};
use mlua_module_manifest::{Manifest, Module, ModuleFile, ModuleFileType, ModuleNamedText};
use mlua_searcher::AddSearcher as _;
use mlua_utils::{IntoCharArray, IsList};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::result::Result;
use std::vec::Vec;

pub mod prelude {
    pub use crate::{Config, ConfigInitError, ConfigInitResult, LoaderRegistry};
}

#[cfg(target_family = "windows")]
macro_rules! path_separator {
    () => {
        r"\"
    };
}

#[cfg(not(target_family = "windows"))]
macro_rules! path_separator {
    () => {
        r"/"
    };
}

/// Fennel macros to aid in writing `manifest.fnl` files.
const MEKA_MACROS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    path_separator!(),
    "meka",
    path_separator!(),
    "macros.fnl"
));

/// Error message for `Iterator::Item.expect()` in `mlua::TablePairs`es - which `mlua`
/// wraps in `Result` to facilitate lazily converting Lua types to Rust. Presumably this
/// can only fail if the user requests a Rust type which doesn't implement `FromLua`.
const PAIRS_EXPECT: &str = "`mlua::TablePairs::pairs()` unexpectedly failed";

/// Error message designed for running `table.get(key)` on `mlua::Table` `table` verified to
/// contain key `key`.
const TABLE_GET_EXPECT: &str = "Unexpectedly couldn't get key from pre-checked table";

#[derive(Debug)]
pub enum ConfigInitError {
    InvalidConfigModuleFileType,
    InvalidConfigModuleResult { got: &'static str },
    InvalidConfigModuleResultTableKey { got: &'static str },
    MalformedConfigModuleResultTableKeyString { content: Vec<u8> },
    InvalidConfigModuleResultTableValue { got: &'static str },
    InvalidConfigModuleResultTableValueUserData,
    InvalidConfigModuleResultUserData,

    FennelCompileError(fennel_compile::Error),
    FennelMountError(fennel_mount::Error),
    FennelSearcherError(fennel_searcher::Error),
    Io(io::Error),
    Lua(mlua::Error),
    LuaModuleManifestModuleFileInitError(mlua_module_manifest::ModuleFileInitError),
    LuaModuleManifestModuleNamedTextInitError(mlua_module_manifest::ModuleNamedTextInitError),
    LuaSearcherError(mlua_searcher::Error),
}

impl fmt::Display for ConfigInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ConfigInitError::InvalidConfigModuleFileType => "Expected Fennel or Lua config module file type, but got FennelMacros".to_string(),
            ConfigInitError::InvalidConfigModuleResult { got } => format!("Expected config module to return table or userdata, but got {}", got),
            ConfigInitError::InvalidConfigModuleResultTableKey { got } => format!("Expected config module to return table of userdata indexed by string, but found key of type {}", got),
            ConfigInitError::MalformedConfigModuleResultTableKeyString { content } => format!("Couldn't convert string key in table returned by config module from Lua to Rust: {:?}", content),
            ConfigInitError::InvalidConfigModuleResultTableValue { got } => format!("Expected config module to return table of userdata indexed by string, but found value of type {}", got),
            ConfigInitError::InvalidConfigModuleResultTableValueUserData => "Expected config module to return table of Manifest userdata indexed by string, but found unsupported userdata type".to_string(),
            ConfigInitError::InvalidConfigModuleResultUserData => "Expected config module to return Manifest userdata, but found unsupported userdata type".to_string(),

            ConfigInitError::FennelCompileError(error) => format!("{}", error),
            ConfigInitError::FennelMountError(error) => format!("{}", error),
            ConfigInitError::FennelSearcherError(error) => format!("{}", error),
            ConfigInitError::Io(error) => format!("{}", error),
            ConfigInitError::Lua(error) => format!("{}", error),
            ConfigInitError::LuaModuleManifestModuleFileInitError(error) => format!("{}", error),
            ConfigInitError::LuaModuleManifestModuleNamedTextInitError(error) => format!("{}", error),
            ConfigInitError::LuaSearcherError(error) => format!("{}", error),
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

impl From<io::Error> for ConfigInitError {
    fn from(error: io::Error) -> Self {
        ConfigInitError::Io(error)
    }
}

impl From<mlua::Error> for ConfigInitError {
    fn from(error: mlua::Error) -> Self {
        ConfigInitError::Lua(error)
    }
}

impl From<mlua_module_manifest::ModuleFileInitError> for ConfigInitError {
    fn from(error: mlua_module_manifest::ModuleFileInitError) -> Self {
        ConfigInitError::LuaModuleManifestModuleFileInitError(error)
    }
}

impl From<mlua_module_manifest::ModuleNamedTextInitError> for ConfigInitError {
    fn from(error: mlua_module_manifest::ModuleNamedTextInitError) -> Self {
        ConfigInitError::LuaModuleManifestModuleNamedTextInitError(error)
    }
}

impl From<mlua_searcher::Error> for ConfigInitError {
    fn from(error: mlua_searcher::Error) -> Self {
        ConfigInitError::LuaSearcherError(error)
    }
}

impl error::Error for ConfigInitError {}

pub type ConfigInitResult<A> = Result<A, ConfigInitError>;

pub struct Config(pub HashMap<String, Manifest>);

impl Config {
    pub fn from_path<P>(path: P, lreg: Option<LoaderRegistry>) -> ConfigInitResult<Self>
    where
        P: AsRef<Path>,
    {
        let path: &Path = path.as_ref();
        let module = ModuleFile::new(path, None)?;
        let module = Module::File(module);
        Config::new(module, lreg)
    }

    pub fn from_str<S>(
        s: S,
        file_type: ModuleFileType,
        lreg: Option<LoaderRegistry>,
    ) -> ConfigInitResult<Self>
    where
        S: AsRef<str>,
    {
        let module = ModuleNamedText::new("manifest", s.as_ref(), file_type)?;
        let module = Module::NamedText(module);
        Config::new(module, lreg)
    }

    pub fn new(module: Module, lreg: Option<LoaderRegistry>) -> ConfigInitResult<Self> {
        let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

        // Set up Lua environment: modify `package.path` and `package.cpath` to prevent loading
        // Lua and C modules from system paths.
        Self::modify_paths(&lua)?;

        // Set up "standard library": enable importing fennel, fennel-src and meka.
        Self::setup_standard_library(&lua)?;

        // Set up "user library": enable importing user-defined libraries.
        Self::setup_user_library(&lua, lreg)?;

        // Set up Lua environment: add Fennel searcher to `package.loaders` to enable importing
        // local Fennel modules.
        Self::insert_fennel_searcher(&lua)?;

        // Get config module as Lua string, converting compile-to-Lua language config module
        // to Lua as needed.
        let config_str = Self::get_config_module_as_lua_string(&lua, module)?;

        // For collecting `Manifest`(s).
        let mut map: HashMap<String, Manifest> = HashMap::new();

        // Evaluate config module and check return value. It should be `Manifest` `mlua::Userdata`
        // or an `mlua::Table` containing `Manifest` `mlua::Userdata`s indexed by string keys.
        let value: Value = lua.load(&config_str).eval().map_err(|e| {
            mlua::Error::RuntimeError(format!(
                "meka-config new function got error evaluating config module: {}",
                e
            ))
        })?;

        match value {
            Value::Table(table) => {
                if table.is_list() {
                    let got = "list";
                    return Err(ConfigInitError::InvalidConfigModuleResult { got });
                }
                for pairs in table.pairs::<Value, Value>() {
                    match pairs.expect(PAIRS_EXPECT) {
                        // Found `mlua::String` key.
                        (Value::String(key), value) => {
                            match key.to_str() {
                                Ok(key) => {
                                    match value {
                                        Value::UserData(ud) => {
                                            let manifest = Manifest::try_from(ud).map_err(|_| {
                                                ConfigInitError::InvalidConfigModuleResultTableValueUserData
                                            })?;
                                            map.insert(key.to_string(), manifest);
                                        }
                                        value => {
                                            let got = mlua_utils::typename(&value);
                                            return Err(ConfigInitError::InvalidConfigModuleResultTableValue { got });
                                        }
                                    }
                                }
                                Err(_) => {
                                    let content = key.into_char_array();
                                    return Err(ConfigInitError::MalformedConfigModuleResultTableKeyString { content });
                                }
                            }
                        }

                        // Found unsupported key.
                        (key, _) => {
                            let got = mlua_utils::typename(&key);
                            return Err(ConfigInitError::InvalidConfigModuleResultTableKey { got });
                        }
                    }
                }
            }
            Value::UserData(ud) => {
                let manifest = Manifest::try_from(ud)
                    .map_err(|_| ConfigInitError::InvalidConfigModuleResultUserData)?;
                // Empty string represents case where config module returns `Manifest` userdata.
                map.insert("".to_string(), manifest);
            }
            value => {
                let got = mlua_utils::typename(&value);
                return Err(ConfigInitError::InvalidConfigModuleResult { got });
            }
        }

        Ok(Self(map))
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
        let mut searcher: HashMap<
            Cow<'static, str>,
            fn(&Lua, Table, &str) -> mlua::Result<Function>,
        > = HashMap::with_capacity(2);

        // Enable importing Fennel at "fennel".
        lua.mount_fennel()?;

        // Enabling importing `fennel_src::loader` at "fennel-src".
        searcher.insert(Cow::from("fennel-src"), fennel_src::loader);

        // Enabling importing `meka_loader` at "meka".
        searcher.insert(Cow::from("meka"), Self::meka_loader);

        lua.add_function_searcher(searcher)?;

        Ok(())
    }

    fn meka_loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
        let globals = lua.globals();

        let tbl = lua.create_table().map_err(|_| {
            mlua::Error::RuntimeError("meka_loader function failed to create Lua table".to_string())
        })?;

        let manifest: Function = Manifest::loader(lua, env.clone(), "manifest").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka_loader function called Manifest::loader and got error".to_string(),
            )
        })?;
        let manifest: Table = manifest.call(()).map_err(|_| {
            mlua::Error::RuntimeError(
                "meka_loader function called Manifest::loader in Lua context and got error"
                    .to_string(),
            )
        })?;
        tbl.set("manifest", manifest).map_err(|_| {
            mlua::Error::RuntimeError("meka_loader function failed to set Lua table".to_string())
        })?;

        globals.set("meka", tbl).map_err(|_| {
            mlua::Error::RuntimeError("meka_loader function failed to set Lua table".to_string())
        })?;

        Ok(lua
            .load("return meka")
            .set_name(name)
            .set_environment(env)
            .into_function()?)
    }

    fn setup_user_library(lua: &Lua, lreg: Option<LoaderRegistry>) -> ConfigInitResult<()> {
        let mut loader_registry: LoaderRegistry = loader_registry_from_cargo_manifest!();
        match lreg {
            Some(lreg) if !lreg.is_empty() => loader_registry.extend(lreg),
            _ => {}
        }
        // TODO: insert library searcher before standard library searcher in `package.searchers`
        // with bespoke implementation of `lua.add_function_searcher()`.
        lua.add_function_searcher(loader_registry)?;
        Ok(())
    }

    /// Insert Fennel's searcher function in `package.searchers` (or `package.loaders`).
    ///
    /// Requires: Fennel library is available for import
    fn insert_fennel_searcher(lua: &Lua) -> ConfigInitResult<()> {
        // TODO: insert fennel searcher before user library searcher in `package.searchers`.
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

        let fennel_searcher: Function = fennel_make_searcher.call(()).map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function called fennel.make-searcher and got error".to_string(),
            )
        })?;

        let package: Table = globals.get("package").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't get Lua package table".to_string(),
            )
        })?;

        let package_loaders: Table = if package.contains_key("loaders").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't check if package table contains loaders key"
                    .to_string(),
            )
        })? {
            package.get("loaders").expect(TABLE_GET_EXPECT)
        } else if package.contains_key("searchers").map_err(|_| {
            mlua::Error::RuntimeError(
                "meka-config new function couldn't check if package table contains searchers key"
                    .to_string(),
            )
        })? {
            package.get("searchers").expect(TABLE_GET_EXPECT)
        } else {
            return Err(ConfigInitError::from(mlua::Error::RuntimeError("meka-config new function couldn't find either Lua package.loaders or package.searchers table".to_string())));
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

    fn get_config_module_as_lua_string(lua: &Lua, module: Module) -> ConfigInitResult<String> {
        // Read config module to string.
        let config_str = Self::read_config_module(module.clone())?;

        // Determine whether the config module is written in Fennel or Lua.
        let file_type = match module {
            Module::File(module_file) => module_file.file_type,
            Module::NamedFile(module_named_file) => module_named_file.file_type,
            Module::NamedText(module_named_text) => module_named_text.file_type,
        };

        let config_str = match file_type {
            ModuleFileType::Fennel => {
                // Add macro searcher to `mlua::Lua` to enable using our Fennel macros.
                let mut searcher_fnl_macros = HashMap::with_capacity(1);
                searcher_fnl_macros.insert(Cow::from("meka.macros"), Cow::from(MEKA_MACROS));
                lua.add_searcher_fnl_macros(searcher_fnl_macros)?;

                // Compile Fennel to Lua.
                lua.compile_fennel_string(&config_str)?
            }
            ModuleFileType::FennelMacros => {
                return Err(ConfigInitError::InvalidConfigModuleFileType);
            }
            ModuleFileType::Lua => config_str,
        };

        Ok(config_str)
    }

    fn read_config_module(module: Module) -> ConfigInitResult<String> {
        let text: String = match module {
            Module::File(module_file) => Self::read_config_module_from_path(&module_file.path)?,
            Module::NamedFile(module_named_file) => {
                Self::read_config_module_from_path(&module_named_file.path)?
            }
            Module::NamedText(module_named_text) => module_named_text.text.into_owned(),
        };
        Ok(text)
    }

    fn read_config_module_from_path(path: &Path) -> ConfigInitResult<String> {
        let mut config_str = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut config_str)?;
        Ok(config_str)
    }
}

/// `LoaderRegistry` is a `HashMap` of Lua loader functions indexed by name.
///
/// Each Lua loader function must return an `mlua::Function` which, when called, returns an
/// `mlua::Table` with a `__call` metamethod defined. Calling said `mlua::Table` must return
/// an `mlua_module_manifest::Manifest`. The idea is to enable Rust crates to export complete
/// Lua modules. We map those exported Lua modules to names which can be `require`d within a
/// Meka config.
pub type LoaderRegistry =
    HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>>;
