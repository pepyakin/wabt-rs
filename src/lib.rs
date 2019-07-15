//! Bindings to the [wabt](https://github.com/WebAssembly/wabt) library.
//!

#![deny(missing_docs)]

extern crate serde;
extern crate serde_json;
extern crate wabt_sys;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::slice;

use wabt_sys as ffi;

pub mod script;

/// A structure to represent errors coming out from wabt.
#[derive(Debug, PartialEq, Eq)]
pub struct Error(ErrorKind);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: A better formatting
        write!(f, "error: {:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.0 {
            ErrorKind::Nul => "string contained nul-byte",
            ErrorKind::Deserialize(_) => "failed to deserialize",
            ErrorKind::Parse(_) => "failed to parse",
            ErrorKind::WriteText => "failed to write text",
            ErrorKind::NonUtf8Result => "result is not a valid utf8",
            ErrorKind::WriteBinary => "failed to write binary",
            ErrorKind::ResolveNames(_) => "failed to resolve names",
            ErrorKind::Validate(_) => "failed to validate",
        }
    }
}

/// ErrorKind describes an error condition from a wasm module operation, as well as the
/// corresponding error message from `wabt`, if any.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Result contained an unexpected null byte.
    Nul,
    /// Error deserializing binary wasm.
    Deserialize(String),
    /// Error parsing textual wasm.
    Parse(String),
    /// Error serializing a wasm module to text.
    WriteText,
    /// Translating a wasm binary module to text yielded non-utf8 characters.
    NonUtf8Result,
    /// Error serializing a wasm module to binary.
    WriteBinary,
    /// Error resolving names in the wasm module.
    ResolveNames(String),
    /// Error validating the wasm module.
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
        // TODO: Don't copy.
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

struct Errors {
    raw: *mut ffi::Errors,
}

impl Errors {
    fn new() -> Errors {
        Errors {
            raw: unsafe { ffi::wabt_new_errors() },
        }
    }

    fn format_text(&self, lexer: &Lexer) -> WabtBuf {
        unsafe {
            let raw_buffer = ffi::wabt_format_text_errors(self.raw, lexer.raw_lexer);
            WabtBuf { raw_buffer }
        }
    }

    fn format_binary(&self) -> WabtBuf {
        unsafe {
            let raw_buffer = ffi::wabt_format_binary_errors(self.raw);
            WabtBuf { raw_buffer }
        }
    }
}

impl Drop for Errors {
    fn drop(&mut self) {
        unsafe { ffi::wabt_destroy_errors(self.raw) }
    }
}

/// Represents which WebAssembly features are enabled in Wabt.
pub struct Features {
    raw: *mut ffi::Features,
}

impl Clone for Features {
    fn clone(&self) -> Self {
        let mut new = Features::new();
        new.set_exceptions_enabled(self.exceptions_enabled());
        new.set_mutable_globals_enabled(self.mutable_globals_enabled());
        new.set_sat_float_to_int_enabled(self.sat_float_to_int_enabled());
        new.set_sign_extension_enabled(self.sign_extension_enabled());
        new.set_simd_enabled(self.simd_enabled());
        new.set_threads_enabled(self.threads_enabled());
        new.set_multi_value_enabled(self.multi_value_enabled());
        new.set_tail_call_enabled(self.tail_call_enabled());
        new.set_bulk_memory_enabled(self.bulk_memory_enabled());
        new
    }
}

impl Features {
    #![allow(missing_docs)]
    pub fn new() -> Features {
        let raw = unsafe { ffi::wabt_new_features() };
        Features { raw }
    }

    pub fn enable_all(&mut self) {
        self.enable_exceptions();
        self.enable_mutable_globals();
        self.enable_sat_float_to_int();
        self.enable_sign_extension();
        self.enable_simd();
        self.enable_threads();
        self.enable_multi_value();
        self.enable_tail_call();
        self.enable_bulk_memory();
    }

