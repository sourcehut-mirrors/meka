use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

///! Keep this file synced with `fennel-src/build.rs`.

pub const FILE_OPEN_EXPECT: &str = "Unexpectedly failed to open file";
pub const FILE_READ_TO_STRING_EXPECT: &str = "Unexpectedly failed to read opened file to string";
pub const GPGRV_KEYRING_APPEND_KEYS_EXPECT: &str =
    "Unexpectedly failed to instantiate gpgrv PGP keyring";
pub const SEMVER_PARSE_EXPECT: &str = "Unexpectedly failed to parse pre-checked semver";

/// Verify Fennel release PGP signature.
pub fn verify_fennel<P>(version: &str, fnl_path: P, asc_path: P) -> bool
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
pub fn get_signing_key(version: &str) -> String {
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
pub fn comptime_root() -> PathBuf {
    PathBuf::new().join(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn fennel_works() {
    use std::env;
    use std::fs::{File, remove_file};
    use std::io::Write;

    const FENNEL: &str = fennel_src::FENNEL160;
    const FENNEL_PATH: &str = fennel_src::FENNEL160_PATH;
    const FENNEL_ASC_PATH: &str = fennel_src::FENNEL160_ASC_PATH;
    const FENNEL_VERSION: &str = fennel_src::FENNEL160_VERSION;

    const FILE_CREATE_EXPECT: &str = "Unexpectedly failed to create file";
    const FILE_WRITE_EXPECT: &str = "Unexpectedly failed to write file";
    const FS_REMOVE_FILE_EXPECT: &str = "Unexpectedly failed to remove file";

    assert!(Path::new(FENNEL_PATH).is_absolute());
    assert!(Path::new(FENNEL_ASC_PATH).is_absolute());

    let version = "1.6.0";
    assert!(FENNEL_VERSION == version);

    // `wc fennel-1.6.0.lua | awk '{print $3}'`
    let wc = 302186;
    assert!(FENNEL.len() == wc);

    let path = PathBuf::new()
        .join(env!("OUT_DIR"))
        .join(format!("fennel-{}.lua", version));

    let mut file = File::create(&path).expect(FILE_CREATE_EXPECT);
    write!(file, "{}", FENNEL).expect(FILE_WRITE_EXPECT);

    assert!(verify_fennel(version, FENNEL_PATH, FENNEL_ASC_PATH));

    remove_file(path).expect(FS_REMOVE_FILE_EXPECT);
}

#[test]
fn lua_works() {
    use mlua::{Function, Lua, LuaOptions, StdLib, Table};
    use mlua_module_manifest::{Manifest, Module, ModuleNamedText};
    use mlua_searcher::AddSearcher;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::convert::From;

    const LUA_MODULE_NAME: &str = "fennel-src";
    const MOUNT_FENNEL_SRC_EXPECT: &str = "mount_fennel_src";
    const LUA_REQUIRE_FENNEL_SRC_EXPECT: &str = r#"require("fennel-src")"#;
    const LUA_REQUIRE_FENNEL_VERSION_EXPECT: &str = r#"require("fennel-src").version"#;
    const MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT: &str =
        "fennel-src should return exactly one NamedText module";

    trait Mount {
        /// Add `fennel-src` release table via the `mlua-searcher` crate.
        fn mount_fennel_src(&self) -> mlua::Result<()>;
    }

    impl Mount for Lua {
        fn mount_fennel_src(&self) -> mlua::Result<()> {
            let mut map: HashMap<
                Cow<'static, str>,
                fn(&Lua, Table, &str) -> mlua::Result<Function>,
            > = HashMap::new();
            map.insert(Cow::from(LUA_MODULE_NAME), fennel_src::loader);
            self.add_function_searcher(map)?;
            Ok(())
        }
    }

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.mount_fennel_src().expect(MOUNT_FENNEL_SRC_EXPECT);
    let manifest: Manifest = lua
        .load(r#"require("fennel-src")()"#)
        .eval()
        .expect(LUA_REQUIRE_FENNEL_SRC_EXPECT);

    let named_text: ModuleNamedText =
        if let [Module::NamedText(named_text)] = manifest.modules.as_slice() {
            named_text.clone()
        } else {
            panic!("{}", MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT);
        };

    let mut modules: HashMap<Cow<'static, str>, Cow<'static, str>> = HashMap::new();
    modules.insert(named_text.name, named_text.text);

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.add_searcher(modules)
        .expect(LUA_REQUIRE_FENNEL_SRC_EXPECT);
    let version: String = lua
        .load(r#"require("fennel").version"#)
        .eval()
        .expect(LUA_REQUIRE_FENNEL_VERSION_EXPECT);

    assert_eq!(version, "1.6.0");

    // Repeat test now passing 'as' option to fennel-src loader function.
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.mount_fennel_src().expect(MOUNT_FENNEL_SRC_EXPECT);
    let manifest: Manifest = lua
        .load(r#"require("fennel-src")({as = "fffennnelll"})"#)
        .eval()
        .expect(LUA_REQUIRE_FENNEL_SRC_EXPECT);

    let named_text: ModuleNamedText =
        if let [Module::NamedText(named_text)] = manifest.modules.as_slice() {
            named_text.clone()
        } else {
            panic!("{}", MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT);
        };

    let mut modules: HashMap<Cow<'static, str>, Cow<'static, str>> = HashMap::new();
    modules.insert(named_text.name, named_text.text);

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.add_searcher(modules)
        .expect(LUA_REQUIRE_FENNEL_SRC_EXPECT);
    let version: String = lua
        .load(r#"require("fffennnelll").version"#)
        .eval()
        .expect(LUA_REQUIRE_FENNEL_VERSION_EXPECT);

    assert_eq!(version, "1.6.0");

    // Repeat test now passing 'version' and 'as' options to fennel-src loader function.
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.mount_fennel_src().expect(MOUNT_FENNEL_SRC_EXPECT);
    let manifest: Manifest = lua
        .load(r#"require("fennel-src")({version = "1.6.0", as = "fffennnelll"})"#)
        .eval()
        .expect(LUA_REQUIRE_FENNEL_SRC_EXPECT);

    let named_text: ModuleNamedText =
        if let [Module::NamedText(named_text)] = manifest.modules.as_slice() {
            named_text.clone()
        } else {
            panic!("{}", MANIFEST_MODULES_MODULE_NAMED_TEXT_EXPECT);
        };

    let mut modules: HashMap<Cow<'static, str>, Cow<'static, str>> = HashMap::new();
    modules.insert(named_text.name, named_text.text);

    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    lua.add_searcher(modules)
        .expect(LUA_REQUIRE_FENNEL_SRC_EXPECT);
    let version: String = lua
        .load(r#"require("fffennnelll").version"#)
        .eval()
        .expect(LUA_REQUIRE_FENNEL_VERSION_EXPECT);

    assert_eq!(version, "1.6.0");
}
