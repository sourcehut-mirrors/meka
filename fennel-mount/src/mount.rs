use mlua::Lua;
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
    /// Add bespoke static string searcher to Lua's `package.searcher` table which enables
    /// importing Fennel unless Fennel is already possible to import.
    fn mount_fennel(&self) -> Result<()>;
}

impl Mount for Lua {
    fn mount_fennel(&self) -> Result<()> {
        // Check for existing `fennel` module in `package.loaded`.
        let package_loaded_contains_fennel = mlua_utils::package_loaded_contains(self, "fennel")?;

        // Check for existing `fennel` module via searcher in `package.searchers`.
        match mlua_utils::pcall_require(self, "fennel")? {
            (true, _) => {
                // Remove `fennel` module from `package.loaded` cache unless it was there already.
                if !package_loaded_contains_fennel {
                    mlua_utils::unload_module(self, "fennel")?;
                }
            }
            (false, _) => {
                // Enable importing Fennel by name.
                let mut map = HashMap::with_capacity(1);
                map.insert(Cow::from("fennel"), Cow::from(FENNEL));
                self.add_searcher(map)?;
            }
        }
        Ok(())
    }
}
