pub mod test_loaders {
    use mlua::{Function, Lua, Table, UserData, UserDataMethods};

    pub struct Cartridge {
        title: String,
    }

    impl Cartridge {
        pub fn pick() -> Self {
            let title = "Super Smash Brothers 64".to_string();
            Self { title }
        }

        pub fn play(&self) -> String {
            self.title.clone()
        }
    }

    impl UserData for Cartridge {
        fn add_methods<M>(methods: &mut M)
        where
            M: UserDataMethods<Self>,
        {
            methods.add_method("play", |_, cartridge, ()| Ok(cartridge.play()));
        }
    }

    pub fn cartridge_loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
        let globals = lua.globals();
        let pick = lua.create_function(|_, ()| Ok(Cartridge::pick()))?;
        let tbl = lua.create_table()?;
        tbl.set("pick", pick)?;
        globals.set("cartridge", tbl)?;
        Ok(lua
            .load("return cartridge")
            .set_name(name)
            .set_environment(env)
            .into_function()?)
    }

    pub fn lua_loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
        // Mock lua loader
        Ok(lua
            .load("return function() return 'lua result' end")
            .set_name(name)
            .set_environment(env)
            .into_function()?)
    }
}

#[test]
fn test_multiple_loaders() {
    use meka_config_macros::loader_registry_from_cargo_manifest;
    use mlua::{Function, Lua, ObjectLike, Table};
    use mlua_module_manifest::{Manifest, Module, ModuleNamedText};
    use std::borrow::Cow;
    use std::collections::HashMap;

    const MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT: &str =
        "fennel-src should return exactly one NamedText module";

    let loader_registry: HashMap<
        Cow<'static, str>,
        fn(&Lua, Table, &str) -> mlua::Result<Function>,
    > = loader_registry_from_cargo_manifest!();

    // Verify all loaders were collected
    assert_eq!(loader_registry.len(), 3);
    assert!(loader_registry.contains_key("fennel-src"));
    assert!(loader_registry.contains_key("lua-src"));
    assert!(loader_registry.contains_key("n64"));

    let lua = Lua::new();

    // Test fennel-src loader
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

    // Test lua-src loader
    let loader = loader_registry
        .get("lua-src")
        .expect("Unexpectedly couldn't find lua-src loader in loader registry");
    let result_fn = loader(&lua, lua.globals(), "lua-src")
        .expect("Calling test_loaders::lua_loader function unexpectedly failed");
    let lua_fn: Function = result_fn
        .call(())
        .expect("Calling test_loaders::lua_loader function in Lua context unexpectedly failed");
    let lua_result: String = lua_fn.call(()).expect("Calling simple function returned by test_loaders::lua_loader function in Lua context unexpectedly failed");
    assert_eq!(lua_result, "lua result");

    // Test n64 loader
    let loader = loader_registry
        .get("n64")
        .expect("Unexpectedly couldn't find n64 loader in loader registry");
    let result_fn = loader(&lua, lua.globals(), "n64")
        .expect("Calling test_loaders::cartridge_loader function unexpectedly failed");
    let cartridge_table: Table = result_fn.call(()).expect(
        "Calling test_loaders::cartridge_loader function in Lua context unexpectedly failed",
    );
    assert!(
        cartridge_table
            .contains_key("pick")
            .expect("Calling Table.contains_key unexpectedly failed")
    );
}
