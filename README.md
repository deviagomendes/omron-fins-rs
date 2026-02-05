# omron-fins

A Rust library for communicating with Omron PLCs using the FINS protocol.

[![Crates.io](https://img.shields.io/crates/v/omron-fins.svg)](https://crates.io/crates/omron-fins)
[![Documentation](https://docs.rs/omron-fins/badge.svg)](https://docs.rs/omron-fins)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Protocol-only library** — no business logic, polling, or schedulers
- **Deterministic execution** — each call produces exactly 1 request and 1 response
- **No implicit behavior** — no automatic retry, caching, or reconnection
- **Complete API** — read, write, fill, run/stop, forced set/reset, transfer, multiple read
- **Type-safe** — memory areas as `enum`, never strings
- **Type helpers** — native support for `f32`, `f64`, `i32`, and ASCII strings
- **Comprehensive error handling** — no `panic!` in public code

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
omron-fins = "0.4.1"
```

## Quick Start

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    // Connect to PLC at factory default IP (192.168.1.250)
    // Using source_node=1, dest_node=0 (same defaults as Python fins-driver)
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
    let client = Client::new(config)?;

    // Read D1 (1 word from DM area)
    let data = client.read(MemoryArea::DM, 1, 1)?;
    println!("D1 = {:?}", data);

    // Read 10 words starting from DM100
    let data = client.read(MemoryArea::DM, 100, 10)?;
    println!("DM100-109: {:?}", data);

    // Write values to DM200
    client.write(MemoryArea::DM, 200, &[0x1234, 0x5678])?;

    // Read a specific bit (CIO 0.05)
    let bit = client.read_bit(MemoryArea::CIO, 0, 5)?;
    println!("CIO 0.05 = {}", bit);

    // Write a bit
    client.write_bit(MemoryArea::CIO, 0, 5, true)?;

    Ok(())
}
```

### Equivalent Python (fins-driver)

This library is compatible with the Python [fins-driver](https://pypi.org/project/fins-driver/) library:

```python
from fins import FinsClient

client = FinsClient(host='192.168.1.250', port=9600)
client.connect()
response = client.memory_area_read('D1')
print(response.data)
client.close()
```

## Memory Areas

The library supports the following memory areas:

| Area | Name | Description | Word Access | Bit Access |
|------|------|-------------|:-----------:|:----------:|
| `CIO` | Core I/O | Inputs/outputs and internal relays | ✓ | ✓ |
| `WR` | Work | Temporary work bits/words | ✓ | ✓ |
| `HR` | Holding | Retentive bits/words | ✓ | ✓ |
| `DM` | Data Memory | Numeric data storage | ✓ | ✗ |
| `AR` | Auxiliary | System status and control | ✓ | ✓ |

```rust
use omron_fins::MemoryArea;

// Check if an area supports bit access
assert!(MemoryArea::CIO.supports_bit_access());
assert!(!MemoryArea::DM.supports_bit_access());
```

## API Reference

### Reading Words

```rust
// Read 'count' words starting from 'address'
let data: Vec<u16> = client.read(area, address, count)?;
```

**Parameters:**
- `area`: Memory area (`MemoryArea::DM`, `CIO`, `WR`, `HR`, `AR`)
- `address`: Starting address (0-65535)
- `count`: Number of words to read (1-999)

### Writing Words

```rust
// Write a slice of words starting from 'address'
client.write(area, address, &[value1, value2, ...])?;
```

**Parameters:**
- `area`: Memory area
- `address`: Starting address
- `data`: Slice of words to write (1-999 words)

### Reading Bits

```rust
// Read a specific bit
let value: bool = client.read_bit(area, address, bit)?;
```

**Parameters:**
- `area`: Memory area (only `CIO`, `WR`, `HR`, `AR` — DM not supported)
- `address`: Word address
- `bit`: Bit position (0-15)

### Writing Bits

```rust
// Write a specific bit
client.write_bit(area, address, bit, value)?;
```

**Parameters:**
- `area`: Memory area (only `CIO`, `WR`, `HR`, `AR`)
- `address`: Word address
- `bit`: Bit position (0-15)
- `value`: Value to write (`true` or `false`)

### Fill (Memory Fill)

```rust
// Fill a memory region with a value
client.fill(MemoryArea::DM, 100, 50, 0x0000)?; // Zero out DM100-DM149
```

**Parameters:**
- `area`: Memory area
- `address`: Starting address
- `count`: Number of words to fill (1-999)
- `value`: Value to repeat

### Run / Stop PLC

```rust
use omron_fins::PlcMode;

// Put the PLC in run mode
client.run(PlcMode::Monitor)?;

// Stop the PLC
client.stop()?;
```

**Available modes:**
- `PlcMode::Debug` — step-by-step execution
- `PlcMode::Monitor` — execution with monitoring
- `PlcMode::Run` — normal execution

### Memory Transfer

```rust
// Copy DM100-DM109 to DM200-DM209
client.transfer(MemoryArea::DM, 100, MemoryArea::DM, 200, 10)?;
```

**Parameters:**
- `src_area`: Source area
- `src_address`: Source address
- `dst_area`: Destination area
- `dst_address`: Destination address
- `count`: Number of words to transfer (1-999)

### Forced Set/Reset

Force bits ON/OFF overriding PLC program (used for maintenance).

```rust
use omron_fins::{ForcedBit, ForceSpec, MemoryArea};

// Force bits
client.forced_set_reset(&[
    ForcedBit { area: MemoryArea::CIO, address: 0, bit: 0, spec: ForceSpec::ForceOn },
    ForcedBit { area: MemoryArea::CIO, address: 0, bit: 1, spec: ForceSpec::ForceOff },
])?;

// Cancel all forced bits
client.forced_set_reset_cancel()?;
```

**ForceSpec:**
- `ForceSpec::ForceOn` — force bit ON
- `ForceSpec::ForceOff` — force bit OFF
- `ForceSpec::Release` — release forced state

### Multiple Read

Read from multiple areas/addresses in a single request (optimizes communication).

```rust
use omron_fins::MultiReadSpec;

let values = client.read_multiple(&[
    MultiReadSpec { area: MemoryArea::DM, address: 100, bit: None },
    MultiReadSpec { area: MemoryArea::DM, address: 200, bit: None },
    MultiReadSpec { area: MemoryArea::CIO, address: 0, bit: Some(5) },
])?;
// values[0] = DM100, values[1] = DM200, values[2] = CIO0.05 (0 or 1)
```

### Data Types

Helpers for reading/writing types that span multiple words.

```rust
// f32 (REAL) - 2 words
let temp: f32 = client.read_f32(MemoryArea::DM, 100)?;
client.write_f32(MemoryArea::DM, 100, 3.14159)?;

// f64 (LREAL) - 4 words
let value: f64 = client.read_f64(MemoryArea::DM, 100)?;
client.write_f64(MemoryArea::DM, 100, 3.141592653589793)?;

// i32 (DINT) - 2 words
let counter: i32 = client.read_i32(MemoryArea::DM, 100)?;
client.write_i32(MemoryArea::DM, 100, -123456)?;

// String (ASCII) - variable words (2 chars per word)
client.write_string(MemoryArea::DM, 200, "PRODUCT-001")?;
let code: String = client.read_string(MemoryArea::DM, 200, 6)?; // 6 words = up to 12 chars
```

### Strings

Read and write ASCII strings to PLC memory. Each word stores 2 characters (big-endian).

```rust
// Write a string to DM100
client.write_string(MemoryArea::DM, 100, "Hello World")?;

// Read a string from DM100 (10 words = up to 20 characters)
let message = client.read_string(MemoryArea::DM, 100, 10)?;
println!("Message: {}", message);
```

**Parameters:**
- `area`: Memory area
- `address`: Starting word address
- `value` (write): String to write (ASCII, max 1998 characters)
- `word_count` (read): Number of words to read (1-999)

**Notes:**
- Strings with odd character count are padded with 0x00
- Null bytes at the end are automatically trimmed when reading
- Non-ASCII characters are converted using UTF-8 lossy conversion

## Advanced Configuration

### Full Client Configuration

```rust
use omron_fins::ClientConfig;
use std::net::Ipv4Addr;
use std::time::Duration;

let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
    .with_port(9601)                        // Custom port (default: 9600)
    .with_timeout(Duration::from_secs(5))   // Custom timeout (default: 2s)
    .with_source_network(1)                 // Source network
    .with_source_unit(0)                    // Source unit
    .with_dest_network(1)                   // Destination network
    .with_dest_unit(0);                     // Destination unit
```

### Node Addressing

The FINS protocol uses three components to address a node:

| Component | Description | Typical Value |
|-----------|-------------|---------------|
| Network | Network number | 0 (local network) |
| Node | Node number | 1-126 |
| Unit | Unit number | 0 (CPU) |

For simple communication on the same network, only the node number is required:

```rust
// Simple local communication
let config = ClientConfig::new(ip, source_node, dest_node);

// Cross-network communication
let config = ClientConfig::new(ip, source_node, dest_node)
    .with_source_network(1)
    .with_dest_network(2);
```

## Error Handling

All operations return `Result<T, FinsError>`. The library never panics in public code.

```rust
use omron_fins::{Client, ClientConfig, MemoryArea, FinsError};
use std::net::Ipv4Addr;

let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
let client = Client::new(config)?;

match client.read(MemoryArea::DM, 100, 10) {
    Ok(data) => println!("Data: {:?}", data),
    
    Err(FinsError::Timeout) => {
        println!("Communication timeout");
    }
    
    Err(FinsError::PlcError { main_code, sub_code }) => {
        println!("PLC error: main=0x{:02X}, sub=0x{:02X}", main_code, sub_code);
    }
    
    Err(FinsError::InvalidAddressing { reason }) => {
        println!("Invalid addressing: {}", reason);
    }
    
    Err(FinsError::InvalidParameter { parameter, reason }) => {
        println!("Invalid parameter '{}': {}", parameter, reason);
    }
    
    Err(e) => println!("Error: {}", e),
}
```

### Error Types

| Error | Description |
|-------|-------------|
| `PlcError` | Error returned by the PLC (with main/sub codes) |
| `Timeout` | Communication timeout |
| `InvalidAddressing` | Invalid addressing (e.g., bit access on DM) |
| `InvalidParameter` | Invalid parameter (e.g., count = 0) |
| `InvalidResponse` | Invalid response from PLC |
| `SidMismatch` | Service ID mismatch between request/response |
| `Io` | System I/O error |

## Examples

### I/O Monitoring

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
    )?;

    // Read digital input states (CIO 0-9)
    let inputs = client.read(MemoryArea::CIO, 0, 10)?;
    
    for (i, word) in inputs.iter().enumerate() {
        println!("CIO {:03}: 0x{:04X} ({:016b})", i, word, word);
    }

    Ok(())
}
```

### Recipe Writing

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn write_recipe(client: &Client, recipe_id: u16, params: &[u16]) -> omron_fins::Result<()> {
    // Write recipe ID to DM100
    client.write(MemoryArea::DM, 100, &[recipe_id])?;
    
    // Write parameters to DM101-DM110
    client.write(MemoryArea::DM, 101, params)?;
    
    // Set "recipe ready" bit at WR 0.00
    client.write_bit(MemoryArea::WR, 0, 0, true)?;
    
    Ok(())
}

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
    )?;

    let recipe_params = [1000, 2000, 3000, 500, 750];
    write_recipe(&client, 42, &recipe_params)?;
    
    println!("Recipe sent successfully!");
    Ok(())
}
```

### Alarm Reading

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn check_alarms(client: &Client) -> omron_fins::Result<Vec<usize>> {
    // Read 10 alarm words (160 bits)
    let alarm_words = client.read(MemoryArea::HR, 0, 10)?;
    
    let mut active_alarms = Vec::new();
    
    for (word_idx, word) in alarm_words.iter().enumerate() {
        for bit in 0..16 {
            if (word >> bit) & 1 == 1 {
                active_alarms.push(word_idx * 16 + bit);
            }
        }
    }
    
    Ok(active_alarms)
}

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
    )?;

    let alarms = check_alarms(&client)?;
    
    if alarms.is_empty() {
        println!("No active alarms");
    } else {
        println!("Active alarms: {:?}", alarms);
    }
    
    Ok(())
}
```

### PLC Control

```rust
use omron_fins::{Client, ClientConfig, PlcMode};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
    )?;

    // Stop PLC for maintenance
    client.stop()?;
    println!("PLC stopped");

    // Perform maintenance operations...

    // Restart in monitor mode
    client.run(PlcMode::Monitor)?;
    println!("PLC running (monitor mode)");
    
    Ok(())
}
```

### Sensor Reading (Float Types)

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
    )?;

    // Read temperature (f32) from DM100-DM101
    let temperature: f32 = client.read_f32(MemoryArea::DM, 100)?;
    println!("Temperature: {:.2}°C", temperature);

    // Read pressure (f32) from DM102-DM103
    let pressure: f32 = client.read_f32(MemoryArea::DM, 102)?;
    println!("Pressure: {:.2} bar", pressure);

    // Read production counter (i32) from DM104-DM105
    let counter: i32 = client.read_i32(MemoryArea::DM, 104)?;
    println!("Parts produced: {}", counter);
    
    Ok(())
}
```

