#[test]
fn it_works() {
    use meka_config::{Config, Env};
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};
    use std::collections::HashMap;

    let module = r#"(local fennel-src (require :fennel-src))
(local meka (require :meka))
(local manifest meka.manifest)

;; Map Fennel module paths to the module names you wish to refer to them by in `mlua::Lua`.
;;
;; N.B. the `name` field is optional and redundant in this case since Meka elides those same
;; names from the paths given.
(local kiwi (manifest.new {:name :kiwi.cite            :path :kiwi/cite.fnl}
                          {:name :kiwi.date            :path :kiwi/date.fnl}
                          {:name :kiwi.date-time       :path :kiwi/date-time.fnl}
                          {:name :kiwi.dine            :path :kiwi/dine/init.fnl}
                          {:name :kiwi.dine.proximates :path :kiwi/dine/proximates.fnl}
                          {:name :kiwi.food            :path :kiwi/food/init.fnl}
                          {:name :kiwi.food.proximates :path :kiwi/food/proximates.fnl}
                          {:name :kiwi.time            :path :kiwi/time.fnl}
                          {:name :kiwi.utils           :path :kiwi/utils.fnl}
                          ;; Embed Fennel via `fennel-src` crate. Enables `(require :fennel)`.
                          (fennel-src)))

;; Refer to `kiwi` manifest in `meka_include!` or `meka_load!` by passing in the string "kiwi".
{: kiwi}"#
        .into();
    let module = Module::NamedText(
        ModuleNamedText::new("manifest", module, ModuleFileType::Fennel).unwrap(),
    );
    let env: Option<Env> = None;
    // `fennel-src` should be available by default to `mlua::Lua`.
    assert!(Config::new(module, env).is_ok());

    // Passing equivalent `fennel-src` mapping should make no difference.
    let mut env = Env::new();
    env.insert("fennel-src", fennel_src::loader);
    assert!(Config::new(module, Some(env)).is_ok());

    let module = r#"local fennel_src = require("fennel-src")
local meka = require("meka")
local manifest = meka.manifest
local kiwi = manifest.new({name = "kiwi.cite",            path = "kiwi/cite.fnl"},
                          {name = "kiwi.date",            path = "kiwi/date.fnl"},
                          {name = "kiwi.date-time",       path = "kiwi/date-time.fnl"},
                          {name = "kiwi.dine",            path = "kiwi/dine/init.fnl"},
                          {name = "kiwi.dine.proximates", path = "kiwi/dine/proximates.fnl"},
                          {name = "kiwi.food",            path = "kiwi/food/init.fnl"},
                          {name = "kiwi.food.proximates", path = "kiwi/food/proximates.fnl"},
                          {name = "kiwi.time",            path = "kiwi/time.fnl"},
                          {name = "kiwi.utils",           path = "kiwi/utils.fnl"},
                          fennel_src())
return {kiwi = kiwi}"#;
    let module =
        Module::NamedText(ModuleNamedText::new("manifest", module, ModuleFileType::Lua).unwrap());

    // Passing equivalent Lua config should make no difference.
    assert!(Config::new(module, None).is_ok());
    assert!(Config::new(module, env).is_ok());
}

#[test]
fn defmanifest_fennel_macro_works() {
    use meka_config::{Config, Env};
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};
    use std::collections::HashMap;

    let module = r#"(local fennel-src (require :fennel-src))

(defmanifest kiwi {:name :kiwi.cite            :path :kiwi/cite.fnl
                   :name :kiwi.date            :path :kiwi/date.fnl
                   :name :kiwi.date-time       :path :kiwi/date-time.fnl
                   :name :kiwi.dine            :path :kiwi/dine/init.fnl
                   :name :kiwi.dine.proximates :path :kiwi/dine/proximates.fnl
                   :name :kiwi.food            :path :kiwi/food/init.fnl
                   :name :kiwi.food.proximates :path :kiwi/food/proximates.fnl
                   :name :kiwi.time            :path :kiwi/time.fnl
                   :name :kiwi.utils           :path :kiwi/utils.fnl}
                   (fennel-src))"#
        .into();
    let module = Module::NamedText(
        ModuleNamedText::new("manifest", module, ModuleFileType::Fennel).unwrap(),
    );
    let env: Option<Env> = None;
    assert!(Config::new(module, env).is_ok());

    let mut env = Env::new();
    env.insert("fennel-src", fennel_src::loader);
    assert!(Config::new(module, Some(env)).is_ok());
}

#[test]
fn manifest_fennel_macro_works() {
    use meka_config::{Config, Env};
    use mlua_module_manifest::{Module, ModuleFileType, ModuleNamedText};
    use std::collections::HashMap;

    let module = r#"(local fennel-src (require :fennel-src))

;; Refer to this manifest in `meka_include!` or `meka_load!` by omitting a string argument.
(manifest {:name :kiwi.cite            :path :kiwi/cite.fnl
           :name :kiwi.date            :path :kiwi/date.fnl
           :name :kiwi.date-time       :path :kiwi/date-time.fnl
           :name :kiwi.dine            :path :kiwi/dine/init.fnl
           :name :kiwi.dine.proximates :path :kiwi/dine/proximates.fnl
           :name :kiwi.food            :path :kiwi/food/init.fnl
           :name :kiwi.food.proximates :path :kiwi/food/proximates.fnl
           :name :kiwi.time            :path :kiwi/time.fnl
           :name :kiwi.utils           :path :kiwi/utils.fnl}
          (fennel-src))"#
        .into();
    let module = Module::NamedText(
        ModuleNamedText::new("manifest", module, ModuleFileType::Fennel).unwrap(),
    );
    let env: Option<Env> = None;
    assert!(Config::new(module, env).is_ok());

    let mut env = Env::new();
    env.insert("fennel-src", fennel_src::loader);
    assert!(Config::new(arg, Some(env)).is_ok());
}
