// Quick integration test for TypeScript mutation generation
// Run with: cargo run --example test_typescript_mutations --features typescript-ast

use pmat::services::mutation::TypeScriptMutationGenerator;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    // Simple TypeScript source with all mutation types
    let source = r#"
export function add(a: number, b: number): number {
    return a + b;
}

export function isEqual(a: number, b: number): boolean {
    return a === b;
}

export function getNestedValue(obj?: { value?: number }): number | undefined {
    return obj?.value;
}

export function getValueOrDefault(value: number | null | undefined, defaultValue: number): number {
    return value ?? defaultValue;
}

export async function fetchValue(): Promise<number> {
    return await Promise.resolve(42);
}
"#;

    println!("📝 Testing TypeScript mutation generation\n");
    println!("Source code: {} bytes\n", source.len());

    // Create mutation generator with default operators
    let generator = TypeScriptMutationGenerator::with_default_operators();

    println!("🔧 Generating mutants...\n");

    // Generate mutants
    let mutants = generator.generate_mutants(source, "test.ts")?;

    println!("✅ Generated {} mutants!\n", mutants.len());

    if mutants.is_empty() {
        println!("⚠️  No mutants generated - operators may not be matching nodes");
        return Ok(());
    }

    // Show first 5 mutants
    println!("📋 First {} mutants:\n", mutants.len().min(5));
    for (i, mutant) in mutants.iter().take(5).enumerate() {
        println!("{}. {} at line {}:{}",
            i + 1,
            mutant.id,
            mutant.location.line,
            mutant.location.column
        );

        // Find the mutated line
        let lines: Vec<&str> = mutant.mutated_source.lines().collect();
        if mutant.location.line > 0 && mutant.location.line <= lines.len() {
            let line = lines[mutant.location.line - 1].trim();
            println!("   Code: {}", line);
        }
        println!();
    }

    // Count by operator type
    let mut operator_counts: HashMap<String, usize> = HashMap::new();
    for mutant in &mutants {
        *operator_counts.entry(format!("{:?}", mutant.operator)).or_insert(0) += 1;
    }

    println!("📊 Mutants by operator type:");
    for (op, count) in operator_counts.iter() {
        println!("  {}: {}", op, count);
    }
    println!();

    // Check mutation coverage
    let expected_mutations = vec![
        "ArithmeticReplacement",  // + in add()
        "RelationalReplacement",   // === in isEqual()
        "StatementDeletion",       // ?. and await
        "ConditionalReplacement",  // ??
    ];

    println!("🎯 Mutation Coverage:");
    for expected in &expected_mutations {
        let found = operator_counts.keys().any(|k| k.contains(expected));
        if found {
            println!("  ✅ {}", expected);
        } else {
            println!("  ❌ {} (not found)", expected);
        }
    }

    println!("\n🎉 TypeScript mutation generation test complete!");

    Ok(())
}
