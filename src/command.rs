//! FINS command structures and serialization.
//!
//! This module contains all FINS command structures that can be sent to a PLC.
//! Each command handles its own serialization to bytes for transmission.
//!
//! # Command Types
//!
//! The module provides the following command types:
//!
//! ## Memory Operations
//! - [`ReadWordCommand`] - Read words from PLC memory
//! - [`WriteWordCommand`] - Write words to PLC memory
//! - [`ReadBitCommand`] - Read a single bit from PLC memory
//! - [`WriteBitCommand`] - Write a single bit to PLC memory
//! - [`FillCommand`] - Fill memory with a repeated value
//! - [`TransferCommand`] - Transfer data between memory areas
//! - [`MultipleReadCommand`] - Read from multiple addresses in one request
//!
//! ## PLC Control
//! - [`RunCommand`] - Put PLC into run mode
//! - [`StopCommand`] - Stop the PLC
//!
//! ## Forced I/O
//! - [`ForcedSetResetCommand`] - Force bits ON/OFF
//! - [`ForcedSetResetCancelCommand`] - Cancel all forced bits
//!
//! # Example
//!
//! Commands are typically created and used through the [`Client`](crate::Client) struct,
//! but can also be used directly for lower-level control:
//!
//! ```
//! use omron_fins::{ReadWordCommand, MemoryArea, NodeAddress};
//!
//! let dest = NodeAddress::new(0, 10, 0);
//! let src = NodeAddress::new(0, 1, 0);
//!
//! let cmd = ReadWordCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 10).unwrap();
//! let bytes = cmd.to_bytes();
//! // bytes can now be sent over UDP
//! ```
//!
//! # Constants
//!
//! - [`MAX_WORDS_PER_COMMAND`] - Maximum number of words (999) per read/write command

use crate::error::{FinsError, Result};
use crate::header::{FinsHeader, NodeAddress, FINS_HEADER_SIZE};
use crate::memory::MemoryArea;

/// Memory Read command code (MRC).
pub(crate) const MRC_MEMORY_READ: u8 = 0x01;
/// Memory Read command sub-code (SRC).
pub(crate) const SRC_MEMORY_READ: u8 = 0x01;
/// Memory Write command code (MRC).
pub(crate) const MRC_MEMORY_WRITE: u8 = 0x01;
/// Memory Write command sub-code (SRC).
pub(crate) const SRC_MEMORY_WRITE: u8 = 0x02;
/// Memory Fill command sub-code (SRC).
pub(crate) const SRC_MEMORY_FILL: u8 = 0x03;
/// Multiple Memory Area Read command sub-code (SRC).
pub(crate) const SRC_MULTIPLE_READ: u8 = 0x04;
/// Memory Area Transfer command sub-code (SRC).
pub(crate) const SRC_MEMORY_TRANSFER: u8 = 0x05;
/// Run command code (MRC).
pub(crate) const MRC_RUN: u8 = 0x04;
/// Run command sub-code (SRC).
pub(crate) const SRC_RUN: u8 = 0x01;
/// Stop command sub-code (SRC).
pub(crate) const SRC_STOP: u8 = 0x02;
/// Forced Set/Reset command code (MRC).
pub(crate) const MRC_FORCED: u8 = 0x23;
/// Forced Set/Reset command sub-code (SRC).
pub(crate) const SRC_FORCED_SET_RESET: u8 = 0x01;
/// Forced Set/Reset Cancel command sub-code (SRC).
pub(crate) const SRC_FORCED_CANCEL: u8 = 0x02;

/// Maximum number of words that can be read/written in a single command.
pub const MAX_WORDS_PER_COMMAND: u16 = 999;

/// Address specification for FINS commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    /// Word address in the memory area.
    pub word: u16,
    /// Bit position (0-15) for bit access, or 0 for word access.
    pub bit: u8,
}

impl Address {
    /// Creates a new word address (bit = 0).
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::Address;
    ///
    /// let addr = Address::word(100);
    /// assert_eq!(addr.word, 100);
    /// assert_eq!(addr.bit, 0);
    /// ```
    pub fn word(word: u16) -> Self {
        Self { word, bit: 0 }
    }

    /// Creates a new bit address.
    ///
    /// # Errors
    ///
    /// Returns an error if bit > 15.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::Address;
    ///
    /// let addr = Address::bit(100, 5).unwrap();
    /// assert_eq!(addr.word, 100);
    /// assert_eq!(addr.bit, 5);
    /// ```
    pub fn bit(word: u16, bit: u8) -> Result<Self> {
        if bit > 15 {
            return Err(FinsError::invalid_parameter("bit", "must be 0-15"));
        }
        Ok(Self { word, bit })
    }

