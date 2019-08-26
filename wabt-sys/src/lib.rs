use std::os::raw::{c_char, c_int, c_void};

pub enum Features {}
pub enum Errors {}
pub enum WastLexer {}
pub enum WabtParseWatResult {}
pub enum WabtParseWastResult {}
pub enum WasmModule {}
pub enum WabtWriteModuleResult {}
pub enum WabtReadBinaryResult {}
pub enum OutputBuffer {}
pub enum Script {}
pub enum WabtWriteScriptResult {}

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Result {
    Ok,
    Error,
}

extern "C" {
    pub fn wabt_new_features() -> *mut Features;

    pub fn wabt_exceptions_enabled(features: *const Features) -> bool;
    pub fn wabt_set_exceptions_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_mutable_globals_enabled(features: *const Features) -> bool;
    pub fn wabt_set_mutable_globals_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_sat_float_to_int_enabled(features: *const Features) -> bool;
    pub fn wabt_set_sat_float_to_int_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_sign_extension_enabled(features: *const Features) -> bool;
    pub fn wabt_set_sign_extension_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_simd_enabled(features: *const Features) -> bool;
    pub fn wabt_set_simd_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_threads_enabled(features: *const Features) -> bool;
    pub fn wabt_set_threads_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_multi_value_enabled(features: *const Features) -> bool;
    pub fn wabt_set_multi_value_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_tail_call_enabled(features: *const Features) -> bool;
    pub fn wabt_set_tail_call_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_bulk_memory_enabled(features: *const Features) -> bool;
    pub fn wabt_set_bulk_memory_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_reference_types_enabled(features: *const Features) -> bool;
    pub fn wabt_set_reference_types_enabled(features: *mut Features, enabled: c_int);
    pub fn wabt_annotations_enabled(features: *const Features) -> bool;
    pub fn wabt_set_annotations_enabled(features: *mut Features, enabled: c_int);

    pub fn wabt_destroy_features(features: *mut Features);

    pub fn wabt_new_wast_buffer_lexer(
        filename: *const c_char,
        data: *const c_void,
        size: usize,
    ) -> *mut WastLexer;

    pub fn wabt_destroy_wast_lexer(lexer: *mut WastLexer);

    pub fn wabt_new_errors() -> *mut Errors;

    pub fn wabt_format_text_errors(errors: *mut Errors, lexer: *mut WastLexer)
        -> *mut OutputBuffer;

    pub fn wabt_format_binary_errors(errors: *mut Errors) -> *mut OutputBuffer;

    pub fn wabt_destroy_errors(errors: *mut Errors);

    pub fn wabt_parse_wat(
        lexer: *mut WastLexer,
        features: *mut Features,
        errors: *mut Errors,
    ) -> *mut WabtParseWatResult;

    pub fn wabt_parse_wast(
        lexer: *mut WastLexer,
        features: *mut Features,
        errors: *mut Errors,
    ) -> *mut WabtParseWastResult;

    pub fn wabt_parse_wat_result_get_result(result: *mut WabtParseWatResult) -> Result;

    pub fn wabt_parse_wat_result_release_module(result: *mut WabtParseWatResult)
        -> *mut WasmModule;

    pub fn wabt_destroy_parse_wat_result(result: *mut WabtParseWatResult);

    pub fn wabt_resolve_names_module(module: *mut WasmModule, errors: *mut Errors) -> Result;

    pub fn wabt_apply_names_module(module: *mut WasmModule) -> Result;

    pub fn wabt_generate_names_module(module: *mut WasmModule) -> Result;

    pub fn wabt_validate_module(
        module: *mut WasmModule,
        features: *mut Features,
        erros: *mut Errors,
    ) -> Result;

    pub fn wabt_destroy_module(module: *mut WasmModule);

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

    pub fn wabt_resolve_names_script(script: *mut Script, errors: *mut Errors) -> Result;

    pub fn wabt_validate_script(
        script: *mut Script,
        features: *mut Features,
        errors: *mut Errors,
    ) -> Result;

    pub fn wabt_write_binary_spec_script(
        script: *mut Script,
        source_filename: *const c_char,
        out_filename: *const c_char,
        log: c_int,
        canonicalize_lebs: c_int,
        relocatable: c_int,
        write_debug_name: c_int,
    ) -> *mut WabtWriteScriptResult;

    pub fn wabt_read_binary(
        data: *const u8,
        size: usize,
        read_debug_names: c_int,
        features: *mut Features,
        errors: *mut Errors,
    ) -> *mut WabtReadBinaryResult;

    pub fn wabt_parse_wast_result_get_result(result: *mut WabtParseWastResult) -> Result;

    pub fn wabt_parse_wast_result_release_module(result: *mut WabtParseWastResult) -> *mut Script;

    pub fn wabt_destroy_parse_wast_result(result: *mut WabtParseWastResult);

    pub fn wabt_read_binary_result_get_result(result: *mut WabtReadBinaryResult) -> Result;

    pub fn wabt_read_binary_result_release_module(
        result: *mut WabtReadBinaryResult,
    ) -> *mut WasmModule;

    pub fn wabt_destroy_read_binary_result(result: *mut WabtReadBinaryResult);

    pub fn wabt_write_text_module(
        module: *mut WasmModule,
        fold_exprs: c_int,
        inline_export: c_int,
    ) -> *mut WabtWriteModuleResult;

    // WabtWriteScriptResult

    pub fn wabt_write_script_result_get_result(result: *mut WabtWriteScriptResult) -> Result;

    pub fn wabt_write_script_result_release_json_output_buffer(
        result: *mut WabtWriteScriptResult,
    ) -> *mut OutputBuffer;

    pub fn wabt_write_script_result_release_log_output_buffer(
        result: *mut WabtWriteScriptResult,
    ) -> *mut OutputBuffer;

    pub fn wabt_write_script_result_get_module_count(result: *mut WabtWriteScriptResult) -> usize;

    pub fn wabt_write_script_result_get_module_filename(
        result: *mut WabtWriteScriptResult,
        index: usize,
    ) -> *const c_char;

    pub fn wabt_write_script_result_release_module_output_buffer(
        result: *mut WabtWriteScriptResult,
        index: usize,
    ) -> *mut OutputBuffer;

    pub fn wabt_destroy_write_script_result(result: *mut WabtWriteScriptResult);
}

#[test]
fn parse_wasm() {
    use std::ptr;

    let data: &[u8] = &[
        0, 97, 115, 109, // \0ASM - magic
        1, 0, 0, 0, //    01 - version
    ];

    unsafe {
        let errors = wabt_new_errors();
        let features = wabt_new_features();
        let result = wabt_read_binary(data.as_ptr(), data.len(), true as c_int, features, errors);
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

        wabt_destroy_features(features);
        wabt_destroy_errors(errors);
    }
}
