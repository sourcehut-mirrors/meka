use std::convert::From;
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ModuleFileTypeInitError {
    /// Path is missing file extension.
    MissingFileExtension { path: PathBuf },
    /// Path is missing file name.
    MissingFileName { path: PathBuf },
    /// Path contains unknown file extension.
    UnknownFileExtension { path: PathBuf },
    /// String contains unknown file type.
    UnknownFileType { file_type: String },
}

impl fmt::Display for ModuleFileTypeInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ModuleFileTypeInitError::MissingFileExtension { path } => {
                format!("Path ({:?}) is missing file extension.", path)
            }
            ModuleFileTypeInitError::MissingFileName { path } => {
                format!("Path ({:?}) is missing file name.", path)
            }
            ModuleFileTypeInitError::UnknownFileExtension { path } => {
                format!("Path ({:?}) contains unknown file extension.", path)
            }
            ModuleFileTypeInitError::UnknownFileType { file_type } => {
                format!("String ({}) contains unknown file type.", file_type)
            }
        };
        write!(f, "{}", res)
    }
}

impl error::Error for ModuleFileTypeInitError {}

#[derive(Debug)]
pub enum ModuleFileInitError {
    /// Optional `file_type` parameter not passed, and `path` parameter is missing file extension.
    MissingFileExtension { path: PathBuf },
    /// Optional `file_type` parameter not passed, and `path` parameter is missing file name.
    MissingFileName { path: PathBuf },
    /// Optional `file_type` parameter not passed, and `path` parameter contains unknown file extension.
    UnknownFileExtension { path: PathBuf },
    /// Optional `file_type` parameter contains unknown file type.
    UnknownFileType { file_type: String },
}

impl fmt::Display for ModuleFileInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ModuleFileInitError::MissingFileExtension { path } => format!(
                "Optional `file_type` parameter not passed, and `path` parameter ({:?}) is missing file extension.",
                path
            ),
            ModuleFileInitError::MissingFileName { path } => format!(
                "Optional `file_type` parameter not passed, and `path` parameter ({:?}) is missing file name.",
                path
            ),
            ModuleFileInitError::UnknownFileExtension { path } => format!(
                "Optional `file_type` parameter not passed, and `path` parameter ({:?}) contains unknown file extension.",
                path
            ),
            ModuleFileInitError::UnknownFileType { file_type } => format!(
                "Optional `file_type` parameter contains unknown file type ({}).",
                file_type
            ),
        };
        write!(f, "{}", res)
    }
}

impl From<ModuleFileTypeInitError> for ModuleFileInitError {
    fn from(error: ModuleFileTypeInitError) -> Self {
        match error {
            ModuleFileTypeInitError::MissingFileExtension { path } => {
                ModuleFileInitError::MissingFileExtension { path }
            }
            ModuleFileTypeInitError::MissingFileName { path } => {
                ModuleFileInitError::MissingFileName { path }
            }
            ModuleFileTypeInitError::UnknownFileExtension { path } => {
                ModuleFileInitError::UnknownFileExtension { path }
            }
            ModuleFileTypeInitError::UnknownFileType { file_type } => {
                ModuleFileInitError::UnknownFileType { file_type }
            }
        }
    }
}

impl error::Error for ModuleFileInitError {}

#[derive(Debug)]
pub enum ModuleNamedFileInitError {
    ModuleFileInitError(ModuleFileInitError),
    UnknownModuleFileType { file_type: String },
}

impl fmt::Display for ModuleNamedFileInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ModuleNamedFileInitError::ModuleFileInitError(error) => format!("{}", error),
            ModuleNamedFileInitError::UnknownModuleFileType { file_type } => {
                format!("Got unsupported module file type ({})", file_type)
            }
        };
        write!(f, "{}", res)
    }
}

impl From<ModuleFileInitError> for ModuleNamedFileInitError {
    fn from(error: ModuleFileInitError) -> Self {
        ModuleNamedFileInitError::ModuleFileInitError(error)
    }
}

impl From<ModuleFileTypeInitError> for ModuleNamedFileInitError {
    fn from(error: ModuleFileTypeInitError) -> Self {
        ModuleNamedFileInitError::ModuleFileInitError(ModuleFileInitError::from(error))
    }
}

impl error::Error for ModuleNamedFileInitError {}

#[derive(Debug)]
pub enum ModuleNamedTextInitError {
    Io(std::io::Error),
    UnknownModuleFileType { file_type: String },
}

impl fmt::Display for ModuleNamedTextInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ModuleNamedTextInitError::Io(e) => format!("IO error:\n{}", e),
            ModuleNamedTextInitError::UnknownModuleFileType { file_type } => {
                format!("Got unsupported module file type ({})", file_type)
            }
        };
        write!(f, "{}", res)
    }
}

impl From<std::io::Error> for ModuleNamedTextInitError {
    fn from(error: std::io::Error) -> Self {
        ModuleNamedTextInitError::Io(error)
    }
}

impl error::Error for ModuleNamedTextInitError {}

#[derive(Debug)]
pub enum ModuleInitError {
    ModuleFileInitError(ModuleFileInitError),
    ModuleNamedFileInitError(ModuleNamedFileInitError),
    ModuleNamedTextInitError(ModuleNamedTextInitError),
}

impl fmt::Display for ModuleInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ModuleInitError::ModuleFileInitError(error) => format!("{}", error),
            ModuleInitError::ModuleNamedFileInitError(error) => format!("{}", error),
            ModuleInitError::ModuleNamedTextInitError(error) => format!("{}", error),
        };
        write!(f, "{}", res)
    }
}

impl From<ModuleFileInitError> for ModuleInitError {
    fn from(error: ModuleFileInitError) -> Self {
        ModuleInitError::ModuleFileInitError(error)
    }
}

impl From<ModuleNamedFileInitError> for ModuleInitError {
    fn from(error: ModuleNamedFileInitError) -> Self {
        ModuleInitError::ModuleNamedFileInitError(error)
    }
}

impl From<ModuleNamedTextInitError> for ModuleInitError {
    fn from(error: ModuleNamedTextInitError) -> Self {
        ModuleInitError::ModuleNamedTextInitError(error)
    }
}

impl error::Error for ModuleInitError {}
