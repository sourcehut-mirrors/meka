use mlua::{IntoLua, Lua, Value};
use std::convert::From;
use std::error;
use std::fmt;
use std::vec::Vec;

use crate::mir_arg::Dict;
use crate::module_error::ModuleInitError;

#[derive(Debug)]
pub enum MirError {
    Input { errors: Vec<MirErrorKind> },
    ModuleInitError(ModuleInitError),
}

impl fmt::Display for MirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            MirError::Input { errors } => format!(
                "`Manifest` instantiation function got malformed input: {:#?}",
                errors
            ),
            MirError::ModuleInitError(e) => format!(
                "`Manifest` instantiation function got malformed module: {:#?}",
                e
            ),
        };
        write!(f, "{}", res)
    }
}

impl IntoLua for MirError {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        match self {
            MirError::Input { errors } => {
                let mut errors: Vec<String> = errors
                    .into_iter()
                    .map(|error| match error {
                        MirErrorKind::String { count, error } => {
                            format!("{} at position {}", error, count)
                        }
                        MirErrorKind::Dict { count, error } => {
                            format!("{} at position {}", error, count)
                        }
                        MirErrorKind::UserData { count, error } => {
                            format!("{} at position {}", error, count)
                        }
                        MirErrorKind::Missing { error } => format!("{}", error),
                        MirErrorKind::Unsupported { count, got } => {
                            format!("Got unsupported input type ({}) at position {}", got, count)
                        }
                    })
                    .collect();
                // Anticipate mlua's automatic insertion of "runtime error:".
                errors.insert(0, "Manifest could not be instantiated".to_string());
                let errors = errors.join("\n\n");
                Ok(Value::String(lua.create_string(&errors)?))
            }
            MirError::ModuleInitError(e) => {
                Ok(Value::String(lua.create_string(format!("{:#?}", e))?))
            }
        }
    }
}

impl From<ModuleInitError> for MirError {
    fn from(error: ModuleInitError) -> Self {
        MirError::ModuleInitError(error)
    }
}

impl error::Error for MirError {}

#[derive(Debug)]
pub enum MirErrorKind {
    /// `Manifest` instantiation function got erroneous string.
    String {
        count: usize,
        error: StringErrorKind,
    },

    /// `Manifest` instantiation function got erroneous table.
    Dict { count: usize, error: DictErrorKind },

    /// `Manifest` instantiation function got erroneous userdata.
    UserData {
        count: usize,
        error: UserDataErrorKind,
    },

    /// `Manifest` instantiation input lacks required data.
    Missing { error: MissingError },

    /// `Manifest` instantiation function got unsupported input type.
    Unsupported { count: usize, got: &'static str },
}

impl fmt::Display for MirErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            MirErrorKind::String { error, .. } => format!("{}", error),
            MirErrorKind::Dict { error, .. } => format!("{}", error),
            MirErrorKind::UserData { error, .. } => format!("{}", error),
            MirErrorKind::Missing { error } => format!("{}", error),
            MirErrorKind::Unsupported { .. } => {
                "`Manifest` instantiation function got unsupported input".to_string()
            }
        };
        write!(f, "{}", res)
    }
}

impl From<MissingError> for MirErrorKind {
    fn from(error: MissingError) -> Self {
        MirErrorKind::Missing { error }
    }
}

impl error::Error for MirErrorKind {}

#[derive(Debug)]
pub enum MissingError {
    /// `Manifest` instantiation function did not receive table with mandatory `path` field
    /// or userdata (`Manifest`).
    TablePathOrUserData,
}

impl fmt::Display for MissingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            MissingError::TablePathOrUserData => {
                "`Manifest` instantiation function did not receive table with mandatory `path` field or userdata (`Manifest`)".to_string()
            }
        };
        write!(f, "{}", res)
    }
}

impl error::Error for MissingError {}

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

#[derive(Debug)]
pub enum StringErrorKind {
    /// `Manifest` instantiation input string contents couldn't be converted to UTF-8.
    Malformed(InputStringError),

    /// `Manifest` instantiation function got valid string input in an unexpected position.
    Unexpected(String),

    /// `Manifest` instantiation function got invalid string input in an unexpected position.
    UnexpectedMalformed(InputStringError),
}

