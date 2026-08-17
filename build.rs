//! Build script.
//!
//! Without this, plain `cargo build` / `cargo test` / `cargo clippy` cannot
//! link this crate on macOS: the `[lib]` target declares `crate-type =
//! ["cdylib", "rlib"]` for PyO3's `extension-module` feature, which
//! intentionally does *not* link against libpython at build time (the
//! symbols are resolved dynamically when Python loads the compiled
//! extension). The linker still needs to be told that's OK via
//! `-undefined dynamic_lookup` on macOS -- `maturin` injects that flag
//! itself when it drives the build, which is why `maturin develop` worked
//! even though bare `cargo build` failed with "symbol(s) not found for
//! architecture ..." for every `Py_*` / `PyObject_*` symbol.
//!
//! This calls the same helper PyO3's own (unused, by us) build script would
//! call, so `cargo build`/`test`/`clippy`/`check` all work the same way
//! `maturin` does, without requiring maturin for local Rust-only iteration
//! (e.g. `cargo test --workspace`, `tests/server_integration.rs`).
fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
