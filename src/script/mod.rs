//! Module for parsing [WebAssembly script format] \(a.k.a. wast).
//!
//! These scripts might be useful to integrate the official spec [testsuite] into implementations
//! of the wasm execution engines (such as [wasmi]) and for developing fine-grained tests of
//! runtimes or/and if it isn't desired to use full fledged compilers.
//!
//! # Example
//!
//! ```rust
//! use wabt::script::{ScriptParser, Command, CommandKind, Action, Value};
//! # use wabt::script::Error;
//!
//! # fn try_main() -> Result<(), Error> {
//! let wast = r#"
//! ;; Define anonymous module with function export named `sub`.
//! (module
//!   (func (export "sub") (param $x i32) (param $y i32) (result i32)
//!     ;; return x - y;
//!     (i32.sub
//!       (get_local $x) (get_local $y)
//!     )
//!   )
//! )
//!
//! ;; Assert that invoking export `sub` with parameters (8, 3)
//! ;; should return 5.
//! (assert_return
//!   (invoke "sub"
//!     (i32.const 8) (i32.const 3)
//!   )
//!   (i32.const 5)
//! )
//! "#;
//!
//! let mut parser = ScriptParser::<f32, f64>::from_str(wast)?;
//! while let Some(Command { kind, .. }) = parser.next()? {
//!     match kind {
//!         CommandKind::Module { module, name } => {
//!             // The module is declared as annonymous.
//!             assert_eq!(name, None);
//!
//!             // Convert the module into the binary representation and check the magic number.
//!             let module_binary = module.into_vec();
//!             assert_eq!(&module_binary[0..4], &[0, 97, 115, 109]);
//!         }
//!         CommandKind::AssertReturn { action, expected } => {
//!             assert_eq!(action, Action::Invoke {
//!                 module: None,
//!                 field: "sub".to_string(),
//!                 args: vec![
//!                     Value::I32(8),
//!                     Value::I32(3)
//!                 ],
//!             });
//!             assert_eq!(expected, vec![Value::I32(5)]);
//!         },
//!         _ => panic!("there are no other commands apart from that defined above"),
//!     }
//! }
//! # Ok(())
//! # }
//! #
//! # fn main() {
//! #     try_main().unwrap();
//! # }
//! ```
//! [WebAssembly script format]: https://github.com/WebAssembly/spec/blob/a25083ac7076b05e3f304ec9e093ef1b1ee09422/interpreter/README.md#scripts
//! [testsuite]: https://github.com/WebAssembly/testsuite
//! [wasmi]: https://github.com/pepyakin/wasmi

use std::collections::HashMap;
use std::error;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::str;
use std::vec;

use serde_json;

use super::{Error as WabtError, Features, Script, WabtBuf, WabtWriteScriptResult};

mod json;

/// Error that can happen when parsing spec.
#[derive(Debug)]
pub enum Error {
    /// IO error happened during parsing or preparing to parse.
    IoError(io::Error),
    /// WABT reported an error while converting wast to json.
    WabtError(WabtError),
    /// Other error represented by String.
    Other(String),
    /// Not a different kind of an error but just a wrapper for a error
    /// which we have a line number information.
    WithLineInfo {
        /// Line number of the script on which just error happen.
        line: u64,
        /// Box with actual error.
        error: Box<Error>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref io_err) => write!(f, "IO error: {}", io_err),
            Error::WabtError(ref wabt_err) => write!(f, "wabt error: {:?}", wabt_err),
            Error::Other(ref message) => write!(f, "{}", message),
            Error::WithLineInfo { line, ref error } => write!(f, "At line {}: {}", line, error),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref e) => e.description(),
            Error::WabtError(_) => "wabt error",
            Error::Other(ref msg) => &msg,
            Error::WithLineInfo { ref error, .. } => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref io_err) => Some(io_err),
            Error::WabtError(ref wabt_err) => Some(wabt_err),
            Error::Other(_) => None,
            Error::WithLineInfo { ref error, .. } => Some(error),
        }
    }
}

