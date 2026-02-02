//! FINS client for communicating with Omron PLCs.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use crate::command::{ReadBitCommand, ReadWordCommand, WriteBitCommand, WriteWordCommand};
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
}
