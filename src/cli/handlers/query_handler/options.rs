#![allow(unused)]
//! Query options, data structures, and argument parsing.

use crate::services::agent_context::{
    CaseSensitivity, QueryOptions, QueryResult, RankBy, SearchMode,
};
use std::collections::HashMap;

// ── ANSI color constants ────────────────────────────────────────────────────
// Re-exported for sibling modules.
//
// These were a second, private copy of `crate::cli::colors` typed as
// `&'static str`, so every `{CYAN}`/`{RESET}` in the query printers emitted its
// escape unconditionally: `pmat query 'error handling' --color never | cat`
// still wrote 20 escape-bearing lines. Round 4 only reached the three special
// modes that go through `tint`, and the document printer. Aliasing the shared
// `Sgr` constants deletes the duplicate outright — the gating is in the value's
// `Display`, so every interpolation site is fixed at once and none of them can
// drift back.

pub(super) use crate::cli::colors::Sgr;

pub(super) const RESET: Sgr = crate::cli::colors::RESET;
pub(super) const BOLD: Sgr = crate::cli::colors::BOLD;
pub(super) const DIM: Sgr = crate::cli::colors::DIM;
pub(super) const UNDERLINE: Sgr = crate::cli::colors::UNDERLINE;
pub(super) const RED: Sgr = crate::cli::colors::RED;
pub(super) const GREEN: Sgr = crate::cli::colors::GREEN;
pub(super) const YELLOW: Sgr = crate::cli::colors::YELLOW;
pub(super) const MAGENTA: Sgr = crate::cli::colors::MAGENTA;
pub(super) const CYAN: Sgr = crate::cli::colors::CYAN;
/// The query printers' "white" has always been the bold variant.
pub(super) const WHITE: Sgr = crate::cli::colors::BOLD_WHITE;
pub(super) const BRIGHT_GREEN: Sgr = crate::cli::colors::BOLD_GREEN;
pub(super) const BRIGHT_RED: Sgr = crate::cli::colors::BOLD_RED;
pub(super) const DIM_CYAN: Sgr = crate::cli::colors::DIM_CYAN;

// ── Data structures ─────────────────────────────────────────────────────────

/// Query performance profile -- Toyota Way Andon cord instrumentation.
///
/// All query phases are timed. If any phase exceeds its threshold,
/// a warning is printed (Andon cord: make problems visible).
pub(super) struct QueryProfile {
    phases: Vec<(&'static str, std::time::Duration)>,
    start: std::time::Instant,
}

/// Andon cord threshold: any single phase exceeding this triggers a warning.
const ANDON_THRESHOLD_MS: u128 = 500;

impl QueryProfile {
    pub(super) fn new() -> Self {
        Self {
            phases: Vec::new(),
            start: std::time::Instant::now(),
        }
    }

    pub(super) fn phase(&mut self, name: &'static str) {
        self.phases.push((name, self.start.elapsed()));
    }

    pub(super) fn emit(&self, quiet: bool) {
        if quiet {
            return;
        }
        let total = self.start.elapsed();
        let mut prev = std::time::Duration::ZERO;
        let mut violations = Vec::new();
        for (name, cumulative) in &self.phases {
            let delta = *cumulative - prev;
            let delta_ms = delta.as_millis();
            if delta_ms > ANDON_THRESHOLD_MS {
                violations.push((*name, delta_ms));
            }
            prev = *cumulative;
        }
        if !violations.is_empty() {
            eprintln!(
                "{DIM}query profile: {:.0}ms total{RESET}",
                total.as_secs_f64() * 1000.0
            );
            for (name, cumulative) in &self.phases {
                let delta = if self.phases.first().map(|f| f.0) == Some(*name) {
                    *cumulative
                } else {
                    let idx = self
                        .phases
                        .iter()
                        .position(|p| p.0 == *name)
                        .expect("phase must exist");
                    *cumulative - self.phases[idx - 1].1
                };
                let delta_ms = delta.as_millis();
                let marker = if delta_ms > ANDON_THRESHOLD_MS {
                    &format!(" {BRIGHT_RED}ANDON{RESET}")
                } else {
                    ""
                };
                eprintln!("  {DIM}{name}: {delta_ms}ms{marker}{RESET}");
            }
        }
    }
}

/// Quality annotations for a file referenced in git history
#[derive(Default, Clone)]
pub(super) struct FileAnnotation {
    pub(super) tdg_grade: Option<String>,
    pub(super) avg_complexity: Option<f32>,
    pub(super) max_pagerank: Option<f32>,
    pub(super) function_count: usize,
    pub(super) dead_code_count: usize,
    pub(super) dead_code_pct: f32,
    pub(super) fault_count: usize,
}

/// Aggregated hotspot info for a file across all commits
#[derive(Default, Clone)]
pub(super) struct FileHotspot {
    pub(super) commit_count: usize,
    pub(super) fix_count: usize,
    pub(super) feat_count: usize,
    pub(super) lines_added: u64,
    pub(super) lines_deleted: u64,
    pub(super) authors: HashMap<String, usize>,
    pub(super) annotation: FileAnnotation,
}

/// Co-change pair
pub(super) struct CoChangePair {
    pub(super) file_a: String,
    pub(super) file_b: String,
    pub(super) count: usize,
    pub(super) jaccard: f32,
}

/// Per-commit enrichment (reserved for JSON output format)
pub(super) struct CommitAnnotation {
    pub(super) work_ticket: Option<WorkTicketInfo>,
    pub(super) commit_quality: Option<CommitQualityMeta>,
    pub(super) decay_score: f32,
    pub(super) impact_risk: f32,
}

/// Work ticket cross-reference
pub(super) struct WorkTicketInfo {
    pub(super) ticket_id: String,
    pub(super) claims_passed: usize,
    pub(super) claims_total: usize,

