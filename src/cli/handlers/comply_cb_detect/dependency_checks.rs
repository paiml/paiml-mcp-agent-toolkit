#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// =============================================================================
// CB-081: Dependency Count Detection (Enhanced v2.9)
// Per rust-project-score spec: Too many dependencies degrades build times,
// increases supply chain risk, and bloats binaries.
//
// Enhancements:
// - CB-081-A: Base dependency count scoring
// - CB-081-B: Duplicate crate detection
// - CB-081-C: Feature flag hygiene analysis
// - CB-081-D: Sovereign stack bonus
// - CB-081-E: Trend tracking
// =============================================================================

/// Sovereign stack crates (batuta ecosystem).
/// Includes deprecated names (renacer, trueno-*) and aprender monorepo names.
const SOVEREIGN_CRATES: &[&str] = &[
    "aprender",
    "aprender-compute",
    "aprender-contracts",
    "aprender-contracts-macros",
    "aprender-profile",
    "aprender-rag",
    "aprender-simulate",
    "trueno",
    "trueno-graph",
    "trueno-db",
    "trueno-rag",
    "trueno-viz",
    "trueno-zram-core",
    "pmcp",
    "presentar-core",
    "renacer",
    "simular",
    "certeza",
    "bashrs",
    "probar",
    "ruchy",
    "rmedia",
    "whisper-apr",
];

/// Dependency count analysis result (enhanced)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyCountReport {
    pub direct_count: usize,
    /// Total packages in Cargo.lock (includes dev-dep transitive)
    pub transitive_count: usize,
    /// Production-only transitive count (excludes dev-dep transitive), used for scoring
    pub prod_transitive_count: Option<usize>,
    pub score: u8, // 0-5 points based on rust-project-score thresholds
    /// Crates with multiple versions in Cargo.lock
    pub duplicate_crates: Vec<DuplicateCrate>,
    /// Dependencies using default-features = false
    pub feature_gated_count: usize,
    pub feature_gated_pct: f64,
    /// Sovereign stack crates used
    pub sovereign_crates: Vec<String>,
    pub sovereign_bonus: u8, // 0-3 bonus points
    /// Delta from previous check (if available)
    pub trend: Option<DependencyTrend>,
    pub violations: Vec<CbPatternViolation>,
}

/// Duplicate crate info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateCrate {
    pub name: String,
    pub versions: Vec<String>,
}

/// Trend tracking data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyTrend {
    pub direct_delta: i32,
    pub transitive_delta: i32,
    pub previous_timestamp: String,
}

/// CB-081 Cache for O(1) dependency analysis (issue #148 fix)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct DependencyCache {
    /// Cargo.lock file mtime (unix timestamp)
    cargo_lock_mtime: u64,
    /// Cached transitive count (all packages from Cargo.lock)
    transitive_count: usize,
    /// Cached production-only transitive count (via cargo tree --no-dev)
    #[serde(default)]
    prod_transitive_count: Option<usize>,
    /// Cached duplicate crates
    duplicate_crates: Vec<DuplicateCrate>,
}

// =============================================================================
// CB-130: Agent Context Adoption (PMAT-470)
// =============================================================================

/// CB-130 Agent Context Adoption report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContextReport {
    /// Whether the RAG index exists
    pub index_exists: bool,
    /// Index age in hours (None if no index)
    pub index_age_hours: Option<f64>,
    /// Whether index is considered stale (>24h)
    pub index_stale: bool,
    /// Number of functions indexed
    pub function_count: usize,
    /// Whether CLAUDE.md mentions pmat_query_code
    pub claude_md_configured: bool,
    /// Required patterns missing from CLAUDE.md
    pub missing_required_patterns: Vec<String>,
    /// Forbidden patterns found in CLAUDE.md
    pub forbidden_patterns_found: Vec<ForbiddenPatternMatch>,
}

