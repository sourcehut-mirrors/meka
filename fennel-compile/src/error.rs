#[derive(Debug)]
pub enum Error {
    /// Could not find `fennel.compileString` function.
    MissingFennelCompileStringFunction,

    Io(std::io::Error),
    Lua(mlua::Error),
    Str(std::str::Utf8Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Error::MissingFennelCompileStringFunction => {
                "Could not find fennel.compileString function".to_string()
            }

            Error::Io(e) => format!("IO error:\n{:#?}", e),
            Error::Lua(e) => format!("mlua error:\n{:#?}", e),
            Error::Str(e) => format!("UTF-8 error:\n{:#?}", e),
        };
        write!(f, "{}", res)
    }
}

impl std::error::Error for Error {}
