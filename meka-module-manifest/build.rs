///! Enforce one version of Fennel be chosen via Cargo feature, and enforce only of one of
///! mlua-external, mlua-module or mlua-vendored be chosen via Cargo feature.

#[allow(dead_code)]
const MISSING_CARGO_MANIFEST_FEATURE_FENNEL: &str =
    "One Fennel version must be specified as feature in Cargo manifest.";
#[allow(dead_code)]
const CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA: &str = "One of mlua-external, mlua-module, or mlua-vendored must be specified as feature in Cargo manifest.";

fn main() {
    #[cfg(not(any(feature = "fennel100", feature = "fennel153")))]
    panic!("{}", MISSING_CARGO_MANIFEST_FEATURE_FENNEL);
    #[cfg(not(any(
        feature = "mlua-external",
        feature = "mlua-module",
        feature = "mlua-vendored"
    )))]
    panic!("{}", CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA);
    #[cfg(all(feature = "mlua-external", feature = "mlua-module"))]
    panic!("{}", CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA);
    #[cfg(all(feature = "mlua-external", feature = "mlua-vendored"))]
    panic!("{}", CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA);
    #[cfg(all(feature = "mlua-module", feature = "mlua-vendored"))]
    panic!("{}", CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA);
}
