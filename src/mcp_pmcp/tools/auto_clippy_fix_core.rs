// Core helper functions for auto_clippy_fix: parsing, filtering, preview, and
// MCP response creation.
//
// There is deliberately no fix-application helper here. See `simulate_fixes`
// and `create_fix_response` below, and #1086.

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

/// Describe the eligible diagnostics. Reads only; writes nothing.
///
/// Every key here names a description, not an edit. The previous payload said
/// `"dry_run": true`, `"total_fixes"` and `"would_fix": true` — a mode flag
/// implying a counterpart that writes, a count of "fixes", and a per-entry
/// promise — for a module that contains no writer (#1086). `would_fix` was also
/// a hardcoded `true` on every entry, so it distinguished nothing.
///
/// Complexity: 4
async fn simulate_fixes(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
) -> Result<Value> {
    let mut previewed = Vec::new();

    for diagnostic in diagnostics {
        let confidence = engine.calculate_confidence(&diagnostic);
        previewed.push(json!({
            "file": diagnostic.file,
            "line": diagnostic.line_start,
            "code": diagnostic.code,
            "message": diagnostic.message,
            "confidence": format!("{:?}", confidence),
        }));
    }

    Ok(json!({
        "preview_only": true,
        "total_previewed": previewed.len(),
        "previewed": previewed,
    }))
}

/// What clippy reported, beside what we were willing to act on.
///
/// These are different numbers and were reported as one. See the comment at the
/// call site in `auto_clippy_fix`.
struct DiagnosticCensus {
    /// Diagnostics `cargo clippy` emitted, before any filtering of ours.
    found: usize,
    /// Of those, the ones that met `min_confidence` and the code filter.
    eligible: usize,
    /// The confidence bar that did the filtering, so the gap is explainable.
    min_confidence: String,
}

impl DiagnosticCensus {
    fn with_eligible(self, eligible: usize) -> Self {
        Self { eligible, ..self }
    }

    /// A sentence that cannot say "successfully" about work not done.
    fn message(&self, action: &str) -> String {
        if self.found == 0 {
            let _ = action;
            return "\u{2705} clippy reported no diagnostics; nothing to fix".to_string();
        }
        let skipped = self.found.saturating_sub(self.eligible);
        if self.eligible == 0 {
            return format!(
                "\u{26a0}\u{fe0f} clippy reported {} diagnostic(s), and none met the required \
                 confidence ({}) — {} left untouched. This is NOT a clean result; \
                 re-run with --confidence low to see them.",
                self.found, self.min_confidence, skipped
            );
        }
        format!(
            "\u{1f50e} clippy reported {} diagnostic(s); {} met confidence {} and were {}, \
             {} left untouched. No file was modified — `analyze clippy` previews only.",
            self.found, self.eligible, self.min_confidence, action, skipped
        )
    }
}

/// Create MCP response (complexity: 2)
///
/// `action` is a constant, and the `is_dry_run` parameter that used to choose it
/// is gone. It read `if is_dry_run { "analyzed" } else { "applied" }`, and the
/// "applied" arm was reached by a code path that wrote nothing to disk (#1086).
/// A field that can only ever report what actually happened cannot be made to
/// lie by a caller passing the wrong flag.
fn create_fix_response(results: Value, census: &DiagnosticCensus) -> ToolResult {
    // Nothing in this module writes, in any mode, so this is the only verb the
    // response is entitled to use.
    const ACTION: &str = "previewed";

    let response = json!({
        "action": ACTION,
        "diagnostics_found": census.found,
        "diagnostics_eligible": census.eligible,
        "diagnostics_filtered_out": census.found.saturating_sub(census.eligible),
        "min_confidence": census.min_confidence,
        "results": results,
        "message": census.message(ACTION)
    });

    ToolResult::new(vec![pmcp::Content::Text {
        text: serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    }])
}
