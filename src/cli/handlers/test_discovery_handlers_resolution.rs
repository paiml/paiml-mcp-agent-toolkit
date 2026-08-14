/// Phase 6: Resolve paths - Find test file paths from test names
async fn handle_resolve_paths(input: &Path, output: &Path, project_path: &Path) -> Result<()> {
    crate::status_println!("🔍 Resolving test file paths");
    crate::status_println!("   Project: {}", project_path.display());
    crate::status_println!();

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

    crate::status_println!("\n✅ Output written to: {}", output.display());

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
