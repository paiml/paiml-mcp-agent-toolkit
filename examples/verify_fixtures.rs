//! Verify test fixtures can be parsed correctly
//!
//! Run with: cargo run --example verify_fixtures

use pmat::services::mutation::json_parser::CargoMutantsReport;
use std::path::PathBuf;

fn main() {
    println!("🧪 Verifying cargo-mutants test fixtures\n");

    let fixtures = vec![
        ("some-missed", 5, 80.0),
        ("all-caught", 5, 100.0),
        ("empty", 0, 0.0),
        ("with-timeout", 5, 60.0), // 3 caught out of 5
        ("unviable", 5, 60.0),     // 3 caught out of 5
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (name, expected_mutants, expected_score) in fixtures {
        let fixture_path = PathBuf::from(format!(
            "server/tests/fixtures/cargo-mutants-output/{}",
            name
        ));

        print!("Testing {:15} ... ", name);

        match CargoMutantsReport::from_output_dir(&fixture_path) {
            Ok(report) => {
                let mutants = report.mutants.len();
                let score = report.mutation_score();

                if mutants == expected_mutants && (score - expected_score).abs() < 0.1 {
                    println!("✅ PASS ({} mutants, {:.1}% score)", mutants, score);
                    passed += 1;
                } else {
                    println!(
                        "❌ FAIL (expected {} mutants & {:.1}% score, got {} mutants & {:.1}% score)",
                        expected_mutants, expected_score, mutants, score
                    );
                    failed += 1;
                }
            }
            Err(e) => {
                println!("❌ FAIL (parse error: {})", e);
                failed += 1;
            }
        }
    }

    println!("\n📊 Results: {} passed, {} failed", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
