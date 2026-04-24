#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::services::agent_context::FunctionEntry;
use crate::services::duplicate_detector::{Language, MinHashSignature};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// --- Public types ---

#[derive(Debug, Clone, Serialize)]
/// Information about crate.
pub struct CrateInfo {
    pub name: String,
    pub path: PathBuf,
    pub cargo_deps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Severity level classification for cc.
pub enum CcSeverity {
    Error,
    Warning,
    Advisory,
}

impl std::fmt::Display for CcSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CcSeverity::Error => write!(f, "error"),
            CcSeverity::Warning => write!(f, "warning"),
            CcSeverity::Advisory => write!(f, "advisory"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
/// Cross crate finding.
pub struct CrossCrateFinding {
    pub rule: String,
    pub severity: CcSeverity,
    pub crate_a: String,
    pub crate_b: String,
    pub function_a: String,
    pub function_b: String,
    pub file_a: String,
    pub file_b: String,
    pub similarity: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
/// Summary of cross crate analysis.
pub struct CrossCrateSummary {
    pub total_findings: usize,
    pub errors: usize,
    pub warnings: usize,
    pub advisories: usize,
    pub rules_triggered: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
/// Report containing cross crate data.
pub struct CrossCrateReport {
    pub findings: Vec<CrossCrateFinding>,
    pub summary: CrossCrateSummary,
    pub crates_analyzed: Vec<String>,
}

// --- Internal types ---

/// A function with its computed MinHash signature, grouped by crate.
pub(super) struct SignedFunction {
    pub(super) crate_name: String,
    pub(super) function_name: String,

    pub(super) signature: String,
    pub(super) file_path: String,
    pub(super) minhash: MinHashSignature,

    pub(super) language: Language,
}

/// Ratchet baseline — persisted to `.pmat/cross-crate-baseline.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CrossCrateBaseline {
    pub(super) version: String,
    pub(super) generated: String,
    pub(super) rule_counts: HashMap<String, usize>,
    pub(super) total_findings: usize,
}

/// Configuration context passed through detection functions.
pub(super) struct DetectionConfig {
    pub(super) excluded_functions: HashSet<String>,
    pub(super) excluded_crate_pairs: HashSet<(String, String)>,
    pub(super) min_body_lines: usize,
    pub(super) min_tokens: usize,
    pub(super) cc003_min_similarity: f64,
}

impl DetectionConfig {
    pub(super) fn from_yaml(cc: &crate::models::comply_config::CrossCrateConfig) -> Self {
        let excluded_functions: HashSet<String> = cc
            .excluded_functions
            .iter()
            .map(|s| s.to_lowercase())
            .collect();
        let excluded_crate_pairs: HashSet<(String, String)> = cc
            .excluded_crate_pairs
            .iter()
            .filter_map(|pair| {
                let parts: Vec<&str> = pair.split(':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();
        Self {
            excluded_functions,
            excluded_crate_pairs,
            min_body_lines: cc.min_body_lines,
            min_tokens: cc.min_tokens,
            cc003_min_similarity: cc.cc003_min_similarity,
        }
    }
}

/// A function reference with crate context for CC-003/CC-004.
pub(super) struct CrateFuncRef<'a> {
    pub(super) crate_info: &'a CrateInfo,
    pub(super) func: &'a FunctionEntry,
}

#[cfg(test)]
mod cross_crate_types_tests {
    //! Covers CcSeverity::Display + DetectionConfig::from_yaml in
    //! cross_crate_handlers/types.rs (32 uncov on broad, 0% cov).
    use super::*;

    #[test]
    fn test_cc_severity_display_all_variants() {
        assert_eq!(format!("{}", CcSeverity::Error), "error");
        assert_eq!(format!("{}", CcSeverity::Warning), "warning");
        assert_eq!(format!("{}", CcSeverity::Advisory), "advisory");
    }

    #[test]
    fn test_detection_config_from_yaml_lowercases_excluded_functions() {
        let cc = crate::models::comply_config::CrossCrateConfig {
            excluded_functions: vec!["Serialize".into(), "DEBUG".into()],
            excluded_crate_pairs: vec![],
            min_body_lines: 5,
            min_tokens: 20,
            cc003_min_similarity: 0.9,
            ..Default::default()
        };
        let dc = DetectionConfig::from_yaml(&cc);
        assert!(dc.excluded_functions.contains("serialize"));
        assert!(dc.excluded_functions.contains("debug"));
        // Original casing dropped.
        assert!(!dc.excluded_functions.contains("Serialize"));
    }

    #[test]
    fn test_detection_config_from_yaml_parses_crate_pairs_colon_separated() {
        let cc = crate::models::comply_config::CrossCrateConfig {
            excluded_functions: vec![],
            excluded_crate_pairs: vec![
                "crate_a:crate_b".into(),
                "foo:bar".into(),
                "no_colon".into(),        // malformed, dropped
                "too:many:colons".into(), // malformed (3 parts), dropped
            ],
            min_body_lines: 0,
            min_tokens: 0,
            cc003_min_similarity: 0.0,
            ..Default::default()
        };
        let dc = DetectionConfig::from_yaml(&cc);
        assert_eq!(dc.excluded_crate_pairs.len(), 2);
        assert!(dc
            .excluded_crate_pairs
            .contains(&("crate_a".to_string(), "crate_b".to_string())));
        assert!(dc
            .excluded_crate_pairs
            .contains(&("foo".to_string(), "bar".to_string())));
    }

    #[test]
    fn test_detection_config_from_yaml_forwards_thresholds() {
        let cc = crate::models::comply_config::CrossCrateConfig {
            excluded_functions: vec![],
            excluded_crate_pairs: vec![],
            min_body_lines: 42,
            min_tokens: 100,
            cc003_min_similarity: 0.75,
            ..Default::default()
        };
        let dc = DetectionConfig::from_yaml(&cc);
        assert_eq!(dc.min_body_lines, 42);
        assert_eq!(dc.min_tokens, 100);
        assert!((dc.cc003_min_similarity - 0.75).abs() < 1e-6);
    }
}
