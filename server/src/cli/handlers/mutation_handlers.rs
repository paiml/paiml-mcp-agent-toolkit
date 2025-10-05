//! Mutation testing handlers
//!
//! Handles the `pmat analyze mutate` command for mutation testing with ML prediction.

use anyhow::{Result, Context};
use std::path::PathBuf;
use crate::cli::OutputFormat;
use crate::services::mutation::{MutationEngine, MutationConfig, RustAdapter, MutantExecutor, MutationScore};
use std::sync::Arc;

/// Handle mutation testing command
///
/// Refactored to reduce complexity from 25 to <20 by extracting helper functions
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
    print_header(&path, &operators);
    validate_path(&path)?;

    let engine = create_mutation_engine();
    let mutants = generate_mutants(&engine, &path).await?;

    if mutants.is_empty() {
        println!("\n⚠️  No mutants generated - file may be too simple or no applicable operators");
        return Ok(());
    }

    let results = execute_mutants(&path, &mutants, _distributed, _workers).await?;
    let score = MutationScore::from_results(&results);

    validate_score_threshold(&score, min_score)?;

    let report = format_report(&score, &results, &operators, format);
    output_report(&report, output).await?;

    print_summary(&score);

    Ok(())
}

/// Print mutation testing header
fn print_header(path: &PathBuf, operators: &Option<Vec<String>>) {
    println!("🧬 Mutation Testing");
    println!("Path: {}", path.display());

    if let Some(ref ops) = operators {
        println!("Operators: {}", ops.join(", "));
    } else {
        println!("Operators: AOR, ROR, COR, UOR (default)");
    }
}

/// Validate path exists
fn validate_path(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }
    Ok(())
}

/// Create mutation engine with default config
fn create_mutation_engine() -> MutationEngine {
    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    MutationEngine::new(adapter, config)
}

/// Generate mutants from path
async fn generate_mutants(engine: &MutationEngine, path: &PathBuf) -> Result<Vec<crate::services::mutation::Mutant>> {
    println!("\n📝 Generating mutants...");

    let mutants = if path.is_file() {
        engine.generate_mutants_from_file(path)
            .await
            .context("Failed to generate mutants")?
    } else {
        anyhow::bail!("Directory mutation testing not yet implemented. Please provide a file path.");
    };

    println!("✅ Generated {} mutants", mutants.len());
    Ok(mutants)
}

/// Execute mutants with appropriate strategy (parallel or sequential)
async fn execute_mutants(
    path: &PathBuf,
    mutants: &[crate::services::mutation::Mutant],
    distributed: bool,
    workers: usize
) -> Result<Vec<crate::services::mutation::MutationResult>> {
    println!("\n🧪 Running tests on mutants...");

    let work_dir = path.parent()
        .and_then(|p| p.parent())
        .or_else(|| path.parent())
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let executor = MutantExecutor::new(work_dir).with_timeout(600);

    if distributed && workers > 1 {
        executor.execute_mutants_parallel(mutants, workers).await
            .context("Failed to execute mutants in parallel")
    } else {
        executor.execute_mutants(mutants).await
            .context("Failed to execute mutants")
    }
}

/// Validate mutation score meets threshold
fn validate_score_threshold(score: &MutationScore, min_score: Option<f64>) -> Result<()> {
    if let Some(min) = min_score {
        if score.score < min {
            anyhow::bail!(
                "Mutation score {:.2}% is below threshold {:.2}%",
                score.score * 100.0,
                min * 100.0
            );
        }
    }
    Ok(())
}

/// Format mutation report based on output format
fn format_report(
    score: &MutationScore,
    results: &[crate::services::mutation::MutationResult],
    operators: &Option<Vec<String>>,
    format: OutputFormat
) -> serde_json::Value {
    match format {
        OutputFormat::Json => format_json_report(score, results, operators),
        _ => format_summary_report(score),
    }
}

/// Format JSON report with full details
fn format_json_report(
    score: &MutationScore,
    results: &[crate::services::mutation::MutationResult],
    operators: &Option<Vec<String>>
) -> serde_json::Value {
    serde_json::json!({
        "mutation_score": score.score,
        "total_mutants": score.total,
        "killed": score.killed,
        "survived": score.survived,
        "compile_errors": score.compile_errors,
        "timeouts": score.timeouts,
        "equivalent": score.equivalent,
        "operators": operators.clone().unwrap_or_else(||
            vec!["AOR".to_string(), "ROR".to_string(), "COR".to_string(), "UOR".to_string()]
        ),
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
}

/// Format summary report
fn format_summary_report(score: &MutationScore) -> serde_json::Value {
    serde_json::json!({
        "summary": format!(
            "Mutation Score: {:.2}% ({}/{} mutants killed)",
            score.score * 100.0,
            score.killed,
            score.total
        ),
        "breakdown": format!(
            "Killed: {}, Survived: {}, Compile Errors: {}, Timeouts: {}, Equivalent: {}",
            score.killed, score.survived, score.compile_errors, score.timeouts, score.equivalent
        )
    })
}

/// Output report to file or console
async fn output_report(report: &serde_json::Value, output: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        let output_str = serde_json::to_string_pretty(report)?;
        tokio::fs::write(&output_path, output_str).await?;
        println!("\n📄 Report written to: {}", output_path.display());
    } else {
        println!("\n📊 Results:");
        println!("{}", serde_json::to_string_pretty(report)?);
    }
    Ok(())
}

/// Print summary statistics
fn print_summary(score: &MutationScore) {
    println!("\n✅ Mutation testing complete!");
    println!("   Mutation score: {:.2}%", score.score * 100.0);
    println!("   {} mutants killed, {} survived", score.killed, score.survived);

    if score.compile_errors > 0 {
        println!("   ⚠️  {} mutants caused compilation errors", score.compile_errors);
    }
    if score.timeouts > 0 {
        println!("   ⏱️  {} mutants timed out", score.timeouts);
    }
}
