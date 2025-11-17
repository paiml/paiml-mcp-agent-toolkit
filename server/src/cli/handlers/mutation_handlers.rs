//! Mutation testing handlers
//!
//! Handles the `pmat analyze mutate` command for mutation testing with ML prediction.

use crate::cli::OutputFormat;
use crate::services::mutation::{
    MutantExecutor, MutationConfig, MutationEngine, MutationScore, RustAdapter,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration for mutation testing
#[derive(Debug, Clone)]
pub struct MutationTestConfig {
    pub operators: Option<Vec<String>>,
    pub ml_predict: bool,
    pub distributed: bool,
    pub workers: usize,
    pub progress: bool,
    pub min_score: Option<f64>,
    pub ci_learning: bool,
    pub ci_provider: Option<String>,
    pub auto_train_threshold: usize,
}

impl MutationTestConfig {
    /// Create config from individual parameters
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operators: Option<Vec<String>>,
        ml_predict: bool,
        distributed: bool,
        workers: usize,
        progress: bool,
        min_score: Option<f64>,
        ci_learning: bool,
        ci_provider: Option<String>,
        auto_train_threshold: usize,
    ) -> Self {
        Self {
            operators,
            ml_predict,
            distributed,
            workers,
            progress,
            min_score,
            ci_learning,
            ci_provider,
            auto_train_threshold,
        }
    }
}

/// Handle mutation testing command
///
/// Performs mutation testing on Rust source files to measure test suite quality.
/// Generates mutants (small code changes), runs tests, and reports the mutation score.
///
/// # Arguments
///
/// * `path` - Path to Rust source file or directory to mutate
/// * `config` - Mutation testing configuration (operators, workers, etc.)
/// * `format` - Output format (Text, Json, Markdown, or Yaml)
/// * `output` - Optional file path to write results to (stdout if None)
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::handlers::mutation_handlers::{handle_mutate, MutationTestConfig};
/// use pmat::cli::OutputFormat;
/// use std::path::PathBuf;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Basic mutation testing with default settings
/// let config = MutationTestConfig::new(
///     None,    // operators (use all)
///     false,   // ml_predict
///     false,   // distributed
///     4,       // workers
///     true,    // progress
///     None,    // min_score
///     false,   // ci_learning
///     None,    // ci_provider
///     100,     // auto_train_threshold
/// );
/// handle_mutate(
///     PathBuf::from("src/lib.rs"),
///     config,
///     OutputFormat::Text,
///     None,
/// ).await?;
///
/// // Mutation testing with minimum score threshold
/// let config = MutationTestConfig::new(
///     Some(vec!["ArithOp".to_string(), "BoolLit".to_string()]),
///     true,    // ml_predict
///     true,    // distributed
///     8,       // workers
///     true,    // progress
///     Some(0.8),  // min_score (80%)
///     false,   // ci_learning
///     None,    // ci_provider
///     100,     // auto_train_threshold
/// );
/// handle_mutate(
///     PathBuf::from("src/main.rs"),
///     config,
///     OutputFormat::Json,
///     Some(PathBuf::from("mutation-results.json")),
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// Returns `Ok(())` if mutation testing completes successfully, or an error if:
/// - Path doesn't exist or isn't accessible
/// - No mutants can be generated
/// - Mutation score is below `min_score` threshold (if specified)
/// - Tests fail to execute
pub async fn handle_mutate(
    path: PathBuf,
    config: MutationTestConfig,
    format: OutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    print_header(&path, &config.operators);
    validate_path(&path)?;

    let engine = create_mutation_engine();
    let mutants = generate_mutants(&engine, &path).await?;

    if mutants.is_empty() {
        println!("\n⚠️  No mutants generated - file may be too simple or no applicable operators");
        return Ok(());
    }

    let results = execute_mutants(&path, &mutants, config.distributed, config.workers).await?;
    let score = MutationScore::from_results(&results);

    validate_score_threshold(&score, config.min_score)?;

    let report = format_report(&score, &results, &config.operators, format);
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
async fn generate_mutants(
    engine: &MutationEngine,
    path: &PathBuf,
) -> Result<Vec<crate::services::mutation::Mutant>> {
    println!("\n📝 Generating mutants...");

    let mutants = if path.is_file() {
        engine
            .generate_mutants_from_file(path)
            .await
            .context("Failed to generate mutants")?
    } else {
        anyhow::bail!(
            "Directory mutation testing not yet implemented. Please provide a file path."
        );
    };

    println!("✅ Generated {} mutants", mutants.len());
    Ok(mutants)
}

/// Execute mutants with appropriate strategy (parallel or sequential)
async fn execute_mutants(
    path: &PathBuf,
    mutants: &[crate::services::mutation::Mutant],
    distributed: bool,
    workers: usize,
) -> Result<Vec<crate::services::mutation::MutationResult>> {
    println!("\n🧪 Running tests on mutants...");

    let work_dir = path
        .parent()
        .and_then(|p| p.parent())
        .or_else(|| path.parent())
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let executor = MutantExecutor::new(work_dir).with_timeout(600);

    if distributed && workers > 1 {
        executor
            .execute_mutants_parallel(mutants, workers)
            .await
            .context("Failed to execute mutants in parallel")
    } else {
        executor
            .execute_mutants(mutants)
            .await
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
    format: OutputFormat,
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
    operators: &Option<Vec<String>>,
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
    println!(
        "   {} mutants killed, {} survived",
        score.killed, score.survived
    );

    if score.compile_errors > 0 {
        println!(
            "   ⚠️  {} mutants caused compilation errors",
            score.compile_errors
        );
    }
    if score.timeouts > 0 {
        println!("   ⏱️  {} mutants timed out", score.timeouts);
    }
}
