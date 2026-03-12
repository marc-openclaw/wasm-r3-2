//! WebAssembly binary decoder

use crate::ast::*;
use crate::error::{Result, WasmError};
use crate::instruction::{Instruction, MemArg};
use crate::leb128;
use crate::types::*;
use crate::{SectionId, WASM_MAGIC, WASM_VERSION};

/// Binary decoder for WASM
pub struct Decoder<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn consume(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        let result = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(result)
    }

    fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        let byte = self.bytes[self.pos];
        self.pos += 1;
        Ok(byte)
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let bytes = self.consume(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64_le(&mut self) -> Result<u64> {
        let bytes = self.consume(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let bytes = self.consume(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_f64_le(&mut self) -> Result<f64> {
        let bytes = self.consume(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_u32_leb128(&mut self) -> Result<u32> {
        let (value, size) = leb128::decode_u32(&self.bytes[self.pos..])?;
        self.pos += size;
        Ok(value)
    }

    fn read_i32_leb128(&mut self) -> Result<i32> {
        let (value, size) = leb128::decode_i32(&self.bytes[self.pos..])?;
        self.pos += size;
        Ok(value)
    }

    fn read_i64_leb128(&mut self) -> Result<i64> {
        let (value, size) = leb128::decode_i64(&self.bytes[self.pos..])?;
        self.pos += size;
        Ok(value)
    }

    fn read_name(&mut self) -> Result<String> {
        let len = self.read_u32_leb128()? as usize;
        let bytes = self.consume(len)?;
        // Allow invalid UTF-8 by using lossy conversion
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn decode(&mut self) -> Result<Module> {
        self.check_magic()?;
        self.check_version()?;

        let mut module = Module::new();

        while self.pos < self.bytes.len() {
            match self.read_section()? {
                (SectionId::Custom, data) => {
                    let mut custom = Decoder::new(data);
                    let name = custom.read_name()?;
                    let remaining = custom.bytes.len() - custom.pos;
                    let data_bytes = custom.consume(remaining)?.to_vec();
                    module.custom_sections.push(CustomSection { name, data: data_bytes });
                }
                (SectionId::Type, data) => module.types = decode_type_section(data)?,
                (SectionId::Import, data) => module.imports = decode_import_section(data)?,
                (SectionId::Function, data) => module.funcs = decode_func_section(data)?,
                (SectionId::Table, data) => module.tables = decode_table_section(data)?,
                (SectionId::Memory, data) => module.memories = decode_memory_section(data)?,
                (SectionId::Global, data) => module.globals = decode_global_section(data)?,
                (SectionId::Export, data) => module.exports = decode_export_section(data)?,
                (SectionId::Start, data) => module.start = Some(decode_start_section(data)?),
                (SectionId::Element, data) => module.elements = decode_element_section(data)?,
                (SectionId::Code, data) => module.code = decode_code_section(data)?,
                (SectionId::Data, data) => module.data = decode_data_section(data)?,
                (SectionId::DataCount, data) => module.data_count = Some(decode_datacount_section(data)?),
            }
        }

        Ok(module)
    }

    fn check_magic(&mut self) -> Result<()> {
        let magic = self.consume(4)?;
        if magic != WASM_MAGIC {
            return Err(WasmError::InvalidMagic(magic.to_vec()));
        }
        Ok(())
    }

    fn check_version(&mut self) -> Result<()> {
        let version = self.consume(4)?;
        if version != WASM_VERSION {
            return Err(WasmError::InvalidVersion(version.to_vec()));
        }
        Ok(())
    }

    fn read_section(&mut self) -> Result<(SectionId, &[u8])> {
        let section_id = self.read_u8()?;
        let section_id = SectionId::try_from(section_id)?;
        let section_len = self.read_u32_leb128()? as usize;
        let section_data = self.consume(section_len)?;
        Ok((section_id, section_data))
    }
}

fn decode_type_section(data: &[u8]) -> Result<Vec<FuncType>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut types = Vec::with_capacity(count);

    for _ in 0..count {
        let form = decoder.read_u8()?;
        if form != 0x60 {
            return Err(WasmError::Custom(format!("Invalid function type form: {}", form)));
        }

        let param_count = decoder.read_u32_leb128()? as usize;
        let mut params = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            params.push(ValueType::from_byte(decoder.read_u8()?)?);
        }

        let result_count = decoder.read_u32_leb128()? as usize;
        let mut results = Vec::with_capacity(result_count);
        for _ in 0..result_count {
            results.push(ValueType::from_byte(decoder.read_u8()?)?);
        }

        types.push(FuncType::new(params, results));
    }

    Ok(types)
}

fn decode_import_section(data: &[u8]) -> Result<Vec<Import>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut imports = Vec::with_capacity(count);

    for _ in 0..count {
        let module = decoder.read_name()?;
        let name = decoder.read_name()?;
        let kind = ExternalKind::from_byte(decoder.read_u8()?)?;
        let idx = decoder.read_u32_leb128()?;
        imports.push(Import { module, name, kind, idx });
    }

    Ok(imports)
}

fn decode_func_section(data: &[u8]) -> Result<Vec<u32>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut funcs = Vec::with_capacity(count);

    for _ in 0..count {
        funcs.push(decoder.read_u32_leb128()?);
    }

    Ok(funcs)
}

fn decode_table_section(data: &[u8]) -> Result<Vec<TableType>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut tables = Vec::with_capacity(count);

    for _ in 0..count {
        let elem_type = ValueType::from_byte(decoder.read_u8()?)?;
        let limits = decode_limits(&mut decoder)?;
        tables.push(TableType { limits, elem_type });
    }

    Ok(tables)
}

fn decode_memory_section(data: &[u8]) -> Result<Vec<MemType>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut memories = Vec::with_capacity(count);

    for _ in 0..count {
        let limits = decode_limits(&mut decoder)?;
        memories.push(MemType { limits });
    }

    Ok(memories)
}

