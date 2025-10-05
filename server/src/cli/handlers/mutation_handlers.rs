//! Mutation testing handlers
//!
//! Handles the `pmat analyze mutate` command for mutation testing with ML prediction.

use anyhow::{Result, Context};
use std::path::PathBuf;
use crate::cli::OutputFormat;
use crate::services::mutation::{MutationEngine, MutationConfig, RustAdapter, MutantExecutor, MutationScore};
use std::sync::Arc;

/// Handle mutation testing command
pub async fn handle_mutate(
    path: PathBuf,
    operators: Option<Vec<String>>,
    _ml_predict: bool,
    _distributed: bool,
    _workers: usize,
    _progress: bool,
    min_score: Option<f64>,
    _ci_learning: bool,
    _ci_provider: Option<String>,
    _auto_train_threshold: usize,
    format: OutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("🧬 Mutation Testing");
    println!("Path: {}", path.display());

    if let Some(ref ops) = operators {
        println!("Operators: {}", ops.join(", "));
    } else {
        println!("Operators: AOR, ROR, COR, UOR (default)");
    }

    // Check if path exists
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Create mutation engine with Rust adapter
    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    // Generate mutants
    println!("\n📝 Generating mutants...");
    let mutants = if path.is_file() {
        engine.generate_mutants_from_file(&path)
            .await
            .context("Failed to generate mutants")?
    } else {
        anyhow::bail!("Directory mutation testing not yet implemented. Please provide a file path.");
    };

    println!("✅ Generated {} mutants", mutants.len());

    if mutants.is_empty() {
        println!("\n⚠️  No mutants generated - file may be too simple or no applicable operators");
        return Ok(());
    }

    // Execute tests on mutants
    println!("\n🧪 Running tests on mutants...");
    let work_dir = path.parent()
        .and_then(|p| p.parent()) // Go up two levels to find cargo project root
        .or_else(|| path.parent())
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let executor = MutantExecutor::new(work_dir)
        .with_timeout(600); // 10 minute timeout per mutant

    let results = executor.execute_mutants(&mutants).await
        .context("Failed to execute mutants")?;

    // Calculate mutation score from actual results
    let score = MutationScore::from_results(&results);
    let mutation_score = score.score;

    // Check minimum score threshold
    if let Some(min) = min_score {
        if mutation_score < min {
            anyhow::bail!(
                "Mutation score {:.2}% is below threshold {:.2}%",
                mutation_score * 100.0,
                min * 100.0
            );
        }
    }

    // Format output
    let report = match format {
        OutputFormat::Json => {
            serde_json::json!({
                "mutation_score": mutation_score,
                "total_mutants": score.total,
                "killed": score.killed,
                "survived": score.survived,
                "compile_errors": score.compile_errors,
                "timeouts": score.timeouts,
                "equivalent": score.equivalent,
                "operators": operators.unwrap_or_else(|| vec!["AOR".to_string(), "ROR".to_string(), "COR".to_string(), "UOR".to_string()]),
                "results": results.iter().take(20).map(|r| {
                    serde_json::json!({
                        "id": r.mutant.id,
                        "operator": format!("{:?}", r.mutant.operator),
                        "line": r.mutant.location.line,
                        "column": r.mutant.location.column,
                        "status": format!("{:?}", r.status),
                        "test_failures": r.test_failures,
                        "execution_time_ms": r.execution_time_ms,
                    })
                }).collect::<Vec<_>>()
            })
        },
        _ => {
            serde_json::json!({
                "summary": format!(
                    "Mutation Score: {:.2}% ({}/{} mutants killed)",
                    mutation_score * 100.0,
                    score.killed,
                    score.total
                ),
                "breakdown": format!(
                    "Killed: {}, Survived: {}, Compile Errors: {}, Timeouts: {}, Equivalent: {}",
                    score.killed, score.survived, score.compile_errors, score.timeouts, score.equivalent
                )
            })
        }
    };

    // Output results
    if let Some(output_path) = output {
        let output_str = serde_json::to_string_pretty(&report)?;
        tokio::fs::write(&output_path, output_str).await?;
        println!("\n📄 Report written to: {}", output_path.display());
    } else {
        println!("\n📊 Results:");
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    println!("\n✅ Mutation testing complete!");
    println!("   Mutation score: {:.2}%", mutation_score * 100.0);
    println!("   {} mutants killed, {} survived", score.killed, score.survived);

    if score.compile_errors > 0 {
        println!("   ⚠️  {} mutants caused compilation errors", score.compile_errors);
    }
    if score.timeouts > 0 {
        println!("   ⏱️  {} mutants timed out", score.timeouts);
    }

    Ok(())
}
