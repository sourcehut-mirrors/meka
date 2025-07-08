#[derive(Debug)]
pub enum Error {
    /// Could not import Fennel by module name "fennel".
    FailedToImportFennel(mlua::Error),

    FennelCompile(fennel_compile::Error),
    Lua(mlua::Error),
    LuaSearcher(mlua_searcher::Error),
}

impl From<fennel_compile::Error> for Error {
    fn from(error: fennel_compile::Error) -> Self {
        Error::FennelCompile(error)
    }
}

impl From<mlua::Error> for Error {
    fn from(error: mlua::Error) -> Self {
        Error::Lua(error)
    }
}

impl From<mlua_searcher::Error> for Error {
    fn from(error: mlua_searcher::Error) -> Self {
        Error::LuaSearcher(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Error::FailedToImportFennel(e) => {
                format!("Could not import Fennel by module name \"fennel\": {:?}", e)
            }

            Error::FennelCompile(e) => format!("fennel-compile error: {:?}", e),
            Error::Lua(e) => format!("mlua error: {:?}", e),
            Error::LuaSearcher(e) => format!("mlua-searcher error: {:?}", e),
        };
        write!(f, "{}", res)
    }
}

impl std::error::Error for Error {}
