use fennel_compile::Compile;
use fennel_mount::Mount;
use fennel_searcher::AddSearcher;
use mlua::{Lua, LuaOptions, StdLib};
use mlua_module_manifest::{ModuleFileType, ModuleNamedText, Name, NamedTextManifest};
use optional_collections::PushOrInit;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Debug;
use std::ops::Index;
use std::vec::Vec;

#[cfg(feature = "mlua-module")]
use cfg_if::cfg_if;
#[cfg(feature = "mlua-module")]
use libloading::Library;

use crate::error::CompiledNamedTextManifestInitError;

#[derive(Clone, Debug)]
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

#[cfg(feature = "mlua-module")]
fn get_lua_library_names() -> Result<Vec<&'static str>, CompiledNamedTextManifestInitError> {
    cfg_if! {
        // Check for Luau first - it's incompatible with mlua-module
        if #[cfg(any(feature = "mlua-luau", feature = "mlua-luau-jit", feature = "mlua-luau-vector4"))] {
            Err(CompiledNamedTextManifestInitError::IncompatibleLuaVersion(
                "Luau does not support shared library loading and is incompatible with mlua-module feature".to_string()
            ))
        }
        // Lua 5.4
        else if #[cfg(all(feature = "mlua-lua54", windows))] {
            Ok(vec!["lua54.dll", "lua.dll"])
        } else if #[cfg(all(feature = "mlua-lua54", target_os = "macos"))] {
            Ok(vec!["liblua5.4.dylib", "liblua.5.4.dylib", "liblua54.dylib"])
        } else if #[cfg(all(feature = "mlua-lua54", unix))] {
            Ok(vec!["liblua5.4.so", "liblua.so.5.4", "liblua5.4.so.0", "liblua54.so"])
        }
        // Lua 5.3
        else if #[cfg(all(feature = "mlua-lua53", windows))] {
            Ok(vec!["lua53.dll", "lua.dll"])
        } else if #[cfg(all(feature = "mlua-lua53", target_os = "macos"))] {
            Ok(vec!["liblua5.3.dylib", "liblua.5.3.dylib", "liblua53.dylib"])
        } else if #[cfg(all(feature = "mlua-lua53", unix))] {
            Ok(vec!["liblua5.3.so", "liblua.so.5.3", "liblua5.3.so.0", "liblua53.so"])
        }
        // Lua 5.2
        else if #[cfg(all(feature = "mlua-lua52", windows))] {
            Ok(vec!["lua52.dll", "lua.dll"])
        } else if #[cfg(all(feature = "mlua-lua52", target_os = "macos"))] {
            Ok(vec!["liblua5.2.dylib", "liblua.5.2.dylib", "liblua52.dylib"])
        } else if #[cfg(all(feature = "mlua-lua52", unix))] {
            Ok(vec!["liblua5.2.so", "liblua.so.5.2", "liblua5.2.so.0", "liblua52.so"])
        }
        // Lua 5.1
        else if #[cfg(all(feature = "mlua-lua51", windows))] {
            Ok(vec!["lua51.dll", "lua5.1.dll", "lua.dll"])
        } else if #[cfg(all(feature = "mlua-lua51", target_os = "macos"))] {
            Ok(vec!["liblua5.1.dylib", "liblua.5.1.dylib", "liblua51.dylib"])
        } else if #[cfg(all(feature = "mlua-lua51", unix))] {
            Ok(vec!["liblua5.1.so", "liblua.so.5.1", "liblua5.1.so.0", "liblua51.so"])
        }
        // LuaJIT
        else if #[cfg(all(feature = "mlua-luajit", windows))] {
            Ok(vec!["lua51.dll", "luajit.dll", "luajit-5.1.dll"])
        } else if #[cfg(all(feature = "mlua-luajit", target_os = "macos"))] {
            Ok(vec!["libluajit-5.1.dylib", "libluajit.dylib", "libluajit-5.1.2.dylib"])
        } else if #[cfg(all(feature = "mlua-luajit", unix))] {
            Ok(vec!["libluajit-5.1.so.2", "libluajit-5.1.so", "libluajit.so"])
        }
        // LuaJIT 5.2 compat
        else if #[cfg(all(feature = "mlua-luajit52", windows))] {
            Ok(vec!["lua52.dll", "luajit.dll", "luajit-5.2.dll"])
        } else if #[cfg(all(feature = "mlua-luajit52", target_os = "macos"))] {
            Ok(vec!["libluajit-5.2.dylib", "libluajit.dylib"])
        } else if #[cfg(all(feature = "mlua-luajit52", unix))] {
            Ok(vec!["libluajit-5.2.so", "libluajit.so"])
        }
        // Unsupported platform for selected feature
        else if #[cfg(any(feature = "mlua-lua54", feature = "mlua-lua53", feature = "mlua-lua52", feature = "mlua-lua51", feature = "mlua-luajit", feature = "mlua-luajit52"))] {
            Err(CompiledNamedTextManifestInitError::LuaLibraryLoadError(
                "Unsupported platform for selected Lua version".to_string()
            ))
        }
        // No Lua version specified
        else {
            Err(CompiledNamedTextManifestInitError::NoLuaVersionSpecified)
        }
    }
}

#[cfg(feature = "mlua-module")]
fn load_lua_library() -> Result<Library, CompiledNamedTextManifestInitError> {
    let lib_names = get_lua_library_names()?;

    let mut last_error = None;

    // First try to load from standard system paths
    for lib_name in &lib_names {
        match unsafe { Library::new(lib_name) } {
            Ok(lib) => return Ok(lib),
            Err(e) => last_error = Some(e),
        }
    }

    // If that fails, try some common installation paths on Unix systems
    cfg_if! {
        if #[cfg(unix)] {
            let additional_paths = vec![
                "/usr/local/lib",
                "/usr/lib",
                "/usr/lib/x86_64-linux-gnu", // Common on Debian/Ubuntu
                "/usr/lib64",                // Common on RedHat/Fedora
                "/opt/homebrew/lib",         // Homebrew on Apple Silicon
                "/usr/local/opt/lua/lib",    // Homebrew on Intel Mac
                "/usr/local/opt/luajit/lib", // Homebrew LuaJIT
            ];

            for path in additional_paths {
                for lib_name in &lib_names {
                    let full_path = format!("{}/{}", path, lib_name);
                    match unsafe { Library::new(&full_path) } {
                        Ok(lib) => return Ok(lib),
                        Err(e) => last_error = Some(e),
                    }
                }
            }
        }
    }

    // If standard names don't work, create meaningful error
    Err(CompiledNamedTextManifestInitError::LuaLibraryLoadError(
        format!(
            "Could not find Lua library. Tried: {:?}. Last error: {:?}",
            lib_names, last_error
        ),
    ))
}

fn fennelc(
    text: &str,
    modules_fnl_macros: Option<&Vec<ModuleNamedText>>,
) -> Result<String, CompiledNamedTextManifestInitError> {
    let modules_fnl_macros = if let Some(modules_fnl_macros) = modules_fnl_macros {
        let modules_fnl_macros = modules_fnl_macros
            .into_iter()
            .map(|module| module.into())
            .collect::<HashMap<Cow<'static, str>, Cow<'static, str>>>();
        Some(modules_fnl_macros)
    } else {
        None
    };

    // Load Lua library when mlua-module feature is active and keep library loaded for duration
    // of this function
    #[cfg(feature = "mlua-module")]
    let _lua_lib = load_lua_library()?;

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
