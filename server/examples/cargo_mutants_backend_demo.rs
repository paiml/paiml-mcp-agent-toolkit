//! GREEN Phase Example for PMAT-070-003: cargo-mutants Backend
//!
//! Demonstrates usage of cargo-mutants backend via `pmat mutate --use-cargo-mutants`
//!
//! Usage:
//! ```bash
//! # Use cargo-mutants backend instead of built-in PMAT mutation testing
//! cargo run --example cargo_mutants_backend_demo
//! ```

use pmat::cli::handlers::cargo_mutants_backend;
use pmat::services::mutation::json_parser::CargoMutantsReport;
use std::path::PathBuf;

fn main() {
    println!("🧪 cargo-mutants Backend Demo (GREEN Phase)\n");

    println!("Expected workflow after GREEN phase implementation:\n");

    println!("1. Detect cargo-mutants installation");
    println!("   CargoMutantsWrapper::new() -> Result<Wrapper>");
    println!();

    println!("2. Validate minimum version (v24.7.0+)");
    println!("   wrapper.validate_version() -> Result<()>");
    println!();

    println!("3. Execute cargo-mutants with options:");
    println!("   cargo mutants --output json \\");
    println!("                 --timeout 300 \\");
    println!("                 --jobs 4 \\");
    println!("                 --features feat1,feat2 \\");
    println!("                 --no-shuffle");
    println!();

    println!("4. Parse JSON output:");
    println!("   let report = CargoMutantsReport::from_json(json_output)?;");
    println!();

    println!("5. Display statistics:");
    println!("   - Total mutants: {}", "<from report.mutants.len()>");
    println!("   - Mutation score: {}", "<from report.mutation_score()>");
    println!("   - Caught: {}", "<from report.count_by_outcome(Caught)>");
    println!("   - Missed: {}", "<from report.count_by_outcome(Missed)>");
    println!(
        "   - Timeout: {}",
        "<from report.count_by_outcome(Timeout)>"
    );
    println!(
        "   - Unviable: {}",
        "<from report.count_by_outcome(Unviable)>"
    );
    println!();

    println!("6. Optionally save to file:");
    println!("   std::fs::write(output_path, json_output)?;");
    println!();

    println!("📝 CLI Usage Examples (after GREEN phase):");
    println!();
    println!("   # Use cargo-mutants backend");
    println!("   pmat mutate --target . --use-cargo-mutants");
    println!();
    println!("   # With specific features");
    println!("   pmat mutate --target . --use-cargo-mutants --features serde,tokio");
    println!();
    println!("   # All features enabled");
    println!("   pmat mutate --target . --use-cargo-mutants --all-features");
    println!();
    println!("   # Custom timeout and parallel jobs");
    println!("   pmat mutate --target . --use-cargo-mutants --timeout 600 --jobs 8");
    println!();
    println!("   # Save results to file");
    println!("   pmat mutate --target . --use-cargo-mutants --output mutation-report.json");
    println!();

    println!("✅ GREEN phase: Working implementation!\n");

    // Demonstrate actual usage (requires cargo-mutants installation)
    println!("Attempting to execute cargo-mutants backend...\n");

    let result = cargo_mutants_backend::execute(
        PathBuf::from("."),
        None,  // No output file
        300,   // 5 minute timeout
        None,  // Auto-detect jobs
        None,  // No specific features
        false, // Not all features
        false, // Use default features
        false, // Shuffle enabled
    );

    match result {
        Ok(json) => {
            println!("✅ cargo-mutants executed successfully!\n");

            // Parse and display results
            match CargoMutantsReport::from_json(&json) {
                Ok(report) => {
                    cargo_mutants_backend::display_statistics(&report);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to parse results: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Demo failed: {}", e);
            eprintln!("\nThis is expected if:");
            eprintln!("  1. cargo-mutants is not installed");
            eprintln!("  2. cargo-mutants version is < v24.7.0");
            eprintln!("  3. Not running in a Rust project directory");
            eprintln!("\nInstall: cargo install cargo-mutants");
        }
    }
}
