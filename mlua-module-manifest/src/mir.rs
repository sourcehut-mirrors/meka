use mlua::{FromLuaMulti, Lua, MultiValue, RegistryKey, Value};
use std::convert::From;
use std::vec::Vec;

use crate::mir_arg::{MirArg, MirArgs};
use crate::mir_consts::{CREATE_REGISTRY_VALUE_EXPECT, REGISTRY_VALUE_EXPECT};

/// Manifest Intermediate Representation (MIR).
pub struct Mir {
    pub mir_args: MirArgs,

    /// `mlua::RegistryKey`s ordered sequentially per `mir_args`.
    ///
    /// Useful if embedding inspect.lua or similar for `MirError` -> `mlua::Error`.
    #[allow(dead_code)]
    pub registry_keys: Vec<RegistryKey>,
}

impl FromLuaMulti for Mir {
    fn from_lua_multi(multi_value: MultiValue, lua: &Lua) -> mlua::Result<Self> {
        let mut registry_keys: Vec<RegistryKey> = Vec::with_capacity(multi_value.len());

        // Make initial pass over `MultiValue` to categorize and extract values.
        let mir_args: Vec<(usize, MirArg)> = multi_value
            .into_vec()
            .into_iter()
            // Enumerate in anticipation of partitioning `mir_args`, which will erase the
            // order in which each argument was originally given.
            .enumerate()
            .map(|(count, value)| {
                let registry_key = lua
                    .create_registry_value(value)
                    .expect(CREATE_REGISTRY_VALUE_EXPECT);

                let value: Value = lua
                    .registry_value(&registry_key)
                    .expect(REGISTRY_VALUE_EXPECT);

                registry_keys.push(registry_key);

                let mir_arg = MirArg::from(value);

                (count, mir_arg)
            })
            .collect();

        let mir_args = MirArgs(mir_args);

        // Always succeeds: `Mir` contains accumulated error values.
        Ok(Mir {
            mir_args,
            registry_keys,
        })
    }
}
