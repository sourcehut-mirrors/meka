#[test]
fn mount_works() {
    use fennel_mount::Mount;
    use mlua::{Lua, LuaOptions, StdLib, Table};

    const MOUNT_FENNEL_EXPECT: &str = "mount_fennel";

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    #[allow(unused_assignments)]
    let mut expected_searchers_len = 0;

    let globals = lua.globals();

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");

    expected_searchers_len = searchers_len + 1;

    lua.mount_fennel().expect(MOUNT_FENNEL_EXPECT);

    let version: String = lua
        .load(r#"return require("fennel").version"#)
        .eval()
        .unwrap();
    assert_eq!(version, "1.6.0");

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");
    assert_eq!(searchers_len, expected_searchers_len);

    lua.mount_fennel().expect(MOUNT_FENNEL_EXPECT);

    let version: String = lua
        .load(r#"return require("fennel").version"#)
        .eval()
        .unwrap();
    assert_eq!(version, "1.6.0");

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");
    assert_eq!(searchers_len, expected_searchers_len);
}
