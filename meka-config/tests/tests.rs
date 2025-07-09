#[test]
fn it_works() {
    use meka_config::{Config, LoaderRegistry};
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};
    use std::borrow::Cow;
    use std::convert::From;

    let module: &str = r#"(local fennel-src (require :fennel-src))
(local meka (require :meka))
(local manifest meka.manifest)

;; Map Fennel module paths to the module names you wish to refer to them by in `mlua::Lua`.
;;
;; N.B. the `name` field is optional and redundant in this case since Meka derives those same
;; names from the paths given.
(local taon (manifest.new {:name :taon.cite            :path :taon/cite.fnl}
                          {:name :taon.date            :path :taon/date.fnl}
                          {:name :taon.date-time       :path :taon/date-time.fnl}
                          {:name :taon.dine            :path :taon/dine/init.fnl}
                          {:name :taon.dine.proximates :path :taon/dine/proximates.fnl}
                          {:name :taon.food            :path :taon/food/init.fnl}
                          {:name :taon.food.proximates :path :taon/food/proximates.fnl}
                          {:name :taon.time            :path :taon/time.fnl}
                          {:name :taon.utils           :path :taon/utils.fnl}
                          ;; Embed Fennel via `fennel-src` crate. Enables `(require :fennel)`.
                          (fennel-src)))

;; Refer to `taon` manifest in `meka_searcher!` or `meka_searcher_hot!` by passing the string
;; "taon" as first argument.
{: taon}"#
        .into();
    let module = Module::NamedText(
        ModuleNamedText::new("manifest", module, ModuleFileType::Fennel).unwrap(),
    );
    let loader_registry: Option<LoaderRegistry> = None;
    // `fennel-src` should be available by default to `mlua::Lua`.
    assert!(Config::new(module.clone(), loader_registry).is_ok());

    // Passing equivalent `fennel-src` mapping should make no difference.
    let mut loader_registry = LoaderRegistry::new();
    loader_registry.insert(Cow::from("fennel-src"), fennel_src::loader);
    assert!(Config::new(module, Some(loader_registry.clone())).is_ok());

    let module: &str = r#"local fennel_src = require("fennel-src")
local meka = require("meka")
local manifest = meka.manifest
local taon = manifest.new({name = "taon.cite",            path = "taon/cite.fnl"},
                          {name = "taon.date",            path = "taon/date.fnl"},
                          {name = "taon.date-time",       path = "taon/date-time.fnl"},
                          {name = "taon.dine",            path = "taon/dine/init.fnl"},
                          {name = "taon.dine.proximates", path = "taon/dine/proximates.fnl"},
                          {name = "taon.food",            path = "taon/food/init.fnl"},
                          {name = "taon.food.proximates", path = "taon/food/proximates.fnl"},
                          {name = "taon.time",            path = "taon/time.fnl"},
                          {name = "taon.utils",           path = "taon/utils.fnl"},
                          fennel_src())
return {taon = taon}"#;
    let module =
        Module::NamedText(ModuleNamedText::new("manifest", module, ModuleFileType::Lua).unwrap());

    // Passing equivalent Lua config should make no difference.
    assert!(Config::new(module.clone(), None).is_ok());
    assert!(Config::new(module, Some(loader_registry)).is_ok());
}

#[test]
fn manifest_fennel_macro_works() {
    use meka_config::{Config, LoaderRegistry};
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};
    use std::borrow::Cow;
    use std::convert::From;

    let module: &str = r#"(import-macros {: manifest} :meka.macros)
    (local fennel-src (require :fennel-src))

;; Meka automatically imports the manifest macro. The following is short for:
;;
;;     (local taon ((. (require :meka) :manifest :new) {...}))
;;
(local taon (manifest {:name :taon.cite            :path :taon/cite.fnl}
                      {:name :taon.date            :path :taon/date.fnl}
                      {:name :taon.date-time       :path :taon/date-time.fnl}
                      {:name :taon.dine            :path :taon/dine/init.fnl}
                      {:name :taon.dine.proximates :path :taon/dine/proximates.fnl}
                      {:name :taon.food            :path :taon/food/init.fnl}
                      {:name :taon.food.proximates :path :taon/food/proximates.fnl}
                      {:name :taon.time            :path :taon/time.fnl}
                      {:name :taon.utils           :path :taon/utils.fnl}
                      ;; Embed Fennel via `fennel-src` crate. Enables `(require :fennel)`.
                      (fennel-src)))

{: taon}"#
        .into();
    let module = Module::NamedText(
        ModuleNamedText::new("manifest", module, ModuleFileType::Fennel).unwrap(),
    );
    let loader_registry: Option<LoaderRegistry> = None;
    assert!(Config::new(module.clone(), loader_registry).is_ok());

    let mut loader_registry = LoaderRegistry::new();
    loader_registry.insert(Cow::from("fennel-src"), fennel_src::loader);
    assert!(Config::new(module, Some(loader_registry)).is_ok());
}

#[test]
fn standalone_manifest_fennel_macro_works() {
    use meka_config::{Config, LoaderRegistry};
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};
    use std::borrow::Cow;
    use std::convert::From;

    let module: &str = r#"(import-macros {: manifest} :meka.macros)
    (local fennel-src (require :fennel-src))

;; Refer to this manifest in `meka_searcher!` or `meka_searcher_hot!` by omitting a string
;; argument.
(manifest {:name :taon.cite            :path :taon/cite.fnl
           :name :taon.date            :path :taon/date.fnl
           :name :taon.date-time       :path :taon/date-time.fnl
           :name :taon.dine            :path :taon/dine/init.fnl
           :name :taon.dine.proximates :path :taon/dine/proximates.fnl
           :name :taon.food            :path :taon/food/init.fnl
           :name :taon.food.proximates :path :taon/food/proximates.fnl
           :name :taon.time            :path :taon/time.fnl
           :name :taon.utils           :path :taon/utils.fnl}
          (fennel-src))"#
        .into();
    let module = Module::NamedText(
        ModuleNamedText::new("manifest", module, ModuleFileType::Fennel).unwrap(),
    );
    let loader_registry: Option<LoaderRegistry> = None;
    assert!(Config::new(module.clone(), loader_registry).is_ok());

    let mut loader_registry = LoaderRegistry::new();
    loader_registry.insert(Cow::from("fennel-src"), fennel_src::loader);
    assert!(Config::new(module, Some(loader_registry)).is_ok());
}
