//! Data types and value representations for Omron PLC memory.
//!
//! This module provides tools to convert between Rust types and the
//! memory formats used by Omron PLCs (Big-Endian with Word Swap).

use crate::error::{FinsError, Result};

/// Data types supported by Omron PLCs in memory areas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// 8-bit unsigned integer (occupies 1 byte in memory, usually aligned to 2 bytes).
    USINT,
    /// 16-bit unsigned integer (1 word).
    UINT,
    /// 32-bit unsigned integer (2 words).
    UDINT,
    /// 64-bit unsigned integer (4 words).
    ULINT,
    /// 8-bit signed integer.
    SINT,
    /// 16-bit signed integer.
    INT,
    /// 32-bit signed integer.
    DINT,
    /// 64-bit signed integer.
    LINT,
    /// 32-bit floating point (REAL).
    REAL,
    /// 64-bit floating point (LREAL).
    LREAL,
    /// Word (16-bit bit string).
    WORD,
    /// Double Word (32-bit bit string).
    DWORD,
    /// Long Word (64-bit bit string).
    LWORD,
}

impl DataType {
    /// Returns the size in bytes for this data type.
    pub fn size(&self) -> usize {
        match self {
            DataType::USINT | DataType::SINT => 1,
            DataType::UINT | DataType::INT | DataType::WORD => 2,
            DataType::UDINT | DataType::DINT | DataType::DWORD | DataType::REAL => 4,
            DataType::ULINT | DataType::LINT | DataType::LWORD | DataType::LREAL => 8,
        }
    }
}

/// A value that can be read from or written to the PLC memory.
#[derive(Debug, Clone, PartialEq)]
pub enum PlcValue {
    /// 8-bit unsigned integer.
    USint(u8),
    /// 16-bit unsigned integer.
    Uint(u16),
    /// 32-bit unsigned integer.
    Udint(u32),
    /// 64-bit unsigned integer.
    Ulint(u64),
    /// 8-bit signed integer.
    Sint(i8),
    /// 16-bit signed integer.
    Int(i16),
    /// 32-bit signed integer.
    Dint(i32),
    /// 64-bit signed integer.
    Lint(i64),
    /// 32-bit floating point.
    Real(f32),
    /// 64-bit floating point.
    Lreal(f64),
    /// 16-bit word.
    Word(u16),
    /// 32-bit double word.
    Dword(u32),
    /// 64-bit long word.
    Lword(u64),
}

impl PlcValue {
    /// Returns the data type of this value.
    pub fn data_type(&self) -> DataType {
        match self {
            PlcValue::USint(_) => DataType::USINT,
            PlcValue::Uint(_) => DataType::UINT,
            PlcValue::Udint(_) => DataType::UDINT,
            PlcValue::Ulint(_) => DataType::ULINT,
            PlcValue::Sint(_) => DataType::SINT,
            PlcValue::Int(_) => DataType::INT,
            PlcValue::Dint(_) => DataType::DINT,
            PlcValue::Lint(_) => DataType::LINT,
            PlcValue::Real(_) => DataType::REAL,
            PlcValue::Lreal(_) => DataType::LREAL,
            PlcValue::Word(_) => DataType::WORD,
            PlcValue::Dword(_) => DataType::DWORD,
            PlcValue::Lword(_) => DataType::LWORD,
        }
    }

    /// Converts the value into bytes suitable for PLC memory.
    pub fn to_plc_bytes(&self) -> Vec<u8> {
        match self {
            PlcValue::USint(v) => vec![0, *v],
            PlcValue::Sint(v) => vec![0, *v as u8],
            PlcValue::Uint(v) => v.to_be_bytes().to_vec(),
            PlcValue::Int(v) => v.to_be_bytes().to_vec(),
            PlcValue::Word(v) => v.to_be_bytes().to_vec(),
            PlcValue::Udint(v) => swap_words_32(&v.to_be_bytes()),
            PlcValue::Dint(v) => swap_words_32(&v.to_be_bytes()),
            PlcValue::Dword(v) => swap_words_32(&v.to_be_bytes()),
            PlcValue::Real(v) => swap_words_32(&v.to_be_bytes()),
            PlcValue::Ulint(v) => reverse_words_64(&v.to_be_bytes()),
            PlcValue::Lint(v) => reverse_words_64(&v.to_be_bytes()),
            PlcValue::Lword(v) => reverse_words_64(&v.to_be_bytes()),
            PlcValue::Lreal(v) => reverse_words_64(&v.to_be_bytes()),
        }
    }

    /// Parses a value from bytes received from the PLC.
    pub fn from_plc_bytes(data_type: DataType, bytes: &[u8]) -> Result<Self> {
        if bytes.len() < data_type.size() {
            return Err(FinsError::invalid_response("Insufficient bytes for data type"));
        }

        match data_type {
            DataType::USINT => Ok(PlcValue::USint(bytes[bytes.len() - 1])),
            DataType::SINT => Ok(PlcValue::Sint(bytes[bytes.len() - 1] as i8)),
            DataType::UINT => Ok(PlcValue::Uint(u16::from_be_bytes([bytes[0], bytes[1]]))),
            DataType::INT => Ok(PlcValue::Int(i16::from_be_bytes([bytes[0], bytes[1]]))),
            DataType::WORD => Ok(PlcValue::Word(u16::from_be_bytes([bytes[0], bytes[1]]))),
            DataType::UDINT => {
                let swapped = swap_words_32(bytes);
                Ok(PlcValue::Udint(u32::from_be_bytes(swapped.try_into().unwrap())))
            }
            DataType::DINT => {
                let swapped = swap_words_32(bytes);
                Ok(PlcValue::Dint(i32::from_be_bytes(swapped.try_into().unwrap())))
            }
            DataType::DWORD => {
                let swapped = swap_words_32(bytes);
                Ok(PlcValue::Dword(u32::from_be_bytes(swapped.try_into().unwrap())))
            }
            DataType::REAL => {
                let swapped = swap_words_32(bytes);
                Ok(PlcValue::Real(f32::from_be_bytes(swapped.try_into().unwrap())))
            }
            DataType::ULINT => {
                let reversed = reverse_words_64(bytes);
                Ok(PlcValue::Ulint(u64::from_be_bytes(reversed.try_into().unwrap())))
            }
            DataType::LINT => {
                let reversed = reverse_words_64(bytes);
                Ok(PlcValue::Lint(i64::from_be_bytes(reversed.try_into().unwrap())))
            }
            DataType::LWORD => {
                let reversed = reverse_words_64(bytes);
                Ok(PlcValue::Lword(u64::from_be_bytes(reversed.try_into().unwrap())))
            }
            DataType::LREAL => {
                let reversed = reverse_words_64(bytes);
                Ok(PlcValue::Lreal(f64::from_be_bytes(reversed.try_into().unwrap())))
            }
        }
    }
}

fn swap_words_32(bytes: &[u8]) -> Vec<u8> {
    vec![bytes[2], bytes[3], bytes[0], bytes[1]]
}

fn reverse_words_64(bytes: &[u8]) -> Vec<u8> {
    vec![
        bytes[6], bytes[7], 
        bytes[4], bytes[5], 
        bytes[2], bytes[3], 
        bytes[0], bytes[1]
    ]
}
