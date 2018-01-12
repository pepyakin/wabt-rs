use std::os::raw::{c_char, c_int, c_void};

pub enum WastLexer {}
pub enum ErrorHandlerBuffer {}
pub enum WabtParseWatResult {}
pub enum WasmModule {}
pub enum WabtWriteModuleResult {}
pub enum OutputBuffer {}

#[derive(PartialEq, Eq)]
#[repr(C)]
pub enum ResultEnum {
    Ok,
    Error,
}

extern "C" {
    pub fn wabt_new_wast_buffer_lexer(
        filename: *const c_char,
        data: *const c_void,
        size: usize,
    ) -> *mut WastLexer;

    pub fn wabt_destroy_wast_lexer(lexer: *mut WastLexer);

    pub fn wabt_new_text_error_handler_buffer() -> *mut ErrorHandlerBuffer;

    pub fn wabt_new_binary_error_handler_buffer() -> *mut ErrorHandlerBuffer;

    pub fn wabt_destroy_error_handler_buffer(error_handler: *mut ErrorHandlerBuffer);

    pub fn wabt_parse_wat(
        lexer: *mut WastLexer,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> *mut WabtParseWatResult;

    pub fn wabt_parse_wat_result_get_result(result: *mut WabtParseWatResult) -> ResultEnum;

    pub fn wabt_parse_wat_result_release_module(result: *mut WabtParseWatResult)
        -> *mut WasmModule;

    pub fn wabt_destroy_parse_wat_result(result: *mut WabtParseWatResult);

    pub fn wabt_resolve_names_module(
        lexer: *mut WastLexer,
        module: *mut WasmModule,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> ResultEnum;

    pub fn wabt_validate_module(
        lexer: *mut WastLexer,
        module: *mut WasmModule,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> ResultEnum;

    pub fn wabt_destroy_module(
        module: *mut WasmModule,
    );

    pub fn wabt_write_binary_module(
        module: *mut WasmModule,
        log: c_int,
        canonicalize_lebs: c_int,
        relocatable: c_int,
        write_debug_name: c_int,
    ) -> *mut WabtWriteModuleResult;

    pub fn wabt_write_module_result_get_result(result: *mut WabtWriteModuleResult) -> ResultEnum;

    pub fn wabt_write_module_result_release_output_buffer(
        result: *mut WabtWriteModuleResult,
    ) -> *mut OutputBuffer;

    pub fn wabt_output_buffer_get_data(buffer: *mut OutputBuffer) -> *const c_void;

    pub fn wabt_output_buffer_get_size(buffer: *mut OutputBuffer) -> usize;

    pub fn wabt_destroy_output_buffer(buffer: *mut OutputBuffer);
}

#[test]
fn create_destroy_lexer() {
    use std::ffi::CString;
    use std::slice;

    let filename = CString::new("test.wast").unwrap();
    let data = CString::new("(module)").unwrap();

    unsafe {
        let error_handler = wabt_new_text_error_handler_buffer();
        let lexer =
            wabt_new_wast_buffer_lexer(filename.as_ptr(), data.as_ptr() as *const c_void, 8);

        let result = wabt_parse_wat(lexer, error_handler);
        assert!(wabt_parse_wat_result_get_result(result) == ResultEnum::Ok);

        let module = wabt_parse_wat_result_release_module(result);

        let result = wabt_resolve_names_module(lexer, module, error_handler);
        assert!(result == ResultEnum::Ok);

        let result = wabt_validate_module(lexer, module, error_handler);
        assert!(result == ResultEnum::Ok);

        let result = wabt_write_binary_module(module, 0, 1, 0, 0);
        assert!(wabt_write_module_result_get_result(result) == ResultEnum::Ok);

        let output_buffer = wabt_write_module_result_release_output_buffer(result);

        let data = slice::from_raw_parts(
            wabt_output_buffer_get_data(output_buffer) as *const u8,
            wabt_output_buffer_get_size(output_buffer),
        );

        assert_eq!(
            data,
            &[
                0, 97, 115, 109, // \0ASM - magic
                1, 0, 0, 0       //    01 - version
            ]
        );

        wabt_destroy_output_buffer(output_buffer);
        wabt_destroy_wast_lexer(lexer);
        wabt_destroy_error_handler_buffer(error_handler);
    }
}
