//! Instruction decoding for WebAssembly

use crate::decode::Decoder;
use crate::error::{Result, WasmError};
use crate::instruction::{Instruction, MemArg};
use crate::types::{BlockType, ValueType};

/// Decode a single instruction from the decoder
pub fn decode_single_instruction(decoder: &mut Decoder) -> Result<Instruction> {
    let opcode = decoder.read_u8()?;
    decode_instruction_with_opcode(decoder, opcode)
}

fn decode_instruction_with_opcode(decoder: &mut Decoder, opcode: u8) -> Result<Instruction> {
    match opcode {
        // Control instructions
        0x00 => Ok(Instruction::Unreachable),
        0x01 => Ok(Instruction::Nop),
        0x02 => {
            let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
            let body = decode_instructions_until_end(decoder)?;
            Ok(Instruction::Block { block_type, body })
        }
        0x03 => {
            let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
            let body = decode_instructions_until_end(decoder)?;
            Ok(Instruction::Loop { block_type, body })
        }
        0x04 => {
            let block_type = BlockType::from_i64(decoder.read_i32_leb128()? as i64)?;
            let (then_branch, else_branch) = decode_if_branches(decoder)?;
            Ok(Instruction::If { block_type, then_branch, else_branch })
        }
        0x05 => Ok(Instruction::Else),
        0x0b => Ok(Instruction::End),
        0x0c => Ok(Instruction::Br { label_idx: decoder.read_u32_leb128()? }),
        0x0d => Ok(Instruction::BrIf { label_idx: decoder.read_u32_leb128()? }),
        0x0e => {
            let num_labels = decoder.read_u32_leb128()? as usize;
            let mut labels = Vec::with_capacity(num_labels);
            for _ in 0..num_labels {
                labels.push(decoder.read_u32_leb128()?);
            }
            let default_label = decoder.read_u32_leb128()?;
            Ok(Instruction::BrTable { labels, default_label })
        }
        0x0f => Ok(Instruction::Return),
        0x10 => Ok(Instruction::Call { function_idx: decoder.read_u32_leb128()? }),
        0x11 => {
            let type_idx = decoder.read_u32_leb128()?;
            let table_idx = decoder.read_u32_leb128()?;
            Ok(Instruction::CallIndirect { type_idx, table_idx })
        }
        0x12 => Ok(Instruction::ReturnCall { function_idx: decoder.read_u32_leb128()? }),
        0x13 => {
            let type_idx = decoder.read_u32_leb128()?;
            let table_idx = decoder.read_u32_leb128()?;
            Ok(Instruction::ReturnCallIndirect { type_idx, table_idx })
        }
        0x14 => Ok(Instruction::CallRef { type_idx: decoder.read_u32_leb128()? }),
        0x15 => Ok(Instruction::ReturnCallRef { type_idx: decoder.read_u32_leb128()? }),

        // Parametric instructions
        0x1a => Ok(Instruction::Drop),
        0x1b => Ok(Instruction::Select),

        // Variable instructions
        0x20 => Ok(Instruction::LocalGet { local_idx: decoder.read_u32_leb128()? }),
        0x21 => Ok(Instruction::LocalSet { local_idx: decoder.read_u32_leb128()? }),
        0x22 => Ok(Instruction::LocalTee { local_idx: decoder.read_u32_leb128()? }),
        0x23 => Ok(Instruction::GlobalGet { global_idx: decoder.read_u32_leb128()? }),
        0x24 => Ok(Instruction::GlobalSet { global_idx: decoder.read_u32_leb128()? }),

        // Memory instructions
        0x28 => Ok(Instruction::I32Load { mem_arg: decode_mem_arg(decoder)? }),
        0x29 => Ok(Instruction::I64Load { mem_arg: decode_mem_arg(decoder)? }),
        0x2a => Ok(Instruction::F32Load { mem_arg: decode_mem_arg(decoder)? }),
        0x2b => Ok(Instruction::F64Load { mem_arg: decode_mem_arg(decoder)? }),
        0x2c => Ok(Instruction::I32Load8S { mem_arg: decode_mem_arg(decoder)? }),
        0x2d => Ok(Instruction::I32Load8U { mem_arg: decode_mem_arg(decoder)? }),
        0x2e => Ok(Instruction::I32Load16S { mem_arg: decode_mem_arg(decoder)? }),
        0x2f => Ok(Instruction::I32Load16U { mem_arg: decode_mem_arg(decoder)? }),
        0x30 => Ok(Instruction::I64Load8S { mem_arg: decode_mem_arg(decoder)? }),
        0x31 => Ok(Instruction::I64Load8U { mem_arg: decode_mem_arg(decoder)? }),
        0x32 => Ok(Instruction::I64Load16S { mem_arg: decode_mem_arg(decoder)? }),
        0x33 => Ok(Instruction::I64Load16U { mem_arg: decode_mem_arg(decoder)? }),
        0x34 => Ok(Instruction::I64Load32S { mem_arg: decode_mem_arg(decoder)? }),
        0x35 => Ok(Instruction::I64Load32U { mem_arg: decode_mem_arg(decoder)? }),
        0x36 => Ok(Instruction::I32Store { mem_arg: decode_mem_arg(decoder)? }),
        0x37 => Ok(Instruction::I64Store { mem_arg: decode_mem_arg(decoder)? }),
        0x38 => Ok(Instruction::F32Store { mem_arg: decode_mem_arg(decoder)? }),
        0x39 => Ok(Instruction::F64Store { mem_arg: decode_mem_arg(decoder)? }),
        0x3a => Ok(Instruction::I32Store8 { mem_arg: decode_mem_arg(decoder)? }),
        0x3b => Ok(Instruction::I32Store16 { mem_arg: decode_mem_arg(decoder)? }),
        0x3c => Ok(Instruction::I64Store8 { mem_arg: decode_mem_arg(decoder)? }),
        0x3d => Ok(Instruction::I64Store16 { mem_arg: decode_mem_arg(decoder)? }),
        0x3e => Ok(Instruction::I64Store32 { mem_arg: decode_mem_arg(decoder)? }),
        0x3f => {
            // Memory size has a reserved byte
            decoder.read_u8()?; // reserved (must be 0x00)
            Ok(Instruction::MemorySize { mem_idx: 0 })
        }
        0x40 => {
            // Memory grow has a reserved byte
            decoder.read_u8()?; // reserved (must be 0x00)
            Ok(Instruction::MemoryGrow { mem_idx: 0 })
        }

        // Numeric constants
        0x41 => Ok(Instruction::I32Const { value: decoder.read_i32_leb128()? }),
        0x42 => Ok(Instruction::I64Const { value: decoder.read_i64_leb128()? }),
        0x43 => Ok(Instruction::F32Const { value: decoder.read_f32_le()? }),
        0x44 => Ok(Instruction::F64Const { value: decoder.read_f64_le()? }),

        // Comparison: i32
        0x45 => Ok(Instruction::I32Eqz),
        0x46 => Ok(Instruction::I32Eq),
        0x47 => Ok(Instruction::I32Ne),
        0x48 => Ok(Instruction::I32LtS),
        0x49 => Ok(Instruction::I32LtU),
        0x4a => Ok(Instruction::I32GtS),
        0x4b => Ok(Instruction::I32GtU),
        0x4c => Ok(Instruction::I32LeS),
        0x4d => Ok(Instruction::I32LeU),
        0x4e => Ok(Instruction::I32GeS),
        0x4f => Ok(Instruction::I32GeU),

        // Comparison: i64
        0x50 => Ok(Instruction::I64Eqz),
        0x51 => Ok(Instruction::I64Eq),
        0x52 => Ok(Instruction::I64Ne),
        0x53 => Ok(Instruction::I64LtS),
        0x54 => Ok(Instruction::I64LtU),
        0x55 => Ok(Instruction::I64GtS),
        0x56 => Ok(Instruction::I64GtU),
        0x57 => Ok(Instruction::I64LeS),
        0x58 => Ok(Instruction::I64LeU),
        0x59 => Ok(Instruction::I64GeS),
        0x5a => Ok(Instruction::I64GeU),

        // Comparison: f32
        0x5b => Ok(Instruction::F32Eq),
        0x5c => Ok(Instruction::F32Ne),
        0x5d => Ok(Instruction::F32Lt),
        0x5e => Ok(Instruction::F32Gt),
        0x5f => Ok(Instruction::F32Le),
        0x60 => Ok(Instruction::F32Ge),

        // Comparison: f64
        0x61 => Ok(Instruction::F64Eq),
        0x62 => Ok(Instruction::F64Ne),
        0x63 => Ok(Instruction::F64Lt),
        0x64 => Ok(Instruction::F64Gt),
        0x65 => Ok(Instruction::F64Le),
        0x66 => Ok(Instruction::F64Ge),

        // i32 arithmetic
        0x67 => Ok(Instruction::I32Clz),
        0x68 => Ok(Instruction::I32Ctz),
        0x69 => Ok(Instruction::I32Popcnt),
        0x6a => Ok(Instruction::I32Add),
        0x6b => Ok(Instruction::I32Sub),
        0x6c => Ok(Instruction::I32Mul),
        0x6d => Ok(Instruction::I32DivS),
        0x6e => Ok(Instruction::I32DivU),
        0x6f => Ok(Instruction::I32RemS),
        0x70 => Ok(Instruction::I32RemU),
        0x71 => Ok(Instruction::I32And),
        0x72 => Ok(Instruction::I32Or),
        0x73 => Ok(Instruction::I32Xor),
        0x74 => Ok(Instruction::I32Shl),
        0x75 => Ok(Instruction::I32ShrS),
        0x76 => Ok(Instruction::I32ShrU),
        0x77 => Ok(Instruction::I32Rotl),
        0x78 => Ok(Instruction::I32Rotr),

        // i64 arithmetic
        0x79 => Ok(Instruction::I64Clz),
        0x7a => Ok(Instruction::I64Ctz),
        0x7b => Ok(Instruction::I64Popcnt),
        0x7c => Ok(Instruction::I64Add),
        0x7d => Ok(Instruction::I64Sub),
        0x7e => Ok(Instruction::I64Mul),
        0x7f => Ok(Instruction::I64DivS),
        0x80 => Ok(Instruction::I64DivU),
        0x81 => Ok(Instruction::I64RemS),
        0x82 => Ok(Instruction::I64RemU),
        0x83 => Ok(Instruction::I64And),
        0x84 => Ok(Instruction::I64Or),
        0x85 => Ok(Instruction::I64Xor),
        0x86 => Ok(Instruction::I64Shl),
        0x87 => Ok(Instruction::I64ShrS),
        0x88 => Ok(Instruction::I64ShrU),
        0x89 => Ok(Instruction::I64Rotl),
        0x8a => Ok(Instruction::I64Rotr),

        // f32 arithmetic
        0x8b => Ok(Instruction::F32Abs),
        0x8c => Ok(Instruction::F32Neg),
        0x8d => Ok(Instruction::F32Ceil),
        0x8e => Ok(Instruction::F32Floor),
        0x8f => Ok(Instruction::F32Trunc),
        0x90 => Ok(Instruction::F32Nearest),
        0x91 => Ok(Instruction::F32Sqrt),
        0x92 => Ok(Instruction::F32Add),
        0x93 => Ok(Instruction::F32Sub),
        0x94 => Ok(Instruction::F32Mul),
        0x95 => Ok(Instruction::F32Div),
        0x96 => Ok(Instruction::F32Min),
        0x97 => Ok(Instruction::F32Max),
        0x98 => Ok(Instruction::F32Copysign),

        // f64 arithmetic
        0x99 => Ok(Instruction::F64Abs),
        0x9a => Ok(Instruction::F64Neg),
        0x9b => Ok(Instruction::F64Ceil),
        0x9c => Ok(Instruction::F64Floor),
        0x9d => Ok(Instruction::F64Trunc),
        0x9e => Ok(Instruction::F64Nearest),
        0x9f => Ok(Instruction::F64Sqrt),
        0xa0 => Ok(Instruction::F64Add),
        0xa1 => Ok(Instruction::F64Sub),
        0xa2 => Ok(Instruction::F64Mul),
        0xa3 => Ok(Instruction::F64Div),
        0xa4 => Ok(Instruction::F64Min),
        0xa5 => Ok(Instruction::F64Max),
        0xa6 => Ok(Instruction::F64Copysign),

        // Conversions
        0xa7 => Ok(Instruction::I32WrapI64),
        0xa8 => Ok(Instruction::I32TruncF32S),
        0xa9 => Ok(Instruction::I32TruncF32U),
        0xaa => Ok(Instruction::I32TruncF64S),
        0xab => Ok(Instruction::I32TruncF64U),
        0xac => Ok(Instruction::I64ExtendI32S),
        0xad => Ok(Instruction::I64ExtendI32U),
        0xae => Ok(Instruction::I64TruncF32S),
        0xaf => Ok(Instruction::I64TruncF32U),
        0xb0 => Ok(Instruction::I64TruncF64S),
        0xb1 => Ok(Instruction::I64TruncF64U),
        0xb2 => Ok(Instruction::F32ConvertI32S),
        0xb3 => Ok(Instruction::F32ConvertI32U),
        0xb4 => Ok(Instruction::F32ConvertI64S),
        0xb5 => Ok(Instruction::F32ConvertI64U),
        0xb6 => Ok(Instruction::F32DemoteF64),
        0xb7 => Ok(Instruction::F64ConvertI32S),
        0xb8 => Ok(Instruction::F64ConvertI32U),
        0xb9 => Ok(Instruction::F64ConvertI64S),
        0xba => Ok(Instruction::F64ConvertI64U),
        0xbb => Ok(Instruction::F64PromoteF32),
        0xbc => Ok(Instruction::I32ReinterpretF32),
        0xbd => Ok(Instruction::I64ReinterpretF64),
        0xbe => Ok(Instruction::F32ReinterpretI32),
        0xbf => Ok(Instruction::F64ReinterpretI64),

        // Sign extension opcodes
        0xc0 => Ok(Instruction::I32Extend8S),
        0xc1 => Ok(Instruction::I32Extend16S),
        0xc2 => Ok(Instruction::I64Extend8S),
        0xc3 => Ok(Instruction::I64Extend16S),
        0xc4 => Ok(Instruction::I64Extend32S),

        // Reserved opcodes (0xc5-0xcf) - skip/ignore
        0xc5 | 0xc6 | 0xc7 | 0xc8 | 0xc9 | 0xca | 0xcb | 0xcc | 0xcd | 0xce | 0xcf => Ok(Instruction::Nop),

        // Bulk memory operations (0xfc prefix)
        0xfc => {
            let sub_opcode = decoder.read_u8()?;
            match sub_opcode {
                0x08 => {
                    let data_idx = decoder.read_u32_leb128()?;
                    decoder.read_u8()?; // reserved
                    Ok(Instruction::MemoryInit { data_idx, mem_idx: 0 })
                }
                0x09 => Ok(Instruction::DataDrop { data_idx: decoder.read_u32_leb128()? }),
                0x0a => {
                    decoder.read_u8()?; // reserved
                    decoder.read_u8()?; // reserved
                    Ok(Instruction::MemoryCopy { src_mem: 0, dst_mem: 0 })
                }
                0x0b => {
                    decoder.read_u8()?; // reserved
                    Ok(Instruction::MemoryFill { mem_idx: 0 })
                }
                _ => Err(WasmError::InvalidOpcode(0xfc)),
            }
        }

        // Reference types (0xd3 prefix)
        0xd0 => Ok(Instruction::RefNull { ty: ValueType::from_byte(decoder.read_u8()?)? }),
        0xd1 => Ok(Instruction::RefIsNull),
        0xd2 => Ok(Instruction::RefFunc { func_idx: decoder.read_u32_leb128()? }),

        _ => {
            eprintln!("DEBUG: Invalid opcode 0x{:02x} at pos {}/{}", 
                opcode, decoder.pos, decoder.bytes.len());
            Err(WasmError::InvalidOpcode(opcode))
        }
    }
}

