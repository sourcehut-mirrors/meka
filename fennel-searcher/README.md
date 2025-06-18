# fennel-searcher

Require [Fennel](https://fennel-lang.org/) modules by name at runtime.

## Synopsis

### In your Cargo manifest:

If you wish to accept the default Fennel release version provided by
[fennel-src][fennel-src]:

```toml
[dependencies]
fennel-searcher = "*"
```

If you wish to specify a Fennel release version:

```toml
# basic
[dependencies]
fennel-searcher = { version = "*", default-features = false, features = ["fennel153"] }
```

```toml
# advanced
[features]
default = ["fennel153"]
fennel153 = ["fennel-searcher/fennel153"]

[dependencies]
fennel-searcher = { version = "*", default-features = false }
```

### Code

```rust
use fennel_searcher::AddSearcherFnl;
use mlua::Lua;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let lime = Cow::from("lime/color");
    let color = PathBuf::new()
        .join(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("testcrate")
        .join("lime")
        .join("color.fnl");
    let mut map = HashMap::new();
    map.insert(lime, color);

    let lua = Lua::new();

    lua.add_path_searcher_fnl(lime).unwrap();
    let color = lua.load(r#"return require("lime/color")"#).eval().unwrap();

    assert_eq!(&color, "green");
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.


[fennel-src]: https://git.sr.ht/~ioiojo/meka/tree/master/item/fennel-src
