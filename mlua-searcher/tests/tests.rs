use meka_types::CatCowMap;
use mlua::{Function, Lua, Table, UserData, UserDataMethods, Value};
use mlua_searcher::AddSearcher;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn add_searcher_works() {
    // These should end up in the same hash slot.
    let name_owned = Cow::from("lume".to_string());
    let name_ref = Cow::from("lume");

    // `lume_ref` should overwrite `lume_owned`.
    let lume_owned = Cow::from(read_lume_to_string());
    let lume_ref = Cow::from(read_lume_to_str());

    let mut map = HashMap::new();
    map.insert(name_owned, lume_owned);
    map.insert(name_ref, lume_ref);

    let lua = Lua::new();

    lua.add_searcher(map).unwrap();
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello ref", hello);

    // Repeat the experiment, but with an additional overwrite.
    let name_owned = Cow::from("lume".to_string());
    let name_ref = Cow::from("lume");
    let lume_owned = Cow::from(read_lume_to_string());
    let lume_ref = Cow::from(read_lume_to_str());

    let mut map = HashMap::new();
    map.insert(name_owned, lume_owned);
    map.insert(name_ref, lume_ref);

    let name_owned = Cow::from("lume".to_string());
    let lume_owned = Cow::from(read_lume_to_string());
    map.insert(name_owned, lume_owned);

    let lua = Lua::new();

    lua.add_searcher(map).unwrap();
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello owned", hello);
}

#[test]
fn add_path_searcher_works() {
    let name = Cow::from("lume".to_string());
    let path = PathBuf::new()
        .join(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("fixtures")
        .join("lume.lua");
    let mut map = HashMap::new();
    map.insert(name, path);

    let lua = Lua::new();

    lua.add_path_searcher(map).unwrap();
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello lume", hello);
}

#[test]
fn module_reloading_works() {
    let name = Cow::from("lume".to_string());
    let path = PathBuf::new()
        .join(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("fixtures")
        .join("lume.lua");
    let mut map = HashMap::new();
    map.insert(name.clone(), path.clone());

    let lua = Lua::new();

    // Add searcher for lume module on disk, and read from it.
    lua.add_path_searcher(map).unwrap();
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello lume", hello);

    // Twice.
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello lume", hello);

    // Modify lume module on disk.
    let mut out = File::create(path.clone()).expect("Could not create Lume module on disk");
    write!(out, "{}\n", r#"return "hello again lume""#)
        .expect("Could not modify Lume module on disk");

    // Thrice. Should still be unchanged due to caching.
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello lume", hello);

    // Remove lume module from Lua's `package.loaded` cache to facilitate reload.
    let globals = lua.globals();
    let loaded: Table = globals
        .get::<Table>("package")
        .unwrap()
        .get("loaded")
        .unwrap();
    loaded.set(name.as_ref(), Value::Nil).unwrap();

    // Re-read from lume module on disk.
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello again lume", hello);

    // Twice.
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello again lume", hello);

    // Revert changes to lume module on disk.
    let mut out = File::create(path).expect("Could not create Lume module on disk");
    write!(out, "{}\n", r#"return "hello lume""#).expect("Could not modify Lume module on disk");

    // Clear cache again.
    let globals = lua.globals();
    let loaded: Table = globals
        .get::<Table>("package")
        .unwrap()
        .get("loaded")
        .unwrap();
    loaded.set(name.as_ref(), Value::Nil).unwrap();

    // Ensure changes have been successfully reverted.
    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();

    assert_eq!("hello lume", hello);
}

fn read_lume_to_string() -> String {
    r#"return "hello owned""#.to_string()
}

fn read_lume_to_str() -> &'static str {
    r#"return "hello ref""#
}

#[test]
fn add_closure_searcher_works() {
    let lua = Lua::new();

    let mut modules: HashMap<
        Cow<'static, str>,
        Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send>,
    > = HashMap::new();

    let instrument_loader: Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send> =
        Box::new(|lua, env, name| {
            let globals = lua.globals();
            let new = lua.create_function(|_, (name, sound): (String, String)| {
                Ok(Instrument::new(name, sound))
            })?;
            let tbl = lua.create_table()?;
            tbl.set("new", new)?;
            globals.set("instrument", tbl)?;
            Ok(lua
                .load("return instrument")
                .set_name(name)
                .set_environment(env)
                .into_function()?)
        });

    modules.insert(Cow::from("instrument".to_string()), instrument_loader);

    lua.add_closure_searcher(modules).unwrap();

    // Ensure global variable `instrument` is unset.
    let nil: String = lua.load("return type(instrument)").eval().unwrap();
    assert_eq!(nil, "nil");

    let sound: String = lua
        .load(
            r#"local instrument = require("instrument")
               local ukulele = instrument.new("ukulele", "twang")
               return ukulele:play()"#,
        )
        .eval()
        .unwrap();

    assert_eq!(sound, "The ukulele goes twang");
}

struct Instrument {
    name: String,
    sound: String,
}

impl Instrument {
    pub fn new(name: String, sound: String) -> Self {
        Self { name, sound }
    }

    pub fn play(&self) -> String {
        format!("The {} goes {}", self.name, self.sound)
    }
}

impl UserData for Instrument {
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_method("play", |_, instrument, ()| Ok(instrument.play()));
    }
}

#[test]
fn add_function_searcher_works() {
    let lua = Lua::new();

    let mut modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>> =
        HashMap::new();

    modules.insert(Cow::from("cartridge".to_string()), cartridge_loader);

    lua.add_function_searcher(modules).unwrap();

    // Ensure global variable `cartridge` is unset.
    let nil: String = lua.load("return type(cartridge)").eval().unwrap();
    assert_eq!(nil, "nil");

    let title: String = lua
        .load(
            r#"local cartridge = require("cartridge")
               local smash = cartridge.pick()
               return smash:play()"#,
        )
        .eval()
        .unwrap();

    assert_eq!(title, "Super Smash Brothers 64");
}

struct Cartridge {
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

fn cartridge_loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
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

#[test]
fn add_cat_searcher_works() {
    let name = Cow::from("lume".to_string());
    let path = PathBuf::new()
        .join(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("fixtures")
        .join("lume.lua");
    let mut map: CatCowMap = CatCowMap::new();
    map.insert(name, Box::new(path));
    map.insert(Cow::from("loon"), Box::new(r#"return "hello loon""#));

    let lua = Lua::new();

    lua.add_cat_searcher(map).unwrap();

    let hello: String = lua.load(r#"return require("lume")"#).eval().unwrap();
    assert_eq!("hello lume", hello);

    let hello: String = lua.load(r#"return require("loon")"#).eval().unwrap();
    assert_eq!("hello loon", hello);
}
