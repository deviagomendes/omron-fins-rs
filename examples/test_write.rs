//! Test write operations with different data types
//!
//! Run with: cargo run --example test_write

use omron_fins::{Client, ClientConfig, MemoryArea};
use std::{net::Ipv4Addr, os::windows::thread};

fn main() -> omron_fins::Result<()> {
    // Connect to your PLC
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 10, 122), 1, 0);
    let client = Client::new(config)?;

    println!("=== Testing Write Operations ===\n");

    // Test 1: Write u16 (single word) to D1
    println!("1. Writing u16 value 12345 to D1...");
    client.write(MemoryArea::DM, 1, &[12345u16])?;
    let read_back = client.read(MemoryArea::DM, 1, 1)?;
    println!("   Read back: {} (expected: 12345)", read_back[0]);
    assert_eq!(read_back[0], 12345);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Test 2: Write u16 (max value) to D2
    println!("2. Writing u16 max value 65535 to D2...");
    client.write(MemoryArea::DM, 2, &[65535u16])?;
    let read_back = client.read(MemoryArea::DM, 2, 1)?;
    println!("   Read back: {} (expected: 65535)", read_back[0]);
    assert_eq!(read_back[0], 65535);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Test 3: Write i16 (signed, negative value) - stored as u16
    println!("3. Writing i16 value -100 to D1 (as u16)...");
    let signed_value: i16 = -100;
    let unsigned_repr = signed_value as u16; // Two's complement
    client.write(MemoryArea::DM, 1, &[unsigned_repr])?;
    let read_back = client.read(MemoryArea::DM, 1, 1)?;
    let read_signed = read_back[0] as i16;
    println!("   Read back as u16: {}, as i16: {} (expected: -100)", read_back[0], read_signed);
    assert_eq!(read_signed, -100);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Test 4: Write f32 (uses D1 and D2 = 4 bytes)
    println!("4. Writing f32 value 3.14159 to D1-D2...");
    client.write_f32(MemoryArea::DM, 1, 3.14159)?;
    let read_back = client.read_f32(MemoryArea::DM, 1)?;
    println!("   Read back: {} (expected: ~3.14159)", read_back);
    assert!((read_back - 3.14159).abs() < 0.0001);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Test 5: Write i32 (uses D1 and D2 = 4 bytes)
    println!("5. Writing i32 value -123456 to D1-D2...");
    client.write_i32(MemoryArea::DM, 1, -123456)?;
    let read_back = client.read_i32(MemoryArea::DM, 1)?;
    println!("   Read back: {} (expected: -123456)", read_back);
    assert_eq!(read_back, -123456);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Test 6: Write multiple words at once
    println!("6. Writing multiple words [100, 200] to D1-D2...");
    client.write(MemoryArea::DM, 1, &[100, 200])?;
    let read_back = client.read(MemoryArea::DM, 1, 2)?;
    println!("   Read back: {:?} (expected: [100, 200])", read_back);
    assert_eq!(read_back, vec![100, 200]);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Test 7: Write hex values
    println!("7. Writing hex values [0xABCD, 0x1234] to D1-D2...");
    client.write(MemoryArea::DM, 1, &[0xABCD, 0x1234])?;
    let read_back = client.read(MemoryArea::DM, 1, 2)?;
    println!("   Read back: [0x{:04X}, 0x{:04X}] (expected: [0xABCD, 0x1234])", read_back[0], read_back[1]);
    assert_eq!(read_back, vec![0xABCD, 0x1234]);
    println!("   OK!\n");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Clean up - write zeros
    println!("8. Cleaning up - writing zeros to D1-D2...");
    client.write(MemoryArea::DM, 1, &[0, 0])?;
    let read_back = client.read(MemoryArea::DM, 1, 2)?;
    println!("   Read back: {:?}", read_back);
    println!("   OK!\n");

    println!("=== All write tests passed! ===");

    Ok(())
}
