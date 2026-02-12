//! Test discovery and fixing handlers for GH-98
//!
//! Systematic test fixing agent with 5-phase automation:
//! 1. Discovery: Run tests, capture ALL failures
//! 2. Categorization: Group by root cause
//! 3. Bulk Marking: Add #[ignore] with reasons
//! 4. Verification: Ensure all tests pass
//! 5. Tracking: Create GitHub issues

use crate::cli::commands::TestDiscoveryCommands;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Test failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    /// Full test name (module::function)
    pub name: String,
    /// File path where test is defined
    pub file: PathBuf,
    /// Line number in file
    pub line: Option<u32>,
    /// Failure reason/message
    pub reason: String,
    /// Failure category
    pub category: FailureCategory,
    /// Test duration (if available)
    pub duration_ms: Option<u64>,
}

/// Failure categories for triage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureCategory {
    /// Test timed out
    Timeout,
    /// Compilation error
    CompileError,
    /// Runtime panic/error
    RuntimeError,
    /// Assertion failure
    AssertionFailure,
    /// Unknown/uncategorized
    Unknown,
}

/// Test discovery report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// Total tests discovered
    pub total_tests: usize,
    /// Number of failures
    pub failures: usize,
    /// List of all failures
    pub test_failures: Vec<TestFailure>,
    /// Discovery timestamp
    pub timestamp: String,
    /// Command used
    pub command: String,
}

/// Handle test-discovery command
pub async fn handle_test_discovery_command(command: TestDiscoveryCommands) -> Result<()> {
    match command {
        TestDiscoveryCommands::Run {
            path,
            output,
            use_nextest,
            timeout,
        } => handle_discovery_run(&path, &output, use_nextest, timeout).await,

        TestDiscoveryCommands::Categorize { input, output } => {
            handle_categorization(&input, &output).await
        }

        TestDiscoveryCommands::Mark { input, apply } => handle_mark(&input, apply).await,

        TestDiscoveryCommands::Verify { path } => handle_verify(&path).await,

        TestDiscoveryCommands::CreateTickets {
            input,
            create,
            output,
            repo,
            labels,
        } => {
            handle_create_tickets(&input, create, output.as_deref(), repo.as_deref(), labels).await
        }

        TestDiscoveryCommands::ResolvePaths {
            input,
            output,
            path,
        } => handle_resolve_paths(&input, &output, &path).await,
    }
}

/// Phase 1: Discovery - Run tests and capture ALL failures
async fn handle_discovery_run(
    project_path: &Path,
    output_path: &Path,
    use_nextest: bool,
    timeout: u64,
) -> Result<()> {
    println!("🔍 Discovering test failures in {}", project_path.display());
    println!(
        "   Using: {}",
        if use_nextest {
            "cargo nextest"
        } else {
            "cargo test"
        }
    );
    println!("   Timeout: {}s", timeout);
    println!();

    // Build the command
    let mut cmd = if use_nextest {
        let mut c = Command::new("cargo");
        c.arg("nextest")
            .arg("run")
            .arg("--workspace")
            .arg("--no-fail-fast")
            .arg("--message-format")
            .arg("json")
            .current_dir(project_path);
        c
    } else {
        let mut c = Command::new("cargo");
        c.arg("test")
            .arg("--workspace")
            .arg("--no-fail-fast")
            .arg("--")
            .arg("--format")
            .arg("json")
            .current_dir(project_path);
        c
    };

    // Run the command and capture output
    println!("📊 Running tests (this may take a while)...");
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run test command")?;

    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("\n📈 Parsing test results...");
    let failures = parse_test_output(&stdout, &stderr)?;

    // Create discovery report
    let report = DiscoveryReport {
        total_tests: count_total_tests(&stdout)?,
        failures: failures.len(),
        test_failures: failures.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        command: format!("{:?}", cmd),
    };

    // Write to output file
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output_path, json)?;

    // Print summary
    println!("\n✅ Discovery complete:");
    println!("   Total tests: {}", report.total_tests);
    println!("   Failures: {}", report.failures);
    println!("   Output: {}", output_path.display());
    println!();

    // Print categorized summary
    print_category_summary(&failures);

    Ok(())
}

