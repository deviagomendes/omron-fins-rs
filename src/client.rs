//! High-level FINS client for communicating with Omron PLCs.
//!
//! This module provides the [`Client`] struct, which is the primary interface
//! for communicating with Omron PLCs using the FINS protocol.
//!
//! # Overview
//!
//! The client provides a high-level API that handles:
//! - Command construction and serialization
//! - Request/response correlation via Service ID
//! - Response parsing and error checking
//! - Type conversion helpers (f32, f64, i32)
//!
//! # Example
//!
//! ```no_run
//! use omron_fins::{Client, ClientConfig, MemoryArea};
//! use std::net::Ipv4Addr;
//!
//! // Create and configure the client
//! let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);
//! let client = Client::new(config)?;
//!
//! // Read data
//! let data = client.read(MemoryArea::DM, 100, 10)?;
//!
//! // Write data
//! client.write(MemoryArea::DM, 200, &[0x1234, 0x5678])?;
//!
//! // Read/write bits
//! let bit = client.read_bit(MemoryArea::CIO, 0, 5)?;
//! client.write_bit(MemoryArea::CIO, 0, 5, true)?;
//!
//! // Read/write typed values
//! let temp: f32 = client.read_f32(MemoryArea::DM, 100)?;
//! client.write_f32(MemoryArea::DM, 100, 25.5)?;
//! # Ok::<(), omron_fins::FinsError>(())
//! ```
//!
//! # Configuration
//!
//! The [`ClientConfig`] struct allows customization of:
//! - PLC IP address and port
//! - Communication timeout
//! - Source and destination node addresses
//! - Network addressing for multi-network setups
//!
//! # Thread Safety
//!
//! The `Client` uses an atomic counter for Service IDs, making it safe to share
//! between threads. However, the underlying UDP socket operations are synchronous
//! and will block.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use crate::command::{
    FillCommand, ForcedBit, ForcedSetResetCancelCommand, ForcedSetResetCommand, MultiReadSpec,
    MultipleReadCommand, PlcMode, ReadBitCommand, ReadWordCommand, RunCommand, StopCommand,
    TransferCommand, WriteBitCommand, WriteWordCommand,
};
use crate::error::Result;
use crate::header::NodeAddress;
use crate::memory::MemoryArea;
use crate::response::FinsResponse;
use crate::transport::{UdpTransport, DEFAULT_FINS_PORT, DEFAULT_TIMEOUT};

/// Configuration for creating a FINS client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// PLC IP address or hostname.
    pub plc_addr: SocketAddr,
    /// Source node address (this client).
    pub source: NodeAddress,
    /// Destination node address (the PLC).
    pub destination: NodeAddress,
    /// Communication timeout.
    pub timeout: Duration,
}

