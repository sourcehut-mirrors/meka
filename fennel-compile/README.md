# fennel-compile

Compile [Fennel](https://fennel-lang.org/) source code to Lua.

## Description

Adds Fennel-to-Lua compilation functions to `mlua::Lua` instance.

## Synopsis

```rust
use fennel_compile::Compile;
use fennel_mount::Mount;
use mlua::{Lua, LuaOptions, StdLib};

fn main() {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    // Add Fennel to Lua's `package.searchers`. Required before running `compile_fennel_string`.
    lua.mount_fennel().unwrap();
    let got = lua.compile_fennel_string("(print (+ 1 1))");
    // Prints "return print((1 + 1))".
    println!("{}", got);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
