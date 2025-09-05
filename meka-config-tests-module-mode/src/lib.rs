#[test]
fn test_loader_paths_from_manifest() {
    use meka_config_macros::loader_paths_from_cargo_manifest;

    // This should pick up `test-loader` from `package.metadata.meka.loaders`.
    let paths = loader_paths_from_cargo_manifest!();

    let has_test_loader = paths.iter().any(|(name, path)| {
        name == "test-loader" && path == "meka_config_tests_module_mode::test_loader"
    });

    assert!(
        has_test_loader,
        "Test loader not found in paths: {:?}",
        paths
    );
}

#[test]
fn test_simple_lua_config() {
    use meka_config::Config;
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};

    let module = ModuleNamedText::new(
        "config",
        r#"return require("meka").manifest.new({name = "test", text = "return {}", type = "lua"})"#,
        ModuleFileType::Lua,
    )
    .expect("Failed to create module");
    let module = Module::NamedText(module);

    let config = Config::new(module, None);
    assert!(config.is_ok(), "Failed to create config: {:?}", config);

    let config = config.unwrap();
    assert_eq!(config.0.len(), 1);
    assert!(config.0.contains_key(""));
}
