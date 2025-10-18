#[test]
fn compile_works() {
    use fennel_compile::Compile;
    use fennel_mount::Mount;
    use mlua::{Lua, Table};
    use std::env;
    use std::path::PathBuf;

    const ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT: &str =
        "Unexpectedly couldn't access $CARGO_MANIFEST_DIR environment variable";
    const MOUNT_FENNEL_EXPECT: &str = "mount_fennel";
    const COMPILE_FENNEL_STRING_EXPECT: &str = "compile_fennel_string";

    let lua = Lua::new();

    let fnl_str = "(print (+ 1 1))";
    let fnl_bytes = fnl_str.as_bytes();
    let lua_str = "return print((1 + 1))";

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

    let got = lua
        .compile_fennel_string(fnl_str)
        .expect(COMPILE_FENNEL_STRING_EXPECT);
    assert_eq!(got, lua_str);

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");
    assert_eq!(searchers_len, expected_searchers_len);

    lua.mount_fennel().expect(MOUNT_FENNEL_EXPECT);

    let got = lua
        .compile_fennel_bytes(fnl_bytes)
        .expect(COMPILE_FENNEL_STRING_EXPECT);
    assert_eq!(got, lua_str);

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");
    assert_eq!(searchers_len, expected_searchers_len);

    let fnl_path = PathBuf::new()
        .join(env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR_EXPECT))
        .join("tests")
        .join("fixtures")
        .join("basic.fnl");

    lua.mount_fennel().expect(MOUNT_FENNEL_EXPECT);

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");
    assert_eq!(searchers_len, expected_searchers_len);

    let got = lua
        .compile_fennel_file(fnl_path)
        .expect(COMPILE_FENNEL_STRING_EXPECT);
    assert_eq!(got, lua_str);

    let searchers_len = globals
        .get::<Table>("package")
        .expect("package")
        .get::<Table>("searchers")
        .expect("searchers")
        .len()
        .expect("len");
    assert_eq!(searchers_len, expected_searchers_len);
}