    pub fn exceptions_enabled(&self) -> bool {
        unsafe { ffi::wabt_exceptions_enabled(self.raw) }
    }
    pub fn enable_exceptions(&mut self) {
        self.set_exceptions_enabled(true);
    }
    pub fn disable_exceptions(&mut self) {
        self.set_exceptions_enabled(false);
    }
    pub fn set_exceptions_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_exceptions_enabled(self.raw, value.into());
        }
    }

    pub fn mutable_globals_enabled(&self) -> bool {
        unsafe { ffi::wabt_mutable_globals_enabled(self.raw) }
    }
    pub fn enable_mutable_globals(&mut self) {
        self.set_mutable_globals_enabled(true);
    }
    pub fn disable_mutable_globals(&mut self) {
        self.set_mutable_globals_enabled(false);
    }
    pub fn set_mutable_globals_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_mutable_globals_enabled(self.raw, value.into());
        }
    }

    pub fn sat_float_to_int_enabled(&self) -> bool {
        unsafe { ffi::wabt_sat_float_to_int_enabled(self.raw) }
    }
    pub fn enable_sat_float_to_int(&mut self) {
        self.set_sat_float_to_int_enabled(true);
    }
    pub fn disable_sat_float_to_int(&mut self) {
        self.set_sat_float_to_int_enabled(false);
    }
    pub fn set_sat_float_to_int_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_sat_float_to_int_enabled(self.raw, value.into());
        }
    }

    pub fn sign_extension_enabled(&self) -> bool {
        unsafe { ffi::wabt_sign_extension_enabled(self.raw) }
    }
    pub fn enable_sign_extension(&mut self) {
        self.set_sign_extension_enabled(true);
    }
    pub fn disable_sign_extension(&mut self) {
        self.set_sign_extension_enabled(false);
    }
    pub fn set_sign_extension_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_sign_extension_enabled(self.raw, value.into());
        }
    }

    pub fn simd_enabled(&self) -> bool {
        unsafe { ffi::wabt_simd_enabled(self.raw) }
    }
    pub fn enable_simd(&mut self) {
        self.set_simd_enabled(true);
    }
    pub fn disable_simd(&mut self) {
        self.set_simd_enabled(false);
    }
    pub fn set_simd_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_simd_enabled(self.raw, value.into());
        }
    }

    pub fn threads_enabled(&self) -> bool {
        unsafe { ffi::wabt_threads_enabled(self.raw) }
    }
    pub fn enable_threads(&mut self) {
        self.set_threads_enabled(true);
    }
    pub fn disable_threads(&mut self) {
        self.set_threads_enabled(false);
    }
    pub fn set_threads_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_threads_enabled(self.raw, value.into());
        }
    }

    pub fn multi_value_enabled(&self) -> bool {
        unsafe { ffi::wabt_multi_value_enabled(self.raw) }
    }
    pub fn enable_multi_value(&mut self) {
        self.set_multi_value_enabled(true);
    }
    pub fn disable_multi_value(&mut self) {
        self.set_multi_value_enabled(false);
    }
    pub fn set_multi_value_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_multi_value_enabled(self.raw, value.into());
        }
    }

    pub fn tail_call_enabled(&self) -> bool {
        unsafe { ffi::wabt_tail_call_enabled(self.raw) }
    }
    pub fn enable_tail_call(&mut self) {
        self.set_tail_call_enabled(true);
    }
    pub fn disable_tail_call(&mut self) {
        self.set_tail_call_enabled(false);
    }
    pub fn set_tail_call_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_tail_call_enabled(self.raw, value.into());
        }
    }

    pub fn bulk_memory_enabled(&self) -> bool {
        unsafe { ffi::wabt_bulk_memory_enabled(self.raw) }
    }
    pub fn enable_bulk_memory(&mut self) {
        self.set_bulk_memory_enabled(true);
    }
    pub fn disable_bulk_memory(&mut self) {
        self.set_bulk_memory_enabled(false);
    }
    pub fn set_bulk_memory_enabled(&mut self, value: bool) {
        unsafe {
            ffi::wabt_set_bulk_memory_enabled(self.raw, value.into());
        }
    }
}