    /// Serializes address to 3 bytes (word high, word low, bit).
    pub(crate) fn to_bytes(self) -> [u8; 3] {
        [(self.word >> 8) as u8, (self.word & 0xFF) as u8, self.bit]
    }
}

/// Command for reading words from PLC memory.
#[derive(Debug, Clone)]
pub struct ReadWordCommand {
    header: FinsHeader,
    area: MemoryArea,
    address: Address,
    count: u16,
}

impl ReadWordCommand {
    /// Creates a new read word command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `area` - Memory area to read from
    /// * `address` - Starting word address
    /// * `count` - Number of words to read (1-999)
    ///
    /// # Errors
    ///
    /// Returns an error if count is 0 or exceeds MAX_WORDS_PER_COMMAND.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{ReadWordCommand, MemoryArea, NodeAddress};
    ///
    /// let cmd = ReadWordCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     MemoryArea::DM,
    ///     100,
    ///     10,
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        area: MemoryArea,
        word_address: u16,
        count: u16,
    ) -> Result<Self> {
        if count == 0 {
            return Err(FinsError::invalid_parameter(
                "count",
                "must be greater than 0",
            ));
        }
        if count > MAX_WORDS_PER_COMMAND {
            return Err(FinsError::invalid_parameter(
                "count",
                format!("must not exceed {}", MAX_WORDS_PER_COMMAND),
            ));
        }

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            area,
            address: Address::word(word_address),
            count,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 8);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_READ);
        bytes.push(SRC_MEMORY_READ);
        bytes.push(self.area.word_code());
        bytes.extend_from_slice(&self.address.to_bytes());
        bytes.push((self.count >> 8) as u8);
        bytes.push((self.count & 0xFF) as u8);
        bytes
    }
}

/// Command for writing words to PLC memory.
#[derive(Debug, Clone)]
pub struct WriteWordCommand {
    header: FinsHeader,
    area: MemoryArea,
    address: Address,
    data: Vec<u16>,
}

impl WriteWordCommand {
    /// Creates a new write word command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `area` - Memory area to write to
    /// * `word_address` - Starting word address
    /// * `data` - Words to write (1-999 words)
    ///
    /// # Errors
    ///
    /// Returns an error if data is empty or exceeds MAX_WORDS_PER_COMMAND.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{WriteWordCommand, MemoryArea, NodeAddress};
    ///
    /// let cmd = WriteWordCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     MemoryArea::DM,
    ///     100,
    ///     &[0x1234, 0x5678],
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        area: MemoryArea,
        word_address: u16,
        data: &[u16],
    ) -> Result<Self> {
        if data.is_empty() {
            return Err(FinsError::invalid_parameter("data", "must not be empty"));
        }
        if data.len() > MAX_WORDS_PER_COMMAND as usize {
            return Err(FinsError::invalid_parameter(
                "data",
                format!("must not exceed {} words", MAX_WORDS_PER_COMMAND),
            ));
        }

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            area,
            address: Address::word(word_address),
            data: data.to_vec(),
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 8 + self.data.len() * 2);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_WRITE);
        bytes.push(SRC_MEMORY_WRITE);
        bytes.push(self.area.word_code());
        bytes.extend_from_slice(&self.address.to_bytes());
        bytes.push((self.data.len() >> 8) as u8);
        bytes.push((self.data.len() & 0xFF) as u8);
        for word in &self.data {
            bytes.push((word >> 8) as u8);
            bytes.push((word & 0xFF) as u8);
        }
        bytes
    }
}

/// Command for reading a single bit from PLC memory.
#[derive(Debug, Clone)]
pub struct ReadBitCommand {
    header: FinsHeader,
    area: MemoryArea,
    address: Address,
}

impl ReadBitCommand {
    /// Creates a new read bit command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `area` - Memory area to read from (must support bit access)
    /// * `word_address` - Word address
    /// * `bit` - Bit position (0-15)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The memory area doesn't support bit access (DM)
    /// - The bit position is > 15
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{ReadBitCommand, MemoryArea, NodeAddress};
    ///
    /// let cmd = ReadBitCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     MemoryArea::CIO,
    ///     100,
    ///     5,
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        area: MemoryArea,
        word_address: u16,
        bit: u8,
    ) -> Result<Self> {
        // Validate bit access is supported
        area.bit_code()?;

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            area,
            address: Address::bit(word_address, bit)?,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 8);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_READ);
        bytes.push(SRC_MEMORY_READ);
        bytes.push(self.area.bit_code()?);
        bytes.extend_from_slice(&self.address.to_bytes());
        bytes.push(0x00); // Count high byte (always 1 bit)
        bytes.push(0x01); // Count low byte
        Ok(bytes)
    }
}

/// Command for writing a single bit to PLC memory.
#[derive(Debug, Clone)]
pub struct WriteBitCommand {
    header: FinsHeader,
    area: MemoryArea,
    address: Address,
    value: bool,
}

