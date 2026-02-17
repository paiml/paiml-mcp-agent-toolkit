#![cfg_attr(coverage_nightly, coverage(off))]

//! Coverage exclusion classification for --coverage-gaps filtering.
//!
//! Detects WHY a function appears as a coverage gap, so `--coverage-gaps`
//! can filter out intentionally excluded functions (coverage(off), Makefile
//! COVERAGE_EXCLUDE patterns, dead code) and only show genuinely testable gaps.

use super::types::QueryResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Why a function is excluded from coverage tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CoverageExclusion {
    /// Testable — genuine coverage gap
    #[default]
    None,
    /// File has `#![cfg_attr(coverage_nightly, coverage(off))]` at module level
    CoverageOff,
    /// File matches Makefile `COVERAGE_EXCLUDE` regex pattern
    MakefileExcluded,
    /// Function is in dead-code-cache.json
    DeadCode,
}

impl CoverageExclusion {
    pub fn is_none(&self) -> bool {
        matches!(self, CoverageExclusion::None)
    }

    pub fn label(&self) -> &'static str {
        match self {
            CoverageExclusion::None => "testable",
            CoverageExclusion::CoverageOff => "coverage(off)",
            CoverageExclusion::DeadCode => "dead code",
            CoverageExclusion::MakefileExcluded => "Makefile pattern",
        }
    }
}

/// Parsed exclusion context: caches file-level checks so we don't re-read files.
pub(crate) struct ExclusionContext {
    /// Files that have module-level `coverage(off)` annotation
    coverage_off_files: HashSet<String>,
    /// Files already checked for coverage(off) — prevents redundant re-reads
    /// for files that do NOT have the annotation (negative cache).
    checked_files: HashSet<String>,
    /// Compiled regex from Makefile COVERAGE_EXCLUDE (if found)
    makefile_regex: Option<regex::Regex>,
    /// Dead function keys: "file_path::function_name"
    dead_functions: HashSet<String>,
    /// Whether coverage_off_files was pre-populated from index cache (skip file I/O)
    use_cached: bool,
}

impl ExclusionContext {
    /// Build exclusion context from project state.
    ///
    /// When `cached_coverage_off` is provided (from SQLite index), skips all file I/O
    /// for coverage(off) detection — O(1) per file instead of O(file_size).
    pub(crate) fn build(
        project_path: &Path,
        cached_coverage_off: Option<&HashSet<String>>,
    ) -> Self {
        let makefile_regex = parse_makefile_coverage_exclude(project_path);
        let dead_functions = load_dead_code_functions(project_path);
        let coverage_off_files = cached_coverage_off.cloned().unwrap_or_default();
        let use_cached = cached_coverage_off.is_some();
        Self {
            coverage_off_files,
            checked_files: HashSet::new(),
            makefile_regex,
            dead_functions,
            use_cached,
        }
    }

    /// Classify a single result's exclusion reason.
    ///
    /// Checks in priority order: dead code > coverage(off) > Makefile pattern.
    /// With cached data, this is pure HashSet lookups — no file I/O.
    pub(crate) fn classify(
        &mut self,
        result: &QueryResult,
        project_path: &Path,
    ) -> CoverageExclusion {
        // 1. Dead code check (function-level, highest signal)
        let dead_key = format!("{}::{}", result.file_path, result.function_name);
        if self.dead_functions.contains(&dead_key) {
            return CoverageExclusion::DeadCode;
        }

        // 2. Module-level coverage(off) check
        if self.is_coverage_off_file(&result.file_path, project_path) {
            return CoverageExclusion::CoverageOff;
        }

        // 3. Makefile COVERAGE_EXCLUDE pattern
        if let Some(ref re) = self.makefile_regex {
            if re.is_match(&result.file_path) {
                return CoverageExclusion::MakefileExcluded;
            }
        }

        CoverageExclusion::None
    }

