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

#[test]
fn test_simple_fennel_config() {
    use meka_config::Config;
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};

    let module = r#"(local meka (require :meka))
(local manifest meka.manifest)
(manifest.new {:name :test :text "return {}" :type :lua})"#;
    let module = ModuleNamedText::new("config", module, ModuleFileType::Fennel)
        .expect("Failed to create module");
    let module = Module::NamedText(module);

    let config = Config::new(module, None);
    assert!(config.is_ok(), "Failed to create config: {:?}", config);

    let config = config.unwrap();
    assert_eq!(config.0.len(), 1);
    assert!(config.0.contains_key(""));
}

#[test]
fn test_multiple_manifests() {
    use meka_config::Config;
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};

    let module = r#"local meka = require("meka")
local manifest1 = meka.manifest.new({ name = "mod1", text = "return 1", type = "lua" })
local manifest2 = meka.manifest.new({ name = "mod2", text = "return 2", type = "lua" })
return { first = manifest1, second = manifest2 }"#;
    let module = ModuleNamedText::new("config", module, ModuleFileType::Lua)
        .expect("Failed to create module");
    let module = Module::NamedText(module);

    let config = Config::new(module, None);
    assert!(config.is_ok(), "Failed to create config: {:?}", config);

    let config = config.unwrap();
    assert_eq!(config.0.len(), 2);
    assert!(config.0.contains_key("first"));
    assert!(config.0.contains_key("second"));
}

#[test]
fn test_with_standard_loaders() {
    use meka_config::Config;
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};

    let module = r#"local fennel_src = require("fennel-src")
assert(fennel_src, "fennel-src not loaded")
return require("meka").manifest.new({ name = "test", text = "return true", type = "lua" })"#;
    let module = ModuleNamedText::new("config", module, ModuleFileType::Lua)
        .expect("Failed to create module");
    let module = Module::NamedText(module);
    let config = Config::new(module, None);
    assert!(config.is_ok(), "Failed to create config: {:?}", config);
}

#[test]
fn test_with_custom_loader_paths() {
    use meka_config::Config;
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};

    let module = r#"return require("meka").manifest.new({ name = "test", text = "return true", type = "lua" })"#;
    let module = ModuleNamedText::new("config", module, ModuleFileType::Lua)
        .expect("Failed to create module");
    let module = Module::NamedText(module);

    // Provide additional loader paths
    let additional_loader_paths = vec![
        // Redundant since this is added automatically, but test override.
        ("fennel-src".to_string(), "fennel_src::loader".to_string()),
    ];

    let config = Config::new(module, Some(additional_loader_paths));
    assert!(config.is_ok(), "Failed to create config: {:?}", config);
}
