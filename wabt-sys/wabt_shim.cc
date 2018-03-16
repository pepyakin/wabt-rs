#include <cassert>
#include "src/wast-lexer.h"
#include "src/wast-parser.h"
#include "src/error-handler.h"
#include "src/resolve-names.h"
#include "src/interp.h"
#include "src/binary-reader.h"
#include "src/binary-reader-interp.h"
#include "src/string-view.h"

extern "C" {

wabt::Result::Enum wabt_resolve_names_script(
    wabt::WastLexer* lexer,
    wabt::Script* script,
    wabt::ErrorHandlerBuffer* error_handler) {
  return ResolveNamesScript(lexer, script, error_handler);
}

wabt::interp::Environment* wabt_interp_create_env() {
  return new wabt::interp::Environment();
}

void wabt_interp_destroy_env(wabt::interp::Environment* env) {
  delete env;
}

/// Version of read_binary that reads DefinedModule.
wabt::Result::Enum wabt_interp_read_binary(
    wabt::interp::Environment* env,
    const void* data,
    size_t size,
    int read_debug_names,
    wabt::ErrorHandlerBuffer* error_handler,
    wabt::interp::DefinedModule** out_module
) {
  wabt::Result result;
  wabt::ReadBinaryOptions options;
  options.read_debug_names = read_debug_names;

  result = ReadBinaryInterp(env, data, size, &options, error_handler, out_module);

  return result;
}

wabt::interp::Executor* wabt_interp_create_executor(wabt::interp::Environment* env) {
  return new wabt::interp::Executor(env);
}

void wabt_interp_destroy_executor(wabt::interp::Executor* exec) {
  delete exec;
}

enum ValueType {
  I32 = -0x01,
  I64 = -0x02,
  F32 = -0x03,
  F64 = -0x04,
};

struct TypedValue {
  ValueType type;
  union {
    uint32_t i32;
    uint64_t i64;
    uint32_t f32_bits;
    uint64_t f64_bits;
  } value;
};

static wabt::interp::TypedValue convert_typed_value_ffi_to_wabt(TypedValue ffi) {
  wabt::interp::TypedValue val;
    
  switch (ffi.type) {
    case ValueType::I32:
      val.type = wabt::Type::I32;
      val.value.i32 = ffi.value.i32;
      break;

    case ValueType::I64:
      val.type = wabt::Type::I64;
      val.value.i64 = ffi.value.i64;
      break;

    case ValueType::F32:
      val.type = wabt::Type::F32;
      val.value.f32_bits = ffi.value.f32_bits;
      break;

    case ValueType::F64:
      val.type = wabt::Type::F64;
      val.value.f64_bits = ffi.value.f64_bits;
      break;
  }

  return val;
}

static TypedValue convert_typed_value_wabt_to_ffi(wabt::interp::TypedValue wabt) {
  TypedValue val;
    
  switch (wabt.type) {
    case wabt::Type::I32:
      val.type = ValueType::I32;
      val.value.i32 = wabt.value.i32;
      break;

    case wabt::Type::I64:
      val.type = ValueType::I64;
      val.value.i64 = wabt.value.i64;
      break;

    case wabt::Type::F32:
      val.type = ValueType::F32;
      val.value.f32_bits = wabt.value.f32_bits;
      break;

    case wabt::Type::F64:
      val.type = ValueType::F64;
      val.value.f64_bits = wabt.value.f64_bits;
      break;

    default:
      // Unsupported value type.
      assert(0);
      break;
  }

  return val;
}

wabt::interp::ExecResult* wabt_interp_executor_run_export(
    wabt::interp::Executor* exec,
    wabt::interp::Module* module,
    const char* export_name_data,
    size_t export_name_len,
    TypedValue* args_data,
    size_t args_len
  ) {
  wabt::string_view export_name(export_name_data, export_name_len);
  wabt::interp::TypedValues args;

  // Push all arguments into a vector `TypedValues`.
  for (int i = 0; i < args_len; i++) {
    wabt::interp::TypedValue val = convert_typed_value_ffi_to_wabt(args_data[i]);
    args.push_back(val);
  }

  wabt::interp::ExecResult exec_result = exec->RunExportByName(module, export_name, args);
  return new wabt::interp::ExecResult(exec_result);
}

wabt::Result::Enum wabt_interp_exec_result_get_result(wabt::interp::ExecResult* result) {
  if (result->result == wabt::interp::Result::Ok) {
    return wabt::Result::Enum::Ok;
  } else {
    return wabt::Result::Enum::Error;
  }
}

size_t wabt_interp_exec_result_get_return_size(wabt::interp::ExecResult* result) {
  return result->values.size();
}

TypedValue wabt_interp_exec_result_get_return(wabt::interp::ExecResult* result, size_t index) {
  return convert_typed_value_wabt_to_ffi(result->values[index]);
}

void wabt_interp_destroy_exec_result(wabt::interp::ExecResult* result) {
  // TODO: Should I manually free the ->values vector?
  delete result;
}

}
