# fennel-src

Contains [Fennel](https://fennel-lang.org/) release sources. Checks release sources against @technomancy PGP signatures during build.

## Synopsis

### In your Cargo manifest:

If you wish to accept the default Fennel release version provided by `fennel-src`:

```toml
[dependencies]
fennel-src = "*"
```

If you wish to specify a Fennel release version:

```toml
# basic
[dependencies]
fennel-src = { version = "*", default-features = false, features = ["fennel153"] }
```

```toml
# advanced
[features]
default = ["fennel153"]
fennel153 = ["fennel-src/fennel153"]

[dependencies]
fennel-src = { version = "*", default-features = false }
```

### Code

```rust
/// Contents of `fennel-1.5.3`.
const FENNEL: &str = fennel_src::FENNEL153;

/// Path to `fennel-1.5.3`.
const FENNEL_PATH: &str = fennel_src::FENNEL153_PATH;
assert!(Path::new(FENNEL_PATH).is_absolute());

/// Path to `fennel-1.5.3.asc`.
const FENNEL_ASC_PATH: &str = fennel_src::FENNEL153_ASC_PATH;
assert!(Path::new(FENNEL_ASC_PATH).is_absolute());

/// Fennel version.
const FENNEL_VERSION: &str = fennel_src::FENNEL153_VERSION;
assert!(FENNEL_VERSION == "1.5.3");
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

All unmodified works from [Fennel](https://fennel-lang.org/) included are made available under the terms of the original license.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
