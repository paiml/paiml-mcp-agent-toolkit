// Quality Gate types and helpers
#[derive(Debug, serde::Serialize)]
/// Quality gate results.
pub struct QualityGateResults {
    pub passed: bool,
    pub total_violations: usize,
    /// How many of `total_violations` actually decided `passed`.
    ///
    /// Advisory (`severity:"info"`) findings are reported but never
    /// verdict-bearing, so `passed:true` can sit beside a non-empty list. The
    /// count that DID decide is stated here rather than left to be inferred —
    /// the same field, with the same meaning, the MCP `quality_gate` tool emits.
    pub blocking_violations: usize,
    pub complexity_violations: usize,
    pub dead_code_violations: usize,
    pub satd_violations: usize,
    pub entropy_violations: usize,
    pub security_violations: usize,
    /// Files whose entire content is byte-identical to another file's.
    ///
    /// This was `identical_files`, and the gate's word "duplicate" read
    /// as the block-level detector `analyze duplicates` runs (21.67 % on this
    /// tree). The gate hashes WHOLE files, so it reported 0 beside that number.
    /// Named for what it measures; the block-level count is a separate item and
    /// is disclosed in `not_measured` under `duplicates` (CRUX-02, #1153).
    pub identical_files: usize,
    pub coverage_violations: usize,
    pub section_violations: usize,
    pub provability_violations: usize,
    /// Files longer than the resolved `max_file_lines` (AD-05).
    pub file_size_violations: usize,
    /// Files with more commits in the last 90 days than the resolved
    /// `max_churn_commits_90d` (AD-05).
    pub churn_violations: usize,
    /// `cargo clippy --all-targets -- -D warnings` findings (AD-05).
    ///
    /// One finding, not one per diagnostic: the question the lint check asks is
    /// "does this tree lint clean", and clippy's own output is the detail.
    pub lint_violations: usize,
    pub provability_score: Option<f64>,
    /// Source files the gate actually looked at.
    ///
    /// Without it, a gate over an EMPTY DIRECTORY and a gate over a clean
    /// project were byte-identical: same JSON, same stderr, both `passed:true`,
    /// both exit 0. "Nothing was wrong" and "nothing was examined" are not the
    /// same claim, and a consumer could not tell them apart.
    pub files_examined: usize,
    /// The population each check measured over, keyed by check name: files
    /// walked, files read, and the rest by reason.
    ///
    /// `files_examined` above is counted independently of any check, by walking
    /// the tree — so it is the population the gate COULD have looked at, and it
    /// catches only the extreme case where that population is zero. It does not
    /// partition against anything and must not be read as a denominator. The
    /// ordinary case, a check that read some files and silently declined others,
    /// had no disclosure at all: `satd_violations: 1` beside `files_examined: 7`
    /// on a tree where the SATD check read one file, refused five, and two of
    /// the five it refused held markers.
    ///
    /// A narrowing of scope now shows up as a change in a disclosed denominator
    /// instead of a silent drop in findings (#1035). Always serialized, empty map
    /// included: an absent key is exactly the ambiguity this removes.
    pub files_not_read:
        std::collections::BTreeMap<String, crate::services::satd_detector::FileCensus>,
    /// The checks that were selected and ran.
    ///
    /// The nine `*_violations` counters below always serialize, so
    /// `--checks complexity` still reported `security_violations: 0` for a
    /// check that never executed — eight zeros that mean "not run" rendered
    /// identically to zeros that mean "clean". This names what ran, so the
    /// difference is recoverable.
    pub checks_run: Vec<String>,
    /// One line per violation, in the same order as the full `violations` array
    /// emitted alongside these results.
    ///
    /// This was left permanently empty while `total_violations` and
    /// `entropy_violations` beside it reported 3 — a count heading a list that
    /// contradicted it. A consumer reading `results.violations` saw nothing.
    pub violations: Vec<String>,
    /// Advertised checks that RAN but could not measure their subject, with
    /// the reason — `dead_code` on a crate `cargo check` cannot compile,
    /// `coverage` whose cache failed a guard, `duplicates` whose block-level
    /// half has no detector behind the gate. Always serialized: an empty list
    /// is the positive claim that every selected check measured what it
    /// claims to (CRUX-02, #1153).
    pub not_measured: Vec<UnmeasuredCheck>,
    /// Advertised checks that do not apply to this path at all — `dead_code`
    /// with no `Cargo.toml` at or above it. Kept apart from `not_measured` so
    /// a non-Rust repository is not permanently amber for a check that has
    /// nothing to measure. Always serialized.
    pub not_applicable: Vec<UnmeasuredCheck>,
}

