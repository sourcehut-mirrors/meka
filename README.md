# Meka

Embed Lua and compile-to-Lua language modules in Rust.

## Synopsis

Assume we have the following directory structure, where `.` is a simple Rust library crate containing subdirectory `taon`. Inside `taon` are [Fennel] modules to embed:

```bash
$ tree
.
├── src
│   └── lib.rs
├── taon
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
{: taon}
```

Or, equivalently:

```fennel
(import-macros {: manifest} :meka.macros)
(local fennel-src (require :fennel-src))

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

{: taon}
```

Or, simply:

```fennel
(import-macros {: manifest} :meka.macros)
(local fennel-src (require :fennel-src))

;; Refer to this manifest in `meka_searcher!` or `meka_searcher_hot!` by omitting a string
;; argument.
(manifest {:path :taon/cite.fnl}
          {:path :taon/date.fnl}
          {:path :taon/date-time.fnl}
          {:path :taon/dine/init.fnl}
          {:path :taon/dine/proximates.fnl}
          {:path :taon/food/init.fnl}
          {:path :taon/food/proximates.fnl}
          {:path :taon/time.fnl}
          {:path :taon/utils.fnl}
          (fennel-src))
```

### Embed Fennel/Lua modules in release builds, or read at runtime in debug builds

`main.rs`:

```rust
// Add `add_meka_searcher` method to `mlua::Lua`, import macros.
use meka::{AddMekaSearcher, meka_searcher, meka_searcher_hot};
use mlua::Lua;

fn main() {
    let lua = Lua::new();

    // In release builds:
    //
    // Embed Fennel/Lua modules specified in `taon` manifest in `manifest.fnl`, with Fennel
    // sources AOT-compiled to Lua during Rust comptime.
    //
    // In debug builds:
    //
    // Read Fennel modules specified in `taon` manifest in `manifest.fnl` to string at runtime
    // and compile to Lua on demand. Read Lua modules specified in `taon` manifest at runtime
    // on demand.
    //
    // In all builds:
    //
    // Our source code can access Fennel via `require("fennel")` because we add the manifest
    // returned by `(fennel-src)` - which makes Fennel available at `fennel` - to our `taon`
    // manifest.
    //
    // Notes:
    //
    // The (optional) map argument to `meka_searcher!` makes manifest-loader functions available
    // in `manifest.fnl` under the name matching the key said loader function is indexed by
    // in the map. Here, calling `fennel-src` inside `manifest.fnl` calls `fennel_src::loader`.
    //
    // Because `fennel_src::loader` is available by default in `manifest.fnl` at `fennel-src`,
    // passing this map is redundant. However, other manifest-loader functions can be passed
    // as well (see: `meka-module-registry`).
    //
    // To use your own manifest-loader function in `manifest.fnl`, either:
    //
    // A) Specify it in `Cargo.toml` rather than passing it as a map to `meka_searcher!`:
    //
    //      [package.metadata.meka.loaders]
    //      fennel-src = "fennel_src::loader"
    //
    // B) Submit a patch to `meka-module-registry` to get it included.
    //
    // C) Build Meka with the `registry` feature disabled (`--no-default-features`).
    //
    // When the `registry` feature is disabled, passing the optional map to the `meka_searcher!`
    // macro causes the macro to generate less efficient code than if the manifest-loader
    // functions were specified in `Cargo.toml` metadata instead.
    #[cfg(not(debug_assertions))]
    let taon = meka_searcher!("taon", {"fennel-src" => fennel_src::loader});
    #[cfg(debug_assertions)]
    let taon = meka_searcher_hot!("taon", {"fennel-src" => fennel_src::loader});

    // In release builds:
    //
    // Enable Lua's `require` to find embedded Fennel modules (AOT-compiled to Lua) and embedded
    // Lua modules by the names configured in the `taon` manifest.
    //
    // In debug builds:
    //
    // Enable Lua's `require` to find Fennel/Lua modules declared in the `taon` manifest.
    lua.add_meka_searcher(taon).unwrap();

    let uuid: String = lua.load(r#"require("taon.utils").uuid()"#).eval().unwrap();
    // e.g. "d717c6d8-ebed-47ca-8c21-2b6624846ddc"
    eprintln!("{}", uuid);

    let version: String = lua.load(r#"require("fennel").version"#).eval().unwrap();
    assert_eq!(&version, "1.6.0");
}
```

## Cargo features

