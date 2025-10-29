//! GREEN Phase Example for PMAT-070-001: cargo-mutants Detection
//!
//! Demonstrates PATH detection and version checking for cargo-mutants.
//!
//! Usage:
//! ```
//! cargo run --example cargo_mutants_detect
//! ```
//!
//! Expected output (when cargo-mutants is installed):
//! ```
//! ✅ cargo-mutants found: cargo
//! ✅ Version: cargo-mutants 25.3.1 (meets minimum v24.7.0)
//! ```

use std::process;
use pmat::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;

fn main() {
    println!("🔍 Detecting cargo-mutants installation...\n");

    match CargoMutantsWrapper::new() {
        Ok(wrapper) => {
            if wrapper.is_installed() {
                println!("✅ cargo-mutants found: {:?}", wrapper.cargo_mutants_path.as_ref().unwrap());

                match wrapper.version() {
                    Ok(version) => {
                        println!("✅ Version: {} (meets minimum v24.7.0)", version);

                        // Validate version meets minimum requirement
                        match wrapper.validate_version() {
                            Ok(_) => println!("✅ Version validation passed"),
                            Err(e) => {
                                eprintln!("❌ Version validation failed: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Could not determine version: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("❌ cargo-mutants not found in PATH");
                eprintln!("\n📦 Installation Instructions:");
                eprintln!("   cargo install cargo-mutants");
                eprintln!("\n   After installation, retry this example.");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Error initializing wrapper: {}", e);
            process::exit(1);
        }
    }

    println!("\n✅ All checks passed! cargo-mutants wrapper is ready to use.");
    process::exit(0);
}
