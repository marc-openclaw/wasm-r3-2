//! WebAssembly Binary Parser
//!
//! A parser for WebAssembly binary format that provides editable AST structures.
//! Supports parsing, modifying, and serializing WASM modules.

pub mod ast;
pub mod decode;
pub mod encode;
pub mod error;
pub mod instruction;
pub mod leb128;
pub mod parser;
pub mod types;
pub mod value;

#[cfg(feature = "wasm")]
pub mod wasm_bindings;

#[cfg(feature = "wasm")]
pub use wasm_bindings::WasmModule;

pub use ast::*;
pub use decode::*;
pub use encode::*;
pub use error::*;
pub use instruction::*;

pub use types::*;
pub use value::*;

/// Magic number for WebAssembly binary format: \0asm
pub const WASM_MAGIC: &[u8] = &[0x00, 0x61, 0x73, 0x6d];

/// Current WebAssembly version (1)
pub const WASM_VERSION: &[u8] = &[0x01, 0x00, 0x00, 0x00];

/// WebAssembly section IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SectionId {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    DataCount = 12,
}

impl TryFrom<u8> for SectionId {
    type Error = WasmError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(SectionId::Custom),
            1 => Ok(SectionId::Type),
            2 => Ok(SectionId::Import),
            3 => Ok(SectionId::Function),
            4 => Ok(SectionId::Table),
            5 => Ok(SectionId::Memory),
            6 => Ok(SectionId::Global),
            7 => Ok(SectionId::Export),
            8 => Ok(SectionId::Start),
            9 => Ok(SectionId::Element),
            10 => Ok(SectionId::Code),
            11 => Ok(SectionId::Data),
            12 => Ok(SectionId::DataCount),
            _ => Err(WasmError::InvalidSectionId(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_and_version() {
        assert_eq!(WASM_MAGIC, &[0x00, 0x61, 0x73, 0x6d]);
        assert_eq!(WASM_VERSION, &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_section_id_conversion() {
        assert_eq!(SectionId::try_from(0u8).unwrap(), SectionId::Custom);
        assert_eq!(SectionId::try_from(1u8).unwrap(), SectionId::Type);
        assert_eq!(SectionId::try_from(10u8).unwrap(), SectionId::Code);
        assert!(SectionId::try_from(13u8).is_err());
    }
}