## Utility Functions

The library provides utility functions for bit manipulation and data formatting:

```rust
use omron_fins::utils::{
    get_bit, set_bit, toggle_bit,
    word_to_bits, bits_to_word,
    get_on_bits, count_on_bits,
    format_binary, format_hex,
    print_bits,
};

let value: u16 = 0b1010_0101_1100_0011;

// Get individual bits
assert!(get_bit(value, 0));   // bit 0 is ON
assert!(!get_bit(value, 2));  // bit 2 is OFF

// Modify bits
let modified = set_bit(value, 2, true);
let toggled = toggle_bit(value, 0);

// Convert word to bit array
let bits = word_to_bits(value);
for (i, bit) in bits.iter().enumerate() {
    if *bit {
        println!("Bit {} is ON", i);
    }
}

// Get list of ON bits
let on_bits = get_on_bits(value);
println!("Bits that are ON: {:?}", on_bits);

// Count ON bits
let count = count_on_bits(value);
println!("Number of ON bits: {}", count);

// Format for display
println!("{}", format_binary(value));  // "0b1010_0101_1100_0011"
println!("{}", format_hex(value));     // "0xA5C3"

// Print all bits to stdout
print_bits(value);
```

## Constants

```rust
use omron_fins::{DEFAULT_FINS_PORT, DEFAULT_TIMEOUT, MAX_PACKET_SIZE, MAX_WORDS_PER_COMMAND};

// Default FINS UDP port
assert_eq!(DEFAULT_FINS_PORT, 9600);

// Default communication timeout
assert_eq!(DEFAULT_TIMEOUT, std::time::Duration::from_secs(2));

// Maximum FINS packet size
assert_eq!(MAX_PACKET_SIZE, 2048);

// Maximum words per command
assert_eq!(MAX_WORDS_PER_COMMAND, 999);
```

## Limitations

- **UDP only** — TCP is not supported in this version
- **Synchronous** — blocking operations (async may be added in the future)
- **No automatic retry** — the application must implement retry logic if needed
- **No caching** — each call generates a network request
- **No automatic reconnection** — the application must recreate the client if needed

## Design Philosophy

This library follows the principle of **determinism over abstraction**:

1. Each operation does exactly what it says
2. No magic or implicit behavior
3. The application has full control over retry, caching, and reconnection
4. Errors are always explicit and descriptive

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read [ARCHITECTURE.md](ARCHITECTURE.md) to understand the project's design rules before submitting PRs.
