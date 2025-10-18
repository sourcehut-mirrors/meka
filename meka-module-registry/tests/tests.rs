#[test]
fn test_lookup_known_loader() {
    use meka_module_registry::lookup_loader;
    assert!(lookup_loader("fennel_src::loader").is_some());
    assert!(lookup_loader("meka::loader").is_some());
}

#[test]
fn test_lookup_unknown_loader() {
    use meka_module_registry::lookup_loader;
    assert!(lookup_loader("unknown::loader").is_none());
}

#[test]
fn test_build_registry_success() {
    use meka_module_registry::build_loader_registry;

    let paths = vec![
        ("fnl".to_string(), "fennel_src::loader".to_string()),
        ("meka".to_string(), "meka::loader".to_string()),
    ];

    let result = build_loader_registry(paths);
    assert!(result.is_ok());

    let registry = result.unwrap();
    assert_eq!(registry.len(), 2);
    assert!(registry.contains_key("fnl"));
    assert!(registry.contains_key("meka"));
}

#[test]
fn test_build_registry_with_unknown() {
    use meka_module_registry::build_loader_registry;

    let paths = vec![
        ("good".to_string(), "fennel_src::loader".to_string()),
        ("bad".to_string(), "unknown::loader".to_string()),
    ];

    let result = build_loader_registry(paths);
    assert!(result.is_err());

    let unknown = result.unwrap_err();
    assert_eq!(unknown.len(), 1);
    assert!(unknown[0].contains("bad -> unknown::loader"));
}

#[test]
fn test_available_loaders() {
    use meka_module_registry::available_loaders;

    let loaders = available_loaders();
    assert!(loaders.contains(&"fennel_src::loader"));
    assert!(loaders.contains(&"meka::loader"));
}

#[test]
fn test_fennel_src_loader() {
    use meka_module_registry::build_loader_registry;
    use mlua::{Function, Lua, ObjectLike, Table};
    use mlua_module_manifest::{Manifest, Module, ModuleNamedText};

    const MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT: &str =
        "fennel-src should return exactly one NamedText module";

    let lua = Lua::new();
    let paths = vec![("fennel-src".to_string(), "fennel_src::loader".to_string())];
    let loader_registry = build_loader_registry(paths).unwrap();
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
}