impl From<WabtError> for Error {
    fn from(e: WabtError) -> Error {
        Error::WabtError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

/// Bitwise conversion from T
pub trait FromBits<T> {
    /// Convert `other` to `Self`, preserving bitwise representation
    fn from_bits(other: T) -> Self;
}

impl<T> FromBits<T> for T {
    fn from_bits(other: T) -> Self {
        other
    }
}

impl FromBits<u32> for f32 {
    fn from_bits(other: u32) -> Self {
        Self::from_bits(other)
    }
}

impl FromBits<u64> for f64 {
    fn from_bits(other: u64) -> Self {
        Self::from_bits(other)
    }
}

/// Wasm value
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Value<F32 = f32, F64 = f64> {
    /// 32-bit signed or unsigned integer.
    I32(i32),
    /// 64-bit signed or unsigned integer.
    I64(i64),
    /// 32-bit floating point number.
    F32(F32),
    /// 64-bit floating point number.
    F64(F64),
}

impl<F32: FromBits<u32>, F64: FromBits<u64>> Value<F32, F64> {
    fn decode_f32(val: u32) -> Self {
        Value::F32(F32::from_bits(val))
    }

    fn decode_f64(val: u64) -> Self {
        Value::F64(F64::from_bits(val))
    }
}

/// Description of action that should be performed on a wasm module.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Action<F32 = f32, F64 = f64> {
    /// Invoke a specified function.
    Invoke {
        /// Name of the module. If `None`, last defined module should be
        /// used.
        module: Option<String>,
        /// Field name on which action should be performed.
        field: String,
        /// Arguments that should be passed to the invoked function.
        args: Vec<Value<F32, F64>>,
    },
    /// Read the specified global variable.
    Get {
        /// Name of the module. If `None`, last defined module should be
        /// used.
        module: Option<String>,
        /// Field name on which action should be performed.
        field: String,
    },
}

fn parse_value<F32: FromBits<u32>, F64: FromBits<u64>>(
    test_val: &json::RuntimeValue,
) -> Result<Value<F32, F64>, Error> {
    fn parse_val<P: str::FromStr>(str_val: &str, str_ty: &str) -> Result<P, Error> {
        str_val
            .parse()
            .map_err(|_| Error::Other(format!("can't parse '{}' as '{}'", str_val, str_ty)))
    }
    let value = match test_val.value_type.as_ref() {
        "i32" => {
            let unsigned: u32 = parse_val(&test_val.value, &test_val.value_type)?;
            Value::I32(unsigned as i32)
        }
        "i64" => {
            let unsigned: u64 = parse_val(&test_val.value, &test_val.value_type)?;
            Value::I64(unsigned as i64)
        }
        "f32" => {
            let unsigned: u32 = parse_val(&test_val.value, &test_val.value_type)?;
            Value::decode_f32(unsigned)
        }
        "f64" => {
            let unsigned: u64 = parse_val(&test_val.value, &test_val.value_type)?;
            Value::decode_f64(unsigned)
        }
        other_ty => {
            return Err(Error::Other(format!("Unknown type '{}'", other_ty)));
        }
    };
    Ok(value)
}

fn parse_value_list<F32: FromBits<u32>, F64: FromBits<u64>>(
    test_vals: &[json::RuntimeValue],
) -> Result<Vec<Value<F32, F64>>, Error> {
    test_vals.iter().map(parse_value).collect()
}

// Convert json string to correct rust UTF8 string.
// The reason is that, for example, rust character "\u{FEEF}" (3-byte UTF8 BOM) is represented as "\u00ef\u00bb\u00bf" in spec json.
// It is incorrect. Correct BOM representation in json is "\uFEFF" => we need to do a double utf8-parse here.
// This conversion is incorrect in general case (casting char to u8)!!!
fn jstring_to_rstring(jstring: &str) -> String {
    let jstring_chars: Vec<u8> = jstring.chars().map(|c| c as u8).collect();
    String::from_utf8(jstring_chars).unwrap()
}

fn parse_action<F32: FromBits<u32>, F64: FromBits<u64>>(
    test_action: &json::Action,
) -> Result<Action<F32, F64>, Error> {
    let action = match *test_action {
        json::Action::Invoke {
            ref module,
            ref field,
            ref args,
        } => Action::Invoke {
            module: module.to_owned(),
            field: jstring_to_rstring(field),
            args: parse_value_list(args)?,
        },
        json::Action::Get {
            ref module,
            ref field,
        } => Action::Get {
            module: module.to_owned(),
            field: jstring_to_rstring(field),
        },
    };
    Ok(action)
}

fn wast2json(
    source: &[u8],
    test_filename: &str,
    features: Features,
) -> Result<WabtWriteScriptResult, Error> {
    let script = Script::parse(test_filename, source, features.clone())?;
    script.resolve_names()?;
    script.validate()?;
    let result = script.write_binaries(test_filename)?;
    Ok(result)
}

/// This is a handle to get the binary representation of the module.
#[derive(Clone, Debug)]
pub struct ModuleBinary {
    module: Vec<u8>,
}

impl Eq for ModuleBinary {}
impl PartialEq for ModuleBinary {
    fn eq(&self, rhs: &Self) -> bool {
        self.module == rhs.module
    }
}

impl ModuleBinary {
    fn from_vec(module: Vec<u8>) -> ModuleBinary {
        ModuleBinary { module }
    }

