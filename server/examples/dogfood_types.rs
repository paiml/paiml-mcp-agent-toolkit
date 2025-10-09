// Dogfooding: Test PMAT's mutation/types.rs with PMAT's Rust mutation testing
// Sprint 25 - Phase 1: Baseline mutation testing

use anyhow::Result;
use pmat::services::mutation::{RustMutationGenerator, MutationScore};

fn main() -> Result<()> {
    println!("🦀 Sprint 25: Dogfooding PMAT with PMAT\n");
    println!("Testing: services/mutation/types.rs");
    println!("=" .repeat(60));

    // Read the types.rs source
    let source = std::fs::read_to_string("src/services/mutation/types.rs")?;
    let lines = source.lines().count();

    println!("\n📊 Module Statistics:");
    println!("   File: src/services/mutation/types.rs");
    println!("   Lines: {}", lines);
    println!("   Size: {} bytes", source.len());

    // Generate mutants
    println!("\n🔧 Generating mutants...");
    let generator = RustMutationGenerator::with_default_operators();
    let start = std::time::Instant::now();
    let mutants = generator.generate_mutants(&source, "types.rs")?;
    let generation_time = start.elapsed();

    println!("   Generated: {} mutants", mutants.len());
    println!("   Time: {:?}", generation_time);

    if mutants.is_empty() {
        println!("\n⚠️  No mutants generated!");
        return Ok(());
    }

    // Show mutant breakdown by operator
    println!("\n📋 Mutant Breakdown:");
    let mut operator_counts = std::collections::HashMap::new();
    for mutant in &mutants {
        *operator_counts.entry(format!("{:?}", mutant.operator)).or_insert(0) += 1;
    }

    let mut operators: Vec<_> = operator_counts.iter().collect();
    operators.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    for (op, count) in operators {
        println!("   {:30} : {}", op, count);
    }

    // Show sample mutants
    println!("\n🔍 Sample Mutants (first 5):");
    for (i, mutant) in mutants.iter().take(5).enumerate() {
        println!("\n   {}. Mutant ID: {}", i + 1, mutant.id);
        println!("      Location: line {}, col {}", mutant.location.line, mutant.location.column);
        println!("      Operator: {:?}", mutant.operator);

        // Show the mutated line
        if let Some(line) = mutant.mutated_source.lines().nth(mutant.location.line - 1) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                println!("      Code: {}", if trimmed.len() > 80 {
                    format!("{}...", &trimmed[..77])
                } else {
                    trimmed.to_string()
                });
            }
        }
    }

    println!("\n" + &"=".repeat(60));
    println!("\n✅ Mutation generation successful!");
    println!("\n📝 Next Steps:");
    println!("   1. Run test suite against each mutant");
    println!("   2. Calculate mutation score");
    println!("   3. Analyze surviving mutants");
    println!("   4. Add missing tests");
    println!("\n🎯 Expected mutation score: 50-70% (baseline)");
    println!("🎯 Target mutation score: 80%+");

    Ok(())
}
