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

fn main() {
    // Enforce one version of Fennel be chosen via Cargo feature.
    #[cfg(not(any(feature = "fennel100", feature = "fennel153")))]
    panic!("{}", MISSING_CARGO_MANIFEST_FEATURE_FENNEL);

    #[cfg(any(windows, unix))]
    println!("cargo:rust-cfg=host_family={}", HOST_FAMILY);
}
