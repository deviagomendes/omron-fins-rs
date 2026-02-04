//! UDP transport layer for FINS communication.
//!
//! This module provides the [`UdpTransport`] struct which handles low-level
//! UDP communication with Omron PLCs. The transport layer is completely
//! separated from the protocol layer—it only knows about sockets and bytes.
//!
//! # Design
//!
//! The transport layer follows these principles:
//!
//! - **Protocol agnostic** - Handles only byte transmission, no FINS knowledge
//! - **Synchronous** - Blocking send/receive with configurable timeout
//! - **Simple** - One socket, one remote address, no connection pooling
//!
//! # Constants
//!
//! - [`DEFAULT_FINS_PORT`] - Default FINS UDP port (9600)
//! - [`DEFAULT_TIMEOUT`] - Default timeout (2 seconds)
//! - [`MAX_PACKET_SIZE`] - Maximum UDP packet size (2048 bytes)
//!
//! # Example
//!
//! The transport is typically used through the [`Client`](crate::Client) struct,
//! but can be used directly for custom implementations:
//!
//! ```no_run
//! use omron_fins::UdpTransport;
//! use std::time::Duration;
//!
//! let transport = UdpTransport::new(
//!     "192.168.1.10:9600".parse().unwrap(),
//!     Duration::from_secs(2),
//! ).unwrap();
//!
//! // Send a FINS frame and receive response
//! let request = vec![0x80, 0x00, 0x02, /* ... rest of FINS frame */];
//! let response = transport.send_receive(&request);
//! ```

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use crate::error::{FinsError, Result};

/// Default FINS UDP port.
pub const DEFAULT_FINS_PORT: u16 = 9600;

/// Default timeout for UDP operations.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// Maximum UDP packet size for FINS.
pub const MAX_PACKET_SIZE: usize = 2048;

/// UDP transport for FINS communication.
///
/// Handles synchronous UDP communication with configurable timeout.
/// The protocol layer doesn't know about sockets; the socket layer doesn't know FINS.
pub struct UdpTransport {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl UdpTransport {
    /// Creates a new UDP transport connected to the specified PLC address.
    ///
    /// # Arguments
    ///
    /// * `plc_addr` - Socket address of the PLC (IP:port)
    /// * `timeout` - Read/write timeout duration
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the socket cannot be created or configured.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::UdpTransport;
    /// use std::time::Duration;
    ///
    /// let transport = UdpTransport::new(
    ///     "192.168.1.10:9600".parse().unwrap(),
    ///     Duration::from_secs(2),
    /// ).unwrap();
    /// ```
    pub fn new(plc_addr: SocketAddr, timeout: Duration) -> Result<Self> {
        // Bind to any available local port
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        // Connect to the PLC (required for proper FINS communication)
        socket.connect(plc_addr)?;
        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;

        Ok(Self {
            socket,
            remote_addr: plc_addr,
        })
    }

    /// Creates a new UDP transport with the default timeout.
    ///
    /// # Arguments
    ///
    /// * `plc_addr` - Socket address of the PLC (IP:port)
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the socket cannot be created or configured.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::UdpTransport;
    ///
    /// let transport = UdpTransport::with_default_timeout(
    ///     "192.168.1.10:9600".parse().unwrap(),
    /// ).unwrap();
    /// ```
    pub fn with_default_timeout(plc_addr: SocketAddr) -> Result<Self> {
        Self::new(plc_addr, DEFAULT_TIMEOUT)
    }

    /// Sends a FINS frame and receives the response.
    ///
    /// This is a synchronous operation that blocks until a response
    /// is received or the timeout expires.
    ///
    /// # Arguments
    ///
    /// * `data` - FINS frame bytes to send
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The send fails
    /// - The receive times out (`FinsError::Timeout`)
    /// - Other I/O errors occur
    ///
    /// # Example
    ///
    /// ```no_run
    /// use omron_fins::UdpTransport;
    /// use std::time::Duration;
    ///
    /// let transport = UdpTransport::new(
    ///     "192.168.1.10:9600".parse().unwrap(),
    ///     Duration::from_secs(2),
    /// ).unwrap();
    ///
    /// let request = vec![0x80, 0x00, 0x02, /* ... */];
    /// let response = transport.send_receive(&request).unwrap();
    /// ```
    pub fn send_receive(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Send the request (socket is already connected)
        self.socket.send(data)?;

        // Receive the response
        let mut buffer = vec![0u8; MAX_PACKET_SIZE];
        match self.socket.recv(&mut buffer) {
            Ok(size) => {
                buffer.truncate(size);
                Ok(buffer)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(FinsError::Timeout),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Err(FinsError::Timeout),
            Err(e) => Err(FinsError::Io(e)),
        }
    }

    /// Returns the remote PLC address.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Returns a reference to the underlying socket.
    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}

impl std::fmt::Debug for UdpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UdpTransport")
            .field("remote_addr", &self.remote_addr)
            .field("local_addr", &self.socket.local_addr().ok())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_FINS_PORT, 9600);
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(2));
        assert_eq!(MAX_PACKET_SIZE, 2048);
    }

    #[test]
    fn test_transport_creation() {
        // This test only verifies that we can create a transport
        // (actual PLC communication tests require hardware)
        let addr: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        let transport = UdpTransport::new(addr, Duration::from_millis(100));
        assert!(transport.is_ok());

        let transport = transport.unwrap();
        assert_eq!(transport.remote_addr(), addr);
    }

    #[test]
    fn test_transport_with_default_timeout() {
        let addr: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        let transport = UdpTransport::with_default_timeout(addr);
        assert!(transport.is_ok());
    }

    #[test]
    fn test_transport_debug() {
        let addr: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        let transport = UdpTransport::new(addr, Duration::from_millis(100)).unwrap();
        let debug_str = format!("{:?}", transport);
        assert!(debug_str.contains("UdpTransport"));
        assert!(debug_str.contains("127.0.0.1:9600"));
    }
}