/// A forbidden pattern match with location info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPatternMatch {
    /// The pattern that was found
    pub pattern: String,
    /// Line number where it was found
    pub line: usize,
    /// The actual line content (truncated)
    pub context: String,
}

// --- Include implementation files ---
include!("dependency_checks_cache.rs");
include!("dependency_checks_analysis.rs");
include!("dependency_checks_agent_context.rs");

#[cfg(test)]
mod analysis_tests {
    //! Wave 36 PR5 — pure-helper coverage for dependency_checks_analysis.rs.
    //! Filesystem-bound helpers (analyze_cargo_toml, save/load_dependency_*,
    //! load_dependency_max_transitive, detect_cb081_dependency_count) are
    //! deliberately not exercised here.
    use super::*;

    // ── is_dependency_section ───────────────────────────────────────────────

    #[test]
    fn test_is_dependency_section_dependencies_exact() {
        assert_eq!(
            is_dependency_section("[dependencies]"),
            (true, false, false)
        );
    }

    #[test]
    fn test_is_dependency_section_dependencies_table_prefix() {
        // PIN: `[dependencies.foo]` per-crate tables count as in_dependencies.
        assert_eq!(
            is_dependency_section("[dependencies.serde]"),
            (true, false, false)
        );
    }

    #[test]
    fn test_is_dependency_section_target_prefix_counts_as_dependencies() {
        // PIN: `[target.'cfg(unix)'.dependencies]` is treated as deps too.
        assert_eq!(
            is_dependency_section("[target.'cfg(unix)'.dependencies]"),
            (true, false, false)
        );
    }

    #[test]
    fn test_is_dependency_section_workspace_dependencies() {
        assert_eq!(
            is_dependency_section("[workspace.dependencies]"),
            (true, false, false)
        );
    }

    #[test]
    fn test_is_dependency_section_dev_dependencies() {
        assert_eq!(
            is_dependency_section("[dev-dependencies]"),
            (false, true, false)
        );
    }

    #[test]
    fn test_is_dependency_section_dev_table_prefix() {
        assert_eq!(
            is_dependency_section("[dev-dependencies.serde]"),
            (false, true, false)
        );
    }

    #[test]
    fn test_is_dependency_section_build_dependencies() {
        assert_eq!(
            is_dependency_section("[build-dependencies]"),
            (false, false, true)
        );
    }

    #[test]
    fn test_is_dependency_section_build_table_prefix() {
        assert_eq!(
            is_dependency_section("[build-dependencies.bindgen]"),
            (false, false, true)
        );
    }

    #[test]
    fn test_is_dependency_section_unrelated_returns_all_false() {
        assert_eq!(is_dependency_section("[package]"), (false, false, false));
        assert_eq!(is_dependency_section("[features]"), (false, false, false));
        assert_eq!(is_dependency_section("[lib]"), (false, false, false));
    }

    // ── is_scoreable_dependency ─────────────────────────────────────────────

    #[test]
    fn test_is_scoreable_dependency_in_deps_with_eq() {
        assert!(is_scoreable_dependency(true, false, false, "serde = \"1\""));
    }

    #[test]
    fn test_is_scoreable_dependency_in_dev_rejected() {
        assert!(!is_scoreable_dependency(true, true, false, "serde = \"1\""));
    }

    #[test]
    fn test_is_scoreable_dependency_in_build_rejected() {
        assert!(!is_scoreable_dependency(true, false, true, "serde = \"1\""));
    }

    #[test]
    fn test_is_scoreable_dependency_no_eq_rejected() {
        // No `=` means it's not a key=value dep line (e.g. blank or section header).
        assert!(!is_scoreable_dependency(true, false, false, "serde"));
    }

    #[test]
    fn test_is_scoreable_dependency_comment_rejected() {
        assert!(!is_scoreable_dependency(
            true,
            false,
            false,
            "# serde = \"1\""
        ));
    }

