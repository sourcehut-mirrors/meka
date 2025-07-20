use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value};
use std::path::MAIN_SEPARATOR;

#[test]
fn try_into_string_works() {
    use mlua_utils::TryIntoString;

    let lua = Lua::new();

    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    assert!(val.try_into_string().is_ok());
}

#[test]
fn extract_non_system_lua_paths_works() {
    let lua = Lua::new();
    assert!(mlua_utils::extract_non_system_lua_paths(&lua).is_ok());
}

#[test]
fn package_config_works() {
    let lua = Lua::new();
    assert!(mlua_utils::package_config(&lua).is_ok());
    let (dir_sep, path_sep, path_mark, exedir_mark, ignore_mark) =
        mlua_utils::package_config(&lua).unwrap();
    assert_eq!(dir_sep, MAIN_SEPARATOR.to_string());
    assert_eq!(path_sep, ";");
    assert_eq!(path_mark, "?");
    assert_eq!(exedir_mark, "!");
    assert_eq!(ignore_mark, "-");
}

#[test]
fn package_cpath_works() {
    let lua = Lua::new();
    assert!(mlua_utils::package_cpath(&lua).is_ok());
}

#[test]
fn package_path_works() {
    let lua = Lua::new();
    assert!(mlua_utils::package_path(&lua).is_ok());
}

#[test]
fn typename_works() {
    let lua = Lua::new();

    let val: Value = lua.load("return nil").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "nil");

    let val: Value = lua.load("return true").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "boolean");

    let val: Value = lua.load("return 1").eval().unwrap();
    #[cfg(any(feature = "mlua-lua53", feature = "mlua-lua54"))]
    assert_eq!(mlua_utils::typename(&val), "integer");
    #[cfg(not(any(feature = "mlua-lua53", feature = "mlua-lua54")))]
    assert_eq!(mlua_utils::typename(&val), "number");

    let val: Value = lua.load("return 1.0").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "number");

    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "string");

    let val: Value = lua.load("return {}").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "table");

    let val: Value = lua.load("return function() end").eval().unwrap();
    assert_eq!(mlua_utils::typename(&val), "function");
}

mod test_loaders {
    use mlua::{FromLua, Function, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};

    #[derive(Clone, Debug)]
    pub struct Cartridge {
        pub title: String,
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
            methods.add_meta_method(MetaMethod::ToString, |lua, cartridge, ()| {
                Ok(Value::String(
                    lua.create_string(format!("{:?}", cartridge))?,
                ))
            });
            methods.add_meta_method("__fennelview", |lua, cartridge, ()| {
                let title = &cartridge.title;
                let cartridge = lua.create_table()?;
                cartridge.push(lua.create_string(title)?)?;
                Ok(Value::Table(cartridge))
            });
        }
    }

    impl FromLua for Cartridge {
        fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
            match value {
                Value::UserData(ud) => {
                    let cartridge = ud.borrow::<Self>()?;
                    Ok(cartridge.clone())
                }
                _ => unreachable!(),
            }
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
}

#[test]
fn unrestrict_getmetatable_works() {
    use fennel_mount::Mount;
    use fennel_utils::FennelView;
    use test_loaders::Cartridge;

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.mount_fennel().unwrap();

    let cartridge: Function = test_loaders::cartridge_loader(&lua, lua.globals(), "cartridge")
        .expect("Calling test_loaders::cartridge_loader function unexpectedly failed");
    let cartridge: Table = cartridge.call(()).expect(
        "Calling test_loaders::cartridge_loader function in Lua context unexpectedly failed",
    );
    let cartridge: Function = cartridge
        .get("pick")
        .expect("Unexpectedly couldn't get pick method from cartridge table");
    let cartridge: Cartridge = cartridge
        .call(())
        .expect("cartridge table pick method unexpectedly failed");

    let cartridge_format = format!("{:?}", cartridge);
    assert_eq!(
        r#"Cartridge { title: "Super Smash Brothers 64" }"#,
        cartridge_format
    );

    let cartridge_tostring = {
        let tostring: Function = lua
            .globals()
            .get("tostring")
            .expect("Unexpectedly couldn't get tostring function");
        let cartridge: String = tostring
            .call(cartridge.clone())
            .expect("tostring function unexpectedly failed with Cartridge argument");
        cartridge
    };
    assert_eq!(
        r#"Cartridge { title: "Super Smash Brothers 64" }"#,
        cartridge_tostring
    );

    let primitive_fennel_view_restricted = {
        lua.fennel_view(Value::Boolean(true), None)
            .expect("fennel.view function unexpectedly failed with bool argument")
    };
    assert_eq!("true", primitive_fennel_view_restricted);

    let cartridge_fennel_view_restricted = {
        let cartridge = lua
            .create_userdata(cartridge.clone())
            .expect("Unexpectedly failed to create userdata from Cartridge");
        lua.fennel_view(Value::UserData(cartridge), None)
            .expect("fennel.view function unexpectedly failed with Cartridge argument")
    };
    assert_eq!(
        r#"#<Cartridge { title: "Super Smash Brothers 64" }>"#,
        cartridge_fennel_view_restricted
    );

    // before unrestrict, mlua returns bool (false)
    let getmetatable: Function = lua
        .globals()
        .get("getmetatable")
        .expect("Unexpectedly couldn't get getmetatable function");
    let getmetatable_cartridge: bool = getmetatable
        .call(cartridge.clone())
        .expect("getmetable(cartridge) unexpectedly failed");
    assert_eq!(false, getmetatable_cartridge);

    mlua_utils::unrestrict_getmetatable(&lua).expect("unrestrict_getmetatable unexpectedly failed");

    let getmetatable: Function = lua
        .globals()
        .get("getmetatable")
        .expect("Unexpectedly couldn't get getmetatable function");
    let getmetatable_bool: Value = getmetatable
        .call(Value::Boolean(true))
        .expect("getmetable(bool) unexpectedly failed");
    assert_eq!(mlua_utils::typename(&getmetatable_bool), "nil");

    let getmetatable: Function = lua
        .globals()
        .get("getmetatable")
        .expect("Unexpectedly couldn't get getmetatable function");
    let getmetatable_table: Value = getmetatable
        .call(Value::Table(lua.create_table().unwrap()))
        .expect("getmetable(table) unexpectedly failed");
    assert_eq!(getmetatable_table, Value::Nil);

    // test implementation of getmetatable override
    let primitive_fennel_view_unrestricted = {
        lua.fennel_view(Value::Boolean(true), None)
            .expect("fennel.view function unexpectedly failed with bool argument")
    };
    assert_eq!("true", primitive_fennel_view_unrestricted);

    // after unrestrict, getmetatable behaves as normal
    let getmetatable: Function = lua
        .globals()
        .get("getmetatable")
        .expect("Unexpectedly couldn't get getmetatable function");
    let getmetatable_cartridge: Value = getmetatable
        .call(cartridge.clone())
        .expect("getmetable(cartridge) unexpectedly failed");
    assert_eq!("table", mlua_utils::typename(&getmetatable_cartridge));

    let cartridge_fennel_view_unrestricted = {
        let cartridge = lua
            .create_userdata(cartridge)
            .expect("Unexpectedly failed to create userdata from Cartridge");
        lua.fennel_view(Value::UserData(cartridge), None)
            .expect("fennel.view function unexpectedly failed with Cartridge argument")
    };
    assert_eq!(
        "Super Smash Brothers 64",
        cartridge_fennel_view_unrestricted
    );
}
