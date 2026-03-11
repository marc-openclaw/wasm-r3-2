//! WebAssembly instruction definitions

use crate::types::{BlockType, ValueType};

/// WebAssembly instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Control instructions
    Unreachable = 0x00,
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    End = 0x0b,
    Br = 0x0c,
    BrIf = 0x0d,
    BrTable = 0x0e,
    Return = 0x0f,
    Call = 0x10,
    CallIndirect = 0x11,

    // Parametric instructions
    Drop = 0x1a,
    Select = 0x1b,

    // Variable instructions
    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,
    GlobalGet = 0x23,
    GlobalSet = 0x24,

    // Memory instructions
    I32Load = 0x28,
    I64Load = 0x29,
    F32Load = 0x2a,
    F64Load = 0x2b,
    I32Load8S = 0x2c,
    I32Load8U = 0x2d,
    I32Load16S = 0x2e,
    I32Load16U = 0x2f,
    I64Load8S = 0x30,
    I64Load8U = 0x31,
    I64Load16S = 0x32,
    I64Load16U = 0x33,
    I64Load32S = 0x34,
    I64Load32U = 0x35,
    I32Store = 0x36,
    I64Store = 0x37,
    F32Store = 0x38,
    F64Store = 0x39,
    I32Store8 = 0x3a,
    I32Store16 = 0x3b,
    I64Store8 = 0x3c,
    I64Store16 = 0x3d,
    I64Store32 = 0x3e,
    MemorySize = 0x3f,
    MemoryGrow = 0x40,

    // Numeric instructions
    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,

    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4a,
    I32GtU = 0x4b,
    I32LeS = 0x4c,
    I32LeU = 0x4d,
    I32GeS = 0x4e,
    I32GeU = 0x4f,

    I64Eqz = 0x50,
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5a,

    F32Eq = 0x5b,
    F32Ne = 0x5c,
    F32Lt = 0x5d,
    F32Gt = 0x5e,
    F32Le = 0x5f,
    F32Ge = 0x60,

    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,

    I32Clz = 0x67,
    I32Ctz = 0x68,
    I32Popcnt = 0x69,
    I32Add = 0x6a,
    I32Sub = 0x6b,
    I32Mul = 0x6c,
    I32DivS = 0x6d,
    I32DivU = 0x6e,
    I32RemS = 0x6f,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    I32Rotl = 0x77,
    I32Rotr = 0x78,

    I64Clz = 0x79,
    I64Ctz = 0x7a,
    I64Popcnt = 0x7b,
    I64Add = 0x7c,
    I64Sub = 0x7d,
    I64Mul = 0x7e,
    I64DivS = 0x7f,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64Shl = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,
    I64Rotl = 0x89,
    I64Rotr = 0x8a,

    F32Abs = 0x8b,
    F32Neg = 0x8c,
    F32Ceil = 0x8d,
    F32Floor = 0x8e,
    F32Trunc = 0x8f,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,
    F32Copysign = 0x98,

    F64Abs = 0x99,
    F64Neg = 0x9a,
    F64Ceil = 0x9b,
    F64Floor = 0x9c,
    F64Trunc = 0x9d,
    F64Nearest = 0x9e,
    F64Sqrt = 0x9f,
    F64Add = 0xa0,
    F64Sub = 0xa1,
    F64Mul = 0xa2,
    F64Div = 0xa3,
    F64Min = 0xa4,
    F64Max = 0xa5,
    F64Copysign = 0xa6,

    I32WrapI64 = 0xa7,
    I32TruncF32S = 0xa8,
    I32TruncF32U = 0xa9,
    I32TruncF64S = 0xaa,
    I32TruncF64U = 0xab,
    I64ExtendI32S = 0xac,
    I64ExtendI32U = 0xad,
    I64TruncF32S = 0xae,
    I64TruncF32U = 0xaf,
    I64TruncF64S = 0xb0,
    I64TruncF64U = 0xb1,
    F32ConvertI32S = 0xb2,
    F32ConvertI32U = 0xb3,
    F32ConvertI64S = 0xb4,
    F32ConvertI64U = 0xb5,
    F32DemoteF64 = 0xb6,
    F64ConvertI32S = 0xb7,
    F64ConvertI32U = 0xb8,
    F64ConvertI64S = 0xb9,
    F64ConvertI64U = 0xba,
    F64PromoteF32 = 0xbb,
    I32ReinterpretF32 = 0xbc,
    I64ReinterpretF64 = 0xbd,
    F32ReinterpretI32 = 0xbe,
    F64ReinterpretI64 = 0xbf,

    I32Extend8S = 0xc0,
    I32Extend16S = 0xc1,
    I64Extend8S = 0xc2,
    I64Extend16S = 0xc3,
    I64Extend32S = 0xc4,

    // Reference types
    RefNull = 0xd0,
    RefIsNull = 0xd1,
    RefFunc = 0xd2,

    // Bulk memory operations (0xfc prefix, followed by sub-opcode)
    // These are handled specially in the decoder
    BulkMemoryPrefix = 0xfc,
}

/// Memory argument for load/store instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl MemArg {
    pub fn new(align: u32, offset: u32) -> Self {
        Self { align, offset }
    }
}