    /// Convert this object into wasm module binary representation.
    pub fn into_vec(self) -> Vec<u8> {
        self.module
    }
}

/// Script's command.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandKind<F32 = f32, F64 = f64> {
    /// Define, validate and instantiate a module.
    Module {
        /// Wasm module binary to define, validate and instantiate.
        module: ModuleBinary,

        /// If the `name` is specified, the module should be registered under this name.
        name: Option<String>,
    },
    /// Assert that specified action should yield specified results.
    AssertReturn {
        /// Action to perform.
        action: Action<F32, F64>,
        /// Values that expected to be yielded by the action.
        expected: Vec<Value<F32, F64>>,
    },
    /// Assert that specified action should yield NaN in canonical form.
    AssertReturnCanonicalNan {
        /// Action to perform.
        action: Action<F32, F64>,
    },
    /// Assert that specified action should yield NaN with 1 in MSB of fraction field.
    AssertReturnArithmeticNan {
        /// Action to perform.
        action: Action<F32, F64>,
    },
    /// Assert that performing specified action must yield in a trap.
    AssertTrap {
        /// Action to perform.
        action: Action<F32, F64>,
        /// Expected failure should be with this message.
        message: String,
    },
    /// Assert that specified module is invalid.
    AssertInvalid {
        /// Module that should be invalid.
        module: ModuleBinary,
        /// Expected failure should be with this message.
        message: String,
    },
    /// Assert that specified module cannot be decoded.
    AssertMalformed {
        /// Module that should be malformed.
        module: ModuleBinary,
        /// Expected failure should be with this message.
        message: String,
    },
    /// Assert that specified module is uninstantiable.
    AssertUninstantiable {
        /// Module that should be uninstantiable.
        module: ModuleBinary,
        /// Expected failure should be with this message.
        message: String,
    },
    /// Assert that specified action should yield in resource exhaustion.
    AssertExhaustion {
        /// Action to perform.
        action: Action<F32, F64>,
    },
    /// Assert that specified module fails to link.
    AssertUnlinkable {
        /// Module that should be unlinkable.
        module: ModuleBinary,
        /// Expected failure should be with this message.
        message: String,
    },
    /// Register a module under specified name (`as_name`).
    Register {
        /// Name of the module, which should be registered under different name.
        ///
        /// If `None` then the last defined [module][`Module`] should be used.
        ///
        /// [`Module`]: #variant.Module
        name: Option<String>,
        /// New name of the specified module.
        as_name: String,
    },
    /// Perform the specified [action].
    ///
    /// [action]: enum.Action.html
    PerformAction(Action<F32, F64>),
}

/// Command in the script.
///
/// It consists of line number and [`CommandKind`].
///
/// [`CommandKind`]: enum.CommandKind.html
#[derive(Clone, Debug, PartialEq)]
pub struct Command<F32 = f32, F64 = f64> {
    /// Line number the command is defined on.
    pub line: u64,

    /// Kind of the command.
    pub kind: CommandKind<F32, F64>,
}

/// Parser which allows to parse WebAssembly script text format.
pub struct ScriptParser<F32 = f32, F64 = f64> {
    cmd_iter: vec::IntoIter<json::Command>,
    modules: HashMap<CString, WabtBuf>,
    _phantom: ::std::marker::PhantomData<(F32, F64)>,
}

impl<F32: FromBits<u32>, F64: FromBits<u64>> ScriptParser<F32, F64> {
    /// Create `ScriptParser` from the script in specified file.
    ///
    /// The `source` should contain valid wast.
    ///
    /// The `test_filename` must have a `.wast` extension.
    pub fn from_source_and_name(source: &[u8], test_filename: &str) -> Result<Self, Error> {
        ScriptParser::from_source_and_name_with_features(source, test_filename, Features::new())
    }

