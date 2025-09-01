use fennel_compile::Compile;
use fennel_mount::Mount;
use fennel_searcher::AddSearcher as _;
use fennel_utils::InsertFennelSearcher;
use meka_config::evaluator_types::{ConfigEvaluatorInput, ConfigEvaluatorOutput};
use meka_loader::{LoaderFn, LoaderRegistry};
use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value};
use mlua_module_manifest::{Manifest, ModuleFileType};
use mlua_searcher::AddSearcher as _;
use mlua_utils::{IntoCharArray, IsList};
use savefile::{CURRENT_SAVEFILE_LIB_VERSION, load_from_mem, save_to_mem};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;
use std::io::{Read, Write};
use std::vec::Vec;

const IO_STDIN_READ_TO_END_EXPECT: &str = "Failed to read from stdin";
const IO_STDOUT_WRITEALL_EXPECT: &str = "Failed to write result";
const SAVEFILE_LOAD_FROM_MEM_EXPECT: &str = "Failed to deserialize input";
const SAVEFILE_SAVE_TO_MEM_EXPECT: &str = "Failed to serialize result";

#[cfg(host_family = "windows")]
macro_rules! path_separator {
    () => {
        r"\"
    };
}

#[cfg(not(host_family = "windows"))]
macro_rules! path_separator {
    () => {
        r"/"
    };
}

/// Fennel macros to aid in writing `manifest.fnl` files.
const MEKA_MACROS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    path_separator!(),
    "..",
    path_separator!(),
    "meka-config",
    path_separator!(),
    "meka",
    path_separator!(),
    "macros.fnl"
));

fn main() {
    // Read serialized input from stdin.
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .expect(IO_STDIN_READ_TO_END_EXPECT);

    // Deserialize input.
    let input: ConfigEvaluatorInput =
        load_from_mem(&buffer, CURRENT_SAVEFILE_LIB_VERSION.into())
            .expect(SAVEFILE_LOAD_FROM_MEM_EXPECT);

    // Evaluate config and get result.
    let result = evaluate_config(input);

    // Serialize result.
    let serialized = save_to_mem(CURRENT_SAVEFILE_LIB_VERSION.into(), &result)
        .expect(SAVEFILE_SAVE_TO_MEM_EXPECT);

    // Write serialized result to stdout.
    io::stdout()
        .write_all(&serialized)
        .expect(IO_STDOUT_WRITEALL_EXPECT);
}

fn evaluate_config(input: ConfigEvaluatorInput) -> ConfigEvaluatorOutput {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    // Set up environment
    if let Err(e) = setup_lua_environment(&lua, input.loader_paths) {
        return ConfigEvaluatorOutput::Err(format!("Failed to setup environment: {}", e));
    }

    // Parse file type
    let file_type = match input.module.file_type {
        ModuleFileType::Fennel => {},
        ModuleFileType::Lua => {},
        _ => return ConfigEvaluatorOutput::Err(format!("Invalid file type: {}", input.file_type)),
    };

    // Get config as Lua string
    let config_str = match get_config_as_lua_string(&lua, input.module_text, file_type) {
        Ok(s) => s,
        Err(e) => return ConfigEvaluatorOutput::Err(format!("Failed to compile config: {}", e)),
    };

    // Evaluate and extract manifests
    match evaluate_and_extract_manifests(&lua, &config_str) {
        Ok(manifests) => ConfigEvaluatorOutput::Ok(manifests),
        Err(e) => ConfigEvaluatorOutput::Err(e),
    }
}

fn setup_lua_environment(lua: &Lua, loader_paths: Vec<(String, String)>) -> mlua::Result<()> {
    // Modify package paths
    modify_paths(lua)?;

    // Setup standard library
    setup_standard_library(lua)?;

    // Setup user library from loader paths
    setup_user_library(lua, loader_paths)?;

    // Insert Fennel searcher
    lua.insert_fennel_searcher()
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to insert Fennel searcher: {}", e)))?;

    Ok(())
}

