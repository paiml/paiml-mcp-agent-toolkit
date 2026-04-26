//! Maps MCP tool parameters to uniform contracts
//! This ensures MCP uses the exact same contracts as CLI and HTTP
#![cfg_attr(coverage_nightly, coverage(off))]

use super::{
    AnalyzeComplexityContract, AnalyzeDeadCodeContract, AnalyzeLintHotspotContract,
    AnalyzeSatdContract, AnalyzeTdgContract, BaseAnalysisContract, ContractValidation,
    OutputFormat, QualityGateContract, QualityProfile, RefactorAutoContract, SatdSeverity,
};
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

/// Convert MCP tool parameters to uniform contracts
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn map_mcp_tool(tool_name: &str, params: Value) -> Result<Box<dyn ContractValidation>> {
    match tool_name {
        "analyze_complexity" => map_complexity_contract(&params),
        "analyze_satd" => map_satd_contract(&params),
        "analyze_dead_code" => map_dead_code_contract(&params),
        "analyze_tdg" => map_tdg_contract(&params),
        "analyze_lint_hotspot" => map_lint_hotspot_contract(&params),
        "quality_gate" => map_quality_gate_contract(&params),
        "refactor_auto" => map_refactor_auto_contract(&params),
        _ => anyhow::bail!("Unknown MCP tool: {tool_name}"),
    }
}

