//! Bindings to the [wabt](https://github.com/WebAssembly/wabt) library.
//!

extern crate wabt_sys;

use std::os::raw::{c_void, c_int};
use std::ffi::{CString, NulError};
use std::slice;
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
    Deserialize(String),
    Parse(String),
    WriteText,
    NonUtf8Result,
    WriteBinary,
    ResolveNames(String),
    Validate(String),
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

struct ErrorHandler {
    raw_buffer: *mut ErrorHandlerBuffer,
}

impl ErrorHandler {
    fn new_binary() -> ErrorHandler {
        let raw_buffer = unsafe {
            ffi::wabt_new_binary_error_handler_buffer()
        };
        ErrorHandler {
            raw_buffer,
        }
    }

    fn new_text() -> ErrorHandler {
        let raw_buffer = unsafe {
            ffi::wabt_new_text_error_handler_buffer()
        };
        ErrorHandler {
            raw_buffer,
        }
    }

    fn raw_message(&self) -> &[u8] {
        unsafe {
            let size = ffi::wabt_error_handler_buffer_get_size(self.raw_buffer);
            if size == 0 {
                return &[];
            }

            let data = ffi::wabt_error_handler_buffer_get_data(self.raw_buffer);
            slice::from_raw_parts(data as *const u8, size)
        }
    }
}

impl Drop for ErrorHandler {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_error_handler_buffer(self.raw_buffer);
        }
    }
}

struct ParseWatResult {
    raw_result: *mut ffi::WabtParseWatResult,
}

impl ParseWatResult {
    fn is_ok(&self) -> bool {
        unsafe {
            ffi::wabt_parse_wat_result_get_result(self.raw_result) == ResultEnum::Ok
        }
    }

    fn module(self) -> Result<*mut ffi::WasmModule, ()> {
        if self.is_ok() {
            unsafe {
                Ok(wabt_parse_wat_result_release_module(self.raw_result))
            }
        } else {
            Err(())
        }
    }
}

impl Drop for ParseWatResult {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_parse_wat_result(self.raw_result);
        }
    }
}

fn parse_wat(lexer: &Lexer, error_handler: &ErrorHandler) -> ParseWatResult {
    let raw_result = unsafe {
        ffi::wabt_parse_wat(lexer.raw_lexer, error_handler.raw_buffer)
    };
    ParseWatResult {
        raw_result,
    }
}

struct ReadBinaryResult {
    raw_result: *mut ffi::WabtReadBinaryResult,
}

impl ReadBinaryResult {
    fn is_ok(&self) -> bool {
        unsafe {
            ffi::wabt_read_binary_result_get_result(self.raw_result) == ResultEnum::Ok
        }
    }

    fn module(self) -> Result<*mut ffi::WasmModule, ()> {
        if self.is_ok() {
            unsafe {
                Ok(wabt_read_binary_result_release_module(self.raw_result))
            }
        } else {
            Err(())
        }
    }
}

impl Drop for ReadBinaryResult {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_read_binary_result(self.raw_result);
        }
    }
}

fn read_binary(wasm: &[u8], error_handler: &ErrorHandler) -> ReadBinaryResult {
    let raw_result = unsafe {
         wabt_read_binary(
            wasm.as_ptr(), 
            wasm.len(), 
            true as c_int, 
            error_handler.raw_buffer
        )
    };
    ReadBinaryResult {
        raw_result,
    }
}

struct OutputBuffer {
    raw_buffer: *mut ffi::OutputBuffer,
}

impl OutputBuffer {
    fn data(&self) -> &[u8] {
        unsafe {
            let size = wabt_output_buffer_get_size(self.raw_buffer);
            if size == 0 {
                return &[];
            }
            
            let data = wabt_output_buffer_get_data(self.raw_buffer) as *const u8;

            slice::from_raw_parts(data, size)
        }
    }
}

impl Drop for OutputBuffer {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_output_buffer(self.raw_buffer);
        }
    }
}

struct WriteModuleResult {
    raw_result: *mut ffi::WabtWriteModuleResult,
}

impl WriteModuleResult {
    fn is_ok(&self) -> bool {
        unsafe {
            ffi::wabt_write_module_result_get_result(self.raw_result) == ResultEnum::Ok
        }
    }

    fn output_buffer(self) -> Result<OutputBuffer, ()> {
        if self.is_ok() {
            let raw_buffer = unsafe {
                ffi::wabt_write_module_result_release_output_buffer(self.raw_result)
            };
            Ok(OutputBuffer {
                raw_buffer,
            })
        } else {
            Err(())
        }
    }
}

