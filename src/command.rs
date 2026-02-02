//! FINS command structures.

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
}
