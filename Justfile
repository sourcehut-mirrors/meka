# just check
default: check

# Run cargo check on workspace
check:
  cargo check --workspace --features mlua-lua54,mlua-vendored

# Run cargo check on fennel-compile
check-fennel-compile:
  cargo check --package fennel-compile --features mlua-lua54,mlua-vendored

# Run cargo check on fennel-mount
check-fennel-mount:
  cargo check --package fennel-mount --features mlua-lua54,mlua-vendored

# Run cargo check on fennel-searcher
check-fennel-searcher:
  cargo check --package fennel-searcher --features mlua-lua54,mlua-vendored

# Run cargo check on fennel-src
check-fennel-src:
  cargo check --package fennel-src --features mlua-lua54,mlua-vendored

# Run cargo check on meka-config
check-meka-config:
  cargo check --package meka-config --features mlua-lua54,mlua-vendored

# Run cargo check on meka-config-macros
check-meka-config-macros:
  cargo check --package meka-config-macros --features mlua-lua54,mlua-vendored

# Run cargo check on meka-config-macros-test-empty-metadata
check-meka-config-macros-test-empty-metadata:
  cargo check --package meka-config-macros-test-empty-metadata

# Run cargo check on meka-config-macros-test-empty-metadata-section
check-meka-config-macros-test-empty-metadata-section:
  cargo check --package meka-config-macros-test-empty-metadata-section

# Run cargo check on meka-config-macros-test-graceful-degradation
check-meka-config-macros-test-graceful-degradation:
  cargo check --package meka-config-macros-test-graceful-degradation

# Run cargo check on meka-config-macros-test-invalid-loader-path
check-meka-config-macros-test-invalid-loader-path:
  cargo check --package meka-config-macros-test-invalid-loader-path

# Run cargo check on meka-config-macros-test-multiple-loaders
check-meka-config-macros-test-multiple-loaders:
  cargo check --package meka-config-macros-test-multiple-loaders

# Run cargo check on meka-config-macros-test-single-loader
check-meka-config-macros-test-single-loader:
  cargo check --package meka-config-macros-test-single-loader

# Run cargo check on meka-config-tests
check-meka-config-tests:
  cargo check --package meka-config-tests

# Run cargo check on meka-macros
check-meka-macros:
  cargo check --package meka-macros --features mlua-lua54,mlua-vendored

# Run cargo check on meka-module-manifest
check-meka-module-manifest:
  cargo check --package meka-module-manifest --features mlua-lua54,mlua-vendored

# Run cargo check on meka-module-manifest-tests
check-meka-module-manifest-tests:
  cargo check --package meka-module-manifest-tests

# Run cargo check on meka-types
check-meka-types:
  cargo check --package meka-types

# Run cargo check on meka-utils
check-meka-utils:
  cargo check --package meka-utils

# Run cargo check on mlua-module-manifest
check-mlua-module-manifest:
  cargo check --package mlua-module-manifest --features mlua-lua54,mlua-vendored

# Run cargo check on mlua-searcher
check-mlua-searcher:
  cargo check --package mlua-searcher --features mlua-lua54,mlua-vendored

# Run cargo check on mlua-utils
check-mlua-utils:
  cargo check --package mlua-utils --features mlua-lua54,mlua-vendored

# Run cargo test on workspace
test:
  cargo test --workspace --features mlua-lua54,mlua-vendored

# Run cargo test on fennel-compile
test-fennel-compile:
  cargo test --package fennel-compile --features mlua-lua54,mlua-vendored

# Run cargo test on fennel-mount
test-fennel-mount:
  cargo test --package fennel-mount --features mlua-lua54,mlua-vendored

# Run cargo test on fennel-searcher
test-fennel-searcher:
  cargo test --package fennel-searcher --features mlua-lua54,mlua-vendored

# Run cargo test on fennel-src
test-fennel-src:
  cargo test --package fennel-src --features mlua-lua54,mlua-vendored

# Run cargo test on meka-config
test-meka-config:
  cargo test --package meka-config --features mlua-lua54,mlua-vendored

# Run cargo test on meka-config-macros
test-meka-config-macros:
  cargo test --package meka-config-macros --features mlua-lua54,mlua-vendored

# Run cargo test on meka-config-macros-test-empty-metadata
test-meka-config-macros-test-empty-metadata:
  cargo test --package meka-config-macros-test-empty-metadata

# Run cargo test on meka-config-macros-test-empty-metadata-section
test-meka-config-macros-test-empty-metadata-section:
  cargo test --package meka-config-macros-test-empty-metadata-section

# Run cargo test on meka-config-macros-test-graceful-degradation
test-meka-config-macros-test-graceful-degradation:
  cargo test --package meka-config-macros-test-graceful-degradation

# Run cargo test on meka-config-macros-test-invalid-loader-path
test-meka-config-macros-test-invalid-loader-path:
  cargo test --package meka-config-macros-test-invalid-loader-path

# Run cargo test on meka-config-macros-test-multiple-loaders
test-meka-config-macros-test-multiple-loaders:
  cargo test --package meka-config-macros-test-multiple-loaders

# Run cargo test on meka-config-macros-test-single-loader
test-meka-config-macros-test-single-loader:
  cargo test --package meka-config-macros-test-single-loader

# Run cargo test on meka-config-tests
test-meka-config-tests:
  cargo test --package meka-config-tests

# Run cargo test on meka-macros
test-meka-macros:
  cargo test --package meka-macros --features mlua-lua54,mlua-vendored

# Run cargo test on meka-module-manifest
test-meka-module-manifest:
  cargo test --package meka-module-manifest --features mlua-lua54,mlua-vendored

# Run cargo test on meka-module-manifest-tests
test-meka-module-manifest-tests:
  cargo test --package meka-module-manifest-tests

# Run cargo test on meka-types
test-meka-types:
  cargo test --package meka-types

# Run cargo test on meka-utils
test-meka-utils:
  cargo test --package meka-utils

# Run cargo test on mlua-module-manifest
test-mlua-module-manifest:
  cargo test --package mlua-module-manifest --features mlua-lua54,mlua-vendored

# Run cargo test on mlua-searcher
test-mlua-searcher:
  cargo test --package mlua-searcher --features mlua-lua54,mlua-vendored

# Run cargo test on mlua-utils
test-mlua-utils:
  cargo test --package mlua-utils --features mlua-lua54,mlua-vendored
