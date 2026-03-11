//! Error types for WASM parsing

use thiserror::Error;

/// Result type alias for WASM operations
pub type Result<T> = std::result::Result<T, WasmError>;

/// Errors that can occur during WASM parsing, validation, or encoding
#[derive(Error, Debug, Clone, PartialEq)]
pub enum WasmError {
    #[error("Invalid magic number: expected \\0asm, got {0:?}")]
    InvalidMagic(Vec<u8>),

    #[error("Invalid version: expected 1, got {0:?}")]
    InvalidVersion(Vec<u8>),

    #[error("Invalid section ID: {0}")]
    InvalidSectionId(u8),

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("LEB128 decode error: {0}")]
    Leb128DecodeError(String),

    #[error("LEB128 encode error: {0}")]
    Leb128EncodeError(String),

    #[error("Invalid value type: {0}")]
    InvalidValueType(u8),

    #[error("Invalid block type: {0}")]
    InvalidBlockType(i64),

    #[error("Invalid opcode: {0:#04x}")]
    InvalidOpcode(u8),

    #[error("Invalid function index: {0}")]
    InvalidFunctionIndex(u32),

    #[error("Invalid type index: {0}")]
    InvalidTypeIndex(u32),

    #[error("Invalid memory alignment: {0}")]
    InvalidAlignment(u32),

    #[error("Invalid limits: max ({max}) < min ({min})")]
    InvalidLimits { min: u32, max: u32 },

    #[error("Invalid import/export kind: {0}")]
    InvalidKind(u8),

    #[error("Invalid mutability: {0}")]
    InvalidMutability(u8),

    #[error("String decode error: {0}")]
    StringDecodeError(String),

    #[error("UTF-8 validation failed")]
    Utf8Error,

    #[error("Invalid element segment kind: {0}")]
    InvalidElementKind(u8),

    #[error("Invalid data segment mode: {0}")]
    InvalidDataMode(u8),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Custom error: {0}")]
    Custom(String),
}

impl From<std::io::Error> for WasmError {
    fn from(e: std::io::Error) -> Self {
        WasmError::IoError(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for WasmError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        WasmError::Utf8Error
    }
}
