//! WebAssembly binary encoder

use crate::ast::*;
use crate::error::{Result, WasmError};
use crate::instruction::Instruction;
use crate::leb128;
use crate::types::ValueType;
use crate::{WASM_MAGIC, WASM_VERSION};

/// Binary encoder for WASM
pub struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn write_slice(&mut self, slice: &[u8]) {
        self.bytes.extend_from_slice(slice);
    }

    fn write_u32_le(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn write_u64_le(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn write_f32_le(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn write_f64_le(&mut self, value: f64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn write_u32_leb128(&mut self, value: u32) {
        self.bytes.extend_from_slice(&leb128::encode_u32(value));
    }

    fn write_i32_leb128(&mut self, value: i32) {
        self.bytes.extend_from_slice(&leb128::encode_i32(value));
    }

    fn write_i64_leb128(&mut self, value: i64) {
        self.bytes.extend_from_slice(&leb128::encode_i64(value));
    }

    fn write_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        self.write_u32_leb128(bytes.len() as u32);
        self.write_slice(bytes);
    }

    fn begin_section(&mut self, section_id: u8) -> SectionBuilder<'_> {
        SectionBuilder::new(self, section_id)
    }

    pub fn encode(&mut self, module: &Module) -> Result<()> {
        self.write_slice(WASM_MAGIC);
        self.write_slice(WASM_VERSION);

        // Type section
        if !module.types.is_empty() {
            self.encode_type_section(&module.types)?;
        }

        // Import section
        if !module.imports.is_empty() {
            self.encode_import_section(&module.imports)?;
        }

        // Function section
        if !module.funcs.is_empty() {
            self.encode_func_section(&module.funcs)?;
        }

        // Table section
        if !module.tables.is_empty() {
            self.encode_table_section(&module.tables)?;
        }

        // Memory section
        if !module.memories.is_empty() {
            self.encode_memory_section(&module.memories)?;
        }

        // Global section
        if !module.globals.is_empty() {
            self.encode_global_section(&module.globals)?;
        }

        // Export section
        if !module.exports.is_empty() {
            self.encode_export_section(&module.exports)?;
        }

        // Start section
        if let Some(start_idx) = module.start {
            self.encode_start_section(start_idx)?;
        }

        // Element section
        if !module.elements.is_empty() {
            self.encode_element_section(&module.elements)?;
        }

        // Data count section
        if let Some(data_count) = module.data_count {
            self.encode_datacount_section(data_count)?;
        }

        // Code section
        if !module.code.is_empty() {
            self.encode_code_section(&module.code)?;
        }

        // Data section
        if !module.data.is_empty() {
            self.encode_data_section(&module.data)?;
        }

        // Custom sections
        for custom in &module.custom_sections {
            self.encode_custom_section(custom)?;
        }

        Ok(())
    }

    fn encode_type_section(&mut self, types: &[crate::types::FuncType]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(types.len() as u32);

        for func_type in types {
            content.write_u8(0x60); // Func type marker
            content.write_u32_leb128(func_type.params.len() as u32);
            for param in &func_type.params {
                content.write_u8(param.to_byte());
            }
            content.write_u32_leb128(func_type.results.len() as u32);
            for result in &func_type.results {
                content.write_u8(result.to_byte());
            }
        }

        self.write_u8(1); // Type section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_import_section(&mut self, imports: &[Import]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(imports.len() as u32);

        for import in imports {
            content.write_name(&import.module);
            content.write_name(&import.name);
            content.write_u8(import.kind as u8);
            content.write_u32_leb128(import.idx);
        }

        self.write_u8(2); // Import section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_func_section(&mut self, funcs: &[u32]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(funcs.len() as u32);

        for &type_idx in funcs {
            content.write_u32_leb128(type_idx);
        }

        self.write_u8(3); // Function section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_table_section(&mut self, tables: &[crate::types::TableType]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(tables.len() as u32);

        for table in tables {
            content.write_u8(table.elem_type.to_byte());
            encode_limits(&mut content, &table.limits);
        }

        self.write_u8(4); // Table section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_memory_section(&mut self, memories: &[crate::types::MemType]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(memories.len() as u32);

        for mem in memories {
            encode_limits(&mut content, &mem.limits);
        }

        self.write_u8(5); // Memory section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_global_section(&mut self, globals: &[Global]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(globals.len() as u32);

        for global in globals {
            content.write_u8(global.ty.value_type.to_byte());
            content.write_u8(if global.ty.mutable { 0x01 } else { 0x00 });
            encode_instructions(&mut content, &global.init)?;
        }

        self.write_u8(6); // Global section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_export_section(&mut self, exports: &[Export]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(exports.len() as u32);

        for export in exports {
            content.write_name(&export.name);
            content.write_u8(export.kind as u8);
            content.write_u32_leb128(export.idx);
        }

        self.write_u8(7); // Export section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_start_section(&mut self, start_idx: u32) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(start_idx);

        self.write_u8(8); // Start section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_element_section(&mut self, elements: &[ElementSegment]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(elements.len() as u32);

        for elem in elements {
            match &elem.mode {
                ElementMode::Passive => {
                    content.write_u32_leb128(1); // flag = 1
                }
                ElementMode::Active { table_idx, offset } => {
                    if *table_idx == 0 {
                        content.write_u32_leb128(0); // flag = 0
                    } else {
                        content.write_u32_leb128(2); // flag = 2
                        content.write_u32_leb128(*table_idx);
                    }
                    encode_instructions(&mut content, offset)?;
                }
                ElementMode::Declared => {
                    content.write_u32_leb128(3); // flag = 3
                }
            }

            content.write_u32_leb128(elem.init.len() as u32);
            for &func_idx in &elem.init {
                content.write_u32_leb128(func_idx);
            }
        }

        self.write_u8(9); // Element section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_code_section(&mut self, bodies: &[FunctionBody]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(bodies.len() as u32);

        for body in bodies {
            let mut func_content = Encoder::new();

            // Encode locals
            func_content.write_u32_leb128(body.locals.len() as u32);
            for local in &body.locals {
                func_content.write_u32_leb128(local.count);
                func_content.write_u8(local.ty.to_byte());
            }

            // Encode instructions
            encode_instructions(&mut func_content, &body.instructions)?;

            // Wrap in function
            let func_bytes = func_content.into_bytes();
            content.write_u32_leb128(func_bytes.len() as u32);
            content.write_slice(&func_bytes);
        }

        self.write_u8(10); // Code section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_data_section(&mut self, data: &[DataSegment]) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(data.len() as u32);

        for segment in data {
            match &segment.mode {
                DataMode::Passive => {
                    content.write_u32_leb128(1); // flag = 1
                }
                DataMode::Active { mem_idx, offset } => {
                    if *mem_idx == 0 {
                        content.write_u32_leb128(0); // flag = 0
                    } else {
                        content.write_u32_leb128(2); // flag = 2
                        content.write_u32_leb128(*mem_idx);
                    }
                    encode_instructions(&mut content, offset)?;
                }
            }

            content.write_u32_leb128(segment.data.len() as u32);
            content.write_slice(&segment.data);
        }

        self.write_u8(11); // Data section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_datacount_section(&mut self, data_count: u32) -> Result<()> {
        let mut content = Encoder::new();
        content.write_u32_leb128(data_count);

        self.write_u8(12); // DataCount section ID
        let content_bytes = content.into_bytes();
        self.write_u32_leb128(content_bytes.len() as u32);
        self.write_slice(&content_bytes);
        Ok(())
    }

    fn encode_custom_section(&mut self, custom: &CustomSection) -> Result<()> {
        self.write_u8(0); // Custom section ID
        let name_bytes = custom.name.as_bytes();
        let total_len = leb128::encode_u32(name_bytes.len() as u32).len() + name_bytes.len() + custom.data.len();
        self.write_u32_leb128(total_len as u32);
        self.write_name(&custom.name);
        self.write_slice(&custom.data);
        Ok(())
    }
}

fn encode_limits(encoder: &mut Encoder, limits: &crate::types::Limits) {
    let mut flags: u8 = 0;
    if limits.max.is_some() {
        flags |= 0x01;
    }
    if limits.shared {
        flags |= 0x02;
    }
    if limits.memory64 {
        flags |= 0x04;
    }
    encoder.write_u8(flags);
    encoder.write_u32_leb128(limits.min);
    if let Some(max) = limits.max {
        encoder.write_u32_leb128(max);
    }
}

fn encode_instructions(encoder: &mut Encoder, instructions: &[Instruction]) -> Result<()> {
    for instr in instructions {
        encode_instruction(encoder, instr)?;
    }
    encoder.write_u8(0x0b); // End
    Ok(())
}

fn encode_instruction(encoder: &mut Encoder, instr: &Instruction) -> Result<()> {
    match instr {
        Instruction::Unreachable => encoder.write_u8(0x00),
        Instruction::Nop => encoder.write_u8(0x01),
        Instruction::Block { block_type, body } => {
            encoder.write_u8(0x02);
            encoder.write_i32_leb128(block_type.to_i64() as i32);
            encode_instructions(encoder, body)?;
        }
        Instruction::Loop { block_type, body } => {
            encoder.write_u8(0x03);
            encoder.write_i32_leb128(block_type.to_i64() as i32);
            encode_instructions(encoder, body)?;
        }
        Instruction::If { block_type, then_branch, else_branch } => {
            encoder.write_u8(0x04);
            encoder.write_i32_leb128(block_type.to_i64() as i32);
            encode_instructions(encoder, then_branch)?;
            if !else_branch.is_empty() {
                encoder.write_u8(0x05); // Else
                encode_instructions(encoder, else_branch)?;
            }
            encoder.write_u8(0x0b); // End
            return Ok(());
        }
        Instruction::End => encoder.write_u8(0x0b),
        Instruction::Br { label_idx } => {
            encoder.write_u8(0x0c);
            encoder.write_u32_leb128(*label_idx);
        }
        Instruction::BrIf { label_idx } => {
            encoder.write_u8(0x0d);
            encoder.write_u32_leb128(*label_idx);
        }
        Instruction::BrTable { labels, default_label } => {
            encoder.write_u8(0x0e);
            encoder.write_u32_leb128(labels.len() as u32);
            for label in labels {
                encoder.write_u32_leb128(*label);
            }
            encoder.write_u32_leb128(*default_label);
        }
        Instruction::Return => encoder.write_u8(0x0f),
        Instruction::Call { function_idx } => {
            encoder.write_u8(0x10);
            encoder.write_u32_leb128(*function_idx);
        }
        Instruction::CallIndirect { type_idx, table_idx } => {
            encoder.write_u8(0x11);
            encoder.write_u32_leb128(*type_idx);
            encoder.write_u32_leb128(*table_idx);
        }
        Instruction::Drop => encoder.write_u8(0x1a),
        Instruction::Select => encoder.write_u8(0x1b),
        Instruction::LocalGet { local_idx } => {
            encoder.write_u8(0x20);
            encoder.write_u32_leb128(*local_idx);
        }
        Instruction::LocalSet { local_idx } => {
            encoder.write_u8(0x21);
            encoder.write_u32_leb128(*local_idx);
        }
        Instruction::LocalTee { local_idx } => {
            encoder.write_u8(0x22);
            encoder.write_u32_leb128(*local_idx);
        }
        Instruction::GlobalGet { global_idx } => {
            encoder.write_u8(0x23);
            encoder.write_u32_leb128(*global_idx);
        }
        Instruction::GlobalSet { global_idx } => {
            encoder.write_u8(0x24);
            encoder.write_u32_leb128(*global_idx);
        }
        Instruction::I32Load { mem_arg } => {
            encoder.write_u8(0x28);
            encode_mem_arg(encoder, mem_arg);
        }
        Instruction::I64Load { mem_arg } => {
            encoder.write_u8(0x29);
            encode_mem_arg(encoder, mem_arg);
        }
        Instruction::F32Load { mem_arg } => {
            encoder.write_u8(0x2a);
            encode_mem_arg(encoder, mem_arg);
        }
        Instruction::F64Load { mem_arg } => {
            encoder.write_u8(0x2b);
            encode_mem_arg(encoder, mem_arg);
        }
        Instruction::I32Const { value } => {
            encoder.write_u8(0x41);
            encoder.write_i32_leb128(*value);
        }
        Instruction::I64Const { value } => {
            encoder.write_u8(0x42);
            encoder.write_i64_leb128(*value);
        }
        Instruction::F32Const { value } => {
            encoder.write_u8(0x43);
            encoder.write_f32_le(*value);
        }
        Instruction::F64Const { value } => {
            encoder.write_u8(0x44);
            encoder.write_f64_le(*value);
        }
        Instruction::I32Eqz => encoder.write_u8(0x45),
        Instruction::I32Eq => encoder.write_u8(0x46),
        Instruction::I32Ne => encoder.write_u8(0x47),
        Instruction::I32Add => encoder.write_u8(0x6a),
        Instruction::I32Sub => encoder.write_u8(0x6b),
        Instruction::I32Mul => encoder.write_u8(0x6c),
        Instruction::I32DivS => encoder.write_u8(0x6d),
        Instruction::I32DivU => encoder.write_u8(0x6e),
        Instruction::I64Add => encoder.write_u8(0x7c),
        Instruction::I64Sub => encoder.write_u8(0x7d),
        Instruction::I64Mul => encoder.write_u8(0x7e),
        Instruction::I32WrapI64 => encoder.write_u8(0xa7),
        Instruction::I64ExtendI32S => encoder.write_u8(0xac),
        Instruction::I64ExtendI32U => encoder.write_u8(0xad),
        Instruction::RefNull { ty } => {
            encoder.write_u8(0xd0);
            encoder.write_u8(ty.to_byte());
        }
        Instruction::RefIsNull => encoder.write_u8(0xd1),
        Instruction::RefFunc { func_idx } => {
            encoder.write_u8(0xd2);
            encoder.write_u32_leb128(*func_idx);
        }
        _ => return Err(WasmError::Custom(format!("Unimplemented instruction: {:?}", instr))),
    }
    Ok(())
}

fn encode_mem_arg(encoder: &mut Encoder, mem_arg: &crate::instruction::MemArg) {
    encoder.write_u32_leb128(mem_arg.align);
    encoder.write_u32_leb128(mem_arg.offset);
}

/// Encode a Module into WASM binary bytes
pub fn encode_module(module: &Module) -> Result<Vec<u8>> {
    let mut encoder = Encoder::new();
    encoder.encode(module)?;
    Ok(encoder.into_bytes())
}

/// SectionBuilder for delayed section encoding
struct SectionBuilder<'a> {
    encoder: &'a mut Encoder,
    section_id: u8,
    content: Encoder,
}

impl<'a> SectionBuilder<'a> {
    fn new(encoder: &'a mut Encoder, section_id: u8) -> Self {
        Self {
            encoder,
            section_id,
            content: Encoder::new(),
        }
    }

    fn finish(self) {
        let content_bytes = self.content.into_bytes();
        self.encoder.write_u8(self.section_id);
        self.encoder.write_u32_leb128(content_bytes.len() as u32);
        self.encoder.write_slice(&content_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FuncType, ValueType};

    #[test]
    fn test_encode_empty_module() {
        let module = Module::new();
        let bytes = encode_module(&module).unwrap();
        assert_eq!(bytes, vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_encode_type_section() {
        let mut module = Module::new();
        module.add_type(FuncType::new(vec![ValueType::I32, ValueType::I32], vec![ValueType::I32]));
        let bytes = encode_module(&module).unwrap();
        assert!(&bytes.starts_with(&[0x00, 0x61, 0x73, 0x6d]));
    }
}
