use mlua::{Function, Lua, Table, Value};

pub mod prelude {
    pub use crate::{FennelView, InsertFennelSearcher};
}

/// Error message designed for running `table.get(key)` on `mlua::Table` `table` verified to
/// contain key `key`.
const TABLE_GET_EXPECT: &str = "Unexpectedly couldn't get key from pre-checked table";

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

        let package: Table = self.globals().get("package").map_err(|_| {
            mlua::Error::RuntimeError(
                "fennel-utils insert_fennel_searcher function couldn't get Lua package table"
                    .to_string(),
            )
        })?;

        let package_loaders: Table = if package.contains_key("loaders").map_err(|_| {
            mlua::Error::RuntimeError(
                "fennel-utils insert_fennel_searcher function couldn't check if package table contains loaders key"
                    .to_string(),
            )
        })? {
            package.get("loaders").expect(TABLE_GET_EXPECT)
        } else if package.contains_key("searchers").map_err(|_| {
            mlua::Error::RuntimeError(
                "fennel-utils insert_fennel_searcher function couldn't check if package table contains searchers key"
                    .to_string(),
            )
        })? {
            package.get("searchers").expect(TABLE_GET_EXPECT)
        } else {
            return Err(mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't find either Lua package.loaders or package.searchers table".to_string()));
        };

        let package_loaders_len = package_loaders.len().map_err(|_| {
            mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't get length of Lua package.loaders (or package.searchers) table".to_string())
        })?;

        // There are 4 seachers in `package.searchers` by default (see: `loadlib.c` in Lua
        // source code), but just in case:
        if package_loaders_len > 2 {
            package_loaders
                .raw_insert(package_loaders_len - 2, fennel_searcher)
                .map_err(|_| {
                    mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't insert Fennel searcher in package.loaders (or package.searchers) table at index before last two searchers".to_string())
                })?;
        } else {
            package_loaders
                .push(fennel_searcher)
                .map_err(|_| {
                    mlua::Error::RuntimeError("fennel-utils insert_fennel_searcher function couldn't append Fennel searcher to package.loaders (or package.searchers) table".to_string())
                })?;
        }

        Ok(())
    }
}
