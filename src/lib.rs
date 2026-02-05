//! # Omron FINS Protocol Library
//!
//! A Rust library for communicating with Omron PLCs using the FINS (Factory Interface Network Service) protocol.
//!
//! This is a **protocol-only** library—no business logic, polling, schedulers,
//! or application-level features. Each call produces exactly 1 request and 1 response.
//! No automatic retries, caching, or reconnection.
//!
//! ## Features
//!
//! - **Protocol-only** — focuses solely on FINS protocol implementation
//! - **Deterministic** — each call produces exactly 1 request and 1 response
//! - **Type-safe** — memory areas as enums, compile-time validation
//! - **No panics** — all errors returned as `Result<T, FinsError>`
//! - **Complete API** — read, write, fill, transfer, run/stop, forced set/reset
//! - **Utility functions** — bit manipulation, formatting, and conversion helpers
//!
//! ## Quick Start
//!
//! ```no_run
//! use omron_fins::{Client, ClientConfig, MemoryArea};
//! use std::net::Ipv4Addr;
//!
//! fn main() -> omron_fins::Result<()> {
//!     // Connect to PLC at factory default IP (192.168.1.250)
//!     // Using source_node=1, dest_node=0 (same defaults as Python fins-driver)
//!     let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
//!     let client = Client::new(config)?;
//!
//!     // Read D1 (1 word from DM area)
//!     let data = client.read(MemoryArea::DM, 1, 1)?;
//!     println!("D1 = {:?}", data);
//!
//!     // Read 10 words from DM100
//!     let data = client.read(MemoryArea::DM, 100, 10)?;
//!     println!("DM100-109: {:?}", data);
//!
//!     // Write values to DM200
//!     client.write(MemoryArea::DM, 200, &[0x1234, 0x5678])?;
//!
//!     // Read a single bit from CIO 0.05
//!     let bit = client.read_bit(MemoryArea::CIO, 0, 5)?;
//!     println!("CIO 0.05 = {}", bit);
//!
//!     // Write a single bit
//!     client.write_bit(MemoryArea::CIO, 0, 5, true)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Equivalent Python (fins-driver)
//!
//! This library is compatible with the Python [fins-driver](https://pypi.org/project/fins-driver/) library:
//!
//! ```python
//! from fins import FinsClient
//!
//! client = FinsClient(host='192.168.1.250', port=9600)
//! client.connect()
//! response = client.memory_area_read('D1')
//! print(response.data)
//! client.close()
//! ```
//!
//! ## Memory Areas
//!
//! The library supports the following Omron PLC memory areas:
//!
//! | Area | Description | Word Access | Bit Access |
//! |------|-------------|:-----------:|:----------:|
//! | [`MemoryArea::CIO`] | Core I/O - inputs, outputs, internal relays | ✓ | ✓ |
//! | [`MemoryArea::WR`] | Work area - temporary work bits/words | ✓ | ✓ |
//! | [`MemoryArea::HR`] | Holding area - retentive bits/words | ✓ | ✓ |
//! | [`MemoryArea::DM`] | Data Memory - numeric data storage | ✓ | ✗ |
//! | [`MemoryArea::AR`] | Auxiliary Relay - system status/control | ✓ | ✓ |
//!
//! ## Core Operations
//!
//! ### Word Operations
//!
//! ```no_run
//! # use omron_fins::{Client, ClientConfig, MemoryArea};
//! # use std::net::Ipv4Addr;
//! # let client = Client::new(ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)).unwrap();
//! // Read words
//! let data = client.read(MemoryArea::DM, 100, 10)?;
//!
//! // Write words
//! client.write(MemoryArea::DM, 200, &[0x1234, 0x5678])?;
//!
//! // Fill memory with a value
//! client.fill(MemoryArea::DM, 100, 50, 0x0000)?;
//!
//! // Transfer between areas
//! client.transfer(MemoryArea::DM, 100, MemoryArea::DM, 200, 10)?;
//! # Ok::<(), omron_fins::FinsError>(())
//! ```
//!
//! ### Bit Operations
//!
//! ```no_run
//! # use omron_fins::{Client, ClientConfig, MemoryArea};
//! # use std::net::Ipv4Addr;
//! # let client = Client::new(ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)).unwrap();
//! // Read a bit (CIO 0.05)
//! let bit = client.read_bit(MemoryArea::CIO, 0, 5)?;
//!
//! // Write a bit
//! client.write_bit(MemoryArea::CIO, 0, 5, true)?;
//! # Ok::<(), omron_fins::FinsError>(())
//! ```
//!
//! ### Type Helpers
//!
//! Read and write multi-word types directly:
//!
//! ```no_run
//! # use omron_fins::{Client, ClientConfig, MemoryArea};
//! # use std::net::Ipv4Addr;
//! # let client = Client::new(ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)).unwrap();
//! // f32 (REAL) - 2 words
//! let temp: f32 = client.read_f32(MemoryArea::DM, 100)?;
//! client.write_f32(MemoryArea::DM, 100, 3.14159)?;
//!
//! // f64 (LREAL) - 4 words
//! let value: f64 = client.read_f64(MemoryArea::DM, 100)?;
//! client.write_f64(MemoryArea::DM, 100, 3.141592653589793)?;
//!
//! // i32 (DINT) - 2 words
//! let counter: i32 = client.read_i32(MemoryArea::DM, 100)?;
//! client.write_i32(MemoryArea::DM, 100, -123456)?;
//!
//! // String (ASCII) - variable words (2 chars per word)
//! client.write_string(MemoryArea::DM, 200, "PRODUCT-001")?;
//! let code: String = client.read_string(MemoryArea::DM, 200, 6)?;
//! # Ok::<(), omron_fins::FinsError>(())
//! ```
//!
//! ### PLC Control
//!
//! ```no_run
//! # use omron_fins::{Client, ClientConfig, PlcMode};
//! # use std::net::Ipv4Addr;
//! # let client = Client::new(ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)).unwrap();
//! // Put PLC in run mode
//! client.run(PlcMode::Monitor)?;
//!
//! // Stop PLC
//! client.stop()?;
//! # Ok::<(), omron_fins::FinsError>(())
//! ```
//!
//! ## Utility Functions
//!
//! The [`utils`] module provides helper functions for bit manipulation and formatting:
//!
//! ```
//! use omron_fins::utils::{get_bit, set_bit, word_to_bits, format_binary, format_hex};
//!
//! let value: u16 = 0b1010_0101;
//!
//! // Get individual bits
//! assert!(get_bit(value, 0));   // bit 0 is ON
//! assert!(!get_bit(value, 1));  // bit 1 is OFF
//!
//! // Modify bits
//! let modified = set_bit(value, 1, true);
//!
//! // Convert to bit array
//! let bits = word_to_bits(value);
//!
//! // Format for display
//! println!("{}", format_binary(value));  // "0b0000_0000_1010_0101"
//! println!("{}", format_hex(value));     // "0x00A5"
//! ```
//!
//! ## Error Handling
//!
//! All operations return [`Result<T, FinsError>`]. The library never panics in public code.
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
//!     Err(FinsError::Timeout) => println!("Communication timeout"),
//!     Err(FinsError::PlcError { main_code, sub_code }) => {
//!         println!("PLC error: main=0x{:02X}, sub=0x{:02X}", main_code, sub_code);
//!     }
//!     Err(FinsError::InvalidAddressing { reason }) => {
//!         println!("Invalid addressing: {}", reason);
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! # Ok::<(), FinsError>(())
//! ```
//!
//! ## Configuration
//!
//! ```no_run
//! use omron_fins::ClientConfig;
//! use std::net::Ipv4Addr;
//! use std::time::Duration;
//!
//! let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
//!     .with_port(9601)                        // Custom port (default: 9600)
//!     .with_timeout(Duration::from_secs(5))   // Custom timeout (default: 2s)
//!     .with_source_network(1)                 // Source network address
//!     .with_dest_network(2);                  // Destination network address
//! ```
//!
//! ## Design Philosophy
//!
//! This library follows the principle of **determinism over abstraction**:
//!
//! 1. Each operation does exactly what it says
//! 2. No magic or implicit behavior
//! 3. The application has full control over retry, caching, and reconnection
//! 4. Errors are always explicit and descriptive
//!
//! For more details, see the [ARCHITECTURE.md](https://github.com/deviagomendes/omron-fins-rs/blob/main/ARCHITECTURE.md) file.

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
pub mod utils;

// Public re-exports
pub use client::{Client, ClientConfig};
pub use command::{
    Address, FillCommand, ForceSpec, ForcedBit, ForcedSetResetCancelCommand, ForcedSetResetCommand,
    MultiReadSpec, MultipleReadCommand, PlcMode, ReadBitCommand, ReadWordCommand, RunCommand,
    StopCommand, TransferCommand, WriteBitCommand, WriteWordCommand, MAX_WORDS_PER_COMMAND,
};
pub use error::{fins_error_description, FinsError, Result};
pub use header::{FinsHeader, NodeAddress, FINS_HEADER_SIZE};
pub use memory::MemoryArea;
pub use response::FinsResponse;
pub use transport::{UdpTransport, DEFAULT_FINS_PORT, DEFAULT_TIMEOUT, MAX_PACKET_SIZE};
