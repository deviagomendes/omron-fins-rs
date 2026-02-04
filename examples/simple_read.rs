//! Simple read example - equivalent to Python fins-driver:
//!
//! ```python
//! from fins import FinsClient
//! client = FinsClient(host='192.168.1.250', port=9600)
//! client.connect()
//! response = client.memory_area_read('D1')
//! print(response.data)
//! client.close()
//! ```
//!
//! Run with: cargo run --example simple_read

use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    // Connect to PLC at factory default IP (192.168.1.250)
    // Using source_node=1, dest_node=0 (same defaults as Python fins-driver)
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
    let client = Client::new(config)?;

    // Read D1 (1 word from DM area, address 1)
    let data = client.read(MemoryArea::DM, 1, 1)?;
    println!("D1 = {:?}", data);

    // Read CIO1
    let cio = client.read(MemoryArea::CIO, 1, 1)?;
    println!("CIO1 = {:?}", cio);

    // Client is automatically closed when dropped (no explicit close needed)
    Ok(())
}
