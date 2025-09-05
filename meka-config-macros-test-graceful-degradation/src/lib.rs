#[test]
fn test_graceful_degradation() {
    use meka_config_macros::loader_registry_from_cargo_manifest;
    // Should return empty map, not panic
    let loader_registry = loader_registry_from_cargo_manifest!();
    assert!(loader_registry.is_empty());
}