    #[test]
    fn test_is_scoreable_dependency_outside_deps_rejected() {
        assert!(!is_scoreable_dependency(
            false,
            false,
            false,
            "serde = \"1\""
        ));
    }

    // ── process_dependency_line ─────────────────────────────────────────────

    #[test]
    fn test_process_dependency_line_simple_is_direct_not_gated() {
        let mut sov = Vec::new();
        let (direct, gated) = process_dependency_line("serde = \"1\"", &mut sov);
        assert!(direct);
        assert!(!gated);
        assert!(sov.is_empty());
    }

    #[test]
    fn test_process_dependency_line_optional_is_not_direct() {
        let mut sov = Vec::new();
        let (direct, _) = process_dependency_line(
            "lazy_static = { version = \"1\", optional = true }",
            &mut sov,
        );
        assert!(!direct, "optional=true should mark not-direct");
    }

    #[test]
    fn test_process_dependency_line_default_features_false_is_gated() {
        let mut sov = Vec::new();
        let (_, gated) = process_dependency_line(
            "serde = { version = \"1\", default-features = false }",
            &mut sov,
        );
        assert!(gated);
    }

    #[test]
    fn test_process_dependency_line_pin_optional_keyword_substring_marks_not_direct() {
        // PIN: detection uses substring match for both "optional" and "true".
        // Any string containing both substrings (even in unrelated positions)
        // will flip is_direct to false. Pinned to prevent silent regression.
        let mut sov = Vec::new();
        let (direct, _) = process_dependency_line("x = \"optional yet true gibberish\"", &mut sov);
        assert!(!direct, "substring match treats this as optional=true");
    }

    #[test]
    fn test_process_dependency_line_sovereign_crate_appended() {
        // SOVEREIGN_CRATES contains aprender — its detection requires
        // the line to start with the crate name and be followed by ' ' or '='.
        let mut sov = Vec::new();
        let _ = process_dependency_line("aprender = \"0.1\"", &mut sov);
        assert!(sov.contains(&"aprender".to_string()));
    }

    #[test]
    fn test_process_dependency_line_sovereign_crate_not_at_start_skipped() {
        // PIN: detection requires `starts_with(crate_name)`. A different prefix
        // (e.g. `apr-aprender`) does NOT match even though aprender is a substring.
        let mut sov = Vec::new();
        let _ = process_dependency_line("apr-aprender = \"0.1\"", &mut sov);
        assert!(sov.is_empty());
    }

    // ── build_threshold_violation ───────────────────────────────────────────

    #[test]
    fn test_build_threshold_violation_within_thresholds_returns_none() {
        let v = build_threshold_violation(
            "Cargo.toml",
            10,  // direct
            50,  // transitive
            50,  // direct_max
            250, // trans_max
            Severity::Error,
        );
        assert!(v.is_none());
    }

    #[test]
    fn test_build_threshold_violation_direct_exceeded_error() {
        let v = build_threshold_violation("Cargo.toml", 60, 50, 50, 250, Severity::Error).unwrap();
        assert_eq!(v.pattern_id, "CB-081-A");
        assert!(v.description.contains("60 direct deps exceed max 50"));
        assert!(v.description.contains("50 transitive OK"));
        assert!(matches!(v.severity, Severity::Error));
    }

    #[test]
    fn test_build_threshold_violation_transitive_exceeded_warning_uses_threshold_phrasing() {
        // PIN: Warning severity uses "(threshold X)" phrasing; Error uses "exceed max X".
        let v =
            build_threshold_violation("Cargo.toml", 10, 300, 40, 200, Severity::Warning).unwrap();
        assert!(v
            .description
            .contains("300 prod transitive deps (threshold 200)"));
        assert!(v.description.contains("10 direct OK"));
        assert!(matches!(v.severity, Severity::Warning));
    }