/// Parse test output to extract failures
fn parse_test_output(stdout: &str, _stderr: &str) -> Result<Vec<TestFailure>> {
    let mut failures = Vec::new();

    // Parse JSON lines from nextest/cargo test
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            // Check if this is a test failure event
            if json.get("type").and_then(|t| t.as_str()) == Some("test")
                && json.get("event").and_then(|e| e.as_str()) == Some("failed")
            {
                let name = json
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let reason = json
                    .get("stdout")
                    .and_then(|s| s.as_str())
                    .or_else(|| json.get("message").and_then(|m| m.as_str()))
                    .unwrap_or("Unknown failure")
                    .to_string();

                let category = categorize_failure(&reason);

                failures.push(TestFailure {
                    name,
                    file: PathBuf::from("unknown"), // Will be resolved later
                    line: None,
                    reason,
                    category,
                    duration_ms: json.get("exec_time").and_then(|d| d.as_u64()),
                });
            }
        }
    }

    Ok(failures)
}

/// Categorize failure by examining the error message
fn categorize_failure(reason: &str) -> FailureCategory {
    if reason.contains("timed out") || reason.contains("Timeout") {
        FailureCategory::Timeout
    } else if reason.contains("failed to compile") || reason.contains("unresolved import") {
        FailureCategory::CompileError
    } else if reason.contains("panicked at") || reason.contains("thread panicked") {
        FailureCategory::RuntimeError
    } else if reason.contains("assert") || reason.contains("expected") {
        FailureCategory::AssertionFailure
    } else {
        FailureCategory::Unknown
    }
}

/// Count total tests from output
fn count_total_tests(_stdout: &str) -> Result<usize> {
    // Simple implementation - count test events
    // TODO: Improve this to get accurate count
    Ok(0)
}

/// Print categorized summary
fn print_category_summary(failures: &[TestFailure]) {
    use std::collections::HashMap;

    let mut by_category: HashMap<String, usize> = HashMap::new();
    for failure in failures {
        let cat = format!("{:?}", failure.category);
        *by_category.entry(cat).or_insert(0) += 1;
    }

    println!("📊 Failures by category:");
    for (category, count) in by_category {
        println!("   {}: {}", category, count);
    }
}

/// Categorized failure group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureGroup {
    /// Root cause description
    pub root_cause: String,
    /// Suggested ignore reason
    pub ignore_reason: String,
    /// Priority: 1 (fix now) to 5 (ignore indefinitely)
    pub priority: u8,
    /// Tests in this category
    pub tests: Vec<TestFailure>,
}

/// Categorization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationReport {
    /// Total failures
    pub total_failures: usize,
    /// Grouped failures
    pub groups: Vec<FailureGroup>,
    /// Timestamp
    pub timestamp: String,
}

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

/// Test file edit for marking
#[derive(Debug, Clone)]
struct TestEdit {
    /// File path
    file: PathBuf,
    /// Test function name
    test_name: String,
    /// Line number (if known)
    #[allow(dead_code)]
    line: Option<u32>,
    /// Ignore reason
    reason: String,
}

/// Phase 3: Mark - Add #[ignore] attributes
async fn handle_mark(input: &Path, apply: bool) -> Result<()> {
    println!("🏷️  Marking tests as #[ignore]");
    if !apply {
        println!("   (DRY RUN - use --apply to make changes)");
    }
    println!();

    // Read categorization report
    let content = std::fs::read_to_string(input).context("Failed to read categorization report")?;
    let report: CategorizationReport =
        serde_json::from_str(&content).context("Failed to parse categorization report")?;

    // Collect all edits needed
    let mut edits: Vec<TestEdit> = Vec::new();
    for group in &report.groups {
        for test in &group.tests {
            edits.push(TestEdit {
                file: test.file.clone(),
                test_name: test.name.clone(),
                line: test.line,
                reason: group.ignore_reason.clone(),
            });
        }
    }

    println!("   Found {} tests to mark across files", edits.len());

    // Group edits by file
    let mut by_file: std::collections::HashMap<PathBuf, Vec<TestEdit>> =
        std::collections::HashMap::new();
    for edit in edits {
        by_file.entry(edit.file.clone()).or_default().push(edit);
    }

    println!("   Files to modify: {}", by_file.len());
    println!();

    let mut modified_count = 0;
    let mut error_count = 0;

    for (file, file_edits) in &by_file {
        if !file.exists() || file.to_string_lossy() == "unknown" {
            // Try to resolve file from test name
            println!("   ⚠️  Skipping {} (file not found)", file.display());
            continue;
        }

        match mark_tests_in_file(file, file_edits, apply) {
            Ok(count) => {
                modified_count += count;
                if apply {
                    println!("   ✅ Modified {} tests in {}", count, file.display());
                } else {
                    println!("   📝 Would modify {} tests in {}", count, file.display());
                }
            }
            Err(e) => {
                error_count += 1;
                println!("   ❌ Error in {}: {}", file.display(), e);
            }
        }
    }

    println!();
    if apply {
        println!(
            "✅ Marking complete: {} tests modified, {} errors",
            modified_count, error_count
        );
    } else {
        println!(
            "✅ Dry run complete: {} tests would be modified, {} errors",
            modified_count, error_count
        );
        println!("   Run with --apply to make changes");
    }

    Ok(())
}

