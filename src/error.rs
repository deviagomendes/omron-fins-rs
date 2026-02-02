//! Error types for the FINS protocol.

use std::io;
use thiserror::Error;

/// Result type alias for FINS operations.
pub type Result<T> = std::result::Result<T, FinsError>;

/// Errors that can occur during FINS communication.
#[derive(Debug, Error)]
pub enum FinsError {
    /// Error returned by the PLC with main and sub codes.
    #[error("PLC error: main code 0x{main_code:02X}, sub code 0x{sub_code:02X}")]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plc_error_display() {
        let err = FinsError::plc_error(0x01, 0x01);
        assert_eq!(err.to_string(), "PLC error: main code 0x01, sub code 0x01");
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
}
