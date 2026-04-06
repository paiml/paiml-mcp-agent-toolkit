// Kaizen scanning: clippy and rustfmt scanners
// Included from scanning.rs — no `use` imports, shares parent scope.

/// Scan for clippy warnings using JSON output
fn scan_clippy(path: &Path) -> Result<Vec<KaizenFinding>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let output = Command::new("cargo")
        .args(["clippy", "--message-format=json", "--quiet"])
        .current_dir(path)
        .output()
        .context("Failed to run cargo clippy")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();

    for line in stdout.lines() {
        let json: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if json.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }
        let Some(message) = json.get("message") else {
            continue;
        };
        let level = message.get("level").and_then(|l| l.as_str()).unwrap_or("");
        if level != "warning" {
            continue;
        }

        let code = message
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        let rendered = message
            .get("rendered")
            .and_then(|r| r.as_str())
            .unwrap_or("");

        let file = extract_file_from_message(message);

        findings.push(KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Medium,
            category: format!("clippy::{code}"),
            message: first_line(rendered),
            file,
            auto_fixable: true,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
            crate_name: None,
        });
    }

    Ok(findings)
}

/// Scan for unformatted files
fn scan_rustfmt(path: &Path) -> Result<Vec<KaizenFinding>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let output = Command::new("cargo")
        .args(["fmt", "--", "--check", "-l"])
        .current_dir(path)
        .output()
        .context("Failed to run cargo fmt --check")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        findings.push(KaizenFinding {
            source: FindingSource::Rustfmt,
            severity: FindingSeverity::Low,
            category: "rustfmt::unformatted".to_string(),
            message: format!("File needs formatting: {line}"),
            file: Some(line.to_string()),
            auto_fixable: true,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
            crate_name: None,
        });
    }

    Ok(findings)
}