pub fn decode_mem_arg(decoder: &mut Decoder) -> Result<MemArg> {
    let align = decoder.read_u32_leb128()?;
    let offset = decoder.read_u32_leb128()?;
    Ok(MemArg { align, offset })
}

fn decode_if_branches(decoder: &mut Decoder) -> Result<(Vec<Instruction>, Vec<Instruction>)> {
    let mut then_branch = Vec::new();
    let mut else_branch = Vec::new();
    let mut depth = 0;
    let mut in_else = false;

    loop {
        match decoder.peek() {
            Some(0x0b) if depth == 0 => {
                decoder.read_u8()?; // consume end
                break;
            }
            Some(0x05) if depth == 0 => {
                decoder.read_u8()?; // consume else
                in_else = true;
            }
            Some(0x02) | Some(0x03) | Some(0x04) => {
                depth += 1;
                let instr = decode_single_instruction(decoder)?;
                if in_else {
                    else_branch.push(instr);
                } else {
                    then_branch.push(instr);
                }
            }
            Some(0x0b) => {
                depth -= 1;
                let instr = decode_single_instruction(decoder)?;
                if in_else {
                    else_branch.push(instr);
                } else {
                    then_branch.push(instr);
                }
            }
            Some(_) => {
                let instr = decode_single_instruction(decoder)?;
                if in_else {
                    else_branch.push(instr);
                } else {
                    then_branch.push(instr);
                }
            }
            None => return Err(WasmError::UnexpectedEof),
        }
    }

    Ok((then_branch, else_branch))
}

