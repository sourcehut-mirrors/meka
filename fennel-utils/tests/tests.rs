use mlua::{Lua, Value};

#[test]
fn fennel_view_works() {
    use fennel_mount::Mount;
    use fennel_utils::FennelView;

    let lua = Lua::new();

    lua.mount_fennel().unwrap();

    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    let got = lua.fennel_view(val, None).unwrap();
    assert_eq!(&got, r#""mlua""#);

    let opts = lua.create_table().unwrap();
    opts.set("prefer-colon?", true).unwrap();
    let val: Value = lua.load(r#"return "mlua""#).eval().unwrap();
    let got = lua.fennel_view(val, Some(opts)).unwrap();
    assert_eq!(&got, ":mlua");
}

#[test]
fn insert_fennel_searcher_works() {
    use fennel_mount::Mount;
    use fennel_utils::InsertFennelSearcher;

    let lua = Lua::new();

    lua.mount_fennel().unwrap();

    assert!(lua.insert_fennel_searcher().is_ok());
}
