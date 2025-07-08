use fennel_compile::Compile;
use fennel_mount::Mount;
use fennel_searcher::AddSearcher;
use io_cat::CatKind;
use meka_types::{CatCow, CatCowMap};
use mlua::{Lua, LuaOptions, StdLib, Table, Value};
use serial_test::serial;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

const ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT: &str =
    "Unexpectedly couldn't access $CARGO_MANIFEST_DIR environment variable";

#[test]
#[serial(lime)]
fn add_path_searcher_fnl_works() {
    let mut lime = HashMap::new();
    lime.insert(
        Cow::from("lime.color"),
        PathBuf::new()
            .join(env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT))
            .join("tests")
            .join("fixtures")
            .join("lime")
            .join("color.fnl"),
    );

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    lua.mount_fennel().unwrap();
    lua.add_path_searcher_fnl(lime).unwrap();
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!(&color, "green");
}

#[test]
#[serial(lime)]
fn module_reloading_works() {
    let name = Cow::from("lime.color");
    let path = PathBuf::new()
        .join(env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT))
        .join("tests")
        .join("fixtures")
        .join("lime")
        .join("color.fnl");
    let mut lime = HashMap::new();
    lime.insert(name.clone(), path.clone());

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    lua.mount_fennel().unwrap();

    // Add searcher for lime module on disk, and read from it.
    lua.add_path_searcher_fnl(lime).unwrap();
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!("green", color);

    // Twice.
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!("green", color);

    // Modify lime module on disk.
    let mut out = File::create(path.clone()).expect("Could not create lime module on disk");
    write!(out, ":snot\n").expect("Could not modify lime module on disk");

    // Thrice. Should still be unchanged due to caching.
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!("green", color);

    // Remove lime module from Lua's `package.loaded` cache to facilitate reload.
    let globals = lua.globals();
    let loaded: Table = globals
        .get::<Table>("package")
        .unwrap()
        .get("loaded")
        .unwrap();
    loaded.set(name.as_ref(), Value::Nil).unwrap();

    // Re-read from lime module on disk.
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!("snot", color);

    // Twice.
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!("snot", color);

    // Revert changes to lime module on disk.
    let mut out = File::create(path).expect("Could not create lime module on disk");
    write!(out, ":green\n").expect("Could not modify lime module on disk");

    // Clear cache again.
    let globals = lua.globals();
    let loaded: Table = globals
        .get::<Table>("package")
        .unwrap()
        .get("loaded")
        .unwrap();
    loaded.set(name.as_ref(), Value::Nil).unwrap();

    // Ensure changes have been successfully reverted.
    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();

    assert_eq!("green", color);
}

