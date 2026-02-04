//! FINS response parsing and validation.
//!
//! This module handles parsing and validation of FINS responses received from PLCs.
//!
//! # Response Structure
//!
//! A FINS response consists of:
//!
//! | Component | Size | Description |
//! |-----------|------|-------------|
//! | Header | 10 bytes | FINS header (same structure as command) |
//! | MRC | 1 byte | Main Response Code |
//! | SRC | 1 byte | Sub Response Code |
//! | Main Code | 1 byte | Error main code (0x00 = success) |
//! | Sub Code | 1 byte | Error sub code (0x00 = success) |
//! | Data | Variable | Response data (if any) |
//!
//! # Error Codes
//!
//! A response is successful if both main_code and sub_code are 0x00.
//! Non-zero codes indicate specific errors - refer to Omron documentation
//! for the complete error code reference.
//!
//! # Example
//!
//! ```
//! use omron_fins::FinsResponse;
//!
//! // Parse a successful response with data
//! let bytes = [
//!     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01, // header
//!     0x01, 0x01, // MRC, SRC
//!     0x00, 0x00, // success codes
//!     0x12, 0x34, 0x56, 0x78, // data: 0x1234, 0x5678
//! ];
//!
//! let response = FinsResponse::from_bytes(&bytes).unwrap();
//! assert!(response.is_success());
//!
//! let words = response.to_words().unwrap();
//! assert_eq!(words, vec![0x1234, 0x5678]);
//! ```

use crate::error::{FinsError, Result};
use crate::header::{FinsHeader, FINS_HEADER_SIZE};

/// Minimum response size: header (10) + MRC (1) + SRC (1) + main code (1) + sub code (1) = 14 bytes.
pub const MIN_RESPONSE_SIZE: usize = FINS_HEADER_SIZE + 4;

/// Parsed FINS response.
#[derive(Debug, Clone)]
pub struct FinsResponse {
    /// Response header.
    pub header: FinsHeader,
    /// Main Response Code (MRC).
    pub mrc: u8,
    /// Sub Response Code (SRC).
    pub src: u8,
    /// Main error code (0x00 = success).
    pub main_code: u8,
    /// Sub error code (0x00 = success).
    pub sub_code: u8,
    /// Response data (if any).
    pub data: Vec<u8>,
}

impl FinsResponse {
    /// Parses a FINS response from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The response is too short
    /// - The header is invalid
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsResponse;
    ///
    /// let bytes = [
    ///     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01, // header
    ///     0x01, 0x01, // MRC, SRC
    ///     0x00, 0x00, // main/sub codes (success)
    ///     0x12, 0x34, // data
    /// ];
    /// let response = FinsResponse::from_bytes(&bytes).unwrap();
    /// assert!(response.is_success());
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < MIN_RESPONSE_SIZE {
            return Err(FinsError::invalid_response(format!(
                "response too short: expected at least {} bytes, got {}",
                MIN_RESPONSE_SIZE,
                data.len()
            )));
        }

        let header = FinsHeader::from_bytes(&data[..FINS_HEADER_SIZE])?;

