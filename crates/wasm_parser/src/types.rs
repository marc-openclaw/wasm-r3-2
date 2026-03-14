//! WebAssembly type definitions

use crate::error::{Result, WasmError};

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum ValueType {
    /// 32-bit integer
    I32 = 0x7f,
    /// 64-bit integer
    I64 = 0x7e,
    /// 32-bit float
    F32 = 0x7d,
    /// 64-bit float (double)
    F64 = 0x7c,
    /// 128-bit vector (SIMD)
    V128 = 0x7b,
    /// Function reference
    FuncRef = 0x70,
    /// External reference
    ExternRef = 0x6f,
}

impl ValueType {
    /// Convert from byte to ValueType
    /// Handles core types, reference types, and GC proposal types
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            // Core value types
            0x7f => Ok(ValueType::I32),
            0x7e => Ok(ValueType::I64),
            0x7d => Ok(ValueType::F32),
            0x7c => Ok(ValueType::F64),
            0x7b => Ok(ValueType::V128),
            // Reference types
            0x70 => Ok(ValueType::FuncRef),
            0x6f => Ok(ValueType::ExternRef),
            // GC proposal packed types - treat as I32 for compatibility
            0x01 => Ok(ValueType::I32), // i8 packed type
            0x02 => Ok(ValueType::I32), // i16 packed type
            // GC proposal reference types - treat as ExternRef for compatibility
            0x03..=0x05 => Ok(ValueType::ExternRef), // anyref, eqref, structref, etc.
            0x06..=0x10 => Ok(ValueType::ExternRef), // more GC ref types
            // GC proposal type indices (0x11-0x3f) - treat as ExternRef
            0x11..=0x3f => Ok(ValueType::ExternRef),
            // Reserved range 0x40-0x6e - treat as ExternRef
            0x40..=0x6e => Ok(ValueType::ExternRef),
            // 0x71-0x7a are reserved - treat as ExternRef
            0x71..=0x7a => Ok(ValueType::ExternRef),
            // Special case: 0x00 is sometimes used as a sentinel/empty type
            // in some WASM encodings - treat as I32 for compatibility
            0x00 => Ok(ValueType::I32),
            // Handle high values that might appear in malformed WASM
            // These are likely from multi-byte encodings - treat as ExternRef
            0x80..=0xff => Ok(ValueType::ExternRef),
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
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
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
    /// Handles core block types, reference types, and GC proposal types
    pub fn from_i64(value: i64) -> Result<Self> {
        match value {
            // Empty block type (0x40)
            -64 => Ok(BlockType::Empty),
            // Core value types
            -1 => Ok(BlockType::Value(ValueType::I32)),  // 0x7f
            -2 => Ok(BlockType::Value(ValueType::I64)),  // 0x7e
            -3 => Ok(BlockType::Value(ValueType::F32)),  // 0x7d
            -4 => Ok(BlockType::Value(ValueType::F64)),  // 0x7c
            -5 => Ok(BlockType::Value(ValueType::V128)),  // 0x7b
            // Reference types
            -16 => Ok(BlockType::Value(ValueType::FuncRef)),  // 0x70
            -17 => Ok(BlockType::Value(ValueType::ExternRef)), // 0x6f
            // GC proposal packed types (0x41-0x42) - treat as I32
            -63 | -62 => Ok(BlockType::Value(ValueType::I32)), // 0x41 (i8), 0x42 (i16)
            // GC proposal reference types (0x43-0x4f) - treat as ExternRef
            -61..=-49 => Ok(BlockType::Value(ValueType::ExternRef)), // 0x43-0x4f
            // Reserved range (0x50-0x5f) - treat as Empty for compatibility
            -48..=-33 => Ok(BlockType::Empty),
            // Reserved range (0x60-0x6e) - treat as Empty for compatibility
            -32..=-18 => Ok(BlockType::Empty),
            // Additional block type values that may appear in the wild
            // These are from various proposals and should be treated gracefully
            -15 => Ok(BlockType::Empty), // 0x71
            -12 => Ok(BlockType::Empty), // 0x74
            -8 => Ok(BlockType::Empty),  // 0x78
            -7 => Ok(BlockType::Empty),  // 0x79
            -6 => Ok(BlockType::Empty),  // 0x7a
            // Type index (positive values)
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
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct FuncType {
    /// Parameter types
    pub params: Vec<ValueType>,
    /// Result types
    pub results: Vec<ValueType>,
}

impl FuncType {
    /// Create a new function type
    pub fn new(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
        Self { params, results }
    }
}

/// Memory type with limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct MemType {
    /// Memory limits
    pub limits: Limits,
}

/// Table type with element type and limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct TableType {
    /// Table limits
    pub limits: Limits,
    /// Element type (usually funcref)
    pub elem_type: ValueType,
}

/// Global type with value type and mutability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct GlobalType {
    /// Value type of the global
    pub value_type: ValueType,
    /// Whether the global is mutable
    pub mutable: bool,
}

/// Limits (min and optional max)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Limits {
    /// Minimum size
    pub min: u32,
    /// Maximum size (optional)
    pub max: Option<u32>,
    /// Memory/table has a 64-bit index (memory64 proposal)
    pub memory64: bool,
    /// Maximum is specified as pages (for memory)
    pub shared: bool,
}

impl Limits {
    /// Create new limits with optional maximum
    pub fn new(min: u32, max: Option<u32>) -> Self {
        Self {
            min,
            max,
            memory64: false,
            shared: false,
        }
    }

    /// Create new limits for memory64
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
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum ExternalKind {
    /// Function
    Func = 0x00,
    /// Table
    Table = 0x01,
    /// Memory
    Mem = 0x02,
    /// Global
    Global = 0x03,
    /// Unknown/custom kind (for extended WASM proposals)
    Unknown(u8),
}

impl ExternalKind {
    /// Convert from byte
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0x00 => Ok(ExternalKind::Func),
            0x01 => Ok(ExternalKind::Table),
            0x02 => Ok(ExternalKind::Mem),
            0x03 => Ok(ExternalKind::Global),
            // Return error for unknown kinds
            _ => Err(WasmError::InvalidExternalKind(byte)),
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        match self {
            ExternalKind::Func => 0x00,
            ExternalKind::Table => 0x01,
            ExternalKind::Mem => 0x02,
            ExternalKind::Global => 0x03,
            ExternalKind::Unknown(byte) => byte,
        }
    }
}

/// Import description (what kind of import)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
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
