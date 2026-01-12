//! Mutation testing handler (Sprint 61)
//!
//! Exposes PMAT's AST-based mutation testing infrastructure via CLI command.

use crate::cli::commands::MutateArgs;
use crate::services::mutation::engine::{MutationConfig, MutationEngine, MutationStrategy};
use crate::services::mutation::types::{MutationResult, MutationScore, SourceLocation};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{Context, Result};
use console::style;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// Handle mutation testing command
pub async fn handle(args: MutateArgs, _server: Arc<StatelessTemplateServer>) -> Result<()> {
    info!("Starting mutation testing on {:?}", args.target);

    // Sprint 70: Route to cargo-mutants backend if requested
    if args.use_cargo_mutants {
        return handle_cargo_mutants_backend(args).await;
    }

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
    let total_mutants = mutants.len();
    eprintln!("Generated {} mutants", total_mutants);

    // 4. Execute mutants with progress indicators
    eprintln!("\nExecuting mutants...");
    let start_time = Instant::now();

    let results = if config.parallel_threads > 1 {
        execute_with_progress(engine, mutants, total_mutants).await?
    } else {
        execute_sequential_with_progress(engine, mutants, total_mutants).await?
    };

    let elapsed = start_time.elapsed();
    eprintln!("\nCompleted in {:.1}s\n", elapsed.as_secs_f64());

    // 5. Calculate score
    let score = MutationScore::from_results(&results);

    // 6. Output
    match args.output_format.as_str() {
        "json" => output_json(&score, &results, args.failures_only)?,
        "markdown" => output_markdown(&score, &results, args.failures_only)?,
        _ => output_text(&score, &results, args.failures_only)?,
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

/// Execute mutants in parallel with progress indicators
async fn execute_with_progress(
    engine: MutationEngine,
    mutants: Vec<crate::services::mutation::types::Mutant>,
    total: usize,
) -> Result<Vec<MutationResult>> {
    use tokio::time::sleep;

    // Start execution in background
    let exec_handle = tokio::spawn(async move { engine.execute_mutants_parallel(mutants).await });

    // Progress reporting loop
    let mut completed = 0;
    while !exec_handle.is_finished() {
        sleep(Duration::from_millis(500)).await;

        // Simple progress indicator (will be enhanced with actual progress tracking)
        completed = (completed + 1) % (total + 1);
        print_progress(completed.min(total), total);
    }

    // Get final results
    let results = exec_handle.await??;
    print_progress(total, total);
    eprintln!(); // New line after progress

    Ok(results)
}

/// Execute mutants sequentially with progress indicators
async fn execute_sequential_with_progress(
    engine: MutationEngine,
    mutants: Vec<crate::services::mutation::types::Mutant>,
    total: usize,
) -> Result<Vec<MutationResult>> {
    let mut results = Vec::new();

    for (i, mutant) in mutants.into_iter().enumerate() {
        print_progress(i, total);

        // Execute single mutant (we need to expose this method)
        // For now, use the batch method with single item
        let single_result = engine.execute_mutants(vec![mutant]).await?;
        results.extend(single_result);
    }

    print_progress(total, total);
    eprintln!(); // New line after progress

    Ok(results)
}

/// Print progress indicator
fn print_progress(completed: usize, total: usize) {
    if total == 0 {
        return;
    }

    let percentage = (completed as f64 / total as f64) * 100.0;
    let bar_width = 40;
    let filled = (bar_width as f64 * completed as f64 / total as f64) as usize;
    let empty = bar_width - filled;

    eprint!(
        "\r[{}{}] {}/{} ({:.1}%)",
        "=".repeat(filled),
        " ".repeat(empty),
        completed,
        total,
        percentage
    );

    use std::io::Write;
    let _ = std::io::stderr().flush();
}

/// Extract code snippet from source file using SourceLocation (Sprint 62)
///
/// Reads the source file and extracts lines from location.line to location.end_line
fn extract_code_snippet(file_path: &Path, location: &SourceLocation) -> Result<String> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path.display()))?;

    let lines: Vec<&str> = content.lines().collect();

    // Line numbers are 1-indexed in SourceLocation
    let start_line = location.line.saturating_sub(1);
    let end_line = location.end_line.min(lines.len());

    if start_line >= lines.len() {
        return Ok(String::from("<code location out of bounds>"));
    }

    let snippet_lines = &lines[start_line..end_line];
    let snippet = snippet_lines.join("\n");

    Ok(snippet.trim().to_string())
}

