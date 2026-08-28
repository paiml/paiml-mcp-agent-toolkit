#![cfg_attr(coverage_nightly, coverage(off))]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Lines `add_item` bills for a dead item whose real extent is unknown.
///
/// Exported so a producer that DOES know the real extent can swap this estimate
/// out instead of adding a second figure on top of it.
pub const UNREACHABLE_BLOCK_LINE_ESTIMATE: usize = 3;

/// Lines `add_item` bills for a dead module whose real extent is unknown.
///
/// A module is a container, so it is charged at least what a container type is
/// charged (`Class`, 10). Exported for the same reason as
/// [`UNREACHABLE_BLOCK_LINE_ESTIMATE`]: a producer that measured the real span
/// should replace this estimate rather than add to it.
pub const DEAD_MODULE_LINE_ESTIMATE: usize = 10;

/// File-level dead code metrics with ranking support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeadCodeMetrics {
    pub path: String,
    pub dead_lines: usize,
    pub total_lines: usize,
    pub dead_percentage: f32,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub dead_modules: usize,
    pub unreachable_blocks: usize,
    pub dead_score: f32,
    pub confidence: ConfidenceLevel,
    pub items: Vec<DeadCodeItem>,
}

/// Confidence level for dead code detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,   // Definitely dead (no references)
    Medium, // Possibly dead (only internal references)
    Low,    // Might be used (dynamic calls possible)
}

/// Individual dead code item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeItem {
    pub item_type: DeadCodeType,
    pub name: String,
    pub line: u32,
    pub reason: String,
}

/// Types of dead code
///
/// #928: this used to have four variants, so two kinds the producers really do
/// emit had nowhere to go and were both filed as `Variable`. A dead MODULE
/// (`module `x` is never used`) was reported as `"item_type": "variable"` in a
/// record whose own `reason` said `module`, and it was then counted a second
/// time in the human report's "Other (fields, constants, statics)" row — which
/// derives that row from the `Variable` items — while the "Dead modules" row
/// above it counted the same item from the summary. Anything the parser could
/// not classify (`union `U` is never used`) landed on `variable` too.
///
/// A kind the enum cannot say is a kind the JSON cannot be trusted about, so
/// both are now representable. There is deliberately no catch-all default: an
/// unrecognised kind is `Other`, which SAYS it is unrecognised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadCodeType {
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "class")]
    Class,
    /// A binding: a constant, a static, a field or a variant.
    #[serde(rename = "variable")]
    Variable,
    /// A whole module rustc reported as never used.
    #[serde(rename = "module")]
    Module,
    #[serde(rename = "unreachable")]
    UnreachableCode,
    /// A dead item whose kind the producer could not name. NOT a synonym for
    /// "variable" — it means the classification is unknown, and a reader who
    /// needs the kind must read `reason`.
    #[serde(rename = "other")]
    Other,
}

/// Complete dead code ranking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeRankingResult {
    pub summary: DeadCodeSummary,
    pub ranked_files: Vec<FileDeadCodeMetrics>,
    pub analysis_timestamp: DateTime<Utc>,
    pub config: DeadCodeAnalysisConfig,
}

/// Dead code analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    pub total_files_analyzed: usize,
    pub files_with_dead_code: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f32,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub dead_modules: usize,
    pub unreachable_blocks: usize,
}

/// Configuration for dead code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeAnalysisConfig {
    pub include_unreachable: bool,
    pub include_tests: bool,
    pub min_dead_lines: usize,
}

impl FileDeadCodeMetrics {
    /// Calculate dead code score using weighted algorithm
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn calculate_score(&mut self) {
        // Weighted scoring similar to complexity ranker
        let percentage_weight = 0.35;
        let absolute_weight = 0.30;
        let function_weight = 0.20;
        let confidence_weight = 0.15;

        let confidence_multiplier = match self.confidence {
            ConfidenceLevel::High => 1.0,
            ConfidenceLevel::Medium => 0.7,
            ConfidenceLevel::Low => 0.4,
        };

        self.dead_score = (self.dead_percentage * percentage_weight)
            + (self.dead_lines.min(1000) as f32 / 10.0 * absolute_weight)
            + (self.dead_functions.min(50) as f32 * 2.0 * function_weight)
            + (100.0 * confidence_multiplier * confidence_weight);
    }