        Ok(Self {
            header,
            mrc: data[FINS_HEADER_SIZE],
            src: data[FINS_HEADER_SIZE + 1],
            main_code: data[FINS_HEADER_SIZE + 2],
            sub_code: data[FINS_HEADER_SIZE + 3],
            data: data[MIN_RESPONSE_SIZE..].to_vec(),
        })
    }

    /// Returns whether the response indicates success (main_code == 0 && sub_code == 0).
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsResponse;
    ///
    /// let success_bytes = [
    ///     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01,
    ///     0x01, 0x01, 0x00, 0x00,
    /// ];
    /// let response = FinsResponse::from_bytes(&success_bytes).unwrap();
    /// assert!(response.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        self.main_code == 0x00 && self.sub_code == 0x00
    }

    /// Validates the response and returns an error if it indicates failure.
    ///
    /// # Note
    ///
    /// Error code 0x0040 (routing table warning) is accepted when data is present,
    /// as this is common behavior with Omron PLCs and the Python fins-driver library
    /// handles it the same way.
    ///
    /// # Errors
    ///
    /// Returns `FinsError::PlcError` if main_code or sub_code is non-zero
    /// (except for the 0x0040 warning with data).
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsResponse;
    ///
    /// let error_bytes = [
    ///     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01,
    ///     0x01, 0x01, 0x01, 0x01, // error codes
    /// ];
    /// let response = FinsResponse::from_bytes(&error_bytes).unwrap();
    /// assert!(response.check_error().is_err());
    /// ```
    pub fn check_error(&self) -> Result<()> {
        if self.is_success() {
            Ok(())
        } else if self.main_code == 0x00 && self.sub_code == 0x40 && !self.data.is_empty() {
            // Accept routing table warning (0x0040) when data is present
            // This matches Python fins-driver behavior
            Ok(())
        } else {
            Err(FinsError::plc_error(self.main_code, self.sub_code))
        }
    }

    /// Validates the Service ID matches the expected value.
    ///
    /// # Errors
    ///
    /// Returns `FinsError::SidMismatch` if the SID doesn't match.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsResponse;
    ///
    /// let bytes = [
    ///     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x05,
    ///     0x01, 0x01, 0x00, 0x00,
    /// ];
    /// let response = FinsResponse::from_bytes(&bytes).unwrap();
    /// assert!(response.check_sid(0x05).is_ok());
    /// assert!(response.check_sid(0x01).is_err());
    /// ```
    pub fn check_sid(&self, expected: u8) -> Result<()> {
        if self.header.sid == expected {
            Ok(())
        } else {
            Err(FinsError::sid_mismatch(expected, self.header.sid))
        }
    }

    /// Converts response data to words (big-endian u16 values).
    ///
    /// # Errors
    ///
    /// Returns an error if the data length is not even.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsResponse;
    ///
    /// let bytes = [
    ///     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01,
    ///     0x01, 0x01, 0x00, 0x00,
    ///     0x12, 0x34, 0x56, 0x78, // data: 0x1234, 0x5678
    /// ];
    /// let response = FinsResponse::from_bytes(&bytes).unwrap();
    /// let words = response.to_words().unwrap();
    /// assert_eq!(words, vec![0x1234, 0x5678]);
    /// ```
    pub fn to_words(&self) -> Result<Vec<u16>> {
        if !self.data.len().is_multiple_of(2) {
            return Err(FinsError::invalid_response(
                "data length must be even for word conversion",
            ));
        }

        Ok(self
            .data
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect())
    }

    /// Converts response data to a single bit value.
    ///
    /// # Errors
    ///
    /// Returns an error if there's no data or the first byte is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsResponse;
    ///
    /// let bytes = [
    ///     0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01,
    ///     0x01, 0x01, 0x00, 0x00,
    ///     0x01, // bit value: true
    /// ];
    /// let response = FinsResponse::from_bytes(&bytes).unwrap();
    /// assert_eq!(response.to_bit().unwrap(), true);
    /// ```
    pub fn to_bit(&self) -> Result<bool> {
        if self.data.is_empty() {
            return Err(FinsError::invalid_response("no data for bit conversion"));
        }

        Ok(self.data[0] != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_response(main_code: u8, sub_code: u8, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![
            0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01, // header
            0x01, 0x01, // MRC, SRC
            main_code, sub_code,
        ];
        bytes.extend_from_slice(data);
        bytes
    }

    #[test]
    fn test_response_from_bytes_success() {
        let bytes = make_response(0x00, 0x00, &[0x12, 0x34]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();

        assert_eq!(response.header.icf, 0xC0);
        assert_eq!(response.header.sid, 0x01);
        assert_eq!(response.mrc, 0x01);
        assert_eq!(response.src, 0x01);
        assert_eq!(response.main_code, 0x00);
        assert_eq!(response.sub_code, 0x00);
        assert_eq!(response.data, vec![0x12, 0x34]);
    }

    #[test]
    fn test_response_from_bytes_too_short() {
        let bytes = [0xC0, 0x00, 0x02];
        let result = FinsResponse::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_success() {
        let success = FinsResponse::from_bytes(&make_response(0x00, 0x00, &[])).unwrap();
        assert!(success.is_success());

        let error = FinsResponse::from_bytes(&make_response(0x01, 0x00, &[])).unwrap();
        assert!(!error.is_success());

        let error2 = FinsResponse::from_bytes(&make_response(0x00, 0x01, &[])).unwrap();
        assert!(!error2.is_success());
    }

    #[test]
    fn test_check_error() {
        let success = FinsResponse::from_bytes(&make_response(0x00, 0x00, &[])).unwrap();
        assert!(success.check_error().is_ok());

        let error = FinsResponse::from_bytes(&make_response(0x02, 0x03, &[])).unwrap();
        let err = error.check_error().unwrap_err();
        match err {
            FinsError::PlcError {
                main_code,
                sub_code,
            } => {
                assert_eq!(main_code, 0x02);
                assert_eq!(sub_code, 0x03);
            }
            _ => panic!("Expected PlcError"),
        }
    }

    #[test]
    fn test_check_sid() {
        let response = FinsResponse::from_bytes(&make_response(0x00, 0x00, &[])).unwrap();
        assert!(response.check_sid(0x01).is_ok());
        assert!(response.check_sid(0x02).is_err());
    }

    #[test]
    fn test_to_words() {
        let bytes = make_response(0x00, 0x00, &[0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();
        let words = response.to_words().unwrap();
        assert_eq!(words, vec![0x1234, 0x5678, 0xABCD]);
    }

    #[test]
    fn test_to_words_empty() {
        let bytes = make_response(0x00, 0x00, &[]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();
        let words = response.to_words().unwrap();
        assert!(words.is_empty());
    }

    #[test]
    fn test_to_words_odd_length() {
        let bytes = make_response(0x00, 0x00, &[0x12, 0x34, 0x56]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();
        assert!(response.to_words().is_err());
    }

    #[test]
    fn test_to_bit_true() {
        let bytes = make_response(0x00, 0x00, &[0x01]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();
        assert!(response.to_bit().unwrap());
    }

    #[test]
    fn test_to_bit_false() {
        let bytes = make_response(0x00, 0x00, &[0x00]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();
        assert!(!response.to_bit().unwrap());
    }

    #[test]
    fn test_to_bit_empty() {
        let bytes = make_response(0x00, 0x00, &[]);
        let response = FinsResponse::from_bytes(&bytes).unwrap();
        assert!(response.to_bit().is_err());
    }
}