- fennel100 - Embed v1.0.0 release of Fennel where appropriate.
- fennel153 - Embed v1.5.3 release of Fennel where appropriate.
- fennel160 - **Enabled** by default. Embed v1.6.0 release of Fennel where appropriate.
- mlua-external - Indicate Lua includes and libraries are to be found externally via `pkg-config` rather than vendoring them.
- mlua-lua54 - Enable Lua [5.4] support.
- mlua-lua53 - Enable Lua [5.3] support.
- mlua-lua52 - Enable Lua [5.2] support.
- mlua-lua51 - Enable Lua [5.1] support.
- mlua-luajit - Enable [LuaJIT] support.
- mlua-luajit52 - Enable [LuaJIT] support with partial compatibility with Lua 5.2.
- mlua-luau - Enable [Luau] support (auto vendored mode).
- mlua-luau-jit - Enable [Luau] support with JIT backend.
- mlua-luau-vector4 - Enable [Luau] support with 4-dimensional vector.
- mlua-module - Enable module mode (building loadable `cdylib` library for Lua).
- mlua-vendored - Build static Lua(JIT) libraries from sources during `mlua` compilation using [lua-src] or [luajit-src].
- preload - Only relevant if building with mlua-module feature active. Indicate environment variable `LD_PRELOAD` or `DYLD_INSERT_LIBRARIES` has been set to preclude need for launching subprocesses in mlua-module mode and to allow more efficient code generation in mlua-module mode when registry feature is active.
- registry - **Enabled** by default. Map strings to function pointers for more efficient code generation in most cases.

## Other topics

### Embedding Fennel Macro modules

It's about as easy to embed Fennel macro modules as it is to embed regular Fennel or Lua modules:

```fennel
(import-macros {: manifest} :meka.macros)
(local fennel-src (require :fennel-src))
(manifest ;; Paths with a `.fnlm` file extension are automatically recognized as Fennel macro
          ;; modules:
          {:name :taon.mu :path :taon/mu.fnlm}
          ;; As are paths whose last segment is `init-macros.fnl`:
          {:name :taon.nu :path :taon/nu/init-macros.fnl}
          ;; Otherwise, the module's `:type` key must be set to `:fennel-macros` in order for
          ;; it to be treated as a Fennel macro module.
          {:name :taon.macros :path :taon/macros.fnl :type :fennel-macros}
          ;; To use embedded Fennel macro modules, Fennel itself must be available for import.
          (fennel-src))
```

However, *using* the Fennel macro modules declared in a manifest (and embedded in a `MekaSearcher`) isn't possible unless Fennel itself is available for import under the module name "fennel".

Meka will make Fennel available for import under module name "fennel" when you add a suitable declaration to your manifest, either by using the `fennel-src` crate per the above example, or by hand, as in this one:

```fennel
;; Functionally equivalent to above example assuming `fennel.lua` contains identical Fennel:
(import-macros {: manifest} :meka.macros)
(manifest {:name :taon.mu :path :taon/mu.fnlm}
          {:name :taon.nu :path :taon/nu/init-macros.fnl}
          ;; N.B. `macros.fnl` isn't automatically detected as being a Fennel macros module.
          ;; (see: `fennel.macro-path`)
          {:name :taon.macros :path :taon/macros.fnl :type :fennel-macros}
          ;; Make Fennel available for import by module name "fennel":
          {:name :fennel :path :path/to/fennel.lua})
```

Here's why this is necessary. *Using* embedded Fennel macro modules entails:

a) Adding a bespoke Fennel macro module searcher which has direct access to said embedded Fennel macro modules to the `fennel.macro-searchers` table.

Meka achieves this via the `MekaSearcher` data structure and the `AddMekaSearcher` trait. This enables the embedded Fennel macro modules to be imported via `import-macros` in Fennel source code much like they would be if they were files on disk being referenced in a normal Fennel project.

b) Calling `fennel.eval(content, {env = "_COMPILER"})`, where `content` is the content of a Fennel macros module.

Meka implements this via the `fennel-searcher` crate's `AddSearcher` trait. This evaluates the given Fennel macros module in the special `_COMPILER` environment, which makes functions/macros defined therein available at compile time.

One way or another, at the point Fennel macro modules are to be used (not just kept around as embedded text), Fennel itself must be available for import under module name "fennel". Again, Meka will handle this automatically for you if, as in the two examples above, you declare Fennel as part of your manifest. Another possible solution is to use the `fennel-mount` crate's `Mount` trait to call `mount_fennel()` on an `mlua::Lua` instance. Alternatively, you might consider circumventing all this by AOT-compiling your Fennel code to Lua (as part of your project's build process, for example).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


[Fennel]: https://fennel-lang.org/
[5.4]: https://www.lua.org/manual/5.4/manual.html
[5.3]: https://www.lua.org/manual/5.3/manual.html
[5.2]: https://www.lua.org/manual/5.2/manual.html
[5.1]: https://www.lua.org/manual/5.1/manual.html
[LuaJIT]: https://luajit.org/
[Luau]: https://github.com/luau-lang/luau
[lua-src]: https://github.com/mlua-rs/lua-src-rs
[luajit-src]: https://github.com/mlua-rs/luajit-src-rs