impl Drop for Features {
    fn drop(&mut self) {
        unsafe { ffi::wabt_destroy_features(self.raw) }
    }
}

struct ParseWatResult {
    raw_result: *mut ffi::WabtParseWatResult,
}

impl ParseWatResult {
    fn is_ok(&self) -> bool {
        unsafe { ffi::wabt_parse_wat_result_get_result(self.raw_result) == ffi::Result::Ok }
    }

    fn take_module(self) -> Result<*mut ffi::WasmModule, ()> {
        if self.is_ok() {
            unsafe { Ok(ffi::wabt_parse_wat_result_release_module(self.raw_result)) }
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

fn parse_wat(lexer: &Lexer, features: &Features, errors: &Errors) -> ParseWatResult {
    let raw_result = unsafe { ffi::wabt_parse_wat(lexer.raw_lexer, features.raw, errors.raw) };
    ParseWatResult { raw_result }
}

struct ReadBinaryResult {
    raw_result: *mut ffi::WabtReadBinaryResult,
}

impl ReadBinaryResult {
    fn is_ok(&self) -> bool {
        unsafe { ffi::wabt_read_binary_result_get_result(self.raw_result) == ffi::Result::Ok }
    }

    fn take_module(self) -> Result<*mut ffi::WasmModule, ()> {
        if self.is_ok() {
            unsafe { Ok(ffi::wabt_read_binary_result_release_module(self.raw_result)) }
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

/// Buffer returned by wabt.
///
/// # Examples
///
/// You can convert it either to `Vec`:
///
/// ```rust
/// # extern crate wabt;
/// # let wabt_buf = wabt::Wat2Wasm::new().convert("(module)").unwrap();
/// let vec: Vec<u8> = wabt_buf.as_ref().to_vec();
/// ```
///
/// Or in `String`:
///
/// ```rust
/// # extern crate wabt;
/// # let wabt_buf = wabt::Wat2Wasm::new().convert("(module)").unwrap();
/// let text = String::from_utf8(wabt_buf.as_ref().to_vec()).unwrap();
/// ```
///
pub struct WabtBuf {
    raw_buffer: *mut ffi::OutputBuffer,
}

impl AsRef<[u8]> for WabtBuf {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let size = ffi::wabt_output_buffer_get_size(self.raw_buffer);
            if size == 0 {
                return &[];
            }

            let data = ffi::wabt_output_buffer_get_data(self.raw_buffer) as *const u8;

            slice::from_raw_parts(data, size)
        }
    }
}

impl Drop for WabtBuf {
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
        unsafe { ffi::wabt_write_module_result_get_result(self.raw_result) == ffi::Result::Ok }
    }

    fn take_wabt_buf(self) -> Result<WabtBuf, ()> {
        if self.is_ok() {
            let raw_buffer =
                unsafe { ffi::wabt_write_module_result_release_output_buffer(self.raw_result) };
            Ok(WabtBuf { raw_buffer })
        } else {
            Err(())
        }
    }
}

impl Drop for WriteModuleResult {
    fn drop(&mut self) {
        unsafe { ffi::wabt_destroy_write_module_result(self.raw_result) }
    }
}

struct WriteBinaryOptions {
    log: bool,
    canonicalize_lebs: bool,
    relocatable: bool,
    write_debug_names: bool,
}

impl Default for WriteBinaryOptions {
    fn default() -> WriteBinaryOptions {
        WriteBinaryOptions {
            log: false,
            canonicalize_lebs: true,
            relocatable: false,
            write_debug_names: false,
        }
    }
}

struct WriteTextOptions {
    fold_exprs: bool,
    inline_export: bool,
}

impl Default for WriteTextOptions {
    fn default() -> WriteTextOptions {
        WriteTextOptions {
            fold_exprs: false,
            inline_export: false,
        }
    }
}

/// Options for reading read binary.
pub struct ReadBinaryOptions {
    read_debug_names: bool,
}

impl Default for ReadBinaryOptions {
    fn default() -> ReadBinaryOptions {
        ReadBinaryOptions {
            read_debug_names: false,
        }
    }
}

struct ParseWastResult {
    raw_result: *mut ffi::WabtParseWastResult,
}

impl ParseWastResult {
    fn is_ok(&self) -> bool {
        unsafe { ffi::wabt_parse_wast_result_get_result(self.raw_result) == ffi::Result::Ok }
    }

    fn take_script(self) -> Result<*mut ffi::Script, ()> {
        if self.is_ok() {
            unsafe { Ok(ffi::wabt_parse_wast_result_release_module(self.raw_result)) }
        } else {
            Err(())
        }
    }
}

impl Drop for ParseWastResult {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_parse_wast_result(self.raw_result);
        }
    }
}

fn parse_wast(lexer: &Lexer, features: &Features, errors: &Errors) -> ParseWastResult {
    let raw_result = unsafe { ffi::wabt_parse_wast(lexer.raw_lexer, features.raw, errors.raw) };
    ParseWastResult { raw_result }
}

struct Script {
    raw_script: *mut ffi::Script,
    lexer: Lexer,
    features: Features,
}

impl Script {
    fn parse<S: AsRef<[u8]>>(
        filename: &str,
        source: S,
        features: Features,
    ) -> Result<Script, Error> {
        let lexer = Lexer::new(filename, source.as_ref())?;
        let errors = Errors::new();
        match parse_wast(&lexer, &features, &errors).take_script() {
            Ok(raw_script) => Ok(Script {
                raw_script,
                features,
                lexer,
            }),
            Err(()) => {
                let msg = String::from_utf8_lossy(errors.format_text(&lexer).as_ref()).to_string();
                Err(Error(ErrorKind::Parse(msg)))
            }
        }
    }

