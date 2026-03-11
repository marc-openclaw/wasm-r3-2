//! WebAssembly AST structures
//!
//! This module defines the editable AST structures for representing WebAssembly modules.

use crate::instruction::Instruction;
use crate::types::{ExternalKind, FuncType, GlobalType, MemType, TableType, ValueType};

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

/// A WebAssembly module
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Module {
    pub custom_sections: Vec<CustomSection>,
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub funcs: Vec<u32>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemType>,
    pub globals: Vec<Global>,
    pub exports: Vec<Export>,
    pub start: Option<u32>,
    pub elements: Vec<ElementSegment>,
    pub code: Vec<FunctionBody>,
    pub data: Vec<DataSegment>,
    pub data_count: Option<u32>,
}

impl Module {
    /// Create a new empty module
    pub fn new() -> Self { Self::default() }
    
    /// Add a type to the module and return its index
    pub fn add_type(&mut self, func_type: FuncType) -> u32 {
        let idx = self.types.len() as u32;
        self.types.push(func_type);
        idx
    }
    
    /// Add a function to the module and return its index
    pub fn add_function(&mut self, type_idx: u32) -> u32 {
        let idx = self.funcs.len() as u32;
        self.funcs.push(type_idx);
        idx
    }
    
    /// Get a reference to a function's body by function index
    pub fn get_function_body(&self, func_idx: u32) -> Option<&FunctionBody> {
        let import_count = self.imports.iter().filter(|i| i.kind == ExternalKind::Func).count() as u32;
        let code_idx = func_idx.checked_sub(import_count)?;
        self.code.get(code_idx as usize)
    }
    
    /// Get a mutable reference to a function's body by function index
    pub fn get_function_body_mut(&mut self, func_idx: u32) -> Option<&mut FunctionBody> {
        let import_count = self.imports.iter().filter(|i| i.kind == ExternalKind::Func).count() as u32;
        let code_idx = func_idx.checked_sub(import_count)?;
        self.code.get_mut(code_idx as usize)
    }
    
    /// Add an import to the module
    pub fn add_import(&mut self, import: Import) { self.imports.push(import); }
    
    /// Add an export to the module
    pub fn add_export(&mut self, name: String, kind: ExternalKind, idx: u32) {
        self.exports.push(Export { name, kind, idx });
    }
    
    /// Set the start function index
    pub fn set_start(&mut self, func_idx: u32) { self.start = Some(func_idx); }
    
    /// Add a memory to the module
    pub fn add_memory(&mut self, mem_type: MemType) { self.memories.push(mem_type); }
    
    /// Add a table to the module
    pub fn add_table(&mut self, table_type: TableType) { self.tables.push(table_type); }
    
    /// Add a global to the module
    pub fn add_global(&mut self, global: Global) { self.globals.push(global); }
    
    /// Add a data segment to the module
    pub fn add_data(&mut self, data: DataSegment) { self.data.push(data); }
}

/// A custom section in the WASM module
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct CustomSection { 
    /// Name of the custom section
    pub name: String, 
    /// Raw bytes of the custom section
    pub data: Vec<u8> 
}

/// An import entry
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Import { 
    /// Module name
    pub module: String, 
    /// Field name within the module
    pub name: String, 
    /// Kind of import (function, table, memory, or global)
    pub kind: ExternalKind, 
    /// Type index or other index
    pub idx: u32 
}

/// An export entry
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Export { 
    /// Name of the export
    pub name: String, 
    /// Kind of export
    pub kind: ExternalKind, 
    /// Index of the exported item
    pub idx: u32 
}

/// A global variable
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Global { 
    /// Type of the global (including mutability)
    pub ty: GlobalType, 
    /// Initialization expression
    pub init: Vec<Instruction> 
}

/// An element segment (function table initialization)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct ElementSegment { 
    /// Mode (passive, active, or declared)
    pub mode: ElementMode, 
    /// Element type (usually funcref)
    pub elem_type: ValueType, 
    /// Function indices in the segment
    pub init: Vec<u32> 
}

/// Element segment mode
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub enum ElementMode { 
    /// Passive element segment
    Passive, 
    /// Active element segment with table index and offset expression
    Active { table_idx: u32, offset: Vec<Instruction> }, 
    /// Declared element segment
    Declared 
}

/// A data segment
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct DataSegment { 
    /// Mode (passive or active)
    pub mode: DataMode, 
    /// Raw bytes of the data
    pub data: Vec<u8> 
}

/// Data segment mode
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub enum DataMode { 
    /// Passive data segment
    Passive, 
    /// Active data segment with memory index and offset expression
    Active { mem_idx: u32, offset: Vec<Instruction> } 
}

/// A function body (code section entry)
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct FunctionBody { 
    /// Local variable declarations
    pub locals: Vec<Local>, 
    /// Instructions in the function body
    pub instructions: Vec<Instruction> 
}

impl FunctionBody {
    /// Create a new empty function body
    pub fn new() -> Self { Self::default() }
    
    /// Add local variables to the function
    pub fn add_local(&mut self, count: u32, ty: ValueType) { 
        self.locals.push(Local { count, ty }); 
    }
    
    /// Add an instruction to the end of the function
    pub fn add_instruction(&mut self, instr: Instruction) { 
        self.instructions.push(instr); 
    }
    
    /// Insert an instruction at a specific position
    pub fn insert_instruction(&mut self, pos: usize, instr: Instruction) {
        if pos > self.instructions.len() { 
            self.instructions.push(instr); 
        } else { 
            self.instructions.insert(pos, instr); 
        }
    }
    
    /// Remove an instruction at a specific position
    pub fn remove_instruction(&mut self, pos: usize) -> Option<Instruction> {
        if pos < self.instructions.len() { 
            Some(self.instructions.remove(pos)) 
        } else { 
            None 
        }
    }
    
    /// Replace an instruction at a specific position
    pub fn replace_instruction(&mut self, pos: usize, instr: Instruction) {
        if pos < self.instructions.len() { 
            self.instructions[pos] = instr; 
        }
    }
    
    /// Get a mutable reference to the instructions
    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> { 
        &mut self.instructions 
    }
}

/// A local variable declaration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Local { 
    /// Number of locals of this type
    pub count: u32, 
    /// Type of the local(s)
    pub ty: ValueType 
}
