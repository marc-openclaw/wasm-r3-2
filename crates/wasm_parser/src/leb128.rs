//! LEB128 encoding and decoding (Little Endian Base 128)
//!
//! WebAssembly uses LEB128 for encoding integers in a space-efficient way.

use crate::error::{Result, WasmError};

/// Maximum number of bytes for a 32-bit LEB128 value
const MAX_LEB128_32: usize = 5;
/// Maximum number of bytes for a 64-bit LEB128 value
const MAX_LEB128_64: usize = 10;

/// Decode an unsigned 32-bit LEB128 value
pub fn decode_u32(bytes: &[u8]) -> Result<(u32, usize)> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    let mut i = 0;

    loop {
        if i >= bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        if i >= MAX_LEB128_32 {
            return Err(WasmError::Leb128DecodeError(
                "u32 exceeds maximum byte length".to_string(),
            ));
        }

        let byte = bytes[i];
        i += 1;

        let value = (byte & 0x7f) as u32;
        if shift == 28 && value >> 4 != 0 {
            return Err(WasmError::Leb128DecodeError(
                "u32 overflow in LEB128".to_string(),
            ));
        }

        result |= value << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    Ok((result, i))
}

/// Decode an unsigned 64-bit LEB128 value
pub fn decode_u64(bytes: &[u8]) -> Result<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut i = 0;

    loop {
        if i >= bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        if i >= MAX_LEB128_64 {
            return Err(WasmError::Leb128DecodeError(
                "u64 exceeds maximum byte length".to_string(),
            ));
        }

        let byte = bytes[i];
        i += 1;

        let value = (byte & 0x7f) as u64;
        result |= value << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    Ok((result, i))
}

/// Decode a signed 32-bit LEB128 value
pub fn decode_i32(bytes: &[u8]) -> Result<(i32, usize)> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    let mut i = 0;
    let mut byte: u8;

    loop {
        if i >= bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        if i >= MAX_LEB128_32 {
            return Err(WasmError::Leb128DecodeError(
                "i32 exceeds maximum byte length".to_string(),
            ));
        }

        byte = bytes[i];
        i += 1;

        let value = (byte & 0x7f) as i32;
        result |= value << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    // Sign extend if negative
    if shift < 32 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }

    Ok((result, i))
}

/// Decode a signed 64-bit LEB128 value
pub fn decode_i64(bytes: &[u8]) -> Result<(i64, usize)> {
    let mut result: i64 = 0;
    let mut shift: u32 = 0;
    let mut i = 0;
    let mut byte: u8;

    loop {
        if i >= bytes.len() {
            return Err(WasmError::UnexpectedEof);
        }
        if i >= MAX_LEB128_64 {
            return Err(WasmError::Leb128DecodeError(
                "i64 exceeds maximum byte length".to_string(),
            ));
        }

        byte = bytes[i];
        i += 1;

        let value = (byte & 0x7f) as i64;
        result |= value << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    // Sign extend if negative
    if shift < 64 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }

    Ok((result, i))
}

/// Encode an unsigned 32-bit value as LEB128
pub fn encode_u32(value: u32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;

    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }

    result
}

/// Encode an unsigned 64-bit value as LEB128
pub fn encode_u64(value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;

    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }

    result
}

/// Encode a signed 32-bit value as LEB128
pub fn encode_i32(value: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;

    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;

        // Check if we need more bytes
        let done = value == 0 && (byte & 0x40) == 0 ||
                   value == -1 && (byte & 0x40) != 0;

        if !done {
            byte |= 0x80;
        }

        result.push(byte);

        if done {
            break;
        }
    }

    result
}

/// Encode a signed 64-bit value as LEB128
pub fn encode_i64(value: i64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;

    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;

        // Check if we need more bytes
        let done = value == 0 && (byte & 0x40) == 0 ||
                   value == -1 && (byte & 0x40) != 0;

        if !done {
            byte |= 0x80;
        }

        result.push(byte);

        if done {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_roundtrip() {
        let values = [0u32, 1, 127, 128, 255, 256, 16383, 16384, 65535, 65536, u32::MAX];
        for &value in &values {
            let encoded = encode_u32(value);
            let (decoded, size) = decode_u32(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(size, encoded.len());
        }
    }

    #[test]
    fn test_i32_roundtrip() {
        let values = [0i32, 1, -1, 127, -127, 128, -128, 255, -255, 256, -256, i32::MAX, i32::MIN];
        for &value in &values {
            let encoded = encode_i32(value);
            let (decoded, size) = decode_i32(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(size, encoded.len());
        }
    }

    #[test]
    fn test_u64_roundtrip() {
        let values = [0u64, 1, 127, 128, u64::MAX];
        for &value in &values {
            let encoded = encode_u64(value);
            let (decoded, size) = decode_u64(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(size, encoded.len());
        }
    }

    #[test]
    fn test_i64_roundtrip() {
        let values = [0i64, 1, -1, 127, -127, i64::MAX, i64::MIN];
        for &value in &values {
            let encoded = encode_i64(value);
            let (decoded, size) = decode_i64(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(size, encoded.len());
        }
    }

    #[test]
    fn test_known_u32_values() {
        // Test known LEB128 encodings
        assert_eq!(encode_u32(624485), vec![0xe5, 0x8e, 0x26]);
        assert_eq!(encode_u32(127), vec![0x7f]);
        assert_eq!(encode_u32(128), vec![0x80, 0x01]);
        assert_eq!(encode_u32(129), vec![0x81, 0x01]);
    }

    #[test]
    fn test_known_i32_values() {
        // Test known LEB128 encodings for signed values
        assert_eq!(encode_i32(-123456), vec![0xc0, 0xbb, 0x78]);
        assert_eq!(encode_i32(127), vec![0x7f]);
        assert_eq!(encode_i32(-1), vec![0x7f]);
        assert_eq!(encode_i32(-128), vec![0x80, 0x7f]);
    }
}
