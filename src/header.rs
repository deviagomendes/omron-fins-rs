//! FINS header structures and node addressing.
//!
//! This module defines the FINS protocol header structure and node addressing
//! used for routing FINS frames between nodes on a FINS network.
//!
//! # FINS Header Structure
//!
//! The FINS header is a 10-byte structure that precedes every FINS command and response:
//!
//! | Byte | Field | Description |
//! |------|-------|-------------|
//! | 0 | ICF | Information Control Field |
//! | 1 | RSV | Reserved (always 0x00) |
//! | 2 | GCT | Gateway Count |
//! | 3 | DNA | Destination Network Address |
//! | 4 | DA1 | Destination Node Address |
//! | 5 | DA2 | Destination Unit Address |
//! | 6 | SNA | Source Network Address |
//! | 7 | SA1 | Source Node Address |
//! | 8 | SA2 | Source Unit Address |
//! | 9 | SID | Service ID |
//!
//! # Node Addressing
//!
//! Each node in a FINS network is identified by three components:
//!
//! - **Network** (0-127): Network number (0 = local network)
//! - **Node** (0-255): Node number within the network
//! - **Unit** (0-255): Unit number within the node (0 = CPU unit)
//!
//! # Example
//!
//! ```
//! use omron_fins::{FinsHeader, NodeAddress};
//!
//! // Create node addresses
//! let source = NodeAddress::new(0, 1, 0);      // Local network, node 1, CPU
//! let destination = NodeAddress::new(0, 10, 0); // Local network, node 10, CPU
//!
//! // Create a command header
//! let header = FinsHeader::new_command(destination, source, 0x01);
//! let bytes = header.to_bytes();
//! assert_eq!(bytes.len(), 10);
//! ```

use crate::error::{FinsError, Result};

/// FINS header size in bytes.
pub const FINS_HEADER_SIZE: usize = 10;

/// Node address for FINS communication.
///
/// Represents a network/node/unit address in the FINS protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeAddress {
    /// Network address (0 = local network).
    pub network: u8,
    /// Node address (0 = local node for destination, or source node number).
    pub node: u8,
    /// Unit address (0 = CPU unit).
    pub unit: u8,
}

impl NodeAddress {
    /// Creates a new node address.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::NodeAddress;
    ///
    /// // Local CPU unit
    /// let local = NodeAddress::new(0, 0, 0);
    ///
    /// // Remote PLC on network 1, node 10, CPU unit
    /// let remote = NodeAddress::new(1, 10, 0);
    /// ```
    pub fn new(network: u8, node: u8, unit: u8) -> Self {
        Self {
            network,
            node,
            unit,
        }
    }

    /// Creates a local node address (network 0, node 0, unit 0).
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::NodeAddress;
    ///
    /// let local = NodeAddress::local();
    /// assert_eq!(local.network, 0);
    /// assert_eq!(local.node, 0);
    /// assert_eq!(local.unit, 0);
    /// ```
    pub fn local() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Default for NodeAddress {
    fn default() -> Self {
        Self::local()
    }
}

/// FINS command/response header (10 bytes).
///
/// The header contains addressing and control information for FINS frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinsHeader {
    /// Information Control Field.
    /// - Bit 7: 1 = response required (command), 0 = response not required
    /// - Bit 6: 0 = command, 1 = response
    /// - For commands: typically 0x80
    /// - For responses: typically 0xC0
    pub icf: u8,
    /// Reserved byte (always 0x00).
    pub rsv: u8,
    /// Gateway Count (number of bridges to pass through, typically 0x02).
    pub gct: u8,
    /// Destination Network Address.
    pub dna: u8,
    /// Destination Node Address.
    pub da1: u8,
    /// Destination Unit Address.
    pub da2: u8,
    /// Source Network Address.
    pub sna: u8,
    /// Source Node Address.
    pub sa1: u8,
    /// Source Unit Address.
    pub sa2: u8,
    /// Service ID (used to match responses with requests).
    pub sid: u8,
}

impl FinsHeader {
    /// Creates a new command header.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{FinsHeader, NodeAddress};
    ///
    /// let dest = NodeAddress::new(0, 10, 0);
    /// let src = NodeAddress::new(0, 1, 0);
    /// let header = FinsHeader::new_command(dest, src, 0x01);
    /// ```
    pub fn new_command(destination: NodeAddress, source: NodeAddress, sid: u8) -> Self {
        Self {
            icf: 0x80, // Command, response required
            rsv: 0x00,
            gct: 0x07, // Gateway count (max hops allowed)
            dna: destination.network,
            da1: destination.node,
            da2: destination.unit,
            sna: source.network,
            sa1: source.node,
            sa2: source.unit,
            sid,
        }
    }

