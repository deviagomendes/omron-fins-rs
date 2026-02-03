# Architecture

This document describes the architecture and design principles of the `omron-fins` library.

## Overview

The `omron-fins` library is a **protocol-only** implementation of the Omron FINS protocol for Rust. It provides a type-safe, deterministic API for communicating with Omron PLCs over UDP.

## Design Principles

### 1. Protocol-Only Scope

The library implements **only** the FINS protocol layer:

- FINS frame construction and parsing
- Memory read/write operations
- PLC control commands (run/stop)
- Error code decoding

**Explicitly out of scope:**

- Business logic
- Polling loops
- Schedulers
- Gateways
- Edge runtime
- Database or message broker integration

### 2. Deterministic Execution

Every public API call follows a strict 1:1 pattern:

```
1 function call → 1 FINS request → 1 FINS response → 1 return value
```

There are no hidden behaviors:

- No automatic retries
- No connection pooling
- No caching of responses
- No background reconnection

This makes the library behavior completely predictable and debuggable.

### 3. Explicit Over Implicit

The API prefers explicitness:

```rust
// Explicit: caller knows exactly what happens
client.read(MemoryArea::DM, 100, 10)?;

// The library will NOT:
// - Retry on failure
// - Cache the result
// - Batch with other requests
// - Reconnect if disconnected
```

## Module Structure

```
src/
├── lib.rs          # Public API re-exports and crate documentation
├── client.rs       # High-level Client API
├── command.rs      # FINS command structures and serialization
├── response.rs     # FINS response parsing
├── header.rs       # FINS header structure
├── memory.rs       # Memory area definitions
├── error.rs        # Error types
└── transport.rs    # UDP transport layer
```

### Layer Responsibilities

| Layer | Responsibility | Knows About |
|-------|----------------|-------------|
| `Client` | High-level API, orchestration | Commands, responses, transport |
| `Command` | Frame construction, serialization | Header, memory areas |
| `Response` | Frame parsing, validation | Header, error codes |
| `Header` | FINS header structure | Nothing else |
| `Memory` | Memory area codes | Nothing else |
| `Transport` | UDP send/receive | Sockets only |
| `Error` | Error types and conversions | Nothing else |

### Separation of Concerns

The transport layer (`transport.rs`) handles only socket operations:

```rust
// Transport knows: sockets, timeouts, byte buffers
// Transport does NOT know: FINS protocol, memory areas, commands
```

The protocol layer (`command.rs`, `response.rs`) handles only FINS semantics:

```rust
// Protocol knows: FINS frames, command codes, memory areas
// Protocol does NOT know: sockets, network addresses, timeouts
```

## Type Safety

### Memory Areas as Enum

Memory areas are represented as an enum, never as strings:

```rust
pub enum MemoryArea {
    CIO,  // Core I/O
    WR,   // Work
    HR,   // Holding
    DM,   // Data Memory
    AR,   // Auxiliary
}
```

This provides:
- Compile-time validation
- IDE autocompletion
- Exhaustive pattern matching

### FINS Codes are Internal

FINS protocol codes are never exposed in the public API:

```rust
impl MemoryArea {
    // Internal only - users never see 0x82, 0xB0, etc.
    pub(crate) fn word_code(self) -> u8 {
        match self {
            MemoryArea::CIO => 0xB0,
            MemoryArea::WR => 0xB1,
            MemoryArea::HR => 0xB2,
            MemoryArea::DM => 0x82,
            MemoryArea::AR => 0xB3,
        }
    }
}
```

## Addressing Model

### Word Addressing

All addresses are explicit word addresses (u16):

```rust
// Read 10 words starting at word 100
client.read(MemoryArea::DM, 100, 10)?;
```

### Bit Addressing

Bit addressing uses word + bit position:

```rust
// Read bit 5 of word 0
client.read_bit(MemoryArea::CIO, 0, 5)?;
```

### DM Restrictions

DM area does not support bit access. Attempting bit operations on DM returns an error:

```rust
// This returns Err(FinsError::InvalidAddressing)
client.read_bit(MemoryArea::DM, 100, 0)?;
```

## Error Handling

### No Panics in Public Code

All public functions return `Result<T, FinsError>`:

```rust
pub fn read(&self, area: MemoryArea, address: u16, count: u16) -> Result<Vec<u16>>
```

The library never panics on invalid input—it returns appropriate errors.

### Error Hierarchy

```rust
pub enum FinsError {
    /// Error from PLC (main/sub codes)
    PlcError { main_code: u8, sub_code: u8 },
    
    /// Invalid memory addressing
    InvalidAddressing { reason: String },
    
    /// Invalid parameter value
    InvalidParameter { parameter: String, reason: String },
    
    /// Invalid response from PLC
    InvalidResponse { reason: String },
    
    /// Communication timeout
    Timeout,
    
    /// Service ID mismatch
    SidMismatch { expected: u8, received: u8 },
    
    /// I/O error
    Io(std::io::Error),
}
```

