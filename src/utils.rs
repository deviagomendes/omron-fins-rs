//! Utility functions for bit manipulation and data conversion.
//!
//! This module provides helper functions for working with PLC data,
//! including bit extraction, conversion, and formatting utilities.
//!
//! # Example
//!
//! ```
//! use omron_fins::utils::{get_bit, get_bits, word_to_bits};
//!
//! let value: u16 = 0b1010_0101_1100_0011;
//!
//! // Get individual bit
//! assert!(get_bit(value, 0));   // bit 0 is ON
//! assert!(!get_bit(value, 2));  // bit 2 is OFF
//!
//! // Get all bits as array
//! let bits = word_to_bits(value);
//! assert_eq!(bits[0], true);
//! assert_eq!(bits[1], true);
//!
//! // Get bits as Vec<BitInfo>
//! let indexed_bits = get_bits(value);
//! for bit_info in indexed_bits {
//!     println!("Bit {}: {}", bit_info.index, bit_info.value);
//! }
//! ```

/// Represents a single bit with its index and value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitInfo {
    /// Bit position (0-15 for u16).
    pub index: u8,
    /// Bit value (true = ON, false = OFF).
    pub value: bool,
}

impl BitInfo {
    /// Creates a new BitInfo.
    pub fn new(index: u8, value: bool) -> Self {
        Self { index, value }
    }
}

impl std::fmt::Display for BitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bit {}: {}",
            self.index,
            if self.value { "ON" } else { "OFF" }
        )
    }
}

/// Gets a single bit from a 16-bit word.
///
/// # Arguments
///
/// * `value` - The 16-bit word to extract from
/// * `bit` - Bit position (0-15, where 0 is LSB)
///
/// # Returns
///
/// `true` if the bit is set, `false` otherwise.
///
/// # Example
///
/// ```
/// use omron_fins::utils::get_bit;
///
/// let value: u16 = 0b0000_0000_0000_0101; // bits 0 and 2 are set
/// assert!(get_bit(value, 0));
/// assert!(!get_bit(value, 1));
/// assert!(get_bit(value, 2));
/// ```
#[inline]
pub fn get_bit(value: u16, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}

/// Sets a single bit in a 16-bit word.
///
/// # Arguments
///
/// * `value` - The original 16-bit word
/// * `bit` - Bit position (0-15, where 0 is LSB)
/// * `state` - Value to set (true = ON, false = OFF)
///
/// # Returns
///
/// The modified word with the bit set or cleared.
///
/// # Example
///
/// ```
/// use omron_fins::utils::set_bit;
///
/// let value: u16 = 0;
/// let result = set_bit(value, 5, true);
/// assert_eq!(result, 0b0000_0000_0010_0000);
/// ```
#[inline]
pub fn set_bit(value: u16, bit: u8, state: bool) -> u16 {
    if state {
        value | (1 << bit)
    } else {
        value & !(1 << bit)
    }
}

/// Toggles a single bit in a 16-bit word.
///
/// # Arguments
///
/// * `value` - The original 16-bit word
/// * `bit` - Bit position (0-15, where 0 is LSB)
///
/// # Returns
///
/// The modified word with the bit toggled.
///
/// # Example
///
/// ```
/// use omron_fins::utils::toggle_bit;
///
/// let value: u16 = 0b0000_0000_0000_0001;
/// let result = toggle_bit(value, 0);
/// assert_eq!(result, 0);
/// let result = toggle_bit(result, 0);
/// assert_eq!(result, 1);
/// ```
#[inline]
pub fn toggle_bit(value: u16, bit: u8) -> u16 {
    value ^ (1 << bit)
}

/// Converts a 16-bit word to an array of 16 boolean values.
///
/// # Arguments
///
/// * `value` - The 16-bit word to convert
///
/// # Returns
///
/// An array of 16 booleans where index 0 is the LSB.
///
/// # Example
///
/// ```
/// use omron_fins::utils::word_to_bits;
///
/// let value: u16 = 0b0000_0000_0000_0011; // bits 0 and 1 are set
/// let bits = word_to_bits(value);
/// assert!(bits[0]);
/// assert!(bits[1]);
/// assert!(!bits[2]);
/// ```
pub fn word_to_bits(value: u16) -> [bool; 16] {
    let mut bits = [false; 16];
    for i in 0..16 {
        bits[i] = get_bit(value, i as u8);
    }
    bits
}

