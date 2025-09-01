#[cfg(any(feature = "mlua-external", feature = "mlua-vendored"))]
use mlua_module_manifest::ModuleFileType;
use mlua_module_manifest::{ModuleNamedText, Name, NamedTextManifest};
#[cfg(any(feature = "mlua-external", feature = "mlua-vendored"))]
use optional_collections::PushOrInit;
use savefile_derive::Savefile;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Debug;
use std::ops::Index;
use std::vec::Vec;

use crate::error::CompiledNamedTextManifestInitError;

#[cfg(host_family = "windows")]
macro_rules! path_separator { () => { r"\" }; }
#[cfg(not(host_family = "windows"))]
macro_rules! path_separator { () => { r"/" }; }

#[derive(Clone, Debug, Savefile)]
pub struct CompiledNamedTextManifest {
    pub docstring: Option<Cow<'static, str>>,
    pub modules: Vec<ModuleNamedText>,
}

impl CompiledNamedTextManifest {
    pub fn get<'a>(&'a self, name: &str) -> Option<&'a ModuleNamedText> {
        self.modules
            .iter()
            .filter(|module| module.name().eq(name))
            .last()
    }

    /// Selectively remove `ModuleNamedText`s from `modules` vector by position.
    pub fn omit(self, omit: Vec<usize>) -> Self {
        let Self { docstring, modules } = self;
        let modules = modules
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !omit.contains(&i))
            .map(|(_, module)| module)
            .collect();
        Self { docstring, modules }
    }
}

impl Index<usize> for CompiledNamedTextManifest {
    type Output = ModuleNamedText;

    fn index(&self, index: usize) -> &Self::Output {
        &self.modules[index]
    }
}

/// Intended to be the only way to instantiate `CompiledNamedTextManifest`. For typestate pattern.
impl TryFrom<NamedTextManifest> for CompiledNamedTextManifest {
    type Error = CompiledNamedTextManifestInitError;

    /// Compile `ModuleFileType::Fennel` strings within `modules` to Lua, and attest to this
    /// having been done in a type-safe way.
    #[cfg(feature = "mlua-module")]
    fn try_from(manifest: NamedTextManifest) -> Result<Self, CompiledNamedTextManifestInitError> {
        use savefile::{CURRENT_SAVEFILE_LIB_VERSION, load_from_mem, save_to_mem};
        use std::io::Write;
        use std::path::Path;
        use std::process::{Command, Stdio};

        const CARGO_MANIFEST_DIR_PARENT_EXPECT: &str = "Failed to find Cargo workspace root";

        // Serialize manifest.
        let serialized = save_to_mem(CURRENT_SAVEFILE_LIB_VERSION.into(), &manifest)?;

        // Run ephemeral crate with isolated `target/`.
        let mut child = {
            let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect(CARGO_MANIFEST_DIR_PARENT_EXPECT);

            // Compile meka-module-manifest-compiler with Lua matching active feature selection.
            let features: &str = include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                path_separator!(),
                "src",
                path_separator!(),
                "include",
                path_separator!(),
                "features.rs"
            ));

