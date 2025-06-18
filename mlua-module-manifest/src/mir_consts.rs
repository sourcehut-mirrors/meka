/// Error message designed for unwrapping `Result` from `mlua::Lua::create_registry_value()`,
/// which should always succeed.
pub const CREATE_REGISTRY_VALUE_EXPECT: &str = "Creating registry value unexpectedly failed";

/// Error message for pre-checked `mlua::FromLuaMulti::from_lua_multi()` result.
pub const FROM_LUA_MULTI_EXPECT: &str =
    "`mlua::FromLuaMulti::from_lua_multi()` unexpectedly failed";

/// Error message for pre-checked `mlua::IntoLua::into_lua()` result.
pub const INTO_LUA_EXPECT: &str = "`mlua::IntoLua::into_lua()` unexpectedly failed";

/// Error message for `Iterator::Item.expect()` in `mlua::TablePairs`es - which `mlua`
/// wraps in `Result` to facilitate lazily converting Lua types to Rust. Presumably this
/// can only fail if the user requests a Rust type which doesn't implement `FromLua`.
pub const PAIRS_EXPECT: &str = "`mlua::TablePairs::pairs()` unexpectedly failed";

/// Error message designed for situations where we're sifting through partitioned data.
pub const PARTITIONED_EXPECT: &str = "Partitioning unexpectedly failed";

/// Error message designed for unwrapping `Result` from `mlua::Lua::registry_value()`, which
/// should always succeed.
pub const REGISTRY_VALUE_EXPECT: &str = "Fetching registry value unexpectedly failed";
