//! High-level WASM parser API
//!
//! This module provides a user-friendly API for parsing and manipulating WASM modules.

use crate::ast::{Module, FunctionBody};
use crate::decode;
use crate::encode;
use crate::error::Result;
use crate::instruction::Instruction;

/// Parse WASM binary from bytes
pub fn parse(bytes: &[u8]) -> Result<Module> {
    decode::parse_bytes(bytes)
}

/// Encode a WASM module to bytes
pub fn encode_module(module: &Module) -> Result<Vec<u8>> {
    encode::encode_module(module)
}

/// Parse and modify WASM binary
///
/// Example:
/// ```
/// use wasm_parser::{parse, encode_module, Module};
///
/// // Parse WASM bytes
/// let bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]; // minimal WASM module
/// let mut module = parse(&bytes).unwrap();
/// // Modify the module...
/// let output = encode_module(&module).unwrap();
/// ```
pub struct WasmParser;

impl WasmParser {
    /// Create a new empty module
    pub fn new_module() -> Module {
        Module::new()
    }

    /// Parse WASM from bytes
    pub fn parse(bytes: &[u8]) -> Result<Module> {
        parse(bytes)
    }

    /// Encode module to bytes
    pub fn encode_wasm(module: &Module) -> Result<Vec<u8>> {
        crate::encode::encode_module(module)
    }

    /// Read WASM from a file
    pub fn read_file(path: &std::path::Path) -> Result<Module> {
        let bytes = std::fs::read(path)?;
        Self::parse(&bytes)
    }

    /// Write WASM to a file
    pub fn write_file(path: &std::path::Path, module: &Module) -> Result<()> {
        let bytes = crate::encode::encode_module(module)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

/// Helper trait for working with WASM modules
pub trait ModuleExt {
    /// Get the number of functions
    fn function_count(&self) -> usize;

    /// Check if a function exists
    fn has_function(&self, idx: u32) -> bool;

    /// Modify a function's instructions
    fn modify_function<F>(&mut self, idx: u32, f: F) -> Result<()>
    where
        F: FnOnce(&mut FunctionBody);

    /// Insert an instruction in a function
    fn insert_instruction(&mut self, func_idx: u32, pos: usize, instr: Instruction) -> Result<()>;

    /// Remove an instruction from a function
    fn remove_instruction(&mut self, func_idx: u32, pos: usize) -> Result<Option<Instruction>>;

    /// Replace an instruction in a function
    fn replace_instruction(&mut self, func_idx: u32, pos: usize, instr: Instruction) -> Result<()>;
}

impl ModuleExt for Module {
    fn function_count(&self) -> usize {
        self.funcs.len() as usize
    }

    fn has_function(&self, idx: u32) -> bool {
        (idx as usize) < self.function_count()
    }

    fn modify_function<F>(&mut self, idx: u32, f: F) -> Result<()>
    where
        F: FnOnce(&mut FunctionBody),
    {
        let body = self.get_function_body_mut(idx)
            .ok_or_else(|| crate::error::WasmError::InvalidFunctionIndex(idx))?;
        f(body);
        Ok(())
    }

    fn insert_instruction(&mut self, func_idx: u32, pos: usize, instr: Instruction) -> Result<()> {
        self.modify_function(func_idx, |body| {
            body.insert_instruction(pos, instr);
        })
    }

    fn remove_instruction(&mut self, func_idx: u32, pos: usize) -> Result<Option<Instruction>> {
        let body = self.get_function_body_mut(func_idx)
            .ok_or_else(|| crate::error::WasmError::InvalidFunctionIndex(func_idx))?;
        Ok(body.remove_instruction(pos))
    }

    fn replace_instruction(&mut self, func_idx: u32, pos: usize, instr: Instruction) -> Result<()> {
        let body = self.get_function_body_mut(func_idx)
            .ok_or_else(|| crate::error::WasmError::InvalidFunctionIndex(func_idx))?;
        body.replace_instruction(pos, instr);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Module;
    use crate::types::{FuncType, ValueType};
    use crate::encode_module;

    #[test]
    fn test_parser_api() {
        let module = WasmParser::new_module();
        assert_eq!(module.function_count(), 0);
    }

    #[test]
    fn test_parse_minimal_module() {
        let bytes = vec![
            0x00, 0x61, 0x73, 0x6d,  // magic
            0x01, 0x00, 0x00, 0x00,  // version
        ];
        let module = WasmParser::parse(&bytes).unwrap();
        assert_eq!(module.function_count(), 0);
    }

    #[test]
    fn test_roundtrip() {
        let mut module = Module::new();

        // Add a type
        let type_idx = module.add_type(FuncType::new(
            vec![ValueType::I32],
            vec![ValueType::I32],
        ));

        // Add a function
        let func_idx = module.add_function(type_idx);

        // Add a function body
        let mut body = FunctionBody::new();
        body.add_local(1, ValueType::I32);
        body.add_instruction(Instruction::LocalGet { local_idx: 0 });
        body.add_instruction(Instruction::LocalGet { local_idx: 0 });
        body.add_instruction(Instruction::I32Add);
        module.code.push(body);

        // Encode
        let encoded = crate::encode_module(&module).unwrap();

        // Parse back
        let module2 = parse(&encoded).unwrap();
        assert_eq!(module2.types.len(), 1);
        assert_eq!(module2.funcs.len(), 1);
        assert_eq!(module2.code.len(), 1);
    }
}