fn decode_limits(decoder: &mut Decoder) -> Result<Limits> {
    let flags = decoder.read_u8()?;
    let has_max = flags & 0x01 != 0;
    let memory64 = flags & 0x04 != 0;
    let shared = flags & 0x02 != 0;

    let min = decoder.read_u32_leb128()?;
    let max = if has_max { Some(decoder.read_u32_leb128()?) } else { None };

    Ok(Limits { min, max, memory64, shared })
}

fn decode_global_section(data: &[u8]) -> Result<Vec<Global>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut globals = Vec::with_capacity(count);

    for _ in 0..count {
        let value_type = ValueType::from_byte(decoder.read_u8()?)?;
        let mutable = decoder.read_u8()? == 0x01;
        let ty = GlobalType { value_type, mutable };
        let init = decode_instructions(&mut decoder)?;
        globals.push(Global { ty, init });
    }

    Ok(globals)
}

fn decode_export_section(data: &[u8]) -> Result<Vec<Export>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut exports = Vec::with_capacity(count);

    for _ in 0..count {
        let name = decoder.read_name()?;
        let kind = ExternalKind::from_byte(decoder.read_u8()?)?;
        let idx = decoder.read_u32_leb128()?;
        exports.push(Export { name, kind, idx });
    }

    Ok(exports)
}

fn decode_start_section(data: &[u8]) -> Result<u32> {
    let mut decoder = Decoder::new(data);
    decoder.read_u32_leb128()
}

fn decode_element_section(data: &[u8]) -> Result<Vec<ElementSegment>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut elements = Vec::with_capacity(count);

    for _ in 0..count {
        let flags = decoder.read_u32_leb128()?;
        let mode = if flags == 0 {
            let offset = decode_instructions(&mut decoder)?;
            ElementMode::Active { table_idx: 0, offset }
        } else if flags == 1 {
            ElementMode::Passive
        } else if flags == 2 {
            let table_idx = decoder.read_u32_leb128()?;
            let offset = decode_instructions(&mut decoder)?;
            ElementMode::Active { table_idx, offset }
        } else if flags == 3 {
            ElementMode::Declared
        } else {
            return Err(WasmError::InvalidElementKind(flags as u8));
        };

        let elem_type = ValueType::FuncRef;
        let func_count = decoder.read_u32_leb128()? as usize;
        let mut init = Vec::with_capacity(func_count);
        for _ in 0..func_count {
            init.push(decoder.read_u32_leb128()?);
        }

        elements.push(ElementSegment { mode, elem_type, init });
    }

    Ok(elements)
}