/// Converts an array of 16 booleans to a 16-bit word.
///
/// # Arguments
///
/// * `bits` - Array of 16 booleans where index 0 is the LSB
///
/// # Returns
///
/// The 16-bit word representation.
///
/// # Example
///
/// ```
/// use omron_fins::utils::bits_to_word;
///
/// let mut bits = [false; 16];
/// bits[0] = true;
/// bits[1] = true;
/// let value = bits_to_word(&bits);
/// assert_eq!(value, 0b0000_0000_0000_0011);
/// ```
pub fn bits_to_word(bits: &[bool; 16]) -> u16 {
    let mut value: u16 = 0;
    for (i, &bit) in bits.iter().enumerate() {
        if bit {
            value |= 1 << i;
        }
    }
    value
}

/// Gets all bits from a 16-bit word as a vector of BitInfo.
///
/// # Arguments
///
/// * `value` - The 16-bit word to analyze
///
/// # Returns
///
/// A vector of 16 BitInfo structs with index and value.
///
/// # Example
///
/// ```
/// use omron_fins::utils::get_bits;
///
/// let value: u16 = 0b0000_0000_0000_0101;
/// let bits = get_bits(value);
/// assert_eq!(bits[0].value, true);
/// assert_eq!(bits[1].value, false);
/// assert_eq!(bits[2].value, true);
/// ```
pub fn get_bits(value: u16) -> Vec<BitInfo> {
    (0..16)
        .map(|i| BitInfo::new(i, get_bit(value, i)))
        .collect()
}

/// Gets only the bits that are ON (set to 1) from a 16-bit word.
///
/// # Arguments
///
/// * `value` - The 16-bit word to analyze
///
/// # Returns
///
/// A vector of bit indices that are ON.
///
/// # Example
///
/// ```
/// use omron_fins::utils::get_on_bits;
///
/// let value: u16 = 0b0000_0000_0010_0101; // bits 0, 2, 5 are ON
/// let on_bits = get_on_bits(value);
/// assert_eq!(on_bits, vec![0, 2, 5]);
/// ```
pub fn get_on_bits(value: u16) -> Vec<u8> {
    (0..16).filter(|&i| get_bit(value, i)).collect()
}

/// Gets only the bits that are OFF (set to 0) from a 16-bit word.
///
/// # Arguments
///
/// * `value` - The 16-bit word to analyze
///
/// # Returns
///
/// A vector of bit indices that are OFF.
///
/// # Example
///
/// ```
/// use omron_fins::utils::get_off_bits;
///
/// let value: u16 = 0xFFFF; // all bits ON
/// let off_bits = get_off_bits(value);
/// assert!(off_bits.is_empty());
///
/// let value: u16 = 0xFFFE; // bit 0 OFF
/// let off_bits = get_off_bits(value);
/// assert_eq!(off_bits, vec![0]);
/// ```
pub fn get_off_bits(value: u16) -> Vec<u8> {
    (0..16).filter(|&i| !get_bit(value, i)).collect()
}

/// Counts the number of bits that are ON in a 16-bit word.
///
/// # Arguments
///
/// * `value` - The 16-bit word to analyze
///
/// # Returns
///
/// The count of bits that are set (1).
///
/// # Example
///
/// ```
/// use omron_fins::utils::count_on_bits;
///
/// let value: u16 = 0b0000_0000_0010_0101;
/// assert_eq!(count_on_bits(value), 3);
/// ```
#[inline]
pub fn count_on_bits(value: u16) -> u32 {
    value.count_ones()
}

/// Formats a 16-bit word as a binary string with bit labels.
///
/// # Arguments
///
/// * `value` - The 16-bit word to format
///
/// # Returns
///
/// A formatted string showing all bits with their indices.
///
/// # Example
///
/// ```
/// use omron_fins::utils::format_bits;
///
/// let value: u16 = 0b0000_0000_0000_0101;
/// let formatted = format_bits(value);
/// // Output shows each bit with its index
/// ```
pub fn format_bits(value: u16) -> String {
    let mut lines = Vec::with_capacity(16);
    for i in 0..16 {
        let bit = get_bit(value, i);
        lines.push(format!("Bit {:2}: {}", i, if bit { "ON" } else { "OFF" }));
    }
    lines.join("\n")
}

