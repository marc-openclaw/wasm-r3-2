//! WebAssembly constant values

use crate::types::ValueType;

/// WebAssembly constant value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    V128([u8; 16]),
    FuncRef(Option<u32>),
    ExternRef(Option<u32>),
}

impl Value {
    /// Get the value type
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
            Value::V128(_) => ValueType::V128,
            Value::FuncRef(_) => ValueType::FuncRef,
            Value::ExternRef(_) => ValueType::ExternRef,
        }
    }

    /// Convert to i32
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Value::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to i64
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Value::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to f32
    pub fn to_f32(&self) -> Option<f32> {
        match self {
            Value::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to f64
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Value::F64(v) => Some(*v),
            _ => None,
        }
    }
}

/// Helper to encode/decode little-endian values
pub mod encoding {
    /// Encode u32 as little-endian 4 bytes
    pub fn encode_u32_le(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }

    /// Encode i32 as little-endian 4 bytes
    pub fn encode_i32_le(value: i32) -> [u8; 4] {
        value.to_le_bytes()
    }

    /// Encode u64 as little-endian 8 bytes
    pub fn encode_u64_le(value: u64) -> [u8; 8] {
        value.to_le_bytes()
    }

    /// Encode i64 as little-endian 8 bytes
    pub fn encode_i64_le(value: i64) -> [u8; 8] {
        value.to_le_bytes()
    }

    /// Encode f32 as little-endian 4 bytes
    pub fn encode_f32_le(value: f32) -> [u8; 4] {
        value.to_le_bytes()
    }

    /// Encode f64 as little-endian 8 bytes
    pub fn encode_f64_le(value: f64) -> [u8; 8] {
        value.to_le_bytes()
    }

    /// Decode u32 from little-endian 4 bytes
    pub fn decode_u32_le(bytes: &[u8]) -> Option<u32> {
        if bytes.len() >= 4 {
            Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            None
        }
    }

    /// Decode i32 from little-endian 4 bytes
    pub fn decode_i32_le(bytes: &[u8]) -> Option<i32> {
        if bytes.len() >= 4 {
            Some(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            None
        }
    }

    /// Decode f32 from little-endian 4 bytes
    pub fn decode_f32_le(bytes: &[u8]) -> Option<f32> {
        if bytes.len() >= 4 {
            Some(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            None
        }
    }

    /// Decode f64 from little-endian 8 bytes
    pub fn decode_f64_le(bytes: &[u8]) -> Option<f64> {
        if bytes.len() >= 8 {
            Some(f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type() {
        assert_eq!(Value::I32(42).value_type(), ValueType::I32);
        assert_eq!(Value::I64(42).value_type(), ValueType::I64);
        assert_eq!(Value::F32(3.14).value_type(), ValueType::F32);
        assert_eq!(Value::F64(3.14).value_type(), ValueType::F64);
    }

    #[test]
    fn test_f32_roundtrip() {
        let values = [0.0f32, -0.0, 1.0, -1.0, f32::MAX, f32::MIN, f32::INFINITY, f32::NAN];
        for &value in &values {
            let encoded = encoding::encode_f32_le(value);
            let decoded = encoding::decode_f32_le(&encoded).unwrap();
            // NaN != NaN, so handle that case
            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn test_f64_roundtrip() {
        let values = [0.0f64, -0.0, 1.0, -1.0, f64::MAX, f64::MIN, f64::INFINITY, f64::NAN];
        for &value in &values {
            let encoded = encoding::encode_f64_le(value);
            let decoded = encoding::decode_f64_le(&encoded).unwrap();
            // NaN != NaN, so handle that case
            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn test_i32_roundtrip() {
        let values = [0i32, 1, -1, i32::MAX, i32::MIN, 42, -42];
        for &value in &values {
            let encoded = encoding::encode_i32_le(value);
            let decoded = encoding::decode_i32_le(&encoded).unwrap();
            assert_eq!(decoded, value);
        }
    }
}
