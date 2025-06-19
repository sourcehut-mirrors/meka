use mlua::{Lua, Value};
use std::path::MAIN_SEPARATOR;

#[test]
fn try_into_string_works() {
    use mlua_utils::TryIntoString;

    let lua = Lua::new();

    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    assert!(val.try_into_string().is_ok());
}

#[test]
fn extract_non_system_lua_paths_works() {
    let lua = Lua::new();
    assert!(mlua_utils::extract_non_system_lua_paths(&lua).is_ok());
}

#[test]
fn package_config_works() {
    let lua = Lua::new();
    assert!(mlua_utils::package_config(&lua).is_ok());
    let (dir_sep, path_sep, path_mark, exedir_mark, ignore_mark) =
        mlua_utils::package_config(&lua).unwrap();
    assert_eq!(dir_sep, MAIN_SEPARATOR.to_string());
    assert_eq!(path_sep, ";");
    assert_eq!(path_mark, "?");
    assert_eq!(exedir_mark, "!");
    assert_eq!(ignore_mark, "-");
}

#[test]
fn package_cpath_works() {
    let lua = Lua::new();
    assert!(mlua_utils::package_cpath(&lua).is_ok());
}

#[test]
fn package_path_works() {
    let lua = Lua::new();
    assert!(mlua_utils::package_path(&lua).is_ok());
}

#[test]
fn typename_works() {
    let lua = Lua::new();

    let val: Value = lua.load("return nil").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "nil");

    let val: Value = lua.load("return true").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "boolean");

    let val: Value = lua.load("return 1").eval().unwrap();
    #[cfg(any(feature = "mlua-lua53", feature = "mlua-lua54"))]
    assert_eq!(mlua_utils::typename(&val), "integer");
    #[cfg(not(any(feature = "mlua-lua53", feature = "mlua-lua54")))]
    assert_eq!(mlua_utils::typename(&val), "number");

    let val: Value = lua.load("return 1.0").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "number");

    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "string");

    let val: Value = lua.load("return {}").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "table");

    let val: Value = lua.load("return function() end").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "function");
}