    /// Create a new `FileDeadCodeMetrics` instance
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(path: String) -> Self {
        Self {
            path,
            dead_lines: 0,
            total_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 0.0,
            confidence: ConfidenceLevel::Medium,
            items: Vec::new(),
        }
    }

    /// Add a dead code item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_item(&mut self, item: DeadCodeItem) {
        match item.item_type {
            DeadCodeType::Function => {
                self.dead_functions += 1;
                self.dead_lines += 10; // Estimate 10 lines per function
            }
            DeadCodeType::Class => {
                self.dead_classes += 1;
                self.dead_lines += 10; // Estimate 10 lines per class
            }
            // #928: this arm used to read `self.dead_modules += 1` under the
            // comment "Track variables in module counter", so `dead_modules`
            // was a count of things that are not modules and a real dead module
            // (which had no variant of its own) was billed to it by accident
            // rather than on purpose. A variable is not a module; it increments
            // no kind counter, and the item list is what accounts for it.
            DeadCodeType::Variable | DeadCodeType::Other => {
                self.dead_lines += 1; // Estimate 1 line per binding
            }
            DeadCodeType::Module => {
                self.dead_modules += 1;
                self.dead_lines += DEAD_MODULE_LINE_ESTIMATE;
            }
            DeadCodeType::UnreachableCode => {
                self.unreachable_blocks += 1;
                self.dead_lines += UNREACHABLE_BLOCK_LINE_ESTIMATE;
            }
        }
        self.items.push(item);
    }

    /// Update dead code percentage based on current counts
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_percentage(&mut self) {
        if self.total_lines > 0 {
            self.dead_percentage = (self.dead_lines as f32 / self.total_lines as f32) * 100.0;
        }
    }
}

impl DeadCodeSummary {
    /// Create a new summary from file metrics
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_files(files: &[FileDeadCodeMetrics]) -> Self {
        let total_files_analyzed = files.len();
        let files_with_dead_code = files.iter().filter(|f| f.dead_lines > 0).count();
        let total_dead_lines = files.iter().map(|f| f.dead_lines).sum();
        let dead_functions = files.iter().map(|f| f.dead_functions).sum();
        let dead_classes = files.iter().map(|f| f.dead_classes).sum();
        let dead_modules = files.iter().map(|f| f.dead_modules).sum();
        let unreachable_blocks = files.iter().map(|f| f.unreachable_blocks).sum();

        let total_lines: usize = files.iter().map(|f| f.total_lines).sum();
        let dead_percentage = if total_lines > 0 {
            (total_dead_lines as f32 / total_lines as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_files_analyzed,
            files_with_dead_code,
            total_dead_lines,
            dead_percentage,
            dead_functions,
            dead_classes,
            dead_modules,
            unreachable_blocks,
        }
    }
}

impl Default for DeadCodeAnalysisConfig {
    fn default() -> Self {
        Self {
            include_unreachable: false,
            include_tests: false,
            min_dead_lines: 10,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_file_dead_code_metrics_creation() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());

        assert_eq!(metrics.path, "test.rs");
        assert_eq!(metrics.dead_lines, 0);
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.dead_percentage, 0.0);
        assert_eq!(metrics.dead_functions, 0);
        assert_eq!(metrics.dead_classes, 0);
        assert_eq!(metrics.dead_modules, 0);
        assert_eq!(metrics.unreachable_blocks, 0);
        assert_eq!(metrics.dead_score, 0.0);
        assert!(matches!(metrics.confidence, ConfidenceLevel::Medium));
        assert!(metrics.items.is_empty());

        // Test adding an item
        let item = DeadCodeItem {
            item_type: DeadCodeType::Function,
            name: "unused_function".to_string(),
            line: 42,
            reason: "Never called".to_string(),
        };

        metrics.add_item(item);
        assert_eq!(metrics.dead_functions, 1);
        assert_eq!(metrics.items.len(), 1);

        // Test score calculation
        metrics.total_lines = 100;
        metrics.dead_lines = 20;
        metrics.update_percentage();
        assert_eq!(metrics.dead_percentage, 20.0);

        metrics.calculate_score();
        assert!(metrics.dead_score > 0.0);
    }

