use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

///! Verify Fennel release PGP signature.

// Workaround for cross-platform `include_str!` usage.
//
// Credit: https://github.com/rust-lang/rust/issues/75075#issuecomment-671370162
#[cfg(windows)]
const HOST_FAMILY: &str = "windows";
#[cfg(unix)]
const HOST_FAMILY: &str = "unix";

const BAD_PGP_SIGNATURE: &str = "Bad PGP signature";
const FILE_OPEN_EXPECT: &str = "Unexpectedly failed to open file";
const FILE_READ_TO_STRING_EXPECT: &str = "Unexpectedly failed to read opened file to string";
const GPGRV_KEYRING_APPEND_KEYS_EXPECT: &str =
    "Unexpectedly failed to instantiate gpgrv PGP keyring";
const SEMVER_PARSE_EXPECT: &str = "Unexpectedly failed to parse pre-checked semver";

#[allow(dead_code)]
const MISSING_CARGO_MANIFEST_FEATURE_FENNEL: &str =
    "One Fennel version must be specified as feature in Cargo manifest.";

fn main() {
    // Enforce one version of Fennel be chosen via Cargo feature.
    #[cfg(feature = "fennel100")]
    let version = "1.0.0";
    #[cfg(feature = "fennel153")]
    let version = "1.5.3";
    #[cfg(not(any(feature = "fennel100", feature = "fennel153")))]
    panic!("{}", MISSING_CARGO_MANIFEST_FEATURE_FENNEL);

    let dirname = format!("fennel-{}", version);
    let extension = "lua";
    let basename = format!("{}.{}", &dirname, extension);
    let fnl_path = comptime_root().join(&dirname).join(&basename);
    let asc_path = fnl_path.with_extension(format!("{}.asc", extension));

    if !verify_fennel(version, fnl_path, asc_path) {
        panic!("{}", BAD_PGP_SIGNATURE);
    }

    #[cfg(any(windows, unix))]
    println!("cargo:rust-cfg=host_family={}", HOST_FAMILY);
}

/// Verify Fennel release PGP signature.
fn verify_fennel<P>(version: &str, fnl_path: P, asc_path: P) -> bool
where
    P: AsRef<Path>,
{
    // Wrap signing key in `BufReader`.
    let key = get_signing_key(version);
    let key = BufReader::new(key.as_bytes());

    // Wrap release file in `BufReader`.
    let fnl = BufReader::new(File::open(fnl_path.as_ref()).expect(FILE_OPEN_EXPECT));

    // Wrap detached signature in `BufReader`.
    let asc = BufReader::new(File::open(asc_path.as_ref()).expect(FILE_OPEN_EXPECT));

    // Read in signing key manually.
    let mut keyring = gpgrv::Keyring::new();
    keyring
        .append_keys_from_armoured(key)
        .expect(GPGRV_KEYRING_APPEND_KEYS_EXPECT);

    match gpgrv::verify_detached(asc, fnl, &keyring) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Return PGP fingerprint with which to verify Fennel official release sources.
fn get_signing_key(version: &str) -> String {
    let version = semver::Version::parse(version).expect(SEMVER_PARSE_EXPECT);

    // Fennel releases are signed with 8F2C85FFC1EBC016A3B683DE8BD38C28CCFD2DA6 from
    // version 0.10.0 onward. Before that, 20242BACBBE95ADA22D0AFD7808A33D379C806C3 was
    // used.
    let path = if version >= semver::Version::parse("1.4.1").expect(SEMVER_PARSE_EXPECT) {
        comptime_root()
            .join("gpg")
            .join("technomancy")
            .join("9D13D9426A0814B3373CF5E3D8A8243577A7859F.asc")
    } else if version >= semver::Version::parse("0.10.0").expect(SEMVER_PARSE_EXPECT) {
        comptime_root()
            .join("gpg")
            .join("technomancy")
            .join("8F2C85FFC1EBC016A3B683DE8BD38C28CCFD2DA6.asc")
    } else {
        comptime_root()
            .join("gpg")
            .join("technomancy")
            .join("20242BACBBE95ADA22D0AFD7808A33D379C806C3.asc")
    };

    let mut key = String::new();
    let mut file = File::open(path).expect(FILE_OPEN_EXPECT);
    file.read_to_string(&mut key)
        .expect(FILE_READ_TO_STRING_EXPECT);
    key
}

/// Return the value of `$CARGO_MANIFEST_DIR` at the time of compiling `fennel-src`.
///
/// Particularly valuable for reading the Fennel release signing key into memory when
/// `fennel-src` is a transitive dependency. Attempting to read `$CARGO_MANIFEST_DIR`
/// at runtime here would prevent finding the release signing key.
fn comptime_root() -> PathBuf {
    PathBuf::new().join(env!("CARGO_MANIFEST_DIR"))
}