### PLC Error Decoding

PLC errors include both main and sub codes for detailed diagnostics:

```rust
Err(FinsError::PlcError { main_code: 0x01, sub_code: 0x01 })
// Interpretation: Local node not in network
```

## API Design

### Public API Surface

The `Client` struct provides the high-level API:

```rust
impl Client {
    // Word operations
    pub fn read(&self, area: MemoryArea, address: u16, count: u16) -> Result<Vec<u16>>;
    pub fn write(&self, area: MemoryArea, address: u16, data: &[u16]) -> Result<()>;
    pub fn fill(&self, area: MemoryArea, address: u16, count: u16, value: u16) -> Result<()>;
    pub fn transfer(&self, src_area: MemoryArea, src_addr: u16, 
                    dst_area: MemoryArea, dst_addr: u16, count: u16) -> Result<()>;
    
    // Bit operations
    pub fn read_bit(&self, area: MemoryArea, address: u16, bit: u8) -> Result<bool>;
    pub fn write_bit(&self, area: MemoryArea, address: u16, bit: u8, value: bool) -> Result<()>;
    
    // Type helpers
    pub fn read_f32(&self, area: MemoryArea, address: u16) -> Result<f32>;
    pub fn write_f32(&self, area: MemoryArea, address: u16, value: f32) -> Result<()>;
    pub fn read_f64(&self, area: MemoryArea, address: u16) -> Result<f64>;
    pub fn write_f64(&self, area: MemoryArea, address: u16, value: f64) -> Result<()>;
    pub fn read_i32(&self, area: MemoryArea, address: u16) -> Result<i32>;
    pub fn write_i32(&self, area: MemoryArea, address: u16, value: i32) -> Result<()>;
    
    // PLC control
    pub fn run(&self, mode: PlcMode) -> Result<()>;
    pub fn stop(&self) -> Result<()>;
    
    // Advanced operations
    pub fn forced_set_reset(&self, specs: &[ForcedBit]) -> Result<()>;
    pub fn forced_set_reset_cancel(&self) -> Result<()>;
    pub fn read_multiple(&self, specs: &[MultiReadSpec]) -> Result<Vec<u16>>;
}
```

### Configuration Pattern

Client configuration uses the builder pattern:

```rust
let config = ClientConfig::new(ip, source_node, dest_node)
    .with_port(9601)
    .with_timeout(Duration::from_secs(5))
    .with_source_network(1)
    .with_dest_network(2);
```

## Transport Layer

### UDP Protocol

The library uses synchronous UDP communication:

```rust
pub struct UdpTransport {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl UdpTransport {
    pub fn send_receive(&self, data: &[u8]) -> Result<Vec<u8>>;
}
```

### Timeout Handling

Read timeouts are converted to `FinsError::Timeout`:

```rust
match self.socket.recv_from(&mut buffer) {
    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(FinsError::Timeout),
    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Err(FinsError::Timeout),
    ...
}
```

## Documentation Requirements

All public items must have documentation including:

```rust
/// Brief description of the function.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Errors
///
/// Returns an error if...
///
/// # Example
///
/// ```
/// // Working example code
/// ```
pub fn example(param: Type) -> Result<Output> { ... }
```

## Testing Strategy

### Unit Tests

Every command has serialization tests:

```rust
#[test]
fn test_read_word_command_serialization() {
    let cmd = ReadWordCommand::new(...);
    let bytes = cmd.to_bytes();
    assert_eq!(bytes, expected_bytes);
}
```

### Response Parsing Tests

Response parsing uses real PLC response fixtures:

```rust
#[test]
fn test_response_parsing() {
    let bytes = [0xC0, 0x00, ...]; // Real PLC response
    let response = FinsResponse::from_bytes(&bytes).unwrap();
    assert!(response.is_success());
}
```

## Version Compatibility

- **0.x.y**: API may change between minor versions
- **1.x.y**: Stable API, no breaking changes in minor/patch versions

## Code Style

### Clarity Over Cleverness

Prefer readable code over clever abstractions:

```rust
// Good: Clear and explicit
for word in &self.data {
    bytes.push((word >> 8) as u8);
    bytes.push((word & 0xFF) as u8);
}

// Avoid: Overly clever
bytes.extend(self.data.iter().flat_map(|w| w.to_be_bytes()));
```

### Minimal Macros

Avoid macros unless they provide significant value. Prefer functions and generics.

### Linting

The crate enables strict lints:

```rust
#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
```

## Future Considerations

The following features may be added in future versions:

- **Async support**: Using `tokio` or `async-std` (if it doesn't break the sync API)
- **TCP transport**: For environments where UDP is unreliable
- **Connection pooling**: Optional feature for high-throughput applications

These will be added as opt-in features to maintain the library's simplicity.
