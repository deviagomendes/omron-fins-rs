//! Memory area definitions for the FINS protocol.
//!
//! This module defines the [`MemoryArea`] enum which represents the different
//! memory areas available in Omron PLCs. Each area has specific characteristics
//! and access capabilities.
//!
//! # Memory Areas Overview
//!
//! | Area | Description | Word Access | Bit Access |
//! |------|-------------|:-----------:|:----------:|
//! | CIO | Core I/O - inputs, outputs, internal relays | ✓ | ✓ |
//! | WR | Work area - temporary work bits/words | ✓ | ✓ |
//! | HR | Holding area - retentive bits/words | ✓ | ✓ |
//! | DM | Data Memory - numeric data storage | ✓ | ✗ |
//! | AR | Auxiliary Relay - system status/control | ✓ | ✓ |
//!
//! # Example
//!
//! ```
//! use omron_fins::MemoryArea;
//!
//! // Check if an area supports bit access
//! assert!(MemoryArea::CIO.supports_bit_access());
//! assert!(MemoryArea::WR.supports_bit_access());
//! assert!(!MemoryArea::DM.supports_bit_access());
//!
//! // Display the area name
//! assert_eq!(MemoryArea::DM.to_string(), "DM");
//! ```

use crate::error::{FinsError, Result};

/// Memory areas available in Omron PLCs.
///
/// Each area has specific FINS codes for word and bit access.
/// The DM area only supports word access; attempting bit operations
/// on DM will return an error.
///
/// # FINS Protocol Codes
///
/// Internally, each area maps to specific FINS protocol codes:
/// - Word access codes are used for reading/writing full 16-bit words
/// - Bit access codes are used for reading/writing individual bits
///
/// These codes are internal to the library and not exposed in the public API.
///
/// # Example
///
/// ```
/// use omron_fins::MemoryArea;
///
/// // All areas support word access
/// let areas = [MemoryArea::CIO, MemoryArea::WR, MemoryArea::HR, MemoryArea::DM, MemoryArea::AR];
///
/// // Only some support bit access
/// for area in areas {
///     println!("{}: bit access = {}", area, area.supports_bit_access());
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryArea {
    /// CIO (Core I/O) area - general purpose I/O and internal relays.
    CIO,
    /// WR (Work) area - work bits/words.
    WR,
    /// HR (Holding) area - holding bits/words that retain values.
    HR,
    /// DM (Data Memory) area - word-only data storage.
    DM,
    /// AR (Auxiliary Relay) area - system status and control bits/words.
    AR,
}

impl MemoryArea {
    /// Returns the FINS code for word access to this memory area.
    ///
    /// These codes are used in FINS commands to identify the memory area.
    pub(crate) fn word_code(self) -> u8 {
        match self {
            MemoryArea::CIO => 0xB0,
            MemoryArea::WR => 0xB1,
            MemoryArea::HR => 0xB2,
            MemoryArea::DM => 0x82,
            MemoryArea::AR => 0xB3,
        }
    }

    /// Returns the FINS code for bit access to this memory area.
    ///
    /// # Errors
    ///
    /// Returns `FinsError::InvalidAddressing` if the memory area does not
    /// support bit access (DM area).
    pub(crate) fn bit_code(self) -> Result<u8> {
        match self {
            MemoryArea::CIO => Ok(0x30),
            MemoryArea::WR => Ok(0x31),
            MemoryArea::HR => Ok(0x32),
            MemoryArea::DM => Err(FinsError::invalid_addressing(
                "DM area does not support bit access",
            )),
            MemoryArea::AR => Ok(0x33),
        }
    }

    /// Returns whether this memory area supports bit access.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::MemoryArea;
    ///
    /// assert!(MemoryArea::CIO.supports_bit_access());
    /// assert!(!MemoryArea::DM.supports_bit_access());
    /// ```
    pub fn supports_bit_access(self) -> bool {
        !matches!(self, MemoryArea::DM)
    }

    /// Returns the maximum number of words supported by this memory area.
    ///
    /// The values represent the common capacities for Omron Ethernet PLCs.
    /// - CIO: 4096 words
    /// - WR: 512 words
    /// - HR: 512 words
    /// - DM: 4096 words
    /// - AR: 1024 words
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::MemoryArea;
    ///
    /// assert_eq!(MemoryArea::CIO.max_words(), 4096);
    /// assert_eq!(MemoryArea::WR.max_words(), 512);
    /// ```
    pub fn max_words(self) -> u16 {
        match self {
            MemoryArea::CIO => 4096,
            MemoryArea::WR => 512,
            MemoryArea::HR => 512,
            MemoryArea::DM => 4096,
            MemoryArea::AR => 1024,
        }
    }

