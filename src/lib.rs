extern crate wabt_sys;

use std::os::raw::c_void;
use std::ffi::CString;
use std::ptr;

use wabt_sys::*;

/// A structure to represent errors coming out from wabt.
#[derive(Debug, PartialEq, Eq)]
pub struct Error(ErrorKind);

#[derive(Debug, PartialEq, Eq)]
enum ErrorKind {
    Parse,
    ResolveNames,
    Validate,
}

/// Translate wasm text source to wasm binary format.
/// 
/// Returned binary should be executable.
pub fn wat2wasm(src: &str) -> Result<Vec<u8>, Error> {
    let filename = CString::new("test.wast").unwrap();
    let data = CString::new(src).unwrap();

    unsafe {
        let error_handler = wabt_new_text_error_handler_buffer();
        let lexer =
            wabt_new_wast_buffer_lexer(filename.as_ptr(), data.as_ptr() as *const c_void, src.len());

        let result = wabt_parse_wat(lexer, error_handler);
        if wabt_parse_wat_result_get_result(result) == ResultEnum::Error {
            return Err(Error(ErrorKind::Parse));
        }

        let module = wabt_parse_wat_result_release_module(result);

        let result = wabt_resolve_names_module(lexer, module, error_handler);
        if result == ResultEnum::Error {
            return Err(Error(ErrorKind::ResolveNames));
        }

        let result = wabt_validate_module(lexer, module, error_handler);
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
        wabt_destroy_wast_lexer(lexer);
        wabt_destroy_error_handler_buffer(error_handler);

        Ok(result)
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
            )"#).unwrap(),
        &[0, 97, 115, 109, 1, 0, 0, 0]
    );

    assert_eq!(
        wat2wasm("(modu"),
        Err(Error(ErrorKind::Parse))
    );
}
