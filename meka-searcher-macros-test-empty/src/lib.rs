#[test]
fn empty_macro_works() {
    use meka_searcher_macros::meka_searcher;
    meka_searcher!();
    assert!(true);
}
