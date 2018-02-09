//! Module for parsing spec testsuite scripts.

use std::fs::File;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use std::io;

use serde_json;
use tempdir;

use super::{Error as WabtError, Script};

mod json;

/// Error that can happen when parsing spec or executing spec tests.
#[derive(Debug)]
pub enum Error<E> {
    /// IO error happened during parsing.
    IoError(io::Error),
    /// WABT reported an error while converting wast to json.
    WabtError(WabtError),
    /// Other error represented by String.
    Other(String),
    /// User provided error.
    User(E),
}

impl<E> Error<E> {
    /// Returns `Some` if this error contains user-specific error, and `None` otherwise.
    pub fn as_user_error(&self) -> Option<&E> {
        match *self {
            Error::User(ref e) => Some(e),
            _ => None,
        }
    }
}

impl<E> From<WabtError> for Error<E> {
    fn from(e: WabtError) -> Error<E> {
        Error::WabtError(e)
    }
}

impl<E> From<io::Error> for Error<E> {
    fn from(e: io::Error) -> Error<E> {
        Error::IoError(e)
    }
}

/// Wasm value
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    /// 32-bit signed or unsigned integer.
    I32(i32),
    /// 64-bit signed or unsigned integer.
    I64(i64),
    /// 32-bit floating point number.
    F32(f32),
    /// 64-bit floating point number.
    F64(f64),
}

impl Value {
    fn decode_f32(val: u32) -> Self {
        Value::F32(f32_from_bits(val))
    }

    fn decode_f64(val: u64) -> Self {
        Value::F64(f64_from_bits(val))
    }
}

// Convert u32 to f32 safely, masking out sNAN
fn f32_from_bits(mut v: u32) -> f32 {
    const EXP_MASK: u32 = 0x7F800000;
    const QNAN_MASK: u32 = 0x00400000;
    const FRACT_MASK: u32 = 0x007FFFFF;

    if v & EXP_MASK == EXP_MASK && v & FRACT_MASK != 0 {
        // If we have a NaN value, we
        // convert signaling NaN values to quiet NaN
        // by setting the the highest bit of the fraction
        // TODO: remove when https://github.com/BurntSushi/byteorder/issues/71 closed.
        // or `f32::from_bits` stabilized.
        v |= QNAN_MASK;
    }

    unsafe { ::std::mem::transmute(v) }
}

// Convert u64 to f64 safely, masking out sNAN
fn f64_from_bits(mut v: u64) -> f64 {
    const EXP_MASK: u64 = 0x7FF0000000000000;
    const QNAN_MASK: u64 = 0x0001000000000000;
    const FRACT_MASK: u64 = 0x000FFFFFFFFFFFFF;

    if v & EXP_MASK == EXP_MASK && v & FRACT_MASK != 0 {
        // If we have a NaN value, we
        // convert signaling NaN values to quiet NaN
        // by setting the the highest bit of the fraction
        // TODO: remove when https://github.com/BurntSushi/byteorder/issues/71 closed.
        // or `f64::from_bits` stabilized.
        v |= QNAN_MASK;
    }

    unsafe { ::std::mem::transmute(v) }
}

/// Description of action that should be performed on a wasm module.
pub enum Action {
    /// Invoke a specified function.
    Invoke { 
        /// Name of the module. If `None`, last defined module should be 
        /// used.
        module: Option<String>,
        /// Field name on which action should be performed.
        field: String,
        /// Arguments that should be passed in the invoked function.
        args: Vec<Value> 
    },
    /// Read the specified global variable.
    Get {
        /// Name of the module. If `None`, last defined module should be 
        /// used.
        module: Option<String>,
        /// Field name on which action should be performed.
        field: String,
    }
}

/// Implement this trait to be able to run the spec scripts.
#[allow(unused)]
pub trait Visitor<E> {
    /// Called upon beginning of the spec script.
    fn begin_spec(&mut self, source_filename: &str) -> Result<(), E> {
        Ok(())
    }

