// Core helper functions for auto_clippy_fix: parsing, filtering, simulation,
// fix application, and MCP response creation.

/// Parse confidence level from string (complexity: 3)
fn parse_confidence_level(level: &Option<String>) -> Result<ConfidenceLevel> {
    match level.as_deref() {
        Some("high") => Ok(ConfidenceLevel::High),
        Some("medium") => Ok(ConfidenceLevel::Medium),
        Some("low") => Ok(ConfidenceLevel::Low),
        None => Ok(ConfidenceLevel::High), // Default to safe fixes
        Some(other) => Err(anyhow::anyhow!("Invalid confidence level: {other}")),
    }
}

/// Run clippy analysis and parse output (complexity: 5)
async fn run_clippy_analysis(path: &str) -> Result<Vec<ClippyDiagnostic>> {
    use tokio::process::Command;

    let output = Command::new("cargo")
        .args(["clippy", "--message-format=json"])
        .current_dir(path)
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Clippy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    parse_clippy_output(&String::from_utf8_lossy(&output.stdout))
}

/// Parse clippy JSON output (complexity: 6)
fn parse_clippy_output(output: &str) -> Result<Vec<ClippyDiagnostic>> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(diagnostic) = ClippyDiagnostic::from_json(line) {
            diagnostics.push(diagnostic);
        }
    }

    Ok(diagnostics)
}

/// Filter diagnostics by criteria (complexity: 7)
fn filter_diagnostics(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
    min_confidence: ConfidenceLevel,
    specific_codes: &Option<Vec<String>>,
) -> Vec<ClippyDiagnostic> {
    diagnostics
        .into_iter()
        .filter(|d| {
            let confidence = engine.calculate_confidence(d);
            confidence_meets_minimum(confidence, min_confidence.clone())
        })
        .filter(|d| {
            if let Some(codes) = specific_codes {
                codes.contains(&d.code)
            } else {
                true
            }
        })
        .collect()
}

/// Check if confidence meets minimum (complexity: 3)
fn confidence_meets_minimum(actual: ConfidenceLevel, minimum: ConfidenceLevel) -> bool {
    matches!(
        (actual, minimum),
        (ConfidenceLevel::High, _)
            | (ConfidenceLevel::Medium, ConfidenceLevel::Low)
            | (ConfidenceLevel::Medium, ConfidenceLevel::Medium)
            | (ConfidenceLevel::Low, ConfidenceLevel::Low)
    )
}

/// Simulate fixes without applying (complexity: 4)
async fn simulate_fixes(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
) -> Result<Value> {
    let mut fixes = Vec::new();

    for diagnostic in diagnostics {
        let confidence = engine.calculate_confidence(&diagnostic);
        fixes.push(json!({
            "file": diagnostic.file,
            "line": diagnostic.line_start,
            "code": diagnostic.code,
            "message": diagnostic.message,
            "confidence": format!("{:?}", confidence),
            "would_fix": true,
        }));
    }

    Ok(json!({
        "dry_run": true,
        "total_fixes": fixes.len(),
        "fixes": fixes,
    }))
}

/// Apply fixes to code (complexity: 5)
async fn apply_fixes(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
) -> Result<Value> {
    let results = engine.apply_batch_fixes(&diagnostics).await?;
    let report = engine.generate_report(results.clone());

    let detailed_results: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "file": r.diagnostic.file,
                "line": r.diagnostic.line_start,
                "code": r.diagnostic.code,
                "success": r.success,
                "error": r.error,
                "duration_ms": r.duration.as_millis(),
            })
        })
        .collect();

    Ok(json!({
        "dry_run": false,
        "report": {
            "total_diagnostics": report.total_diagnostics,
            "successful_fixes": report.successful_fixes,
            "failed_fixes": report.failed_fixes,
            "success_rate": report.success_rate,
            "total_duration_ms": report.total_duration.as_millis(),
            "fixed_files": report.fixed_files,
        },
        "detailed_results": detailed_results,
    }))
}

/// Create MCP response (complexity: 2)
fn create_fix_response(results: Value, is_dry_run: bool) -> ToolResult {
    let action = if is_dry_run { "analyzed" } else { "applied" };

    let response = json!({
        "action": action,
        "results": results,
        "message": format!("\u{1f527} Clippy fixes {} successfully", action)
    });

    ToolResult::new(vec![pmcp::Content::Text {
        text: serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    }])
}