    fn resolve_names(&self) -> Result<(), Error> {
        let errors = Errors::new();
        unsafe {
            let result = ffi::wabt_resolve_names_script(self.raw_script, errors.raw);
            if result == ffi::Result::Error {
                let msg =
                    String::from_utf8_lossy(errors.format_text(&self.lexer).as_ref()).to_string();
                return Err(Error(ErrorKind::ResolveNames(msg)));
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        let errors = Errors::new();
        unsafe {
            let result = ffi::wabt_validate_script(self.raw_script, self.features.raw, errors.raw);
            if result == ffi::Result::Error {
                let msg =
                    String::from_utf8_lossy(errors.format_text(&self.lexer).as_ref()).to_string();
                return Err(Error(ErrorKind::Validate(msg)));
            }
        }
        Ok(())
    }

    fn write_binaries(&self, source: &str) -> Result<WabtWriteScriptResult, Error> {
        let source_cstr = CString::new(source)?;

        unsafe {
            let raw_script_result = ffi::wabt_write_binary_spec_script(
                self.raw_script,
                source_cstr.as_ptr(),
                ptr::null(),
                0,
                1,
                0,
                0,
            );
            Ok(WabtWriteScriptResult { raw_script_result })
        }
    }
}

/// WebAssembly module.
pub struct Module {
    raw_module: *mut ffi::WasmModule,
    lexer: Option<Lexer>,
    features: Features,
}

impl Module {
    /// Parse source in WebAssembly text format.
    pub fn parse_wat<S: AsRef<[u8]>>(
        filename: &str,
        source: S,
        features: Features,
    ) -> Result<Module, Error> {
        let lexer = Lexer::new(filename, source.as_ref())?;
        let errors = Errors::new();
        match parse_wat(&lexer, &features, &errors).take_module() {
            Ok(module) => Ok(Module {
                raw_module: module,
                features,
                lexer: Some(lexer),
            }),
            Err(()) => {
                let msg = String::from_utf8_lossy(errors.format_text(&lexer).as_ref()).to_string();
                Err(Error(ErrorKind::Parse(msg)))
            }
        }
    }

    /// Read WebAssembly binary.
    ///
    /// `read_binary` doesn't do any validation. If you want to validate, you can the module you can
    /// call [`validate`].
    ///
    /// [`validate`]: #method.validate
    pub fn read_binary<S: AsRef<[u8]>>(
        wasm: S,
        options: &ReadBinaryOptions,
    ) -> Result<Module, Error> {
        let errors = Errors::new();
        let features = Features::new();
        let result = {
            let wasm = wasm.as_ref();
            let raw_result = unsafe {
                ffi::wabt_read_binary(
                    wasm.as_ptr(),
                    wasm.len(),
                    options.read_debug_names as c_int,
                    features.raw,
                    errors.raw,
                )
            };
            ReadBinaryResult { raw_result }
        };
        match result.take_module() {
            Ok(module) => Ok(Module {
                raw_module: module,
                features,
                lexer: None,
            }),
            Err(()) => {
                let msg = String::from_utf8_lossy(errors.format_binary().as_ref()).to_string();
                Err(Error(ErrorKind::Deserialize(msg)))
            }
        }
    }

    fn resolve_names(&mut self) -> Result<(), Error> {
        let errors = Errors::new();
        unsafe {
            let result = ffi::wabt_resolve_names_module(self.raw_module, errors.raw);
            if result == ffi::Result::Error {
                let buf = if let Some(ref lexer) = self.lexer {
                    errors.format_text(lexer)
                } else {
                    errors.format_binary()
                };
                let msg = String::from_utf8_lossy(buf.as_ref()).to_string();
                return Err(Error(ErrorKind::ResolveNames(msg)));
            }
        }
        Ok(())
    }

    /// Validate the module.
    pub fn validate(&self) -> Result<(), Error> {
        let errors = Errors::new();
        unsafe {
            let result = ffi::wabt_validate_module(self.raw_module, self.features.raw, errors.raw);
            if result == ffi::Result::Error {
                let buf = if let Some(ref lexer) = self.lexer {
                    errors.format_text(lexer)
                } else {
                    errors.format_binary()
                };
                let msg = String::from_utf8_lossy(buf.as_ref()).to_string();
                return Err(Error(ErrorKind::Validate(msg)));
            }
        }
        Ok(())
    }

    fn write_binary(&self, options: &WriteBinaryOptions) -> Result<WabtBuf, Error> {
        let result = unsafe {
            let raw_result = ffi::wabt_write_binary_module(
                self.raw_module,
                options.log as c_int,
                options.canonicalize_lebs as c_int,
                options.relocatable as c_int,
                options.write_debug_names as c_int,
            );
            WriteModuleResult { raw_result }
        };
        result
            .take_wabt_buf()
            .map_err(|_| Error(ErrorKind::WriteBinary))
    }

    fn write_text(&self, options: &WriteTextOptions) -> Result<WabtBuf, Error> {
        let result = unsafe {
            let raw_result = ffi::wabt_write_text_module(
                self.raw_module,
                options.fold_exprs as c_int,
                options.inline_export as c_int,
            );
            WriteModuleResult { raw_result }
        };
        result
            .take_wabt_buf()
            .map_err(|_| Error(ErrorKind::WriteText))
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_module(self.raw_module);
        }
    }
}

/// A builder for translating wasm text source to wasm binary format.
///
/// This version allows you to tweak parameters. If you need simple version
/// check out [`wat2wasm`].
///
/// [`wat2wasm`]: fn.wat2wasm.html
///
/// # Examples
///
/// ```rust
/// extern crate wabt;
/// use wabt::Wat2Wasm;
///
/// fn main() {
///     let wasm_binary = Wat2Wasm::new()
///         .canonicalize_lebs(false)
///         .write_debug_names(true)
///         .convert(
///             r#"
///                 (module
///                     (import "spectest" "print" (func $print (param i32)))
///                     (func (export "main")
///                         i32.const 1312
///                         call $print
///                     )
///                 )
///             "#
///         ).unwrap();
///
///     # wasm_binary;
/// }
/// ```
///
pub struct Wat2Wasm {
    validate: bool,
    write_binary_options: WriteBinaryOptions,
    features: Features,
}

impl Wat2Wasm {
    /// Create `Wat2Wasm` with default configuration.
    pub fn new() -> Wat2Wasm {
        Wat2Wasm {
            write_binary_options: WriteBinaryOptions::default(),
            validate: true,
            features: Features::new(),
        }
    }