    /// Define a module with specified optional name.
    fn module(&mut self, line: u64, wasm: &[u8], name: Option<String>) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified action should yield expected results.
    fn assert_return(&mut self, line: u64, action: &Action, expected: &[Value]) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified action should yield canonical NaN.
    fn assert_return_canonical_nan(&mut self, line: u64, action: &Action) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified action should yield arithmetic NaN.
    fn assert_return_arithmetic_nan(&mut self, line: u64, action: &Action) -> Result<(), E> {
        Ok(())
    }

    /// Assert resource exhaustion.
    fn assert_exhaustion(&mut self, line: u64, action: &Action) -> Result<(), E> {
        Ok(())
    }

    /// Assert that performing specified action will result in a trap.
    fn assert_trap(&mut self, line: u64, action: &Action, text: &str) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified module is invalid.
    fn assert_invalid(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified module is malformed.
    fn assert_malformed(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified module is unlinkable.
    fn assert_unlinkable(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), E> {
        Ok(())
    }

    /// Assert that specified module is uninstantiable.
    fn assert_uninstantiable(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), E> {
        Ok(())
    }

    /// Register specified module (or last defined module) with specified name.
    fn register(&mut self, line: u64, name: Option<&str>, as_name: &str) -> Result<(), E> {
        Ok(())
    }
    
    /// Perform specified action.
    fn perform_action(&mut self, line: u64, action: &Action) -> Result<(), E> {
        Ok(())
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ::std::io::Error> {
    use std::io::Read;
    let mut buf = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn runtime_value(test_val: &json::RuntimeValue) -> Value {
    match test_val.value_type.as_ref() {
        "i32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            Value::I32(unsigned as i32)
        }
        "i64" => {
            let unsigned: u64 = test_val.value.parse().expect("Literal parse error");
            Value::I64(unsigned as i64)
        }
        "f32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            Value::decode_f32(unsigned)
        }
        "f64" => {
            let unsigned: u64 = test_val.value.parse().expect("Literal parse error");
            Value::decode_f64(unsigned)
        }
        _ => panic!("Unknwon runtime value type"),
    }
}

fn runtime_values(test_vals: &[json::RuntimeValue]) -> Vec<Value> {
    test_vals.iter().map(runtime_value).collect::<Vec<Value>>()
}

// Convert json string to correct rust UTF8 string.
// The reason is that, for example, rust character "\u{FEEF}" (3-byte UTF8 BOM) is represented as "\u00ef\u00bb\u00bf" in spec json.
// It is incorrect. Correct BOM representation in json is "\uFEFF" => we need to do a double utf8-parse here.
// This conversion is incorrect in general case (casting char to u8)!!!
fn jstring_to_rstring(jstring: &str) -> String {
    let jstring_chars: Vec<u8> = jstring.chars().map(|c| c as u8).collect();
    let rstring = String::from_utf8(jstring_chars).unwrap();
    rstring
}

fn parse_action(test_action: &json::Action) -> Action {
    match *test_action {
        json::Action::Invoke {
            ref module,
            ref field,
            ref args,
        } => Action::Invoke {
            module: module.to_owned(),
            field: jstring_to_rstring(field),
            args: runtime_values(args),
        },
        json::Action::Get {
            ref module,
            ref field,
        } => Action::Get {
            module: module.to_owned(),
            field: jstring_to_rstring(field),
        },
    }
}

fn wast2json<E>(path: &Path, test_filename: &str, json_spec_path: &Path) -> Result<(), Error<E>> {
    let source = read_file(path)?;
    let script = Script::parse(test_filename, source)?;
    script.validate()?;
    script.write_binaries(test_filename, &json_spec_path)?;
    Ok(())
}

/// Run spec script at the specified path.
///
/// Path should exists and point to a file with `.wast` extension.
pub fn run_spec<P: AsRef<Path>, E: Debug, V: Visitor<E>>(
    path: P,
    visitor: &mut V,
) -> Result<(), Error<E>> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(Error::Other(format!(
            "Path {} doesn't exists",
            path.display()
        )));
    }

    let extension = path.extension();
    if extension != Some(OsStr::new("wast")) {
        return Err(Error::Other(format!(
            "Provided {} should have .wast extension",
            path.display()
        )));
    }

    // Get test name: filename without an extension ('.wast').
    let test_filename = path.file_name().and_then(|f| f.to_str()).ok_or_else(|| {
        Error::Other(format!(
            "Provided {} should have .wast extension",
            path.display()
        ))
    })?;
    let test_name = &test_filename[0..test_filename.len() - 5];

    // Create temporary directory for collecting all artifacts of wast2json.
    let temp_dir_name = format!("spec-testsuite-{}", test_name);
    let temp_dir = tempdir::TempDir::new(&temp_dir_name)?;
    let outdir = temp_dir.path().clone();

    // Construct path for output file of wast2json. Wasm binaries will be named similarly.
    let mut json_spec_path = PathBuf::from(outdir.clone());
    json_spec_path.push(&format!("{}.json", test_name));

    // Convert wasm script into json spec and binaries. The output artifacts
    // will be written relative to json_spec_path.
    wast2json(path, test_filename, &json_spec_path)?;

    let mut f = File::open(json_spec_path).expect("Failed to load json file");
    let spec: json::Spec =
        serde_json::from_reader(&mut f).expect("Failed to deserialize JSON file");
    visit_spec(spec, outdir, visitor)?;

    Ok(())
}

fn visit_spec<E: Debug, V: Visitor<E>>(
    spec: json::Spec,
    root: &Path,
    v: &mut V,
) -> Result<(), Error<E>> {
    let json::Spec {
        source_filename,
        commands,
    } = spec;
    v.begin_spec(&source_filename).map_err(Error::User)?;

    for command in commands {
        match command {
            json::Command::Module {
                line,
                name,
                filename,
            } => {
                let mut module_path = PathBuf::from(root.clone());
                module_path.push(filename);
                let wasm = read_file(module_path)?;
                v.module(line, &wasm, name).map_err(Error::User)?;
            }
            json::Command::AssertReturn {
                line,
                action,
                expected,
            } => {
                let expected = runtime_values(&expected);
                let action = parse_action(&action);
                v.assert_return(line, &action, &expected)
                    .map_err(Error::User)?;
            }
            json::Command::AssertReturnCanonicalNan { line, action } => {
                let action = parse_action(&action);
                v.assert_return_canonical_nan(line, &action)
                    .map_err(Error::User)?;
            }
            json::Command::AssertReturnArithmeticNan { line, action } => {
                let action = parse_action(&action);
                v.assert_return_arithmetic_nan(line, &action)
                    .map_err(Error::User)?;
            }
            json::Command::AssertExhaustion { line, action } => {
                let action = parse_action(&action);
                v.assert_exhaustion(line, &action).map_err(Error::User)?;
            }
            json::Command::AssertTrap { line, action, text } => {
                let action = parse_action(&action);
                v.assert_trap(line, &action, &text).map_err(Error::User)?;
            }
            json::Command::AssertInvalid {
                line,
                filename,
                text,
            } => {
                let mut module_path = PathBuf::from(root.clone());
                module_path.push(filename);
                let wasm = read_file(module_path)?;
                v.assert_invalid(line, &wasm, &text).map_err(Error::User)?;
            }
            json::Command::AssertMalformed {
                line,
                filename,
                text,
            } => {
                let mut module_path = PathBuf::from(root.clone());
                module_path.push(filename);
                let wasm = read_file(module_path)?;
                v.assert_malformed(line, &wasm, &text).map_err(Error::User)?;
            }
            json::Command::AssertUnlinkable {
                line,
                filename,
                text,
            } => {
                let mut module_path = PathBuf::from(root.clone());
                module_path.push(filename);
                let wasm = read_file(module_path)?;
                v.assert_unlinkable(line, &wasm, &text)
                    .map_err(Error::User)?;
            }
            json::Command::AssertUninstantiable {
                line,
                filename,
                text,
            } => {
                let mut module_path = PathBuf::from(root.clone());
                module_path.push(filename);
                let wasm = read_file(module_path)?;
                v.assert_uninstantiable(line, &wasm, &text)
                    .map_err(Error::User)?;
            }
            json::Command::Register {
                line,
                name,
                as_name,
            } => {
                v.register(line, name.as_ref().map(|n| n.as_ref()), &as_name)
                    .map_err(Error::User)?;
            }
            json::Command::Action { line, action } => {
                let action = parse_action(&action);
                v.perform_action(line, &action).map_err(Error::User)?;
            }
        }
    }

    Ok(())
}