impl WriteBitCommand {
    /// Creates a new write bit command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `area` - Memory area to write to (must support bit access)
    /// * `word_address` - Word address
    /// * `bit` - Bit position (0-15)
    /// * `value` - Bit value to write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The memory area doesn't support bit access (DM)
    /// - The bit position is > 15
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{WriteBitCommand, MemoryArea, NodeAddress};
    ///
    /// let cmd = WriteBitCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     MemoryArea::CIO,
    ///     100,
    ///     5,
    ///     true,
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        area: MemoryArea,
        word_address: u16,
        bit: u8,
        value: bool,
    ) -> Result<Self> {
        // Validate bit access is supported
        area.bit_code()?;

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            area,
            address: Address::bit(word_address, bit)?,
            value,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 9);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_WRITE);
        bytes.push(SRC_MEMORY_WRITE);
        bytes.push(self.area.bit_code()?);
        bytes.extend_from_slice(&self.address.to_bytes());
        bytes.push(0x00); // Count high byte (always 1 bit)
        bytes.push(0x01); // Count low byte
        bytes.push(if self.value { 0x01 } else { 0x00 });
        Ok(bytes)
    }
}

/// Command for filling a memory area with a single value.
#[derive(Debug, Clone)]
pub struct FillCommand {
    header: FinsHeader,
    area: MemoryArea,
    address: Address,
    count: u16,
    value: u16,
}

impl FillCommand {
    /// Creates a new fill command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `area` - Memory area to fill
    /// * `word_address` - Starting word address
    /// * `count` - Number of words to fill (1-999)
    /// * `value` - Value to fill with
    ///
    /// # Errors
    ///
    /// Returns an error if count is 0 or exceeds MAX_WORDS_PER_COMMAND.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{FillCommand, MemoryArea, NodeAddress};
    ///
    /// let cmd = FillCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     MemoryArea::DM,
    ///     100,
    ///     50,
    ///     0x0000,
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        area: MemoryArea,
        word_address: u16,
        count: u16,
        value: u16,
    ) -> Result<Self> {
        if count == 0 {
            return Err(FinsError::invalid_parameter(
                "count",
                "must be greater than 0",
            ));
        }
        if count > MAX_WORDS_PER_COMMAND {
            return Err(FinsError::invalid_parameter(
                "count",
                format!("must not exceed {}", MAX_WORDS_PER_COMMAND),
            ));
        }

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            area,
            address: Address::word(word_address),
            count,
            value,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 10);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_READ); // Memory commands use 0x01
        bytes.push(SRC_MEMORY_FILL);
        bytes.push(self.area.word_code());
        bytes.extend_from_slice(&self.address.to_bytes());
        bytes.push((self.count >> 8) as u8);
        bytes.push((self.count & 0xFF) as u8);
        bytes.push((self.value >> 8) as u8);
        bytes.push((self.value & 0xFF) as u8);
        bytes
    }
}

/// PLC operating mode for Run command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlcMode {
    /// Debug mode - step execution.
    Debug,
    /// Monitor mode - run with monitoring enabled.
    Monitor,
    /// Run mode - normal execution.
    Run,
}

impl PlcMode {
    /// Returns the FINS code for this mode.
    pub(crate) fn code(self) -> u8 {
        match self {
            PlcMode::Debug => 0x01,
            PlcMode::Monitor => 0x02,
            PlcMode::Run => 0x04,
        }
    }
}

/// Command for putting the PLC into run mode.
#[derive(Debug, Clone)]
pub struct RunCommand {
    header: FinsHeader,
    mode: PlcMode,
}

impl RunCommand {
    /// Creates a new run command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `mode` - PLC operating mode
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{RunCommand, PlcMode, NodeAddress};
    ///
    /// let cmd = RunCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     PlcMode::Monitor,
    /// );
    /// ```
    pub fn new(destination: NodeAddress, source: NodeAddress, sid: u8, mode: PlcMode) -> Self {
        Self {
            header: FinsHeader::new_command(destination, source, sid),
            mode,
        }
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 5);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_RUN);
        bytes.push(SRC_RUN);
        bytes.push(0xFF); // Program number high byte (current program)
        bytes.push(0xFF); // Program number low byte
        bytes.push(self.mode.code());
        bytes
    }
}

/// Command for stopping the PLC.
#[derive(Debug, Clone)]
pub struct StopCommand {
    header: FinsHeader,
}

impl StopCommand {
    /// Creates a new stop command.
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
    /// use omron_fins::{StopCommand, NodeAddress};
    ///
    /// let cmd = StopCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    /// );
    /// ```
    pub fn new(destination: NodeAddress, source: NodeAddress, sid: u8) -> Self {
        Self {
            header: FinsHeader::new_command(destination, source, sid),
        }
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 2);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_RUN);
        bytes.push(SRC_STOP);
        bytes
    }
}

