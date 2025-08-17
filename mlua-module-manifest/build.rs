///! Enforce only of one of mlua-external, mlua-module or mlua-vendored be chosen via Cargo
///! feature.

#[allow(dead_code)]
const CONFLICTING_CARGO_MANIFEST_FEATURE_MLUA: &str = "One, and only one, of mlua-external, mlua-module, or mlua-vendored must be specified as feature in Cargo manifest.";
#[allow(dead_code)]
const LUAU_MODULE_MODE_REQUESTED: &str = "Luau doesn't support loading Lua C modules.";

fn main() {
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
    #[cfg(all(
        feature = "mlua-module",
        any(
            feature = "mlua-luau",
            feature = "mlua-luau-jit",
            feature = "mlua-luau-vector4"
        )
    ))]
    panic!("{}", LUAU_MODULE_MODE_REQUESTED);
}
