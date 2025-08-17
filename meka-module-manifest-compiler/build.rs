///! Enforce only of one of mlua-external or mlua-vendored be chosen via Cargo feature.

#[allow(dead_code)]
const CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA: &str = "One, and only one, of mlua-external or mlua-vendored must be specified as feature in Cargo manifest.";

fn main() {
    #[cfg(not(any(
        feature = "mlua-external",
        feature = "mlua-vendored"
    )))]
    panic!("{}", CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA);
    #[cfg(all(feature = "mlua-external", feature = "mlua-vendored"))]
    panic!("{}", CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA);
}
