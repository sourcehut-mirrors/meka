# mlua-searcher

`require` Lua modules by name

## Description

Encode a Lua module as a `HashMap` of Lua strings indexed by module
name. In an `mlua::Lua`, pass the `HashMap` to `add_searcher()`, and
`require` the module.

## Synopsis

```rust
use mlua::Lua;
use mlua_searcher::{AddSearcher, Result};
use std::collections::HashMap;

fn main() {
    let lume = Cow::from(read_lume_to_string());
    let name = Cow::from("lume");
    let mut map = HashMap::new();
    map.insert(name, lume);

    let lua = Lua::new();

    lua.add_searcher(map).unwrap();
    let hello = lua.load(r#"return require("lume")"#).eval().unwrap();

    // prints "hello lume"
    println!("{}", hello);
}

fn read_lume_to_string() -> String {
    r#"return "hello lume""#.to_string()
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
