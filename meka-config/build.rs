///! Enforce one version of Fennel be chosen via Cargo feature, and enforce only of one of
///! mlua-external, mlua-module or mlua-vendored be chosen via Cargo feature.

// Workaround for cross-platform `include_str!` usage.
//
// Credit: https://github.com/rust-lang/rust/issues/75075#issuecomment-671370162
#[cfg(windows)]
const HOST_FAMILY: &str = "windows";
#[cfg(unix)]
const HOST_FAMILY: &str = "unix";

#[allow(dead_code)]
const MISSING_CARGO_MANIFEST_FEATURE_FENNEL: &str =
    "One Fennel version must be specified as feature in Cargo manifest.";
#[allow(dead_code)]
const CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA: &str = "One, and only one, of mlua-external, mlua-module, or mlua-vendored must be specified as feature in Cargo manifest.";
#[allow(dead_code)]
const LUAU_MODULE_MODE_REQUESTED: &str = "Luau doesn't support loading Lua C modules.";

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

    // Luau purposefully lacks support for loading Lua C modules.
    #[cfg(all(feature = "mlua-module", any(
        feature = "mlua-luau",
        feature = "mlua-luau-jit",
        feature = "mlua-luau-vector4"
    )))]
    panic!("{}", LUAU_MODULE_MODE_REQUESTED);

    #[cfg(any(windows, unix))]
    println!("cargo:rust-cfg=host_family={}", HOST_FAMILY);
}
