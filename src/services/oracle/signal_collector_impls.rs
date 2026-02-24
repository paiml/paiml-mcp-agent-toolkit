// Signal collector implementations for RustcCollector, ClippyCollector, and TestCollector.
// Included by signal_collector.rs - shares parent module scope.

#[async_trait]
impl SignalCollector for RustcCollector {
    fn source(&self) -> SignalSource {
        SignalSource::Rustc
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        let output = Command::new("cargo")
            .args(["build", "--message-format=json"])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut signals = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(message) = json.get("message") {
                        if let Some(level) = message.get("level").and_then(|l| l.as_str()) {
                            if level == "error" {
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

                                signals.push(SignalEvidence {
                                    source: SignalSource::Rustc,
                                    raw_message: rendered,
                                    error_code: code,
                                    weight: 1.0,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(signals)
    }
}

#[async_trait]
impl SignalCollector for ClippyCollector {
    fn source(&self) -> SignalSource {
        SignalSource::Clippy
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        let output = Command::new("cargo")
            .args(["clippy", "--message-format=json", "--", "-D", "warnings"])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut signals = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(message) = json.get("message") {
                        if let Some(level) = message.get("level").and_then(|l| l.as_str()) {
                            if level == "warning" || level == "error" {
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

                                // Weight based on lint category
                                let weight = if code
                                    .as_ref()
                                    .map(|c| c.starts_with("clippy::correctness"))
                                    .unwrap_or(false)
                                {
                                    1.0
                                } else if code
                                    .as_ref()
                                    .map(|c| c.starts_with("clippy::suspicious"))
                                    .unwrap_or(false)
                                {
                                    0.9
                                } else if code
                                    .as_ref()
                                    .map(|c| c.starts_with("clippy::complexity"))
                                    .unwrap_or(false)
                                {
                                    0.7
                                } else {
                                    0.5
                                };

                                signals.push(SignalEvidence {
                                    source: SignalSource::Clippy,
                                    raw_message: rendered,
                                    error_code: code,
                                    weight,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(signals)
    }
}

#[async_trait]
impl SignalCollector for TestCollector {
    fn source(&self) -> SignalSource {
        SignalSource::CargoTest
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
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
        let mut signals = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json.get("type").and_then(|t| t.as_str()) == Some("test")
                    && json.get("event").and_then(|e| e.as_str()) == Some("failed")
                {
                    let name = json
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let stdout_text = json
                        .get("stdout")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    signals.push(SignalEvidence {
                        source: SignalSource::CargoTest,
                        raw_message: format!("Test failed: {}\n{}", name, stdout_text),
                        error_code: None,
                        weight: 1.0,
                    });
                }
            }
        }

        Ok(signals)
    }
}
