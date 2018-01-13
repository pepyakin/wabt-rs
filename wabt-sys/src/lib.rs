use std::os::raw::{c_char, c_int, c_void};

pub enum WastLexer {}
pub enum ErrorHandlerBuffer {}
pub enum WabtParseWatResult {}
pub enum WasmModule {}
pub enum WabtWriteModuleResult {}
pub enum WabtReadBinaryResult {}
pub enum OutputBuffer {}

#[derive(Debug, PartialEq, Eq)]
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

    pub fn wabt_error_handler_buffer_get_data(
        error_handler: *mut ErrorHandlerBuffer
    ) -> *const c_void;

    pub fn wabt_error_handler_buffer_get_size(
        error_handler: *mut ErrorHandlerBuffer
    ) -> usize;

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

    pub fn wabt_apply_names_module(
        module: *mut WasmModule,
    ) -> ResultEnum;

    pub fn wabt_generate_names_module(
        module: *mut WasmModule,
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

    pub fn wabt_destroy_write_module_result(result: *mut WabtWriteModuleResult);

    pub fn wabt_output_buffer_get_data(buffer: *mut OutputBuffer) -> *const c_void;

    pub fn wabt_output_buffer_get_size(buffer: *mut OutputBuffer) -> usize;

    pub fn wabt_destroy_output_buffer(buffer: *mut OutputBuffer);

    pub fn wabt_read_binary(
        data: *const u8,
        size: usize,
        read_debug_names: c_int,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> *mut WabtReadBinaryResult;

    pub fn wabt_read_binary_result_get_result(
        result: *mut WabtReadBinaryResult,
    ) -> ResultEnum;

    pub fn wabt_read_binary_result_release_module(
        result: *mut WabtReadBinaryResult,
    ) -> *mut WasmModule;

    pub fn wabt_destroy_read_binary_result(
        result: *mut WabtReadBinaryResult,
    );

    pub fn wabt_write_text_module(
        module: *mut WasmModule,
        fold_exprs: c_int,
        inline_export: c_int,
    ) -> *mut WabtWriteModuleResult;
}

#[test]
fn parse_wasm() {
    use std::ptr;

    let data: &[u8] = &[
        0, 97, 115, 109, // \0ASM - magic
        1, 0, 0, 0       //    01 - version
    ];

    unsafe {
        let error_handler = wabt_new_binary_error_handler_buffer();
        let result = wabt_read_binary(
            data.as_ptr(),
            data.len(),
            true as c_int,
            error_handler,
        );
        assert_eq!(wabt_read_binary_result_get_result(result), ResultEnum::Ok);
        let module = wabt_read_binary_result_release_module(result);
        
        wabt_destroy_read_binary_result(result);

        let result = wabt_write_text_module(module, 0, 0);
        assert_eq!(wabt_write_module_result_get_result(result), ResultEnum::Ok);
        let output_buffer = wabt_write_module_result_release_output_buffer(result);

        let data = wabt_output_buffer_get_data(output_buffer);
        let size = wabt_output_buffer_get_size(output_buffer);

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        ptr::copy_nonoverlapping(data as *const u8, buf.as_mut_ptr(), size);
        buf.set_len(size);

        let text = String::from_utf8(buf).unwrap();
        assert_eq!(&text, "(module)\n");
    }
}
