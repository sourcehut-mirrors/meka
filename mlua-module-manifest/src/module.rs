use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::clone::Clone;
use std::convert::{From, TryFrom};
use std::env;
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::io::Read;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::string::String;

use crate::mir_arg::Dict;
use crate::module_error::{
    ModuleFileInitError, ModuleInitError, ModuleNamedFileInitError, ModuleNamedTextInitError,
};
use crate::module_traits::Name;
use crate::module_types::{ModuleFileType, ModuleInitResult};

/// Error message designed for running `env::var_os("CARGO_MANIFEST_DIR")`.
const ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT: &str =
    "Unexpectedly couldn't access $CARGO_MANIFEST_DIR environment variable";

/// Error message designed for running `strip_suffix(suffix)` on string verified to contain
/// suffix.
const STR_STRIP_SUFFIX_EXPECT: &str = "Unexpectedly couldn't strip suffix from pre-checked string";

/// Runtime root directory path.
static CARGO_MANIFEST_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let s = env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT);
    PathBuf::from(s)
});

#[derive(Clone, Debug)]
pub struct ModuleFile {
    pub path: PathBuf,
    pub file_type: ModuleFileType,
}

impl ModuleFile {
    pub fn new<P>(path: P, file_type: Option<ModuleFileType>) -> Result<Self, ModuleFileInitError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let file_type = match file_type {
            Some(file_type) => file_type,
            None => ModuleFileType::try_from(path)?,
        };
        Ok(Self {
            path: path.to_owned().into(),
            file_type,
        })
    }
}

impl Name for ModuleFile {
    fn name(&self) -> Cow<'static, str> {
        let path = self.path.as_path();

        // Special handling for init.lua, init.fnl files
        if let Some(file_stem) = path.file_stem() {
            if file_stem == "init" || file_stem == "init-macros" {
                let name = path
                    .parent()
                    .map(|parent| parent.to_string_lossy().into_owned())
                    // Return empty string if init file appears in root directory.
                    .unwrap_or_else(|| "".to_string());
                return replace_path_separators_with_dots(name).into();
            }
        }

        // Standard handling: strip file extension
        let name = if let Some(file_extension) = path.extension() {
            let file_extension = file_extension.to_string_lossy();
            let file_extension: &str = file_extension.as_ref();
            let suffix = format!(".{}", file_extension);
            path.to_string_lossy()
                .strip_suffix(&suffix)
                .expect(STR_STRIP_SUFFIX_EXPECT)
                .to_owned()
        } else {
            path.to_string_lossy().into_owned()
        };

        replace_path_separators_with_dots(name).into()
    }
}

fn replace_path_separators_with_dots(name: String) -> String {
    let name = name.replace(MAIN_SEPARATOR, ".");

    // Windows allows backslash and forward-slash path separators.
    #[cfg(target_family = "windows")]
    let name = name.replace("/", ".");

    name
}

impl fmt::Display for ModuleFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = format!(
            "ModuleFile {{path => {:?}, file_type => {}}}",
            self.path.as_path(),
            self.file_type
        );
        write!(f, "{}", res)
    }
}

#[derive(Clone, Debug)]
pub struct ModuleNamedFile {
    pub name: Cow<'static, str>,
    pub path: PathBuf,
    pub file_type: ModuleFileType,
}

impl ModuleNamedFile {
    pub fn new<P, S>(
        name: S,
        path: P,
        file_type: Option<ModuleFileType>,
    ) -> Result<Self, ModuleNamedFileInitError>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let path = path.as_ref();
        let file_type = match file_type {
            Some(file_type) => file_type,
            None => ModuleFileType::try_from(path)
                // Reuse error for d-r-y and laziness++.
                .map_err(|e| ModuleFileInitError::from(e))?,
        };
        Ok(Self {
            name: name.as_ref().to_owned().into(),
            path: path.to_owned().into(),
            file_type,
        })
    }
}

impl From<ModuleNamedFile> for ModuleFile {
    fn from(
        ModuleNamedFile {
            name: _,
            path,
            file_type,
        }: ModuleNamedFile,
    ) -> Self {
        Self { path, file_type }
    }
}

impl Name for ModuleNamedFile {
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}

impl fmt::Display for ModuleNamedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = format!(
            "ModuleNamedFile {{name => {:?}, path => {:?}, file_type => {}}}",
            self.name.as_ref(),
            self.path.as_path(),
            self.file_type
        );
        write!(f, "{}", res)
    }
}

#[derive(Clone, Debug)]
pub struct ModuleNamedText {
    pub name: Cow<'static, str>,
    pub text: Cow<'static, str>,
    pub file_type: ModuleFileType,
}

impl ModuleNamedText {
    // Return `Result` for API consistency, and because some form of validation might be incoming.
    pub fn new<A, B>(
        name: A,
        text: B,
        file_type: ModuleFileType,
    ) -> Result<Self, ModuleNamedTextInitError>
    where
        A: AsRef<str>,
        B: AsRef<str>,
    {
        Ok(Self {
            name: name.as_ref().to_owned().into(),
            text: text.as_ref().to_owned().into(),
            file_type,
        })
    }
}

impl TryFrom<ModuleFile> for ModuleNamedText {
    type Error = ModuleNamedTextInitError;