/// Mark tests in a single file
fn mark_tests_in_file(file: &Path, edits: &[TestEdit], apply: bool) -> Result<usize> {
    let content = std::fs::read_to_string(file)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut modified = 0;

    for edit in edits {
        // Find the test function
        let test_fn_name = edit.test_name.split("::").last().unwrap_or(&edit.test_name);

        for (i, line) in lines.iter().enumerate() {
            // Look for fn test_name( or async fn test_name(
            let pattern = format!("fn {}(", test_fn_name);
            if line.contains(&pattern) {
                // Check if previous line already has #[ignore]
                if i > 0 && lines[i - 1].contains("#[ignore") {
                    continue; // Already marked
                }

                // Check if previous line has #[test]
                let insert_at = if i > 0 && lines[i - 1].contains("#[test") {
                    i // Insert between #[test] and fn
                } else {
                    i // Insert before fn
                };

                // Calculate indentation
                let indent = line.len() - line.trim_start().len();
                let indent_str = " ".repeat(indent);

                // Create ignore attribute
                let ignore_attr = format!("{}#[ignore = \"{}\"]", indent_str, edit.reason);

                if apply {
                    lines.insert(insert_at, ignore_attr);
                }
                modified += 1;
                break;
            }
        }
    }

    if apply && modified > 0 {
        std::fs::write(file, lines.join("\n"))?;
    }

    Ok(modified)
}

/// Phase 4: Verify - Ensure all tests pass
async fn handle_verify(path: &Path) -> Result<()> {
    println!("✅ Verifying tests pass in {}", path.display());
    println!();

    // Run cargo test
    let mut cmd = Command::new("cargo");
    cmd.arg("test")
        .arg("--workspace")
        .arg("--")
        .arg("--include-ignored")
        .current_dir(path);

    println!("📊 Running: cargo test --workspace -- --include-ignored");
    println!("   (This includes ignored tests to verify they're marked correctly)");
    println!();

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run cargo test")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse results
    let (passed, failed, ignored) = parse_test_summary(&stdout, &stderr);

    println!("📈 Test Results:");
    println!("   ✅ Passed: {}", passed);
    println!("   ❌ Failed: {}", failed);
    println!("   ⏭️  Ignored: {}", ignored);
    println!();

    if failed > 0 {
        println!("⚠️  {} tests still failing!", failed);
        println!("   Run 'pmat test-discovery run' to discover remaining failures");
        anyhow::bail!("{} tests still failing", failed);
    } else {
        println!("✅ All tests passing or properly ignored!");
    }

    Ok(())
}

/// GitHub issue template for test failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIssueTemplate {
    /// Issue title
    pub title: String,
    /// Issue body (markdown)
    pub body: String,
    /// Labels to apply
    pub labels: Vec<String>,
    /// Root cause category
    pub category: String,
    /// Number of tests in this issue
    pub test_count: usize,
}

/// Tickets report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketsReport {
    /// Generated tickets
    pub tickets: Vec<TestIssueTemplate>,
    /// Total tests covered
    pub total_tests: usize,
    /// Timestamp
    pub timestamp: String,
}