    #[test]
    fn test_dead_code_item_creation() {
        let item = DeadCodeItem {
            item_type: DeadCodeType::Class,
            name: "UnusedClass".to_string(),
            line: 15,
            reason: "Never instantiated".to_string(),
        };

        assert_eq!(item.item_type, DeadCodeType::Class);
        assert_eq!(item.name, "UnusedClass");
        assert_eq!(item.line, 15);
        assert_eq!(item.reason, "Never instantiated");
    }

    #[test]
    fn test_dead_code_type_variants() {
        let types = [
            DeadCodeType::Function,
            DeadCodeType::Class,
            DeadCodeType::Variable,
            DeadCodeType::Module,
            DeadCodeType::UnreachableCode,
            DeadCodeType::Other,
        ];

        for dead_type in types {
            // Test that the types can be created and compared
            let item = DeadCodeItem {
                item_type: dead_type,
                name: "test".to_string(),
                line: 1,
                reason: "test".to_string(),
            };
            assert_eq!(item.item_type, dead_type);
        }
    }

    #[test]
    fn test_confidence_levels() {
        let levels = [
            ConfidenceLevel::High,
            ConfidenceLevel::Medium,
            ConfidenceLevel::Low,
        ];

        for level in levels {
            let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());
            metrics.confidence = level;
            assert_eq!(metrics.confidence, level);
        }
    }

    #[test]
    fn test_dead_code_ranking_result() {
        let config = DeadCodeAnalysisConfig::default();
        let summary = DeadCodeSummary::from_files(&[]);
        let timestamp = Utc::now();

        let result = DeadCodeRankingResult {
            summary: summary.clone(),
            ranked_files: vec![],
            analysis_timestamp: timestamp,
            config: config.clone(),
        };

        assert_eq!(result.summary.total_files_analyzed, 0);
        assert_eq!(result.ranked_files.len(), 0);
        assert_eq!(result.config.min_dead_lines, config.min_dead_lines);
    }

    #[test]
    fn test_dead_code_summary_from_files() {
        let mut file1 = FileDeadCodeMetrics::new("file1.rs".to_string());
        file1.dead_lines = 10;
        file1.total_lines = 100;
        file1.dead_functions = 2;
        file1.dead_classes = 1;
        file1.dead_modules = 0;
        file1.unreachable_blocks = 1;

        let mut file2 = FileDeadCodeMetrics::new("file2.rs".to_string());
        file2.dead_lines = 5;
        file2.total_lines = 50;
        file2.dead_functions = 1;
        file2.dead_classes = 0;
        file2.dead_modules = 1;
        file2.unreachable_blocks = 0;

        let files = vec![file1, file2];
        let summary = DeadCodeSummary::from_files(&files);

        assert_eq!(summary.total_files_analyzed, 2);
        assert_eq!(summary.files_with_dead_code, 2);
        assert_eq!(summary.total_dead_lines, 15);
        assert_eq!(summary.dead_functions, 3);
        assert_eq!(summary.dead_classes, 1);
        assert_eq!(summary.dead_modules, 1);
        assert_eq!(summary.unreachable_blocks, 1);
        assert_eq!(summary.dead_percentage, 10.0); // 15 dead lines out of 150 total
    }

    #[test]
    fn test_dead_code_analysis_config_default() {
        let config = DeadCodeAnalysisConfig::default();

        assert!(!config.include_unreachable);
        assert!(!config.include_tests);
        assert_eq!(config.min_dead_lines, 10);
    }

    #[test]
    fn test_file_metrics_add_different_item_types() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());

        // Add function
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Function,
            name: "fn1".to_string(),
            line: 10,
            reason: "unused".to_string(),
        });

        // Add class
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Class,
            name: "Class1".to_string(),
            line: 20,
            reason: "unused".to_string(),
        });

        // Add variable
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Variable,
            name: "var1".to_string(),
            line: 30,
            reason: "unused".to_string(),
        });

        // Add unreachable code
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::UnreachableCode,
            name: "block".to_string(),
            line: 40,
            reason: "unreachable".to_string(),
        });

        assert_eq!(metrics.dead_functions, 1);
        assert_eq!(metrics.dead_classes, 1);
        assert_eq!(metrics.unreachable_blocks, 1);
        assert_eq!(metrics.items.len(), 4);
    }

    /// #928 REGRESSION. `dead_modules` counted VARIABLES ("Track variables in
    /// module counter") because `DeadCodeType` had no `Module` variant, so the
    /// one figure named after modules was the one figure that could not be a
    /// module count. On the old enum this test does not compile
    /// (`DeadCodeType::Module` does not exist); with the old `add_item` it
    /// fails on both asserts.
    #[test]
    fn test_dead_modules_counts_modules_and_not_variables() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());
        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Variable,
            name: "CONST".to_string(),
            line: 1,
            reason: "constant `CONST` is never used".to_string(),
        });
        assert_eq!(
            metrics.dead_modules, 0,
            "a constant is not a module and must not be counted as one"
        );

        metrics.add_item(DeadCodeItem {
            item_type: DeadCodeType::Module,
            name: "dead_mod".to_string(),
            line: 2,
            reason: "module `dead_mod` is never used".to_string(),
        });
        assert_eq!(metrics.dead_modules, 1, "a dead module is counted once");
    }

    /// The two new variants must be legible to a JSON consumer, and must not
    /// collide with the bucket they used to be folded into.
    #[test]
    fn test_module_and_other_serialize_under_their_own_names() {
        assert_eq!(
            serde_json::to_string(&DeadCodeType::Module).unwrap(),
            "\"module\""
        );
        assert_eq!(
            serde_json::to_string(&DeadCodeType::Other).unwrap(),
            "\"other\""
        );
        assert_ne!(DeadCodeType::Module, DeadCodeType::Variable);
        assert_ne!(DeadCodeType::Other, DeadCodeType::Variable);
    }

    #[test]
    fn test_score_calculation_with_different_confidence_levels() {
        let mut metrics = FileDeadCodeMetrics::new("test.rs".to_string());
        metrics.dead_lines = 50;
        metrics.total_lines = 100;
        metrics.dead_functions = 5;
        metrics.update_percentage();

        // Test with high confidence
        metrics.confidence = ConfidenceLevel::High;
        metrics.calculate_score();
        let high_score = metrics.dead_score;

        // Test with medium confidence
        metrics.confidence = ConfidenceLevel::Medium;
        metrics.calculate_score();
        let medium_score = metrics.dead_score;

        // Test with low confidence
        metrics.confidence = ConfidenceLevel::Low;
        metrics.calculate_score();
        let low_score = metrics.dead_score;

        // High confidence should result in higher score than medium, which should be higher than low
        assert!(high_score > medium_score);
        assert!(medium_score > low_score);
    }
}