fn decode_code_section(data: &[u8]) -> Result<Vec<FunctionBody>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut bodies = Vec::with_capacity(count);

    for _ in 0..count {
        let body_size = decoder.read_u32_leb128()? as usize;
        let body_start = decoder.pos;

        let local_count = decoder.read_u32_leb128()? as usize;
        let mut locals = Vec::with_capacity(local_count);

        for _ in 0..local_count {
            let count = decoder.read_u32_leb128()?;
            let ty = ValueType::from_byte(decoder.read_u8()?)?;
            locals.push(Local { count, ty });
        }

        let instructions = decode_instructions(&mut decoder)?;

        if decoder.pos - body_start != body_size {
            return Err(WasmError::ValidationError("Code size mismatch".to_string()));
        }

        bodies.push(FunctionBody { locals, instructions });
    }

    Ok(bodies)
}

fn decode_data_section(data: &[u8]) -> Result<Vec<DataSegment>> {
    let mut decoder = Decoder::new(data);
    let count = decoder.read_u32_leb128()? as usize;
    let mut segments = Vec::with_capacity(count);

    for _ in 0..count {
        let flags = decoder.read_u32_leb128()?;
        let mode = if flags == 0 {
            let offset = decode_instructions(&mut decoder)?;
            DataMode::Active { mem_idx: 0, offset }
        } else if flags == 1 {
            DataMode::Passive
        } else if flags == 2 {
            let mem_idx = decoder.read_u32_leb128()?;
            let offset = decode_instructions(&mut decoder)?;
            DataMode::Active { mem_idx, offset }
        } else {
            return Err(WasmError::InvalidDataMode(flags as u8));
        };

        let data_len = decoder.read_u32_leb128()? as usize;
        let data = decoder.consume(data_len)?.to_vec();

        segments.push(DataSegment { mode, data });
    }

    Ok(segments)
}

fn decode_datacount_section(data: &[u8]) -> Result<u32> {
    let mut decoder = Decoder::new(data);
    decoder.read_u32_leb128()
}

