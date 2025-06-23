use meka_config_macros::loader_registry_from_cargo_manifest;
use mlua::{Function, Lua, LuaOptions, ObjectLike, StdLib, Table};
use mlua_module_manifest::{Manifest, Module, ModuleNamedText};
use std::borrow::Cow;
use std::collections::HashMap;

const MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT: &str =
    "fennel-src should return exactly one NamedText module";

#[test]
fn test_single_loader() {
    let loader_registry: HashMap<
        Cow<'static, str>,
        fn(&Lua, Table, &str) -> mlua::Result<Function>,
    > = loader_registry_from_cargo_manifest!();

    // Verify the loader was collected
    assert_eq!(loader_registry.len(), 1);
    assert!(loader_registry.contains_key("fennel-src"));

    // Test that the loader actually works
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let loader: &fn(&Lua, Table, &str) -> mlua::Result<Function> = loader_registry
        .get("fennel-src")
        .expect("Unexpectedly couldn't find fennel-src loader in loader registry");
    let result_fn: Function = loader(&lua, lua.globals(), "fennel-src")
        .expect("Calling fennel_src::loader function unexpectedly failed");
    let fennel_src_table: Table = result_fn
        .call(())
        .expect("Calling fennel_src::loader function in Lua context unexpectedly failed");
    let manifest: Manifest = fennel_src_table
        .call(())
        .expect("Calling callable table returned by fennel_src::loader function in Lua context unexpectedly failed");
    let named_text: ModuleNamedText =
        if let [Module::NamedText(named_text)] = manifest.modules.as_slice() {
            named_text.clone()
        } else {
            panic!("{}", MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT);
        };
    assert_eq!(named_text.name, "fennel");
}
