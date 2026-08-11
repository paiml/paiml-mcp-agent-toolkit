#![cfg_attr(coverage_nightly, coverage(off))]
use crate::services::deep_context::DeepContextAnalyzer;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
/// Output format options for diagnostic.
pub enum DiagnosticFormat {
    Pretty,
    Json,
    Compact,
}

#[derive(Debug, clap::Args)]
/// Diagnose args.
pub struct DiagnoseArgs {
    /// Output format for diagnostic report
    #[arg(long, value_enum, default_value = "pretty")]
    pub format: DiagnosticFormat,

    /// Only run specific feature tests (can be repeated)
    #[arg(long)]
    pub only: Vec<String>,

    /// Skip specific feature tests (can be repeated)
    #[arg(long)]
    pub skip: Vec<String>,

    /// Maximum time to run diagnostics (in seconds)
    #[arg(long, default_value = "60")]
    pub timeout: u64,
}

#[derive(Debug, Serialize)]
/// Report containing diagnostic data.
pub struct DiagnosticReport {
    pub version: String,
    pub build_info: BuildInfo,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub features: BTreeMap<String, FeatureResult>,
    pub summary: DiagnosticSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_context: Option<CompactErrorContext>,
}

#[derive(Debug, Serialize)]
/// Information about build.
pub struct BuildInfo {
    pub rust_version: String,
    pub build_date: String,
    pub git_commit: Option<String>,
    pub features: Vec<String>,
}

impl BuildInfo {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Current.
    pub fn current() -> Self {
        Self {
            rust_version: option_env!("RUSTC_VERSION")
                .unwrap_or("unknown")
                .to_string(),
            build_date: option_env!("BUILD_DATE").unwrap_or("unknown").to_string(),
            git_commit: option_env!("GIT_HASH").map(String::from),
            features: vec!["cli".to_string()],
        }
    }
}

#[derive(Debug, Serialize)]
/// Result of feature operation.
pub struct FeatureResult {
    pub status: FeatureStatus,
    pub duration_us: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
/// Status of feature operation.
pub enum FeatureStatus {
    Ok,
    Degraded(String),
    Failed,
    Skipped(String),
}

#[derive(Debug, Serialize)]
/// Summary of diagnostic analysis.
pub struct DiagnosticSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub degraded: usize,
    pub skipped: usize,
    pub all_passed: bool,
    /// Share of the checks that actually EXECUTED and passed.
    ///
    /// The denominator used to be `total`, which counts skipped checks, so
    /// `--skip ast.rust` reported "success_rate: 87.5" next to `failed: 0`
    /// and `all_passed: true` — a rate that fell as the user skipped more,
    /// with nothing failing. `None` (null) when no check ran at all: there is
    /// no rate to report, and `0.0` there read as "everything failed".
    pub success_rate: Option<f64>,
}

#[derive(Debug, Serialize)]
/// Context for compact error operations.
pub struct CompactErrorContext {
    pub failed_features: Vec<String>,
    pub error_patterns: BTreeMap<String, Vec<String>>,
    pub suggested_fixes: Vec<SuggestedFix>,
    pub environment: EnvironmentSnapshot,
}

#[derive(Debug, Serialize)]
/// Fix suggestion for suggested.
pub struct SuggestedFix {
    pub feature: String,
    pub error_pattern: String,
    pub fix_command: Option<String>,
    pub documentation_link: Option<String>,
}

#[derive(Debug, Serialize)]
/// Point-in-time snapshot of environment.
pub struct EnvironmentSnapshot {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub memory_mb: u64,
    pub cwd: String,
}

impl EnvironmentSnapshot {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    #[must_use]
    /// Capture.
    pub fn capture() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: num_cpus::get(),
            memory_mb: {
                #[cfg(feature = "diagnostics")]
                {
                    sys_info::mem_info().map(|m| m.total / 1024).unwrap_or(0)
                }
                #[cfg(not(feature = "diagnostics"))]
                {
                    0
                }
            },
            cwd: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        }
    }
}

#[async_trait::async_trait]
/// Test interface for feature.
pub trait FeatureTest: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(&self) -> Result<serde_json::Value>;
}

/// Self diagnostic.
pub struct SelfDiagnostic {
    tests: Vec<Box<dyn FeatureTest>>,
}

// Feature test implementations (AST, cache, mermaid, complexity, deep context, git)
include!("diagnose_feature_tests.rs");

// SelfDiagnostic runner, error context extraction, output formatting, handle_diagnose
include!("diagnose_runner.rs");

// Property-based tests
include!("diagnose_proptests.rs");

#[cfg(test)]
#[path = "diagnose_tests.rs"]
mod tests;