fn decode_instructions(decoder: &mut Decoder) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();

    loop {
        let opcode_byte = decoder.read_u8()?;

        let instr = match opcode_byte {
            0x00 => Instruction::Unreachable,
            0x01 => Instruction::Nop,
            0x02 => {
                let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
                let body = decode_instructions_until_end(decoder)?;
                Instruction::Block { block_type, body }
            }
            0x03 => {
                let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
                let body = decode_instructions_until_end(decoder)?;
                Instruction::Loop { block_type, body }
            }
            0x04 => {
                let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
                let (then_branch, else_branch) = decode_if_branches(decoder)?;
                Instruction::If { block_type, then_branch, else_branch }
            }
            0x0b => break,
            0x0c => Instruction::Br { label_idx: decoder.read_u32_leb128()? },
            0x0d => Instruction::BrIf { label_idx: decoder.read_u32_leb128()? },
            0x0e => {
                let label_count = decoder.read_u32_leb128()? as usize;
                let mut labels = Vec::with_capacity(label_count);
                for _ in 0..label_count {
                    labels.push(decoder.read_u32_leb128()?);
                }
                let default_label = decoder.read_u32_leb128()?;
                Instruction::BrTable { labels, default_label }
            }
            0x0f => Instruction::Return,
            0x10 => Instruction::Call { function_idx: decoder.read_u32_leb128()? },
            0x11 => {
                let type_idx = decoder.read_u32_leb128()?;
                let table_idx = decoder.read_u32_leb128()?;
                Instruction::CallIndirect { type_idx, table_idx }
            }
            0x1a => Instruction::Drop,
            0x1b => Instruction::Select,
            0x20 => Instruction::LocalGet { local_idx: decoder.read_u32_leb128()? },
            0x21 => Instruction::LocalSet { local_idx: decoder.read_u32_leb128()? },
            0x22 => Instruction::LocalTee { local_idx: decoder.read_u32_leb128()? },
            0x23 => Instruction::GlobalGet { global_idx: decoder.read_u32_leb128()? },
            0x24 => Instruction::GlobalSet { global_idx: decoder.read_u32_leb128()? },
            0x28 => Instruction::I32Load { mem_arg: decode_mem_arg(decoder)? },
            0x29 => Instruction::I64Load { mem_arg: decode_mem_arg(decoder)? },
            0x2a => Instruction::F32Load { mem_arg: decode_mem_arg(decoder)? },
            0x2b => Instruction::F64Load { mem_arg: decode_mem_arg(decoder)? },
            0x2c => Instruction::I32Load8S { mem_arg: decode_mem_arg(decoder)? },
            0x2d => Instruction::I32Load8U { mem_arg: decode_mem_arg(decoder)? },
            0x2e => Instruction::I32Load16S { mem_arg: decode_mem_arg(decoder)? },
            0x2f => Instruction::I32Load16U { mem_arg: decode_mem_arg(decoder)? },
            0x30 => Instruction::I64Load8S { mem_arg: decode_mem_arg(decoder)? },
            0x31 => Instruction::I64Load8U { mem_arg: decode_mem_arg(decoder)? },
            0x32 => Instruction::I64Load16S { mem_arg: decode_mem_arg(decoder)? },
            0x33 => Instruction::I64Load16U { mem_arg: decode_mem_arg(decoder)? },
            0x34 => Instruction::I64Load32S { mem_arg: decode_mem_arg(decoder)? },
            0x35 => Instruction::I64Load32U { mem_arg: decode_mem_arg(decoder)? },
            0x36 => Instruction::I32Store { mem_arg: decode_mem_arg(decoder)? },
            0x37 => Instruction::I64Store { mem_arg: decode_mem_arg(decoder)? },
            0x38 => Instruction::F32Store { mem_arg: decode_mem_arg(decoder)? },
            0x39 => Instruction::F64Store { mem_arg: decode_mem_arg(decoder)? },
            0x3a => Instruction::I32Store8 { mem_arg: decode_mem_arg(decoder)? },
            0x3b => Instruction::I32Store16 { mem_arg: decode_mem_arg(decoder)? },
            0x3c => Instruction::I64Store8 { mem_arg: decode_mem_arg(decoder)? },
            0x3d => Instruction::I64Store16 { mem_arg: decode_mem_arg(decoder)? },
            0x3e => Instruction::I64Store32 { mem_arg: decode_mem_arg(decoder)? },
            0x3f => {
                let mem_idx = decoder.read_u8()?;
                Instruction::MemorySize { mem_idx }
            }
            0x40 => {
                let mem_idx = decoder.read_u8()?;
                Instruction::MemoryGrow { mem_idx }
            }
            0x41 => Instruction::I32Const { value: decoder.read_i32_leb128()? },
            0x42 => Instruction::I64Const { value: decoder.read_i64_leb128()? },
            0x43 => Instruction::F32Const { value: decoder.read_f32_le()? },
            0x44 => Instruction::F64Const { value: decoder.read_f64_le()? },
            0x45 => Instruction::I32Eqz,
            0x46 => Instruction::I32Eq,
            0x47 => Instruction::I32Ne,
            0x48 => Instruction::I32LtS,
            0x49 => Instruction::I32LtU,
            0x4a => Instruction::I32GtS,
            0x4b => Instruction::I32GtU,
            0x4c => Instruction::I32LeS,
            0x4d => Instruction::I32LeU,
            0x4e => Instruction::I32GeS,
            0x4f => Instruction::I32GeU,
            0x50 => Instruction::I64Eqz,
            0x51 => Instruction::I64Eq,
            0x52 => Instruction::I64Ne,
            0x53 => Instruction::I64LtS,
            0x54 => Instruction::I64LtU,
            0x55 => Instruction::I64GtS,
            0x56 => Instruction::I64GtU,
            0x57 => Instruction::I64LeS,
            0x58 => Instruction::I64LeU,
            0x59 => Instruction::I64GeS,
            0x5a => Instruction::I64GeU,
            0x5b => Instruction::F32Eq,
            0x5c => Instruction::F32Ne,
            0x5d => Instruction::F32Lt,
            0x5e => Instruction::F32Gt,
            0x5f => Instruction::F32Le,
            0x60 => Instruction::F32Ge,
            0x61 => Instruction::F64Eq,
            0x62 => Instruction::F64Ne,
            0x63 => Instruction::F64Lt,
            0x64 => Instruction::F64Gt,
            0x65 => Instruction::F64Le,
            0x66 => Instruction::F64Ge,
            0x67 => Instruction::I32Clz,
            0x68 => Instruction::I32Ctz,
            0x69 => Instruction::I32Popcnt,
            0x6a => Instruction::I32Add,
            0x6b => Instruction::I32Sub,
            0x6c => Instruction::I32Mul,
            0x6d => Instruction::I32DivS,
            0x6e => Instruction::I32DivU,
            0x6f => Instruction::I32RemS,
            0x70 => Instruction::I32RemU,
            0x71 => Instruction::I32And,
            0x72 => Instruction::I32Or,
            0x73 => Instruction::I32Xor,
            0x74 => Instruction::I32Shl,
            0x75 => Instruction::I32ShrS,
            0x76 => Instruction::I32ShrU,
            0x77 => Instruction::I32Rotl,
            0x78 => Instruction::I32Rotr,
            0x79 => Instruction::I64Clz,
            0x7a => Instruction::I64Ctz,
            0x7b => Instruction::I64Popcnt,
            0x7c => Instruction::I64Add,
            0x7d => Instruction::I64Sub,
            0x7e => Instruction::I64Mul,
            0x7f => Instruction::I64DivS,
            0x80 => Instruction::I64DivU,
            0x81 => Instruction::I64RemS,
            0x82 => Instruction::I64RemU,
            0x83 => Instruction::I64And,
            0x84 => Instruction::I64Or,
            0x85 => Instruction::I64Xor,
            0x86 => Instruction::I64Shl,
            0x87 => Instruction::I64ShrS,
            0x88 => Instruction::I64ShrU,
            0x89 => Instruction::I64Rotl,
            0x8a => Instruction::I64Rotr,
            0x8b => Instruction::F32Abs,
            0x8c => Instruction::F32Neg,
            0x8d => Instruction::F32Ceil,
            0x8e => Instruction::F32Floor,
            0x8f => Instruction::F32Trunc,
            0x90 => Instruction::F32Nearest,
            0x91 => Instruction::F32Sqrt,
            0x92 => Instruction::F32Add,
            0x93 => Instruction::F32Sub,
            0x94 => Instruction::F32Mul,
            0x95 => Instruction::F32Div,
            0x96 => Instruction::F32Min,
            0x97 => Instruction::F32Max,
            0x98 => Instruction::F32Copysign,
            0x99 => Instruction::F64Abs,
            0x9a => Instruction::F64Neg,
            0x9b => Instruction::F64Ceil,
            0x9c => Instruction::F64Floor,
            0x9d => Instruction::F64Trunc,
            0x9e => Instruction::F64Nearest,
            0x9f => Instruction::F64Sqrt,
            0xa0 => Instruction::F64Add,
            0xa1 => Instruction::F64Sub,
            0xa2 => Instruction::F64Mul,
            0xa3 => Instruction::F64Div,
            0xa4 => Instruction::F64Min,
            0xa5 => Instruction::F64Max,
            0xa6 => Instruction::F64Copysign,
            0xa7 => Instruction::I32WrapI64,
            0xa8 => Instruction::I32TruncF32S,
            0xa9 => Instruction::I32TruncF32U,
            0xaa => Instruction::I32TruncF64S,
            0xab => Instruction::I32TruncF64U,
            0xac => Instruction::I64ExtendI32S,
            0xad => Instruction::I64ExtendI32U,
            0xae => Instruction::I64TruncF32S,
            0xaf => Instruction::I64TruncF32U,
            0xb0 => Instruction::I64TruncF64S,
            0xb1 => Instruction::I64TruncF64U,
            0xb2 => Instruction::F32ConvertI32S,
            0xb3 => Instruction::F32ConvertI32U,
            0xb4 => Instruction::F32ConvertI64S,
            0xb5 => Instruction::F32ConvertI64U,
            0xb6 => Instruction::F32DemoteF64,
            0xb7 => Instruction::F64ConvertI32S,
            0xb8 => Instruction::F64ConvertI32U,
            0xb9 => Instruction::F64ConvertI64S,
            0xba => Instruction::F64ConvertI64U,
            0xbb => Instruction::F64PromoteF32,
            0xbc => Instruction::I32ReinterpretF32,
            0xbd => Instruction::I64ReinterpretF64,
            0xbe => Instruction::F32ReinterpretI32,
            0xbf => Instruction::F64ReinterpretI64,
            0xc0 => Instruction::I32Extend8S,
            0xc1 => Instruction::I32Extend16S,
            0xc2 => Instruction::I64Extend8S,
            0xc3 => Instruction::I64Extend16S,
            0xc4 => Instruction::I64Extend32S,
            0xd0 => {
                let ty = ValueType::from_byte(decoder.read_u8()?)?;
                Instruction::RefNull { ty }
            }
            0xd1 => Instruction::RefIsNull,
            0xd2 => Instruction::RefFunc { func_idx: decoder.read_u32_leb128()? },
            _ => return Err(WasmError::InvalidOpcode(opcode_byte)),
        };

        instructions.push(instr);
    }

    Ok(instructions)
}