    /// Serializes the header to bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{FinsHeader, NodeAddress};
    ///
    /// let header = FinsHeader::new_command(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01
    /// );
    /// let bytes = header.to_bytes();
    /// assert_eq!(bytes.len(), 10);
    /// ```
    pub fn to_bytes(self) -> [u8; FINS_HEADER_SIZE] {
        [
            self.icf, self.rsv, self.gct, self.dna, self.da1, self.da2, self.sna, self.sa1,
            self.sa2, self.sid,
        ]
    }

    /// Parses a header from bytes.
    ///
    /// # Errors
    ///
    /// Returns `FinsError::InvalidResponse` if the slice is too short.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::FinsHeader;
    ///
    /// let bytes = [0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01];
    /// let header = FinsHeader::from_bytes(&bytes).unwrap();
    /// assert_eq!(header.icf, 0xC0);
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < FINS_HEADER_SIZE {
            return Err(FinsError::invalid_response(format!(
                "header too short: expected {} bytes, got {}",
                FINS_HEADER_SIZE,
                data.len()
            )));
        }

        Ok(Self {
            icf: data[0],
            rsv: data[1],
            gct: data[2],
            dna: data[3],
            da1: data[4],
            da2: data[5],
            sna: data[6],
            sa1: data[7],
            sa2: data[8],
            sid: data[9],
        })
    }

    /// Returns whether this is a response header.
    pub fn is_response(self) -> bool {
        (self.icf & 0x40) != 0
    }

    /// Returns the destination node address.
    pub fn destination(self) -> NodeAddress {
        NodeAddress::new(self.dna, self.da1, self.da2)
    }

    /// Returns the source node address.
    pub fn source(self) -> NodeAddress {
        NodeAddress::new(self.sna, self.sa1, self.sa2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_address_new() {
        let addr = NodeAddress::new(1, 10, 0);
        assert_eq!(addr.network, 1);
        assert_eq!(addr.node, 10);
        assert_eq!(addr.unit, 0);
    }

    #[test]
    fn test_node_address_local() {
        let addr = NodeAddress::local();
        assert_eq!(addr.network, 0);
        assert_eq!(addr.node, 0);
        assert_eq!(addr.unit, 0);
    }

    #[test]
    fn test_header_new_command() {
        let dest = NodeAddress::new(0, 10, 0);
        let src = NodeAddress::new(0, 1, 0);
        let header = FinsHeader::new_command(dest, src, 0x42);

        assert_eq!(header.icf, 0x80);
        assert_eq!(header.rsv, 0x00);
        assert_eq!(header.gct, 0x07);
        assert_eq!(header.dna, 0);
        assert_eq!(header.da1, 10);
        assert_eq!(header.da2, 0);
        assert_eq!(header.sna, 0);
        assert_eq!(header.sa1, 1);
        assert_eq!(header.sa2, 0);
        assert_eq!(header.sid, 0x42);
    }

    #[test]
    fn test_header_to_bytes() {
        let dest = NodeAddress::new(0, 10, 0);
        let src = NodeAddress::new(0, 1, 0);
        let header = FinsHeader::new_command(dest, src, 0x01);
        let bytes = header.to_bytes();

        assert_eq!(
            bytes,
            [0x80, 0x00, 0x07, 0x00, 0x0A, 0x00, 0x00, 0x01, 0x00, 0x01]
        );
    }

    #[test]
    fn test_header_from_bytes() {
        let bytes = [0xC0, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x01];
        let header = FinsHeader::from_bytes(&bytes).unwrap();

        assert_eq!(header.icf, 0xC0);
        assert_eq!(header.rsv, 0x00);
        assert_eq!(header.gct, 0x02);
        assert_eq!(header.dna, 0);
        assert_eq!(header.da1, 1);
        assert_eq!(header.da2, 0);
        assert_eq!(header.sna, 0);
        assert_eq!(header.sa1, 10);
        assert_eq!(header.sa2, 0);
        assert_eq!(header.sid, 0x01);
    }

    #[test]
    fn test_header_from_bytes_too_short() {
        let bytes = [0xC0, 0x00, 0x02];
        let result = FinsHeader::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_is_response() {
        let command_header = FinsHeader {
            icf: 0x80,
            rsv: 0,
            gct: 2,
            dna: 0,
            da1: 10,
            da2: 0,
            sna: 0,
            sa1: 1,
            sa2: 0,
            sid: 1,
        };
        assert!(!command_header.is_response());

        let response_header = FinsHeader {
            icf: 0xC0,
            ..command_header
        };
        assert!(response_header.is_response());
    }

    #[test]
    fn test_header_roundtrip() {
        let original =
            FinsHeader::new_command(NodeAddress::new(1, 20, 0), NodeAddress::new(2, 30, 0), 0xFF);
        let bytes = original.to_bytes();
        let parsed = FinsHeader::from_bytes(&bytes).unwrap();
        assert_eq!(original, parsed);
    }
}
