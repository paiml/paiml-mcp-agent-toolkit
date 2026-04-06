// Signal collector implementations for RustcCollector, ClippyCollector, and TestCollector.
// Included by signal_collector.rs - shares parent module scope.

/// Extract the "message" object from a cargo JSON diagnostic line.
/// Returns `None` if the line isn't valid JSON, isn't a "compiler-message",
/// or has no "message" field.
fn extract_compiler_message(line: &str) -> Option<serde_json::Value> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let json: serde_json::Value = serde_json::from_str(line).ok()?;
    if json.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
        return None;
    }
    json.get("message").cloned()
}

/// Extract error code and rendered text from a cargo diagnostic message object.
fn extract_code_and_rendered(message: &serde_json::Value) -> (Option<String>, String) {
    let code = message
        .get("code")
        .and_then(|c| c.get("code"))
        .and_then(|c| c.as_str())
        .map(String::from);

    let rendered = message
        .get("rendered")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    (code, rendered)
}

/// Compute clippy lint weight based on the error code category.
fn clippy_weight(code: &Option<String>) -> f32 {
    let code_str = match code.as_deref() {
        Some(s) => s,
        None => return 0.5,
    };
    if code_str.starts_with("clippy::correctness") {
        1.0
    } else if code_str.starts_with("clippy::suspicious") {
        0.9
    } else if code_str.starts_with("clippy::complexity") {
        0.7
    } else {
        0.5
    }
}

#[async_trait]
impl SignalCollector for RustcCollector {
    fn source(&self) -> SignalSource {
        SignalSource::Rustc
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let output = Command::new("cargo")
            .args(["build", "--message-format=json"])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let signals = stdout
            .lines()
            .filter_map(|line| {
                let message = extract_compiler_message(line)?;
                let level = message.get("level").and_then(|l| l.as_str())?;
                if level != "error" {
                    return None;
                }
                let (code, rendered) = extract_code_and_rendered(&message);
                Some(SignalEvidence {
                    source: SignalSource::Rustc,
                    raw_message: rendered,
                    error_code: code,
                    weight: 1.0,
                })
            })
            .collect();

        Ok(signals)
    }
}

#[async_trait]
impl SignalCollector for ClippyCollector {
    fn source(&self) -> SignalSource {
        SignalSource::Clippy
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let output = Command::new("cargo")
            .args(["clippy", "--message-format=json", "--", "-D", "warnings"])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let signals = stdout
            .lines()
            .filter_map(|line| {
                let message = extract_compiler_message(line)?;
                let level = message.get("level").and_then(|l| l.as_str())?;
                if level != "warning" && level != "error" {
                    return None;
                }
                let (code, rendered) = extract_code_and_rendered(&message);
                let weight = clippy_weight(&code);
                Some(SignalEvidence {
                    source: SignalSource::Clippy,
                    raw_message: rendered,
                    error_code: code,
                    weight,
                })
            })
            .collect();

        Ok(signals)
    }
}

#[async_trait]
impl SignalCollector for TestCollector {
    fn source(&self) -> SignalSource {
        SignalSource::CargoTest
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let output = Command::new("cargo")
            .args([
                "test",
                "--no-fail-fast",
                "--",
                "--format=json",
                "-Z",
                "unstable-options",
            ])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let signals = stdout
            .lines()
            .filter_map(|line| {
                let json: serde_json::Value = serde_json::from_str(line).ok()?;
                let is_failed_test =
                    json.get("type").and_then(|t| t.as_str()) == Some("test")
                        && json.get("event").and_then(|e| e.as_str()) == Some("failed");
                if !is_failed_test {
                    return None;
                }
                let name = json
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                let stdout_text = json
                    .get("stdout")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                Some(SignalEvidence {
                    source: SignalSource::CargoTest,
                    raw_message: format!("Test failed: {}\n{}", name, stdout_text),
                    error_code: None,
                    weight: 1.0,
                })
            })
            .collect();

        Ok(signals)
    }
}
