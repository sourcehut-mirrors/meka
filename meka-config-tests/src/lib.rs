#[test]
fn it_works() {
    use meka_module_manifest::CompiledNamedTextManifest;
    use mlua_module_manifest::{Manifest, Module, ModuleFile};
    use std::borrow::Cow;
    use std::convert::TryFrom;

    let manifest = Manifest::new(
        Some(Cow::from("Basic example")),
        vec![
            Module::File(ModuleFile::new("fruit/macros.fnlm", None).unwrap()),
            Module::File(ModuleFile::new("fruit/orchard.fnl", None).unwrap()),
            Module::File(ModuleFile::new("lime/color.fnl", None).unwrap()),
            Module::File(ModuleFile::new("lime/time.lua", None).unwrap()),
        ],
    );

    assert!(CompiledNamedTextManifest::try_from(manifest).is_ok());
}
