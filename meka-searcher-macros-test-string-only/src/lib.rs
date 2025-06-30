use meka_searcher_macros::meka_searcher;

#[test]
fn string_only_works() {
    meka_searcher!("test_component");
    assert!(true);
}
