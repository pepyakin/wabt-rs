use std::os::raw::{c_char, c_int, c_void};

pub enum WastLexer {}
pub enum ErrorHandlerBuffer {}
pub enum WabtParseWatResult {}
pub enum WabtParseWastResult {}
pub enum WasmModule {}
pub enum WabtWriteModuleResult {}
pub enum WabtReadBinaryResult {}
pub enum OutputBuffer {}
pub enum Script {}
pub enum Environment {}
pub enum Executor {}
pub enum DefinedModule {}
pub enum ExecResult {}

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Result {
    Ok,
    Error,
}

pub type ValueType = i32;
pub const VALUETYPE_I32: ValueType = -1;
pub const VALUETYPE_I64: ValueType = -2;
pub const VALUETYPE_F32: ValueType = -3;
pub const VALUETYPE_F64: ValueType = -4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TypedValue {
    pub type_: ValueType,
    pub value: UntypedValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union UntypedValue {
    pub i32: u32,
    pub i64: u64,
    pub f32_bits: u32,
    pub f64_bits: u64,
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

    pub fn wabt_parse_wast(
        lexer: *mut WastLexer,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> *mut WabtParseWastResult;

    pub fn wabt_parse_wat_result_get_result(result: *mut WabtParseWatResult) -> Result;

    pub fn wabt_parse_wat_result_release_module(result: *mut WabtParseWatResult)
        -> *mut WasmModule;

    pub fn wabt_destroy_parse_wat_result(result: *mut WabtParseWatResult);

    pub fn wabt_resolve_names_module(
        lexer: *mut WastLexer,
        module: *mut WasmModule,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> Result;

    pub fn wabt_apply_names_module(
        module: *mut WasmModule,
    ) -> Result;

    pub fn wabt_generate_names_module(
        module: *mut WasmModule,
    ) -> Result;

    pub fn wabt_validate_module(
        lexer: *mut WastLexer,
        module: *mut WasmModule,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> Result;

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

    pub fn wabt_write_module_result_get_result(result: *mut WabtWriteModuleResult) -> Result;

    pub fn wabt_write_module_result_release_output_buffer(
        result: *mut WabtWriteModuleResult,
    ) -> *mut OutputBuffer;

    pub fn wabt_destroy_write_module_result(result: *mut WabtWriteModuleResult);

    pub fn wabt_output_buffer_get_data(buffer: *mut OutputBuffer) -> *const c_void;

    pub fn wabt_output_buffer_get_size(buffer: *mut OutputBuffer) -> usize;

    pub fn wabt_destroy_output_buffer(buffer: *mut OutputBuffer);

    pub fn wabt_resolve_names_script(
        lexer: *mut WastLexer,
        script: *mut Script,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> Result;

    pub fn wabt_validate_script(
        lexer: *mut WastLexer,
        script: *mut Script,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> Result;

    pub fn wabt_write_binary_spec_script(
        script: *mut Script,
        source_filename: *const c_char,
        out_filename: *const c_char,
        log: c_int,
        canonicalize_lebs: c_int,
        relocatable: c_int,
        write_debug_name: c_int,
    ) -> *mut WabtWriteModuleResult;

    pub fn wabt_read_binary(
        data: *const u8,
        size: usize,
        read_debug_names: c_int,
        error_handler: *mut ErrorHandlerBuffer,
    ) -> *mut WabtReadBinaryResult;

    pub fn wabt_parse_wast_result_get_result(
        result: *mut WabtParseWastResult,
    ) -> Result;

    pub fn wabt_parse_wast_result_release_module(
        result: *mut WabtParseWastResult,
    ) -> *mut Script;

    pub fn wabt_destroy_parse_wast_result(
        result: *mut WabtParseWastResult,
    );

    pub fn wabt_read_binary_result_get_result(
        result: *mut WabtReadBinaryResult,
    ) -> Result;

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

    pub fn wabt_interp_create_env() -> *mut Environment;

    pub fn wabt_interp_destroy_env(env: *mut Environment);

    pub fn wabt_interp_read_binary(
        env: *mut Environment,
        data: *const u8,
        size: usize,
        read_debug_names: c_int,
        error_handler: *mut ErrorHandlerBuffer,
        module: *mut *mut DefinedModule,
    ) -> Result;

    pub fn wabt_interp_create_executor(env: *mut Environment) -> *mut Executor;

    pub fn wabt_interp_destroy_executor(exec: *mut Executor);

    pub fn wabt_interp_executor_run_export(
        exec: *mut Executor,
        module: *mut DefinedModule,
        export_name_data: *const u8,
        export_name_len: usize,
        args_data: *const TypedValue,
        args_len: usize,
    ) -> *mut ExecResult;

    pub fn wabt_interp_exec_result_get_result(result: *mut ExecResult) -> Result;
    
    pub fn wabt_interp_exec_result_get_return_size(result: *mut ExecResult) -> usize;

    pub fn wabt_interp_exec_result_get_return(result: *mut ExecResult, index: usize) -> TypedValue;

    pub fn wabt_interp_destroy_exec_result(result: *mut ExecResult);
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
        assert_eq!(wabt_read_binary_result_get_result(result), Result::Ok);
        let module = wabt_read_binary_result_release_module(result);
        
        wabt_destroy_read_binary_result(result);

        let result = wabt_write_text_module(module, 0, 0);
        assert_eq!(wabt_write_module_result_get_result(result), Result::Ok);
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