    /// Write canonicalized LEB128 for var ints.
    ///
    /// Set this to `false` to write all LEB128 sizes as 5-bytes instead of their minimal size.
    /// `true` by default.
    pub fn canonicalize_lebs(&mut self, canonicalize_lebs: bool) -> &mut Wat2Wasm {
        self.write_binary_options.canonicalize_lebs = canonicalize_lebs;
        self
    }

    /// Create a relocatable wasm binary
    ///
    /// (suitable for linking with wasm-link).
    /// `false` by default.
    pub fn relocatable(&mut self, relocatable: bool) -> &mut Wat2Wasm {
        self.write_binary_options.relocatable = relocatable;
        self
    }

    /// Write debug names to the generated binary file
    ///
    /// `false` by default.
    pub fn write_debug_names(&mut self, write_debug_names: bool) -> &mut Wat2Wasm {
        self.write_binary_options.write_debug_names = write_debug_names;
        self
    }

    /// Check for validity of module before writing.
    ///
    /// `true` by default.
    pub fn validate(&mut self, validate: bool) -> &mut Wat2Wasm {
        self.validate = validate;
        self
    }

    // TODO: Add logged version of convert

    /// Perform conversion.
    pub fn convert<S: AsRef<[u8]>>(&self, source: S) -> Result<WabtBuf, Error> {
        let mut module = Module::parse_wat("test.wast", source, self.features.clone())?;
        module.resolve_names()?;

        if self.validate {
            module.validate()?;
        }

        let result = module.write_binary(&self.write_binary_options)?;
        Ok(result)
    }
}

/// A builder for converting wasm binary to wasm text format.
///
/// # Examples
///
/// ```rust
/// extern crate wabt;
/// use wabt::Wasm2Wat;
///
/// fn main() {
///     let wasm_text = Wasm2Wat::new()
///         .fold_exprs(true)
///         .inline_export(true)
///         .convert(
///             &[
///                 0, 97, 115, 109, // \0ASM - magic
///                 1, 0, 0, 0       //  0x01 - version
///             ]
///         ).unwrap();
///
///     # wasm_text;
/// }
/// ```
///
pub struct Wasm2Wat {
    read_binary_options: ReadBinaryOptions,
    write_text_options: WriteTextOptions,
}

impl Wasm2Wat {
    /// Create `Wasm2Wat` with default configuration.
    pub fn new() -> Wasm2Wat {
        Wasm2Wat {
            read_binary_options: ReadBinaryOptions::default(),
            write_text_options: WriteTextOptions::default(),
        }
    }

