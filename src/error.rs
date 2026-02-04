//! Error types for the FINS protocol.
//!
//! This module defines the [`FinsError`] enum and the [`Result`] type alias
//! used throughout the library for error handling.
//!
//! # Error Categories
//!
//! Errors are categorized into several types:
//!
//! - **PLC Errors** - Errors returned by the PLC itself, with main/sub codes
//! - **Communication Errors** - Timeouts and I/O errors
//! - **Validation Errors** - Invalid parameters or addressing
//! - **Protocol Errors** - Invalid responses or SID mismatches
//!
//! # Example
//!
//! ```no_run
//! use omron_fins::{Client, ClientConfig, MemoryArea, FinsError};
//! use std::net::Ipv4Addr;
//!
//! let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
//! let client = Client::new(config)?;
//!
//! match client.read(MemoryArea::DM, 100, 10) {
//!     Ok(data) => println!("Data: {:?}", data),
//!     Err(FinsError::Timeout) => {
//!         eprintln!("Communication timed out");
//!     }
//!     Err(ref e @ FinsError::PlcError { main_code, sub_code }) => {
//!         // The error message now includes the description automatically:
//!         // e.g., "PLC error (0x11:0x04): The end of specified word range exceeds acceptable range"
//!         eprintln!("{}", e);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # Ok::<(), FinsError>(())
//! ```
//!
//! # Creating Errors
//!
//! The library provides convenience constructors for creating errors:
//!
//! ```
//! use omron_fins::FinsError;
//!
//! // Create a PLC error
//! let err = FinsError::plc_error(0x01, 0x01);
//!
//! // Create a parameter error
//! let err = FinsError::invalid_parameter("count", "must be greater than 0");
//!
//! // Create an addressing error
//! let err = FinsError::invalid_addressing("DM area does not support bit access");
//! ```

use std::io;
use thiserror::Error;

