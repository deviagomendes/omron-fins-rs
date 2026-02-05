//! Example: Reading data from PLC memory
//!
//! Run with: cargo run --example simple_read
//!
//! This example demonstrates:
//! - Reading words from different memory areas
//! - Reading individual bits
//! - Type conversions (f32, f64, i32, strings)
//! - Using utility functions for bit analysis

use omron_fins::{Client, ClientConfig, MemoryArea};
use omron_fins::utils::{print_bits, format_binary, format_hex, get_on_bits, word_to_bits};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    // =========================================================================
    // Connect to PLC
    // =========================================================================
    
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
    let client = Client::new(config)?;

    // =========================================================================
    // Reading Words (16-bit values)
    // =========================================================================
    
    println!("=== Reading Words ===\n");
    
    // Read single word from DM area
    let data = client.read(MemoryArea::DM, 0, 1)?;
    println!("DM0 = {} (0x{:04X})", data[0], data[0]);
    
    // Read multiple words
    let data = client.read(MemoryArea::DM, 100, 5)?;
    println!("DM100-DM104: {:?}", data);
    
    // Read from different memory areas
    let cio_data = client.read(MemoryArea::CIO, 0, 1)?;
    let wr_data = client.read(MemoryArea::WR, 0, 1)?;
    let hr_data = client.read(MemoryArea::HR, 0, 1)?;
    
    println!("CIO0 = 0x{:04X}", cio_data[0]);
    println!("WR0  = 0x{:04X}", wr_data[0]);
    println!("HR0  = 0x{:04X}", hr_data[0]);

    // =========================================================================
    // Reading Bits
    // =========================================================================
    
    println!("\n=== Reading Bits ===\n");
    
    // Read individual bit (CIO 0.05)
    let bit = client.read_bit(MemoryArea::CIO, 0, 5)?;
    println!("CIO 0.05 = {}", bit);
    
    // Read a word and analyze its bits
    let value = client.read(MemoryArea::CIO, 100, 1)?[0];
    println!("\nCIO100 = {} ({})", value, format_hex(value));
    println!("Binary: {}", format_binary(value));
    
    // Get list of ON bits
    let on_bits = get_on_bits(value);
    println!("Bits that are ON: {:?}", on_bits);
    
    // Print all bits with indices
    println!("\nAll bits of CIO100:");
    print_bits(value);
    
    // Convert to array for programmatic access
    let bits_array = word_to_bits(value);
    for (i, bit_value) in bits_array.iter().enumerate() {
        if *bit_value {
            println!("  Bit {} is ON", i);
        }
    }

    // =========================================================================
    // Type Conversions
    // =========================================================================
    
    println!("\n=== Type Conversions ===\n");
    
    // Read f32 (REAL) - 2 words
    // Omron uses word-swapped big-endian format
    let temperature: f32 = client.read_f32(MemoryArea::DM, 200)?;
    println!("Temperature (f32 from DM200-201): {:.2}°C", temperature);
    
    // Read f64 (LREAL) - 4 words
    let precision_value: f64 = client.read_f64(MemoryArea::DM, 210)?;
    println!("Precision value (f64 from DM210-213): {:.10}", precision_value);
    
    // Read i32 (DINT) - 2 words
    let counter: i32 = client.read_i32(MemoryArea::DM, 220)?;
    println!("Counter (i32 from DM220-221): {}", counter);
    
    // Read ASCII string - variable words (2 chars per word)
    let product_code: String = client.read_string(MemoryArea::DM, 230, 10)?;
    println!("Product code (string from DM230, 10 words): \"{}\"", product_code);

    // =========================================================================
    // Conversion Examples (from raw words)
    // =========================================================================
    
    println!("\n=== Manual Conversions ===\n");
    
    // Example: Converting words to different formats
    let raw_words = client.read(MemoryArea::DM, 300, 4)?;
    println!("Raw words: {:?}", raw_words);
    
    // Interpret as unsigned integers
    println!("As u16: {:?}", raw_words);
    
    // Interpret as signed integers
    let signed: Vec<i16> = raw_words.iter().map(|&w| w as i16).collect();
    println!("As i16: {:?}", signed);
    
    // Convert two words to u32 (big-endian)
    let u32_value = ((raw_words[0] as u32) << 16) | (raw_words[1] as u32);
    println!("Words [0,1] as u32 (BE): {}", u32_value);
    
    // Convert two words to u32 (little-endian)
    let u32_value_le = ((raw_words[1] as u32) << 16) | (raw_words[0] as u32);
    println!("Words [0,1] as u32 (LE): {}", u32_value_le);
    
    // BCD conversion (if data is BCD encoded)
    fn bcd_to_decimal(bcd: u16) -> u16 {
        let d0 = bcd & 0x000F;
        let d1 = (bcd >> 4) & 0x000F;
        let d2 = (bcd >> 8) & 0x000F;
        let d3 = (bcd >> 12) & 0x000F;
        d3 * 1000 + d2 * 100 + d1 * 10 + d0
    }
    
    println!("Word 0 as BCD: {}", bcd_to_decimal(raw_words[0]));

    // =========================================================================
    // Multiple Read (Single Request)
    // =========================================================================
    
    println!("\n=== Multiple Read ===\n");
    
    use omron_fins::MultiReadSpec;
    
    // Read from multiple addresses in one request (more efficient)
    let values = client.read_multiple(&[
        MultiReadSpec { area: MemoryArea::DM, address: 0, bit: None },
        MultiReadSpec { area: MemoryArea::DM, address: 100, bit: None },
        MultiReadSpec { area: MemoryArea::CIO, address: 0, bit: Some(5) },
    ])?;
    
    println!("DM0 = {}", values[0]);
    println!("DM100 = {}", values[1]);
    println!("CIO0.05 = {} (0=OFF, 1=ON)", values[2]);

    // =========================================================================
    // Display Formatting Examples
    // =========================================================================
    
    println!("\n=== Display Formatting ===\n");
    
    let sample: u16 = 0xA5C3;
    println!("Sample value: {}", sample);
    println!("  Decimal:     {}", sample);
    println!("  Hexadecimal: {}", format_hex(sample));
    println!("  Binary:      {}", format_binary(sample));
    println!("  Bits ON:     {:?}", get_on_bits(sample));

    println!("\nRead example completed!");
    Ok(())
}