    /// Read debug names in the binary file.
    ///
    /// `false` by default.
    pub fn read_debug_names(&mut self, read_debug_names: bool) -> &mut Wasm2Wat {
        self.read_binary_options.read_debug_names = read_debug_names;
        self
    }

    /// Write folded expressions where possible.
    ///
    /// Example of folded code (if `true`):
    ///
    /// ```WebAssembly
    /// (module
    ///     (func (param i32 i32) (result i32)
    ///         (i32.add ;; Add loaded arguments
    ///             (get_local 0) ;; Load first arg
    ///             (get_local 1) ;; Load second arg
    ///         )
    ///     )
    /// )
    /// ```
    ///
    /// Example of straight code (if `false`):
    ///
    /// ```WebAssembly
    /// (module
    ///     (func (param i32 i32) (result i32)
    ///         get_local 0 ;; Load first arg
    ///         get_local 1 ;; Load second arg
    ///         i32.add     ;; Add loaded arguments
    ///     )
    /// )
    /// ```
    ///
    /// `false` by default.
    pub fn fold_exprs(&mut self, fold_exprs: bool) -> &mut Wasm2Wat {
        self.write_text_options.fold_exprs = fold_exprs;
        self
    }

    /// Write all exports inline.
    ///
    /// Example of code with inline exports (if `true`):
    ///
    /// ```WebAssembly
    /// (module
    /// (func $addTwo (export "addTwo") (param $p0 i32) (param $p1 i32) (result i32)
    ///   (i32.add
    ///     (get_local $p0)
    ///     (get_local $p1))))
    /// ```
    ///
    /// Example of code with separate exports (if `false`):
    ///
    /// ```WebAssembly
    /// (module
    ///   (func $addTwo (param $p0 i32) (param $p1 i32) (result i32)
    ///     (i32.add
    ///       (get_local $p0)
    ///       (get_local $p1)))
    ///   (export "addTwo" (func $addTwo)))
    /// ```
    ///
    /// `false` by default.
    pub fn inline_export(&mut self, inline_export: bool) -> &mut Wasm2Wat {
        self.write_text_options.inline_export = inline_export;
        self
    }