/// What the analyzer decided about the analysed target being a LIBRARY.
///
/// A library's exported API is un-called *by construction* — its callers are
/// outside the tree — so an engine whose only rule is "nothing calls it" reports
/// the whole API as dead. Which way that question was answered decides which
/// findings exist, so it is published beside them rather than left as an
/// invisible default.
///
/// The `undetermined` verdict is the one that matters most: it means exported
/// items were NOT kept, so an un-called export IS in the list below, and the
/// reader has to supply the knowledge the analyzer lacked. Naming that gap is
/// the point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryTargetReport {
    /// `"library"`, `"not-a-library"` or `"undetermined"`.
    pub verdict: String,
    /// The evidence behind the verdict, or — for `"undetermined"` — what could
    /// not be decided and why.
    pub detail: String,
    /// How many exported items this analyzer seeded as reachability roots.
    ///
    /// `None` when the analyzer did not make that decision itself: the cargo
    /// engine defers to rustc's own dead-code pass, which already treats a
    /// library's public API as reachable. A `0` there would read as "it looked
    /// and found none".
    pub exported_roots: Option<usize>,
}

/// Verdict for a compiler-lint layer that ran and reported.
pub const COMPILER_SCAN_FULL: &str = "full";
/// Verdict for a run whose compiler-lint layer was refused, leaving only the
/// layers that need no compiler.
pub const COMPILER_SCAN_REDUCED: &str = "reduced";
/// Machine-readable cause for [`COMPILER_SCAN_FULL`].
pub const COMPILER_SCAN_REASON_OK: &str = "compiler-lint-ran";
/// Machine-readable cause for a scan given up because running it would have
/// written a `Cargo.lock` into the analysed repository.
pub const COMPILER_SCAN_REASON_LOCKFILE: &str = "lockfile-would-be-written";
/// Machine-readable cause for a scan suppressed by `PMAT_DEAD_CODE_SKIP`.
pub const COMPILER_SCAN_REASON_ENV_SKIP: &str = "suppressed-by-env";

