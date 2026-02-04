use omron_fins::{Client, ClientConfig, MemoryArea};
use std::{net::Ipv4Addr, os::windows::thread};

fn main() -> omron_fins::Result<()> {
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 10, 122), 1, 0);
    let client = Client::new(config)?;

    println!("=== Testing Write Operations ===\n");

    client.write_f32(MemoryArea::DM, 2, 1.45)?;

    Ok(())
}