    /// Perform conversion.
    pub fn convert<S: AsRef<[u8]>>(&self, wasm: S) -> Result<WabtBuf, Error> {
        let module = Module::read_binary(wasm, &self.read_binary_options)?;
        let output_buffer = module.write_text(&self.write_text_options)?;
        Ok(output_buffer)
    }
}

/// Translate wasm text source to wasm binary format.
///
/// If wasm source is valid wasm binary will be returned in the vector.
/// Returned binary is validated and can be executed.
///
/// This function will make translation with default parameters.
/// If you want to find out what default parameters are or you want to tweak them
/// you can use [`Wat2Wasm`]
///
/// For more examples and online demo you can check online version
/// of [wat2wasm](https://cdn.rawgit.com/WebAssembly/wabt/aae5a4b7/demo/wat2wasm/).
///
/// [`Wat2Wasm`]: struct.Wat2Wasm.html
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
pub fn wat2wasm<S: AsRef<[u8]>>(source: S) -> Result<Vec<u8>, Error> {
    let result_buf = Wat2Wasm::new().convert(source)?;
    Ok(result_buf.as_ref().to_vec())
}

/// Translate wasm text source to wasm binary format.
///
/// If wasm source is valid wasm binary will be returned in the vector.
/// Returned binary is validated and can be executed.
///
/// This function will make translation with custom features.
/// If you want to find out what default parameters are or you want to tweak them
/// you can use [`Wat2Wasm`]
///
/// For more examples and online demo you can check online version
/// of [wat2wasm](https://cdn.rawgit.com/WebAssembly/wabt/aae5a4b7/demo/wat2wasm/).
///
/// [`Wat2Wasm`]: struct.Wat2Wasm.html
///
/// # Examples
///
/// ```rust
/// extern crate wabt;
/// use wabt::{Features, wat2wasm_with_features};
///
/// fn main() {
///     let mut features = Features::new();
///     features.enable_simd();
///     assert_eq!(
///         wat2wasm_with_features("(module)", features).unwrap(),
///         &[
///             0, 97, 115, 109, // \0ASM - magic
///             1, 0, 0, 0       //  0x01 - version
///         ]
///     );
/// }
/// ```
///
pub fn wat2wasm_with_features<S: AsRef<[u8]>>(
    source: S,
    features: Features,
) -> Result<Vec<u8>, Error> {
    let mut wat2wasm = Wat2Wasm::new();
    wat2wasm.features = features;
    let result_buf = wat2wasm.convert(source)?;
    Ok(result_buf.as_ref().to_vec())
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
pub fn wasm2wat<S: AsRef<[u8]>>(wasm: S) -> Result<String, Error> {
    let result_buf = Wasm2Wat::new().convert(wasm)?;
    let text = String::from_utf8(result_buf.as_ref().to_vec())
        .map_err(|_| Error(ErrorKind::NonUtf8Result))?;
    Ok(text)
}

struct WabtWriteScriptResult {
    raw_script_result: *mut ffi::WabtWriteScriptResult,
}

struct WabtWriteScriptResultRelease {
    json_output_buffer: WabtBuf,
    _log_output_buffer: WabtBuf,
    module_output_buffers: HashMap<CString, WabtBuf>,
}

impl WabtWriteScriptResult {
    fn is_ok(&self) -> bool {
        unsafe {
            ffi::wabt_write_script_result_get_result(self.raw_script_result) == ffi::Result::Ok
        }
    }

    fn module_count(&self) -> usize {
        unsafe { ffi::wabt_write_script_result_get_module_count(self.raw_script_result) }
    }

    fn module_filename(&self, index: usize) -> &CStr {
        assert!(index < self.module_count());
        unsafe {
            let s =
                ffi::wabt_write_script_result_get_module_filename(self.raw_script_result, index);
            CStr::from_ptr(s)
        }
    }

    fn take_all(self) -> Result<WabtWriteScriptResultRelease, ()> {
        if self.is_ok() {
            let json_output_buffer;
            let log_output_buffer;
            let mut module_output_buffers = HashMap::new();
            unsafe {
                json_output_buffer = ffi::wabt_write_script_result_release_json_output_buffer(
                    self.raw_script_result,
                );
                log_output_buffer =
                    ffi::wabt_write_script_result_release_log_output_buffer(self.raw_script_result);
            }
            for i in 0..self.module_count() {
                let module_output_buffer = unsafe {
                    ffi::wabt_write_script_result_release_module_output_buffer(
                        self.raw_script_result,
                        i,
                    )
                };
                let name = self.module_filename(i);
                module_output_buffers.insert(
                    name.to_owned(),
                    WabtBuf {
                        raw_buffer: module_output_buffer,
                    },
                );
            }
            Ok(WabtWriteScriptResultRelease {
                json_output_buffer: WabtBuf {
                    raw_buffer: json_output_buffer,
                },
                _log_output_buffer: WabtBuf {
                    raw_buffer: log_output_buffer,
                },
                module_output_buffers,
            })
        } else {
            Err(())
        }
    }
}

impl Drop for WabtWriteScriptResult {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_destroy_write_script_result(self.raw_script_result);
        }
    }
}

#[test]
fn features() {
    let example_wat = r#"
(module
  (func $simd (result v128)
    (v128.const i8x16 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16)
    return)
)"#;

    assert!(wat2wasm(example_wat).is_err());

    let mut features = Features::new();
    features.enable_simd();
    assert!(wat2wasm_with_features(example_wat, features).is_ok());
}

#[test]
fn module() {
    let binary_module = wat2wasm(
        r#"
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
"#,
    )
    .unwrap();

    let mut module = Module::read_binary(&binary_module, &ReadBinaryOptions::default()).unwrap();
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
            )"#
        )
        .unwrap(),
        &[0, 97, 115, 109, 1, 0, 0, 0]
    );

    assert_eq!(
        wat2wasm("(modu"),
        Err(Error(ErrorKind::Parse(
            r#"test.wast:1:2: error: unexpected token "modu", expected a module field or a module.
(modu
 ^^^^
"#
            .to_string()
        )))
    );
}

#[test]
fn test_wasm2wat() {
    assert_eq!(
        wasm2wat(&[
            0, 97, 115, 109, // \0ASM - magic
            1, 0, 0, 0 //    01 - version
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
    let factorial: &[u8] = &[
        0, 97, 115, 109, 1, 0, 0, 0, 1, 6, 1, 96, 1, 124, 1, 124, 3, 2, 1, 0, 7, 7,
        1, 3, 102, 97, 99, 0, 0, 10, 46, 1, 44, 0, 32, 0, 68, 0, 0, 0, 0, 0, 0, 240,
        63, 99, 4, 124, 68, 0, 0, 0, 0, 0, 0, 240, 63, 5, 32, 0, 32, 0, 68, 0, 0, 0,
        0, 0, 0, 240, 63, 161, 16, 0, 162, 11, 11
    ];

    let text = wasm2wat(&factorial).unwrap();
    let binary = wat2wasm(&text).unwrap();

    assert_eq!(&*factorial, &*binary);
}