    pub(super) baseline_tdg: f64,
}

/// Quality metadata from .pmat-metrics/commit-*-meta.json
#[derive(serde::Deserialize)]
pub(super) struct CommitQualityMeta {
    #[serde(default)]
    pub(super) work_item_id: String,
    #[serde(default)]
    pub(super) tdg_score: f64,
    #[serde(default)]
    pub(super) repo_score: f64,
    #[serde(default)]
    pub(super) rust_project_score: Option<f64>,
}

/// Dead code cache entry
#[derive(serde::Deserialize)]
pub(super) struct DeadCodeCache {
    #[serde(default)]
    pub(super) report: DeadCodeReport,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct DeadCodeReport {
    #[serde(default)]
    pub(super) files_with_dead_code: Vec<DeadCodeFile>,
}

#[derive(serde::Deserialize)]
pub(super) struct DeadCodeFile {
    pub(super) file_path: String,
    #[serde(default)]
    pub(super) dead_items: Vec<serde_json::Value>,
    #[serde(default)]
    pub(super) file_dead_percentage: f32,
}

/// Bug hunter cache entry
#[derive(serde::Deserialize)]
pub(super) struct BugHunterCache {
    #[serde(default)]
    pub(super) findings: Vec<BugHunterFinding>,
}

#[derive(serde::Deserialize)]
pub(super) struct BugHunterFinding {
    #[serde(default)]
    pub(super) file: String,
    #[serde(default)]
    pub(super) severity: String,
    #[serde(default)]
    pub(super) suspiciousness: f32,
}

/// Context for raw+indexed merge operations.
pub(super) struct MergeContext<'a> {
    pub(super) query: &'a str,
    pub(super) literal: bool,
    pub(super) ignore_case: bool,
    pub(super) language: &'a Option<String>,
    pub(super) exclude_file: &'a [String],
    pub(super) exclude: &'a [String],
    pub(super) project_path: &'a std::path::Path,
    pub(super) is_regex_or_literal: bool,
    pub(super) exclude_tests: bool,
}

pub(super) type GitData = Option<(
    Vec<crate::services::git_history::GitSearchResult>,
    Vec<crate::services::git_history::CommitInfo>,
)>;

// ── Option building ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(super) fn build_query_options(
    limit: usize,
    min_grade: Option<String>,
    max_complexity: Option<u32>,
    language: Option<String>,
    path_pattern: Option<String>,
    include_source: bool,
    rank_by: &Option<String>,
    min_pagerank: Option<f32>,
    regex: bool,
    literal: bool,
    case_sensitive: bool,
    ignore_case: bool,
    exclude: Vec<String>,
    exclude_file: Vec<String>,
) -> QueryOptions {
    let rank_by_enum = rank_by
        .as_ref()
        .map(|s| s.parse::<RankBy>().unwrap_or_default())
        .unwrap_or_default();
    let search_mode = if regex {
        SearchMode::Regex
    } else if literal {
        SearchMode::Literal
    } else {
        SearchMode::Semantic
    };
    let case_sensitivity = if case_sensitive {
        CaseSensitivity::Sensitive
    } else if ignore_case {
        CaseSensitivity::Insensitive
    } else {
        CaseSensitivity::Smart
    };
    QueryOptions {
        limit,
        min_grade,
        max_complexity,
        max_loc: None,
        language,
        path_pattern,
        include_source,
        rank_by: rank_by_enum,
        min_pagerank,
        search_mode,
        case_sensitivity,
        exclude_pattern: exclude,
        exclude_file_pattern: exclude_file,
    }
}

// ── Filters ─────────────────────────────────────────────────────────────────

/// Check if a result looks like a test function.
///
/// Mirrors `is_test_chunk` (the index-build filter): besides `test_`-prefixed
/// names and `tests/` dirs, it catches mid-filename variants like
/// `*_tests_basic.rs` (commonly `include!()`-ed into a #[cfg(test)] module, so
/// they lack a standalone test attribute) and test-helper prefixes.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(super) fn is_test_function(r: &QueryResult) -> bool {
    r.function_name.starts_with("test_")
        || r.function_name.starts_with("setup_test")
        || r.function_name.starts_with("create_test")
        || is_test_path(&r.file_path)
}

