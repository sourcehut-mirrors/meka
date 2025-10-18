use fennel_mount::Mount;
use io_cat::CatKind;
use meka_searcher::{AddMekaSearcher, MekaSearcher, RuntimeRead};
use meka_types::{CatCow, CatCowMap};
use mlua::Lua;
use std::borrow::Cow;
use std::convert::From;
use std::env;
use std::path::PathBuf;

const ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT: &str =
    "Unexpectedly couldn't access $CARGO_MANIFEST_DIR environment variable";

#[test]
fn add_meka_searcher_runtime_works() {
    let runtime_read = {
        let cargo_manifest_dir =
            env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT);

        let mut fnl = CatCowMap::new();
        fnl.insert(
            Cow::from("fruit.orchard"),
            CatKind::Path(
                PathBuf::new()
                    .join(&cargo_manifest_dir)
                    .join("tests")
                    .join("fixtures")
                    .join("fruit")
                    .join("orchard.fnl"),
            ),
        );
        fnl.insert(
            Cow::from("lime.color"),
            CatKind::Path(
                PathBuf::new()
                    .join(&cargo_manifest_dir)
                    .join("tests")
                    .join("fixtures")
                    .join("lime")
                    .join("color.fnl"),
            ),
        );
        let fnl = CatCow(fnl);
        let fnl = Some(fnl);

        let mut fnl_macros = CatCowMap::new();
        fnl_macros.insert(
            Cow::from("fruit.hat"),
            CatKind::Path(
                PathBuf::new()
                    .join(&cargo_manifest_dir)
                    .join("tests")
                    .join("fixtures")
                    .join("fruit")
                    .join("macros.fnl"),
            ),
        );
        let fnl_macros = CatCow(fnl_macros);
        let fnl_macros = Some(fnl_macros);

        let mut lua = CatCowMap::new();
        lua.insert(
            Cow::from("lime.time"),
            CatKind::Path(
                PathBuf::new()
                    .join(&cargo_manifest_dir)
                    .join("tests")
                    .join("fixtures")
                    .join("lime")
                    .join("time.lua"),
            ),
        );
        let lua = CatCow(lua);
        let lua = Some(lua);

        RuntimeRead {
            fnl,
            fnl_macros,
            lua,
        }
    };

    let meka_searcher = MekaSearcher::RuntimeRead(runtime_read);

    let lua = Lua::new();

    lua.mount_fennel().unwrap();
    lua.add_meka_searcher(meka_searcher)
        .expect("Unexpectedly couldn't add MekaSearcher");

    let color: String = lua
        .load(r#"return require("lime.color")"#)
        .eval()
        .expect("Unexpectly failed to eval lime.color Lua content");
    assert_eq!(&color, "green");

    let shape: String = lua
        .load(r#"return require("fruit.orchard").pear.shape"#)
        .eval()
        .expect("Unexpectly failed to eval fruit.orchard.pear.shape Lua content");
    assert_eq!(&shape, "teardrop");

    let time: String = lua
        .load(r#"return require("lime.time")"#)
        .eval()
        .expect("Unexpectly failed to eval lime.time Lua content");
    assert_eq!(&time, "The time is now 1 PM.");
}
