//! WebAssembly type definitions

use crate::error::{Result, WasmError};

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValueType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    V128 = 0x7b,
    FuncRef = 0x70,
    ExternRef = 0x6f,
}

impl ValueType {
    /// Convert from byte to ValueType
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0x7f => Ok(ValueType::I32),
            0x7e => Ok(ValueType::I64),
            0x7d => Ok(ValueType::F32),
            0x7c => Ok(ValueType::F64),
            0x7b => Ok(ValueType::V128),
            0x70 => Ok(ValueType::FuncRef),
            0x6f => Ok(ValueType::ExternRef),
            _ => Err(WasmError::InvalidValueType(byte)),
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// WebAssembly block type (used in control flow instructions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// Empty block (void)
    Empty,
    /// Single value type
    Value(ValueType),
    /// Type index (for multi-value blocks)
    TypeIndex(i64),
}

impl BlockType {
    /// Convert from signed LEB128 value
    pub fn from_i64(value: i64) -> Result<Self> {
        match value {
            -64 => Ok(BlockType::Empty), // 0x40 in signed LEB128
            -1 => Ok(BlockType::Value(ValueType::I32)),
            -2 => Ok(BlockType::Value(ValueType::I64)),
            -3 => Ok(BlockType::Value(ValueType::F32)),
            -4 => Ok(BlockType::Value(ValueType::F64)),
            -5 => Ok(BlockType::Value(ValueType::V128)),
            -16 => Ok(BlockType::Value(ValueType::FuncRef)),
            -17 => Ok(BlockType::Value(ValueType::ExternRef)),
            idx if idx >= 0 => Ok(BlockType::TypeIndex(idx)),
            _ => Err(WasmError::InvalidBlockType(value)),
        }
    }

    /// Convert to i64 for encoding
    pub fn to_i64(self) -> i64 {
        match self {
            BlockType::Empty => -64, // 0x40
            BlockType::Value(vt) => match vt {
                ValueType::I32 => -1,
                ValueType::I64 => -2,
                ValueType::F32 => -3,
                ValueType::F64 => -4,
                ValueType::V128 => -5,
                ValueType::FuncRef => -16,
                ValueType::ExternRef => -17,
            },
            BlockType::TypeIndex(idx) => idx,
        }
    }
}

/// Function type (parameters and results)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl FuncType {
    pub fn new(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
        Self { params, results }
    }
}

/// Memory type with limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

/// Table type with element type and limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableType {
    pub limits: Limits,
    pub elem_type: ValueType,
}

/// Global type with value type and mutability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutable: bool,
}

/// Limits (min and optional max)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
    /// Memory/table has a 64-bit index (memory64 proposal)
    pub memory64: bool,
    /// Maximum is specified as pages (for memory)
    pub shared: bool,
}

impl Limits {
    pub fn new(min: u32, max: Option<u32>) -> Self {
        Self {
            min,
            max,
            memory64: false,
            shared: false,
        }
    }

    pub fn new64(min: u32, max: Option<u32>) -> Self {
        Self {
            min,
            max,
            memory64: true,
            shared: false,
        }
    }

    /// Validate that max >= min if max is present
    pub fn validate(&self) -> Result<()> {
        if let Some(max) = self.max {
            if max < self.min {
                return Err(WasmError::InvalidLimits {
                    min: self.min,
                    max,
                });
            }
        }
        Ok(())
    }
}

/// External kind (for imports and exports)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExternalKind {
    Func = 0x00,
    Table = 0x01,
    Mem = 0x02,
    Global = 0x03,
}

impl ExternalKind {
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0x00 => Ok(ExternalKind::Func),
            0x01 => Ok(ExternalKind::Table),
            0x02 => Ok(ExternalKind::Mem),
            0x03 => Ok(ExternalKind::Global),
            _ => Err(WasmError::InvalidKind(byte)),
        }
    }
}

/// Import description (what kind of import)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportDesc {
    /// Function with type index
    Func(u32),
    /// Table
    Table(TableType),
    /// Memory
    Mem(MemType),
    /// Global
    Global(GlobalType),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_from_byte() {
        assert_eq!(ValueType::from_byte(0x7f).unwrap(), ValueType::I32);
        assert_eq!(ValueType::from_byte(0x7e).unwrap(), ValueType::I64);
        assert_eq!(ValueType::from_byte(0x7d).unwrap(), ValueType::F32);
        assert_eq!(ValueType::from_byte(0x7c).unwrap(), ValueType::F64);
        assert_eq!(ValueType::from_byte(0x7b).unwrap(), ValueType::V128);
        assert_eq!(ValueType::from_byte(0x70).unwrap(), ValueType::FuncRef);
        assert_eq!(ValueType::from_byte(0x6f).unwrap(), ValueType::ExternRef);
        assert!(ValueType::from_byte(0x00).is_err());
    }

    #[test]
    fn test_block_type_roundtrip() {
        let types = vec![
            BlockType::Empty,
            BlockType::Value(ValueType::I32),
            BlockType::Value(ValueType::I64),
            BlockType::Value(ValueType::F32),
            BlockType::Value(ValueType::F64),
            BlockType::TypeIndex(0),
            BlockType::TypeIndex(5),
        ];

        for bt in &types {
            let encoded = bt.to_i64();
            let decoded = BlockType::from_i64(encoded).unwrap();
            assert_eq!(*bt, decoded);
        }
    }

    #[test]
    fn test_limits_validation() {
        let valid = Limits::new(0, Some(10));
        assert!(valid.validate().is_ok());

        let valid2 = Limits::new(10, None);
        assert!(valid2.validate().is_ok());

        let invalid = Limits::new(10, Some(5));
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_external_kind() {
        assert_eq!(ExternalKind::from_byte(0x00).unwrap(), ExternalKind::Func);
        assert_eq!(ExternalKind::from_byte(0x01).unwrap(), ExternalKind::Table);
        assert_eq!(ExternalKind::from_byte(0x02).unwrap(), ExternalKind::Mem);
        assert_eq!(ExternalKind::from_byte(0x03).unwrap(), ExternalKind::Global);
        assert!(ExternalKind::from_byte(0x04).is_err());
    }
}