/// JSON output wrapper for serialization (Sprint 62 - enhanced with code snippets)
#[derive(Serialize)]
struct MutationTestOutput {
    score: MutationScore,
    results: Vec<EnhancedMutationResult>,
}

/// Enhanced mutation result with code snippets (Sprint 62)
#[derive(Serialize)]
struct EnhancedMutationResult {
    #[serde(flatten)]
    result: MutationResult,
    original_code_snippet: Option<String>,
    mutated_code_snippet: Option<String>,
}

fn output_json(
    score: &MutationScore,
    results: &[MutationResult],
    failures_only: bool,
) -> Result<()> {
    use crate::services::mutation::types::MutantStatus;

    // Sprint 62 Day 2: Filter for failures-only mode
    let filtered_results: Vec<&MutationResult> = if failures_only {
        results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect()
    } else {
        results.iter().collect()
    };

    // Enhance results with code snippets
    let enhanced_results: Vec<EnhancedMutationResult> = filtered_results
        .iter()
        .map(|r| {
            let original_snippet =
                extract_code_snippet(&r.mutant.original_file, &r.mutant.location).ok();
            let mutated_snippet = Some(r.mutant.mutated_source.clone());

            EnhancedMutationResult {
                result: (*r).clone(),
                original_code_snippet: original_snippet,
                mutated_code_snippet: mutated_snippet,
            }
        })
        .collect();

    let output = MutationTestOutput {
        score: score.clone(),
        results: enhanced_results,
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);
    Ok(())
}

fn output_markdown(
    score: &MutationScore,
    results: &[MutationResult],
    failures_only: bool,
) -> Result<()> {
    use crate::services::mutation::types::MutantStatus;

    // Sprint 62 Day 2: Filter for failures-only mode
    let filtered_results: Vec<&MutationResult> = if failures_only {
        results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect()
    } else {
        results.iter().collect()
    };

    if failures_only {
        println!("# Mutation Testing Failures\n");
    } else {
        println!("# Mutation Testing Results\n");
    }

    if !failures_only {
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
    }

    // Show survived mutants for test improvements
    let survived: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::Survived)
        .collect();

    if !survived.is_empty() {
        println!("## Survived Mutants (Test Gaps)\n");
        println!("The following mutants survived, indicating potential test coverage gaps:\n");

        for (i, result) in survived.iter().enumerate() {
            println!("### Mutant #{}", i + 1);
            println!(
                "- **Location**: {}:{}:{}",
                result.mutant.original_file.display(),
                result.mutant.location.line,
                result.mutant.location.column
            );
            println!("- **Operator**: {:?}", result.mutant.operator);
            println!("- **Status**: Survived");

            // Sprint 62: Add diff block for code changes
            if let Ok(original) =
                extract_code_snippet(&result.mutant.original_file, &result.mutant.location)
            {
                println!("\n**Code Change:**");
                println!("```diff");
                println!("- {}", original);
                println!(
                    "+ {}",
                    result
                        .mutant
                        .mutated_source
                        .lines()
                        .next()
                        .unwrap_or("<empty>")
                );
                println!("```");
            }

            println!();
        }
    }

    Ok(())
}

