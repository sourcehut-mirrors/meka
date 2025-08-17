///! Enforce one version of Fennel be chosen via Cargo feature, and enforce only of one of
///! mlua-external, mlua-module or mlua-vendored be chosen via Cargo feature. Additionally,
///! build meka-module-manifest-compiler if mlua-module feature is active.
use cfg_if::cfg_if;

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
            use std::env;
            use std::path::Path;
            use std::process::Command;

            const CARGO_MANIFEST_DIR_EXPECT: &str = "Failed to find Cargo manifest directory";
            const CARGO_MANIFEST_DIR_PARENT_EXPECT: &str = "Failed to find Cargo workspace root";
            const CARGO_BUILD_EXPECT: &str = "Failed to build meka-module-manifest-compiler";

            // Luau purposefully lacks support for loading Lua C modules.
            #[cfg(any(
                feature = "mlua-luau",
                feature = "mlua-luau-jit",
                feature = "mlua-luau-vector4"
            ))]
            panic!("{}", LUAU_MODULE_MODE_REQUESTED);

            println!(concat!("cargo:rerun-if-changed=../meka-module-manifest-compiler/"));

            let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").expect(CARGO_MANIFEST_DIR_EXPECT);
            let workspace_root = Path::new(&cargo_manifest_dir).parent().expect(CARGO_MANIFEST_DIR_PARENT_EXPECT);

            // Compile meka-module-manifest-compiler with Lua matching active feature selection.
            #[cfg(all(feature = "mlua-external", feature = "mlua-lua54"))]
            let features = "mlua-lua54";
            #[cfg(all(feature = "mlua-external", feature = "mlua-lua53"))]
            let features = "mlua-lua53";
            #[cfg(all(feature = "mlua-external", feature = "mlua-lua52"))]
            let features = "mlua-lua52";
            #[cfg(all(feature = "mlua-external", feature = "mlua-lua51"))]
            let features = "mlua-lua51";
            #[cfg(all(feature = "mlua-external", feature = "mlua-luajit"))]
            let features = "mlua-luajit";
            #[cfg(all(feature = "mlua-external", feature = "mlua-luajit52"))]
            let features = "mlua-luajit52";
            #[cfg(feature = "mlua-lua54")]
            let features = "mlua-lua54,mlua-vendored";
            #[cfg(feature = "mlua-lua53")]
            let features = "mlua-lua53,mlua-vendored";
            #[cfg(feature = "mlua-lua52")]
            let features = "mlua-lua52,mlua-vendored";
            #[cfg(feature = "mlua-lua51")]
            let features = "mlua-lua51,mlua-vendored";
            #[cfg(feature = "mlua-luajit")]
            let features = "mlua-luajit,mlua-vendored";
            #[cfg(feature = "mlua-luajit52")]
            let features = "mlua-luajit52,mlua-vendored";

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
}