/// Instruction with operands
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Control instructions
    Unreachable,
    Nop,
    Block { block_type: BlockType, body: Vec<Instruction> },
    Loop { block_type: BlockType, body: Vec<Instruction> },
    If { block_type: BlockType, then_branch: Vec<Instruction>, else_branch: Vec<Instruction> },
    Else,
    End,
    Br { label_idx: u32 },
    BrIf { label_idx: u32 },
    BrTable { labels: Vec<u32>, default_label: u32 },
    Return,
    Call { function_idx: u32 },
    CallIndirect { type_idx: u32, table_idx: u32 },

    // Parametric instructions
    Drop,
    Select,

    // Variable instructions
    LocalGet { local_idx: u32 },
    LocalSet { local_idx: u32 },
    LocalTee { local_idx: u32 },
    GlobalGet { global_idx: u32 },
    GlobalSet { global_idx: u32 },

    // Memory instructions
    I32Load { mem_arg: MemArg },
    I64Load { mem_arg: MemArg },
    F32Load { mem_arg: MemArg },
    F64Load { mem_arg: MemArg },
    I32Load8S { mem_arg: MemArg },
    I32Load8U { mem_arg: MemArg },
    I32Load16S { mem_arg: MemArg },
    I32Load16U { mem_arg: MemArg },
    I64Load8S { mem_arg: MemArg },
    I64Load8U { mem_arg: MemArg },
    I64Load16S { mem_arg: MemArg },
    I64Load16U { mem_arg: MemArg },
    I64Load32S { mem_arg: MemArg },
    I64Load32U { mem_arg: MemArg },
    I32Store { mem_arg: MemArg },
    I64Store { mem_arg: MemArg },
    F32Store { mem_arg: MemArg },
    F64Store { mem_arg: MemArg },
    I32Store8 { mem_arg: MemArg },
    I32Store16 { mem_arg: MemArg },
    I64Store8 { mem_arg: MemArg },
    I64Store16 { mem_arg: MemArg },
    I64Store32 { mem_arg: MemArg },
    MemorySize { mem_idx: u8 },
    MemoryGrow { mem_idx: u8 },
    MemoryInit { data_idx: u32, mem_idx: u8 },
    DataDrop { data_idx: u32 },
    MemoryCopy { src_mem: u8, dst_mem: u8 },
    MemoryFill { mem_idx: u8 },

    // Numeric instructions
    I32Const { value: i32 },
    I64Const { value: i64 },
    F32Const { value: f32 },
    F64Const { value: f64 },

    // Comparison instructions
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    // Bitwise instructions
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,

    // Float instructions
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    // Conversion instructions
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    // Sign extension
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    // Reference instructions
    RefNull { ty: ValueType },
    RefIsNull,
    RefFunc { func_idx: u32 },

    // Multi-value extension
    SelectTyped { results: Vec<ValueType> },
}

impl Instruction {
    /// Create a new block instruction
    pub fn block(block_type: BlockType, body: Vec<Instruction>) -> Self {
        Instruction::Block { block_type, body }
    }

    /// Create a new loop instruction
    pub fn loop_(block_type: BlockType, body: Vec<Instruction>) -> Self {
        Instruction::Loop { block_type, body }
    }

    /// Create a new if instruction
    pub fn if_(block_type: BlockType, then_branch: Vec<Instruction>, else_branch: Vec<Instruction>) -> Self {
        Instruction::If { block_type, then_branch, else_branch }
    }

    /// Check if this instruction is a terminal instruction (end, else, etc.)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Instruction::End | Instruction::Else)
    }

    /// Check if this instruction is a control flow instruction that can contain nested instructions
    pub fn is_nested_container(&self) -> bool {
        matches!(
            self,
            Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. }
        )
    }
}

/// Helper to build instruction sequences
#[derive(Debug, Default)]
pub struct InstructionBuilder {
    instructions: Vec<Instruction>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, instr: Instruction) -> Self {
        self.instructions.push(instr);
        self
    }

    pub fn i32_const(mut self, value: i32) -> Self {
        self.instructions.push(Instruction::I32Const { value });
        self
    }

    pub fn i64_const(mut self, value: i64) -> Self {
        self.instructions.push(Instruction::I64Const { value });
        self
    }

    pub fn local_get(mut self, local_idx: u32) -> Self {
        self.instructions.push(Instruction::LocalGet { local_idx });
        self
    }

    pub fn local_set(mut self, local_idx: u32) -> Self {
        self.instructions.push(Instruction::LocalSet { local_idx });
        self
    }

    pub fn call(mut self, function_idx: u32) -> Self {
        self.instructions.push(Instruction::Call { function_idx });
        self
    }

    pub fn build(self) -> Vec<Instruction> {
        self.instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_builder() {
        let instructions = InstructionBuilder::new()
            .local_get(0)
            .i32_const(42)
            .push(Instruction::I32Add)
            .local_set(1)
            .build();

        assert_eq!(instructions.len(), 4);
        assert!(matches!(instructions[0], Instruction::LocalGet { local_idx: 0 }));
        assert!(matches!(instructions[1], Instruction::I32Const { value: 42 }));
        assert!(matches!(instructions[2], Instruction::I32Add));
        assert!(matches!(instructions[3], Instruction::LocalSet { local_idx: 1 }));
    }

    #[test]
    fn test_nested_container_detection() {
        assert!(!Instruction::I32Add.is_nested_container());
        assert!(!Instruction::Nop.is_nested_container());
        assert!(Instruction::Block { 
            block_type: BlockType::Empty, 
            body: vec![] 
        }.is_nested_container());
        assert!(Instruction::Loop { 
            block_type: BlockType::Empty, 
            body: vec![] 
        }.is_nested_container());
    }
}