/// Formats a 16-bit word as a compact binary representation.
///
/// # Arguments
///
/// * `value` - The 16-bit word to format
///
/// # Returns
///
/// A string in the format "0b0000_0000_0000_0000".
///
/// # Example
///
/// ```
/// use omron_fins::utils::format_binary;
///
/// let value: u16 = 0x1234;
/// let formatted = format_binary(value);
/// assert_eq!(formatted, "0b0001_0010_0011_0100");
/// ```
pub fn format_binary(value: u16) -> String {
    let binary = format!("{:016b}", value);
    format!(
        "0b{}_{}_{}_{}", 
        &binary[0..4],
        &binary[4..8],
        &binary[8..12],
        &binary[12..16]
    )
}

/// Formats a 16-bit word as hexadecimal.
///
/// # Arguments
///
/// * `value` - The 16-bit word to format
///
/// # Returns
///
/// A string in the format "0x0000".
///
/// # Example
///
/// ```
/// use omron_fins::utils::format_hex;
///
/// let value: u16 = 0x1234;
/// let formatted = format_hex(value);
/// assert_eq!(formatted, "0x1234");
/// ```
pub fn format_hex(value: u16) -> String {
    format!("0x{:04X}", value)
}

/// Prints all bits of a 16-bit word to stdout.
///
/// This is a convenience function for debugging that displays
/// each bit's index and value.
///
/// # Arguments
///
/// * `value` - The 16-bit word to print
///
/// # Example
///
/// ```no_run
/// use omron_fins::utils::print_bits;
///
/// let value: u16 = 0b0000_0000_0000_0101;
/// print_bits(value);
/// // Output:
/// // Bit  0: ON
/// // Bit  1: OFF
/// // Bit  2: ON
/// // ...
/// ```
pub fn print_bits(value: u16) {
    println!("{}", format_bits(value));
}

/// Extracts a range of bits from a 16-bit word.
///
/// # Arguments
///
/// * `value` - The 16-bit word to extract from
/// * `start_bit` - Starting bit position (inclusive)
/// * `end_bit` - Ending bit position (inclusive)
///
/// # Returns
///
/// The extracted bits as a u16, shifted to start from bit 0.
///
/// # Example
///
/// ```
/// use omron_fins::utils::extract_bits;
///
/// let value: u16 = 0b1111_0000_1010_0101;
/// let extracted = extract_bits(value, 4, 7); // bits 4-7
/// assert_eq!(extracted, 0b1010);
/// ```
pub fn extract_bits(value: u16, start_bit: u8, end_bit: u8) -> u16 {
    let mask = ((1u32 << (end_bit - start_bit + 1)) - 1) as u16;
    (value >> start_bit) & mask
}

/// Checks if all specified bits are ON.
///
/// # Arguments
///
/// * `value` - The 16-bit word to check
/// * `bits` - Slice of bit indices to check
///
/// # Returns
///
/// `true` if all specified bits are ON, `false` otherwise.
///
/// # Example
///
/// ```
/// use omron_fins::utils::all_bits_on;
///
/// let value: u16 = 0b0000_0000_0000_0111; // bits 0, 1, 2 are ON
/// assert!(all_bits_on(value, &[0, 1, 2]));
/// assert!(!all_bits_on(value, &[0, 1, 2, 3]));
/// ```
pub fn all_bits_on(value: u16, bits: &[u8]) -> bool {
    bits.iter().all(|&b| get_bit(value, b))
}

