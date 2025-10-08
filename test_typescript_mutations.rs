// Quick integration test for TypeScript mutation generation
// Run with: cargo run --features typescript-ast --bin test_typescript_mutations

use std::fs;

fn main() -> anyhow::Result<()> {
    // Read calculator.ts
    let source = fs::read_to_string("fixtures/typescript/calculator.ts")?;

    println!("📝 Source file loaded: {} bytes\n", source.len());

    // Create mutation generator
    let generator = pmat::services::mutation::TypeScriptMutationGenerator::with_default_operators();

    println!("🔧 Generating mutants...\n");

    // Generate mutants
    let mutants = generator.generate_mutants(&source, "fixtures/typescript/calculator.ts")?;

    println!("✅ Generated {} mutants!\n", mutants.len());

    // Show first 10 mutants
    for (i, mutant) in mutants.iter().take(10).enumerate() {
        println!("Mutant #{}: {} at {}:{}",
            i + 1,
            mutant.id,
            mutant.location.line,
            mutant.location.column
        );

        // Show first 100 chars of mutated code
        let preview = mutant.mutated_source.chars().take(100).collect::<String>();
        println!("  Preview: {}...\n", preview.replace('\n', " "));
    }

    if mutants.len() > 10 {
        println!("... and {} more mutants\n", mutants.len() - 10);
    }

    // Count by operator type
    use std::collections::HashMap;
    let mut operator_counts: HashMap<String, usize> = HashMap::new();
    for mutant in &mutants {
        *operator_counts.entry(format!("{:?}", mutant.operator)).or_insert(0) += 1;
    }

    println!("📊 Mutants by operator:");
    for (op, count) in operator_counts.iter() {
        println!("  {}: {}", op, count);
    }

    Ok(())
}
