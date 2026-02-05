//! Example: PLC setup and control operations
//!
//! Run with: cargo run --example simple_setup
//!
//! This example demonstrates:
//! - Client configuration with custom settings
//! - PLC run/stop control
//! - Network addressing configuration
//! - Error handling patterns

use omron_fins::{Client, ClientConfig, FinsError, PlcMode};
use std::net::Ipv4Addr;
use std::time::Duration;

fn main() -> omron_fins::Result<()> {
    // =========================================================================
    // Basic Configuration
    // =========================================================================
    //
    // ClientConfig::new(ip, source_node, dest_node) creates a basic configuration:
    // - ip: PLC IP address
    // - source_node: This client's node number (typically 1-254)
    // - dest_node: PLC's node number (typically 0 for direct connection)

    let basic_config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0);
    println!("Basic config created: {:?}", basic_config);

    // =========================================================================
    // Advanced Configuration
    // =========================================================================
    //
    // For complex setups (multi-network, custom ports, longer timeouts):

    let advanced_config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 250), 1, 0)
        // Custom port (default is 9600)
        .with_port(9600)
        // Longer timeout for slow networks (default is 2 seconds)
        .with_timeout(Duration::from_secs(5))
        // Source network/unit for multi-network setups
        .with_source_network(0)
        .with_source_unit(0)
        // Destination network/unit
        .with_dest_network(0)
        .with_dest_unit(0);

    println!("Advanced config created: {:?}", advanced_config);

    // =========================================================================
    // Network Addressing Explained
    // =========================================================================
    //
    // FINS uses a 3-component address: Network.Node.Unit
    //
    // | Component | Description              | Typical Value        |
    // |-----------|--------------------------|---------------------|
    // | Network   | Network number           | 0 (local network)   |
    // | Node      | Node number (1-254)      | Based on IP octet   |
    // | Unit      | Unit number              | 0 (CPU unit)        |
    //
    // For simple setups on the same network, only Node is important.
    // Network 0 means "local network" (no routing needed).

    // =========================================================================
    // Creating and Connecting a Client
    // =========================================================================
    //
    // Note: This example creates a client but actual communication requires
    // a real PLC on the network. Uncomment the operations below when connected.

    println!("\nAttempting to connect...");

    // Use basic config for demonstration
    match Client::new(basic_config) {
        Ok(client) => {
            println!("Client created successfully!");
            println!("  Source: {:?}", client.source());
            println!("  Destination: {:?}", client.destination());

            // =================================================================
            // PLC Control Operations (uncomment when connected to real PLC)
            // =================================================================

            // Stop the PLC
            // WARNING: This will stop PLC execution!
            // client.stop()?;
            // println!("PLC stopped");

            // Start PLC in different modes:
            //
            // PlcMode::Debug   - Step-by-step execution (for debugging)
            // PlcMode::Monitor - Normal execution with monitoring capability
            // PlcMode::Run     - Normal execution

            // client.run(PlcMode::Monitor)?;
            // println!("PLC running in Monitor mode");

            // client.run(PlcMode::Run)?;
            // println!("PLC running in Run mode");

            // client.run(PlcMode::Debug)?;
            // println!("PLC running in Debug mode");

            // =================================================================
            // Example: Safe mode switching pattern
            // =================================================================

            /*
            fn safe_switch_to_run(client: &Client) -> omron_fins::Result<()> {
                // First stop the PLC
                client.stop()?;
                println!("PLC stopped for mode change");

                // Wait a moment for PLC to fully stop
                std::thread::sleep(Duration::from_millis(500));

                // Now start in desired mode
                client.run(PlcMode::Monitor)?;
                println!("PLC started in Monitor mode");

                Ok(())
            }
            */
        }
        Err(FinsError::Io(e)) => {
            println!("Connection error (expected if no PLC): {}", e);
            println!("\nTo test this example, ensure:");
            println!("  1. PLC is powered on and connected to network");
            println!("  2. PLC IP address matches the configuration");
            println!("  3. FINS UDP port (9600) is not blocked");
        }
        Err(e) => {
            println!("Unexpected error: {}", e);
        }
    }

    // =========================================================================
    // Error Handling Patterns
    // =========================================================================

    println!("\n--- Error Handling Examples ---");

    // Pattern 1: Simple propagation with ?
    fn example_simple(client: &Client) -> omron_fins::Result<()> {
        client.stop()?;
        client.run(PlcMode::Monitor)?;
        Ok(())
    }

    // Pattern 2: Match specific errors
    fn example_match_errors(client: &Client) {
        match client.stop() {
            Ok(()) => println!("Stop successful"),
            Err(FinsError::Timeout) => println!("Timeout - check network connection"),
            Err(FinsError::PlcError { main_code, sub_code }) => {
                println!("PLC error: main=0x{:02X}, sub=0x{:02X}", main_code, sub_code);
                // Check specific error codes here
            }
            Err(e) => println!("Other error: {}", e),
        }
    }

    // Pattern 3: Retry logic
    fn example_retry(client: &Client, max_retries: u32) -> omron_fins::Result<()> {
        for attempt in 1..=max_retries {
            match client.stop() {
                Ok(()) => return Ok(()),
                Err(FinsError::Timeout) if attempt < max_retries => {
                    println!("Attempt {} timed out, retrying...", attempt);
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    // Suppress unused function warnings for examples
    let _ = example_simple;
    let _ = example_match_errors;
    let _ = example_retry;

    println!("\nSetup example completed!");
    println!("See simple_read.rs and simple_write.rs for data operations.");

    Ok(())
}