    #[test]
    fn test_build_threshold_violation_both_exceeded_no_ok_parts_no_parens() {
        let v = build_threshold_violation("Cargo.toml", 60, 300, 50, 250, Severity::Error).unwrap();
        // Both exceed → ok_parts is empty → no parenthetical wrapping.
        assert!(!v.description.contains("("));
        assert!(v.description.contains("60 direct deps exceed max 50"));
        assert!(v
            .description
            .contains("300 prod transitive deps exceed max 250"));
    }

    #[test]
    fn test_build_threshold_violation_carries_file_and_zero_line() {
        let v =
            build_threshold_violation("/abs/Cargo.toml", 60, 50, 50, 250, Severity::Error).unwrap();
        assert_eq!(v.file, "/abs/Cargo.toml");
        assert_eq!(v.line, 0);
    }

    // ── check_duplicate_crates_violation ────────────────────────────────────

    #[test]
    fn test_check_duplicate_crates_violation_empty_appends_nothing() {
        let mut violations = Vec::new();
        check_duplicate_crates_violation("Cargo.lock", &[], &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_duplicate_crates_violation_lists_names() {
        let dups = vec![
            DuplicateCrate {
                name: "a".into(),
                versions: vec!["1".into(), "2".into()],
            },
            DuplicateCrate {
                name: "b".into(),
                versions: vec!["1".into(), "2".into()],
            },
        ];
        let mut violations = Vec::new();
        check_duplicate_crates_violation("Cargo.lock", &dups, &mut violations);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-081-B");
        assert!(violations[0].description.contains("2 duplicate crates"));
        assert!(violations[0].description.contains("a, b"));
        assert!(matches!(violations[0].severity, Severity::Warning));
    }

    // ── check_trend_regression_violation ────────────────────────────────────

    #[test]
    fn test_check_trend_regression_no_trend_no_violation() {
        let mut violations = Vec::new();
        check_trend_regression_violation("Cargo.toml", &None, 100, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_trend_regression_no_increase_no_violation() {
        // delta == 0 → pct stays 0.0, no violation.
        let trend = Some(DependencyTrend {
            direct_delta: 0,
            transitive_delta: 0,
            previous_timestamp: "2026-04-01".into(),
        });
        let mut violations = Vec::new();
        check_trend_regression_violation("Cargo.toml", &trend, 100, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_trend_regression_small_increase_no_violation() {
        // 100 transitive, +5 delta → 5/(100-5) = 5.26% — under the 10% gate.
        let trend = Some(DependencyTrend {
            direct_delta: 0,
            transitive_delta: 5,
            previous_timestamp: "2026-04-01".into(),
        });
        let mut violations = Vec::new();
        check_trend_regression_violation("Cargo.toml", &trend, 100, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_trend_regression_large_increase_emits_warning() {
        // 100 transitive, +20 delta → 20/(100-20) = 25% — crosses the 10% gate.
        let trend = Some(DependencyTrend {
            direct_delta: 0,
            transitive_delta: 20,
            previous_timestamp: "2026-04-01T00:00:00Z".into(),
        });
        let mut violations = Vec::new();
        check_trend_regression_violation("Cargo.toml", &trend, 100, &mut violations);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-081-E");
        assert!(violations[0].description.contains("Dependency creep"));
        assert!(violations[0].description.contains("+20 transitive deps"));
        assert!(matches!(violations[0].severity, Severity::Warning));
    }

    #[test]
    fn test_check_trend_regression_pin_negative_delta_branch_unreachable() {
        // PIN: `if t.transitive_delta > 0` gates the math — a NEGATIVE delta
        // (deps removed) silently bypasses the regression check, even though
        // the "else 0.0" branch in the function exists. Pinned so refactors
        // don't mistakenly assume removed deps are also tracked here.
        let trend = Some(DependencyTrend {
            direct_delta: 0,
            transitive_delta: -50,
            previous_timestamp: "2026-04-01".into(),
        });
        let mut violations = Vec::new();
        check_trend_regression_violation("Cargo.toml", &trend, 100, &mut violations);
        assert!(violations.is_empty());
    }
}
