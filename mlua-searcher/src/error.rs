#[derive(Debug)]
pub enum Error {
    Lua(mlua::Error),
}

impl From<mlua::Error> for Error {
    fn from(error: mlua::Error) -> Self {
        Error::Lua(error)
    }
}

impl From<Error> for mlua::Error {
    fn from(error: Error) -> Self {
        mlua::Error::RuntimeError(format!("mlua-searcher error: {}", error))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Error::Lua(e) => format!("mlua error:\n{:#?}", e),
        };
        write!(f, "{}", res)
    }
}

impl std::error::Error for Error {}
