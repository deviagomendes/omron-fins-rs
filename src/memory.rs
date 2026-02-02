//! Memory area definitions for FINS protocol.

use crate::error::{FinsError, Result};

/// Memory areas available in Omron PLCs.
///
/// Each area has specific FINS codes for word and bit access.
/// DM area only supports word access.
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
}

impl std::fmt::Display for MemoryArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryArea::CIO => write!(f, "CIO"),
            MemoryArea::WR => write!(f, "WR"),
            MemoryArea::HR => write!(f, "HR"),
            MemoryArea::DM => write!(f, "DM"),
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
    }

    #[test]
    fn test_bit_codes() {
        assert_eq!(MemoryArea::CIO.bit_code().unwrap(), 0x30);
        assert_eq!(MemoryArea::WR.bit_code().unwrap(), 0x31);
        assert_eq!(MemoryArea::HR.bit_code().unwrap(), 0x32);
        assert!(MemoryArea::DM.bit_code().is_err());
    }

    #[test]
    fn test_supports_bit_access() {
        assert!(MemoryArea::CIO.supports_bit_access());
        assert!(MemoryArea::WR.supports_bit_access());
        assert!(MemoryArea::HR.supports_bit_access());
        assert!(!MemoryArea::DM.supports_bit_access());
    }

    #[test]
    fn test_display() {
        assert_eq!(MemoryArea::CIO.to_string(), "CIO");
        assert_eq!(MemoryArea::WR.to_string(), "WR");
        assert_eq!(MemoryArea::HR.to_string(), "HR");
        assert_eq!(MemoryArea::DM.to_string(), "DM");
    }
}
