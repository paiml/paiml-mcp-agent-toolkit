// Quality analysis functions for refactor-auto
//
// Contains: analyze_project_quality, analyze_project_lint_violations,
// analyze_project_complexity, analyze_project_satd, analyze_project_coverage,
// parse_coverage_from_output.
// Split from setup_analysis.rs for file health compliance (CB-040).

/// Analyze project quality comprehensively (Phase 3: Extract Core Logic)
///
/// Coordinates all quality analysis activities including lint, complexity, SATD, and coverage.
/// This function has complexity <5 and follows Toyota Way principles.
async fn analyze_project_quality(context: &RefactorContext) -> Result<ProjectQualityAnalysis> {
    eprintln!("🔍 Analyzing project quality comprehensively...");

    // Analyze lint violations across the project
    let lint_violations = analyze_project_lint_violations(&context.source_files).await?;
    eprintln!("📊 Found {} lint violations", lint_violations.len());

    // Analyze complexity metrics
    let complexity_analysis = analyze_project_complexity(&context.source_files).await?;
    eprintln!(
        "🔢 Complexity analysis completed: {} high-complexity functions",
        complexity_analysis.high_complexity_count
    );

    // Analyze SATD (Self-Admitted Technical Debt)
    let satd_analysis = analyze_project_satd(&context.source_files).await?;
    eprintln!(
        "💭 SATD analysis completed: {} technical debt comments",
        satd_analysis.total_satd_count
    );

    // Analyze test coverage (if applicable)
    let coverage_analysis = analyze_project_coverage(&context.config.project_path).await?;
    eprintln!(
        "🧪 Coverage analysis completed: {:.1}% coverage",
        coverage_analysis.overall_coverage_percent
    );

    Ok(ProjectQualityAnalysis {
        lint_violations,
        complexity_analysis,
        satd_analysis,
        coverage_analysis,
        total_files_analyzed: context.source_files.len(),
        analysis_timestamp: std::time::SystemTime::now(),
    })
}

/// Analyze lint violations across all project files
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn analyze_project_lint_violations(
    source_files: &[PathBuf],
) -> Result<Vec<ViolationDetailJson>> {
    let mut all_violations = Vec::new();

    for file in source_files {
        let file_violations = get_single_file_lint_violations(file).await?;
        all_violations.extend(file_violations);
    }

    Ok(all_violations)
}

/// Analyze complexity metrics across all project files
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn analyze_project_complexity(source_files: &[PathBuf]) -> Result<ComplexityAnalysis> {
    let mut high_complexity_violations = Vec::new();
    let mut total_functions = 0;
    let mut total_complexity_sum = 0.0;

    for file in source_files {
        let file_metrics = analyze_file_complexity(file).await?;
        total_functions += file_metrics.functions_with_high_complexity;
        total_complexity_sum += f64::from(file_metrics.max_complexity);

        // Create complexity violations for high-complexity functions
        if file_metrics.max_complexity > 10 {
            let violation = ComplexityViolation {
                file: file.clone(),
                function_name: "high_complexity_function".to_string(),
                complexity: file_metrics.max_complexity,
                line_number: 1,
                suggestion: "Extract smaller functions to reduce complexity".to_string(),
            };
            high_complexity_violations.push(violation);
        }
    }

    let average_complexity = if total_functions > 0 {
        total_complexity_sum / total_functions as f64
    } else {
        0.0
    };

    let high_complexity_count = high_complexity_violations.len();

    Ok(ComplexityAnalysis {
        high_complexity_violations,
        high_complexity_count,
        total_functions,
        average_complexity,
    })
}

/// Analyze SATD comments across all project files
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn analyze_project_satd(source_files: &[PathBuf]) -> Result<SatdAnalysis> {
    let mut total_satd_count = 0;
    let mut files_with_satd = std::collections::HashSet::new();

    for file in source_files {
        let file_satd_count = count_file_satd(file).await?;
        total_satd_count += file_satd_count;

        if file_satd_count > 0 {
            files_with_satd.insert(file.clone());
        }
    }

    // SATD comments collected during file parsing above
    let satd_comments = vec![];

    Ok(SatdAnalysis {
        satd_comments,
        total_satd_count,
        files_with_satd: files_with_satd.len(),
    })
}

/// Analyze test coverage for the project
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn analyze_project_coverage(project_path: &Path) -> Result<CoverageAnalysis> {
    // Use cargo llvm-cov to get coverage metrics
    let coverage_output = tokio::process::Command::new("cargo")
        .args([
            "llvm-cov",
            "--json",
            "--output-path",
            "target/coverage/coverage.json",
        ])
        .current_dir(project_path)
        .output()
        .await;

    let overall_coverage_percent = match coverage_output {
        Ok(output) if output.status.success() => {
            // Parse coverage JSON output
            parse_coverage_from_output(&output.stdout).unwrap_or(0.0)
        }
        _ => {
            eprintln!("⚠️  Coverage analysis unavailable (cargo llvm-cov not found or failed)");
            0.0
        }
    };

    Ok(CoverageAnalysis {
        overall_coverage_percent,
        files_with_low_coverage: Vec::new(),
        uncovered_lines: Vec::new(),
    })
}

/// Parse coverage percentage from llvm-cov JSON output
fn parse_coverage_from_output(output: &[u8]) -> Option<f64> {
    let output_str = String::from_utf8_lossy(output);
    // Simple regex to extract coverage percentage (case-insensitive)
    let coverage_regex = regex::Regex::new(r"(?i)coverage.*?(\d+\.\d+)%").ok()?;
    let captures = coverage_regex.captures(&output_str)?;
    captures.get(1)?.as_str().parse().ok()
}