    /// Check if a file has module-level `cfg_attr(coverage_nightly, coverage(off))`.
    ///
    /// With cached data (from index build), this is a pure HashSet lookup.
    /// Falls back to lazy file I/O only when no cached data is available.
    fn is_coverage_off_file(&mut self, file_path: &str, project_path: &Path) -> bool {
        if self.coverage_off_files.contains(file_path) {
            return true;
        }
        // If we have cached data, trust it — no need for file I/O fallback
        if self.use_cached {
            return false;
        }
        // Negative cache: already checked this file and it didn't have coverage(off)
        if self.checked_files.contains(file_path) {
            return false;
        }

        self.checked_files.insert(file_path.to_string());

        let full_path = project_path.join(file_path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            // Check first 50 lines for module-level coverage(off)
            let has_coverage_off = content.lines().take(50).any(|line| {
                let trimmed = line.trim();
                trimmed.contains("cfg_attr(coverage_nightly, coverage(off))")
                    || trimmed.contains("cfg_attr(coverage_nightly,coverage(off))")
            });
            if has_coverage_off {
                self.coverage_off_files.insert(file_path.to_string());
                return true;
            }
        }

        false
    }
}

/// Classify coverage exclusions for a batch of results.
///
/// When `cached_coverage_off` is provided (from index build), coverage(off)
/// detection is O(1) HashSet lookup with zero file I/O.
/// Mutates results in-place, setting `coverage_exclusion` and `coverage_excluded`.
pub fn classify_exclusions(
    results: &mut [QueryResult],
    project_path: &Path,
    cached_coverage_off: Option<&HashSet<String>>,
) {
    let mut ctx = ExclusionContext::build(project_path, cached_coverage_off);
    for result in results.iter_mut() {
        let exclusion = ctx.classify(result, project_path);
        result.coverage_excluded = !exclusion.is_none();
        result.coverage_exclusion = exclusion;
    }
}

/// Summary of excluded function counts by category.
#[derive(Default)]
pub struct ExclusionSummary {
    pub coverage_off_count: usize,
    pub coverage_off_files: usize,
    pub dead_code_count: usize,
    pub dead_code_files: usize,
    pub makefile_count: usize,
    pub makefile_files: usize,
}

impl ExclusionSummary {
    pub fn from_results(excluded: &[&QueryResult]) -> Self {
        let mut summary = Self::default();
        let mut cov_off_files: HashSet<&str> = HashSet::new();
        let mut dead_files: HashSet<&str> = HashSet::new();
        let mut make_files: HashSet<&str> = HashSet::new();

        for r in excluded {
            match r.coverage_exclusion {
                CoverageExclusion::CoverageOff => {
                    summary.coverage_off_count += 1;
                    cov_off_files.insert(&r.file_path);
                }
                CoverageExclusion::DeadCode => {
                    summary.dead_code_count += 1;
                    dead_files.insert(&r.file_path);
                }
                CoverageExclusion::MakefileExcluded => {
                    summary.makefile_count += 1;
                    make_files.insert(&r.file_path);
                }
                CoverageExclusion::None => {}
            }
        }
        summary.coverage_off_files = cov_off_files.len();
        summary.dead_code_files = dead_files.len();
        summary.makefile_files = make_files.len();
        summary
    }

