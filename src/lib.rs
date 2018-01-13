//! Bindings to the [wabt](https://github.com/WebAssembly/wabt) library.
//!

extern crate wabt_sys;

use std::os::raw::{c_void, c_int};
use std::ffi::{CString, NulError};
use std::ptr;

use wabt_sys::*;
use wabt_sys as ffi;

/// A structure to represent errors coming out from wabt.
///
/// Actual errors are not yet published.
#[derive(Debug, PartialEq, Eq)]
pub struct Error(ErrorKind);

#[derive(Debug, PartialEq, Eq)]
enum ErrorKind {
    Nul,
    Deserialize,
    Parse,
    WriteText,
    ResolveNames,
    Validate,
}

impl From<NulError> for Error {
    fn from(_e: NulError) -> Error {
        Error(ErrorKind::Nul)
    }
}

struct Lexer {
    _filename: CString,
    _buffer: Vec<u8>,
    raw_lexer: *mut ffi::WastLexer,
}

impl Lexer {
    fn new(filename: &str, buffer: &[u8]) -> Result<Lexer, Error> {
        let filename = CString::new(filename)?;
        let buffer = buffer.to_owned();
        let lexer = unsafe {
            ffi::wabt_new_wast_buffer_lexer(
                filename.as_ptr(),
                buffer.as_ptr() as *const c_void,
                buffer.len(),
            )
        };

        Ok(Lexer {
            _filename: filename,
            _buffer: buffer,
            raw_lexer: lexer,
        })
    }
}

impl Drop for Lexer {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_wast_lexer(self.raw_lexer);
        }
    }
}

/// Translate wasm text source to wasm binary format.
///
/// If wasm source is valid wasm binary will be returned in the vector.
/// Returned binary is validated and can be executed.
///
/// For more examples and online demo you can check online version
/// of [wat2wasm](https://cdn.rawgit.com/WebAssembly/wabt/aae5a4b7/demo/wat2wasm/).
///
/// # Examples
///
/// ```rust
/// extern crate wabt;
/// use wabt::wat2wasm;
///
/// fn main() {
///     assert_eq!(
///         wat2wasm("(module)").unwrap(),
///         &[
///             0, 97, 115, 109, // \0ASM - magic
///             1, 0, 0, 0       //  0x01 - version
///         ]
///     );
/// }
/// ```
///
pub fn wat2wasm(src: &str) -> Result<Vec<u8>, Error> {
    let lexer = Lexer::new("test.wast", src.as_bytes())
        .expect("filename is passed as literal and can't contain nul characters");

    unsafe {
        let error_handler = wabt_new_text_error_handler_buffer();

        let result = wabt_parse_wat(lexer.raw_lexer, error_handler);
        if wabt_parse_wat_result_get_result(result) == ResultEnum::Error {
            return Err(Error(ErrorKind::Parse));
        }
        let module = wabt_parse_wat_result_release_module(result);

        let result = wabt_resolve_names_module(lexer.raw_lexer, module, error_handler);
        if result == ResultEnum::Error {
            return Err(Error(ErrorKind::ResolveNames));
        }

        let result = wabt_validate_module(lexer.raw_lexer, module, error_handler);
        if result == ResultEnum::Error {
            return Err(Error(ErrorKind::Validate));
        }

        let result = wabt_write_binary_module(module, 0, 1, 0, 0);
        assert!(wabt_write_module_result_get_result(result) == ResultEnum::Ok);

        let output_buffer = wabt_write_module_result_release_output_buffer(result);

        let out_data = wabt_output_buffer_get_data(output_buffer) as *const u8;
        let out_size = wabt_output_buffer_get_size(output_buffer);

        let mut result = Vec::with_capacity(out_size);
        result.set_len(out_size);
        ptr::copy_nonoverlapping(out_data, result.as_mut_ptr(), out_size);

        wabt_destroy_output_buffer(output_buffer);
        wabt_destroy_module(module);
        wabt_destroy_error_handler_buffer(error_handler);

        Ok(result)
    }
}

/// Disassemble wasm binary to wasm text format.
///
/// # Examples
///
/// ```rust
/// extern crate wabt;
/// use wabt::wasm2wat;
///
/// fn main() {
///     assert_eq!(
///         wasm2wat(&[
///             0, 97, 115, 109, // \0ASM - magic
///             1, 0, 0, 0       //    01 - version
///         ]),
///         Ok("(module)\n".to_owned()),
///     );
/// }
/// ```
///
pub fn wasm2wat(wasm: &[u8]) -> Result<String, Error> {
    unsafe {
        let error_handler = wabt_new_binary_error_handler_buffer();
        let result = wabt_read_binary(wasm.as_ptr(), wasm.len(), true as c_int, error_handler);
        if wabt_read_binary_result_get_result(result) == ResultEnum::Error {
            return Err(Error(ErrorKind::Deserialize));
        }
        let module = wabt_read_binary_result_release_module(result);
        wabt_destroy_read_binary_result(result);

        let result = wabt_write_text_module(module, 0, 0);
        if wabt_write_module_result_get_result(result) == ResultEnum::Error {
            return Err(Error(ErrorKind::WriteText));
        }
        let output_buffer = wabt_write_module_result_release_output_buffer(result);

        let data = wabt_output_buffer_get_data(output_buffer);
        let size = wabt_output_buffer_get_size(output_buffer);

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        ptr::copy_nonoverlapping(data as *const u8, buf.as_mut_ptr(), size);
        buf.set_len(size);

        let text = String::from_utf8(buf).unwrap();
        Ok(text)
    }
}

#[test]
fn test_wat2wasm() {
    assert_eq!(
        wat2wasm("(module)").unwrap(),
        &[0, 97, 115, 109, 1, 0, 0, 0]
    );

    assert_eq!(
        wat2wasm(
            r#"
            (module
            )"#,
        ).unwrap(),
        &[0, 97, 115, 109, 1, 0, 0, 0]
    );

    assert_eq!(wat2wasm("(modu"), Err(Error(ErrorKind::Parse)));
}

#[test]
fn test_wasm2wat() {
    assert_eq!(
        wasm2wat(&[
            0, 97, 115, 109, // \0ASM - magic
            1, 0, 0, 0       //    01 - version
        ]),
        Ok("(module)\n".to_owned()),
    );

    assert_eq!(
        wasm2wat(&[
            0, 97, 115, 109, // \0ASM - magic
        ]),
        Err(Error(ErrorKind::Deserialize)),
    );
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn roundtrip() {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let factorial = &[
        0, 97, 115, 109, 1, 0, 0, 0, 1, 6, 1, 96, 1, 124, 1, 124, 3, 2, 1, 0, 7, 7, 
        1, 3, 102, 97, 99, 0, 0, 10, 46, 1, 44, 0, 32, 0, 68, 0, 0, 0, 0, 0, 0, 240, 
        63, 99, 4, 124, 68, 0, 0, 0, 0, 0, 0, 240, 63, 5, 32, 0, 32, 0, 68, 0, 0, 0, 
        0, 0, 0, 240, 63, 161, 16, 0, 162, 11, 11
    ];

    let text = wasm2wat(factorial).unwrap();
    let binary = wat2wasm(&text).unwrap();

    assert_eq!(&*factorial as &[u8], &*binary);
}
