// Cargo.toml
/*
[package]
name = "py_rust_bridge"
version = "0.1.0"
edition = "2021"

[dependencies]
pyo3 = { version = "0.19", features = ["extension-module", "abi3-py38"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ndarray = "0.15"
numpy = "0.19"
anyhow = "1.0"
thiserror = "1.0"

[lib]
name = "py_rust_bridge"
crate-type = ["cdylib", "rlib"]
*/

// src/lib.rs
use anyhow::Result;
use base64::Engine;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;

/// Errors that can occur in the Python-Rust bridge
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Python error: {0}")]
    PythonError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Type conversion error: {0}")]
    TypeConversionError(String),

    #[error("Array shape mismatch: {0}")]
    ArrayShapeMismatch(String),
}

/// Types of results we can get from Python
#[derive(Debug, PartialEq)]
pub enum PythonResult {
    /// Bytes data (base64 encoded)
    Bytes(Vec<u8>),

    /// String data
    String(String),

    /// Array data with shape and dtype
    Array {
        dtype: String,
        shape: Vec<usize>,
        data: Vec<u8>,
    },
}

impl TryFrom<&str> for PythonResult {
    type Error = BridgeError;

    fn try_from(json_str: &str) -> Result<Self, Self::Error> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| BridgeError::SerializationError(e.to_string()))?;

        let obj = value
            .as_object()
            .ok_or_else(|| BridgeError::SerializationError("Expected JSON object".into()))?;

        let result_type = obj
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| BridgeError::SerializationError("Missing 'type' field".into()))?;

        let data = obj
            .get("data")
            .ok_or_else(|| BridgeError::SerializationError("Missing 'data' field".into()))?;

        match result_type {
            "bytes" => {
                let base64_data = data.as_str().ok_or_else(|| {
                    BridgeError::SerializationError("'data' field should be a string".into())
                })?;

                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(base64_data)
                    .map_err(|e| {
                        BridgeError::SerializationError(format!("Invalid base64: {}", e))
                    })?;

                Ok(PythonResult::Bytes(bytes))
            }
            "string" => {
                let string_data = data.as_str().ok_or_else(|| {
                    BridgeError::SerializationError("'data' field should be a string".into())
                })?;

                Ok(PythonResult::String(string_data.to_string()))
            }
            "array" => {
                let dtype = obj
                    .get("dtype")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BridgeError::SerializationError("Missing 'dtype' field".into()))?
                    .to_string();

                let shape = obj
                    .get("shape")
                    .and_then(Value::as_array)
                    .ok_or_else(|| BridgeError::SerializationError("Missing 'shape' field".into()))?
                    .iter()
                    .map(|v| {
                        v.as_u64().ok_or_else(|| {
                            BridgeError::SerializationError(
                                "Shape should be array of integers".into(),
                            )
                        })
                    })
                    .collect::<Result<Vec<u64>, _>>()?
                    .into_iter()
                    .map(|v| v as usize)
                    .collect();

                let base64_data = data.as_str().ok_or_else(|| {
                    BridgeError::SerializationError("'data' field should be a string".into())
                })?;

                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(base64_data)
                    .map_err(|e| {
                        BridgeError::SerializationError(format!("Invalid base64: {}", e))
                    })?;

                Ok(PythonResult::Array {
                    dtype,
                    shape,
                    data: bytes,
                })
            }
            _ => Err(BridgeError::SerializationError(format!(
                "Unknown result type: {}",
                result_type
            ))),
        }
    }
}

/// Core bridge for running Python code and converting results to Rust
pub struct PyRustBridge {
    py_globals: Py<PyDict>,
}

fn get_helper_code() -> &'static str {
    include_str!("helper.py")
}