    pub fn total(&self) -> usize {
        self.coverage_off_count + self.dead_code_count + self.makefile_count
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Parse the `COVERAGE_EXCLUDE` regex from the project Makefile.
///
/// Looks for `--ignore-filename-regex='...'` pattern and extracts the
/// inner regex, converting it from a filename regex to a path-matching regex.
fn parse_makefile_coverage_exclude(project_path: &Path) -> Option<regex::Regex> {
    let makefile_path = project_path.join("Makefile");
    let content = std::fs::read_to_string(makefile_path).ok()?;

    for line in content.lines() {
        if !line.contains("COVERAGE_EXCLUDE") || !line.contains("--ignore-filename-regex") {
            continue;
        }
        // Extract regex between single quotes after --ignore-filename-regex=
        if let Some(start) = line.find("--ignore-filename-regex='") {
            let after = line
                .get(start + "--ignore-filename-regex='".len()..)
                .unwrap_or_default();
            if let Some(end) = after.find('\'') {
                let raw_pattern = after.get(..end).unwrap_or_default();
                // Normalize escaping: Makefile uses `\\.` (backslash-backslash-dot) which
                // cargo-llvm-cov interprets as literal dot. But Rust regex sees `\\` as
                // literal backslash + `.` as any char. Replace `\\.` with `\.` so Rust
                // regex correctly matches literal dots in file paths.
                let pattern = raw_pattern.replace("\\\\.", "\\.");
                return regex::Regex::new(&pattern).ok();
            }
        }
    }
    None
}

/// Extract dead item keys from a single file entry in the dead-code cache.
fn collect_dead_items(file_entry: &serde_json::Value, dead: &mut HashSet<String>) {
    let file_path = match file_entry.get("file_path").and_then(|p| p.as_str()) {
        Some(p) => p,
        None => return,
    };
    let items = file_entry.get("dead_items").and_then(|d| d.as_array());
    for item in items.into_iter().flatten() {
        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
            dead.insert(format!("{}::{}", file_path, name));
        }
    }
}

/// Load dead function names from `.pmat/dead-code-cache.json`.
///
/// Returns a set of "file_path::function_name" keys for O(1) lookup.
fn load_dead_code_functions(project_path: &Path) -> HashSet<String> {
    let cache_path = project_path.join(".pmat/dead-code-cache.json");
    let mut dead = HashSet::new();

    let data = match std::fs::read_to_string(cache_path) {
        Ok(d) => d,
        Err(_) => return dead,
    };
    let value: serde_json::Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return dead,
    };

    let files = value
        .get("report")
        .and_then(|r| r.get("files_with_dead_code"))
        .and_then(|f| f.as_array());

    for file_entry in files.into_iter().flatten() {
        collect_dead_items(file_entry, &mut dead);
    }

    dead
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(file_path: &str, function_name: &str) -> QueryResult {
        QueryResult {
            file_path: file_path.to_string(),
            function_name: function_name.to_string(),
            signature: format!("fn {}()", function_name),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            tdg_score: 5.0,
            tdg_grade: "C".to_string(),
            complexity: 5,
            big_o: "O(n)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.0,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 10,
            missed_lines: 10,
            impact_score: 0.0,
            coverage_status: "uncovered".to_string(),
            coverage_diff: 0.0,
            coverage_exclusion: CoverageExclusion::None,
            coverage_excluded: false,
            cross_project_callers: 0,
        }
    }

    #[test]
    fn test_coverage_exclusion_default() {
        let e = CoverageExclusion::default();
        assert_eq!(e, CoverageExclusion::None);
        assert!(e.is_none());
    }

    #[test]
    fn test_coverage_exclusion_labels() {
        assert_eq!(CoverageExclusion::None.label(), "testable");
        assert_eq!(CoverageExclusion::CoverageOff.label(), "coverage(off)");
        assert_eq!(CoverageExclusion::DeadCode.label(), "dead code");
        assert_eq!(
            CoverageExclusion::MakefileExcluded.label(),
            "Makefile pattern"
        );
    }

    #[test]
    fn test_coverage_exclusion_is_none() {
        assert!(CoverageExclusion::None.is_none());
        assert!(!CoverageExclusion::CoverageOff.is_none());
        assert!(!CoverageExclusion::DeadCode.is_none());
        assert!(!CoverageExclusion::MakefileExcluded.is_none());
    }