/// One check the gate could not measure (or could not apply) for one path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnmeasuredCheck {
    /// The check's name, spelled the way `--checks` spells it.
    pub check: String,
    /// The path it was not measured for.
    pub path: String,
    /// Why — names the guard or the failure, never a generic sentence.
    pub reason: String,
}

impl QualityGateResults {
    /// Fill `violations` from the violation list these counts describe, so the
    /// summary object never heads an empty list with a non-zero count.
    pub fn set_violation_lines(&mut self, violations: &[QualityViolation]) {
        self.violations = violations
            .iter()
            .map(|v| {
                let where_ = v
                    .line
                    .map_or_else(|| v.file.clone(), |l| format!("{}:{l}", v.file));
                format!("[{}] {} - {}", v.check_type, where_, v.message)
            })
            .collect();
    }

    /// Recalculate per-category violation counts from the filtered violations list (#196).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn recalculate_from(&mut self, violations: &[QualityViolation]) {
        self.complexity_violations = violations.iter().filter(|v| v.check_type == "complexity").count();
        self.dead_code_violations = violations.iter().filter(|v| v.check_type == "dead_code").count();
        self.satd_violations = violations.iter().filter(|v| v.check_type == "satd").count();
        self.entropy_violations = violations.iter().filter(|v| v.check_type == "entropy").count();
        self.security_violations = violations.iter().filter(|v| v.check_type == "security").count();
        self.identical_files = violations.iter().filter(|v| v.check_type == "duplicates").count();
        self.coverage_violations = violations.iter().filter(|v| v.check_type == "coverage").count();
        self.section_violations = violations.iter().filter(|v| v.check_type == "sections").count();
        self.provability_violations = violations.iter().filter(|v| v.check_type == "provability").count();
        self.file_size_violations = violations.iter().filter(|v| v.check_type == "file-size").count();
        self.churn_violations = violations.iter().filter(|v| v.check_type == "churn").count();
        self.lint_violations = violations.iter().filter(|v| v.check_type == "lint").count();
        self.total_violations = violations.len();
        self.blocking_violations = blocking_violation_count(violations);
        self.set_violation_lines(violations);
    }
}

impl Default for QualityGateResults {
    fn default() -> Self {
        Self {
            passed: true, // Default to passed when no violations
            files_examined: 0,
            files_not_read: std::collections::BTreeMap::new(),
            checks_run: Vec::new(),
            not_measured: Vec::new(),
            not_applicable: Vec::new(),
            total_violations: 0,
            blocking_violations: 0,
            complexity_violations: 0,
            dead_code_violations: 0,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            identical_files: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            file_size_violations: 0,
            churn_violations: 0,
            lint_violations: 0,
            provability_score: None,
            violations: Vec::new(),
        }
    }
}

// Comprehensive analysis types
#[derive(Debug, Default, serde::Serialize)]
struct ComprehensiveReport {
    complexity: Option<ComplexityReport>,
    satd: Option<SatdReport>,
    tdg: Option<TdgReport>,
    dead_code: Option<DeadCodeReport>,
    defects: Option<DefectReport>,
    duplicates: Option<DuplicateReport>,
}

#[derive(Debug, serde::Serialize)]
struct ComplexityReport {
    total_functions: usize,
    high_complexity_count: usize,
    average_complexity: f64,
    p99_complexity: u32,
    hotspots: Vec<ComplexityHotspot>,
}

#[derive(Debug, serde::Serialize)]
struct ComplexityHotspot {
    function: String,
    file: String,
    complexity: u32,
}

#[derive(Debug, serde::Serialize)]
struct SatdReport {
    total_items: usize,
    by_type: HashMap<String, usize>,
    by_severity: HashMap<String, usize>,
    items: Vec<SatdItem>,
}

#[derive(Debug, serde::Serialize)]
struct SatdItem {
    file: String,
    line: usize,
    text: String,
    satd_type: String,
    severity: String,
}

#[derive(Debug, serde::Serialize)]
struct TdgReport {
    average_tdg: f64,
    critical_files: Vec<TdgFile>,
    hotspot_count: usize,
}