/// Whether the COMPILER-LINT layer of the scan actually ran.
///
/// Rust dead code is found by two layers: a source scan for explicit
/// `allow(dead_code)` admissions, and rustc's own dead-code lint driven by
/// `cargo check`. Only the second one finds code nobody admitted was dead — the
/// overwhelming majority of real findings — so a run without it produces the
/// same report SHAPE over a far smaller search, and `0 dead items` then means
/// "nothing was admitted", not "nothing is dead".
///
/// That difference is invisible in the numbers, so it is published as a field.
/// This is the same disclosure contract as [`LibraryTargetReport`]: name what
/// the analyzer could not do and why, rather than let a silent default stand in
/// for a measurement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerScanReport {
    /// [`COMPILER_SCAN_FULL`] or [`COMPILER_SCAN_REDUCED`].
    pub verdict: String,
    /// A stable token for the cause, so a consumer never has to match on prose.
    /// One of [`COMPILER_SCAN_REASON_OK`], [`COMPILER_SCAN_REASON_LOCKFILE`],
    /// [`COMPILER_SCAN_REASON_ENV_SKIP`].
    pub reason: String,
    /// The evidence: for a reduced scan, what was refused and what the reader
    /// can do about it.
    pub detail: String,
}

impl CompilerScanReport {
    /// The compiler layer ran; the report covers everything both layers see.
    #[must_use]
    pub fn full() -> Self {
        Self {
            verdict: COMPILER_SCAN_FULL.to_string(),
            reason: COMPILER_SCAN_REASON_OK.to_string(),
            detail: "cargo check ran against the existing lockfile; rustc's dead-code lint \
                     contributed to these findings"
                .to_string(),
        }
    }

    /// The compiler layer did not run. `detail` must say what stopped it.
    #[must_use]
    pub fn reduced(reason: &str, detail: String) -> Self {
        Self {
            verdict: COMPILER_SCAN_REDUCED.to_string(),
            reason: reason.to_string(),
            detail,
        }
    }

    /// Did the compiler layer contribute to this report?
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.verdict == COMPILER_SCAN_FULL
    }
}

// Additional type for handler compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Result of dead code operation.
pub struct DeadCodeResult {
    /// Aggregate of the files in `files` — never of a wider set. Reporting the
    /// pre-filter count here is what made `files_with_dead_code: 26` head a
    /// 4-entry array (and `1` head an EMPTY one).
    pub summary: DeadCodeSummary,
    pub files: Vec<FileDeadCodeMetrics>,
    pub total_files: usize,
    pub analyzed_files: usize,
    /// Files found to contain dead code BEFORE `--min-dead-lines` and
    /// `--top-files` reduced the list. `files.len()` is what is listed; both
    /// numbers are named so the cap is never mistaken for the total.
    #[serde(default)]
    pub files_with_dead_code_found: usize,
    /// True when `--top-files` cut the list short.
    #[serde(default)]
    pub files_truncated: bool,
    /// Whether the analyzer decided this target was a library, and hence
    /// whether its exported items were kept as entry points rather than listed
    /// as dead. See [`LibraryTargetReport`].
    #[serde(default)]
    pub library_target: Option<LibraryTargetReport>,
    /// Whether rustc's dead-code lint contributed to these findings, and — when
    /// it did not — what stopped it. See [`CompilerScanReport`].
    ///
    /// `None` only for engines that have no compiler layer to report on (the
    /// multi-language analyzer), never as a stand-in for "it ran".
    #[serde(default)]
    pub compiler_scan: Option<CompilerScanReport>,
}

impl DeadCodeResult {
    /// How many files with dead code are not listed (threshold + cap).
    #[must_use]
    pub fn files_omitted(&self) -> usize {
        self.files_with_dead_code_found
            .saturating_sub(self.files.len())
    }
}
