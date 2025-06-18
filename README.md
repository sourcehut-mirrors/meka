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
(local fennel-src (require :fennel-src))

(defmanifest kiwi {:name :kiwi.cite            :path :kiwi/cite.fnl}
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

Or, simply:

```fennel
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

### Embed Fennel/Lua modules at compile time

`main.rs`:

```rust
use mlua::Lua;

// Get `meka_include!` macro.
use meka_macros::meka_include;

// Add `add_meka_searcher` method to `mlua::Lua`.
use meka_searcher::AddMekaSearcher;

fn main() {
    let lua = Lua::new();

    // Embed Fennel/Lua modules specified in `kiwi` manifest in `manifest.fnl`, with Fennel
    // sources AOT-compiled to Lua during Rust comptime. Our source code can access Fennel via
    // `require("fennel")` because we added the manifest returned by `(fennel-src)` - which
    // makes Fennel available at `fennel` - to our `kiwi` manifest.
    //
    // The (optional) map argument to `meka_include!` makes manifest-loader functions available
    // in `manifest.fnl` under the name matching the key said loader function is indexed by
    // in the map. Here, calling `fennel-src` inside `manifest.fnl` calls `fennel_src::loader`.
    //
    // Because `fennel_src::loader` is available by default in `manifest.fnl` at `fennel-src`,
    // this map is redundant. However, it can come in handy if you package your own Fennel/Lua
    // modules as Rust crates in the style of ~ioiojo/meka/fennel-src.
    let kiwi = meka_include!("kiwi", {"fennel-src" => fennel_src::loader});

    // Enable Lua's `require` to find embedded Fennel modules (AOT-compiled to Lua) and embedded
    // Lua modules by the names configured in the `kiwi` manifest.
    lua.add_meka_searcher(kiwi).unwrap();

    let uuid = lua.load(r#"require("kiwi.utils").uuid()"#).eval().unwrap();
    // e.g. "d717c6d8-ebed-47ca-8c21-2b6624846ddc"
    eprintln!("{}", uuid);

    let version = lua.load(r#"require("fennel").version"#).eval().unwrap();
    assert_eq!(&version, "1.0.0");
}
```

### Load Fennel/Lua modules at runtime

`main.rs`:

```rust
use mlua::Lua;

// For `meka_load!` macro.
use meka_macros::meka_load;

// Adds the `add_meka_searcher` method to `mlua::Lua`.
use meka_searcher::AddMekaSearcher;

fn main() {
    let lua = Lua::new();

    // Read Fennel modules specified in `kiwi` manifest in `manifest.fnl` to string at runtime
    // and compile to Lua on demand. Read Lua modules specified in `kiwi` manifest at runtime
    // on demand. Our source code can access Fennel via `require("fennel")` because we configured
    // the `kiwi` manifest to embed the Fennel library.
    //
    // Note: `fennel_src::loader` is available by default in `manifest.fnl` at `fennel-src`
    // despite not passing the optional map in here per the `meka_include!` example.
    let kiwi = meka_load!("kiwi");

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
