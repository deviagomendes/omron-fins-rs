//! Example: Writing data to PLC memory
//!
//! Run with: cargo run --example simple_write
//!
//! This example demonstrates:
//! - Connecting to an Omron PLC via FINS/UDP
//! - Testing max limits of memory read/write according to specific memory area capacities

use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

use std::time::{Duration, Instant};

fn main() -> omron_fins::Result<()> {
    // =========================================================================
    // Connect to PLC
    // =========================================================================
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 10, 122), 1, 122)
        .with_timeout(Duration::from_millis(20000));
    let client = Client::new(config)?;

    let start_cio = Instant::now();

    match client.read(MemoryArea::CIO, 0, MemoryArea::CIO.max_words()) {
        Ok(data) => {
            let duration = start_cio.elapsed();
            println!("Successfully read {} words in {:?}", data.len(), duration);
        }
        Err(e) => println!("Error reading CIO max capacity: {:?}", e),
    }

    println!("----------------------------------------");

    Ok(())
}
