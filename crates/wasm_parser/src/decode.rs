//! WebAssembly binary decoder

use crate::ast::*;
use crate::decode_instructions::{decode_instructions, decode_instructions_bounded, decode_instructions_until_end};
use crate::error::{Result, WasmError};
use crate::instruction::Instruction;
use crate::leb128;
use crate::types::*;
use crate::{SectionId, WASM_MAGIC, WASM_VERSION};

/// Binary decoder for WASM
pub struct Decoder<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub fn consume(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        let result = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(result)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        let byte = self.bytes[self.pos];
        self.pos += 1;
        Ok(byte)
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        let bytes = self.consume(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_u64_le(&mut self) -> Result<u64> {
        let bytes = self.consume(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_f32_le(&mut self) -> Result<f32> {
        let bytes = self.consume(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_f64_le(&mut self) -> Result<f64> {
        let bytes = self.consume(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_u32_leb128(&mut self) -> Result<u32> {
        let (value, size) = leb128::decode_u32(&self.bytes[self.pos..])?;
        self.pos += size;
        Ok(value)
    }

    pub fn read_i32_leb128(&mut self) -> Result<i32> {
        let (value, size) = leb128::decode_i32(&self.bytes[self.pos..])?;
        self.pos += size;
        Ok(value)
    }

    pub fn read_i64_leb128(&mut self) -> Result<i64> {
        let (value, size) = leb128::decode_i64(&self.bytes[self.pos..])?;
        self.pos += size;
        Ok(value)
    }

    pub fn read_name(&mut self) -> Result<String> {
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
    let section_end = data.len();
    let count = decoder.read_u32_leb128()? as usize;
    let mut globals = Vec::with_capacity(count);

    for _ in 0..count {
        let value_type = ValueType::from_byte(decoder.read_u8()?)?;
        let mutable = decoder.read_u8()? == 0x01;
        let ty = GlobalType { value_type, mutable };
        let init = decode_instructions_bounded(&mut decoder, section_end)?;
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
    let section_end = data.len();
    let count = decoder.read_u32_leb128()? as usize;
    let mut elements = Vec::with_capacity(count);

    for i in 0..count {
        
        if decoder.pos >= section_end {
            eprintln!("WARNING: Reached section end early at element {}/{}", i, count);
            break;
        }
        
        let flags = decoder.read_u32_leb128()?;
        
        // Determine mode based on flags
        let mode = if flags == 0 {
            // Active, table idx 0, offset is expr
            let offset = decode_instructions_bounded(&mut decoder, section_end)?;
            ElementMode::Active { table_idx: 0, offset }
        } else if flags == 1 {
            // Passive
            ElementMode::Passive
        } else if flags == 2 {
            // Active, table idx follows, offset is expr
            let table_idx = decoder.read_u32_leb128()?;
            let offset = decode_instructions_bounded(&mut decoder, section_end)?;
            ElementMode::Active { table_idx, offset }
        } else if flags == 3 {
            // Declared
            ElementMode::Declared
        } else if flags == 4 {
            // Active, table idx 0, offset is expr, elem type is expr-based
            let offset = decode_instructions_bounded(&mut decoder, section_end)?;
            ElementMode::Active { table_idx: 0, offset }
        } else if flags == 5 {
            // Passive, elem type is expr-based
            ElementMode::Passive
        } else if flags == 6 {
            // Active, table idx follows, offset is expr, elem type is expr-based
            let table_idx = decoder.read_u32_leb128()?;
            let offset = decode_instructions_bounded(&mut decoder, section_end)?;
            ElementMode::Active { table_idx, offset }
        } else if flags == 7 {
            // Declared, elem type is expr-based
            ElementMode::Declared
        } else {
            return Err(WasmError::InvalidElementKind(flags as u8));
        };

        // Check if elem type should be read
        // flags 1, 2, 3 have explicit elemkind byte
        // flags 0 has implicit funcref
        // flags 4+ are bulk memory/expr-based
        let elem_type = if flags == 1 || flags == 2 || flags == 3 {
            // Read elem type for passive/active-explicit/declared segments
            ValueType::from_byte(decoder.read_u8()?)?
        } else {
            // For flags 0 and expr-based (4+), default to funcref
            ValueType::FuncRef
        };

        let func_count = decoder.read_u32_leb128()? as usize;
        let mut init = Vec::with_capacity(func_count);
        
        // Check if expr-based (flags & 0x04 != 0)
        if flags & 0x04 != 0 {
            // Expr-based: each init is an expr that evaluates to a ref
            for _ in 0..func_count {
                // Just decode the expression and discard for now
                // In a full implementation, we'd store the expression
                let _expr = decode_instructions_bounded(&mut decoder, section_end)?;
                init.push(0); // Placeholder - would need to store expr instead
            }
        } else {
            // Function index based
            for _ in 0..func_count {
                init.push(decoder.read_u32_leb128()?);
            }
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
    let section_end = data.len();
    let count = decoder.read_u32_leb128()? as usize;
    let mut segments = Vec::with_capacity(count);

    for _ in 0..count {
        let flags = decoder.read_u32_leb128()?;
        let mode = if flags == 0 {
            let offset = decode_instructions_bounded(&mut decoder, section_end)?;
            DataMode::Active { mem_idx: 0, offset }
        } else if flags == 1 {
            DataMode::Passive
        } else if flags == 2 {
            let mem_idx = decoder.read_u32_leb128()?;
            let offset = decode_instructions_bounded(&mut decoder, section_end)?;
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
