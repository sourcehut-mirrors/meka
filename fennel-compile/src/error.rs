use std::error;
use std::fmt;
use std::io;
use std::str;

#[derive(Debug)]
pub enum Error {
    /// Could not import Fennel by module name "fennel".
    FailedToImportFennel(mlua::Error),
    /// Could not find `fennel.compileString` function.
    MissingFennelCompileStringFunction,

    Io(io::Error),
    Lua(mlua::Error),
    Str(str::Utf8Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(error: str::Utf8Error) -> Self {
        Error::Str(error)
    }
}

impl From<mlua::Error> for Error {
    fn from(error: mlua::Error) -> Self {
        Error::Lua(error)
    }
}

impl From<Error> for mlua::Error {
    fn from(error: Error) -> Self {
        mlua::Error::RuntimeError(format!("fennel-compile error: {}", error))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Error::FailedToImportFennel(e) => {
                format!("Could not import Fennel by module name \"fennel\": {:?}", e)
            }
            Error::MissingFennelCompileStringFunction => {
                "Could not find fennel.compileString function".to_string()
            }

            Error::Io(e) => format!("IO error: {:?}", e),
            Error::Lua(e) => format!("mlua error: {:?}", e),
            Error::Str(e) => format!("UTF-8 error: {:?}", e),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for Error {}