impl ClientConfig {
    /// Creates a new client configuration with minimal required parameters.
    ///
    /// Uses default timeout and local node addresses.
    ///
    /// # Arguments
    ///
    /// * `plc_ip` - PLC IP address (port defaults to 9600)
    /// * `source_node` - Source node number (this client)
    /// * `dest_node` - Destination node number (the PLC)
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::ClientConfig;
    /// use std::net::Ipv4Addr;
    ///
    /// let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);
    /// ```
    pub fn new(plc_ip: std::net::Ipv4Addr, source_node: u8, dest_node: u8) -> Self {
        Self {
            plc_addr: SocketAddr::from((plc_ip, DEFAULT_FINS_PORT)),
            source: NodeAddress::new(0, source_node, 0),
            destination: NodeAddress::new(0, dest_node, 0),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Sets a custom PLC port (default is 9600).
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::ClientConfig;
    /// use std::net::Ipv4Addr;
    ///
    /// let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    ///     .with_port(9601);
    /// ```
    pub fn with_port(mut self, port: u16) -> Self {
        self.plc_addr.set_port(port);
        self
    }

    /// Sets a custom timeout (default is 2 seconds).
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::ClientConfig;
    /// use std::net::Ipv4Addr;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    ///     .with_timeout(Duration::from_secs(5));
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets custom source network/unit addresses.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::ClientConfig;
    /// use std::net::Ipv4Addr;
    ///
    /// let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    ///     .with_source_network(1)
    ///     .with_source_unit(0);
    /// ```
    pub fn with_source_network(mut self, network: u8) -> Self {
        self.source.network = network;
        self
    }

    /// Sets custom source unit address.
    pub fn with_source_unit(mut self, unit: u8) -> Self {
        self.source.unit = unit;
        self
    }

    /// Sets custom destination network/unit addresses.
    pub fn with_dest_network(mut self, network: u8) -> Self {
        self.destination.network = network;
        self
    }

    /// Sets custom destination unit address.
    pub fn with_dest_unit(mut self, unit: u8) -> Self {
        self.destination.unit = unit;
        self
    }
}

/// FINS client for communicating with Omron PLCs.
///
/// Provides a simple API for reading and writing PLC memory.
/// Each operation produces exactly 1 request and 1 response.
/// No automatic retries, caching, or reconnection.
///
/// # Example
///
/// ```no_run
/// use omron_fins::{Client, ClientConfig, MemoryArea};
/// use std::net::Ipv4Addr;
///
/// let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);
/// let client = Client::new(config).unwrap();
///
/// // Read 10 words from DM100
/// let data = client.read(MemoryArea::DM, 100, 10).unwrap();
///
/// // Write values to DM200
/// client.write(MemoryArea::DM, 200, &[0x1234, 0x5678]).unwrap();
///
/// // Read a single bit
/// let bit = client.read_bit(MemoryArea::CIO, 0, 5).unwrap();
///
/// // Write a single bit
/// client.write_bit(MemoryArea::CIO, 0, 5, true).unwrap();
/// ```
pub struct Client {
    transport: UdpTransport,
    source: NodeAddress,
    destination: NodeAddress,
    sid_counter: AtomicU8,
}

impl Client {
    /// Creates a new FINS client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the UDP transport cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig};
    /// use std::net::Ipv4Addr;
    ///
    /// let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);
    /// let client = Client::new(config).unwrap();
    /// ```
    pub fn new(config: ClientConfig) -> Result<Self> {
        let transport = UdpTransport::new(config.plc_addr, config.timeout)?;

        Ok(Self {
            transport,
            source: config.source,
            destination: config.destination,
            sid_counter: AtomicU8::new(0),
        })
    }

    /// Generates the next Service ID.
    fn next_sid(&self) -> u8 {
        self.sid_counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Reads words from PLC memory.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to read from
    /// * `address` - Starting word address
    /// * `count` - Number of words to read (1-999)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Count is 0 or > 999
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// let data = client.read(MemoryArea::DM, 100, 10).unwrap();
    /// println!("Read {} words: {:?}", data.len(), data);
    /// ```
    pub fn read(&self, area: MemoryArea, address: u16, count: u16) -> Result<Vec<u16>> {
        let sid = self.next_sid();
        let cmd = ReadWordCommand::new(self.destination, self.source, sid, area, address, count)?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        response.to_words()
    }

    /// Writes words to PLC memory.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to write to
    /// * `address` - Starting word address
    /// * `data` - Words to write (1-999 words)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Data is empty or > 999 words
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.write(MemoryArea::DM, 100, &[0x1234, 0x5678]).unwrap();
    /// ```
    pub fn write(&self, area: MemoryArea, address: u16, data: &[u16]) -> Result<()> {
        let sid = self.next_sid();
        let cmd = WriteWordCommand::new(self.destination, self.source, sid, area, address, data)?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Reads a single bit from PLC memory.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to read from (must support bit access)
    /// * `address` - Word address
    /// * `bit` - Bit position (0-15)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Area doesn't support bit access (DM)
    /// - Bit position > 15
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// let bit = client.read_bit(MemoryArea::CIO, 0, 5).unwrap();
    /// println!("CIO 0.05 = {}", bit);
    /// ```
    pub fn read_bit(&self, area: MemoryArea, address: u16, bit: u8) -> Result<bool> {
        let sid = self.next_sid();
        let cmd = ReadBitCommand::new(self.destination, self.source, sid, area, address, bit)?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes()?)?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        response.to_bit()
    }

    /// Writes a single bit to PLC memory.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to write to (must support bit access)
    /// * `address` - Word address
    /// * `bit` - Bit position (0-15)
    /// * `value` - Bit value to write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Area doesn't support bit access (DM)
    /// - Bit position > 15
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.write_bit(MemoryArea::CIO, 0, 5, true).unwrap();
    /// ```
    pub fn write_bit(&self, area: MemoryArea, address: u16, bit: u8, value: bool) -> Result<()> {
        let sid = self.next_sid();
        let cmd = WriteBitCommand::new(
            self.destination,
            self.source,
            sid,
            area,
            address,
            bit,
            value,
        )?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes()?)?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Fills a memory area with a single value.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to fill
    /// * `address` - Starting word address
    /// * `count` - Number of words to fill (1-999)
    /// * `value` - Value to fill with
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Count is 0 or > 999
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// // Zero out DM100-DM149
    /// client.fill(MemoryArea::DM, 100, 50, 0x0000).unwrap();
    /// ```
    pub fn fill(&self, area: MemoryArea, address: u16, count: u16, value: u16) -> Result<()> {
        let sid = self.next_sid();
        let cmd = FillCommand::new(
            self.destination,
            self.source,
            sid,
            area,
            address,
            count,
            value,
        )?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Puts the PLC into run mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - PLC operating mode (Debug, Monitor, or Run)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, PlcMode};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.run(PlcMode::Monitor).unwrap();
    /// ```
    pub fn run(&self, mode: PlcMode) -> Result<()> {
        let sid = self.next_sid();
        let cmd = RunCommand::new(self.destination, self.source, sid, mode);

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Stops the PLC.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.stop().unwrap();
    /// ```
    pub fn stop(&self) -> Result<()> {
        let sid = self.next_sid();
        let cmd = StopCommand::new(self.destination, self.source, sid);

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Transfers data from one memory area to another within the PLC.
    ///
    /// # Arguments
    ///
    /// * `src_area` - Source memory area
    /// * `src_address` - Source starting address
    /// * `dst_area` - Destination memory area
    /// * `dst_address` - Destination starting address
    /// * `count` - Number of words to transfer (1-999)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Count is 0 or > 999
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// // Copy DM100-DM109 to DM200-DM209
    /// client.transfer(MemoryArea::DM, 100, MemoryArea::DM, 200, 10).unwrap();
    /// ```
    pub fn transfer(
        &self,
        src_area: MemoryArea,
        src_address: u16,
        dst_area: MemoryArea,
        dst_address: u16,
        count: u16,
    ) -> Result<()> {
        let sid = self.next_sid();
        let cmd = TransferCommand::new(
            self.destination,
            self.source,
            sid,
            src_area,
            src_address,
            dst_area,
            dst_address,
            count,
        )?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Forces bits ON/OFF in the PLC, overriding normal program control.
    ///
    /// # Arguments
    ///
    /// * `specs` - List of bits to force with their specifications
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Specs is empty
    /// - Any area doesn't support bit access
    /// - Any bit position > 15
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, ForcedBit, ForceSpec, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.forced_set_reset(&[
    ///     ForcedBit { area: MemoryArea::CIO, address: 0, bit: 0, spec: ForceSpec::ForceOn },
    ///     ForcedBit { area: MemoryArea::CIO, address: 0, bit: 1, spec: ForceSpec::ForceOff },
    /// ]).unwrap();
    /// ```
    pub fn forced_set_reset(&self, specs: &[ForcedBit]) -> Result<()> {
        let sid = self.next_sid();
        let cmd = ForcedSetResetCommand::new(self.destination, self.source, sid, specs.to_vec())?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes()?)?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Cancels all forced bits in the PLC.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.forced_set_reset_cancel().unwrap();
    /// ```
    pub fn forced_set_reset_cancel(&self) -> Result<()> {
        let sid = self.next_sid();
        let cmd = ForcedSetResetCancelCommand::new(self.destination, self.source, sid);

        let response_bytes = self.transport.send_receive(&cmd.to_bytes())?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        Ok(())
    }

    /// Reads from multiple memory areas in a single request.
    ///
    /// # Arguments
    ///
    /// * `specs` - List of read specifications
    ///
    /// # Returns
    ///
    /// A vector of u16 values in the same order as the specs.
    /// For word reads, the full u16 value is returned.
    /// For bit reads, 0x0000 (OFF) or 0x0001 (ON) is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Specs is empty
    /// - Any bit area doesn't support bit access
    /// - Any bit position > 15
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MultiReadSpec, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// let values = client.read_multiple(&[
    ///     MultiReadSpec { area: MemoryArea::DM, address: 100, bit: None },
    ///     MultiReadSpec { area: MemoryArea::DM, address: 200, bit: None },
    ///     MultiReadSpec { area: MemoryArea::CIO, address: 0, bit: Some(5) },
    /// ]).unwrap();
    /// // values[0] = DM100, values[1] = DM200, values[2] = CIO0.05 (0 or 1)
    /// ```
    pub fn read_multiple(&self, specs: &[MultiReadSpec]) -> Result<Vec<u16>> {
        let sid = self.next_sid();
        let cmd = MultipleReadCommand::new(self.destination, self.source, sid, specs.to_vec())?;

        let response_bytes = self.transport.send_receive(&cmd.to_bytes()?)?;
        let response = FinsResponse::from_bytes(&response_bytes)?;
        response.check_sid(sid)?;
        response.check_error()?;
        response.to_words()
    }

    /// Reads an f32 (REAL) value from 2 consecutive words.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to read from
    /// * `address` - Starting word address
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// let temperature: f32 = client.read_f32(MemoryArea::DM, 100).unwrap();
    /// ```
    pub fn read_f32(&self, area: MemoryArea, address: u16) -> Result<f32> {
        let words = self.read(area, address, 2)?;
        let bytes = [
            (words[0] >> 8) as u8,
            (words[0] & 0xFF) as u8,
            (words[1] >> 8) as u8,
            (words[1] & 0xFF) as u8,
        ];
        Ok(f32::from_be_bytes(bytes))
    }

    /// Writes an f32 (REAL) value to 2 consecutive words.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to write to
    /// * `address` - Starting word address
    /// * `value` - f32 value to write
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.write_f32(MemoryArea::DM, 100, 3.14159).unwrap();
    /// ```
    pub fn write_f32(&self, area: MemoryArea, address: u16, value: f32) -> Result<()> {
        let bytes = value.to_be_bytes();
        let words = [
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ];
        self.write(area, address, &words)
    }

    /// Reads an f64 (LREAL) value from 4 consecutive words.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to read from
    /// * `address` - Starting word address
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// let value: f64 = client.read_f64(MemoryArea::DM, 100).unwrap();
    /// ```
    pub fn read_f64(&self, area: MemoryArea, address: u16) -> Result<f64> {
        let words = self.read(area, address, 4)?;
        let bytes = [
            (words[0] >> 8) as u8,
            (words[0] & 0xFF) as u8,
            (words[1] >> 8) as u8,
            (words[1] & 0xFF) as u8,
            (words[2] >> 8) as u8,
            (words[2] & 0xFF) as u8,
            (words[3] >> 8) as u8,
            (words[3] & 0xFF) as u8,
        ];
        Ok(f64::from_be_bytes(bytes))
    }

    /// Writes an f64 (LREAL) value to 4 consecutive words.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to write to
    /// * `address` - Starting word address
    /// * `value` - f64 value to write
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.write_f64(MemoryArea::DM, 100, 3.141592653589793).unwrap();
    /// ```
    pub fn write_f64(&self, area: MemoryArea, address: u16, value: f64) -> Result<()> {
        let bytes = value.to_be_bytes();
        let words = [
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]),
        ];
        self.write(area, address, &words)
    }

    /// Reads an i32 (DINT) value from 2 consecutive words.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to read from
    /// * `address` - Starting word address
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// let counter: i32 = client.read_i32(MemoryArea::DM, 100).unwrap();
    /// ```
    pub fn read_i32(&self, area: MemoryArea, address: u16) -> Result<i32> {
        let words = self.read(area, address, 2)?;
        let bytes = [
            (words[0] >> 8) as u8,
            (words[0] & 0xFF) as u8,
            (words[1] >> 8) as u8,
            (words[1] & 0xFF) as u8,
        ];
        Ok(i32::from_be_bytes(bytes))
    }

    /// Writes an i32 (DINT) value to 2 consecutive words.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to write to
    /// * `address` - Starting word address
    /// * `value` - i32 value to write
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or PLC returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// client.write_i32(MemoryArea::DM, 100, -123456).unwrap();
    /// ```
    pub fn write_i32(&self, area: MemoryArea, address: u16, value: i32) -> Result<()> {
        let bytes = value.to_be_bytes();
        let words = [
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ];
        self.write(area, address, &words)
    }

    /// Writes an ASCII string to consecutive words.
    ///
    /// Each word stores 2 ASCII characters (big-endian). If the string has an
    /// odd number of characters, the last byte is padded with 0x00.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to write to
    /// * `address` - Starting word address
    /// * `value` - String to write (ASCII only)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - String is empty
    /// - String exceeds 1998 characters (999 words)
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// // Write a product code to DM100
    /// client.write_string(MemoryArea::DM, 100, "PRODUCT-001").unwrap();
    /// ```
    pub fn write_string(&self, area: MemoryArea, address: u16, value: &str) -> Result<()> {
        use crate::command::MAX_WORDS_PER_COMMAND;
        use crate::error::FinsError;

        if value.is_empty() {
            return Err(FinsError::InvalidParameter {
                parameter: "value".to_string(),
                reason: "string cannot be empty".to_string(),
            });
        }

        let bytes = value.as_bytes();
        let word_count = (bytes.len() + 1) / 2;

        if word_count > MAX_WORDS_PER_COMMAND as usize {
            return Err(FinsError::InvalidParameter {
                parameter: "value".to_string(),
                reason: format!(
                    "string too long: {} bytes requires {} words, max is {}",
                    bytes.len(),
                    word_count,
                    MAX_WORDS_PER_COMMAND
                ),
            });
        }

        let words: Vec<u16> = bytes
            .chunks(2)
            .map(|chunk| {
                let high = chunk[0] as u16;
                let low = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                (high << 8) | low
            })
            .collect();

        self.write(area, address, &words)
    }

    /// Reads an ASCII string from consecutive words.
    ///
    /// Each word contains 2 ASCII characters (big-endian). Null bytes (0x00)
    /// at the end of the string are trimmed.
    ///
    /// # Arguments
    ///
    /// * `area` - Memory area to read from
    /// * `address` - Starting word address
    /// * `word_count` - Number of words to read (1-999)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Word count is 0 or > 999
    /// - Communication fails
    /// - PLC returns an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::{Client, ClientConfig, MemoryArea};
    /// use std::net::Ipv4Addr;
    ///
    /// let client = Client::new(ClientConfig::new(
    ///     Ipv4Addr::new(192, 168, 1, 10), 1, 10
    /// )).unwrap();
    ///
    /// // Read a product code from DM100 (up to 20 characters = 10 words)
    /// let code = client.read_string(MemoryArea::DM, 100, 10).unwrap();
    /// println!("Product code: {}", code);
    /// ```
    pub fn read_string(&self, area: MemoryArea, address: u16, word_count: u16) -> Result<String> {
        let words = self.read(area, address, word_count)?;

        let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
        for word in &words {
            bytes.push((word >> 8) as u8);
            bytes.push((word & 0xFF) as u8);
        }

        // Trim null bytes from the end
        while bytes.last() == Some(&0) {
            bytes.pop();
        }

        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Returns the source node address.
    pub fn source(&self) -> NodeAddress {
        self.source
    }

    /// Returns the destination node address.
    pub fn destination(&self) -> NodeAddress {
        self.destination
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("transport", &self.transport)
            .field("source", &self.source)
            .field("destination", &self.destination)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_client_config_new() {
        let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);

        assert_eq!(config.plc_addr.ip(), Ipv4Addr::new(192, 168, 1, 10));
        assert_eq!(config.plc_addr.port(), DEFAULT_FINS_PORT);
        assert_eq!(config.source.node, 1);
        assert_eq!(config.destination.node, 10);
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_client_config_with_port() {
        let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10).with_port(9601);

        assert_eq!(config.plc_addr.port(), 9601);
    }

    #[test]
    fn test_client_config_with_timeout() {
        let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
            .with_timeout(Duration::from_secs(5));

        assert_eq!(config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_client_config_with_network() {
        let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
            .with_source_network(1)
            .with_dest_network(2);

        assert_eq!(config.source.network, 1);
        assert_eq!(config.destination.network, 2);
    }

    #[test]
    fn test_client_creation() {
        // Note: This creates a socket but doesn't actually connect to a PLC
        let config = ClientConfig::new(Ipv4Addr::new(127, 0, 0, 1), 1, 10);
        let client = Client::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_sid_increment() {
        let config = ClientConfig::new(Ipv4Addr::new(127, 0, 0, 1), 1, 10);
        let client = Client::new(config).unwrap();

        assert_eq!(client.next_sid(), 0);
        assert_eq!(client.next_sid(), 1);
        assert_eq!(client.next_sid(), 2);
    }

    #[test]
    fn test_client_debug() {
        let config = ClientConfig::new(Ipv4Addr::new(127, 0, 0, 1), 1, 10);
        let client = Client::new(config).unwrap();
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("Client"));
    }

    #[test]
    fn test_string_to_words_even_length() {
        // "Hi" = [0x48, 0x69] -> [0x4869]
        let s = "Hi";
        let bytes = s.as_bytes();
        let words: Vec<u16> = bytes
            .chunks(2)
            .map(|chunk| {
                let high = chunk[0] as u16;
                let low = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                (high << 8) | low
            })
            .collect();
        assert_eq!(words, vec![0x4869]);
    }

    #[test]
    fn test_string_to_words_odd_length() {
        // "Hello" = [0x48, 0x65, 0x6C, 0x6C, 0x6F] -> [0x4865, 0x6C6C, 0x6F00]
        let s = "Hello";
        let bytes = s.as_bytes();
        let words: Vec<u16> = bytes
            .chunks(2)
            .map(|chunk| {
                let high = chunk[0] as u16;
                let low = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                (high << 8) | low
            })
            .collect();
        assert_eq!(words, vec![0x4865, 0x6C6C, 0x6F00]);
    }

    #[test]
    fn test_words_to_string() {
        // [0x4865, 0x6C6C, 0x6F00] -> "Hello"
        let words = vec![0x4865u16, 0x6C6C, 0x6F00];
        let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
        for word in &words {
            bytes.push((word >> 8) as u8);
            bytes.push((word & 0xFF) as u8);
        }
        while bytes.last() == Some(&0) {
            bytes.pop();
        }
        let result = String::from_utf8_lossy(&bytes).to_string();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_words_to_string_no_null() {
        // [0x4869] -> "Hi" (no null padding)
        let words = vec![0x4869u16];
        let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
        for word in &words {
            bytes.push((word >> 8) as u8);
            bytes.push((word & 0xFF) as u8);
        }
        while bytes.last() == Some(&0) {
            bytes.pop();
        }
        let result = String::from_utf8_lossy(&bytes).to_string();
        assert_eq!(result, "Hi");
    }

    #[test]
    fn test_string_roundtrip() {
        // Test that string -> words -> string preserves the original
        let original = "PRODUCT-001";
        let bytes = original.as_bytes();
        let words: Vec<u16> = bytes
            .chunks(2)
            .map(|chunk| {
                let high = chunk[0] as u16;
                let low = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                (high << 8) | low
            })
            .collect();

        let mut result_bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
        for word in &words {
            result_bytes.push((word >> 8) as u8);
            result_bytes.push((word & 0xFF) as u8);
        }
        while result_bytes.last() == Some(&0) {
            result_bytes.pop();
        }
        let result = String::from_utf8_lossy(&result_bytes).to_string();
        assert_eq!(result, original);
    }
}