/// Command for transferring memory from one area to another.
#[derive(Debug, Clone)]
pub struct TransferCommand {
    header: FinsHeader,
    src_area: MemoryArea,
    src_address: Address,
    dst_area: MemoryArea,
    dst_address: Address,
    count: u16,
}

impl TransferCommand {
    /// Creates a new transfer command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `src_area` - Source memory area
    /// * `src_address` - Source starting address
    /// * `dst_area` - Destination memory area
    /// * `dst_address` - Destination starting address
    /// * `count` - Number of words to transfer (1-999)
    ///
    /// # Errors
    ///
    /// Returns an error if count is 0 or exceeds MAX_WORDS_PER_COMMAND.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{TransferCommand, MemoryArea, NodeAddress};
    ///
    /// let cmd = TransferCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     MemoryArea::DM,
    ///     100,
    ///     MemoryArea::DM,
    ///     200,
    ///     10,
    /// ).unwrap();
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        src_area: MemoryArea,
        src_address: u16,
        dst_area: MemoryArea,
        dst_address: u16,
        count: u16,
    ) -> Result<Self> {
        if count == 0 {
            return Err(FinsError::invalid_parameter(
                "count",
                "must be greater than 0",
            ));
        }
        if count > MAX_WORDS_PER_COMMAND {
            return Err(FinsError::invalid_parameter(
                "count",
                format!("must not exceed {}", MAX_WORDS_PER_COMMAND),
            ));
        }

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            src_area,
            src_address: Address::word(src_address),
            dst_area,
            dst_address: Address::word(dst_address),
            count,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 12);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_READ); // Memory commands use 0x01
        bytes.push(SRC_MEMORY_TRANSFER);
        bytes.push(self.src_area.word_code());
        bytes.extend_from_slice(&self.src_address.to_bytes());
        bytes.push(self.dst_area.word_code());
        bytes.extend_from_slice(&self.dst_address.to_bytes());
        bytes.push((self.count >> 8) as u8);
        bytes.push((self.count & 0xFF) as u8);
        bytes
    }
}

/// Specification for forcing a bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceSpec {
    /// Force the bit OFF.
    ForceOff,
    /// Force the bit ON.
    ForceOn,
    /// Release the forced state.
    Release,
}

impl ForceSpec {
    /// Returns the FINS code for this spec.
    pub(crate) fn code(self) -> u16 {
        match self {
            ForceSpec::ForceOff => 0x0000,
            ForceSpec::ForceOn => 0x0001,
            ForceSpec::Release => 0x8000,
        }
    }
}

/// A bit to be forced.
#[derive(Debug, Clone)]
pub struct ForcedBit {
    /// Memory area of the bit.
    pub area: MemoryArea,
    /// Word address of the bit.
    pub address: u16,
    /// Bit position (0-15).
    pub bit: u8,
    /// Force specification.
    pub spec: ForceSpec,
}

/// Command for forcing bits ON/OFF.
#[derive(Debug, Clone)]
pub struct ForcedSetResetCommand {
    header: FinsHeader,
    specs: Vec<ForcedBit>,
}

impl ForcedSetResetCommand {
    /// Creates a new forced set/reset command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `specs` - List of bits to force
    ///
    /// # Errors
    ///
    /// Returns an error if specs is empty, any area doesn't support bit access,
    /// or any bit position is > 15.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{ForcedSetResetCommand, ForcedBit, ForceSpec, MemoryArea, NodeAddress};
    ///
    /// let cmd = ForcedSetResetCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     vec![
    ///         ForcedBit { area: MemoryArea::CIO, address: 0, bit: 0, spec: ForceSpec::ForceOn },
    ///     ],
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        specs: Vec<ForcedBit>,
    ) -> Result<Self> {
        if specs.is_empty() {
            return Err(FinsError::invalid_parameter("specs", "must not be empty"));
        }

        // Validate all specs
        for spec in &specs {
            spec.area.bit_code()?;
            if spec.bit > 15 {
                return Err(FinsError::invalid_parameter("bit", "must be 0-15"));
            }
        }

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            specs,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 4 + self.specs.len() * 6);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_FORCED);
        bytes.push(SRC_FORCED_SET_RESET);
        bytes.push((self.specs.len() >> 8) as u8);
        bytes.push((self.specs.len() & 0xFF) as u8);

        for spec in &self.specs {
            let code = spec.spec.code();
            bytes.push((code >> 8) as u8);
            bytes.push((code & 0xFF) as u8);
            bytes.push(spec.area.bit_code()?);
            bytes.push((spec.address >> 8) as u8);
            bytes.push((spec.address & 0xFF) as u8);
            bytes.push(spec.bit);
        }

        Ok(bytes)
    }
}

