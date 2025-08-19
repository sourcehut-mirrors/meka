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
