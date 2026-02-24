/// Phase 2: Categorization - Group failures by root cause
async fn handle_categorization(input: &Path, output: &Path) -> Result<()> {
    println!("📋 Categorizing test failures from {}", input.display());

    // Read discovery report
    let content = std::fs::read_to_string(input).context("Failed to read discovery report")?;
    let report: DiscoveryReport =
        serde_json::from_str(&content).context("Failed to parse discovery report")?;

    println!(
        "   Found {} failures to categorize",
        report.test_failures.len()
    );

    // Group failures by category and pattern
    let groups = categorize_failures(&report.test_failures);

    // Create categorization report
    let cat_report = CategorizationReport {
        total_failures: report.test_failures.len(),
        groups,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Write to output
    let json = serde_json::to_string_pretty(&cat_report)?;
    std::fs::write(output, json)?;

    println!("\n✅ Categorization complete:");
    println!("   Groups: {}", cat_report.groups.len());
    println!("   Output: {}", output.display());
    println!();

    // Print summary
    for group in &cat_report.groups {
        println!(
            "   📁 {} (priority {}): {} tests",
            group.root_cause,
            group.priority,
            group.tests.len()
        );
        println!("      Reason: {}", group.ignore_reason);
    }

    Ok(())
}

/// Categorize failures into groups
fn categorize_failures(failures: &[TestFailure]) -> Vec<FailureGroup> {
    use std::collections::HashMap;

    // Group by (category, pattern)
    let mut groups: HashMap<(FailureCategory, String), Vec<TestFailure>> = HashMap::new();

    for failure in failures {
        let pattern = extract_pattern(&failure.reason);
        let key = (failure.category.clone(), pattern);
        groups.entry(key).or_default().push(failure.clone());
    }

    // Convert to FailureGroup
    groups
        .into_iter()
        .map(|((category, pattern), tests)| {
            let (root_cause, ignore_reason, priority) = match category {
                FailureCategory::Timeout => (
                    format!("Timeout: {}", pattern),
                    "GH-98: Slow test - needs optimization or async fix".to_string(),
                    3,
                ),
                FailureCategory::CompileError => (
                    format!("Compile error: {}", pattern),
                    "GH-98: Compilation issue - needs feature gate or fix".to_string(),
                    2,
                ),
                FailureCategory::RuntimeError => (
                    format!("Runtime error: {}", pattern),
                    "GH-98: Runtime panic - needs investigation".to_string(),
                    2,
                ),
                FailureCategory::AssertionFailure => (
                    format!("Assertion failure: {}", pattern),
                    "GH-98: Test expectation changed - needs update".to_string(),
                    1,
                ),
                FailureCategory::Unknown => (
                    format!("Unknown: {}", pattern),
                    "GH-98: Uncategorized failure - needs triage".to_string(),
                    4,
                ),
            };

            FailureGroup {
                root_cause,
                ignore_reason,
                priority,
                tests,
            }
        })
        .collect()
}

/// Extract pattern from failure reason for grouping
fn extract_pattern(reason: &str) -> String {
    // Extract key pattern from error message
    if reason.contains("panicked at") {
        // Extract panic message
        if let Some(start) = reason.find("panicked at") {
            let rest = reason.get(start..).unwrap_or_default();
            if let Some(end) = rest.find('\n') {
                return rest.get(..end).unwrap_or_default().to_string();
            }
            return rest.chars().take(80).collect();
        }
    }

    if reason.contains("assertion") {
        return "assertion failed".to_string();
    }

    if reason.contains("timed out") {
        return "test timeout".to_string();
    }

    // Default: first 50 chars
    reason.chars().take(50).collect()
}