/// Generate issue templates from categorization
pub fn generate_issue_templates(
    report: &CategorizationReport,
    labels: Option<Vec<String>>,
) -> Vec<TestIssueTemplate> {
    let default_labels = vec!["test-quality".to_string(), "automated".to_string()];
    let extra_labels = labels.unwrap_or_default();

    report
        .groups
        .iter()
        .map(|group| {
            let test_list = group
                .tests
                .iter()
                .take(20) // Limit to avoid huge issues
                .map(|t| format!("- `{}`", t.name))
                .collect::<Vec<_>>()
                .join("\n");

            let more_tests = if group.tests.len() > 20 {
                format!("\n... and {} more tests", group.tests.len() - 20)
            } else {
                String::new()
            };

            let body = format!(
                r#"## Root Cause
{}

## Affected Tests ({} total)
{}{}

## Suggested Fix
{}

## Priority
{} (1=fix now, 5=low priority)

---
*Generated by `pmat test-discovery create-tickets`*
"#,
                group.root_cause,
                group.tests.len(),
                test_list,
                more_tests,
                group.ignore_reason,
                group.priority
            );

            let mut all_labels = default_labels.clone();
            all_labels.extend(extra_labels.clone());
            all_labels.push(format!("priority-{}", group.priority));

            TestIssueTemplate {
                title: format!("[Test Failures] {}", group.root_cause),
                body,
                labels: all_labels,
                category: group.root_cause.clone(),
                test_count: group.tests.len(),
            }
        })
        .collect()
}

/// Phase 5: Create tickets - Generate GitHub issues from categories
async fn handle_create_tickets(
    input: &Path,
    create: bool,
    output: Option<&Path>,
    repo: Option<&str>,
    labels: Option<Vec<String>>,
) -> Result<()> {
    println!("🎫 Creating tickets from categorized failures");
    if !create {
        println!("   (DRY RUN - use --create to create GitHub issues)");
    }
    println!();

    // Read categorization report
    let content = std::fs::read_to_string(input).context("Failed to read categorization report")?;
    let report: CategorizationReport =
        serde_json::from_str(&content).context("Failed to parse categorization report")?;

    println!("   Found {} failure groups", report.groups.len());

    // Generate issue templates
    let tickets = generate_issue_templates(&report, labels);

    // Create tickets report
    let tickets_report = TicketsReport {
        tickets: tickets.clone(),
        total_tests: report.total_failures,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Write output if requested
    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&tickets_report)?;
        std::fs::write(output_path, json)?;
        println!("   Wrote tickets to: {}", output_path.display());
    }

    // Print summary
    println!("\n📋 Generated {} issues:", tickets.len());
    for ticket in &tickets {
        println!("   📝 {} ({} tests)", ticket.title, ticket.test_count);
    }

    if create {
        if let Some(repo_name) = repo {
            println!("\n🚀 Creating GitHub issues in {}...", repo_name);
            for ticket in &tickets {
                match create_github_issue(repo_name, ticket).await {
                    Ok(url) => println!("   ✅ Created: {}", url),
                    Err(e) => println!("   ❌ Failed: {}", e),
                }
            }
        } else {
            println!("\n⚠️  No --repo specified. Use --repo owner/repo to create issues.");
        }
    } else {
        println!("\n   Run with --create --repo owner/repo to create GitHub issues");
    }

    Ok(())
}

/// Create a single GitHub issue using gh CLI
async fn create_github_issue(repo: &str, ticket: &TestIssueTemplate) -> Result<String> {
    let labels_arg = ticket.labels.join(",");

    let output = Command::new("gh")
        .arg("issue")
        .arg("create")
        .arg("--repo")
        .arg(repo)
        .arg("--title")
        .arg(&ticket.title)
        .arg("--body")
        .arg(&ticket.body)
        .arg("--label")
        .arg(&labels_arg)
        .output()
        .context("Failed to run gh CLI")?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(url)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue create failed: {}", stderr)
    }
}

/// Phase 6: Resolve paths - Find test file paths from test names
async fn handle_resolve_paths(input: &Path, output: &Path, project_path: &Path) -> Result<()> {
    println!("🔍 Resolving test file paths");
    println!("   Project: {}", project_path.display());
    println!();

    // Read discovery report
    let content = std::fs::read_to_string(input).context("Failed to read discovery report")?;
    let mut report: DiscoveryReport =
        serde_json::from_str(&content).context("Failed to parse discovery report")?;

    println!(
        "   Found {} failures to resolve",
        report.test_failures.len()
    );

    // Build test name -> file mapping
    let test_file_map = build_test_file_map(project_path)?;
    println!("   Found {} test files", test_file_map.len());

    // Resolve paths
    let mut resolved = 0;
    for failure in &mut report.test_failures {
        if let Some(path) = resolve_test_path(&failure.name, &test_file_map) {
            failure.file = path;
            resolved += 1;
        }
    }

    println!(
        "   Resolved {} of {} paths",
        resolved,
        report.test_failures.len()
    );

    // Write output
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output, json)?;

    println!("\n✅ Output written to: {}", output.display());

    Ok(())
}

