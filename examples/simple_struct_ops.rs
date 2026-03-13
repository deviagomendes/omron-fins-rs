//! Example: Reading and writing custom structures (structs) in Omron PLC memory.
//!
//! Omron PLCs organize structs contiguously in memory. Each data type occupies
//! a specific number of bytes (Words), and fields are typically aligned to 16-bit
//! boundaries. Multi-word types (DINT, LINT, REAL, etc.) follow a specific "Word Swap"
//! convention which is automatically handled by this library.

use omron_fins::{Client, ClientConfig, DataType, MemoryArea, PlcValue};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    // Client configuration (adjust to your PLC's IP and node addresses)
    // Common defaults for FINS: source node 250, destination node 1.
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 250, 1), 250, 1)
        .with_timeout(std::time::Duration::from_secs(10));
    let client = Client::new(config)?;

    println!("Example: Reading and Writing Custom Structs");

    // 1. Define data for writing
    // We create a list of values representing a structure in the PLC memory.
    // The library handles 16-bit alignment and Word Swapping for us.
    let values = vec![
        PlcValue::Udint(555555555),  // UDINT (32-bit) - 4 bytes (2 words)
        PlcValue::Uint(200),         // UINT (16-bit) - 2 bytes (1 word)
        PlcValue::Uint(300),         // UINT (16-bit) - 2 bytes (1 word)
    ];

    println!("Writing struct to DM0...");
    client.write_struct(MemoryArea::DM, 0, values)?;

    // 2. Read the struct back from the PLC
    // To read, we define the structure's blueprint using DataType enums.
    println!("Reading struct from DM0...");
    let definition = vec![
        DataType::UDINT,
        DataType::UINT,
        DataType::UINT,
    ];

    let results = client.read_struct(MemoryArea::DM, 0, definition)?;

    // 3. Display results
    // Values are returned as PlcValue variants which can be matched or debug-printed.
    for (i, val) in results.iter().enumerate() {
        println!("Field {}: {:?}", i, val);
    }

    Ok(())
}
