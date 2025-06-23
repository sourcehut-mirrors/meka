use meka_config_macros::loader_registry_from_cargo_manifest;

#[test]
fn test_empty_metadata() {
    let loader_registry = loader_registry_from_cargo_manifest!();
    assert_eq!(loader_registry.len(), 0);
    assert!(loader_registry.is_empty());
}