#[test]
fn add_searcher_fnl_macros_works() {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    // Mock macros.
    let mut macro_searcher = HashMap::new();
    let macro_name = Cow::from("fruit.macros");
    let macro_content = r#"
    (fn fruit [name traits]
      `(local ,name ,traits))
    {: fruit}
    "#;
    let macro_content = Cow::from(macro_content);
    macro_searcher.insert(macro_name, macro_content);

    // Mock Fennel program utilizing macros.
    let content = r#"
    (import-macros {: fruit} :fruit.macros)
    (fruit pear {:color :green
                 :shape :teardrop
                 :size :small})
    {: pear}
    "#;

    lua.mount_fennel().expect("mount_fennel");
    lua.add_searcher_fnl_macros(macro_searcher)
        .expect("add_searcher_fnl_macros");
    let content = lua
        .compile_fennel_string(content)
        .expect("compile_fennel_string");

    let orchard: Table = lua.load(&content).eval().expect("eval");

    let pear: Table = orchard.get("pear").unwrap();

    let color: String = pear.get("color").unwrap();
    assert_eq!("green", color);

    let shape: String = pear.get("shape").unwrap();
    assert_eq!("teardrop", shape);

    let size: String = pear.get("size").unwrap();
    assert_eq!("small", size);
}

#[test]
#[serial(fruit)]
fn add_path_searcher_fnl_macros_works() {
    let mut hat = HashMap::new();
    hat.insert(
        // Intentionally index fruit/macros by name which does not correspond to the path.
        Cow::from("fruit.hat"),
        PathBuf::new()
            .join(env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT))
            .join("tests")
            .join("fixtures")
            .join("fruit")
            .join("macros.fnl"),
    );

    let mut stand = HashMap::new();
    stand.insert(
        // Intentionally index fruit/orchard by name which does not correspond to the path.
        Cow::from("fruit.stand"),
        PathBuf::new()
            .join(env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT))
            .join("tests")
            .join("fixtures")
            .join("fruit")
            .join("orchard.fnl"),
    );

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    lua.mount_fennel().expect("mount_fennel");
    lua.add_path_searcher_fnl_macros(hat)
        .expect("add_path_searcher_fnl_macros");
    lua.add_path_searcher_fnl(stand)
        .expect("add_path_searcher_fnl");

    let orchard: Table = lua
        .load(r#"return require("fruit.stand")"#)
        .eval()
        .expect(r#"require("fruit.stand")"#);

    let pear: Table = orchard.get("pear").unwrap();

    let color: String = pear.get("color").unwrap();
    assert_eq!("green", color);

    let shape: String = pear.get("shape").unwrap();
    assert_eq!("teardrop", shape);

    let size: String = pear.get("size").unwrap();
    assert_eq!("small", size);
}

#[test]
#[serial(fruit)]
fn macro_module_reloading_works() {
    // Create `HashMap` for macros module searcher.
    let name = Cow::from("fruit.macros");
    let path = PathBuf::new()
        .join(env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT))
        .join("tests")
        .join("fixtures")
        .join("fruit")
        .join("macros.fnl");
    let mut fruit_macros = HashMap::new();
    fruit_macros.insert(name.clone(), path.clone());

    // Mock module importing macros.
    let content = r#"(import-macros {: fruit} :fruit.macros)

    (fruit pear {:color :green
                 :shape :teardrop
                 :size :small})

    {: pear}"#;

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    // Make Fennel available for evaluating macros.
    lua.mount_fennel().expect("mount_fennel");

    // Add macro searcher for macros module on disk.
    lua.add_path_searcher_fnl_macros(fruit_macros).unwrap();

    // Test macros module.
    {
        let content: String = lua
            .compile_fennel_string(&content)
            .expect("compile_fennel_string");

        let orchard: Table = lua.load(&content).eval().unwrap();

        let pear: Table = orchard.get("pear").unwrap();

        let color: String = pear.get("color").unwrap();
        assert_eq!("green", color);

        let shape: String = pear.get("shape").unwrap();
        assert_eq!("teardrop", shape);

        let size: String = pear.get("size").unwrap();
        assert_eq!("small", size);
    }

    // Twice.
    {
        let content: String = lua
            .compile_fennel_string(&content)
            .expect("compile_fennel_string");

        let orchard: Table = lua.load(&content).eval().unwrap();

        let pear: Table = orchard.get("pear").unwrap();

        let color: String = pear.get("color").unwrap();
        assert_eq!("green", color);

        let shape: String = pear.get("shape").unwrap();
        assert_eq!("teardrop", shape);

        let size: String = pear.get("size").unwrap();
        assert_eq!("small", size);
    }

    // Save original macros module on disk prior to altering it.
    let mut original = String::new();
    let mut file = File::open(&path).expect("Could not open macros module on disk");
    file.read_to_string(&mut original)
        .expect("Could not read macros module on disk to string");

    // Modify macros module on disk.
    let mut out = File::create(path.clone()).expect("Could not create fruit macros module on disk");

    let altered = r#"
    (fn fruit [name traits]
      `(local ,name {:color :yellow
                     :shape :hexagon
                     :size :medium}))

    {: fruit}
    "#
    .trim();

    write!(out, "{}", altered).expect("Could not modify fruit macros module on disk");

    {
        // Thrice. Should still be unchanged due to caching.
        let content: String = lua
            .compile_fennel_string(&content)
            .expect("compile_fennel_string");

        let orchard: Table = lua.load(&content).eval().unwrap();

        let pear: Table = orchard.get("pear").unwrap();

        let color: String = pear.get("color").unwrap();
        assert_eq!("green", color);

        let shape: String = pear.get("shape").unwrap();
        assert_eq!("teardrop", shape);

        let size: String = pear.get("size").unwrap();
        assert_eq!("small", size);
    }

    // Remove macro module from Fennel's `fennel.macro-loaded` cache to facilitate reload.
    let fennel: Table = lua.load(r#"return require("fennel")"#).eval().unwrap();
    let macro_loaded: Table = fennel.get("macro-loaded").unwrap();
    macro_loaded.set(name.as_ref(), Value::Nil).unwrap();

    {
        // Test altered macros module.
        let content: String = lua
            .compile_fennel_string(&content)
            .expect("compile_fennel_string");

        let orchard: Table = lua.load(&content).eval().unwrap();

        let pear: Table = orchard.get("pear").unwrap();

        let color: String = pear.get("color").unwrap();
        assert_eq!("yellow", color);

        let shape: String = pear.get("shape").unwrap();
        assert_eq!("hexagon", shape);

        let size: String = pear.get("size").unwrap();
        assert_eq!("medium", size);
    }

    // Twice.
    {
        let content: String = lua
            .compile_fennel_string(&content)
            .expect("compile_fennel_string");

        let orchard: Table = lua.load(&content).eval().unwrap();

        let pear: Table = orchard.get("pear").unwrap();

        let color: String = pear.get("color").unwrap();
        assert_eq!("yellow", color);

        let shape: String = pear.get("shape").unwrap();
        assert_eq!("hexagon", shape);

        let size: String = pear.get("size").unwrap();
        assert_eq!("medium", size);
    }

    // Revert changes to macros module on disk.
    let mut out = File::create(path).expect("Could not create macros module on disk");
    write!(out, "{}", original).expect("Could not modify macros module on disk");

    // Clear cache again.
    let fennel: Table = lua.load(r#"return require("fennel")"#).eval().unwrap();
    let macro_loaded: Table = fennel.get("macro-loaded").unwrap();
    macro_loaded.set(name.as_ref(), Value::Nil).unwrap();

    // Ensure changes have been successfully reverted.
    let content: String = lua
        .compile_fennel_string(&content)
        .expect("compile_fennel_string");

    let orchard: Table = lua.load(&content).eval().unwrap();

    let pear: Table = orchard.get("pear").unwrap();

    let color: String = pear.get("color").unwrap();
    assert_eq!("green", color);

    let shape: String = pear.get("shape").unwrap();
    assert_eq!("teardrop", shape);

    let size: String = pear.get("size").unwrap();
    assert_eq!("small", size);
}

#[test]
#[serial(lime)]
fn add_cat_searcher_fnl_works() {
    let mut lime = CatCowMap::new();
    lime.insert(
        Cow::from("lime.color"),
        CatKind::Path(
            PathBuf::new()
                .join(
                    env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT),
                )
                .join("tests")
                .join("fixtures")
                .join("lime")
                .join("color.fnl"),
        ),
    );
    let lime = CatCow(lime);

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    lua.mount_fennel().unwrap();
    lua.add_cat_searcher_fnl(lime).unwrap();

    let color: String = lua.load(r#"return require("lime.color")"#).eval().unwrap();
    assert_eq!(&color, "green");
}

#[test]
#[serial(fruit)]
fn add_cat_searcher_fnl_macros_works() {
    let mut hat = CatCowMap::new();
    hat.insert(
        Cow::from("fruit.hat"),
        CatKind::Path(
            PathBuf::new()
                .join(
                    env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT),
                )
                .join("tests")
                .join("fixtures")
                .join("fruit")
                .join("macros.fnl"),
        ),
    );
    let hat = CatCow(hat);

    let mut stand = CatCowMap::new();
    stand.insert(
        Cow::from("fruit.stand"),
        CatKind::Path(
            PathBuf::new()
                .join(
                    env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT),
                )
                .join("tests")
                .join("fixtures")
                .join("fruit")
                .join("orchard.fnl"),
        ),
    );
    let stand = CatCow(stand);

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    lua.mount_fennel().expect("mount_fennel");

    lua.add_cat_searcher_fnl_macros(hat)
        .expect("add_path_searcher_fnl_macros");

    lua.add_cat_searcher_fnl(stand)
        .expect("add_path_searcher_fnl");

    let orchard: Table = lua
        .load(r#"return require("fruit.stand")"#)
        .eval()
        .expect(r#"require("fruit.stand")"#);

    let pear: Table = orchard.get("pear").unwrap();

    let color: String = pear.get("color").unwrap();
    assert_eq!("green", color);

    let shape: String = pear.get("shape").unwrap();
    assert_eq!("teardrop", shape);

    let size: String = pear.get("size").unwrap();
    assert_eq!("small", size);
}
