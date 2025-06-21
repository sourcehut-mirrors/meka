use io_cat::CatBox;
use std::borrow::Cow;
use std::clone::Clone;
use std::fmt;
use std::fmt::Debug;
use std::path::Path;
use std::result::Result;

use crate::module_error::{ModuleFileTypeInitError, ModuleInitError};

pub type ModuleInitResult<A> = Result<A, ModuleInitError>;

#[derive(Clone, Debug)]
pub enum ModuleFileType {
    Fennel,
    FennelMacros,
    Lua,
}

impl TryFrom<&Path> for ModuleFileType {
    type Error = ModuleFileTypeInitError;

    fn try_from(path: &Path) -> Result<Self, ModuleFileTypeInitError> {
        let file_extension =
            path.extension()
                .ok_or_else(|| ModuleFileTypeInitError::MissingFileExtension {
                    path: path.to_owned(),
                })?;
        let file_type = match file_extension.to_string_lossy().as_ref() {
            "fnl" => ModuleFileType::Fennel,
            "fnlm" => ModuleFileType::FennelMacros,
            "lua" => ModuleFileType::Lua,
            _ => Err(ModuleFileTypeInitError::UnknownFileExtension {
                path: path.to_owned(),
            })?,
        };
        let file_type = match file_type {
            ModuleFileType::Fennel => {
                // Check for `init-macros.fnl`.
                let file_stem = path
                    .file_stem()
                    .ok_or_else(|| ModuleFileTypeInitError::MissingFileName {
                        path: path.to_owned(),
                    })?
                    .to_string_lossy();
                if file_stem.eq("init-macros") {
                    ModuleFileType::FennelMacros
                } else {
                    ModuleFileType::Fennel
                }
            }
            // `ModuleFileType::FennelMacros` requires no further action.
            ModuleFileType::FennelMacros => ModuleFileType::FennelMacros,
            // `ModuleFileType::Lua` requires no further action.
            ModuleFileType::Lua => ModuleFileType::Lua,
        };
        Ok(file_type)
    }
}

impl TryFrom<&str> for ModuleFileType {
    type Error = ModuleFileTypeInitError;

    fn try_from(file_type: &str) -> Result<Self, ModuleFileTypeInitError> {
        match file_type {
            "fennel" => Ok(ModuleFileType::Fennel),
            "fennel-macros" => Ok(ModuleFileType::FennelMacros),
            "lua" => Ok(ModuleFileType::Lua),
            _ => Err(ModuleFileTypeInitError::UnknownFileType {
                file_type: file_type.to_owned(),
            }),
        }
    }
}

impl fmt::Display for ModuleFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            ModuleFileType::Fennel => "ModuleFileType::Fennel",
            ModuleFileType::FennelMacros => "ModuleFileType::FennelMacros",
            ModuleFileType::Lua => "ModuleFileType::Lua",
        };
        write!(f, "{}", res)
    }
}
