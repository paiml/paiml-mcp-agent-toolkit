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

/// Print the summary statistics block (total, killed, survived, etc.)
fn output_text_summary(score: &MutationScore) {
    println!("Total mutants:  {}", score.total);

    if score.total > 0 {
        let pct = |n: usize| (n as f64 / score.total as f64) * 100.0;

        println!(
            "Killed:         {} ({:.1}%)",
            score.killed,
            pct(score.killed)
        );
        println!(
            "Survived:       {} ({:.1}%)",
            score.survived,
            pct(score.survived)
        );

        if score.compile_errors > 0 {
            println!(
                "Compile errors: {} ({:.1}%)",
                score.compile_errors,
                pct(score.compile_errors)
            );
        }

        if score.timeouts > 0 {
            println!(
                "Timeouts:       {} ({:.1}%)",
                score.timeouts,
                pct(score.timeouts)
            );
        }

        if score.equivalent > 0 {
            println!(
                "Equivalent:     {} ({:.1}%)",
                score.equivalent,
                pct(score.equivalent)
            );
        }
    }

    let score_percent = score.score * 100.0;
    println!("\nMutation Score: {:.1}%\n", score_percent);
}

/// Print survived mutants with code snippets
fn output_survived_mutants(results: &[&MutationResult]) {
    if results.is_empty() {
        return;
    }
    println!("Survived Mutants (needs test coverage):\n");
    for (i, result) in results.iter().enumerate() {
        println!(
            "{}. {}:{}:{}",
            i + 1,
            result.mutant.original_file.display(),
            result.mutant.location.line,
            result.mutant.location.column
        );
        println!("   Operator: {:?}", result.mutant.operator);

        if let Ok(snippet) =
            extract_code_snippet(&result.mutant.original_file, &result.mutant.location)
        {
            println!("   Code: {}", snippet);
        }

        println!(
            "   Time: {:.2}s\n",
            result.execution_time_ms as f64 / 1000.0
        );
    }
}

/// Print a list of mutant results under a titled section
fn output_mutant_section(title: &str, results: &[&MutationResult]) {
    if results.is_empty() {
        return;
    }
    println!("{}:\n", title);
    for (i, result) in results.iter().enumerate() {
        println!(
            "{}. {}:{}:{}",
            i + 1,
            result.mutant.original_file.display(),
            result.mutant.location.line,
            result.mutant.location.column
        );
        println!("   Operator: {:?}\n", result.mutant.operator);
    }
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
        println!("\nMutation Testing Failures\n");
    } else {
        println!("\nMutation Testing Results\n");
    }

    // Summary statistics
    if !failures_only {
        output_text_summary(score);
    }

    // Sprint 62: Show failures with code snippets
    let survived: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::Survived)
        .map(|r| *r)
        .collect();
    output_survived_mutants(&survived);

    // Show compile errors if any
    let compile_errors: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::CompileError)
        .map(|r| *r)
        .collect();
    output_mutant_section("Compile Errors", &compile_errors);

    // Show timeouts if any
    let timeouts: Vec<_> = filtered_results
        .iter()
        .filter(|r| r.status == MutantStatus::Timeout)
        .map(|r| *r)
        .collect();
    output_mutant_section("Timeouts", &timeouts);

    Ok(())
}
