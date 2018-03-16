#![allow(missing_docs)]

use std::ptr;
use std::os::raw::c_int;
use wabt_sys as ffi;
use super::ErrorHandler;

#[derive(Debug)]
pub struct Trap;

pub struct Environment {
    raw_env: *mut ffi::Environment,
}

impl Environment {
    pub fn new() -> Environment {
        let raw_env = unsafe { ffi::wabt_interp_create_env() };
        Environment { raw_env }
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe { ffi::wabt_interp_destroy_env(self.raw_env) }
    }
}

pub struct Module {
    raw_module: *mut ffi::DefinedModule,
}

impl Module {
    pub fn read_binary(env: &Environment, wasm: &[u8]) -> Result<Module, String> {
        let error_handler = ErrorHandler::new_binary();
        let mut raw_module: *mut ffi::DefinedModule = ptr::null_mut();
        unsafe {
            let result = ffi::wabt_interp_read_binary(
                env.raw_env,
                wasm.as_ptr(),
                wasm.len(),
                0 as c_int,
                error_handler.raw_buffer,
                &mut raw_module as *mut *mut ffi::DefinedModule,
            );
            if result == ffi::Result::Error {
                return Err(error_handler.to_string());
            }
        }
        Ok(Module { raw_module })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    I32(u32),
    I64(u64),
    F32(u32),
    F64(u64),
}

impl Value {
    fn to_typed_value(&self) -> ffi::TypedValue {
        match *self {
            Value::I32(v) => ffi::TypedValue {
                type_: ffi::VALUETYPE_I32,
                value: ffi::UntypedValue { i32: v },
            },
            Value::I64(v) => ffi::TypedValue {
                type_: ffi::VALUETYPE_I64,
                value: ffi::UntypedValue { i64: v },
            },
            Value::F32(v) => ffi::TypedValue {
                type_: ffi::VALUETYPE_F32,
                value: ffi::UntypedValue { f32_bits: v },
            },
            Value::F64(v) => ffi::TypedValue {
                type_: ffi::VALUETYPE_F64,
                value: ffi::UntypedValue { f64_bits: v },
            },
        }
    }

    unsafe fn from_typed_value(typed_value: ffi::TypedValue) -> Value {
        match typed_value.type_ {
            ffi::VALUETYPE_I32 => {
                Value::I32(typed_value.value.i32)
            }
            ffi::VALUETYPE_I64 => {
                Value::I64(typed_value.value.i64)
            }
            ffi::VALUETYPE_F32 => {
                Value::F32(typed_value.value.f32_bits)
            }
            ffi::VALUETYPE_F64 => {
                Value::F64(typed_value.value.f64_bits)
            }
            other_ty => panic!("Unsupported type: {}", other_ty)
        }
    }
}

pub struct Executor {
    raw_exec: *mut ffi::Executor,
}

impl Executor {
    pub fn new(env: &Environment) -> Executor {
        let raw_exec = unsafe { ffi::wabt_interp_create_executor(env.raw_env) };
        Executor { raw_exec }
    }

    pub fn execute(&self, module: &Module, export_name: &str, args: &[Value]) -> Result<Option<Value>, Trap> {
        let typed_value_args: Vec<ffi::TypedValue> =
            args.iter().map(|v| v.to_typed_value()).collect();
        let raw_result = unsafe {
            ffi::wabt_interp_executor_run_export(
                self.raw_exec,
                module.raw_module,
                export_name.as_ptr(),
                export_name.len(),
                typed_value_args.as_ptr(),
                typed_value_args.len(),
            )
        };

        let result = ExecResult::new(raw_result);
        result.to_return_value().map_err(|_| Trap)
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        unsafe { ffi::wabt_interp_destroy_executor(self.raw_exec) }
    }
}

pub struct ExecResult {
    raw_result: *mut ffi::ExecResult,
}

impl ExecResult {
    fn new(raw_result: *mut ffi::ExecResult) -> ExecResult {
        ExecResult {
            raw_result,
        }
    }

    fn is_ok(&self) -> bool {
        unsafe {
            ffi::wabt_interp_exec_result_get_result(self.raw_result) == ffi::Result::Ok
        }
    }

    fn return_size(&self) -> usize {
        unsafe {
            ffi::wabt_interp_exec_result_get_return_size(self.raw_result)
        }
    }

    fn return_at(&self, index: usize) -> Value {
        unsafe {
            let typed_value = ffi::wabt_interp_exec_result_get_return(self.raw_result, index);
            Value::from_typed_value(typed_value)
        }
    }

    fn to_return_value(&self) -> Result<Option<Value>, ()> {
        if self.is_ok() {
            let return_size = self.return_size();
            let value = match return_size {
                0 => None,
                1 => Some(self.return_at(0)),
                _ => panic!(
                    "Unsupported number of return values. Was multi-value propolsal implemented?"
                ),
            };
            Ok(value)
        } else {
            Err(())
        }
    }
}

impl Drop for ExecResult {
    fn drop(&mut self) {
        unsafe {
            ffi::wabt_interp_destroy_exec_result(self.raw_result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wat2wasm;

    #[test]
    fn it_works() {
        let env = Environment::new();
        let exec = Executor::new(&env);
        let wasm = wat2wasm(
            r#"
            (module
                (func (export "test") (param i32) (result i32)
                    get_local 0
                    i32.const 1
                    i32.add
                )
            )"#).unwrap();
        let m = Module::read_binary(&env, &wasm).unwrap();

        let result = exec.execute(&m, "test", &[Value::I32(41)]).unwrap();
        assert_eq!(result, Some(Value::I32(42)));
    }
}
