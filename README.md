# meka

Embed Lua and compile-to-Lua language modules in Rust.

## Synopsis

Assume we have the following directory structure, where `.` is a simple
Rust library crate containing subdirectory `kiwi`. Inside `kiwi` are
Fennel modules to embed:

```bash
$ tree
.
├── kiwi
│   ├── dine
│   │   ├── init.fnl
│   │   └── proximates.fnl
│   ├── food
│   │   ├── init.fnl
│   │   └── proximates.fnl
│   ├── cite.fnl
│   ├── date-time.fnl
│   ├── date.fnl
│   ├── time.fnl
│   └── utils.fnl
├── src
│   └── lib.rs
└── Cargo.toml

4 directories, 11 files
```

Create `manifest.fnl` alongside `Cargo.toml` in `$CARGO_MANIFEST_DIR`:

```fennel
(local fennel-src (require :fennel-src))
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
{: kiwi}
```

Or, equivalently:

```fennel
(import-macros {: manifest} :meka.macros)
(local fennel-src (require :fennel-src))

(local kiwi (manifest {:name :kiwi.cite            :path :kiwi/cite.fnl}
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

{: kiwi}
```

Or, simply:

```fennel
(import-macros {: manifest} :meka.macros)
(local fennel-src (require :fennel-src))

;; Refer to this manifest in `meka_include!` or `meka_load!` by omitting a string argument.
(manifest {:name :kiwi.cite            :path :kiwi/cite.fnl}
          {:name :kiwi.date            :path :kiwi/date.fnl}
          {:name :kiwi.date-time       :path :kiwi/date-time.fnl}
          {:name :kiwi.dine            :path :kiwi/dine/init.fnl}
          {:name :kiwi.dine.proximates :path :kiwi/dine/proximates.fnl}
          {:name :kiwi.food            :path :kiwi/food/init.fnl}
          {:name :kiwi.food.proximates :path :kiwi/food/proximates.fnl}
          {:name :kiwi.time            :path :kiwi/time.fnl}
          {:name :kiwi.utils           :path :kiwi/utils.fnl}
          (fennel-src))
```

### Embed Fennel/Lua modules in release builds, or read at runtime in debug builds

`main.rs`:

```rust
// Add `add_meka_searcher` method to `mlua::Lua`, and get `meka_searcher!` macro.
use meka::{AddMekaSearcher, meka_searcher};
use mlua::{Lua, LuaOptions, StdLib};

fn main() {
    // Fennel requires instantiating `mlua::Lua` like this.
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };

    // If you won't be needing Fennel, `mlua::Lua` can be instantiated more safely.
    let lua = Lua::new();

    // In release builds:
    //
    // Embed Fennel/Lua modules specified in `kiwi` manifest in `manifest.fnl`, with Fennel
    // sources AOT-compiled to Lua during Rust comptime.
    //
    // In debug builds:
    //
    // Read Fennel modules specified in `kiwi` manifest in `manifest.fnl` to string at runtime
    // and compile to Lua on demand. Read Lua modules specified in `kiwi` manifest at runtime
    // on demand.
    //
    // In all builds:
    //
    // Our source code can access Fennel via `require("fennel")` because we add the manifest
    // returned by `(fennel-src)` - which makes Fennel available at `fennel` - to our `kiwi`
    // manifest.
    //
    // Notes:
    //
    // The (optional) map argument to `meka_searcher!` makes manifest-loader functions available
    // in `manifest.fnl` under the name matching the key said loader function is indexed by
    // in the map. Here, calling `fennel-src` inside `manifest.fnl` calls `fennel_src::loader`.
    //
    // Because `fennel_src::loader` is available by default in `manifest.fnl` at `fennel-src`,
    // passing this map is redundant. However, it can come in handy if you package your own
    // Fennel/Lua modules as Rust crates in the style of ~ioiojo/meka/fennel-src.
    //
    // Passing the optional map to the `meka_searcher!` macro is not recommended, as it
    // causes the macro to generate less efficient code than if the manifest-loader functions
    // were specified in `Cargo.toml` metadata instead:
    //
    //      # Specify manifest-loader functions in Cargo.toml (equivalent, recommended)
    //      [package.metadata.meka.loaders]
    //      fennel-src = "fennel_src::loader"
    let kiwi = meka_searcher!("kiwi", {"fennel-src" => fennel_src::loader});

    // In release builds:
    //
    // Enable Lua's `require` to find embedded Fennel modules (AOT-compiled to Lua) and embedded
    // Lua modules by the names configured in the `kiwi` manifest.
    //
    // In debug builds:
    //
    // Enable Lua's `require` to find Fennel/Lua modules declared in the `kiwi` manifest.
    lua.add_meka_searcher(kiwi).unwrap();

    let uuid = lua.load(r#"require("kiwi.utils").uuid()"#).eval().unwrap();
    // e.g. "d717c6d8-ebed-47ca-8c21-2b6624846ddc"
    eprintln!("{}", uuid);

    let version = lua.load(r#"require("fennel").version"#).eval().unwrap();
    assert_eq!(&version, "1.0.0");
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.