/// Returns a human-readable description for FINS error codes.
///
/// This function maps the main and sub error codes returned by Omron PLCs
/// to their corresponding descriptions according to the FINS protocol specification.
///
/// # Example
///
/// ```
/// use omron_fins::fins_error_description;
///
/// let desc = fins_error_description(0x11, 0x04);
/// assert_eq!(desc, "The end of specified word range exceeds acceptable range");
/// ```
pub fn fins_error_description(main_code: u8, sub_code: u8) -> &'static str {
    match (main_code, sub_code) {
        // Normal completion
        (0x00, 0x00) => "Normal completion",
        (0x00, 0x01) => "Service was interrupted",

        // Local node errors (0x01)
        (0x01, 0x01) => "Local node not part of Network",
        (0x01, 0x02) => "Token time-out, node number too large",
        (0x01, 0x03) => "Number of transmit retries exceeded",
        (0x01, 0x04) => "Maximum number of frames exceeded",
        (0x01, 0x05) => "Node number setting error (range)",
        (0x01, 0x06) => "Node number duplication error",

        // Destination node errors (0x02)
        (0x02, 0x01) => "Destination node not part of Network",
        (0x02, 0x02) => "No node with the specified node number",
        (0x02, 0x03) => "Third node not part of Network: Broadcasting was specified",
        (0x02, 0x04) => "Busy error, destination node busy",
        (0x02, 0x05) => "Response time-out",

        // Controller errors (0x03)
        (0x03, 0x01) => "Error occurred: ERC indicator is lit",
        (0x03, 0x02) => "CPU error occurred in the PC at the destination node",
        (0x03, 0x03) => "A controller error has prevented a normal response",
        (0x03, 0x04) => "Node number setting error",

        // Service unsupported errors (0x04)
        (0x04, 0x01) => "An undefined command has been used",
        (0x04, 0x02) => "Cannot process command because the specified unit model or version is wrong",

        // Routing errors (0x05)
        (0x05, 0x01) => "Destination node number is not set in the routing table",
        (0x05, 0x02) => "Routing table isn't registered",
        (0x05, 0x03) => "Routing table error",
        (0x05, 0x04) => "Max relay nodes (2) was exceeded",

        // Command format errors (0x10)
        (0x10, 0x01) => "The command is longer than the max permissible length",
        (0x10, 0x02) => "The command is shorter than the min permissible length",
        (0x10, 0x03) => "The designated number of data items differs from the actual number",
        (0x10, 0x04) => "An incorrect command format has been used",
        (0x10, 0x05) => "An incorrect header has been used",

        // Parameter errors (0x11)
        (0x11, 0x01) => "Memory area code invalid or DM is not available",
        (0x11, 0x02) => "Access size is wrong in command",
        (0x11, 0x03) => "First address in inaccessible area",
        (0x11, 0x04) => "The end of specified word range exceeds acceptable range",
        (0x11, 0x06) => "A non-existent program number",
        (0x11, 0x09) => "The size of data items in command block are wrong",
        (0x11, 0x0A) => "The IOM break function cannot be executed",
        (0x11, 0x0B) => "The response block is longer than the max length",
        (0x11, 0x0C) => "An incorrect parameter code has been specified",

        // Read errors (0x20)
        (0x20, 0x02) => "The data is protected",
        (0x20, 0x03) => "Registered table does not exist",
        (0x20, 0x04) => "Search data does not exist",
        (0x20, 0x05) => "Non-existent program number",
        (0x20, 0x06) => "Non-existent file",
        (0x20, 0x07) => "Verification error",

        // Write errors (0x21)
        (0x21, 0x01) => "Specified area is read-only",
        (0x21, 0x02) => "The data is protected",
        (0x21, 0x03) => "Too many files open",
        (0x21, 0x05) => "Non-existent program number",
        (0x21, 0x06) => "Non-existent file",
        (0x21, 0x07) => "File already exists",
        (0x21, 0x08) => "Data cannot be changed",

        // Mode errors (0x22)
        (0x22, 0x01) => "The mode is wrong (executing)",
        (0x22, 0x02) => "The mode is wrong (stopped)",
        (0x22, 0x03) => "The PC is in the PROGRAM mode",
        (0x22, 0x04) => "The PC is in the DEBUG mode",
        (0x22, 0x05) => "The PC is in the MONITOR mode",
        (0x22, 0x06) => "The PC is in the RUN mode",
        (0x22, 0x07) => "The specified node is not the control node",
        (0x22, 0x08) => "The mode is wrong and the step cannot be executed",

        // Device errors (0x23)
        (0x23, 0x01) => "The file device does not exist where specified",
        (0x23, 0x02) => "The specified memory does not exist",
        (0x23, 0x03) => "No clock exists",

        // Data link errors (0x24)
        (0x24, 0x01) => "Data link table is incorrect",

        // Unit errors (0x25)
        (0x25, 0x02) => "Parity / checksum error occurred",
        (0x25, 0x03) => "I/O setting error",
        (0x25, 0x04) => "Too many I/O points",
        (0x25, 0x05) => "CPU bus error",
        (0x25, 0x06) => "I/O duplication error",
        (0x25, 0x07) => "I/O bus error",
        (0x25, 0x09) => "SYSMAC BUS/2 error",
        (0x25, 0x0A) => "Special I/O Unit error",
        (0x25, 0x0D) => "Duplication in SYSMAC BUS word allocation",
        (0x25, 0x0F) => "A memory error has occurred",
        (0x25, 0x10) => "Terminator not connected in SYSMAC BUS system",

        // Access errors (0x26)
        (0x26, 0x01) => "The specified area is not protected",
        (0x26, 0x02) => "An incorrect password has been specified",
        (0x26, 0x04) => "The specified area is protected",
        (0x26, 0x05) => "The service is being executed",
        (0x26, 0x06) => "The service is not being executed",
        (0x26, 0x07) => "Service cannot be executed from local node",
        (0x26, 0x08) => "Service cannot be executed, settings are incorrect",
        (0x26, 0x09) => "Service cannot be executed, incorrect settings in command data",
        (0x26, 0x0A) => "The specified action has already been registered",
        (0x26, 0x0B) => "Cannot clear error, error still exists",

        // Access right errors (0x30)
        (0x30, 0x01) => "The access right is held by another device",

        // Abort errors (0x40)
        (0x40, 0x01) => "Command aborted with ABORT command",

        // Unknown error
        _ => "Unknown error code",
    }
}

/// Result type alias for FINS operations.
pub type Result<T> = std::result::Result<T, FinsError>;

/// Errors that can occur during FINS communication.
#[derive(Debug, Error)]
pub enum FinsError {
    /// Error returned by the PLC with main and sub codes.
    #[error("PLC error (0x{main_code:02X}:0x{sub_code:02X}): {}", fins_error_description(*.main_code, *.sub_code))]
    PlcError {
        /// Main error code from PLC response.
        main_code: u8,
        /// Sub error code from PLC response.
        sub_code: u8,
    },

    /// Invalid memory addressing.
    #[error("Invalid addressing: {reason}")]
    InvalidAddressing {
        /// Description of the addressing error.
        reason: String,
    },

    /// Invalid parameter provided.
    #[error("Invalid parameter '{parameter}': {reason}")]
    InvalidParameter {
        /// Name of the invalid parameter.
        parameter: String,
        /// Description of why the parameter is invalid.
        reason: String,
    },

    /// Invalid response received from PLC.
    #[error("Invalid response: {reason}")]
    InvalidResponse {
        /// Description of the response error.
        reason: String,
    },

    /// Communication timeout.
    #[error("Communication timeout")]
    Timeout,

