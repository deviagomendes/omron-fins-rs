//! Example: Writing data to PLC memory
//!
//! Run with: cargo run --example simple_write
//!
//! This example demonstrates:
//! - Writing words to different memory areas
//! - Writing individual bits
//! - Type conversions for write operations
//! - Fill and transfer operations
//! - Forced set/reset operations

use omron_fins::{Client, ClientConfig, ForcedBit, ForceSpec, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    // =========================================================================
    // Connect to PLC
    // =========================================================================
    
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
    let client = Client::new(config)?;

    // =========================================================================
    // Writing Words (16-bit values)
    // =========================================================================
    
    println!("=== Writing Words ===\n");
    
    // Write single word to DM area
    // The write method accepts a slice of u16 values
    client.write(MemoryArea::DM, 0, &[1234])?;
    println!("Wrote 1234 to DM0");
    
    // Write multiple words at once
    client.write(MemoryArea::DM, 100, &[100, 200, 300, 400, 500])?;
    println!("Wrote [100, 200, 300, 400, 500] to DM100-DM104");
    
    // Write from different formats - all automatically convert to u16
    
    // From decimal values
    client.write(MemoryArea::DM, 110, &[1000, 2000, 3000])?;
    
    // From hexadecimal values
    client.write(MemoryArea::DM, 120, &[0x1234, 0xABCD, 0xFF00])?;
    
    // From binary values
    client.write(MemoryArea::DM, 130, &[0b1010_1010, 0b1111_0000])?;
    
    println!("Wrote values in different formats");

    // =========================================================================
    // Writing to Different Memory Areas
    // =========================================================================
    
    println!("\n=== Writing to Different Areas ===\n");
    
    // CIO (Core I/O) - for outputs and internal relays
    client.write(MemoryArea::CIO, 100, &[0x00FF])?;
    println!("Wrote 0x00FF to CIO100");
    
    // WR (Work) - for temporary work data
    client.write(MemoryArea::WR, 0, &[42])?;
    println!("Wrote 42 to WR0");
    
    // HR (Holding) - retentive data that survives power cycles
    client.write(MemoryArea::HR, 0, &[9999])?;
    println!("Wrote 9999 to HR0");

    // =========================================================================
    // Writing Bits
    // =========================================================================
    
    println!("\n=== Writing Bits ===\n");
    
    // Write individual bits (not supported on DM area)
    client.write_bit(MemoryArea::CIO, 100, 0, true)?;
    println!("Set CIO100.00 to ON");
    
    client.write_bit(MemoryArea::CIO, 100, 1, false)?;
    println!("Set CIO100.01 to OFF");
    
    // Set multiple bits in sequence
    for bit in 0..8 {
        client.write_bit(MemoryArea::CIO, 200, bit, bit % 2 == 0)?;
    }
    println!("Set CIO200 bits 0,2,4,6 to ON and 1,3,5,7 to OFF");

    // =========================================================================
    // Type Conversions for Writing
    // =========================================================================
    
    println!("\n=== Type Conversions ===\n");
    
    // Write f32 (REAL) - automatically converts to 2 words
    client.write_f32(MemoryArea::DM, 200, 3.14159)?;
    println!("Wrote f32 3.14159 to DM200-201");
    
    // Write f64 (LREAL) - automatically converts to 4 words
    client.write_f64(MemoryArea::DM, 210, 3.141592653589793)?;
    println!("Wrote f64 3.141592653589793 to DM210-213");
    
    // Write i32 (DINT) - automatically converts to 2 words
    client.write_i32(MemoryArea::DM, 220, -123456)?;
    println!("Wrote i32 -123456 to DM220-221");
    
    // Write ASCII string - automatically converts to words (2 chars per word)
    client.write_string(MemoryArea::DM, 230, "PRODUCT-001")?;
    println!("Wrote string \"PRODUCT-001\" to DM230+");

    // =========================================================================
    // Manual Conversions
    // =========================================================================
    
    println!("\n=== Manual Conversions ===\n");
    
    // Convert larger types to u16 slices manually
    
    // u32 to two u16 (big-endian)
    let value_u32: u32 = 0x12345678;
    let words_be = [
        (value_u32 >> 16) as u16,  // High word: 0x1234
        (value_u32 & 0xFFFF) as u16, // Low word: 0x5678
    ];
    client.write(MemoryArea::DM, 300, &words_be)?;
    println!("Wrote u32 {} as BE to DM300-301", value_u32);
    
    // u32 to two u16 (little-endian / word-swapped)
    let words_le = [
        (value_u32 & 0xFFFF) as u16, // Low word first
        (value_u32 >> 16) as u16,    // High word second
    ];
    client.write(MemoryArea::DM, 302, &words_le)?;
    println!("Wrote u32 {} as LE to DM302-303", value_u32);
    
    // BCD encoding
    fn decimal_to_bcd(value: u16) -> u16 {
        let d0 = value % 10;
        let d1 = (value / 10) % 10;
        let d2 = (value / 100) % 10;
        let d3 = (value / 1000) % 10;
        (d3 << 12) | (d2 << 8) | (d1 << 4) | d0
    }
    
    let bcd_value = decimal_to_bcd(1234);
    client.write(MemoryArea::DM, 310, &[bcd_value])?;
    println!("Wrote 1234 as BCD (0x{:04X}) to DM310", bcd_value);

    // =========================================================================
    // Fill Operation
    // =========================================================================
    
    println!("\n=== Fill Operation ===\n");
    
    // Fill a range with a single value (efficient for initialization)
    client.fill(MemoryArea::DM, 400, 100, 0x0000)?;
    println!("Filled DM400-DM499 with 0x0000 (100 words)");
    
    client.fill(MemoryArea::DM, 500, 50, 0xFFFF)?;
    println!("Filled DM500-DM549 with 0xFFFF (50 words)");

    // =========================================================================
    // Transfer Operation
    // =========================================================================
    
    println!("\n=== Transfer Operation ===\n");
    
    // Copy data within PLC memory (PLC-side operation, very fast)
    client.transfer(MemoryArea::DM, 0, MemoryArea::DM, 600, 10)?;
    println!("Transferred DM0-DM9 to DM600-DM609");
    
    // Transfer between different areas
    client.transfer(MemoryArea::DM, 100, MemoryArea::WR, 100, 5)?;
    println!("Transferred DM100-DM104 to WR100-WR104");

    // =========================================================================
    // Forced Set/Reset (Maintenance Mode)
    // =========================================================================
    
    println!("\n=== Forced Set/Reset ===\n");
    
    // WARNING: Forced bits override PLC program control!
    // Use only for maintenance and testing.
    
    // Force multiple bits at once
    client.forced_set_reset(&[
        ForcedBit { area: MemoryArea::CIO, address: 300, bit: 0, spec: ForceSpec::ForceOn },
        ForcedBit { area: MemoryArea::CIO, address: 300, bit: 1, spec: ForceSpec::ForceOff },
    ])?;
    println!("Forced CIO300.00 ON and CIO300.01 OFF");
    
    // Release forced state (return to normal program control)
    client.forced_set_reset(&[
        ForcedBit { area: MemoryArea::CIO, address: 300, bit: 0, spec: ForceSpec::Release },
        ForcedBit { area: MemoryArea::CIO, address: 300, bit: 1, spec: ForceSpec::Release },
    ])?;
    println!("Released forced state on CIO300.00 and CIO300.01");
    
    // Cancel ALL forced bits in PLC
    client.forced_set_reset_cancel()?;
    println!("Cancelled all forced bits");

    // =========================================================================
    // Batch Write Pattern
    // =========================================================================
    
    println!("\n=== Batch Write Pattern ===\n");
    
    // Efficient pattern: prepare data, then write once
    let sensor_data: Vec<u16> = (0..10).map(|i| i * 100).collect();
    client.write(MemoryArea::DM, 700, &sensor_data)?;
    println!("Wrote sensor data batch to DM700-DM709");
    
    // Recipe write pattern
    struct Recipe {
        id: u16,
        speed: u16,
        temperature: f32,
        name: &'static str,
    }
    
    let recipe = Recipe {
        id: 42,
        speed: 1500,
        temperature: 75.5,
        name: "RECIPE-A",
    };
    
    // Write recipe fields
    client.write(MemoryArea::DM, 800, &[recipe.id])?;
    client.write(MemoryArea::DM, 801, &[recipe.speed])?;
    client.write_f32(MemoryArea::DM, 802, recipe.temperature)?;
    client.write_string(MemoryArea::DM, 804, recipe.name)?;
    println!("Wrote recipe '{}' to DM800+", recipe.name);

    println!("\nWrite example completed!");
    Ok(())
}