/// Command for canceling all forced bits.
#[derive(Debug, Clone)]
pub struct ForcedSetResetCancelCommand {
    header: FinsHeader,
}

impl ForcedSetResetCancelCommand {
    /// Creates a new forced set/reset cancel command.
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
    /// use omron_fins::{ForcedSetResetCancelCommand, NodeAddress};
    ///
    /// let cmd = ForcedSetResetCancelCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    /// );
    /// ```
    pub fn new(destination: NodeAddress, source: NodeAddress, sid: u8) -> Self {
        Self {
            header: FinsHeader::new_command(destination, source, sid),
        }
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 2);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_FORCED);
        bytes.push(SRC_FORCED_CANCEL);
        bytes
    }
}

/// Specification for reading from multiple memory areas.
#[derive(Debug, Clone)]
pub struct MultiReadSpec {
    /// Memory area to read from.
    pub area: MemoryArea,
    /// Word address.
    pub address: u16,
    /// Optional bit position (None for word, Some(n) for bit n).
    pub bit: Option<u8>,
}

/// Command for reading from multiple memory areas.
#[derive(Debug, Clone)]
pub struct MultipleReadCommand {
    header: FinsHeader,
    specs: Vec<MultiReadSpec>,
}

impl MultipleReadCommand {
    /// Creates a new multiple memory area read command.
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination node address
    /// * `source` - Source node address
    /// * `sid` - Service ID for request/response matching
    /// * `specs` - List of read specifications
    ///
    /// # Errors
    ///
    /// Returns an error if specs is empty, any bit area doesn't support bit access,
    /// or any bit position is > 15.
    ///
    /// # Example
    ///
    /// ```
    /// use omron_fins::{MultipleReadCommand, MultiReadSpec, MemoryArea, NodeAddress};
    ///
    /// let cmd = MultipleReadCommand::new(
    ///     NodeAddress::new(0, 10, 0),
    ///     NodeAddress::new(0, 1, 0),
    ///     0x01,
    ///     vec![
    ///         MultiReadSpec { area: MemoryArea::DM, address: 100, bit: None },
    ///         MultiReadSpec { area: MemoryArea::DM, address: 200, bit: None },
    ///     ],
    /// ).unwrap();
    /// ```
    pub fn new(
        destination: NodeAddress,
        source: NodeAddress,
        sid: u8,
        specs: Vec<MultiReadSpec>,
    ) -> Result<Self> {
        if specs.is_empty() {
            return Err(FinsError::invalid_parameter("specs", "must not be empty"));
        }

        // Validate all specs
        for spec in &specs {
            if let Some(bit) = spec.bit {
                spec.area.bit_code()?;
                if bit > 15 {
                    return Err(FinsError::invalid_parameter("bit", "must be 0-15"));
                }
            }
        }

        Ok(Self {
            header: FinsHeader::new_command(destination, source, sid),
            specs,
        })
    }

    /// Returns the service ID.
    pub fn sid(&self) -> u8 {
        self.header.sid
    }

    /// Serializes the command to bytes for transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(FINS_HEADER_SIZE + 2 + self.specs.len() * 4);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.push(MRC_MEMORY_READ);
        bytes.push(SRC_MULTIPLE_READ);

        for spec in &self.specs {
            if let Some(bit) = spec.bit {
                bytes.push(spec.area.bit_code()?);
                bytes.push((spec.address >> 8) as u8);
                bytes.push((spec.address & 0xFF) as u8);
                bytes.push(bit);
            } else {
                bytes.push(spec.area.word_code());
                bytes.push((spec.address >> 8) as u8);
                bytes.push((spec.address & 0xFF) as u8);
                bytes.push(0x00);
            }
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addresses() -> (NodeAddress, NodeAddress) {
        (NodeAddress::new(0, 10, 0), NodeAddress::new(0, 1, 0))
    }

    #[test]
    fn test_address_word() {
        let addr = Address::word(0x1234);
        assert_eq!(addr.word, 0x1234);
        assert_eq!(addr.bit, 0);
        assert_eq!(addr.to_bytes(), [0x12, 0x34, 0x00]);
    }

    #[test]
    fn test_address_bit() {
        let addr = Address::bit(0x1234, 5).unwrap();
        assert_eq!(addr.word, 0x1234);
        assert_eq!(addr.bit, 5);
        assert_eq!(addr.to_bytes(), [0x12, 0x34, 0x05]);
    }

    #[test]
    fn test_address_bit_invalid() {
        let result = Address::bit(100, 16);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_word_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = ReadWordCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 10).unwrap();
        let bytes = cmd.to_bytes();

        // Header (10 bytes) + MRC + SRC + Area + Address (3 bytes) + Count (2 bytes) = 18 bytes
        assert_eq!(bytes.len(), 18);

        // Check header
        assert_eq!(bytes[0], 0x80); // ICF
        assert_eq!(bytes[9], 0x01); // SID

        // Check command
        assert_eq!(bytes[10], MRC_MEMORY_READ);
        assert_eq!(bytes[11], SRC_MEMORY_READ);
        assert_eq!(bytes[12], 0x82); // DM word code

        // Check address (100 = 0x0064)
        assert_eq!(bytes[13], 0x00);
        assert_eq!(bytes[14], 0x64);
        assert_eq!(bytes[15], 0x00); // bit

        // Check count (10 = 0x000A)
        assert_eq!(bytes[16], 0x00);
        assert_eq!(bytes[17], 0x0A);
    }