fn output_text(
    score: &MutationScore,
    results: &[MutationResult],
    failures_only: bool,
) -> Result<()> {
    use crate::services::mutation::types::MutantStatus;

    // Sprint 62 Day 2: Filter for failures-only mode
    let filtered_results: Vec<_> = if failures_only {
        results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect()
    } else {
        results.iter().collect()
    };

    if failures_only {
        println!("\n{}\n", style("Mutation Testing Failures").bold().red());
    } else {
        println!("\n{}\n", style("Mutation Testing Results").bold());
    }

    // Summary statistics (always show, with color coding)
    if !failures_only {
        println!("Total mutants:  {}", score.total);

        if score.total > 0 {
            println!(
                "{}         {} ({:.1}%)",
                style("Killed:").green(),
                score.killed,
                (score.killed as f64 / score.total as f64) * 100.0
            );
            println!(
                "{}       {} ({:.1}%)",
                style("Survived:").red(),
                score.survived,
                (score.survived as f64 / score.total as f64) * 100.0
            );

            if score.compile_errors > 0 {
                println!(
                    "{} {} ({:.1}%)",
                    style("Compile errors:").yellow(),
                    score.compile_errors,
                    (score.compile_errors as f64 / score.total as f64) * 100.0
                );
            }

            if score.timeouts > 0 {
                println!(
                    "{}       {} ({:.1}%)",
                    style("Timeouts:").yellow(),
                    score.timeouts,
                    (score.timeouts as f64 / score.total as f64) * 100.0
                );
            }

            if score.equivalent > 0 {
                println!(
                    "{}     {} ({:.1}%)",
                    style("Equivalent:").cyan(),
                    score.equivalent,
                    (score.equivalent as f64 / score.total as f64) * 100.0
                );
            }
        }

        // Color-code mutation score
        let score_percent = score.score * 100.0;
        let score_styled = if score_percent >= 80.0 {
            style(format!("{:.1}%", score_percent)).green().bold()
        } else if score_percent >= 60.0 {
            style(format!("{:.1}%", score_percent)).yellow().bold()
        } else {
            style(format!("{:.1}%", score_percent)).red().bold()
        };
        println!("\n{} {}\n", style("Mutation Score:").bold(), score_styled);
    }

    // Sprint 62: Show failures with code snippets
    let survived: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::Survived)
        .collect();

    if !survived.is_empty() {
        println!(
            "{}\n",
            style("Survived Mutants (needs test coverage):")
                .red()
                .bold()
        );
        for (i, result) in survived.iter().enumerate() {
            println!(
                "{}. {}",
                style(format!("{}", i + 1)).red().bold(),
                style(format!(
                    "{}:{}:{}",
                    result.mutant.original_file.display(),
                    result.mutant.location.line,
                    result.mutant.location.column
                ))
                .cyan()
            );
            println!(
                "   {}: {:?}",
                style("Operator").bold(),
                result.mutant.operator
            );

            // Extract and display code snippet
            if let Ok(snippet) =
                extract_code_snippet(&result.mutant.original_file, &result.mutant.location)
            {
                println!("   {}: {}", style("Code").bold(), snippet);
            }

            println!(
                "   {}: {:.2}s\n",
                style("Time").bold(),
                result.execution_time_ms as f64 / 1000.0
            );
        }
    }

    // Show compile errors if any
    let compile_errors: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::CompileError)
        .collect();

    if !compile_errors.is_empty() {
        println!("{}\n", style("Compile Errors:").yellow().bold());
        for (i, result) in compile_errors.iter().enumerate() {
            println!(
                "{}. {}",
                style(format!("{}", i + 1)).yellow().bold(),
                style(format!(
                    "{}:{}:{}",
                    result.mutant.original_file.display(),
                    result.mutant.location.line,
                    result.mutant.location.column
                ))
                .cyan()
            );
            println!(
                "   {}: {:?}\n",
                style("Operator").bold(),
                result.mutant.operator
            );
        }
    }

    // Show timeouts if any
    let timeouts: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::Timeout)
        .collect();

    if !timeouts.is_empty() {
        println!("{}\n", style("Timeouts:").yellow().bold());
        for (i, result) in timeouts.iter().enumerate() {
            println!(
                "{}. {}",
                style(format!("{}", i + 1)).yellow().bold(),
                style(format!(
                    "{}:{}:{}",
                    result.mutant.original_file.display(),
                    result.mutant.location.line,
                    result.mutant.location.column
                ))
                .cyan()
            );
            println!(
                "   {}: {:?}\n",
                style("Operator").bold(),
                result.mutant.operator
            );
        }
    }

    Ok(())
}

// ============================================================================
// Sprint 70: cargo-mutants Backend Handler
// ============================================================================