impl fmt::Display for StringErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            StringErrorKind::Malformed(error) => format!(
                "`Manifest` instantiation input string contents couldn't be converted to UTF-8: {}",
                error
            ),
            StringErrorKind::Unexpected(string) => format!(
                "`Manifest` instantiation function got valid string input in an unexpected position: {}",
                string
            ),
            StringErrorKind::UnexpectedMalformed(error) => format!(
                "`Manifest` instantiation function got invalid string input in an unexpected position: {}",
                error
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for StringErrorKind {}

#[derive(Debug)]
pub enum DictErrorKind {
    /// `Manifest` instantiation function got table input in an unexpected position.
    Unexpected,

    /// `Manifest` instantiation function got malformed table input.
    Malformed(Vec<DictKeyPairError>),

    /// `Manifest` instantiation function got malformed table input in an unexpected position.
    UnexpectedMalformed(Vec<DictKeyPairError>),
}

impl fmt::Display for DictErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictErrorKind::Unexpected => {
                "`Manifest` instantiation function got table input in an unexpected position"
                    .to_string()
            }
            DictErrorKind::Malformed(dict_error) => format!(
                "`Manifest` instantiation function got malformed table input: {:#?}",
                dict_error
            ),
            DictErrorKind::UnexpectedMalformed(dict_error) => format!(
                "`Manifest` instantiation function got malformed table input in an unexpected position: {:#?}",
                dict_error
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for DictErrorKind {}

#[derive(Debug)]
pub enum DictError {
    /// Got invalid `Manifest` instantiation input table.
    Malformed {
        dict: Dict,
        unsupported: Vec<DictKeyPairError>,
    },
}

impl fmt::Display for DictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictError::Malformed { unsupported, .. } => format!(
                "Got invalid `Manifest` instantiation input table: {:#?}",
                unsupported
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for DictError {}

#[derive(Debug)]
pub enum DictKeyPairError {
    MissingRequiredKey,
    MissingRequiredNameKey,
    MissingRequiredTypeKey,
    MutuallyExclusiveKeys,

    /// Expected `Manifest` instantiation input table key to be an `mlua::String`, but got `got`.
    MissingKeyString {
        got: &'static str,
    },

    /// `Manifest` instantiation input table string key couldn't be converted to UTF-8.
    MalformedKeyString {
        key: Vec<u8>,
    },

    /// Got unsupported `Manifest` instantiation input table string key.
    UnexpectedKeyString {
        key: String,
        value: &'static str,
    },

    Name(DictNameError),
    Path(DictPathError),
    Text(DictTextError),
    Type(DictTypeError),
}

impl fmt::Display for DictKeyPairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictKeyPairError::MissingRequiredKey => {
                "Expected `Manifest` instantiation input table to contain either `path` or `text` key, but neither was found".to_string()
            },
            DictKeyPairError::MissingRequiredNameKey => {
                "Expected `Manifest` instantiation input table containing `text` key to contain `name` key, but `name` key was not found".to_string()
            },
            DictKeyPairError::MissingRequiredTypeKey => {
                "Expected `Manifest` instantiation input table containing `text` key to contain `type` key, but `type` key was not found".to_string()
            },
            DictKeyPairError::MutuallyExclusiveKeys => {
                "Expected `Manifest` instantiation input table to contain either `path` or `text` key, but both keys were found".to_string()
            },
            DictKeyPairError::MissingKeyString { got } => format!(
                "Expected `Manifest` instantiation input table key to be an `mlua::String`, but got `{}`",
                got
            ),
            DictKeyPairError::MalformedKeyString { key } => format!(
                "`Manifest` instantiation input table string key couldn't be converted to UTF-8: {:?}",
                key
            ),
            DictKeyPairError::UnexpectedKeyString { key, value } => format!(
                "Got unsupported `Manifest` instantiation input table string key ({}) with a {} value",
                key, value
            ),
            DictKeyPairError::Name(dict_name_error) => format!("{}", dict_name_error),
            DictKeyPairError::Path(dict_name_error) => format!("{}", dict_name_error),
            DictKeyPairError::Text(dict_name_error) => format!("{}", dict_name_error),
            DictKeyPairError::Type(dict_name_error) => format!("{}", dict_name_error),
        };
        write!(f, "{}", res)
    }
}

