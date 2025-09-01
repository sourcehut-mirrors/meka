///! Enforce one version of Fennel be chosen via Cargo feature, and enforce only one of
///! mlua-external, mlua-module or mlua-vendored be chosen via Cargo feature. Additionally,
///! build meka-module-manifest-compiler if mlua-module feature is active.
use cfg_if::cfg_if;

// Workaround for cross-platform `include!` usage.
//
// Credit: https://github.com/rust-lang/rust/issues/75075#issuecomment-671370162
cfg_if! {
    if #[cfg(windows)] {
        const HOST_FAMILY: &str = "windows";
        macro_rules! path_separator {
            () => {
                r"\"
            };
        }
    } else if #[cfg(unix)] {
        const HOST_FAMILY: &str = "unix";
        macro_rules! path_separator {
            () => {
                r"/"
            };
        }
    }
}

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

    cfg_if! {
        if #[cfg(feature = "mlua-module")] {
            use std::path::Path;
            use std::process::Command;

            const CARGO_MANIFEST_DIR_PARENT_EXPECT: &str = "Failed to find Cargo workspace root";
            const CARGO_BUILD_EXPECT: &str = "Failed to build meka-module-manifest-compiler";

            // Luau purposefully lacks support for loading Lua C modules.
            #[cfg(any(
                feature = "mlua-luau",
                feature = "mlua-luau-jit",
                feature = "mlua-luau-vector4"
            ))]
            panic!("{}", LUAU_MODULE_MODE_REQUESTED);

            println!("cargo:rerun-if-changed=../meka-module-manifest-compiler/");
            println!("cargo:rerun-if-changed=src/include/features.rs");

            let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
            let workspace_root = Path::new(&cargo_manifest_dir).parent().expect(CARGO_MANIFEST_DIR_PARENT_EXPECT);

            // Compile meka-module-manifest-compiler with Lua matching active feature selection.
            let features: &str = include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                path_separator!(),
                "src",
                path_separator!(),
                "include",
                path_separator!(),
                "features.rs"
            ));

            Command::new("cargo")
                .arg("build")
                .arg("--release")
                .arg("--quiet")
                .args(["--package", "meka-module-manifest-compiler"])
                .args(["--features", features])
                .current_dir(workspace_root)
                .spawn()
                .expect(CARGO_BUILD_EXPECT);
        }
    }

    #[cfg(any(windows, unix))]
    println!("cargo:rust-cfg=host_family={}", HOST_FAMILY);
}
