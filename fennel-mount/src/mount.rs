use mlua::{Lua, Value};
use mlua_searcher::AddSearcher;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::types::Result;

#[cfg(feature = "fennel100")]
pub const FENNEL: &str = fennel_src::FENNEL100;
#[cfg(feature = "fennel153")]
pub const FENNEL: &str = fennel_src::FENNEL153;
#[cfg(not(any(feature = "fennel100", feature = "fennel153")))]
unreachable!("Enforced by Cargo build script");

pub trait Mount {
    /// Add Fennel to Lua's `package.searcher` table via the `mlua-searcher` crate.
    fn mount_fennel(&self) -> Result<()>;
}

impl Mount for Lua {
    fn mount_fennel(&self) -> Result<()> {
        // Check for existing `fennel` module in `package.searchers`.
        match self
            .load(r#"pcall(require, "fennel")"#)
            .eval::<(bool, Value)>()?
        {
            (true, _) => Ok(()),
            (false, _) => {
                let mut map = HashMap::with_capacity(1);
                map.insert(Cow::from("fennel"), Cow::from(FENNEL));
                self.add_searcher(map)?;
                Ok(())
            }
        }
    }
}