impl From<DictNameError> for DictKeyPairError {
    fn from(error: DictNameError) -> Self {
        DictKeyPairError::Name(error)
    }
}

impl From<DictPathError> for DictKeyPairError {
    fn from(error: DictPathError) -> Self {
        DictKeyPairError::Path(error)
    }
}

impl From<DictTextError> for DictKeyPairError {
    fn from(error: DictTextError) -> Self {
        DictKeyPairError::Text(error)
    }
}

impl From<DictTypeError> for DictKeyPairError {
    fn from(error: DictTypeError) -> Self {
        DictKeyPairError::Type(error)
    }
}

#[derive(Debug)]
pub enum DictNameError {
    /// `Manifest` instantiation input table `name` string couldn't be converted to UTF-8.
    MalformedString { name: Vec<u8> },

    /// Expected `Manifest` instantiation input table `name` value to be a string, but got `got`.
    MissingString { got: &'static str },
}

impl fmt::Display for DictNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictNameError::MalformedString { name } => format!(
                "`Manifest` instantiation input table `name` string couldn't be converted to UTF-8: {:?}",
                name
            ),
            DictNameError::MissingString { got } => format!(
                "Expected `Manifest` instantiation input table `name` value to be a string, but got `{}`",
                got
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for DictNameError {}

#[derive(Debug)]
pub enum DictPathError {
    /// `Manifest` instantiation input table `path` string couldn't be converted to UTF-8.
    MalformedString { path: Vec<u8> },

    /// Expected `Manifest` instantiation input table `path` value to be a string, but got `got`.
    MissingString { got: &'static str },
}

impl fmt::Display for DictPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictPathError::MalformedString { path } => format!(
                "`Manifest` instantiation input table `path` string couldn't be converted to UTF-8: {:?}",
                path
            ),
            DictPathError::MissingString { got } => format!(
                "Expected `Manifest` instantiation input table `path` value to be a string, but got `{}`",
                got
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for DictPathError {}

#[derive(Debug)]
pub enum DictTextError {
    /// `Manifest` instantiation input table `text` string couldn't be converted to UTF-8.
    MalformedString { text: Vec<u8> },

    /// Expected `Manifest` instantiation input table `text` value to be a string, but got `got`.
    MissingString { got: &'static str },
}

impl fmt::Display for DictTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictTextError::MalformedString { text } => format!(
                "`Manifest` instantiation input table `text` string couldn't be converted to UTF-8: {:?}",
                text
            ),
            DictTextError::MissingString { got } => format!(
                "Expected `Manifest` instantiation input table `text` value to be a string, but got `{}`",
                got
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for DictTextError {}

#[derive(Debug)]
pub enum DictTypeError {
    /// `Manifest` instantiation input table `text` string couldn't be converted to UTF-8.
    MalformedString { file_type: Vec<u8> },

    /// Expected `Manifest` instantiation input table `text` value to be a string, but got `got`.
    MissingString { got: &'static str },
}

impl fmt::Display for DictTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            DictTypeError::MalformedString { file_type } => format!(
                "`Manifest` instantiation input table `type` string couldn't be converted to UTF-8: {:?}",
                file_type
            ),
            DictTypeError::MissingString { got } => format!(
                "Expected `Manifest` instantiation input table `type` value to be a string, but got `{}`",
                got
            ),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for DictTypeError {}

// Error extracting `Manifest` from `mlua::UserData`.
#[derive(Debug)]
pub enum InputManifestError {
    MalformedManifestUserData,
}

impl fmt::Display for InputManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            InputManifestError::MalformedManifestUserData => {
                "Could not find `Manifest` in userdata".to_string()
            }
        };
        write!(f, "{}", res)
    }
}

impl error::Error for InputManifestError {}

#[derive(Debug)]
pub enum UserDataErrorKind {
    /// `Manifest` instantiation function got unexpected userdata type.
    Unexpected,
}

impl fmt::Display for UserDataErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            UserDataErrorKind::Unexpected => {
                "`Manifest` instantiation function got unexpected userdata type".to_string()
            }
        };
        write!(f, "{}", res)
    }
}

impl error::Error for UserDataErrorKind {}
