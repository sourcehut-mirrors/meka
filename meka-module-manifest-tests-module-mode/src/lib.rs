#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_simple_fennel_compilation() {
    use meka_module_manifest::CompiledNamedTextManifest;
    use mlua_module_manifest::{ModuleFileType, ModuleNamedText, NamedTextManifest};
    use std::borrow::Cow;
    use std::convert::TryFrom;

    // Create a simple manifest with Fennel code
    let manifest = NamedTextManifest {
        docstring: Some(Cow::Borrowed("Test manifest")),
        modules: vec![ModuleNamedText {
            name: Cow::Borrowed("test-module"),
            text: Cow::Borrowed(r#"(fn hello [] "Hello from Fennel!")"#),
            file_type: ModuleFileType::Fennel,
        }],
    };

    // Test compilation via TryFrom.
    let result = CompiledNamedTextManifest::try_from(manifest);

    // This should work with ephemeral crate.
    assert!(
        result.is_ok(),
        "Fennel compilation should succeed: {:?}",
        result.err()
    );

    let compiled = result.unwrap();
    assert_eq!(compiled.modules.len(), 1);

    // Our function should be in compiled Lua.
    let lua_code = &compiled.modules[0].text;
    assert!(
        lua_code.contains("hello"),
        "Compiled Lua should contain function name"
    );
    assert!(
        lua_code.contains("Hello from Fennel!"),
        "Compiled Lua should contain string"
    );
}

#[test]
#[serial]
fn test_fennel_with_macros() {
    use meka_module_manifest::CompiledNamedTextManifest;
    use mlua_module_manifest::{ModuleFileType, ModuleNamedText, NamedTextManifest};
    use std::borrow::Cow;
    use std::convert::TryFrom;

    // Create manifest with macros and Fennel code that uses them.
    let manifest = NamedTextManifest {
        docstring: None,
        modules: vec![
            ModuleNamedText {
                name: Cow::Borrowed("macros"),
                text: Cow::Borrowed(r#"(fn twice [x] `(do ,x ,x)) {: twice}"#),
                file_type: ModuleFileType::FennelMacros,
            },
            ModuleNamedText {
                name: Cow::Borrowed("main"),
                text: Cow::Borrowed(
                    r#"(import-macros {: twice} :macros) (fn run [] (var x 0) (twice (set x (+ x 1))) x)"#,
                ),
                file_type: ModuleFileType::Fennel,
            },
        ],
    };

    let result = CompiledNamedTextManifest::try_from(manifest);
    assert!(
        result.is_ok(),
        "Compilation with macros should succeed: {:?}",
        result.err()
    );

    let compiled = result.unwrap();
    // Macro module should remain as FennelMacros.
    assert!(matches!(
        compiled.modules[0].file_type,
        ModuleFileType::FennelMacros
    ));
    // Main module should be compiled.
    assert!(compiled.modules[1].text.contains("run"));
}

#[test]
#[serial]
fn test_mixed_lua_fennel() {
    use meka_module_manifest::CompiledNamedTextManifest;
    use mlua_module_manifest::{ModuleFileType, ModuleNamedText, NamedTextManifest};
    use std::borrow::Cow;
    use std::convert::TryFrom;

    let manifest = NamedTextManifest {
        docstring: None,
        modules: vec![
            ModuleNamedText {
                name: Cow::Borrowed("lua-module"),
                text: Cow::Borrowed(r#"return { hello = "from Lua" }"#),
                file_type: ModuleFileType::Lua,
            },
            ModuleNamedText {
                name: Cow::Borrowed("fennel-module"),
                text: Cow::Borrowed(r#"{:hello "from Fennel"}"#),
                file_type: ModuleFileType::Fennel,
            },
        ],
    };

    let result = CompiledNamedTextManifest::try_from(manifest);
    assert!(result.is_ok());

    let compiled = result.unwrap();
    // Lua module should be unchanged
    assert_eq!(compiled.modules[0].text, r#"return { hello = "from Lua" }"#);
    // Fennel module should be compiled to Lua
    assert!(compiled.modules[1].text.contains("hello"));
}

#[test]
#[serial]
fn test_compilation_error_handling() {
    use meka_module_manifest::CompiledNamedTextManifest;
    use mlua_module_manifest::{ModuleFileType, ModuleNamedText, NamedTextManifest};
    use std::borrow::Cow;
    use std::convert::TryFrom;

    let manifest = NamedTextManifest {
        docstring: None,
        modules: vec![ModuleNamedText {
            name: Cow::Borrowed("bad-module"),
            text: Cow::Borrowed("(this is invalid fennel syntax"),
            file_type: ModuleFileType::Fennel,
        }],
    };

    let result = CompiledNamedTextManifest::try_from(manifest);
    assert!(result.is_err(), "Invalid Fennel should fail compilation");
}

/// Verify whole workflow works outside of proc macros.
#[test]
#[serial]
fn test_end_to_end_with_mlua_module() {
    use meka_module_manifest::CompiledNamedTextManifest;
    use meka_searcher::MekaSearcher;
    use mlua_module_manifest::{ModuleFileType, ModuleNamedText, NamedTextManifest};
    use std::borrow::Cow;
    use std::convert::TryFrom;

    let manifest = NamedTextManifest {
        docstring: Some(Cow::Borrowed("Integration test")),
        modules: vec![ModuleNamedText {
            name: Cow::Borrowed("utils"),
            text: Cow::Borrowed(
                r#"(fn add [a b] (+ a b)) (fn multiply [a b] (* a b)) {: add : multiply}"#,
            ),
            file_type: ModuleFileType::Fennel,
        }],
    };

    // This should work at runtime (not during proc macro expansion).
    let compiled = CompiledNamedTextManifest::try_from(manifest);
    assert!(compiled.is_ok());

    // Verify we can create MekaSearcher from it.
    let _searcher = MekaSearcher::from(compiled.unwrap());
    assert!(true);
}
