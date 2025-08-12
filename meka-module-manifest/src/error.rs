use mlua_module_manifest::NamedTextManifestInitError;
use std::convert::From;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum CompiledNamedTextManifestInitError {
    NamedTextManifestInitError(NamedTextManifestInitError),
    FennelCompileError(fennel_compile::Error),
    FennelMountError(fennel_mount::Error),
    FennelSearcherError(fennel_searcher::Error),
    #[cfg(feature = "mlua-module")]
    LuaLibraryLoadError(String),
    #[cfg(feature = "mlua-module")]
    NoLuaVersionSpecified,
    #[cfg(feature = "mlua-module")]
    IncompatibleLuaVersion(String),
}

impl fmt::Display for CompiledNamedTextManifestInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            CompiledNamedTextManifestInitError::NamedTextManifestInitError(error) => {
                format!("{}", error)
            }
            CompiledNamedTextManifestInitError::FennelCompileError(error) => format!("{}", error),
            CompiledNamedTextManifestInitError::FennelMountError(error) => format!("{}", error),
            CompiledNamedTextManifestInitError::FennelSearcherError(error) => format!("{}", error),
            #[cfg(feature = "mlua-module")]
            CompiledNamedTextManifestInitError::LuaLibraryLoadError(msg) => format!("{}", msg),
            #[cfg(feature = "mlua-module")]
            CompiledNamedTextManifestInitError::NoLuaVersionSpecified => {
                "No Lua, LuaJIT version specified".to_string()
            }
            #[cfg(feature = "mlua-module")]
            CompiledNamedTextManifestInitError::IncompatibleLuaVersion(msg) => format!("{}", msg),
        };
        write!(f, "{}", res)
    }
}

impl From<NamedTextManifestInitError> for CompiledNamedTextManifestInitError {
    fn from(error: NamedTextManifestInitError) -> Self {
        CompiledNamedTextManifestInitError::NamedTextManifestInitError(error)
    }
}

impl From<fennel_compile::Error> for CompiledNamedTextManifestInitError {
    fn from(error: fennel_compile::Error) -> Self {
        CompiledNamedTextManifestInitError::FennelCompileError(error)
    }
}

impl From<fennel_mount::Error> for CompiledNamedTextManifestInitError {
    fn from(error: fennel_mount::Error) -> Self {
        CompiledNamedTextManifestInitError::FennelMountError(error)
    }
}

impl From<fennel_searcher::Error> for CompiledNamedTextManifestInitError {
    fn from(error: fennel_searcher::Error) -> Self {
        CompiledNamedTextManifestInitError::FennelSearcherError(error)
    }
}

impl error::Error for CompiledNamedTextManifestInitError {}