    fn try_from(module_file: ModuleFile) -> Result<Self, ModuleNamedTextInitError> {
        let name = module_file.name();
        let ModuleFile { path, file_type } = module_file;
        let mut text = String::new();
        let module_file = CARGO_MANIFEST_DIR.join(path.as_path());
        let mut module_file = fs::File::open(&module_file)?;
        module_file.read_to_string(&mut text)?;
        Ok(ModuleNamedText {
            name: name.into(),
            text: text.into(),
            file_type,
        })
    }
}

impl TryFrom<ModuleNamedFile> for ModuleNamedText {
    type Error = ModuleNamedTextInitError;

    fn try_from(
        ModuleNamedFile {
            name,
            path,
            file_type,
        }: ModuleNamedFile,
    ) -> Result<Self, ModuleNamedTextInitError> {
        let mut text = String::new();
        let module_named_file = CARGO_MANIFEST_DIR.join(path);
        let mut module_named_file = fs::File::open(&module_named_file)?;
        module_named_file.read_to_string(&mut text)?;
        Ok(ModuleNamedText {
            name,
            text: text.into(),
            file_type,
        })
    }
}

impl From<&ModuleNamedText> for (Cow<'static, str>, Cow<'static, str>) {
    fn from(
        ModuleNamedText {
            name,
            text,
            file_type: _,
        }: &ModuleNamedText,
    ) -> Self {
        (
            name.clone().into_owned().into(),
            text.clone().into_owned().into(),
        )
    }
}

impl Name for ModuleNamedText {
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}

impl fmt::Display for ModuleNamedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = format!(
            "ModuleNamedText {{name => {:?}, text => {:?}, file_type => {}}}",
            self.name.as_ref(),
            self.text.as_ref(),
            self.file_type
        );
        write!(f, "{}", res)
    }
}

#[derive(Clone, Debug)]
pub enum Module {
    File(ModuleFile),
    NamedFile(ModuleNamedFile),
    NamedText(ModuleNamedText),
}

impl Name for Module {
    fn name(&self) -> Cow<'static, str> {
        match self {
            Module::File(module_file) => module_file.name(),
            Module::NamedFile(module_named_file) => module_named_file.name(),
            Module::NamedText(module_named_text) => module_named_text.name(),
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Module::File(module_file) => format!("Module::File({})", module_file),
            Module::NamedFile(module_named_file) => {
                format!("Module::NamedFile({})", module_named_file)
            }
            Module::NamedText(module_named_text) => {
                format!("Module::NamedText({})", module_named_text)
            }
        };
        write!(f, "{}", res)
    }
}

impl TryFrom<Dict> for Module {
    type Error = ModuleInitError;

    fn try_from(dict: Dict) -> ModuleInitResult<Self> {
        // `unreachable!`s below hinge upon `Dict.validate` function
        match (&dict.name, &dict.path, &dict.text, &dict.file_type) {
            // either `path` or `text` field must be present
            (_, None, None, _) => unreachable!(),
            // `path` and `text` fields are mutually exclusive
            (_, Some(_), Some(_), _) => unreachable!(),
            // `text` field must appear with `type` field
            (_, _, Some(_), None) => unreachable!(),
            // `text` field must appear with `name` field
            (None, _, Some(_), _) => unreachable!(),
            // `path` given; this will succeed if `type` field contains valid file type or
            // `path` contains valid file extension
            (None, Some(path), None, maybe_file_type) => {
                let path: &Path = path.as_ref();
                let file_type = if let Some(file_type) = maybe_file_type {
                    let file_type: &str = file_type.as_ref();
                    ModuleFileType::try_from(file_type)
                        .map_err(|e| ModuleInitError::from(ModuleFileInitError::from(e)))?
                } else {
                    ModuleFileType::try_from(path)
                        .map_err(|e| ModuleInitError::from(ModuleFileInitError::from(e)))?
                };
                let path = path.to_owned();
                Ok(Module::File(ModuleFile { path, file_type }))
            }
            // `name` and `path` given; this will succeed if `path` contains valid file extension
            (Some(name), Some(path), None, None) => {
                let name: String = name.to_owned();
                let path: &Path = path.as_ref();
                let file_type = ModuleFileType::try_from(path)
                    .map_err(|e| ModuleInitError::from(ModuleNamedFileInitError::from(e)))?;
                let name = name.into();
                let path = path.to_owned();
                Ok(Module::NamedFile(ModuleNamedFile {
                    name,
                    path,
                    file_type,
                }))
            }
            // `name`, `path` and `type` given; this will always succeed
            (Some(name), Some(path), None, Some(file_type)) => {
                let name: String = name.to_owned();
                let path = path.into();
                let file_type: &str = file_type.as_ref();
                let file_type = ModuleFileType::try_from(file_type)
                    .map_err(|e| ModuleInitError::from(ModuleNamedFileInitError::from(e)))?;
                let name = name.into();
                Ok(Module::NamedFile(ModuleNamedFile {
                    name,
                    path,
                    file_type,
                }))
            }
            // `name`, `text` and `type` given; this will succeed if `type` is valid
            (Some(name), None, Some(text), Some(file_type)) => {
                let name: String = name.to_owned();
                let text = text.to_owned();
                let file_type: &str = file_type.as_ref();
                let file_type = ModuleFileType::try_from(file_type).map_err(|_| {
                    let file_type = file_type.to_owned();
                    let e = ModuleNamedTextInitError::UnknownModuleFileType { file_type };
                    ModuleInitError::ModuleNamedTextInitError(e)
                })?;
                let name = name.into();
                let text = text.into();
                Ok(Module::NamedText(ModuleNamedText {
                    name,
                    text,
                    file_type,
                }))
            }
        }
    }
}