    /// Checks if a read or write operation fits within the memory boundaries.
    ///
    /// # Arguments
    ///
    /// * `address` - Starting word address
    /// * `count` - Number of words to read/write
    ///
    /// # Errors
    ///
    /// Returns `FinsError::InvalidParameter` if the requested range exceeds the memory size.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::MemoryArea;
    ///
    /// assert!(MemoryArea::CIO.check_bounds(0, 10).is_ok());
    /// assert!(MemoryArea::WR.check_bounds(500, 20).is_err());
    /// ```
    pub fn check_bounds(self, address: u16, count: u16) -> Result<()> {
        let max = self.max_words();
        // Check if address + count - 1 exceeds max-1, which simplifies to address + count > max.
        // Also safeguard against overflow
        if let Some(end) = address.checked_add(count) {
            if end > max {
                return Err(FinsError::invalid_parameter(
                    "address/count",
                    format!(
                        "operation exceeds the maximum {} capacity of {} words (requested: address {} + count {})",
                        self, max, address, count
                    ),
                ));
            }
        } else {
            return Err(FinsError::invalid_parameter(
                "address/count",
                "operation address+count causes arithmetic overflow",
            ));
        }

        Ok(())
    }
}

impl std::fmt::Display for MemoryArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryArea::CIO => write!(f, "CIO"),
            MemoryArea::WR => write!(f, "WR"),
            MemoryArea::HR => write!(f, "HR"),
            MemoryArea::DM => write!(f, "DM"),
            MemoryArea::AR => write!(f, "AR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_codes() {
        assert_eq!(MemoryArea::CIO.word_code(), 0xB0);
        assert_eq!(MemoryArea::WR.word_code(), 0xB1);
        assert_eq!(MemoryArea::HR.word_code(), 0xB2);
        assert_eq!(MemoryArea::DM.word_code(), 0x82);
        assert_eq!(MemoryArea::AR.word_code(), 0xB3);
    }

    #[test]
    fn test_bit_codes() {
        assert_eq!(MemoryArea::CIO.bit_code().unwrap(), 0x30);
        assert_eq!(MemoryArea::WR.bit_code().unwrap(), 0x31);
        assert_eq!(MemoryArea::HR.bit_code().unwrap(), 0x32);
        assert!(MemoryArea::DM.bit_code().is_err());
        assert_eq!(MemoryArea::AR.bit_code().unwrap(), 0x33);
    }

    #[test]
    fn test_supports_bit_access() {
        assert!(MemoryArea::CIO.supports_bit_access());
        assert!(MemoryArea::WR.supports_bit_access());
        assert!(MemoryArea::HR.supports_bit_access());
        assert!(!MemoryArea::DM.supports_bit_access());
        assert!(MemoryArea::AR.supports_bit_access());
    }

    #[test]
    fn test_display() {
        assert_eq!(MemoryArea::CIO.to_string(), "CIO");
        assert_eq!(MemoryArea::WR.to_string(), "WR");
        assert_eq!(MemoryArea::HR.to_string(), "HR");
        assert_eq!(MemoryArea::DM.to_string(), "DM");
        assert_eq!(MemoryArea::AR.to_string(), "AR");
    }

    #[test]
    fn test_max_words() {
        assert_eq!(MemoryArea::CIO.max_words(), 4096);
        assert_eq!(MemoryArea::WR.max_words(), 512);
        assert_eq!(MemoryArea::HR.max_words(), 512);
        assert_eq!(MemoryArea::DM.max_words(), 4096);
        assert_eq!(MemoryArea::AR.max_words(), 1024);
    }

    #[test]
    fn test_check_bounds() {
        assert!(MemoryArea::CIO.check_bounds(0, 4096).is_ok());
        assert!(MemoryArea::CIO.check_bounds(4095, 1).is_ok());
        assert!(MemoryArea::CIO.check_bounds(4096, 1).is_err());
        assert!(MemoryArea::CIO.check_bounds(0, 4097).is_err());

        assert!(MemoryArea::WR.check_bounds(0, 512).is_ok());
        assert!(MemoryArea::WR.check_bounds(500, 12).is_ok());
        assert!(MemoryArea::WR.check_bounds(500, 13).is_err());

        // Overflow check
        assert!(MemoryArea::CIO.check_bounds(u16::MAX, 2).is_err());
    }
}
