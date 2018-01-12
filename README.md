# WABT bindings for Rust

[![crates.io](https://img.shields.io/crates/v/wabt.svg)](https://crates.io/crates/wabt)
[![docs.rs](https://docs.rs/wabt/badge.svg)](https://docs.rs/wabt/)

Rust bindings for [WABT](https://github.com/WebAssembly/wabt). Work in progress.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wabt = "0.1"
```

## Example

```rust
extern crate wabt;
use wabt::wat2wasm;

fn main() {
    assert_eq!(
        wat2wasm("(module)").unwrap(),
        &[
            0, 97, 115, 109, // \0ASM - magic
            1, 0, 0, 0       //  0x01 - version
        ]
    );
}
```