    /// Create `ScriptParser` from the script in specified file, parsing with
    /// the given features.
    ///
    /// The `source` should contain valid wast.
    ///
    /// The `test_filename` must have a `.wast` extension.
    pub fn from_source_and_name_with_features(
        source: &[u8],
        test_filename: &str,
        features: Features,
    ) -> Result<Self, Error> {
        if !test_filename.ends_with(".wast") {
            return Err(Error::Other(format!(
                "Provided {} should have .wast extension",
                test_filename
            )));
        }

        // Convert wasm script into json spec and binaries. The output artifacts
        // will be placed in result.

        let results = wast2json(source, test_filename, features.clone())?;
        let results = results.take_all().expect("Failed to release");

        let json_str = results.json_output_buffer.as_ref();

        let spec: json::Spec =
            serde_json::from_slice(json_str).expect("Failed to deserialize JSON buffer");

        let json::Spec { commands, .. } = spec;

        Ok(ScriptParser {
            cmd_iter: commands.into_iter(),
            modules: results.module_output_buffers,
            _phantom: Default::default(),
        })
    }

    /// Create `ScriptParser` from the script source.
    pub fn from_str(source: &str) -> Result<Self, Error> {
        ScriptParser::from_source_and_name(source.as_bytes(), "test.wast")
    }

    /// Returns the next [`Command`] from the script.
    ///
    /// Returns `Err` if an error occurred while parsing the script,
    /// or returns `None` if the parser reached end of script.
    ///
    /// [`Command`]: struct.Command.html
    pub fn next(&mut self) -> Result<Option<Command<F32, F64>>, Error> {
        let command = match self.cmd_iter.next() {
            Some(cmd) => cmd,
            None => return Ok(None),
        };

        let get_module = |filename: String, s: &Self| {
            let filename = CString::new(filename).unwrap();
            s.modules
                .get(&filename)
                .map(|module| ModuleBinary::from_vec(module.as_ref().to_owned()))
                .expect("Module referenced in JSON does not exist.")
        };

        let (line, kind) = match command {
            json::Command::Module {
                line,
                name,
                filename,
            } => (
                line,
                CommandKind::Module {
                    module: get_module(filename, self),
                    name,
                },
            ),
            json::Command::AssertReturn {
                line,
                action,
                expected,
            } => (
                line,
                CommandKind::AssertReturn {
                    action: parse_action(&action)?,
                    expected: parse_value_list(&expected)?,
                },
            ),
            json::Command::AssertReturnCanonicalNan { line, action } => (
                line,
                CommandKind::AssertReturnCanonicalNan {
                    action: parse_action(&action)?,
                },
            ),
            json::Command::AssertReturnArithmeticNan { line, action } => (
                line,
                CommandKind::AssertReturnArithmeticNan {
                    action: parse_action(&action)?,
                },
            ),
            json::Command::AssertExhaustion { line, action } => (
                line,
                CommandKind::AssertExhaustion {
                    action: parse_action(&action)?,
                },
            ),
            json::Command::AssertTrap { line, action, text } => (
                line,
                CommandKind::AssertTrap {
                    action: parse_action(&action)?,
                    message: text,
                },
            ),
            json::Command::AssertInvalid {
                line,
                filename,
                text,
            } => (
                line,
                CommandKind::AssertInvalid {
                    module: get_module(filename, self),
                    message: text,
                },
            ),
            json::Command::AssertMalformed {
                line,
                filename,
                text,
            } => (
                line,
                CommandKind::AssertMalformed {
                    module: get_module(filename, self),
                    message: text,
                },
            ),
            json::Command::AssertUnlinkable {
                line,
                filename,
                text,
            } => (
                line,
                CommandKind::AssertUnlinkable {
                    module: get_module(filename, self),
                    message: text,
                },
            ),
            json::Command::AssertUninstantiable {
                line,
                filename,
                text,
            } => (
                line,
                CommandKind::AssertUninstantiable {
                    module: get_module(filename, self),
                    message: text,
                },
            ),
            json::Command::Register {
                line,
                name,
                as_name,
            } => (line, CommandKind::Register { name, as_name }),
            json::Command::Action { line, action } => {
                (line, CommandKind::PerformAction(parse_action(&action)?))
            }
        };

        Ok(Some(Command { line, kind }))
    }
}