    /// I/O error during communication.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Service ID mismatch between request and response.
    #[error("SID mismatch: expected 0x{expected:02X}, received 0x{received:02X}")]
    SidMismatch {
        /// Expected SID value.
        expected: u8,
        /// Received SID value.
        received: u8,
    },
}

impl FinsError {
    /// Creates a new `PlcError` from main and sub codes.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsError;
    ///
    /// let err = FinsError::plc_error(0x01, 0x01);
    /// ```
    pub fn plc_error(main_code: u8, sub_code: u8) -> Self {
        Self::PlcError {
            main_code,
            sub_code,
        }
    }

    /// Creates a new `InvalidAddressing` error.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsError;
    ///
    /// let err = FinsError::invalid_addressing("DM area does not support bit access");
    /// ```
    pub fn invalid_addressing(reason: impl Into<String>) -> Self {
        Self::InvalidAddressing {
            reason: reason.into(),
        }
    }

    /// Creates a new `InvalidParameter` error.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsError;
    ///
    /// let err = FinsError::invalid_parameter("count", "must be greater than 0");
    /// ```
    pub fn invalid_parameter(parameter: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidParameter {
            parameter: parameter.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new `InvalidResponse` error.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsError;
    ///
    /// let err = FinsError::invalid_response("response too short");
    /// ```
    pub fn invalid_response(reason: impl Into<String>) -> Self {
        Self::InvalidResponse {
            reason: reason.into(),
        }
    }

    /// Creates a new `SidMismatch` error.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsError;
    ///
    /// let err = FinsError::sid_mismatch(0x01, 0x02);
    /// ```
    pub fn sid_mismatch(expected: u8, received: u8) -> Self {
        Self::SidMismatch { expected, received }
    }

    /// Returns the error description if this is a `PlcError`.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsError;
    ///
    /// let err = FinsError::plc_error(0x11, 0x04);
    /// assert_eq!(
    ///     err.description(),
    ///     Some("The end of specified word range exceeds acceptable range")
    /// );
    ///
    /// let timeout = FinsError::Timeout;
    /// assert_eq!(timeout.description(), None);
    /// ```
    pub fn description(&self) -> Option<&'static str> {
        match self {
            Self::PlcError {
                main_code,
                sub_code,
            } => Some(fins_error_description(*main_code, *sub_code)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plc_error_display() {
        let err = FinsError::plc_error(0x01, 0x01);
        assert_eq!(
            err.to_string(),
            "PLC error (0x01:0x01): Local node not part of Network"
        );
    }

    #[test]
    fn test_plc_error_display_unknown() {
        let err = FinsError::plc_error(0xFF, 0xFF);
        assert_eq!(err.to_string(), "PLC error (0xFF:0xFF): Unknown error code");
    }

    #[test]
    fn test_invalid_addressing_display() {
        let err = FinsError::invalid_addressing("DM area does not support bit access");
        assert_eq!(
            err.to_string(),
            "Invalid addressing: DM area does not support bit access"
        );
    }

    #[test]
    fn test_timeout_display() {
        let err = FinsError::Timeout;
        assert_eq!(err.to_string(), "Communication timeout");
    }

    #[test]
    fn test_sid_mismatch_display() {
        let err = FinsError::sid_mismatch(0x01, 0x02);
        assert_eq!(
            err.to_string(),
            "SID mismatch: expected 0x01, received 0x02"
        );
    }

    #[test]
    fn test_plc_error_description_method() {
        let err = FinsError::plc_error(0x11, 0x04);
        assert_eq!(
            err.description(),
            Some("The end of specified word range exceeds acceptable range")
        );

        let timeout = FinsError::Timeout;
        assert_eq!(timeout.description(), None);
    }

    #[test]
    fn test_fins_error_description_various_codes() {
        // Normal completion
        assert_eq!(fins_error_description(0x00, 0x00), "Normal completion");

        // Local node errors
        assert_eq!(
            fins_error_description(0x01, 0x03),
            "Number of transmit retries exceeded"
        );

        // Destination node errors
        assert_eq!(
            fins_error_description(0x02, 0x04),
            "Busy error, destination node busy"
        );

        // Command format errors
        assert_eq!(
            fins_error_description(0x10, 0x04),
            "An incorrect command format has been used"
        );

        // Memory area errors
        assert_eq!(
            fins_error_description(0x11, 0x01),
            "Memory area code invalid or DM is not available"
        );

        // Mode errors
        assert_eq!(fins_error_description(0x22, 0x06), "The PC is in the RUN mode");

        // Unit errors
        assert_eq!(fins_error_description(0x25, 0x05), "CPU bus error");

        // Access errors
        assert_eq!(
            fins_error_description(0x26, 0x02),
            "An incorrect password has been specified"
        );

        // Abort errors
        assert_eq!(
            fins_error_description(0x40, 0x01),
            "Command aborted with ABORT command"
        );
    }
}