    #[test]
    fn test_read_word_command_invalid_count() {
        let (dest, src) = test_addresses();

        let result = ReadWordCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 0);
        assert!(result.is_err());

        let result = ReadWordCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_word_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd =
            WriteWordCommand::new(dest, src, 0x02, MemoryArea::DM, 100, &[0x1234, 0x5678]).unwrap();
        let bytes = cmd.to_bytes();

        // Header (10) + MRC + SRC + Area + Address (3) + Count (2) + Data (4) = 22 bytes
        assert_eq!(bytes.len(), 22);

        // Check command codes
        assert_eq!(bytes[10], MRC_MEMORY_WRITE);
        assert_eq!(bytes[11], SRC_MEMORY_WRITE);

        // Check count (2)
        assert_eq!(bytes[16], 0x00);
        assert_eq!(bytes[17], 0x02);

        // Check data
        assert_eq!(bytes[18], 0x12);
        assert_eq!(bytes[19], 0x34);
        assert_eq!(bytes[20], 0x56);
        assert_eq!(bytes[21], 0x78);
    }

    #[test]
    fn test_write_word_command_invalid_data() {
        let (dest, src) = test_addresses();

        let result = WriteWordCommand::new(dest, src, 0x01, MemoryArea::DM, 100, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_bit_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = ReadBitCommand::new(dest, src, 0x03, MemoryArea::CIO, 100, 5).unwrap();
        let bytes = cmd.to_bytes().unwrap();

        // Header (10) + MRC + SRC + Area + Address (3) + Count (2) = 18 bytes
        assert_eq!(bytes.len(), 18);

        // Check area code (CIO bit)
        assert_eq!(bytes[12], 0x30);

        // Check address with bit
        assert_eq!(bytes[13], 0x00);
        assert_eq!(bytes[14], 0x64); // 100
        assert_eq!(bytes[15], 0x05); // bit 5

        // Check count (always 1 for bit)
        assert_eq!(bytes[16], 0x00);
        assert_eq!(bytes[17], 0x01);
    }

    #[test]
    fn test_read_bit_command_dm_fails() {
        let (dest, src) = test_addresses();
        let result = ReadBitCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_bit_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = WriteBitCommand::new(dest, src, 0x04, MemoryArea::WR, 50, 10, true).unwrap();
        let bytes = cmd.to_bytes().unwrap();

        // Header (10) + MRC + SRC + Area + Address (3) + Count (2) + Data (1) = 19 bytes
        assert_eq!(bytes.len(), 19);

        // Check area code (WR bit)
        assert_eq!(bytes[12], 0x31);

        // Check address with bit
        assert_eq!(bytes[13], 0x00);
        assert_eq!(bytes[14], 0x32); // 50
        assert_eq!(bytes[15], 0x0A); // bit 10

        // Check value
        assert_eq!(bytes[18], 0x01); // true
    }

    #[test]
    fn test_write_bit_command_false_value() {
        let (dest, src) = test_addresses();
        let cmd = WriteBitCommand::new(dest, src, 0x05, MemoryArea::HR, 200, 0, false).unwrap();
        let bytes = cmd.to_bytes().unwrap();

        assert_eq!(bytes[12], 0x32); // HR bit code
        assert_eq!(bytes[18], 0x00); // false
    }

    #[test]
    fn test_fill_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = FillCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 50, 0xABCD).unwrap();
        let bytes = cmd.to_bytes();

        // Header (10) + MRC + SRC + Area + Address (3) + Count (2) + Value (2) = 20 bytes
        assert_eq!(bytes.len(), 20);

        // Check command codes
        assert_eq!(bytes[10], MRC_MEMORY_READ); // 0x01
        assert_eq!(bytes[11], SRC_MEMORY_FILL); // 0x03
        assert_eq!(bytes[12], 0x82); // DM word code

        // Check address (100 = 0x0064)
        assert_eq!(bytes[13], 0x00);
        assert_eq!(bytes[14], 0x64);
        assert_eq!(bytes[15], 0x00); // bit

        // Check count (50 = 0x0032)
        assert_eq!(bytes[16], 0x00);
        assert_eq!(bytes[17], 0x32);

        // Check value (0xABCD)
        assert_eq!(bytes[18], 0xAB);
        assert_eq!(bytes[19], 0xCD);
    }

    #[test]
    fn test_fill_command_invalid_count() {
        let (dest, src) = test_addresses();

        let result = FillCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 0, 0x0000);
        assert!(result.is_err());

        let result = FillCommand::new(dest, src, 0x01, MemoryArea::DM, 100, 1000, 0x0000);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = RunCommand::new(dest, src, 0x01, PlcMode::Monitor);
        let bytes = cmd.to_bytes();

        // Header (10) + MRC + SRC + Program (2) + Mode (1) = 15 bytes
        assert_eq!(bytes.len(), 15);

        // Check command codes
        assert_eq!(bytes[10], MRC_RUN); // 0x04
        assert_eq!(bytes[11], SRC_RUN); // 0x01

        // Check program number (0xFFFF = current)
        assert_eq!(bytes[12], 0xFF);
        assert_eq!(bytes[13], 0xFF);

        // Check mode (Monitor = 0x02)
        assert_eq!(bytes[14], 0x02);
    }

    #[test]
    fn test_run_command_modes() {
        let (dest, src) = test_addresses();

        let cmd = RunCommand::new(dest, src, 0x01, PlcMode::Debug);
        assert_eq!(cmd.to_bytes()[14], 0x01);

        let cmd = RunCommand::new(dest, src, 0x01, PlcMode::Monitor);
        assert_eq!(cmd.to_bytes()[14], 0x02);

        let cmd = RunCommand::new(dest, src, 0x01, PlcMode::Run);
        assert_eq!(cmd.to_bytes()[14], 0x04);
    }

    #[test]
    fn test_stop_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = StopCommand::new(dest, src, 0x01);
        let bytes = cmd.to_bytes();

        // Header (10) + MRC + SRC = 12 bytes
        assert_eq!(bytes.len(), 12);

        // Check command codes
        assert_eq!(bytes[10], MRC_RUN); // 0x04
        assert_eq!(bytes[11], SRC_STOP); // 0x02
    }

    #[test]
    fn test_transfer_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = TransferCommand::new(
            dest,
            src,
            0x01,
            MemoryArea::DM,
            100,
            MemoryArea::DM,
            200,
            10,
        )
        .unwrap();
        let bytes = cmd.to_bytes();

        // Header (10) + MRC + SRC + SrcArea + SrcAddr (3) + DstArea + DstAddr (3) + Count (2) = 22 bytes
        assert_eq!(bytes.len(), 22);

        // Check command codes
        assert_eq!(bytes[10], MRC_MEMORY_READ); // 0x01
        assert_eq!(bytes[11], SRC_MEMORY_TRANSFER); // 0x05

        // Check source area and address
        assert_eq!(bytes[12], 0x82); // DM word code
        assert_eq!(bytes[13], 0x00);
        assert_eq!(bytes[14], 0x64); // 100
        assert_eq!(bytes[15], 0x00);

        // Check destination area and address
        assert_eq!(bytes[16], 0x82); // DM word code
        assert_eq!(bytes[17], 0x00);
        assert_eq!(bytes[18], 0xC8); // 200
        assert_eq!(bytes[19], 0x00);

        // Check count (10 = 0x000A)
        assert_eq!(bytes[20], 0x00);
        assert_eq!(bytes[21], 0x0A);
    }

    #[test]
    fn test_transfer_command_invalid_count() {
        let (dest, src) = test_addresses();

        let result =
            TransferCommand::new(dest, src, 0x01, MemoryArea::DM, 100, MemoryArea::DM, 200, 0);
        assert!(result.is_err());

        let result = TransferCommand::new(
            dest,
            src,
            0x01,
            MemoryArea::DM,
            100,
            MemoryArea::DM,
            200,
            1000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_forced_set_reset_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = ForcedSetResetCommand::new(
            dest,
            src,
            0x01,
            vec![
                ForcedBit {
                    area: MemoryArea::CIO,
                    address: 0,
                    bit: 0,
                    spec: ForceSpec::ForceOn,
                },
                ForcedBit {
                    area: MemoryArea::CIO,
                    address: 0,
                    bit: 1,
                    spec: ForceSpec::ForceOff,
                },
            ],
        )
        .unwrap();
        let bytes = cmd.to_bytes().unwrap();

        // Header (10) + MRC + SRC + Count (2) + 2 * Spec (6) = 26 bytes
        assert_eq!(bytes.len(), 26);

        // Check command codes
        assert_eq!(bytes[10], MRC_FORCED); // 0x23
        assert_eq!(bytes[11], SRC_FORCED_SET_RESET); // 0x01

        // Check count (2)
        assert_eq!(bytes[12], 0x00);
        assert_eq!(bytes[13], 0x02);

        // Check first spec (ForceOn)
        assert_eq!(bytes[14], 0x00); // spec code high
        assert_eq!(bytes[15], 0x01); // spec code low (ForceOn = 0x0001)
        assert_eq!(bytes[16], 0x30); // CIO bit code
        assert_eq!(bytes[17], 0x00); // address high
        assert_eq!(bytes[18], 0x00); // address low
        assert_eq!(bytes[19], 0x00); // bit

        // Check second spec (ForceOff)
        assert_eq!(bytes[20], 0x00); // spec code high
        assert_eq!(bytes[21], 0x00); // spec code low (ForceOff = 0x0000)
        assert_eq!(bytes[22], 0x30); // CIO bit code
        assert_eq!(bytes[23], 0x00); // address high
        assert_eq!(bytes[24], 0x00); // address low
        assert_eq!(bytes[25], 0x01); // bit
    }

    #[test]
    fn test_forced_set_reset_command_empty_specs() {
        let (dest, src) = test_addresses();
        let result = ForcedSetResetCommand::new(dest, src, 0x01, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_forced_set_reset_command_dm_fails() {
        let (dest, src) = test_addresses();
        let result = ForcedSetResetCommand::new(
            dest,
            src,
            0x01,
            vec![ForcedBit {
                area: MemoryArea::DM,
                address: 0,
                bit: 0,
                spec: ForceSpec::ForceOn,
            }],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_forced_set_reset_cancel_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = ForcedSetResetCancelCommand::new(dest, src, 0x01);
        let bytes = cmd.to_bytes();

        // Header (10) + MRC + SRC = 12 bytes
        assert_eq!(bytes.len(), 12);

        // Check command codes
        assert_eq!(bytes[10], MRC_FORCED); // 0x23
        assert_eq!(bytes[11], SRC_FORCED_CANCEL); // 0x02
    }

    #[test]
    fn test_multiple_read_command_serialization() {
        let (dest, src) = test_addresses();
        let cmd = MultipleReadCommand::new(
            dest,
            src,
            0x01,
            vec![
                MultiReadSpec {
                    area: MemoryArea::DM,
                    address: 100,
                    bit: None,
                },
                MultiReadSpec {
                    area: MemoryArea::DM,
                    address: 200,
                    bit: None,
                },
                MultiReadSpec {
                    area: MemoryArea::CIO,
                    address: 0,
                    bit: Some(5),
                },
            ],
        )
        .unwrap();
        let bytes = cmd.to_bytes().unwrap();

        // Header (10) + MRC + SRC + 3 * Spec (4) = 24 bytes
        assert_eq!(bytes.len(), 24);

        // Check command codes
        assert_eq!(bytes[10], MRC_MEMORY_READ); // 0x01
        assert_eq!(bytes[11], SRC_MULTIPLE_READ); // 0x04

        // Check first spec (DM100 word)
        assert_eq!(bytes[12], 0x82); // DM word code
        assert_eq!(bytes[13], 0x00);
        assert_eq!(bytes[14], 0x64); // 100
        assert_eq!(bytes[15], 0x00);

        // Check second spec (DM200 word)
        assert_eq!(bytes[16], 0x82); // DM word code
        assert_eq!(bytes[17], 0x00);
        assert_eq!(bytes[18], 0xC8); // 200
        assert_eq!(bytes[19], 0x00);

        // Check third spec (CIO0.05 bit)
        assert_eq!(bytes[20], 0x30); // CIO bit code
        assert_eq!(bytes[21], 0x00);
        assert_eq!(bytes[22], 0x00); // 0
        assert_eq!(bytes[23], 0x05); // bit 5
    }

    #[test]
    fn test_multiple_read_command_empty_specs() {
        let (dest, src) = test_addresses();
        let result = MultipleReadCommand::new(dest, src, 0x01, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_read_command_dm_bit_fails() {
        let (dest, src) = test_addresses();
        let result = MultipleReadCommand::new(
            dest,
            src,
            0x01,
            vec![MultiReadSpec {
                area: MemoryArea::DM,
                address: 100,
                bit: Some(5),
            }],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_force_spec_codes() {
        assert_eq!(ForceSpec::ForceOff.code(), 0x0000);
        assert_eq!(ForceSpec::ForceOn.code(), 0x0001);
        assert_eq!(ForceSpec::Release.code(), 0x8000);
    }

    #[test]
    fn test_plc_mode_codes() {
        assert_eq!(PlcMode::Debug.code(), 0x01);
        assert_eq!(PlcMode::Monitor.code(), 0x02);
        assert_eq!(PlcMode::Run.code(), 0x04);
    }
}
