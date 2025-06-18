use mlua::{Lua, Table, Value};
use std::error;
use std::fmt;
use std::vec::Vec;

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
