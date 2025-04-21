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
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use serde_json::json;
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

/// Core bridge for running Python code and converting results to Rust
pub struct PyRustBridge {
    py_globals: Py<PyDict>,
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

            // Import json for serialization
            let json = PyModule::import(py, "json")?;
            locals.set_item("json", json)?;

            Ok(Self {
                py_globals: locals.into(),
            })
        })
    }

    /// Run Python code and return the result as a JSON value
    pub fn run_py_to_json(&self, code: &str) -> Result<String, BridgeError> {
        Python::with_gil(|py| {
            let globals = self.py_globals.as_ref(py);

            // Create a locals dictionary for the execution
            let locals = PyDict::new(py);

            // Try to execute the code
            match py.run(code, Some(globals), Some(locals)) {
                Ok(_) => {}
                Err(e) => {
                    // Format Python exception nicely
                    let py_err = format_py_error(py, &e);
                    return Err(BridgeError::PythonError(py_err).into());
                }
            }

            // Check if there's a result variable defined
            match locals.get_item("result") {
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
    fn test_json_serialization() -> Result<()> {
        let bridge = PyRustBridge::new()?;

        let code = r#"
import json

# Create a sample JSON structure
data = {
    "name": "Test",
    "values": [1, 2, 3, 4, 5],
    "nested": {
        "a": True,
        "b": None,
        "c": 3.14
    }
}

# Set as result
result = data
"#;

        let json_result = bridge.run_py_to_json(code)?;

        // Expected structure
        let expected = json!({
            "name": "Test",
            "values": [1, 2, 3, 4, 5],
            "nested": {
                "a": true,
                "b": null,
                "c": 3.14
            }
        });

        assert_eq!(json_result, expected);
        Ok(())
    }

    #[test]
    fn test_error_handling() {
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
