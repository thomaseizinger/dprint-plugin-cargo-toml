# Obsolete

This functionality is now part of the official dprint toml plugin https://github.com/dprint/dprint-plugin-toml.

# dprint-plugin-cargo-toml

A dprint plugin that formats Cargo.toml files according to Rust's formatting convention: https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/cargo.md

## Limitations

- No configuration
- Cannot handle target-specific dependency blocks like `[target.'cfg(target_arch = "wasm32")'.dependencies]`.
  This is an upstream limitation: https://github.com/DevinR528/toml-parse/pull/7
