use mlua::{Lua, LuaOptions, StdLib, Value};

#[test]
fn fennel_view_works() {
    use fennel_mount::Mount;
    use fennel_utils::FennelView;

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    lua.mount_fennel().unwrap();

    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    let got = lua.fennel_view(val).unwrap();
    assert_eq!(&got, r#""mlua""#)
}
