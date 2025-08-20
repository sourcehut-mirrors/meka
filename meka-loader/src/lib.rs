use mlua::{Function, Lua, Table};
use mlua_module_manifest::Manifest;
use std::borrow::Cow;
use std::collections::HashMap;

pub mod prelude {
    pub use crate::{LoaderFn, LoaderRegistry, loader};
}

/// Type alias for mlua loader function signature.
pub type LoaderFn = fn(&Lua, Table, &str) -> mlua::Result<Function>;

/// `LoaderRegistry` is a `HashMap` of Lua loader functions indexed by name.
///
/// Each Lua loader function must return an `mlua::Function` which, when called, returns an
/// `mlua::Table` with a `__call` metamethod defined. Calling said `mlua::Table` must return
/// an `mlua_module_manifest::Manifest`. The idea is to enable Rust crates to export complete
/// Lua modules. We map those exported Lua modules to names which can be `require`d within a
/// Meka config.
///
/// Type alias for loader registry used by mlua-searcher's `add_function_searcher`.
pub type LoaderRegistry = HashMap<Cow<'static, str>, LoaderFn>;

/// Implementation of the Meka loader function.
///
/// Provides `meka.manifest` module within Lua configs.
pub fn loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
    let globals = lua.globals();

    let tbl = lua.create_table().map_err(|_| {
        mlua::Error::RuntimeError(
            "meka_loader::loader function failed to create Lua table".to_string(),
        )
    })?;

    let manifest: Function = Manifest::loader(lua, env.clone(), "manifest").map_err(|_| {
        mlua::Error::RuntimeError(
            "meka_loader::loader function called Manifest::loader and got error".to_string(),
        )
    })?;
    let manifest: Table = manifest.call(()).map_err(|_| {
        mlua::Error::RuntimeError(
            "meka_loader::loader function called Manifest::loader in Lua context and got error"
                .to_string(),
        )
    })?;
    tbl.set("manifest", manifest).map_err(|_| {
        mlua::Error::RuntimeError(
            "meka_loader::loader function failed to set Lua table".to_string(),
        )
    })?;

    globals.set("meka", tbl).map_err(|_| {
        mlua::Error::RuntimeError(
            "meka_loader::loader function failed to set Lua table".to_string(),
        )
    })?;

    Ok(lua
        .load("return meka")
        .set_name(name)
        .set_environment(env)
        .into_function()?)
}