impl PyRustBridge {
    /// Create a new bridge instance
    pub fn new() -> Result<Self> {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let locals = PyDict::new(py);

            // Import commonly used modules
            let builtins = PyModule::import(py, "builtins")?;
            locals.set_item("__builtins__", builtins)?;

            Ok(Self {
                py_globals: locals.into(),
            })
        })
    }

    /// Run Python code and return the result as a JSON value
    pub fn run_py_to_json(&self, code: &str) -> Result<String, BridgeError> {
        Python::with_gil(|py| {
            let globals = self.py_globals.as_ref(py);

            // No need for locals dictionary since we're using globals for both

            // load the helper code
            let helper_code = get_helper_code();

            let get_result_code = "_result = rust.get_result()";

            for code_block in vec![helper_code, code, get_result_code] {
                println!("Running code block: \n{}", code_block);
                match py.run(code_block, Some(globals), None) {
                    Ok(_) => {}
                    Err(e) => {
                        // Format Python exception nicely
                        let py_err = format_py_error(py, &e);
                        return Err(BridgeError::PythonError(py_err).into());
                    }
                }
            }

            // Check if there's a result variable defined
            match globals.get_item("_result") {
                Some(value) => {
                    if let Ok(value) = value.extract::<String>() {
                        Ok(value)
                    } else {
                        Err(BridgeError::TypeConversionError(
                            "Failed to convert Python result to string".into(),
                        ))
                    }
                }
                None => Err(BridgeError::PythonError(
                    "No 'result' variable defined in Python code".into(),
                )
                .into()),
            }
        })
    }
}

/// Format Python exception in a readable way
fn format_py_error(py: Python, error: &PyErr) -> String {
    // Get exception type and message
    let exc_type = error.get_type(py);
    let exc_name = exc_type.name().unwrap_or("Unknown");

    // Get value and traceback as strings if available
    let value = match error.value(py).str() {
        Ok(s) => s.to_string(),
        Err(_) => String::from("(no error message)"),
    };

    // Try to get traceback information
    let mut tb_info = String::new();
    if let Some(tb) = error.traceback(py) {
        if let Ok(tb_module) = PyModule::import(py, "traceback") {
            if let Ok(format_exc) = tb_module.getattr("format_tb") {
                if let Ok(tb_list) = format_exc.call1((tb,)) {
                    if let Ok(tb_list) = tb_list.downcast::<PyList>() {
                        for line in tb_list.iter() {
                            if let Ok(line_str) = line.extract::<String>() {
                                tb_info.push_str(&line_str);
                            }
                        }
                    }
                }
            }
        }
    }

    // Format the error nicely
    if tb_info.is_empty() {
        format!("{}: {}", exc_name, value)
    } else {
        format!("{}: {}\n\nTraceback:\n{}", exc_name, value, tb_info)
    }
}

// Tests for the bridge
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_print_ld_library_path() {
        // print the path after breaking it up by :
        if let Ok(val) = std::env::var("LD_LIBRARY_PATH") {
            let paths: Vec<&str> = val.split(':').collect();
            for p in paths {
                println!("Path: {}", p);
            }
        } else {
            println!("LD_LIBRARY_PATH is not set.");
        }
    }

    #[test]
    fn test_python_get_interp_info() {
        let bridge = PyRustBridge::new().unwrap();

        let code = r#"import sys; rust(sys.executable)"#;

        let result = bridge.run_py_to_json(code).unwrap();
        println!("Result Interp: {}", result);
    }

    #[test]
    fn test_python_helper_code_loaded() -> Result<()> {
        // Print the helper code to see what's being loaded
        println!("Helper code: {}", get_helper_code());

        Ok(())
    }

    #[test]
    fn test_python_bytes_serialization() -> Result<()> {
        let bridge = PyRustBridge::new()?;

        let code = r#"rust.collect_bytes(b"abcd")"#;

        let json_result = bridge.run_py_to_json(code)?;
        println!("JSON result: {}", json_result);

        // Parse the JSON result into our PythonResult enum
        let python_result = PythonResult::try_from(json_result.as_str())?;

        // Verify it's the correct type
        match python_result {
            PythonResult::Bytes(bytes) => {
                assert_eq!(bytes, b"abcd");
            }
            _ => panic!("Expected Bytes type, got {:?}", python_result),
        }

        Ok(())
    }

    #[test]
    fn test_python_error_handling() {
        let bridge = PyRustBridge::new().unwrap();

        // This code will raise a ZeroDivisionError
        let code = r#"result = 1 / 0"#;

        let result = bridge.run_py_to_json(code);
        assert!(result.is_err());

        if let Err(err) = result {
            let err_string = err.to_string();
            assert!(err_string.contains("ZeroDivisionError"));
        }
    }
}
