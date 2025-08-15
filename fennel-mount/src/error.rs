use savefile_derive::Savefile;
use std::error;
use std::fmt;

#[derive(Debug, Savefile)]
pub enum Error {
    Lua(mlua::Error),
    LuaSearcher(mlua_searcher::Error),
}

impl From<mlua::Error> for Error {
    fn from(error: mlua::Error) -> Self {
        Error::Lua(error)
    }
}

impl From<Error> for mlua::Error {
    fn from(error: Error) -> Self {
        mlua::Error::RuntimeError(format!("fennel-mount error: {}", error))
    }
}

impl From<mlua_searcher::Error> for Error {
    fn from(error: mlua_searcher::Error) -> Self {
        Error::LuaSearcher(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Error::Lua(e) => format!("mlua error: {:?}", e),
            Error::LuaSearcher(e) => format!("mlua-searcher error: {:?}", e),
        };
        write!(f, "{}", res)
    }
}

impl error::Error for Error {}