fn decode_instructions_until_end(decoder: &mut Decoder) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let _depth = 1;
    
    loop {
        let opcode_byte = decoder.read_u8()?;
        
        if opcode_byte == 0x0b {
            break;
        }
        
        // Handle nested blocks
        if opcode_byte == 0x02 || opcode_byte == 0x03 || opcode_byte == 0x04 {
            instructions.push(decode_single_instruction(decoder, opcode_byte)?);
        } else {
            decoder.pos -= 1; // Put back the byte
            let opcode = decoder.read_u8()?;
            instructions.push(decode_single_instruction(decoder, opcode)?);
        }
    }
    
    Ok(instructions)
}

fn decode_if_branches(decoder: &mut Decoder) -> Result<(Vec<Instruction>, Vec<Instruction>)> {
    let mut then_branch = Vec::new();
    let mut else_branch = Vec::new();
    let mut in_else = false;
    
    loop {
        let opcode_byte = decoder.read_u8()?;
        
        if opcode_byte == 0x0b {
            break;
        } else if opcode_byte == 0x05 {
            in_else = true;
            continue;
        }
        
        let instr = decode_single_instruction(decoder, opcode_byte)?;
        if in_else {
            else_branch.push(instr);
        } else {
            then_branch.push(instr);
        }
    }
    
    Ok((then_branch, else_branch))
}