#[derive(Debug, serde::Serialize)]
struct TdgFile {
    file: String,
    tdg_score: f64,
    complexity: u32,
    churn: u32,
}

#[derive(Debug, serde::Serialize)]
struct DeadCodeReport {
    total_items: usize,
    dead_code_percentage: f64,
    items: Vec<DeadCodeItem>,
}

#[derive(Debug, serde::Serialize)]
struct DeadCodeItem {
    name: String,
    file: String,
    line: usize,
    item_type: String,
}

#[derive(Debug, serde::Serialize)]
struct DefectReport {
    high_risk_files: Vec<DefectPrediction>,
    total_analyzed: usize,
    high_risk_count: usize,
}

#[derive(Debug, serde::Serialize)]
struct DefectPrediction {
    file: String,
    probability: f64,
    factors: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct DuplicateReport {
    duplicate_blocks: usize,
    duplicate_lines: usize,
    duplicate_percentage: f64,
    blocks: Vec<DuplicateBlock>,
}

#[derive(Debug, serde::Serialize)]
struct DuplicateBlock {
    files: Vec<String>,
    lines: usize,
    tokens: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Violation record for quality.
pub struct QualityViolation {
    pub check_type: String,
    pub severity: String,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
    /// Detailed explanation for explainability (#226, #229).
    /// Contains affected files, example code, and score breakdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<ViolationDetails>,
}

/// Detailed violation context for explainability (#226, #229).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationDetails {
    /// Files affected by this violation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_files: Vec<String>,
    /// Example code snippet showing the pattern
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_code: Option<String>,
    /// Concrete fix suggestion
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_suggestion: Option<String>,
    /// Score factors that contributed to this violation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub score_factors: Vec<String>,
}

impl QualityViolation {
    /// Create a simple violation without details (backwards-compatible).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(
        check_type: impl Into<String>,
        severity: impl Into<String>,
        file: impl Into<String>,
        line: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check_type: check_type.into(),
            severity: severity.into(),
            file: file.into(),
            line,
            message: message.into(),
            details: None,
        }
    }

    /// Attach details for explainability (#226).
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_details(mut self, details: ViolationDetails) -> Self {
        self.details = Some(details);
        self
    }
}

// Helper function to check if file is source code
fn is_source_file(path: &Path) -> bool {
    has_source_extension(path) && !is_excluded_test_path(path) && !is_test_filename(path)
}

/// Extract Method: Check if path has a source code extension
fn has_source_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c")
    )
}

/// Extract Method: Check if path should be excluded (test/example directories)
fn is_excluded_test_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/tests/")
        || path_str.contains("/test/")
        || path_str.contains("/examples/")
        || path_str.contains("/benches/")
        || path_str.contains("/fixtures/")
        || path_str.contains("/testdata/")
        || path_str.contains("/test_data/")
        || path_str.contains("/debug_test/")
        || path_str.contains("/test-")
}

/// Extract Method: Check if filename follows test patterns
fn is_test_filename(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        let fname = file_name.to_string_lossy();
        // Use the same logic as is_excluded_filename for consistency
        is_excluded_filename(&fname)
    } else {
        false
    }
}

#[cfg(test)]
mod unmeasured_shape_tests {
    use super::*;

    /// CRUX-02 leg 3 and the disclosure lists: the payload names what it
    /// measured (`identical_files`), never the word it used to borrow
    /// (`duplicate_violations`), and both disclosure lists serialize even when
    /// empty — an absent key would be the ambiguity these remove.
    #[test]
    fn the_results_payload_carries_the_honest_name_and_both_disclosure_lists() {
        let mut r = QualityGateResults::default();
        r.not_measured.push(UnmeasuredCheck {
            check: "dead_code".into(),
            path: "/x".into(),
            reason: "could not compile: `cargo check` failed (error: e)".into(),
        });
        let v = serde_json::to_value(&r).expect("serialize");
        assert!(v.get("duplicate_violations").is_none(), "the misleading key must be gone");
        assert_eq!(v["identical_files"], 0);
        assert_eq!(v["not_measured"][0]["check"], "dead_code");
        assert!(v["not_applicable"].as_array().is_some_and(Vec::is_empty), "empty list, not absent");
        let empty = serde_json::to_value(QualityGateResults::default()).expect("serialize");
        assert!(empty["not_measured"].as_array().is_some_and(Vec::is_empty));
    }
}
