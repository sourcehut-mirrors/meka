use mlua::{Lua, Table, Value};
use std::error;
use std::fmt;
use std::vec::Vec;

/// Error message designed for running `path.chars().nth(n)` on `path` verified to contain
/// at least n+1 chars.
const PATH_CHARS_NTH_EXPECT: &str = "Unexpectedly couldn't get nth char from pre-checked path";

/// Error message for `mlua::Table::contains_key(1).expect()` - which should always succeed.
const TABLE_CONTAINS_KEY_1_EXPECT: &str = "`mlua::Table::contains_key(1)` unexpectedly failed";

// Error extracting `String` from an `mlua::String` or `mlua::Value`.
#[derive(Debug)]
pub enum InputStringError {
    MalformedString { content: Vec<u8> },
    MissingString { got: &'static str },
}

impl fmt::Display for InputStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            InputStringError::MalformedString { content } => {
                format!("Malformed input string: {:?}", content)
            }
            InputStringError::MissingString { got } => {
                format!("Expected string, but got {}", got)
            }
        };
        write!(f, "{}", res)
    }
}

impl error::Error for InputStringError {}

pub type InputStringResult<A> = Result<A, InputStringError>;

pub trait IntoCharArray {
    /// Convert `mlua::String` into `Vec<u8>`.
    fn into_char_array(&self) -> Vec<u8>;
}

impl IntoCharArray for mlua::String {
    fn into_char_array(&self) -> Vec<u8> {
        self.as_bytes().iter().cloned().collect()
    }
}

pub trait TryIntoString {
    /// Bespoke replacement for `TryInto<String>` for `mlua::{String, Value}`.
    fn try_into_string(self) -> InputStringResult<String>;
}

impl TryIntoString for Value {
    fn try_into_string(self) -> InputStringResult<String> {
        match self {
            Value::String(content) => content.try_into_string(),
            value => {
                let got = typename(&value);
                Err(InputStringError::MissingString { got })
            }
        }
    }
}

impl TryIntoString for mlua::String {
    fn try_into_string(self) -> InputStringResult<String> {
        match self.to_str() {
            Ok(content) => Ok(content.to_string()),
            Err(_) => Err(InputStringError::MalformedString {
                content: self.into_char_array(),
            }),
        }
    }
}

pub trait IsList {
    /// Ascertain whether the given `mlua::Table` is a list.
    fn is_list(&self) -> bool;
}

impl IsList for Table {
    fn is_list(&self) -> bool {
        // Presume `mlua::Table` contains exclusively numeric keys if the value of key 1
        // is non-nil.
        self.contains_key(1).expect(TABLE_CONTAINS_KEY_1_EXPECT)
    }
}

