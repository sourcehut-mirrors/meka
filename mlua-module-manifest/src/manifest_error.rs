use std::convert::From;
use std::error;
use std::fmt;
use std::fmt::Debug;

use crate::module_error::{ModuleInitError, ModuleNamedTextInitError};

#[derive(Debug)]
pub enum ManifestInitError {
    ModuleInitError(ModuleInitError),
}

impl fmt::Display for ManifestInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ManifestInitError::ModuleInitError(error) => format!("{}", error),
        };
        write!(f, "{}", res)
    }
}

impl From<ModuleInitError> for ManifestInitError {
    fn from(error: ModuleInitError) -> Self {
        ManifestInitError::ModuleInitError(error)
    }
}

impl From<ManifestInitError> for mlua::Error {
    fn from(error: ManifestInitError) -> Self {
        mlua::Error::RuntimeError(format!("mlua-module-manifest error: {}", error))
    }
}

impl error::Error for ManifestInitError {}

#[derive(Debug)]
pub enum NamedTextManifestInitError {
    ManifestInitError(ManifestInitError),
    ModuleNamedTextInitError(ModuleNamedTextInitError),
}

impl fmt::Display for NamedTextManifestInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            NamedTextManifestInitError::ManifestInitError(error) => format!("{}", error),
            NamedTextManifestInitError::ModuleNamedTextInitError(error) => format!("{}", error),
        };
        write!(f, "{}", res)
    }
}

impl From<ManifestInitError> for NamedTextManifestInitError {
    fn from(error: ManifestInitError) -> Self {
        NamedTextManifestInitError::ManifestInitError(error)
    }
}

impl From<ModuleNamedTextInitError> for NamedTextManifestInitError {
    fn from(error: ModuleNamedTextInitError) -> Self {
        NamedTextManifestInitError::ModuleNamedTextInitError(error)
    }
}

impl error::Error for NamedTextManifestInitError {}
