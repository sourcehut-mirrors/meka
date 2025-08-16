#[test]
fn string_only_works() {
    use meka_searcher_macros::meka_searcher;
    let _ = meka_searcher!("test_component");
    assert!(true);
}