/// Path-based test-file heuristic, shared by `is_test_function` and the raw
/// (literal/regex) search post-filter. Catches `tests/` dirs, `_test.rs`, and
/// mid-filename `_tests_basic.rs` / `_test_helpers.rs` (`include!()`-ed into a
/// #[cfg(test)] module, so they have no standalone test attribute).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(super) fn is_test_path(file_path: &str) -> bool {
    file_path.starts_with("tests/")
        || file_path.starts_with("test/")
        || file_path.starts_with("benches/")
        || file_path.starts_with("examples/")
        || file_path.contains("/tests/")
        || file_path.contains("/test/")
        || file_path.contains("/benches/")
        || file_path.contains("/examples/")
        || file_path.contains("/testdata/")
        || file_path.contains("/test_data/")
        || file_path.contains("_tests") // _tests.rs, _tests_basic.rs, …
        || file_path.contains("_test.")
        || file_path.contains("_test_")
        || file_path.contains("fixtures") // test-support fixtures (e.g. *_coverage_fixtures.rs)
}

/// Normalize a definition type filter string to the canonical form
/// Canonical `definition_type` for a `--type` value.
///
/// Delegates to the one mapping in `agent_context::query::types`; the CLI's
/// `--type` value_parser validates against the same function, so an unknown
/// value is rejected at parse time and can never reach here.
pub(super) fn normalize_definition_type(def_type: &str) -> String {
    crate::services::agent_context::normalize_definition_type(def_type)
        .unwrap_or_else(|| def_type.to_lowercase())
}

/// Apply result filters: exclude-tests and definition-type
pub(super) fn apply_result_filters(
    results: &mut Vec<QueryResult>,
    exclude_tests: bool,
    definition_type: &Option<String>,
) {
    if exclude_tests {
        results.retain(|r| !is_test_function(r));
    }
    if let Some(ref def_type) = definition_type {
        let filter_type = normalize_definition_type(def_type);
        results.retain(|r| r.definition_type == filter_type);
    }
}

/// Apply filters for coverage-gaps mode (language, path, exclude-tests)
pub(super) fn apply_result_filters_coverage(
    results: &mut Vec<QueryResult>,
    language: &Option<String>,
    path_pattern: &Option<String>,
    exclude_tests: bool,
) {
    // Always exclude test fixture directories (not part of the project)
    results.retain(|r| !is_test_fixture_path(&r.file_path));

    if let Some(ref lang) = language {
        let lang_lower = lang.to_lowercase();
        results.retain(|r| r.language.to_lowercase() == lang_lower);
    }
    if let Some(ref pattern) = path_pattern {
        results.retain(|r| r.file_path.contains(pattern));
    }
    if exclude_tests {
        results.retain(|r| !is_test_function(r));
    }
}

/// Check if a file path belongs to a test fixture directory (not real project code).
fn is_test_fixture_path(path: &str) -> bool {
    path.contains("comprehensive_language_test/")
        || path.contains("fixtures/")
        || path.contains("test_fixtures/")
        || path.contains("testdata/")
        || path.contains("test_enhanced_naming/")
}

/// Apply post-enrichment re-sort for Impact ranking
pub(super) fn apply_post_enrichment_sort(results: &mut [QueryResult], rank_by: &Option<String>) {
    if let Some(ref rank_str) = rank_by {
        let r = rank_str.to_lowercase();
        if r == "impact" || r == "roi" || r == "coverage" {
            results.sort_by(|a, b| {
                b.impact_score
                    .partial_cmp(&a.impact_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else if r == "cross-project" || r == "crossproject" || r == "xproject" {
            // Secondary sort: boost by cross_project_callers (already set by engine)
            results.sort_by(|a, b| {
                let score_a = a.pagerank * (1.0 + 0.5 * a.cross_project_callers as f32);
                let score_b = b.pagerank * (1.0 + 0.5 * b.cross_project_callers as f32);
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}

#[cfg(test)]
mod is_test_path_tests {
    use super::is_test_path;

    #[test]
    fn excludes_standard_test_paths() {
        assert!(is_test_path("tests/foo.rs"));
        assert!(is_test_path("src/services/foo/tests/bar.rs"));
        assert!(is_test_path("src/foo_test.rs"));
        assert!(is_test_path("src/foo_tests.rs"));
    }

    #[test]
    fn excludes_include_macro_test_fragments() {
        // The original --exclude-tests leak: include!()-ed test fragments whose
        // name has `_tests_` mid-filename (no standalone #[cfg(test)]).
        assert!(is_test_path(
            "src/ast/polyglot/language_mapper_tests_basic.rs"
        ));
        assert!(is_test_path("src/foo/bar_test_helpers.rs"));
    }

    #[test]
    fn excludes_fixture_support_files() {
        assert!(is_test_path(
            "src/services/context_graph_coverage_fixtures.rs"
        ));
        assert!(is_test_path("fixtures/typescript/calculator.ts"));
    }

    #[test]
    fn keeps_production_source() {
        assert!(!is_test_path("src/lib.rs"));
        assert!(!is_test_path("src/services/agent_context/query/options.rs"));
        assert!(!is_test_path("src/cli/handlers/query_handler.rs"));
        // `latest.rs` must not trip the `_test`/`_tests` checks.
        assert!(!is_test_path("src/services/latest.rs"));
    }
}
