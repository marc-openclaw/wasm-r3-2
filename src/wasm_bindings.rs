//! WebAssembly bindings for wasm_parser
//!
//! This module provides JavaScript bindings when the crate is compiled to WASM.
//! Enable with the `wasm` feature flag.

use wasm_bindgen::prelude::*;
use js_sys::{Uint8Array, Error as JsError};
use crate::{Module, parse, encode_module};
use serde::{Serialize, Deserialize};

/// WASM module wrapper for JavaScript interop
#[wasm_bindgen]
pub struct WasmModule {
    inner: Module,
}

#[wasm_bindgen]
impl WasmModule {
    /// Create a new empty module
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmModule { inner: Module::new() }
    }

    /// Parse WASM binary from a JavaScript Uint8Array
    #[wasm_bindgen(js_name = parse)]
    pub fn parse_wasm(data: Uint8Array) -> Result<WasmModule, JsValue> {
        let bytes = data.to_vec();
        match parse(&bytes) {
            Ok(module) => Ok(WasmModule { inner: module }),
            Err(e) => Err(JsError::new(&format!("Parse error: {}", e)).into()),
        }
    }

    /// Encode module to bytes as a Uint8Array
    #[wasm_bindgen(js_name = encode)]
    pub fn encode(&self) -> Result<Uint8Array, JsValue> {
        match encode_module(&self.inner) {
            Ok(bytes) => {
                let array = Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(&bytes);
                Ok(array)
            }
            Err(e) => Err(JsError::new(&format!("Encode error: {}", e)).into()),
        }
    }

    /// Get function count
    #[wasm_bindgen(js_name = functionCount, getter)]
    pub fn function_count(&self) -> usize {
        self.inner.funcs.len()
    }

    /// Get type count
    #[wasm_bindgen(js_name = typeCount, getter)]
    pub fn type_count(&self) -> usize {
        self.inner.types.len()
    }

    /// Get export count
    #[wasm_bindgen(js_name = exportCount, getter)]
    pub fn export_count(&self) -> usize {
        self.inner.exports.len()
    }

    /// Get import count
    #[wasm_bindgen(js_name = importCount, getter)]
    pub fn import_count(&self) -> usize {
        self.inner.imports.len()
    }

    /// Get memory count
    #[wasm_bindgen(js_name = memoryCount, getter)]
    pub fn memory_count(&self) -> usize {
        self.inner.memories.len()
    }

    /// Get table count
    #[wasm_bindgen(js_name = tableCount, getter)]
    pub fn table_count(&self) -> usize {
        self.inner.tables.len()
    }

    /// Get global count
    #[wasm_bindgen(js_name = globalCount, getter)]
    pub fn global_count(&self) -> usize {
        self.inner.globals.len()
    }

    /// Get data segment count
    #[wasm_bindgen(js_name = dataCount, getter)]
    pub fn data_count(&self) -> usize {
        self.inner.data.len()
    }

    /// Get element segment count
    #[wasm_bindgen(js_name = elementCount, getter)]
    pub fn element_count(&self) -> usize {
        self.inner.elements.len()
    }

    /// Get module as JSON representation
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsError::new(&format!("JSON error: {}", e)).into())
    }
}

impl Default for WasmModule {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize WASM module (call once from JS)
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