/// Handle mutation testing via cargo-mutants backend
async fn handle_cargo_mutants_backend(args: MutateArgs) -> Result<()> {
    use crate::cli::handlers::cargo_mutants_backend::{self, CargoMutantsConfig};
    use crate::services::mutation::json_parser::CargoMutantsReport;

    // Build configuration
    let config = CargoMutantsConfig {
        path: args.target.clone(),
        output: args.output.clone(),
        timeout: args.timeout,
        jobs: args.jobs,
        features: args.features,
        all_features: args.all_features,
        no_default_features: args.no_default_features,
        no_shuffle: args.no_shuffle,
    };

    // Execute cargo-mutants (returns path to output directory)
    let output_dir = cargo_mutants_backend::execute(config)?;

    // Parse output directory (reads outcomes.json)
    let report = CargoMutantsReport::from_output_dir(&output_dir)
        .map_err(|e| anyhow::anyhow!("Failed to parse cargo-mutants output: {}", e))?;

    // Display statistics
    cargo_mutants_backend::display_statistics(&report);

    // Check threshold if specified
    if let Some(threshold) = args.threshold {
        let mutation_score = report.mutation_score();
        if mutation_score < threshold {
            anyhow::bail!(
                "Mutation score {:.1}% below threshold {:.1}%",
                mutation_score,
                threshold
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::types::{Mutant, MutantStatus, MutationOperator, SourceLocation};
    use tempfile::TempDir;

    fn create_temp_rust_file() -> (TempDir, std::path::PathBuf) {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.rs");
        std::fs::write(
            &file_path,
            r#"fn main() {
    let x = 5;
    let y = 10;
    println!("{}", x + y);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        )
        .unwrap();
        (temp, file_path)
    }

    fn create_test_mutant(file_path: &Path, line: usize) -> Mutant {
        Mutant {
            id: format!("mutant_{}", line),
            original_file: file_path.to_path_buf(),
            location: SourceLocation {
                line,
                column: 1,
                end_line: line,
                end_column: 10,
            },
            operator: MutationOperator::ArithmeticReplace,
            original_source: "a + b".to_string(),
            mutated_source: "a - b".to_string(),
        }
    }

    fn create_test_mutation_result(file_path: &Path, status: MutantStatus) -> MutationResult {
        MutationResult {
            mutant: create_test_mutant(file_path, 8),
            status,
            execution_time_ms: 100,
            test_output: Some("test output".to_string()),
        }
    }

    // ============================================================================
    // extract_code_snippet tests
    // ============================================================================

    #[test]
    fn test_extract_code_snippet_success() {
        let (temp, file_path) = create_temp_rust_file();

        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 2,
            end_column: 10,
        };

        let snippet = extract_code_snippet(&file_path, &location).unwrap();
        assert!(snippet.contains("fn main()"));
        drop(temp);
    }

    #[test]
    fn test_extract_code_snippet_single_line() {
        let (temp, file_path) = create_temp_rust_file();

        let location = SourceLocation {
            line: 7,
            column: 1,
            end_line: 7,
            end_column: 30,
        };

        let snippet = extract_code_snippet(&file_path, &location).unwrap();
        assert!(!snippet.is_empty());
        drop(temp);
    }

    #[test]
    fn test_extract_code_snippet_out_of_bounds() {
        let (temp, file_path) = create_temp_rust_file();

        let location = SourceLocation {
            line: 1000,
            column: 1,
            end_line: 1001,
            end_column: 10,
        };

        let snippet = extract_code_snippet(&file_path, &location).unwrap();
        assert!(snippet.contains("<code location out of bounds>"));
        drop(temp);
    }

    #[test]
    fn test_extract_code_snippet_file_not_found() {
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 2,
            end_column: 10,
        };

        let result = extract_code_snippet(Path::new("/nonexistent/file.rs"), &location);
        assert!(result.is_err());
    }

    // ============================================================================
    // print_progress tests
    // ============================================================================

    #[test]
    fn test_print_progress_zero_total() {
        // Should not panic with zero total
        print_progress(0, 0);
    }

    #[test]
    fn test_print_progress_partial() {
        print_progress(5, 10);
    }

    #[test]
    fn test_print_progress_complete() {
        print_progress(10, 10);
    }

    // ============================================================================
    // MutationScore tests
    // ============================================================================

    #[test]
    fn test_mutation_score_from_empty_results() {
        let results: Vec<MutationResult> = vec![];
        let score = MutationScore::from_results(&results);
        assert_eq!(score.total, 0);
        assert_eq!(score.killed, 0);
    }

    #[test]
    fn test_mutation_score_from_results_killed() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![create_test_mutation_result(&file_path, MutantStatus::Killed)];

        let score = MutationScore::from_results(&results);
        assert_eq!(score.total, 1);
        assert_eq!(score.killed, 1);
        assert_eq!(score.survived, 0);
        drop(temp);
    }

    #[test]
    fn test_mutation_score_from_results_survived() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![create_test_mutation_result(
            &file_path,
            MutantStatus::Survived,
        )];

        let score = MutationScore::from_results(&results);
        assert_eq!(score.survived, 1);
        drop(temp);
    }

    #[test]
    fn test_mutation_score_from_results_mixed() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::Survived),
            create_test_mutation_result(&file_path, MutantStatus::CompileError),
            create_test_mutation_result(&file_path, MutantStatus::Timeout),
        ];

        let score = MutationScore::from_results(&results);
        assert_eq!(score.total, 4);
        assert_eq!(score.killed, 1);
        assert_eq!(score.survived, 1);
        assert_eq!(score.compile_errors, 1);
        assert_eq!(score.timeouts, 1);
        drop(temp);
    }

    // ============================================================================
    // EnhancedMutationResult tests
    // ============================================================================

    #[test]
    fn test_enhanced_mutation_result_serialization() {
        let (temp, file_path) = create_temp_rust_file();
        let result = create_test_mutation_result(&file_path, MutantStatus::Killed);

        let enhanced = EnhancedMutationResult {
            result,
            original_code_snippet: Some("a + b".to_string()),
            mutated_code_snippet: Some("a - b".to_string()),
        };

        let json = serde_json::to_string(&enhanced).unwrap();
        assert!(json.contains("a + b"));
        assert!(json.contains("a - b"));
        drop(temp);
    }

    // ============================================================================
    // MutationTestOutput tests
    // ============================================================================

    #[test]
    fn test_mutation_test_output_serialization() {
        let score = MutationScore {
            total: 10,
            killed: 8,
            survived: 2,
            compile_errors: 0,
            timeouts: 0,
            equivalent: 0,
            score: 0.8,
        };

        let output = MutationTestOutput {
            score,
            results: vec![],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"total\":10"));
        assert!(json.contains("\"killed\":8"));
        assert!(json.contains("\"score\":0.8"));
    }

    // ============================================================================
    // SourceLocation tests
    // ============================================================================

    #[test]
    fn test_source_location_creation() {
        let location = SourceLocation {
            line: 10,
            column: 5,
            end_line: 12,
            end_column: 20,
        };

        assert_eq!(location.line, 10);
        assert_eq!(location.column, 5);
        assert_eq!(location.end_line, 12);
        assert_eq!(location.end_column, 20);
    }

    // ============================================================================
    // Mutant tests
    // ============================================================================

    #[test]
    fn test_mutant_creation() {
        let (temp, file_path) = create_temp_rust_file();
        let mutant = create_test_mutant(&file_path, 8);

        assert_eq!(mutant.original_source, "a + b");
        assert_eq!(mutant.mutated_source, "a - b");
        assert_eq!(mutant.operator, MutationOperator::ArithmeticReplace);
        drop(temp);
    }

    // ============================================================================
    // MutationResult tests
    // ============================================================================

    #[test]
    fn test_mutation_result_creation() {
        let (temp, file_path) = create_temp_rust_file();
        let result = create_test_mutation_result(&file_path, MutantStatus::Killed);

        assert_eq!(result.status, MutantStatus::Killed);
        assert_eq!(result.execution_time_ms, 100);
        assert!(result.test_output.is_some());
        drop(temp);
    }

    #[test]
    fn test_mutation_result_serialization() {
        let (temp, file_path) = create_temp_rust_file();
        let result = create_test_mutation_result(&file_path, MutantStatus::Survived);

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Survived"));
        assert!(json.contains("execution_time_ms"));
        drop(temp);
    }

    // ============================================================================
    // Output format filtering tests
    // ============================================================================

    #[test]
    fn test_filter_failures_only_keeps_survived() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::Survived),
        ];

        // Simulate failures_only filtering
        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, MutantStatus::Survived);
        drop(temp);
    }

    #[test]
    fn test_filter_failures_only_keeps_compile_error() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::CompileError),
        ];

        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, MutantStatus::CompileError);
        drop(temp);
    }

    #[test]
    fn test_filter_failures_only_keeps_timeout() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::Timeout),
        ];

        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, MutantStatus::Timeout);
        drop(temp);
    }

    // ============================================================================
    // MutationOperator tests
    // ============================================================================

    #[test]
    fn test_mutation_operator_equality() {
        assert_eq!(
            MutationOperator::ArithmeticReplace,
            MutationOperator::ArithmeticReplace
        );
        assert_ne!(
            MutationOperator::ArithmeticReplace,
            MutationOperator::ComparisonReplace
        );
    }

    #[test]
    fn test_mutant_status_equality() {
        assert_eq!(MutantStatus::Killed, MutantStatus::Killed);
        assert_ne!(MutantStatus::Killed, MutantStatus::Survived);
    }
}