/// Checks if any of the specified bits are ON.
///
/// # Arguments
///
/// * `value` - The 16-bit word to check
/// * `bits` - Slice of bit indices to check
///
/// # Returns
///
/// `true` if any of the specified bits are ON, `false` otherwise.
///
/// # Example
///
/// ```
/// use omron_fins::utils::any_bit_on;
///
/// let value: u16 = 0b0000_0000_0000_0001; // only bit 0 is ON
/// assert!(any_bit_on(value, &[0, 1, 2]));
/// assert!(!any_bit_on(value, &[3, 4, 5]));
/// ```
pub fn any_bit_on(value: u16, bits: &[u8]) -> bool {
    bits.iter().any(|&b| get_bit(value, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bit() {
        let value: u16 = 0b0000_0000_0000_0101;
        assert!(get_bit(value, 0));
        assert!(!get_bit(value, 1));
        assert!(get_bit(value, 2));
        assert!(!get_bit(value, 15));
    }

    #[test]
    fn test_set_bit() {
        assert_eq!(set_bit(0, 0, true), 1);
        assert_eq!(set_bit(1, 0, false), 0);
        assert_eq!(set_bit(0, 15, true), 0x8000);
        assert_eq!(set_bit(0xFFFF, 8, false), 0xFEFF);
    }

    #[test]
    fn test_toggle_bit() {
        assert_eq!(toggle_bit(0, 0), 1);
        assert_eq!(toggle_bit(1, 0), 0);
        assert_eq!(toggle_bit(0x5555, 0), 0x5554);
        assert_eq!(toggle_bit(0x5554, 0), 0x5555);
    }

    #[test]
    fn test_word_to_bits() {
        let value: u16 = 0b0000_0000_0000_0011;
        let bits = word_to_bits(value);
        assert!(bits[0]);
        assert!(bits[1]);
        assert!(!bits[2]);
        for i in 3..16 {
            assert!(!bits[i]);
        }
    }

    #[test]
    fn test_bits_to_word() {
        let mut bits = [false; 16];
        bits[0] = true;
        bits[1] = true;
        assert_eq!(bits_to_word(&bits), 0b0000_0000_0000_0011);
    }

    #[test]
    fn test_word_bits_roundtrip() {
        let original: u16 = 0xA5C3;
        let bits = word_to_bits(original);
        let result = bits_to_word(&bits);
        assert_eq!(original, result);
    }

    #[test]
    fn test_get_bits() {
        let value: u16 = 0b0000_0000_0000_0101;
        let bits = get_bits(value);
        assert_eq!(bits.len(), 16);
        assert_eq!(bits[0].index, 0);
        assert!(bits[0].value);
        assert!(!bits[1].value);
        assert!(bits[2].value);
    }

    #[test]
    fn test_get_on_bits() {
        let value: u16 = 0b0000_0000_0010_0101;
        let on_bits = get_on_bits(value);
        assert_eq!(on_bits, vec![0, 2, 5]);
    }

    #[test]
    fn test_get_off_bits() {
        let value: u16 = 0xFFFF;
        assert!(get_off_bits(value).is_empty());

        let value: u16 = 0xFFFE;
        assert_eq!(get_off_bits(value), vec![0]);
    }

    #[test]
    fn test_count_on_bits() {
        assert_eq!(count_on_bits(0), 0);
        assert_eq!(count_on_bits(1), 1);
        assert_eq!(count_on_bits(0b0010_0101), 3);
        assert_eq!(count_on_bits(0xFFFF), 16);
    }

    #[test]
    fn test_format_binary() {
        assert_eq!(format_binary(0x1234), "0b0001_0010_0011_0100");
        assert_eq!(format_binary(0xFFFF), "0b1111_1111_1111_1111");
        assert_eq!(format_binary(0x0000), "0b0000_0000_0000_0000");
    }

    #[test]
    fn test_format_hex() {
        assert_eq!(format_hex(0x1234), "0x1234");
        assert_eq!(format_hex(0x00FF), "0x00FF");
        assert_eq!(format_hex(0xABCD), "0xABCD");
    }

    #[test]
    fn test_extract_bits() {
        let value: u16 = 0b1111_0000_1010_0101;
        assert_eq!(extract_bits(value, 0, 3), 0b0101);
        assert_eq!(extract_bits(value, 4, 7), 0b1010);
        assert_eq!(extract_bits(value, 8, 11), 0b0000);
        assert_eq!(extract_bits(value, 12, 15), 0b1111);
    }

    #[test]
    fn test_all_bits_on() {
        let value: u16 = 0b0000_0000_0000_0111;
        assert!(all_bits_on(value, &[0, 1, 2]));
        assert!(!all_bits_on(value, &[0, 1, 2, 3]));
        assert!(all_bits_on(value, &[]));
    }

    #[test]
    fn test_any_bit_on() {
        let value: u16 = 0b0000_0000_0000_0001;
        assert!(any_bit_on(value, &[0, 1, 2]));
        assert!(!any_bit_on(value, &[3, 4, 5]));
        assert!(!any_bit_on(value, &[]));
    }

    #[test]
    fn test_bit_info_display() {
        let bit = BitInfo::new(5, true);
        assert_eq!(bit.to_string(), "Bit 5: ON");

        let bit = BitInfo::new(0, false);
        assert_eq!(bit.to_string(), "Bit 0: OFF");
    }
}
