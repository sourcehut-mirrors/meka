use meka_searcher_macros::meka_include;

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
}

#[test]
fn map_only_works() {
    meka_include!({
        "fennel-src" => fennel_src::loader,
        "cartridge-src" => test_loaders::cartridge_loader
    });
    assert!(true);
}
