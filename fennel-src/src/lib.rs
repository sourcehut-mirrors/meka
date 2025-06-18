use mlua::{Function, Lua, Table, Value};
use mlua_module_manifest::{Manifest, Module, ModuleFileType, ModuleNamedText};
use mlua_utils::TryIntoString;
use paste::paste;
use std::borrow::Cow;
use std::convert::From;

const PAIRS_EXPECT: &str = "`mlua::TablePairs::pairs()` unexpectedly failed";

#[cfg(target_family = "windows")]
macro_rules! path_separator {
    () => {
        r"\"
    };
}

#[cfg(not(target_family = "windows"))]
macro_rules! path_separator {
    () => {
        r"/"
    };
}

macro_rules! path_fennel {
    ($version:expr) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            path_separator!(),
            concat!("fennel-", $version),
            path_separator!(),
            concat!("fennel-", $version, ".lua")
        )
    };
}

macro_rules! path_fennel_asc {
    ($version:expr) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            path_separator!(),
            concat!("fennel-", $version),
            path_separator!(),
            concat!("fennel-", $version, ".lua.asc")
        )
    };
}

macro_rules! setup {
    ($version:expr, $number:tt) => {
        paste! {
            pub const [<FENNEL $number>]: &std::primitive::str = include_str!(path_fennel!($version));
            pub const [<FENNEL $number _PATH>]: &std::primitive::str = path_fennel!($version);
            pub const [<FENNEL $number _ASC_PATH>]: &std::primitive::str = path_fennel_asc!($version);
            pub const [<FENNEL $number _VERSION>]: &std::primitive::str = $version;
        }
    };
}

#[cfg(feature = "fennel100")]
setup!("1.0.0", 100);
#[cfg(feature = "fennel153")]
setup!("1.5.3", 153);

pub fn loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
    let tbl = lua.create_table()?;
    let mt = lua.create_table()?;
    // N.B. first parameter is Lua's special `self` as this is a metatable `_call` function.
    let call = lua.create_function(|_, (_, value): (Value, Value)| {
        let mut version: Option<String> = None;
        let mut name: Option<String> = None;
        match value {
            // Optional argument.
            Value::Nil => {}
            // String argument. Assuming version string.
            Value::String(vsn) => match vsn.to_str() {
                Ok(vsn) => {
                    let vsn = &*vsn;
                    version = Some(vsn.to_string());
                }
                Err(e) => {
                    return Err(mlua::Error::RuntimeError(format!("fennel-src loader function couldn't process string argument: {}", e)));
                }
            }
            // Table argument. Seeking 'version' and 'as' keys.
            Value::Table(table) => {
                for pairs in table.pairs::<Value, Value>() {
                    match pairs.expect(PAIRS_EXPECT) {
                        // Found `mlua::String` key.
                        (Value::String(key), value) => {
                            match key.to_str() {
                                Ok(key) => match &*key {
                                    "version" => {
                                        let value = value.try_into_string().map_err(|e| {
                                            mlua::Error::RuntimeError(format!("fennel-src loader function couldn't process 'version' value string found in table argument: {:#?}", e))
                                        })?;
                                        version = Some(value);
                                    }
                                    "as" => {
                                        let value = value.try_into_string().map_err(|e| {
                                            mlua::Error::RuntimeError(format!("fennel-src loader function couldn't process 'as' value string found in table argument: {:#?}", e))
                                        })?;
                                        name = Some(value);
                                    }
                                    key => {
                                        return Err(mlua::Error::RuntimeError(format!("fennel-src loader function got unsupported key in table argument ({})", key)));
                                    }
                                }
                                Err(e) => {
                                    return Err(mlua::Error::RuntimeError(format!("fennel-src loader function couldn't process string argument: {:#?}", e)));
                                }
                            }
                        }
                        // Found unsupported key.
                        (key, _) => {
                            let got = mlua_utils::typename(&key);
                            return Err(mlua::Error::RuntimeError(format!("fennel-src loader function got table with non-string key (type: {})", got)));
                        }
                    }
                }
            }
            value => {
                let got = mlua_utils::typename(&value);
                return Err(mlua::Error::RuntimeError(format!("fennel-src loader function got unsupported argument type ({})", got)));
            }
        }
        let manifest = manifest(version, name).map_err(|e| mlua::Error::RuntimeError(e))?;
        Ok(manifest)
    })?;
    mt.set("__call", call)?;
    tbl.set_metatable(Some(mt));
    let globals = lua.globals();
    globals.set("fennel_src", tbl)?;
    Ok(lua
        .load("return fennel_src")
        .set_name(name)
        .set_environment(env)
        .into_function()?)
}

pub fn manifest(version: Option<String>, name: Option<String>) -> Result<Manifest, String> {
    let name = if let Some(name) = name {
        name.into()
    } else {
        Cow::from("fennel")
    };

    #[cfg(feature = "fennel153")]
    let text153 = Cow::from(FENNEL153);
    #[cfg(feature = "fennel153")]
    let text = if let Some(version) = version {
        match version.as_ref() {
            "1.5.3" => {
                #[cfg(not(feature = "fennel153"))]
                return Err(
                    "fennel-1.5.3 requested but fennel153 feature flag inactive".to_string()
                );
                text153
            }
            "1.0.0" => {
                return Err(
                    "fennel-1.0.0 requested but fennel100 feature flag inactive".to_string()
                );
            }
            version => {
                return Err(format!(
                    "Unsupported Fennel version requested ({})",
                    version
                ));
            }
        }
    } else {
        text153
    };

    // Repeated text RHS is workaround for limitations of Rust's `#[cfg]` macro.
    #[cfg(feature = "fennel100")]
    let text100 = Cow::from(FENNEL100);
    #[cfg(feature = "fennel100")]
    let text = if let Some(version) = version {
        // Order match branches to prioritize later versions of Fennel in case more than one
        // feature flag is active.
        match version.as_ref() {
            "1.5.3" => {
                return Err(
                    "fennel-1.5.3 requested but fennel153 feature flag inactive".to_string()
                );
            }
            "1.0.0" => {
                #[cfg(not(feature = "fennel100"))]
                return Err(
                    "fennel-1.0.0 requested but fennel100 feature flag inactive".to_string()
                );
                text100
            }
            version => {
                return Err(format!(
                    "Unsupported Fennel version requested ({})",
                    version
                ));
            }
        }
    } else {
        text100
    };

    let file_type = ModuleFileType::Lua;

    Ok(Manifest::new(
        None,
        vec![Module::NamedText(ModuleNamedText {
            name,
            text,
            file_type,
        })],
    ))
}
