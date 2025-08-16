use mlua_module_manifest::NamedTextManifestInitError;
#[cfg(feature = "mlua-module")]
use savefile::SavefileError;
use savefile_derive::Savefile;
use std::convert::From;
use std::error;
use std::fmt;
#[cfg(feature = "mlua-module")]
use std::io;

#[derive(Debug, Savefile)]
pub enum CompiledNamedTextManifestInitError {
    NamedTextManifestInitError(String),
    FennelCompileError(String),
    FennelMountError(String),
    FennelSearcherError(String),
    #[cfg(feature = "mlua-module")]
    MekaModuleManifestCompiler(String),
    #[cfg(feature = "mlua-module")]
    Io(String),
    #[cfg(feature = "mlua-module")]
    Savefile(String),
}

impl fmt::Display for CompiledNamedTextManifestInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            CompiledNamedTextManifestInitError::NamedTextManifestInitError(msg) => msg,
            CompiledNamedTextManifestInitError::FennelCompileError(msg) => msg,
            CompiledNamedTextManifestInitError::FennelMountError(msg) => msg,
            CompiledNamedTextManifestInitError::FennelSearcherError(msg) => msg,
            #[cfg(feature = "mlua-module")]
            CompiledNamedTextManifestInitError::MekaModuleManifestCompiler(msg) => msg,
            #[cfg(feature = "mlua-module")]
            CompiledNamedTextManifestInitError::Io(msg) => msg,
            #[cfg(feature = "mlua-module")]
            CompiledNamedTextManifestInitError::Savefile(msg) => msg,
        };
        write!(f, "{}", res)
    }
}

impl From<NamedTextManifestInitError> for CompiledNamedTextManifestInitError {
    fn from(error: NamedTextManifestInitError) -> Self {
        CompiledNamedTextManifestInitError::NamedTextManifestInitError(error.to_string())
    }
}

impl From<fennel_compile::Error> for CompiledNamedTextManifestInitError {
    fn from(error: fennel_compile::Error) -> Self {
        CompiledNamedTextManifestInitError::FennelCompileError(error.to_string())
    }
}

impl From<fennel_mount::Error> for CompiledNamedTextManifestInitError {
    fn from(error: fennel_mount::Error) -> Self {
        CompiledNamedTextManifestInitError::FennelMountError(error.to_string())
    }
}

impl From<fennel_searcher::Error> for CompiledNamedTextManifestInitError {
    fn from(error: fennel_searcher::Error) -> Self {
        CompiledNamedTextManifestInitError::FennelSearcherError(error.to_string())
    }
}

#[cfg(feature = "mlua-module")]
impl From<io::Error> for CompiledNamedTextManifestInitError {
    fn from(error: io::Error) -> Self {
        CompiledNamedTextManifestInitError::Io(error.to_string())
    }
}

#[cfg(feature = "mlua-module")]
impl From<SavefileError> for CompiledNamedTextManifestInitError {
    fn from(error: SavefileError) -> Self {
        CompiledNamedTextManifestInitError::Savefile(error.to_string())
    }
}

impl error::Error for CompiledNamedTextManifestInitError {}
