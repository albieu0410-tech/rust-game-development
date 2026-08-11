# Tests

This folder is reserved for future cross-crate scenario notes and fixtures.

Because the root `Cargo.toml` is a virtual workspace, Cargo does not run Rust integration tests from this directory. Runnable integration tests live under the crate they exercise, starting with:

```text
crates/deduced-core/tests/
```
