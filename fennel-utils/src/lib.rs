use mlua::{Function, Lua, Table, Value};

pub mod prelude {
    pub use crate::{FennelView, InsertFennelSearcher};
}

pub trait FennelView {
    fn fennel_view(&self, value: Value, opts: Option<Table>) -> mlua::Result<String>;
}

impl FennelView for Lua {
    fn fennel_view(&self, value: Value, opts: Option<Table>) -> mlua::Result<String> {
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
        if let Some(opts) = opts {
            view.call((value, opts))
        } else {
            view.call(value)
        }
    }
}

pub trait InsertFennelSearcher {
    /// Insert Fennel's searcher function in `package.searchers` (or `package.loaders`).
    ///
    /// Requires: Fennel library is available for import
    fn insert_fennel_searcher(&self) -> mlua::Result<()>;
}

impl InsertFennelSearcher for Lua {
    fn insert_fennel_searcher(&self) -> mlua::Result<()> {
        let fennel = mlua_utils::require::<Table>(self, "fennel").map_err(|_| {
            mlua::Error::RuntimeError(
                "fennel-utils insert_fennel_searcher function couldn't import Fennel".to_string(),
            )
        })?;

        let fennel_make_searcher: Function = fennel.get("make-searcher").map_err(|_| {
            mlua::Error::RuntimeError(
                "fennel-utils insert_fennel_searcher function couldn't get fennel.make-searcher function".to_string(),
            )
        })?;

        let fennel_searcher: Function = fennel_make_searcher.call(()).map_err(|_| {
            mlua::Error::RuntimeError(
                "fennel-utils insert_fennel_searcher function called fennel.make-searcher and got error".to_string(),
            )
        })?;

        let package_searchers: Table = mlua_utils::package_searchers_or_loaders(self).map_err(|e| {
            mlua::Error::RuntimeError(format!("fennel-utils insert_fennel_searcher function couldn't get Lua package.searchers or package.loaders table: {}", e))
        })?;

        let package_searchers_len = package_searchers.len().map_err(|_| {
            mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't get length of Lua package.loaders (or package.searchers) table".to_string())
        })?;

        // There are 4 seachers in `package.searchers` by default (see: `loadlib.c` in Lua
        // source code), but just in case:
        if package_searchers_len > 2 {
            package_searchers
                .raw_insert(package_searchers_len - 2, fennel_searcher)
                .map_err(|_| {
                    mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't insert Fennel searcher in package.loaders (or package.searchers) table at index before last two searchers".to_string())
                })?;
        } else {
            package_searchers
                .push(fennel_searcher)
                .map_err(|_| {
                    mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't append Fennel searcher to package.loaders (or package.searchers) table".to_string())
                })?;
        }

        Ok(())
    }
}
