use meka_searcher_macros::meka_include;

#[test]
fn empty_macro_works() {
    meka_include!();
    assert!(true);
}
