use meka_searcher_macros::meka_searcher;

#[test]
fn empty_macro_works() {
    meka_searcher!();
    assert!(true);
}
