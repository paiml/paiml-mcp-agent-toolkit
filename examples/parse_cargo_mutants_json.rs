//! GREEN Phase Example for PMAT-070-002: JSON Parsing
//!
//! Example demonstrating cargo-mutants JSON parsing.
//! Now using real implementation from json_parser.rs

use pmat::services::mutation::json_parser::{CargoMutantsReport, MutantOutcome};
use std::process;

fn main() {
    println!("🧪 cargo-mutants JSON Parser - GREEN Phase Example\n");

    // Sample cargo-mutants JSON output (from Phase 2 kickoff guide)
    let sample_json = r#"{
  "mutants": [
    {
      "outcome": "caught",
      "file": "src/lib.rs",
      "function": "add",
      "line": 10,
      "replacement": "0"
    },
    {
      "outcome": "missed",
      "file": "src/lib.rs",
      "function": "subtract",
      "line": 15,
      "replacement": "1"
    },
    {
      "outcome": "timeout",
      "file": "src/lib.rs",
      "function": "multiply",
      "line": 20,
      "replacement": "panic!()"
    },
    {
      "outcome": "unviable",
      "file": "src/lib.rs",
      "function": "divide",
      "line": 25,
      "replacement": "compile_error!()"
    }
  ]
}"#;

    println!("📄 Sample cargo-mutants JSON:");
    println!("{}\n", sample_json);

    // GREEN Phase: Parse and convert JSON
    // Deliberately the deprecated legacy-format entry point: this example
    // demonstrates parsing a JSON string, and the replacement
    // (`from_output_dir`) reads a directory instead.
    #[allow(deprecated)]
    let parsed = CargoMutantsReport::from_json(sample_json);
    match parsed {
        Ok(report) => {
            println!("✅ Parsed {} mutants from JSON\n", report.mutants.len());

            // Convert to PMAT format
            let pmat_report = report.to_pmat_report();
            println!(
                "✅ Converted to PMAT format ({} mutants)\n",
                pmat_report.len()
            );

            // Calculate statistics
            let total = report.mutants.len();
            let caught = report
                .mutants
                .iter()
                .filter(|m| matches!(m.outcome, MutantOutcome::Caught))
                .count();
            let missed = report
                .mutants
                .iter()
                .filter(|m| matches!(m.outcome, MutantOutcome::Missed))
                .count();
            let timeout = report
                .mutants
                .iter()
                .filter(|m| matches!(m.outcome, MutantOutcome::Timeout))
                .count();
            let unviable = report
                .mutants
                .iter()
                .filter(|m| matches!(m.outcome, MutantOutcome::Unviable))
                .count();

            let mutation_score = if total > 0 {
                (caught as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            // Display results
            println!("📊 Mutation Testing Results:");
            println!("   Total mutants: {}", total);
            println!("   Caught: {} ({:.1}%)", caught, mutation_score);
            println!(
                "   Missed: {} ({:.1}%)",
                missed,
                (missed as f64 / total as f64) * 100.0
            );
            println!(
                "   Timeout: {} ({:.1}%)",
                timeout,
                (timeout as f64 / total as f64) * 100.0
            );
            println!(
                "   Unviable: {} ({:.1}%)\n",
                unviable,
                (unviable as f64 / total as f64) * 100.0
            );

            println!("📈 Mutation Score: {:.1}%", mutation_score);

            if mutation_score >= 90.0 {
                println!("✅ Excellent! Test suite quality is very high");
            } else if mutation_score >= 75.0 {
                println!("⚠️  Good, but room for improvement");
            } else {
                println!("❌ Test suite needs significant improvement");
            }

            println!("\n✅ GREEN Phase implementation complete!");
        }
        Err(e) => {
            eprintln!("❌ Failed to parse JSON: {}", e);
            process::exit(1);
        }
    }
}