            Command::new("cargo")
                .arg("run")
                .arg("--release")
                .arg("--quiet")
                .args(["--package", "meka-module-manifest-compiler"])
                .args(["--features", features])
                .current_dir(workspace_root)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        // Send serialized manifest.
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&serialized)?;
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(
                CompiledNamedTextManifestInitError::MekaModuleManifestCompiler(format!(
                    "Ephemeral crate failed: {}",
                    stderr
                )),
            );
        }

        // Deserialize result.
        let result: Result<CompiledNamedTextManifest, CompiledNamedTextManifestInitError> =
            load_from_mem(&output.stdout, CURRENT_SAVEFILE_LIB_VERSION.into())?;

        result
    }

    #[cfg(any(feature = "mlua-external", feature = "mlua-vendored"))]
    fn try_from(
        NamedTextManifest { docstring, modules }: NamedTextManifest,
    ) -> Result<Self, CompiledNamedTextManifestInitError> {
        let mut modules_fnl_macros: Option<Vec<ModuleNamedText>> = None;
        for module in modules.iter().cloned() {
            if let ModuleFileType::FennelMacros = &module.file_type {
                modules_fnl_macros.push_or_init(module);
            }
        }
        let modules = modules
            .into_iter()
            .map(
                |ModuleNamedText {
                     name,
                     text,
                     file_type,
                 }| match file_type {
                    // Compile Fennel to Lua. Ensure all Fennel macros in searcher config are
                    // available for evaluation during Fennel-to-Lua compilation.
                    ModuleFileType::Fennel => {
                        match fennelc(text.as_ref(), modules_fnl_macros.as_ref()) {
                            Ok(text) => Ok(ModuleNamedText {
                                name,
                                text: text.into(),
                                file_type,
                            }),
                            Err(e) => Err(e),
                        }
                    }

                    // Fennel macros are evaluated during Fennel-to-Lua compilation. They
                    // aren't AOT compiled themselves.
                    ModuleFileType::FennelMacros => Ok(ModuleNamedText {
                        name,
                        text,
                        file_type,
                    }),

                    // Lua modules require no further processing.
                    ModuleFileType::Lua => Ok(ModuleNamedText {
                        name,
                        text,
                        file_type,
                    }),
                },
            )
            .collect::<Result<Vec<ModuleNamedText>, CompiledNamedTextManifestInitError>>()?;

        Ok(Self { docstring, modules })
    }
}

impl TryFrom<mlua_module_manifest::Manifest> for CompiledNamedTextManifest {
    type Error = CompiledNamedTextManifestInitError;

    fn try_from(
        manifest: mlua_module_manifest::Manifest,
    ) -> Result<Self, CompiledNamedTextManifestInitError> {
        CompiledNamedTextManifest::try_from(NamedTextManifest::try_from(manifest)?)
    }
}

impl fmt::Display for CompiledNamedTextManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res =
            fmt::<ModuleNamedText>("CompiledNamedTextManifest", &self.docstring, &self.modules);
        write!(f, "{}", res)
    }
}

#[cfg(any(feature = "mlua-external", feature = "mlua-vendored"))]
fn fennelc(
    text: &str,
    modules_fnl_macros: Option<&Vec<ModuleNamedText>>,
) -> Result<String, CompiledNamedTextManifestInitError> {
    use fennel_compile::Compile;
    use fennel_mount::Mount;
    use fennel_searcher::AddSearcher;
    use mlua::{Lua, LuaOptions, StdLib};
    use std::collections::HashMap;
    let modules_fnl_macros = if let Some(modules_fnl_macros) = modules_fnl_macros {
        let modules_fnl_macros = modules_fnl_macros
            .into_iter()
            .map(|module| module.into())
            .collect::<HashMap<Cow<'static, str>, Cow<'static, str>>>();
        Some(modules_fnl_macros)
    } else {
        None
    };
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.mount_fennel()?;
    // Mount all modules containing Fennel macros prior to compilation.
    if let Some(modules_fnl_macros) = modules_fnl_macros {
        lua.add_searcher_fnl_macros(modules_fnl_macros)?;
    }
    Ok(lua.compile_fennel_string(text)?)
}

fn fmt<T>(type_name: &str, docstring: &Option<Cow<'static, str>>, modules: &Vec<T>) -> String
where
    T: fmt::Display,
{
    let mut v = String::new();
    for module in modules {
        String::push_str(&mut v, &format!("{}, ", module));
    }

    // Trim trailing whitespace.
    _ = v.pop();
    // Trim trailing comma separator.
    _ = v.pop();

    if let Some(docstring) = docstring.as_ref() {
        format!(
            "{} {{ docstring: {:?}, modules: vec![{}] }}",
            type_name, docstring, v
        )
    } else {
        format!("{} {{ modules: vec![{}] }}", type_name, v)
    }
}