/// Checks if a Lua path template string looks like an absolute path. Performs a simplified
/// check because we can't use `std::path`.
fn is_absolute_path(path: &str) -> bool {
    // Trim leading whitespace just in case.
    let path = path.trim_start();

    if path.is_empty() {
        return false;
    }

    // POSIX
    if path.starts_with('/') {
        return true;
    }

    // Windows
    if path.starts_with('\\') {
        return true;
    }

    // Windows: does it start with a drive letter like "C:"? Check if the second character is
    // ':' and the first is an alphabet char.
    if path.len() >= 3 && &path[1..2] == ":" {
        if let Some(drive_letter) = path.chars().next() {
            if drive_letter.is_ascii_alphabetic() {
                // After finding e.g. "C:", check if third char is a separator
                // Safe due to path.len() >= 3 check
                let third_char = path.chars().nth(2).expect(PATH_CHARS_NTH_EXPECT);
                if third_char == '\\' || third_char == '/' {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod is_absolute_path_tests {
    #[test]
    fn is_absolute_path_works() {
        // POSIX
        assert!(super::is_absolute_path("/usr/share/lua/5.4/?.lua"));
        assert!(super::is_absolute_path("/usr/share/lua/5.4/?/init.lua"));
        assert!(!super::is_absolute_path("./?.lua"));
        assert!(!super::is_absolute_path("./?.lua"));
        assert!(!super::is_absolute_path("?.lua"));
        assert!(!super::is_absolute_path("?/init.lua"));
        assert!(!super::is_absolute_path("some/relative/path.lua"));

        // Windows
        assert!(super::is_absolute_path("\\\\server\\share\\?.lua"));
        assert!(super::is_absolute_path("\\relative\\to\\root.lua"));
        assert!(super::is_absolute_path("C:\\Users\\Alice\\?.lua"));
        // lowercase drive letter
        assert!(super::is_absolute_path("d:/?/init.lua"));
        // drive letter without separator isn't absolute
        assert!(!super::is_absolute_path("C:no_slash.lua"));
        // must start with an alphabet char
        assert!(!super::is_absolute_path("1:\\not_a_drive.lua"));

        // Edge cases
        assert!(!super::is_absolute_path(""));
        assert!(!super::is_absolute_path(" "));
        assert!(super::is_absolute_path("  /with/leading/space.lua"));
        assert!(super::is_absolute_path("  \tC:\\with\\leading\\space.lua"));
    }
}

fn prune_system_paths(paths: String, path_sep: String) -> String {
    paths
        .split(&path_sep)
        .filter(|s| !s.is_empty())
        .filter(|&s| !is_absolute_path(s))
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(&path_sep)
}

#[cfg(test)]
mod prune_system_paths_tests {
    #[test]
    fn prune_system_paths_works() {
        let package_path = "/usr/share/lua/5.4/?.lua;/usr/share/lua/5.4/?/init.lua;/usr/lib/lua/5.4/?.lua;/usr/lib/lua/5.4/?/init.lua;./?.lua;./?/init.lua".to_string();
        let path_sep = ";".to_string();
        assert_eq!(
            super::prune_system_paths(package_path, path_sep.clone()),
            "./?.lua;./?/init.lua"
        );
        let package_cpath = "/usr/lib/lua/5.4/?.so;/usr/lib/lua/5.4/loadall.so;./?.so".to_string();
        assert_eq!(super::prune_system_paths(package_cpath, path_sep), "./?.so");
    }
}

/// Returns 2-tuple of Lua's `package.path`, `package.cpath` stripped of system path components.
pub fn extract_non_system_lua_paths(lua: &Lua) -> mlua::Result<(String, String)> {
    let (_, path_sep, _, _, _) = package_config(lua)?;
    let package_path = package_path(lua)?;
    let package_path = prune_system_paths(package_path, path_sep.clone());
    let package_cpath = package_cpath(lua)?;
    let package_cpath = prune_system_paths(package_cpath, path_sep);
    Ok((package_path, package_cpath))
}

/// Return 5-tuple containing components of Lua's `package.config`.
pub fn package_config(lua: &Lua) -> mlua::Result<(String, String, String, String, String)> {
    let globals: Table = lua.globals();
    let package: Table = globals.get("package").map_err(|_| {
        mlua::Error::RuntimeError(
            "mlua-utils package_config function couldn't get Lua package table".to_string(),
        )
    })?;
    let package_config: Value = package.get("config").map_err(|_| {
        mlua::Error::RuntimeError(
            "mlua-utils package_config function couldn't get Lua package.config value".to_string(),
        )
    })?;
    let package_config: String = match package_config {
        Value::String(package_config) => package_config.to_str().map_err(|_| mlua::Error::RuntimeError("mlua-utils package_config function couldn't convert mlua::String to Rust string".to_string()))?.to_owned(),
        val => return Err(mlua::Error::RuntimeError(format!("mlua-utils package_config function expected string value for Lua's package.config, but got {}", typename(&val)))),
    };
    let mut package_config = package_config.split_whitespace();
    let package_config = (
        package_config.next().map(String::from),
        package_config.next().map(String::from),
        package_config.next().map(String::from),
        package_config.next().map(String::from),
        package_config.next().map(String::from),
    );
    if let (Some(dir_sep), Some(path_sep), Some(path_mark), Some(exedir_mark), Some(ignore_mark)) =
        package_config
    {
        Ok((dir_sep, path_sep, path_mark, exedir_mark, ignore_mark))
    } else {
        Err(mlua::Error::RuntimeError(
            "mlua-utils package_config function found malformed package.config value".to_string(),
        ))
    }
}

/// Return Lua's `package.cpath` as `String`.
pub fn package_cpath(lua: &Lua) -> mlua::Result<String> {
    let globals: Table = lua.globals();
    let package: Table = globals.get("package").map_err(|_| {
        mlua::Error::RuntimeError(
            "mlua-utils package_cpath function couldn't get Lua package table".to_string(),
        )
    })?;
    let package_cpath: Value = package.get("cpath").map_err(|_| {
        mlua::Error::RuntimeError(
            "mlua-utils package_cpath function couldn't get Lua package.cpath value".to_string(),
        )
    })?;
    let package_cpath: String = match package_cpath {
        Value::String(package_cpath) => package_cpath
            .to_str()
            .map_err(|_| {
                mlua::Error::RuntimeError(
                    "mlua-utils package_cpath function couldn't convert mlua::String to Rust string"
                        .to_string(),
                )
            })?
            .to_owned(),
        val => {
            return Err(mlua::Error::RuntimeError(format!(
                "mlua-utils package_cpath function expected string value for Lua's package.cpath, but got {}",
                typename(&val)
            )));
        }
    };
    Ok(package_cpath)
}

/// Return Lua's `package.path` as `String`.
pub fn package_path(lua: &Lua) -> mlua::Result<String> {
    let globals: Table = lua.globals();
    let package: Table = globals.get("package").map_err(|_| {
        mlua::Error::RuntimeError(
            "mlua-utils package_path function couldn't get Lua package table".to_string(),
        )
    })?;
    let package_path: Value = package.get("path").map_err(|_| {
        mlua::Error::RuntimeError(
            "mlua-utils package_path function couldn't get Lua package.path value".to_string(),
        )
    })?;
    let package_path: String = match package_path {
        Value::String(package_path) => package_path
            .to_str()
            .map_err(|_| {
                mlua::Error::RuntimeError(
                    "mlua-utils package_path function couldn't convert mlua::String to Rust string"
                        .to_string(),
                )
            })?
            .to_owned(),
        val => {
            return Err(mlua::Error::RuntimeError(format!(
                "mlua-utils package_path function expected string value for Lua's package.path, but got {}",
                typename(&val)
            )));
        }
    };
    Ok(package_path)
}

/// Convert an `mlua::Value` into type `String`.
pub fn typename(val: &Value) -> &'static str {
    match val {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "light userdata",
        Value::Integer(_) => "integer",
        Value::Number(_) => "number",
        #[cfg(any(
            feature = "mlua-luau",
            feature = "mlua-luau-jit",
            feature = "mlua-luau-vector4"
        ))]
        Value::Vector(_) => "vector",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::UserData(_) => "userdata",
        #[cfg(any(
            feature = "mlua-luau",
            feature = "mlua-luau-jit",
            feature = "mlua-luau-vector4"
        ))]
        Value::Buffer(_) => "buffer",
        Value::Error(_) => "error",
        Value::Other(_) => "other",
    }
}
