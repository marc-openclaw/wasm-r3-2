//! WebAssembly AST structures
//!
//! This module defines the editable AST structures for representing WebAssembly modules.

use crate::instruction::Instruction;
use crate::types::{ExternalKind, FuncType, GlobalType, MemType, TableType, ValueType};

/// A WebAssembly module
#[derive(Debug, Clone, Default)]
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
    pub fn new() -> Self { Self::default() }
    
    pub fn add_type(&mut self, func_type: FuncType) -> u32 {
        let idx = self.types.len() as u32;
        self.types.push(func_type);
        idx
    }
    
    pub fn add_function(&mut self, type_idx: u32) -> u32 {
        let idx = self.funcs.len() as u32;
        self.funcs.push(type_idx);
        idx
    }
    
    pub fn get_function_body(&self, func_idx: u32) -> Option<&FunctionBody> {
        let import_count = self.imports.iter().filter(|i| i.kind == ExternalKind::Func).count() as u32;
        let code_idx = func_idx.checked_sub(import_count)?;
        self.code.get(code_idx as usize)
    }
    
    pub fn get_function_body_mut(&mut self, func_idx: u32) -> Option<&mut FunctionBody> {
        let import_count = self.imports.iter().filter(|i| i.kind == ExternalKind::Func).count() as u32;
        let code_idx = func_idx.checked_sub(import_count)?;
        self.code.get_mut(code_idx as usize)
    }
    
    pub fn add_import(&mut self, import: Import) { self.imports.push(import); }
    pub fn add_export(&mut self, name: String, kind: ExternalKind, idx: u32) {
        self.exports.push(Export { name, kind, idx });
    }
    pub fn set_start(&mut self, func_idx: u32) { self.start = Some(func_idx); }
    pub fn add_memory(&mut self, mem_type: MemType) { self.memories.push(mem_type); }
    pub fn add_table(&mut self, table_type: TableType) { self.tables.push(table_type); }
    pub fn add_global(&mut self, global: Global) { self.globals.push(global); }
    pub fn add_data(&mut self, data: DataSegment) { self.data.push(data); }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomSection { pub name: String, pub data: Vec<u8> }

#[derive(Debug, Clone, PartialEq)]
pub struct Import { pub module: String, pub name: String, pub kind: ExternalKind, pub idx: u32 }

#[derive(Debug, Clone, PartialEq)]
pub struct Export { pub name: String, pub kind: ExternalKind, pub idx: u32 }

#[derive(Debug, Clone, PartialEq)]
pub struct Global { pub ty: GlobalType, pub init: Vec<Instruction> }

#[derive(Debug, Clone, PartialEq)]
pub struct ElementSegment { pub mode: ElementMode, pub elem_type: ValueType, pub init: Vec<u32> }

#[derive(Debug, Clone, PartialEq)]
pub enum ElementMode { Passive, Active { table_idx: u32, offset: Vec<Instruction> }, Declared }

#[derive(Debug, Clone, PartialEq)]
pub struct DataSegment { pub mode: DataMode, pub data: Vec<u8> }

#[derive(Debug, Clone, PartialEq)]
pub enum DataMode { Passive, Active { mem_idx: u32, offset: Vec<Instruction> } }

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FunctionBody { pub locals: Vec<Local>, pub instructions: Vec<Instruction> }

impl FunctionBody {
    pub fn new() -> Self { Self::default() }
    pub fn add_local(&mut self, count: u32, ty: ValueType) { self.locals.push(Local { count, ty }); }
    pub fn add_instruction(&mut self, instr: Instruction) { self.instructions.push(instr); }
    pub fn insert_instruction(&mut self, pos: usize, instr: Instruction) {
        if pos > self.instructions.len() { self.instructions.push(instr); }
        else { self.instructions.insert(pos, instr); }
    }
    pub fn remove_instruction(&mut self, pos: usize) -> Option<Instruction> {
        if pos < self.instructions.len() { Some(self.instructions.remove(pos)) } else { None }
    }
    pub fn replace_instruction(&mut self, pos: usize, instr: Instruction) {
        if pos < self.instructions.len() { self.instructions[pos] = instr; }
    }
    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> { &mut self.instructions }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local { pub count: u32, pub ty: ValueType }
