use mlua::{Function, Lua, Table, Value};

pub mod prelude {
    pub use crate::FennelView;
}

pub trait FennelView {
    fn fennel_view(&self, value: Value) -> mlua::Result<String>;
}

impl FennelView for Lua {
    fn fennel_view(&self, value: Value) -> mlua::Result<String> {
        let fennel = mlua_utils::require::<Table>(self, "fennel").map_err(|e| {
            mlua::Error::RuntimeError(format!(
                "fennel-utils fennel_view function couldn't import Fennel: {}",
                e
            ))
        })?;
        let view: Function = fennel.get("view").map_err(|e| {
            mlua::Error::RuntimeError(format!(
                "fennel-utils fennel_view function couldn't get fennel.view: {}",
                e
            ))
        })?;
        view.call(value)
    }
}