fn map_complexity_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let contract = AnalyzeComplexityContract {
        base: parse_base_params(params)?,
        max_cyclomatic: params["max_cyclomatic"].as_u64().map(|n| n as u32),
        max_cognitive: params["max_cognitive"].as_u64().map(|n| n as u32),
        max_halstead: params["max_halstead"].as_f64(),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

fn map_satd_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let contract = AnalyzeSatdContract {
        base: parse_base_params(params)?,
        severity: parse_severity(&params["severity"]),
        critical_only: params["critical_only"].as_bool().unwrap_or(false),
        strict: params["strict"].as_bool().unwrap_or(false),
        fail_on_violation: params["fail_on_violation"].as_bool().unwrap_or(false),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

fn map_dead_code_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let contract = AnalyzeDeadCodeContract {
        base: parse_base_params(params)?,
        include_unreachable: params["include_unreachable"].as_bool().unwrap_or(false),
        min_dead_lines: params["min_dead_lines"].as_u64().unwrap_or(10) as usize,
        max_percentage: params["max_percentage"].as_f64().unwrap_or(15.0),
        fail_on_violation: params["fail_on_violation"].as_bool().unwrap_or(false),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

fn map_tdg_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let contract = AnalyzeTdgContract {
        base: parse_base_params(params)?,
        threshold: params["threshold"].as_f64().unwrap_or(1.5),
        include_components: params["include_components"].as_bool().unwrap_or(false),
        critical_only: params["critical_only"].as_bool().unwrap_or(false),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

fn map_lint_hotspot_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let contract = AnalyzeLintHotspotContract {
        base: parse_base_params(params)?,
        file: params["file"].as_str().map(PathBuf::from),
        max_density: params["max_density"].as_f64().unwrap_or(5.0),
        min_confidence: params["min_confidence"].as_f64().unwrap_or(0.8),
        enforce: params["enforce"].as_bool().unwrap_or(false),
        dry_run: params["dry_run"].as_bool().unwrap_or(false),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

fn map_quality_gate_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let contract = QualityGateContract {
        base: parse_base_params(params)?,
        profile: parse_quality_profile(&params["profile"]),
        file: params["file"].as_str().map(PathBuf::from),
        fail_on_violation: params["fail_on_violation"].as_bool().unwrap_or(false),
        verbose: params["verbose"].as_bool().unwrap_or(false),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

fn map_refactor_auto_contract(params: &Value) -> Result<Box<dyn ContractValidation>> {
    let file_path = params["file"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file"))?;

    let contract = RefactorAutoContract {
        file: PathBuf::from(file_path),
        format: parse_output_format(&params["format"]),
        output: params["output"].as_str().map(PathBuf::from),
        target_complexity: params["target_complexity"].as_u64().unwrap_or(8) as u32,
        dry_run: params["dry_run"].as_bool().unwrap_or(false),
        timeout: params["timeout"].as_u64().unwrap_or(60),
    };
    contract.validate()?;
    Ok(Box::new(contract))
}

/// Parse base parameters that are common to all analysis commands
fn parse_base_params(params: &Value) -> Result<BaseAnalysisContract> {
    let path = params["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;

    Ok(BaseAnalysisContract {
        path: PathBuf::from(path),
        format: parse_output_format(&params["format"]),
        output: params["output"].as_str().map(PathBuf::from),
        top_files: params["top_files"].as_u64().map(|n| n as usize),
        include_tests: params["include_tests"].as_bool().unwrap_or(false),
        timeout: params["timeout"].as_u64().unwrap_or(60),
    })
}

fn parse_output_format(value: &Value) -> OutputFormat {
    value
        .as_str()
        .and_then(|s| match s {
            "table" => Some(OutputFormat::Table),
            "json" => Some(OutputFormat::Json),
            "yaml" => Some(OutputFormat::Yaml),
            "markdown" => Some(OutputFormat::Markdown),
            "csv" => Some(OutputFormat::Csv),
            "summary" => Some(OutputFormat::Summary),
            _ => None,
        })
        .unwrap_or_default()
}

fn parse_severity(value: &Value) -> Option<SatdSeverity> {
    value.as_str().and_then(|s| match s {
        "low" => Some(SatdSeverity::Low),
        "medium" => Some(SatdSeverity::Medium),
        "high" => Some(SatdSeverity::High),
        "critical" => Some(SatdSeverity::Critical),
        _ => None,
    })
}

fn parse_quality_profile(value: &Value) -> QualityProfile {
    value
        .as_str()
        .and_then(|s| match s {
            "standard" => Some(QualityProfile::Standard),
            "strict" => Some(QualityProfile::Strict),
            "extreme" => Some(QualityProfile::Extreme),
            "toyota" => Some(QualityProfile::Toyota),
            _ => None,
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        TempDir::new().unwrap()
    }

    // ── parse_output_format ─────────────────────────────────────────────────

    #[test]
    fn test_parse_output_format_all_known_arms() {
        assert_eq!(parse_output_format(&json!("table")), OutputFormat::Table);
        assert_eq!(parse_output_format(&json!("json")), OutputFormat::Json);
        assert_eq!(parse_output_format(&json!("yaml")), OutputFormat::Yaml);
        assert_eq!(
            parse_output_format(&json!("markdown")),
            OutputFormat::Markdown
        );
        assert_eq!(parse_output_format(&json!("csv")), OutputFormat::Csv);
        assert_eq!(
            parse_output_format(&json!("summary")),
            OutputFormat::Summary
        );
    }

    #[test]
    fn test_parse_output_format_unknown_falls_back_to_default() {
        assert_eq!(
            parse_output_format(&json!("invalid")),
            OutputFormat::default()
        );
        assert_eq!(parse_output_format(&Value::Null), OutputFormat::default());
        assert_eq!(parse_output_format(&json!(42)), OutputFormat::default());
    }

    // ── parse_severity ──────────────────────────────────────────────────────

    #[test]
    fn test_parse_severity_all_known_arms() {
        assert_eq!(parse_severity(&json!("low")), Some(SatdSeverity::Low));
        assert_eq!(parse_severity(&json!("medium")), Some(SatdSeverity::Medium));
        assert_eq!(parse_severity(&json!("high")), Some(SatdSeverity::High));
        assert_eq!(
            parse_severity(&json!("critical")),
            Some(SatdSeverity::Critical)
        );
    }

    #[test]
    fn test_parse_severity_unknown_returns_none() {
        assert!(parse_severity(&json!("invalid")).is_none());
        assert!(parse_severity(&Value::Null).is_none());
        assert!(parse_severity(&json!(123)).is_none());
    }

    // ── parse_quality_profile ───────────────────────────────────────────────

    #[test]
    fn test_parse_quality_profile_all_known_arms() {
        assert_eq!(
            parse_quality_profile(&json!("standard")),
            QualityProfile::Standard
        );
        assert_eq!(
            parse_quality_profile(&json!("strict")),
            QualityProfile::Strict
        );
        assert_eq!(
            parse_quality_profile(&json!("extreme")),
            QualityProfile::Extreme
        );
        assert_eq!(
            parse_quality_profile(&json!("toyota")),
            QualityProfile::Toyota
        );
    }

    #[test]
    fn test_parse_quality_profile_unknown_falls_back_to_default() {
        assert_eq!(
            parse_quality_profile(&json!("nope")),
            QualityProfile::default()
        );
        assert_eq!(
            parse_quality_profile(&Value::Null),
            QualityProfile::default()
        );
    }

    // ── parse_base_params ───────────────────────────────────────────────────

    #[test]
    fn test_parse_base_params_minimum_path_only() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let v = json!({ "path": path_str });
        let base = parse_base_params(&v).unwrap();
        assert_eq!(base.path, dir.path());
        assert_eq!(base.format, OutputFormat::default());
        assert!(base.output.is_none());
        assert!(base.top_files.is_none());
        assert!(!base.include_tests);
        assert_eq!(base.timeout, 60);
    }

    #[test]
    fn test_parse_base_params_full_overrides() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let v = json!({
            "path": path_str,
            "format": "json",
            "output": "/tmp/out.json",
            "top_files": 10,
            "include_tests": true,
            "timeout": 120,
        });
        let base = parse_base_params(&v).unwrap();
        assert_eq!(base.format, OutputFormat::Json);
        assert_eq!(base.output, Some(PathBuf::from("/tmp/out.json")));
        assert_eq!(base.top_files, Some(10));
        assert!(base.include_tests);
        assert_eq!(base.timeout, 120);
    }

    #[test]
    fn test_parse_base_params_missing_path_errors() {
        let v = json!({});
        let err = parse_base_params(&v).unwrap_err();
        assert!(err.to_string().contains("path"));
    }

    // ── map_mcp_tool dispatcher ─────────────────────────────────────────────

    #[test]
    fn test_map_mcp_tool_complexity_happy_path() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let params = json!({ "path": path_str });
        let res = map_mcp_tool("analyze_complexity", params);
        assert!(res.is_ok());
    }

    #[test]
    fn test_map_mcp_tool_satd_happy_path() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let params = json!({ "path": path_str, "severity": "high", "critical_only": true });
        assert!(map_mcp_tool("analyze_satd", params).is_ok());
    }

    #[test]
    fn test_map_mcp_tool_dead_code_happy_path() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let params = json!({ "path": path_str, "min_dead_lines": 25, "max_percentage": 10.0 });
        assert!(map_mcp_tool("analyze_dead_code", params).is_ok());
    }

    #[test]
    fn test_map_mcp_tool_tdg_happy_path() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let params = json!({ "path": path_str, "threshold": 2.0 });
        assert!(map_mcp_tool("analyze_tdg", params).is_ok());
    }

    #[test]
    fn test_map_mcp_tool_lint_hotspot_happy_path() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let params = json!({ "path": path_str, "max_density": 3.0 });
        assert!(map_mcp_tool("analyze_lint_hotspot", params).is_ok());
    }

    #[test]
    fn test_map_mcp_tool_quality_gate_happy_path() {
        let dir = tmp();
        let path_str = dir.path().to_string_lossy().to_string();
        let params = json!({ "path": path_str, "profile": "strict" });
        assert!(map_mcp_tool("quality_gate", params).is_ok());
    }

    #[test]
    fn test_map_mcp_tool_refactor_auto_happy_path() {
        let dir = tmp();
        // refactor_auto's validate() calls PathValidator::ensure_file (must be
        // a regular file), so a temp dir alone is not enough — write a file.
        let file = dir.path().join("src.rs");
        std::fs::write(&file, "// stub\n").unwrap();
        let params = json!({ "file": file.to_string_lossy(), "format": "json" });
        assert!(map_mcp_tool("refactor_auto", params).is_ok());
    }

    fn err_string(res: Result<Box<dyn ContractValidation>>) -> String {
        match res {
            Ok(_) => panic!("expected Err"),
            Err(e) => e.to_string(),
        }
    }

    #[test]
    fn test_map_mcp_tool_refactor_auto_missing_file_errors() {
        let s = err_string(map_mcp_tool("refactor_auto", json!({})));
        assert!(s.contains("file"));
    }

    #[test]
    fn test_map_mcp_tool_unknown_tool_errors() {
        let s = err_string(map_mcp_tool("nonexistent_tool", json!({})));
        assert!(s.contains("Unknown MCP tool"));
        assert!(s.contains("nonexistent_tool"));
    }

    #[test]
    fn test_map_mcp_tool_missing_path_errors_via_base_params() {
        // Most analyze_* tools delegate to parse_base_params which requires `path`.
        let s = err_string(map_mcp_tool("analyze_complexity", json!({})));
        assert!(s.contains("path"));
    }

    #[test]
    fn test_map_mcp_tool_path_does_not_exist_errors_via_validate() {
        // parse_base_params accepts the path string, then validate() fails.
        let params = json!({ "path": "/nonexistent/definitely/not/here/xyz" });
        let res = map_mcp_tool("analyze_complexity", params);
        assert!(res.is_err());
    }
}