impl Drop for WriteModuleResult {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_write_module_result(self.raw_result)
        }
    }
}

struct Module {
    raw_module: *mut ffi::WasmModule,
    lexer: Option<Lexer>,
}

impl Module {
    fn parse_wat<S: AsRef<[u8]>>(filename: &str, source: S) -> Result<Module, Error> {
        let lexer = Lexer::new(filename, source.as_ref())?;
        let error_handler = ErrorHandler::new_text();
        match parse_wat(&lexer, &error_handler).module() {
            Ok(module) => Ok(
                Module {
                    raw_module: module,
                    lexer: Some(lexer),
                }
            ),
            Err(()) => {
                let msg = String::from_utf8_lossy(error_handler.raw_message()).to_string();
                Err(Error(ErrorKind::Parse(msg)))
            }
        }
    }

    fn read_binary(wasm: &[u8]) -> Result<Module, Error> {
        let error_handler = ErrorHandler::new_binary();
        match read_binary(wasm, &error_handler).module() {
            Ok(module) => Ok(
                Module {
                    raw_module: module,
                    lexer: None
                }
            ),
            Err(()) => {
                let msg = String::from_utf8_lossy(error_handler.raw_message()).to_string();
                Err(Error(ErrorKind::Deserialize(msg)))
            }
        }
    }

    fn resolve_names(&mut self) -> Result<(), Error> {
        let error_handler = ErrorHandler::new_text();
        unsafe {
            let raw_lexer = self.lexer.as_ref().map(|lexer| lexer.raw_lexer).unwrap_or(ptr::null_mut());
            let result = ffi::wabt_resolve_names_module(raw_lexer, self.raw_module, error_handler.raw_buffer);
            if result == ResultEnum::Error {
                let msg = String::from_utf8_lossy(error_handler.raw_message()).to_string();
                return Err(Error(ErrorKind::ResolveNames(msg)));
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        let error_handler = ErrorHandler::new_text();
        unsafe {
            let raw_lexer = self.lexer.as_ref().map(|lexer| lexer.raw_lexer).unwrap_or(ptr::null_mut());
            let result = wabt_validate_module(raw_lexer, self.raw_module, error_handler.raw_buffer);
            if result == ResultEnum::Error {
                let msg = String::from_utf8_lossy(error_handler.raw_message()).to_string();
                return Err(Error(ErrorKind::Validate(msg)));
            }
        }
        Ok(())
    }

    fn write_binary(&self) -> Result<OutputBuffer, Error> {
        let result = unsafe {
            let raw_result = ffi::wabt_write_binary_module(
                self.raw_module, 0, 1, 0, 0
            );

            WriteModuleResult { raw_result }
        };
        result
            .output_buffer()
            .map_err(|_| Error(ErrorKind::WriteBinary))
    }

    fn write_text(&self) -> Result<OutputBuffer, Error> {
        let result = unsafe {
            let raw_result = ffi::wabt_write_text_module(
                self.raw_module, 0, 0
            );
            WriteModuleResult { raw_result }
        };
        result
            .output_buffer()
            .map_err(|_| Error(ErrorKind::WriteText))
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            wabt_destroy_module(self.raw_module);
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
    let mut module = Module::parse_wat("test.wast", src)?;
    module.resolve_names()?;
    module.validate()?;

    let output_buffer = module.write_binary()?;
    let result = output_buffer.data().to_vec();

    Ok(result)
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
    let module = Module::read_binary(wasm)?;
    let output_buffer = module.write_text()?;
    let text = String::from_utf8(output_buffer.data().to_vec())
        .map_err(|_| Error(ErrorKind::NonUtf8Result))?;
    Ok(text)
}

#[test]
fn module() {
    let binary_module = wat2wasm(r#"
(module
  (import "foo" "bar" (func (param f32)))
  (memory (data "hi"))
  (type (func (param i32) (result i32)))
  (start 1)
  (table 0 1 anyfunc)
  (func)
  (func (type 1)
    i32.const 42
    drop)
  (export "e" (func 1)))
"#).unwrap();

    let mut module = Module::read_binary(&binary_module).unwrap();
    module.resolve_names().unwrap();
    module.validate().unwrap();
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

    assert_eq!(wat2wasm("(modu"), Err(Error(ErrorKind::Parse(
r#"test.wast:1:2: error: unexpected token "modu", expected a module field or a module.
(modu
 ^^^^
"#.to_string()))));
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
        Err(Error(ErrorKind::Deserialize(
            "0000004: error: unable to read uint32_t: version\n".to_owned()
        ))),
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
