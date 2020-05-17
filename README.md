# WABT bindings for Rust

[![crates.io](https://img.shields.io/crates/v/wabt.svg)](https://crates.io/crates/wabt)
[![docs.rs](https://docs.rs/wabt/badge.svg)](https://docs.rs/wabt/)

Rust bindings for [WABT](https://github.com/WebAssembly/wabt).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wabt = "0.9.0"
```

## Use cases

Assemble a given program in WebAssembly text format (aka wat) and translate it into binary.

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

or disassemble a wasm binary into the text format.

```rust
extern crate wabt;
use wabt::wasm2wat;
fn main() {
    assert_eq!(
        wasm2wat(&[
            0, 97, 115, 109, // \0ASM - magic
            1, 0, 0, 0       //    01 - version
        ]),
        Ok("(module)\n".to_owned()),
    );
}
```

`wabt` can be also used for parsing the official [testsuite](https://github.com/WebAssembly/testsuite) scripts.

```rust
use wabt::script::{ScriptParser, Command, CommandKind, Action, Value};

let wast = r#"
;; Define anonymous module with function export named `sub`.
(module
  (func (export "sub") (param $x i32) (param $y i32) (result i32)
    ;; return x - y;
    (i32.sub
      (get_local $x) (get_local $y)
    )
  )
)

;; Assert that invoking export `sub` with parameters (8, 3)
;; should return 5.
(assert_return
  (invoke "sub"
    (i32.const 8) (i32.const 3)
  )
  (i32.const 5)
)
"#;

let mut parser = ScriptParser::<f32, f64>::from_str(wast)?;
while let Some(Command { kind, .. }) = parser.next()? {
    match kind {
        CommandKind::Module { module, name } => {
            // The module is declared as anonymous.
            assert_eq!(name, None);

            // Convert the module into the binary representation and check the magic number.
            let module_binary = module.into_vec();
            assert_eq!(&module_binary[0..4], &[0, 97, 115, 109]);
        }
        CommandKind::AssertReturn { action, expected } => {
            assert_eq!(action, Action::Invoke {
                module: None,
                field: "sub".to_string(),
                args: vec![
                    Value::I32(8),
                    Value::I32(3)
                ],
            });
            assert_eq!(expected, vec![Value::I32(5)]);
        },
        _ => panic!("there are no other commands apart from that defined above"),
    }
}
```

# Alternatives

You might find [`wat`](https://crates.io/crates/wat) or [`wast`](https://crates.io/crates/wast)
crate useful if you only want to parse `.wat` or `.wast` source. The advantage of using them is that
they are implemented completely in Rust. Moreover, [`wast`](https://crates.io/crates/wast) among other things
allows you to add your own extensions to WebAssembly text format.

For print the text representation of a wasm binary, [`wasmprinter`](https://crates.io/crates/wasmprinter)
can work better for you, since it is implemented completely in Rust.