    #[test]
    fn test_classify_coverage_off_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let src_dir = temp.path().join("src/cli");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("handler.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn foo() {}\n",
        )
        .unwrap();

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        let r = make_result("src/cli/handler.rs", "foo");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::CoverageOff);
    }

    #[test]
    fn test_classify_dead_code() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/lib.rs"), "fn dead_fn() {}\n").unwrap();

        let mut dead = HashSet::new();
        dead.insert("src/lib.rs::dead_fn".to_string());

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: dead,
            use_cached: false,
        };

        let r = make_result("src/lib.rs", "dead_fn");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::DeadCode);
    }

    #[test]
    fn test_classify_makefile_excluded() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src/cli/handlers")).unwrap();
        std::fs::write(temp.path().join("src/cli/handlers/foo.rs"), "fn bar() {}\n").unwrap();

        let re = regex::Regex::new(r"/(cli|mcp[^/]*)/").unwrap();
        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: Some(re),
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        let r = make_result("src/cli/handlers/foo.rs", "bar");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::MakefileExcluded);
    }

    #[test]
    fn test_classify_testable() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src/services")).unwrap();
        std::fs::write(
            temp.path().join("src/services/core.rs"),
            "fn important() {}\n",
        )
        .unwrap();

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        let r = make_result("src/services/core.rs", "important");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::None);
    }

    #[test]
    fn test_classify_exclusions_batch() {
        let temp = tempfile::TempDir::new().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(src.join("services")).unwrap();
        std::fs::create_dir_all(src.join("cli")).unwrap();
        std::fs::write(src.join("services/core.rs"), "fn testable() {}\n").unwrap();
        std::fs::write(
            src.join("cli/handler.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn excluded() {}\n",
        )
        .unwrap();

        let mut results = vec![
            make_result("src/services/core.rs", "testable"),
            make_result("src/cli/handler.rs", "excluded"),
        ];

        classify_exclusions(&mut results, temp.path(), None);

        assert_eq!(results[0].coverage_exclusion, CoverageExclusion::None);
        assert!(!results[0].coverage_excluded);
        assert_eq!(
            results[1].coverage_exclusion,
            CoverageExclusion::CoverageOff
        );
        assert!(results[1].coverage_excluded);
    }

    #[test]
    fn test_exclusion_summary() {
        let mut r1 = make_result("src/cli/a.rs", "f1");
        r1.coverage_exclusion = CoverageExclusion::CoverageOff;
        let mut r2 = make_result("src/cli/b.rs", "f2");
        r2.coverage_exclusion = CoverageExclusion::CoverageOff;
        let mut r3 = make_result("src/cli/a.rs", "f3");
        r3.coverage_exclusion = CoverageExclusion::CoverageOff;
        let mut r4 = make_result("src/dead.rs", "f4");
        r4.coverage_exclusion = CoverageExclusion::DeadCode;
        let mut r5 = make_result("src/mcp/x.rs", "f5");
        r5.coverage_exclusion = CoverageExclusion::MakefileExcluded;

        let refs: Vec<&QueryResult> = vec![&r1, &r2, &r3, &r4, &r5];
        let summary = ExclusionSummary::from_results(&refs);

        assert_eq!(summary.coverage_off_count, 3);
        assert_eq!(summary.coverage_off_files, 2);
        assert_eq!(summary.dead_code_count, 1);
        assert_eq!(summary.dead_code_files, 1);
        assert_eq!(summary.makefile_count, 1);
        assert_eq!(summary.makefile_files, 1);
        assert_eq!(summary.total(), 5);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_exclusion_summary_empty() {
        let summary = ExclusionSummary::from_results(&[]);
        assert!(summary.is_empty());
        assert_eq!(summary.total(), 0);
    }

    #[test]
    fn test_parse_makefile_coverage_exclude() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Makefile"),
            "COVERAGE_EXCLUDE := --ignore-filename-regex='(_tests?\\.rs|/(tests|benches)/|main\\.rs)'\n",
        ).unwrap();

        let re = parse_makefile_coverage_exclude(temp.path());
        assert!(re.is_some());
        let re = re.unwrap();
        assert!(re.is_match("src/foo_test.rs"));
        assert!(re.is_match("src/tests/bar.rs"));
        assert!(re.is_match("main.rs"));
        assert!(!re.is_match("src/services/core.rs"));
    }

    #[test]
    fn test_parse_makefile_double_backslash_dot() {
        // Real Makefiles use `\\.` (double-backslash-dot) which cargo-llvm-cov
        // interprets as literal dot. Verify our normalization handles this.
        let temp = tempfile::TempDir::new().unwrap();
        // Write raw bytes with double-backslash (0x5C 0x5C) + dot (0x2E)
        let content = b"COVERAGE_EXCLUDE := --ignore-filename-regex='(build_perf_impl\\\\.rs|storage_impl\\\\.rs)'\n";
        std::fs::write(temp.path().join("Makefile"), content).unwrap();

        let re = parse_makefile_coverage_exclude(temp.path());
        assert!(re.is_some(), "Should parse double-backslash regex");
        let re = re.unwrap();
        // Must match actual file paths (without backslashes)
        assert!(
            re.is_match("src/services/build_perf_impl.rs"),
            "Should match build_perf_impl.rs with literal dot"
        );
        assert!(
            re.is_match("src/tdg/storage_impl.rs"),
            "Should match storage_impl.rs with literal dot"
        );
        // Must NOT match paths without the exact filename
        assert!(
            !re.is_match("src/services/core.rs"),
            "Should not match unrelated files"
        );
    }

    #[test]
    fn test_parse_makefile_no_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let re = parse_makefile_coverage_exclude(temp.path());
        assert!(re.is_none());
    }

    #[test]
    fn test_load_dead_code_functions() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".pmat")).unwrap();
        std::fs::write(
            temp.path().join(".pmat/dead-code-cache.json"),
            r#"{
                "report": {
                    "files_with_dead_code": [
                        {
                            "file_path": "src/old.rs",
                            "dead_items": [
                                {"name": "unused_fn", "kind": "function"},
                                {"name": "OldStruct", "kind": "struct"}
                            ],
                            "file_dead_percentage": 50.0
                        }
                    ]
                }
            }"#,
        )
        .unwrap();

        let dead = load_dead_code_functions(temp.path());
        assert!(dead.contains("src/old.rs::unused_fn"));
        assert!(dead.contains("src/old.rs::OldStruct"));
        assert_eq!(dead.len(), 2);
    }

    #[test]
    fn test_load_dead_code_no_cache() {
        let temp = tempfile::TempDir::new().unwrap();
        let dead = load_dead_code_functions(temp.path());
        assert!(dead.is_empty());
    }

    #[test]
    fn test_coverage_exclusion_serde_roundtrip() {
        let variants = vec![
            CoverageExclusion::None,
            CoverageExclusion::CoverageOff,
            CoverageExclusion::DeadCode,
            CoverageExclusion::MakefileExcluded,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let deserialized: CoverageExclusion = serde_json::from_str(&json).unwrap();
            assert_eq!(v, deserialized);
        }
    }

    #[test]
    fn test_dead_code_priority_over_coverage_off() {
        // Dead code should be classified as DeadCode even if file also has coverage(off)
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/mixed.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn dead_fn() {}\n",
        )
        .unwrap();

        let mut dead = HashSet::new();
        dead.insert("src/mixed.rs::dead_fn".to_string());

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: dead,
            use_cached: false,
        };

        let r = make_result("src/mixed.rs", "dead_fn");
        let excl = ctx.classify(&r, temp.path());
        // Dead code has higher priority
        assert_eq!(excl, CoverageExclusion::DeadCode);
    }

    #[test]
    fn test_coverage_off_caching() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/cached.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn a() {}\nfn b() {}\n",
        )
        .unwrap();

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        // First check reads file
        let r1 = make_result("src/cached.rs", "a");
        assert_eq!(
            ctx.classify(&r1, temp.path()),
            CoverageExclusion::CoverageOff
        );
        // Second check uses cache
        assert!(ctx.coverage_off_files.contains("src/cached.rs"));
        let r2 = make_result("src/cached.rs", "b");
        assert_eq!(
            ctx.classify(&r2, temp.path()),
            CoverageExclusion::CoverageOff
        );
    }
}
