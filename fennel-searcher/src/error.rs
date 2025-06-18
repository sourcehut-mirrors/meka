#[derive(Debug)]
pub enum Error {
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
            Error::FennelCompile(e) => format!("fennel-compile error:\n{:#?}", e),
            Error::Lua(e) => format!("mlua error:\n{:#?}", e),
            Error::LuaSearcher(e) => format!("mlua-searcher error:\n{:#?}", e),
        };
        write!(f, "{}", res)
    }
}

impl std::error::Error for Error {}