fn decode_single_instruction(decoder: &mut Decoder, opcode_byte: u8) -> Result<Instruction> {
    // Simplified for brevity - in full implementation would decode all opcodes
    match opcode_byte {
        0x00 => Ok(Instruction::Unreachable),
        0x01 => Ok(Instruction::Nop),
        0x02 => {
            let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
            let body = decode_instructions_until_end(decoder)?;
            Ok(Instruction::Block { block_type, body })
        }
        0x20 => Ok(Instruction::LocalGet { local_idx: decoder.read_u32_leb128()? }),
        0x21 => Ok(Instruction::LocalSet { local_idx: decoder.read_u32_leb128()? }),
        0x41 => Ok(Instruction::I32Const { value: decoder.read_i32_leb128()? }),
        _ => Err(WasmError::InvalidOpcode(opcode_byte)),
    }
}

fn decode_mem_arg(decoder: &mut Decoder) -> Result<MemArg> {
    let align = decoder.read_u32_leb128()?;
    let offset = decoder.read_u32_leb128()?;
    Ok(MemArg { align, offset })
}

/// Parse WASM binary bytes into a Module
pub fn parse_bytes(bytes: &[u8]) -> Result<Module> {
    let mut decoder = Decoder::new(bytes);
    decoder.decode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_module() {
        // Minimal valid WASM module: magic + version
        let bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let module = parse_bytes(&bytes).unwrap();
        assert!(module.types.is_empty());
        assert!(module.funcs.is_empty());
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        assert!(parse_bytes(&bytes).is_err());
    }

    #[test]
    fn test_invalid_version() {
        let bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x02, 0x00, 0x00, 0x00];
        assert!(parse_bytes(&bytes).is_err());
    }
}
