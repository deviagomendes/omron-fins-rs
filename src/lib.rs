//! Omron FINS protocol library for communicating with Omron PLCs.
//!
//! This is a **protocol-only** library—no business logic, polling, schedulers,
//! or application-level features. Each call produces exactly 1 request and 1 response.
//! No automatic retries, caching, or reconnection.
//!
//! # Quick Start
//!
//! ```no_run
//! use omron_fins::{Client, ClientConfig, MemoryArea};
//! use std::net::Ipv4Addr;
//!
//! // Create a client configuration
//! let config = ClientConfig::new(
//!     Ipv4Addr::new(192, 168, 1, 150),  // PLC IP address
//!     1,                                // Source node (this client)
//!     10,                               // Destination node (the PLC)
//! );
//!
//! // Connect to the PLC
//! let client = Client::new(config).unwrap();
//!
//! // Read 10 words from DM100
//! let data = client.read(MemoryArea::DM, 100, 10).unwrap();
//! println!("Read {} words: {:?}", data.len(), data);
//!
//! // Write values to DM200
//! client.write(MemoryArea::DM, 200, &[0x1234, 0x5678]).unwrap();
//!
//! // Read a single bit from CIO 0.05
//! let bit = client.read_bit(MemoryArea::CIO, 0, 5).unwrap();
//! println!("CIO 0.05 = {}", bit);
//!
//! // Write a single bit
//! client.write_bit(MemoryArea::CIO, 0, 5, true).unwrap();
//! ```
//!
//! # Memory Areas
//!
//! The library supports the following memory areas:
//!
//! - [`MemoryArea::CIO`] - Core I/O area (word and bit access)
//! - [`MemoryArea::WR`] - Work area (word and bit access)
//! - [`MemoryArea::HR`] - Holding area (word and bit access)
//! - [`MemoryArea::DM`] - Data Memory area (word access only)
//!
//! # Error Handling
//!
//! All operations return [`Result<T, FinsError>`]. The library never panics in public code.
//!
//! ```no_run
//! use omron_fins::{Client, ClientConfig, MemoryArea, FinsError};
//! use std::net::Ipv4Addr;
//!
//! let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);
//! let client = Client::new(config).unwrap();
//!
//! match client.read(MemoryArea::DM, 100, 10) {
//!     Ok(data) => println!("Data: {:?}", data),
//!     Err(FinsError::Timeout) => println!("Communication timeout"),
//!     Err(FinsError::PlcError { main_code, sub_code }) => {
//!         println!("PLC error: main=0x{:02X}, sub=0x{:02X}", main_code, sub_code);
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```

#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod client;
mod command;
mod error;
mod header;
mod memory;
mod response;
mod transport;

// Public re-exports
pub use client::{Client, ClientConfig};
pub use command::{Address, ReadBitCommand, ReadWordCommand, WriteBitCommand, WriteWordCommand};
pub use error::{FinsError, Result};
pub use header::{FinsHeader, NodeAddress, FINS_HEADER_SIZE};
pub use memory::MemoryArea;
pub use response::FinsResponse;
pub use transport::{UdpTransport, DEFAULT_FINS_PORT, DEFAULT_TIMEOUT, MAX_PACKET_SIZE};
