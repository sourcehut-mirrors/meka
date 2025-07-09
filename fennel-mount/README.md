# fennel-mount

## Description

Make [Fennel](https://fennel-lang.org/) available in `mlua::Lua` instance via [fennel-src][fennel-src] crate.

## Synopsis

### In your Cargo manifest:

If you wish to accept the default Fennel release version provided by [fennel-src][fennel-src]:

```toml
[dependencies]
fennel-mount = "*"
```

If you wish to specify a Fennel release version:

```toml
# basic
[dependencies]
fennel-mount = { version = "*", default-features = false, features = ["fennel153"] }
```

```toml
# advanced
[features]
default = ["fennel153"]
fennel153 = ["fennel-mount/fennel153"]

[dependencies]
fennel-mount = { version = "*", default-features = false }
```

### Code

```rust
use fennel_mount::Mount;
use mlua::{Lua, LuaOptions, StdLib};

fn main() {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    // Add Fennel to Lua's `package.searchers`.
    lua.mount_fennel().unwrap();
    let version = lua.load(r#"return require("fennel").version"#).eval().unwrap();
    // Prints "1.5.3"
    println!("{}", version);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


[fennel-src]: https://git.sr.ht/~ioiojo/meka/tree/master/item/fennel-src