/// Build mapping of test function names to file paths
fn build_test_file_map(project_path: &Path) -> Result<std::collections::HashMap<String, PathBuf>> {
    use std::collections::HashMap;

    let mut map = HashMap::new();

    // Find all Rust test files
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();

        // Read file and find test functions
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                // Look for #[test] followed by fn name
                if line.trim().starts_with("fn ") && line.contains("(") {
                    // Extract function name
                    let trimmed = line.trim().strip_prefix("fn ").unwrap_or(line);
                    if let Some(name_end) = trimmed.find('(') {
                        let fn_name = trimmed.get(..name_end).unwrap_or_default();
                        map.insert(fn_name.to_string(), path.to_path_buf());
                    }
                }
            }
        }
    }

    Ok(map)
}

/// Resolve a test name to its file path
fn resolve_test_path(
    test_name: &str,
    map: &std::collections::HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    // Test name format: module::submodule::test_function
    // Try full name first, then just the function name
    if let Some(path) = map.get(test_name) {
        return Some(path.clone());
    }

    // Try just the function name (last part after ::)
    if let Some(fn_name) = test_name.split("::").last() {
        if let Some(path) = map.get(fn_name) {
            return Some(path.clone());
        }
    }

    None
}

/// Parse test summary from output
fn parse_test_summary(stdout: &str, stderr: &str) -> (usize, usize, usize) {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for "X passed; Y failed; Z ignored" pattern
    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;

    for line in combined.lines() {
        if line.contains("passed") && line.contains("filtered") {
            // Parse: "test result: ok. X passed; Y failed; Z ignored; W filtered out"
            if let Some(p) = extract_number(line, "passed") {
                passed = p;
            }
            if let Some(f) = extract_number(line, "failed") {
                failed = f;
            }
            if let Some(i) = extract_number(line, "ignored") {
                ignored = i;
            }
        }
    }

    (passed, failed, ignored)
}