fn modify_paths(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let package: Table = globals.get("package")?;

    let (package_path, package_cpath) = mlua_utils::extract_non_system_lua_paths(lua)?;

    package.set("path", package_path)?;
    package.set("cpath", package_cpath)?;

    Ok(())
}

fn setup_standard_library(lua: &Lua) -> mlua::Result<()> {
    let mut searcher: HashMap<Cow<'static, str>, LoaderFn> = HashMap::with_capacity(2);

    // Enable importing Fennel
    lua.mount_fennel()?;

    // Enable importing fennel-src
    searcher.insert(Cow::from("fennel-src"), fennel_src::loader);

    // Enable importing meka
    searcher.insert(Cow::from("meka"), meka_loader::loader);

    lua.add_function_searcher(searcher)?;

    Ok(())
}

fn setup_user_library(lua: &Lua, loader_paths: Vec<(String, String)>) -> mlua::Result<()> {
    match meka_module_registry::build_loader_registry(loader_paths) {
        Ok(registry) => {
            lua.add_function_searcher(registry)?;
            Ok(())
        }
        Err(unknown) => {
            // Log but continue - partial success
            eprintln!("Warning: Unknown loader paths: {:?}", unknown);
            Ok(())
        }
    }
}

fn get_config_as_lua_string(
    lua: &Lua,
    config_str: String,
    file_type: ModuleFileType
) -> mlua::Result<String> {
    match file_type {
        ModuleFileType::Fennel => {
            // Add macro searcher
            let mut searcher_fnl_macros = HashMap::with_capacity(1);
            searcher_fnl_macros.insert(Cow::from("meka.macros"), Cow::from(MEKA_MACROS));
            lua.add_searcher_fnl_macros(searcher_fnl_macros)?;

            // Compile Fennel to Lua
            lua.compile_fennel_string(&config_str)
        }
        ModuleFileType::Lua => Ok(config_str),
        ModuleFileType::FennelMacros => {
            Err(mlua::Error::RuntimeError("FennelMacros not supported".to_string()))
        }
    }
}

fn evaluate_and_extract_manifests(
    lua: &Lua,
    config_str: &str
) -> Result<HashMap<String, Vec<u8>>, String> {
    // Evaluate config module
    let value: Value = lua.load(config_str).eval()
        .map_err(|e| format!("Failed to evaluate config: {}", e))?;

    let mut map = HashMap::new();

    match value {
        Value::Table(table) => {
            if table.is_list() {
                return Err("Config returned a list, expected table or userdata".to_string());
            }

            for pairs in table.pairs::<Value, Value>() {
                let (key, value) = pairs.map_err(|e| format!("Failed to iterate table: {}", e))?;

                match key {
                    Value::String(key_str) => {
                        let key = key_str.to_str()
                            .map_err(|_| "Invalid string key".to_string())?
                            .to_string();

                        match value {
                            Value::UserData(ud) => {
                                let manifest = Manifest::try_from(ud)
                                    .map_err(|_| "Invalid Manifest userdata".to_string())?;

                                // Serialize the manifest
                                let serialized = save_to_mem(
                                    CURRENT_SAVEFILE_LIB_VERSION.into(),
                                    &manifest
                                ).map_err(|e| format!("Failed to serialize manifest: {}", e))?;

                                map.insert(key, serialized);
                            }
                            _ => return Err(format!("Expected Manifest userdata value")),
                        }
                    }
                    _ => return Err("Expected string key".to_string()),
                }
            }
        }
        Value::UserData(ud) => {
            let manifest = Manifest::try_from(ud)
                .map_err(|_| "Invalid Manifest userdata".to_string())?;

            let serialized = save_to_mem(
                CURRENT_SAVEFILE_LIB_VERSION.into(),
                &manifest
            ).map_err(|e| format!("Failed to serialize manifest: {}", e))?;

            // Empty string key for single manifest return
            map.insert("".to_string(), serialized);
        }
        _ => return Err("Config must return table or Manifest userdata".to_string()),
    }

    Ok(map)
}