pub fn decode_instructions_until_end(decoder: &mut Decoder) -> Result<Vec<Instruction>> {
    decode_instructions_until_end_bounded(decoder, None)
}

/// Decode instructions until end (0x0b) or until reaching a boundary position
/// If end_pos is Some(pos), decoding stops when decoder.pos >= pos
pub fn decode_instructions_until_end_bounded(
    decoder: &mut Decoder,
    end_pos: Option<usize>,
) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let mut depth = 0;
    let mut iteration = 0;

    loop {
        iteration += 1;
        if iteration > 1000 {
            eprintln!("DEBUG: Too many iterations in decode_instructions_until_end_bounded");
            return Err(WasmError::UnexpectedEof);
        }

        // Check if we've reached the boundary
        if let Some(end) = end_pos {
            if decoder.pos >= end {
                eprintln!("DEBUG: decode_instructions_until_end_bounded hit boundary at pos {}/end {}", decoder.pos, end);
                // At boundary without finding end - this is an error
                return Err(WasmError::UnexpectedEof);
            }
        }

        let byte = decoder.peek();
        eprintln!("DEBUG: decode_instructions_until_end_bounded iteration {} pos {} byte {:?} depth {}", 
            iteration, decoder.pos, byte, depth);

        match byte {
            Some(0x0b) if depth == 0 => {
                decoder.read_u8()?; // consume end
                eprintln!("DEBUG: Found end at pos {}", decoder.pos);
                break;
            }
            Some(0x02) | Some(0x03) | Some(0x04) => {
                depth += 1;
                instructions.push(decode_single_instruction(decoder)?);
            }
            Some(0x0b) => {
                depth -= 1;
                instructions.push(decode_single_instruction(decoder)?);
            }
            Some(0x05) if depth == 0 => {
                // else at depth 0 - stop here, don't consume
                eprintln!("DEBUG: Found else at pos {}", decoder.pos);
                break;
            }
            Some(_) => {
                instructions.push(decode_single_instruction(decoder)?);
            }
            None => return Err(WasmError::UnexpectedEof),
        }
    }

    Ok(instructions)
}

/// Decode instructions until end of function body (0x0b)
pub fn decode_instructions(decoder: &mut Decoder) -> Result<Vec<Instruction>> {
    decode_instructions_until_end(decoder)
}

/// Decode instructions with a known end position (for section-bounded decoding)
pub fn decode_instructions_bounded(decoder: &mut Decoder, end_pos: usize) -> Result<Vec<Instruction>> {
    decode_instructions_until_end_bounded(decoder, Some(end_pos))
}