/// Extract number before keyword
fn extract_number(line: &str, keyword: &str) -> Option<usize> {
    if let Some(pos) = line.find(keyword) {
        // Look backwards for the number
        let before = line.get(..pos).unwrap_or_default();
        let parts: Vec<&str> = before.split_whitespace().collect();
        if let Some(num_str) = parts.last() {
            return num_str.parse().ok();
        }
    }
    None
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_failure_timeout() {
        let category = categorize_failure("test timed out after 60 seconds");
        assert_eq!(category, FailureCategory::Timeout);

        let category = categorize_failure("Timeout waiting for response");
        assert_eq!(category, FailureCategory::Timeout);
    }

    #[test]
    fn test_categorize_failure_compile_error() {
        let category = categorize_failure("failed to compile: unresolved import `foo`");
        assert_eq!(category, FailureCategory::CompileError);

        let category = categorize_failure("error: unresolved import `bar::baz`");
        assert_eq!(category, FailureCategory::CompileError);
    }

    #[test]
    fn test_categorize_failure_runtime_error() {
        let category = categorize_failure("thread 'main' panicked at 'oops'");
        assert_eq!(category, FailureCategory::RuntimeError);

        let category = categorize_failure("thread panicked while executing test");
        assert_eq!(category, FailureCategory::RuntimeError);
    }

    #[test]
    fn test_categorize_failure_assertion() {
        let category = categorize_failure("assertion failed: expected 1, got 2");
        assert_eq!(category, FailureCategory::AssertionFailure);

        let category = categorize_failure("expected value to be true");
        assert_eq!(category, FailureCategory::AssertionFailure);
    }

    #[test]
    fn test_categorize_failure_unknown() {
        let category = categorize_failure("something weird happened");
        assert_eq!(category, FailureCategory::Unknown);
    }

    #[test]
    fn test_extract_pattern_panic() {
        let pattern = extract_pattern("thread 'test' panicked at 'message here'\nmore stuff");
        assert_eq!(pattern, "panicked at 'message here'");
    }

    #[test]
    fn test_extract_pattern_assertion() {
        let pattern = extract_pattern("assertion failed: x != y");
        assert_eq!(pattern, "assertion failed");
    }

    #[test]
    fn test_extract_pattern_timeout() {
        let pattern = extract_pattern("test timed out after 60s");
        assert_eq!(pattern, "test timeout");
    }

    #[test]
    fn test_extract_pattern_default() {
        let pattern = extract_pattern("some random error message that is quite long");
        assert_eq!(pattern, "some random error message that is quite long");
    }

    #[test]
    fn test_extract_number_passed() {
        let line = "test result: ok. 42 passed; 3 failed; 10 ignored; 5 filtered out";
        assert_eq!(extract_number(line, "passed"), Some(42));
    }

    #[test]
    fn test_extract_number_failed() {
        let line = "test result: ok. 42 passed; 3 failed; 10 ignored; 5 filtered out";
        assert_eq!(extract_number(line, "failed"), Some(3));
    }

    #[test]
    fn test_extract_number_ignored() {
        let line = "test result: ok. 42 passed; 3 failed; 10 ignored; 5 filtered out";
        assert_eq!(extract_number(line, "ignored"), Some(10));
    }

    #[test]
    fn test_extract_number_not_found() {
        let line = "no numbers here";
        assert_eq!(extract_number(line, "passed"), None);
    }

    #[test]
    fn test_parse_test_summary() {
        let stdout = "test result: ok. 100 passed; 5 failed; 20 ignored; 10 filtered out";
        let stderr = "";
        let (passed, failed, ignored) = parse_test_summary(stdout, stderr);
        assert_eq!(passed, 100);
        assert_eq!(failed, 5);
        assert_eq!(ignored, 20);
    }

    #[test]
    fn test_categorize_failures_groups() {
        let failures = vec![
            TestFailure {
                name: "test1".to_string(),
                file: PathBuf::from("test.rs"),
                line: Some(10),
                reason: "test timed out".to_string(),
                category: FailureCategory::Timeout,
                duration_ms: Some(60000),
            },
            TestFailure {
                name: "test2".to_string(),
                file: PathBuf::from("test.rs"),
                line: Some(20),
                reason: "test timed out".to_string(),
                category: FailureCategory::Timeout,
                duration_ms: Some(60000),
            },
            TestFailure {
                name: "test3".to_string(),
                file: PathBuf::from("test.rs"),
                line: Some(30),
                reason: "assertion failed".to_string(),
                category: FailureCategory::AssertionFailure,
                duration_ms: Some(100),
            },
        ];

        let groups = categorize_failures(&failures);

        // Should have 2 groups: timeout and assertion
        assert_eq!(groups.len(), 2);

        // Find timeout group
        let timeout_group = groups.iter().find(|g| g.root_cause.contains("Timeout"));
        assert!(timeout_group.is_some());
        assert_eq!(timeout_group.unwrap().tests.len(), 2);

        // Find assertion group
        let assertion_group = groups.iter().find(|g| g.root_cause.contains("Assertion"));
        assert!(assertion_group.is_some());
        assert_eq!(assertion_group.unwrap().tests.len(), 1);
    }

    #[test]
    fn test_failure_group_priority() {
        let failures = vec![TestFailure {
            name: "test1".to_string(),
            file: PathBuf::from("test.rs"),
            line: Some(10),
            reason: "assertion failed".to_string(),
            category: FailureCategory::AssertionFailure,
            duration_ms: None,
        }];

        let groups = categorize_failures(&failures);
        assert_eq!(groups.len(), 1);
        // Assertion failures should be priority 1 (fix now)
        assert_eq!(groups[0].priority, 1);
    }

    #[test]
    fn test_discovery_report_serialization() {
        let report = DiscoveryReport {
            total_tests: 100,
            failures: 5,
            test_failures: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            command: "cargo test".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let parsed: DiscoveryReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_tests, 100);
        assert_eq!(parsed.failures, 5);
    }

    #[test]
    fn test_categorization_report_serialization() {
        let report = CategorizationReport {
            total_failures: 10,
            groups: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let parsed: CategorizationReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_failures, 10);
    }

    // Phase 5: CreateTickets tests (V2 feature)

    #[test]
    fn test_generate_issue_templates_creates_tickets() {
        let report = CategorizationReport {
            total_failures: 5,
            groups: vec![FailureGroup {
                root_cause: "Timeout: slow tests".to_string(),
                ignore_reason: "GH-98: Slow test".to_string(),
                priority: 3,
                tests: vec![
                    TestFailure {
                        name: "test_slow_1".to_string(),
                        file: PathBuf::from("test.rs"),
                        line: Some(10),
                        reason: "timed out".to_string(),
                        category: FailureCategory::Timeout,
                        duration_ms: Some(60000),
                    },
                    TestFailure {
                        name: "test_slow_2".to_string(),
                        file: PathBuf::from("test.rs"),
                        line: Some(20),
                        reason: "timed out".to_string(),
                        category: FailureCategory::Timeout,
                        duration_ms: Some(60000),
                    },
                ],
            }],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let tickets = generate_issue_templates(&report, None);

        assert_eq!(tickets.len(), 1);
        assert!(tickets[0].title.contains("Timeout"));
        assert_eq!(tickets[0].test_count, 2);
        assert!(tickets[0].body.contains("test_slow_1"));
        assert!(tickets[0].body.contains("test_slow_2"));
    }

    #[test]
    fn test_generate_issue_templates_applies_labels() {
        let report = CategorizationReport {
            total_failures: 1,
            groups: vec![FailureGroup {
                root_cause: "Assertion failure".to_string(),
                ignore_reason: "Fix needed".to_string(),
                priority: 1,
                tests: vec![TestFailure {
                    name: "test_assert".to_string(),
                    file: PathBuf::from("test.rs"),
                    line: None,
                    reason: "assertion failed".to_string(),
                    category: FailureCategory::AssertionFailure,
                    duration_ms: None,
                }],
            }],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let custom_labels = vec!["bug".to_string(), "urgent".to_string()];
        let tickets = generate_issue_templates(&report, Some(custom_labels));

        assert!(tickets[0].labels.contains(&"test-quality".to_string()));
        assert!(tickets[0].labels.contains(&"bug".to_string()));
        assert!(tickets[0].labels.contains(&"urgent".to_string()));
        assert!(tickets[0].labels.contains(&"priority-1".to_string()));
    }

    #[test]
    fn test_generate_issue_templates_limits_test_list() {
        let tests: Vec<TestFailure> = (0..30)
            .map(|i| TestFailure {
                name: format!("test_{}", i),
                file: PathBuf::from("test.rs"),
                line: Some(i as u32),
                reason: "failed".to_string(),
                category: FailureCategory::Unknown,
                duration_ms: None,
            })
            .collect();

        let report = CategorizationReport {
            total_failures: 30,
            groups: vec![FailureGroup {
                root_cause: "Many failures".to_string(),
                ignore_reason: "Investigate".to_string(),
                priority: 4,
                tests,
            }],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let tickets = generate_issue_templates(&report, None);

        // Should limit to 20 tests in body
        assert!(tickets[0].body.contains("test_0"));
        assert!(tickets[0].body.contains("test_19"));
        assert!(!tickets[0].body.contains("test_20")); // This one should be cut off
        assert!(tickets[0].body.contains("... and 10 more tests"));
    }

    #[test]
    fn test_tickets_report_serialization() {
        let report = TicketsReport {
            tickets: vec![TestIssueTemplate {
                title: "Test Issue".to_string(),
                body: "Body".to_string(),
                labels: vec!["bug".to_string()],
                category: "Category".to_string(),
                test_count: 5,
            }],
            total_tests: 5,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let parsed: TicketsReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.tickets.len(), 1);
        assert_eq!(parsed.tickets[0].title, "Test Issue");
        assert_eq!(parsed.total_tests, 5);
    }

    // Path resolution tests

    #[test]
    fn test_resolve_test_path_exact_match() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("test_foo".to_string(), PathBuf::from("src/tests.rs"));

        let result = resolve_test_path("test_foo", &map);
        assert_eq!(result, Some(PathBuf::from("src/tests.rs")));
    }

    #[test]
    fn test_resolve_test_path_module_prefix() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("test_bar".to_string(), PathBuf::from("src/module/tests.rs"));

        // Should resolve module::submodule::test_bar to test_bar
        let result = resolve_test_path("module::submodule::test_bar", &map);
        assert_eq!(result, Some(PathBuf::from("src/module/tests.rs")));
    }

    #[test]
    fn test_resolve_test_path_not_found() {
        use std::collections::HashMap;

        let map: HashMap<String, PathBuf> = HashMap::new();

        let result = resolve_test_path("nonexistent_test", &map);
        assert!(result.is_none());
    }
}
