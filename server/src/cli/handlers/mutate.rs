//! Mutation testing handler (Sprint 61)
//!
//! Exposes PMAT's AST-based mutation testing infrastructure via CLI command.

use crate::cli::commands::MutateArgs;
use crate::services::mutation::engine::{MutationEngine, MutationConfig, MutationStrategy};
use crate::services::mutation::types::{MutationResult, MutationScore};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{Context, Result};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

/// Handle mutation testing command
pub async fn handle(
    args: MutateArgs,
    _server: Arc<StatelessTemplateServer>,
) -> Result<()> {
    info!("Starting mutation testing on {:?}", args.target);

    // 1. Validate target
    let target = args
        .target
        .canonicalize()
        .context("Target file not found")?;

    // 2. Create engine
    let config = MutationConfig {
        strategy: MutationStrategy::Selective,
        max_mutants: 0,
        parallel_threads: args.jobs.unwrap_or_else(num_cpus::get),
    };
    let engine = MutationEngine::default_rust();

    // 3. Generate mutants
    let mutants = engine.generate_mutants_from_file(&target).await?;
    eprintln!("Generated {} mutants", mutants.len());

    // 4. Execute mutants
    let results = if config.parallel_threads > 1 {
        engine.execute_mutants_parallel(mutants).await?
    } else {
        engine.execute_mutants(mutants).await?
    };

    // 5. Calculate score
    let score = MutationScore::from_results(&results);

    // 6. Output
    match args.output_format.as_str() {
        "json" => output_json(&score, &results)?,
        "markdown" => output_markdown(&score, &results)?,
        _ => output_text(&score, &results)?,
    }

    // 7. Check threshold
    if let Some(threshold) = args.threshold {
        if score.score < threshold / 100.0 {
            anyhow::bail!(
                "Mutation score {:.1}% below threshold {:.1}%",
                score.score * 100.0,
                threshold
            );
        }
    }

    Ok(())
}

/// JSON output wrapper for serialization
#[derive(Serialize)]
struct MutationTestOutput {
    score: MutationScore,
    results: Vec<MutationResult>,
}

fn output_json(score: &MutationScore, results: &[MutationResult]) -> Result<()> {
    let output = MutationTestOutput {
        score: score.clone(),
        results: results.to_vec(),
    };
    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);
    Ok(())
}

fn output_markdown(score: &MutationScore, results: &[MutationResult]) -> Result<()> {
    println!("# Mutation Testing Results\n");
    println!("## Summary\n");
    println!("| Metric | Count | Percentage |");
    println!("|--------|-------|------------|");
    println!("| **Total Mutants** | {} | 100.0% |", score.total);

    if score.total > 0 {
        println!(
            "| Killed | {} | {:.1}% |",
            score.killed,
            (score.killed as f64 / score.total as f64) * 100.0
        );
        println!(
            "| Survived | {} | {:.1}% |",
            score.survived,
            (score.survived as f64 / score.total as f64) * 100.0
        );
        println!(
            "| Compile Errors | {} | {:.1}% |",
            score.compile_errors,
            (score.compile_errors as f64 / score.total as f64) * 100.0
        );
        println!(
            "| Timeouts | {} | {:.1}% |",
            score.timeouts,
            (score.timeouts as f64 / score.total as f64) * 100.0
        );
        println!(
            "| Equivalent | {} | {:.1}% |",
            score.equivalent,
            (score.equivalent as f64 / score.total as f64) * 100.0
        );
    }

    println!("\n## Mutation Score: **{:.1}%**\n", score.score * 100.0);

    // Show survived mutants for test improvements
    let survived: Vec<_> = results
        .iter()
        .filter(|r| r.status == crate::services::mutation::types::MutantStatus::Survived)
        .collect();

    if !survived.is_empty() {
        println!("## Survived Mutants (Test Gaps)\n");
        println!("The following mutants survived, indicating potential test coverage gaps:\n");

        for (i, result) in survived.iter().enumerate() {
            println!("### Mutant #{}", i + 1);
            println!("- **Location**: {}:{}:{}",
                result.mutant.original_file.display(),
                result.mutant.location.line,
                result.mutant.location.column
            );
            println!("- **Operator**: {:?}", result.mutant.operator);
            println!("- **Status**: Survived");
            println!();
        }
    }

    Ok(())
}

fn output_text(score: &MutationScore, _results: &[MutationResult]) -> Result<()> {
    println!("\nMutation Testing Results\n");
    println!("Total mutants:  {}", score.total);

    if score.total > 0 {
        println!(
            "Killed:         {} ({:.1}%)",
            score.killed,
            (score.killed as f64 / score.total as f64) * 100.0
        );
        println!(
            "Survived:       {} ({:.1}%)",
            score.survived,
            (score.survived as f64 / score.total as f64) * 100.0
        );

        if score.compile_errors > 0 {
            println!(
                "Compile errors: {} ({:.1}%)",
                score.compile_errors,
                (score.compile_errors as f64 / score.total as f64) * 100.0
            );
        }

        if score.timeouts > 0 {
            println!(
                "Timeouts:       {} ({:.1}%)",
                score.timeouts,
                (score.timeouts as f64 / score.total as f64) * 100.0
            );
        }

        if score.equivalent > 0 {
            println!(
                "Equivalent:     {} ({:.1}%)",
                score.equivalent,
                (score.equivalent as f64 / score.total as f64) * 100.0
            );
        }
    }

    println!("\nMutation Score: {:.1}%\n", score.score * 100.0);
    Ok(())
}
