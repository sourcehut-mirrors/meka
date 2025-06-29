use meka_searcher_macros::meka_include;

#[test]
fn string_only_works() {
    meka_include!("test_component");
    assert!(true);
}
