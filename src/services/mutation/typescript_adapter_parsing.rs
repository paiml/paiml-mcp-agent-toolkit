/// Find package.json by traversing up from source file
pub fn find_package_json_root(start: &Path) -> Option<&Path> {
    debug_assert!(start.exists(), "start must exist: {}", start.display());
    let mut current = start;

    loop {
        if current.join("package.json").exists() {
            return Some(current);
        }

        current = current.parent()?;
    }
}

/// Parse test failures from npm test or jest output
pub fn parse_test_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        if line.contains('✕') || line.contains("FAIL") {
            if let Some(test_name) = extract_test_name(line) {
                failures.push(test_name);
            }
        }
    }

    failures
}

/// Extract test name from failure line
pub fn extract_test_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if trimmed.starts_with('✕') {
        // Jest failure marker - skip the Unicode character
        return Some(
            trimmed
                .chars()
                .skip(1)
                .collect::<String>()
                .trim()
                .to_string(),
        );
    }

    if trimmed.starts_with("FAIL") {
        // File-level failure
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(parts[1].to_string());
        }
    }

    None
}

/// Detect test command from package.json
pub fn detect_test_command(package_json: &str) -> Result<String> {
    use serde_json::Value;

    let pkg: Value = serde_json::from_str(package_json)?;

    // Check scripts for test command
    if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
        if scripts.contains_key("test") {
            return Ok("test".to_string());
        }
    }

    // Check devDependencies for framework
    if let Some(deps) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key("vitest") {
            return Ok("vitest".to_string());
        }
        if deps.contains_key("jest") {
            return Ok("jest".to_string());
        }
        if deps.contains_key("mocha") {
            return Ok("mocha".to_string());
        }
    }

    Err(anyhow::anyhow!("No test command found in package.json"))
}
