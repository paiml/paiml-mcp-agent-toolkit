/// Formats and outputs project results
async fn output_project_results(
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_quality_gate_output(results, violations, format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Quality gate report written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Prints the final quality gate status
fn print_quality_gate_final_status(results: &QualityGateResults, violations: &[QualityViolation]) {
    if results.passed {
        eprintln!("\n✅ Quality gate PASSED");
    } else {
        eprintln!("\n⚠️ Quality gate found {} violations", violations.len());
    }
}

/// Handles the exit status based on quality gate results
fn handle_quality_gate_exit_status(fail_on_violation: bool, passed: bool) {
    if fail_on_violation && !passed {
        eprintln!("\n❌ Quality gate FAILED");
        std::process::exit(1);
    }
}

/// Persist all quality gate violations to SQLite for `pmat sql` queryability.
///
/// Opens the existing `.pmat/context.db` and writes all violations to the
/// `quality_violations` table. This makes them queryable via:
///   `pmat sql "SELECT * FROM quality_violations WHERE check_type = 'complexity'"`
fn persist_violations_to_sqlite(
    project_path: &std::path::Path,
    violations: &[QualityViolation],
    quiet: bool,
) {
    let db_path = project_path.join(".pmat").join("context.db");
    if !db_path.exists() {
        if !quiet {
            eprintln!("  ⚠️  No .pmat/context.db found — violations not persisted to SQL");
        }
        return;
    }

    #[allow(clippy::type_complexity)]
    let tuples: Vec<(String, String, String, Option<usize>, String, Option<String>)> = violations
        .iter()
        .map(|v| {
            let details_json = v
                .details
                .as_ref()
                .and_then(|d| serde_json::to_string(d).ok());
            (
                v.check_type.clone(),
                v.severity.clone(),
                v.file.clone(),
                v.line,
                v.message.clone(),
                details_json,
            )
        })
        .collect();

    match crate::services::agent_context::persist_quality_violations(&db_path, &tuples) {
        Ok(()) => {
            if !quiet {
                eprintln!(
                    "  💾 Persisted {} violations to .pmat/context.db",
                    violations.len()
                );
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("  ⚠️  Failed to persist violations to SQL: {e}");
            }
        }
    }

    // Persist entropy violations to specialized table (#231)
    persist_entropy_details_to_sqlite(&db_path, violations, quiet);
}

/// Extract entropy violation details from QualityViolation and persist to `entropy_violations` (#231).
fn persist_entropy_details_to_sqlite(
    db_path: &std::path::Path,
    violations: &[QualityViolation],
    quiet: bool,
) {
    let entropy_tuples: Vec<_> = violations
        .iter()
        .filter(|v| v.check_type == "entropy")
        .filter_map(entropy_violation_to_tuple)
        .collect();

    if entropy_tuples.is_empty() {
        return;
    }

    if let Err(e) =
        crate::services::agent_context::persist_entropy_violations(db_path, &entropy_tuples)
    {
        if !quiet {
            eprintln!("  ⚠️  Failed to persist entropy violations: {e}");
        }
    }
}

/// Convert a single entropy QualityViolation into an entropy_violations tuple (#231).
#[allow(clippy::type_complexity)]
fn entropy_violation_to_tuple(
    v: &QualityViolation,
) -> Option<(String, String, String, usize, f64, usize, String, Option<String>)> {
    let details = v.details.as_ref()?;
    let (pattern_type, repetitions, variation_score) = parse_entropy_score_factors(&details.score_factors);
    let loc_reduction = parse_loc_reduction(&v.message);
    let pattern_hash = format!("{pattern_type}:{}", v.file);
    Some((
        v.file.clone(),
        pattern_type,
        pattern_hash,
        repetitions,
        variation_score,
        loc_reduction,
        v.severity.clone(),
        details.example_code.clone(),
    ))
}

/// Parse score_factors strings to extract pattern_type, repetitions, variation_score.
fn parse_entropy_score_factors(factors: &[String]) -> (String, usize, f64) {
    let mut pattern_type = String::new();
    let mut repetitions: usize = 0;
    let mut variation_score: f64 = 0.0;
    for factor in factors {
        if let Some(pt) = factor.strip_prefix("pattern_type: ") {
            pattern_type = pt.to_string();
        } else if let Some(r) = factor.strip_prefix("repetitions: ") {
            repetitions = r.parse().unwrap_or(0);
        } else if let Some(vs) = factor.strip_prefix("variation_score: ") {
            variation_score = vs.parse().unwrap_or(0.0);
        }
    }
    (pattern_type, repetitions, variation_score)
}

/// Parse "saves N lines" from a violation message to extract LOC reduction.
fn parse_loc_reduction(message: &str) -> usize {
    message
        .find("saves ")
        .and_then(|i| {
            let rest = &message[i + 6..];
            rest.split_whitespace().next()?.parse::<usize>().ok()
        })
        .unwrap_or(0)
}

/// Run lightweight provability analysis and persist per-function scores (#231).
///
/// Must be called from async context. Runs the analyzer on sampled functions
/// and writes results to `provability_scores` table.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn persist_provability_to_sqlite(
    project_path: &std::path::Path,
    quiet: bool,
) {
    let db_path = project_path.join(".pmat").join("context.db");
    if !db_path.exists() {
        return;
    }

    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    let analyzer = LightweightProvabilityAnalyzer::new();
    let sample_functions = collect_project_functions(project_path, 50);

    if sample_functions.is_empty() {
        return;
    }

    let summaries = analyzer.analyze_incrementally(&sample_functions).await;
    let scores: Vec<(String, String, f64, usize)> = sample_functions
        .iter()
        .zip(summaries.iter())
        .map(|(f, s)| {
            (
                f.file_path.clone(),
                f.function_name.clone(),
                s.provability_score,
                s.verified_properties.len(),
            )
        })
        .collect();

    if let Err(e) =
        crate::services::agent_context::persist_provability_scores(&db_path, &scores)
    {
        if !quiet {
            eprintln!("  ⚠️  Failed to persist provability scores: {e}");
        }
    }
}
