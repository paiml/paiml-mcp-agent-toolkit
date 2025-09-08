//! MCP Tool for Automated Clippy Fix
//! 
//! A+ Code Standard: ALL functions ≤10 complexity
//! MCP-First Dogfooding: Primary interface for clippy fixes

use crate::services::clippy_fix::{ClippyFixEngine, ClippyDiagnostic, ConfidenceLevel};
use anyhow::Result;
use pmcp::ToolResult;
use serde_json::{json, Value};

/// Auto-fix clippy warnings with confidence-based filtering
/// 
/// Complexity: 8 (within A+ standard ≤10)
pub async fn auto_clippy_fix(
    project_path: Option<String>,
    confidence_level: Option<String>, 
    dry_run: Option<bool>,
    fix_specific_codes: Option<Vec<String>>,
) -> Result<ToolResult> {
    let path = project_path.unwrap_or_else(|| ".".to_string());
    let min_confidence = parse_confidence_level(&confidence_level)?;
    let is_dry_run = dry_run.unwrap_or(false);
    
    // Run clippy and get diagnostics
    let diagnostics = run_clippy_analysis(&path).await?;
    
    // Filter by confidence level
    let engine = ClippyFixEngine::new();
    let filtered = filter_diagnostics(&engine, diagnostics, min_confidence, &fix_specific_codes);
    
    // Apply fixes or show what would be fixed
    let results = if is_dry_run {
        simulate_fixes(&engine, filtered).await?
    } else {
        apply_fixes(&engine, filtered).await?
    };
    
    Ok(create_fix_response(results, is_dry_run))
}

/// Parse confidence level from string (complexity: 3)
fn parse_confidence_level(level: &Option<String>) -> Result<ConfidenceLevel> {
    match level.as_deref() {
        Some("high") => Ok(ConfidenceLevel::High),
        Some("medium") => Ok(ConfidenceLevel::Medium),
        Some("low") => Ok(ConfidenceLevel::Low),
        None => Ok(ConfidenceLevel::High), // Default to safe fixes
        Some(other) => Err(anyhow::anyhow!("Invalid confidence level: {}", other)),
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
        return Err(anyhow::anyhow!("Clippy failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
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
    diagnostics.into_iter()
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
    match (actual, minimum) {
        (ConfidenceLevel::High, _) => true,
        (ConfidenceLevel::Medium, ConfidenceLevel::Low) => true,
        (ConfidenceLevel::Medium, ConfidenceLevel::Medium) => true,
        (ConfidenceLevel::Low, ConfidenceLevel::Low) => true,
        _ => false,
    }
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
    
    let detailed_results: Vec<Value> = results.iter()
        .map(|r| json!({
            "file": r.diagnostic.file,
            "line": r.diagnostic.line_start,
            "code": r.diagnostic.code,
            "success": r.success,
            "error": r.error,
            "duration_ms": r.duration.as_millis(),
        }))
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
        "message": format!("🔧 Clippy fixes {} successfully", action)
    });
    
    ToolResult {
        content: vec![pmcp::Content::Text {
            text: serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
        }],
        is_error: false,
    }
}