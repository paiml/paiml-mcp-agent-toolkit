// Quality gate orchestration and report formatting.
// Contains: execute_all_gates (runs all configured gates) and format_report (human-readable output).

/// Execute all configured quality gates
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 5
pub fn execute_all_gates(config: &GateConfig, project_dir: &Path) -> Result<QualityReport> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use std::time::Instant;

    let start = Instant::now();
    let mut gates = Vec::new();

    if config.run_clippy {
        eprintln!("Running clippy...");
        gates.push(execute_clippy(config, project_dir)?);
        if let Some(last) = gates.last() {
            eprintln!("  clippy: {:.1}s", last.duration.as_secs_f64());
        }
    }

    if config.run_tests {
        eprintln!("Running tests (--lib)...");
        gates.push(execute_tests(config, project_dir)?);
        if let Some(last) = gates.last() {
            eprintln!("  tests: {:.1}s", last.duration.as_secs_f64());
        }
    }

    if config.check_coverage {
        eprintln!("Running coverage...");
        gates.push(execute_coverage(config, project_dir)?);
        if let Some(last) = gates.last() {
            eprintln!("  coverage: {:.1}s", last.duration.as_secs_f64());
        }
    }

    if config.check_complexity {
        eprintln!("Running complexity check...");
        gates.push(execute_complexity(config, project_dir)?);
    }

    let total_duration = start.elapsed();
    let passed = gates.iter().all(|g| g.passed);

    Ok(QualityReport {
        gates,
        passed,
        total_duration,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Format quality report as human-readable text
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 4
pub fn format_report(report: &QualityReport) -> String {
    let mut output = String::new();

    output.push_str("# Quality Gate Report\n\n");
    output.push_str(&format!("**Timestamp**: {}\n\n", report.timestamp));

    let status = if report.passed {
        "✅ PASS"
    } else {
        "❌ FAIL"
    };
    output.push_str(&format!("**Status**: {}\n\n", status));

    output.push_str("## Gate Results\n\n");
    for gate in &report.gates {
        let icon = if gate.passed { "✓" } else { "✗" };
        output.push_str(&format!(
            "- {} **{}** ({:.2}s)\n",
            icon,
            gate.name,
            gate.duration.as_secs_f64()
        ));
        if !gate.message.is_empty() {
            output.push_str(&format!("  {}\n", gate.message));
        }
    }

    output.push_str(&format!(
        "\n**Total Time**: {:.2}s\n",
        report.total_duration.as_secs_f64()
    ));

    output
}
