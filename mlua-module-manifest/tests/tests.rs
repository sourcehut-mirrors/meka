use mlua::{Function, Lua, Table};
use mlua_module_manifest::{
    Manifest, ManifestInitError, Module, ModuleFile, ModuleFileType, ModuleInitError,
    ModuleNamedFile, ModuleNamedText, Name,
};
use mlua_searcher::AddSearcher;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::From;

const ADD_FUNCTION_SEARCHER_EXPECT: &str = "lua.add_function_searcher() unexpectedly failed";

fn basic() -> Result<Manifest, ManifestInitError> {
    Ok(Manifest::new(
        Some(Cow::from("Basic example")),
        vec![
            Module::File(
                ModuleFile::new("path/to/file.fnl", None).map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::File(
                ModuleFile::new("path/to/another/file.fnl", Some(ModuleFileType::Fennel))
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::File(
                ModuleFile::new("another/path.lua", None).map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::File(
                ModuleFile::new("yet/another/path.lua", Some(ModuleFileType::Lua))
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::File(
                ModuleFile::new("path/to/macros.fnl", None)
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::File(
                ModuleFile::new("path/to/1337.fnl", Some(ModuleFileType::FennelMacros))
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::NamedFile(
                ModuleNamedFile::new("arbitrary", "protoss.fnl", None)
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::NamedFile(
                ModuleNamedFile::new("the.thing", "vw/thing.vw", Some(ModuleFileType::Fennel))
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::NamedText(
                ModuleNamedText::new("the.answer", "42", ModuleFileType::Fennel)
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::NamedText(
                ModuleNamedText::new("a.resposta", "return 42", ModuleFileType::Lua)
                    .map_err(|e| ModuleInitError::from(e))?,
            ),
        ],
    ))
}

const BASIC_LUA: &str = r#"local manifest = require("manifest")
return manifest.new("Basic example",
                    {path = "path/to/file.fnl"},
                    {path = "path/to/another/file.fnl", type = "fennel"},
                    {path = "another/path.lua"},
                    {path = "yet/another/path.lua", type = "lua"},
                    {path = "path/to/macros.fnl"},
                    {path = "path/to/1337.fnl", type = "fennel-macros"},
                    {name = "arbitrary", path = "protoss.fnl"},
                    {name = "the.thing", path = "vw/thing.vw", type = "fennel"},
                    {name = "the.answer", text = "42", type = "fennel"},
                    {name = "a.resposta", text = "return 42", type = "lua"})"#;

fn basic_lua() -> mlua::Result<Manifest> {
    let lua = Lua::new();
    let mut modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>> =
        HashMap::new();
    modules.insert("manifest".into(), Manifest::loader);
    lua.add_function_searcher(modules)
        .expect(ADD_FUNCTION_SEARCHER_EXPECT);
    lua.load(BASIC_LUA).eval()
}

fn dozer() -> Result<Manifest, ManifestInitError> {
    Ok(Manifest::new(
        None,
        vec![
            Module::NamedText(
                ModuleNamedText::new(
                    "dozer/cli",
                    r#"dozer_src::loader("dozer/cli")"#,
                    ModuleFileType::Fennel,
                )
                .map_err(|e| ModuleInitError::from(e))?,
            ),
            Module::NamedText(
                ModuleNamedText::new(
                    "dozer/utils",
                    r#"dozer_src::loader("dozer/utils")"#,
                    ModuleFileType::Fennel,
                )
                .map_err(|e| ModuleInitError::from(e))?,
            ),
        ],
    ))
}

const DOZER_LUA: &str = r#"local manifest = require("manifest")
return manifest.new({name = "dozer/cli",
                     text = [[dozer_src::loader("dozer/cli")]],
                     type = "fennel"},
                    {name = "dozer/utils",
                     text = [[dozer_src::loader("dozer/utils")]],
                     type = "fennel"})"#;

fn dozer_lua() -> mlua::Result<Manifest> {
    let lua = Lua::new();
    let mut modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>> =
        HashMap::new();
    modules.insert("manifest".into(), Manifest::loader);
    lua.add_function_searcher(modules)
        .expect(ADD_FUNCTION_SEARCHER_EXPECT);
    lua.load(DOZER_LUA).eval()
}

fn fennel() -> Result<Manifest, ManifestInitError> {
    Ok(Manifest::new(
        None,
        vec![Module::NamedText(
            ModuleNamedText::new(
                "fennel",
                r#"return "fennel_src::FENNEL100""#,
                ModuleFileType::Lua,
            )
            .map_err(|e| ModuleInitError::from(e))?,
        )],
    ))
}

const FENNEL_LUA: &str = r#"local manifest = require("manifest")
return manifest.new({name = "fennel", text = [[return "fennel_src::FENNEL100"]], type = "lua"})"#;

fn fennel_lua() -> mlua::Result<Manifest> {
    let lua = Lua::new();
    let mut modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>> =
        HashMap::new();
    modules.insert("manifest".into(), Manifest::loader);
    lua.add_function_searcher(modules)
        .expect(ADD_FUNCTION_SEARCHER_EXPECT);
    lua.load(FENNEL_LUA).eval()
}

fn walkman() -> Result<Manifest, ManifestInitError> {
    let Manifest {
        docstring: _,
        modules,
    } = Manifest::from_dir("tests/fixtures/walkman")?;
    Ok(Manifest::new(
        Some(Cow::from("Directory walking example")),
        modules,
    ))
}

const WALKMAN_LUA: &str = r#"local manifest = require("manifest")
return manifest.new("Directory walking example", manifest.walk("tests/fixtures/walkman"))"#;

fn walkman_lua() -> mlua::Result<Manifest> {
    let lua = Lua::new();
    let mut modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>> =
        HashMap::new();
    modules.insert("manifest".into(), Manifest::loader);
    lua.add_function_searcher(modules)
        .expect(ADD_FUNCTION_SEARCHER_EXPECT);
    lua.load(WALKMAN_LUA).eval()
}

#[test]
fn rust_works() {
    assert!(basic().is_ok(), "Basic (Rust)");
    assert!(dozer().is_ok(), "Dozer (Rust)");
    assert!(fennel().is_ok(), "Fennel (Rust)");
    assert!(walkman().is_ok(), "Walkman (Rust)");
}

#[test]
fn lua_works() {
    assert!(basic_lua().is_ok(), "Basic (Lua)");
    assert!(dozer_lua().is_ok(), "Dozer (Lua)");
    assert!(fennel_lua().is_ok(), "Fennel (Lua)");
    assert!(walkman_lua().is_ok(), "Walkman (Lua)");
}

#[test]
fn name_works() {
    let module_file = ModuleFile::new("path/to/1337.fnl", Some(ModuleFileType::Fennel)).unwrap();
    let name = module_file.name();
    assert_eq!(name.as_ref(), "path.to.1337");

    let module_file = ModuleFile::new("path/to/init.fnl", Some(ModuleFileType::Fennel)).unwrap();
    let name = module_file.name();
    assert_eq!(name.as_ref(), "path.to");

    let module_file = ModuleFile::new("another/path.lua", Some(ModuleFileType::Lua)).unwrap();
    let name = module_file.name();
    assert_eq!(name.as_ref(), "another.path");

    let module_file = ModuleFile::new("another/path/init.lua", Some(ModuleFileType::Lua)).unwrap();
    let name = module_file.name();
    assert_eq!(name.as_ref(), "another.path");

    let module_file =
        ModuleFile::new("macs/init-macros.fnl", Some(ModuleFileType::FennelMacros)).unwrap();
    let name = module_file.name();
    assert_eq!(name.as_ref(), "macs");
}
