// Proof annotations and incremental coverage handlers - extracted for file health (CB-040)
/// Analyzes and extracts formal proof annotations from source code.
///
/// This advanced analysis command identifies formal verification annotations,
/// proof hints, and mathematical properties embedded in code comments and
/// attributes. Essential for projects using formal methods or seeking to
/// understand verification potential.
///
/// # Parameters
///
/// * `project_path` - Root directory of the project to analyze
/// * `format` - Output format for proof annotation results
/// * `high_confidence_only` - Only include annotations with high confidence scores
/// * `include_evidence` - Include supporting evidence and context for annotations
/// * `property_type` - Filter by specific property types (safety, liveness, etc.)
/// * `verification_method` - Filter by verification method (model checking, theorem proving, etc.)
/// * `output` - Optional output file path
/// * `perf` - Enable performance optimizations
/// * `clear_cache` - Clear analysis cache before processing
///
/// # Returns
///
/// * `Ok(())` - Proof annotation analysis completed successfully
/// * `Err(anyhow::Error)` - Analysis failed with detailed error context
///
/// # Proof Annotation Types
///
/// ## Mathematical Properties
/// - **Invariants**: Loop and data structure invariants
/// - **Preconditions**: Function input requirements
/// - **Postconditions**: Function output guarantees
/// - **Assertions**: Runtime verification checkpoints
///
/// ## Verification Annotations
/// - **Safety Properties**: Memory safety, bounds checking
/// - **Liveness Properties**: Termination, progress guarantees
/// - **Security Properties**: Information flow, access control
/// - **Performance Properties**: Time/space complexity bounds
///
/// # Supported Annotation Formats
///
/// - **Rust**: `#[requires]`, `#[ensures]`, `#[invariant]` attributes
/// - **ACSL**: C/C++ specification language annotations
/// - **JML**: Java Modeling Language specifications
/// - **Dafny**: Verification-aware programming language constructs
/// - **Custom**: Project-specific proof annotation patterns
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::handle_analyze_proof_annotations;
/// use pmat::cli::enums::{ProofAnnotationOutputFormat, PropertyTypeFilter, VerificationMethodFilter};
/// use std::path::{Path, PathBuf};
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a project with proof annotations
/// let dir = tempdir().unwrap();
/// let annotated_rs = dir.path().join("verified.rs");
/// fs::write(&annotated_rs, r#"
/// /// @requires x >= 0
/// /// @ensures result >= x
/// fn increment(x: i32) -> i32 {
///     x + 1
/// }
/// "#).unwrap();
///
/// // Standard proof annotation analysis
/// let result = handle_analyze_proof_annotations(
///     dir.path().to_path_buf(),
///     ProofAnnotationOutputFormat::Summary,
///     false, // include all confidence levels
///     true,  // include evidence
///     None,  // all property types
///     None,  // all verification methods
///     None,  // stdout output
///     false, // normal performance
///     false, // keep cache
///     10,    // top_files
/// ).await;
///
/// assert!(result.is_ok());
///
/// // High-confidence safety properties only
/// let safety_result = handle_analyze_proof_annotations(
///     dir.path().to_path_buf(),
///     ProofAnnotationOutputFormat::Json,
///     true,  // high confidence only
///     true,  // include evidence
///     Some(PropertyTypeFilter::MemorySafety),
///     Some(VerificationMethodFilter::ModelChecking),
///     Some(dir.path().join("safety-proofs.json")),
///     true,  // performance mode
///     true,  // clear cache
///     10,    // top_files
/// ).await;
///
/// assert!(safety_result.is_ok());
/// # });
/// ```
///
/// # CLI Usage Examples
///
/// ```bash
/// # Extract all proof annotations
/// pmat analyze proof-annotations /path/to/project --format summary \
///   --include-evidence
///
/// # High-confidence safety properties only
/// pmat analyze proof-annotations /path/to/project --format json \
///   --high-confidence-only --property-type safety \
///   --output safety-annotations.json
///
/// # Full analysis with evidence for formal verification
/// pmat analyze proof-annotations /path/to/project --format full \
///   --include-evidence --verification-method theorem-proving \
///   --clear-cache --output formal-specs.md
/// ```ignore
/// This is the one implementation, re-exported.
///
/// There used to be a SECOND `handle_analyze_proof_annotations` here: a
/// copy-paste of the wired handler that nothing dispatched to, kept alive by
/// its own unit test. It had drifted into a strictly worse copy — no
/// `ensure_analysis_path_exists` guard (so it produced a full report for a
/// path that does not exist), no disclosure of files the collector could not
/// parse, and no `--top-files` limit. Syncing three fixes into a duplicate is
/// how the next divergence starts; the duplicate is gone instead.
pub use crate::cli::handlers::proof_annotations_handler::handle_analyze_proof_annotations;

/// Analyzes incremental test coverage between Git branches.
///
/// This command performs differential coverage analysis, comparing test coverage
/// between a base branch and target branch to identify coverage gaps introduced
/// by new code changes. Critical for maintaining test quality in CI/CD pipelines.
///
/// # Parameters
///
/// * `project_path` - Root directory of the Git repository to analyze
/// * `base_branch` - Base branch for comparison (e.g., "main", "develop")
/// * `target_branch` - Target branch to analyze (defaults to HEAD if None)
/// * `format` - Output format for coverage analysis results
/// * `coverage_threshold` - Minimum coverage percentage required (0.0-1.0)
/// * `changed_files_only` - Only analyze files modified between branches
/// * `detailed` - Include detailed line-by-line coverage information
/// * `output` - Optional output file path
/// * `perf` - Enable performance optimizations
/// * `cache_dir` - Directory for caching coverage data
/// * `force_refresh` - Force refresh of cached coverage data
///
/// # Returns
///
/// * `Ok(())` - Coverage analysis completed successfully
/// * `Err(anyhow::Error)` - Analysis failed (Git errors, coverage tool failures, etc.)
///
/// # Coverage Analysis Process
///
/// 1. **Git Diff Analysis**: Identify changed files between branches
/// 2. **Coverage Collection**: Run test suite with coverage instrumentation
/// 3. **Differential Calculation**: Compare coverage between base and target
/// 4. **Gap Identification**: Highlight uncovered lines in new/modified code
/// 5. **Threshold Validation**: Check if coverage meets required standards
///
/// # Supported Coverage Tools
///
/// - **Rust**: cargo-llvm-cov, grcov
/// - **JavaScript/TypeScript**: nyc, jest coverage, c8
/// - **Python**: coverage.py, pytest-cov
/// - **Java**: `JaCoCo`, Cobertura
/// - **C/C++**: gcov, lcov
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::handle_analyze_incremental_coverage;
/// use pmat::cli::IncrementalCoverageOutputFormat;
/// use std::path::{Path, PathBuf};
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a Git repository-like structure
/// let dir = tempdir().unwrap();
/// let main_rs = dir.path().join("src/main.rs");
/// fs::create_dir_all(dir.path().join("src")).unwrap();
/// fs::write(&main_rs, "fn main() { println!(\"Hello, world!\"); }").unwrap();
///
/// // Standard incremental coverage analysis
/// let result = handle_analyze_incremental_coverage(
///     dir.path().to_path_buf(),
///     "main".to_string(),          // base branch
///     Some("feature".to_string()), // target branch
///     IncrementalCoverageOutputFormat::Summary,
///     0.8,   // 80% coverage threshold
///     false, // analyze all files
///     false, // summary only
///     None,  // stdout output
///     false, // normal performance
///     None,  // default cache dir
///     false, // use cache
///     10,    // top files
/// ).await;
///
/// assert!(result.is_ok());
///
/// // Detailed analysis for changed files only
/// let detailed_result = handle_analyze_incremental_coverage(
///     dir.path().to_path_buf(),
///     "main".to_string(),
///     None,    // compare with HEAD
///     IncrementalCoverageOutputFormat::Detailed,
///     0.9,     // 90% coverage threshold
///     true,    // changed files only
///     true,    // detailed coverage
///     Some(dir.path().join("coverage-report.json")),
///     true,    // performance mode
///     Some(dir.path().join(".coverage-cache")),
///     true,    // force refresh
///     15,      // top files
/// ).await;
///
/// assert!(detailed_result.is_ok());
/// # });
/// ```
///
/// # CLI Usage Examples
///
/// ```bash
/// # Basic incremental coverage between main and current branch
/// pmat analyze incremental-coverage /path/to/project --base-branch main \
///   --coverage-threshold 0.8 --format summary
///
/// # Detailed analysis for changed files only
/// pmat analyze incremental-coverage /path/to/project --base-branch develop \
///   --target-branch feature/new-api --changed-files-only --detailed \
///   --format json --output coverage-diff.json
///
/// # CI/CD pipeline usage with high threshold
/// pmat analyze incremental-coverage /path/to/project --base-branch main \
///   --coverage-threshold 0.95 --perf --force-refresh \
///   --output coverage-gate.json
/// ```ignore
///
/// This is the one implementation, forwarded to the wired handler.
///
/// There used to be a SECOND implementation here — the one this doc comment
/// belongs to — and it was fed entirely by fabrications. Its producer,
/// `IncrementalCoverageAnalyzer::compute_coverage`, is marked
/// `// For now, return mock data`: `branch_coverage: 75.0` and
/// `function_coverage: 80.0` are literals and `line_coverage` is
/// `(0..total).filter(|i| i % 3 != 0)`, i.e. 66.67% for every file in every
/// project. Its converter then derived the "previous" figure as
/// `line_coverage.max(50.0) - 10.0  // Simulate previous coverage`, so every
/// file reported a fixed +10.0 delta. Four fabricated quantities per file,
/// behind a `pub fn` whose own doc examples above recommend it for CI gating
/// (#954).
///
/// The CLI never routed here — `platform_routes_routing.rs:159` dispatches to
/// `incremental_coverage_handler`, which measures and reports "not measured"
/// for the files it cannot measure. That is now the only implementation, and
/// this entry point is a signature-compatible forwarder to it. Two
/// implementations of one command name is how the contradictions in this
/// project keep recurring; the fabricating one is gone rather than synced.
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_incremental_coverage(
    project_path: PathBuf,
    base_branch: String,
    target_branch: Option<String>,
    format: IncrementalCoverageOutputFormat,
    coverage_threshold: f64,
    changed_files_only: bool,
    detailed: bool,
    output: Option<PathBuf>,
    perf: bool,
    cache_dir: Option<PathBuf>,
    force_refresh: bool,
    top_files: usize,
) -> Result<()> {
    use crate::cli::handlers::incremental_coverage_handler::{
        handle_analyze_incremental_coverage as wired, IncrementalCoverageConfig,
    };

    wired(IncrementalCoverageConfig {
        project_path,
        base_branch,
        target_branch,
        format,
        coverage_threshold,
        changed_files_only,
        detailed,
        output,
        perf,
        cache_dir,
        force_refresh,
        top_files,
    })
    .await
}

#[cfg(test)]
mod incremental_coverage_library_entry_tests {
    //! #954: the public library entry point used to report a fabricated
    //! coverage figure and a simulated baseline. It now produces the same
    //! document the CLI does.
    use super::*;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(["-c", "core.hooksPath=/dev/null"])
            .args(["-c", "user.email=a@b", "-c", "user.name=c"])
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git");
        assert!(
            status.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    /// A repo with a `main` baseline and one changed file on top of it.
    fn repo_with_one_changed_file() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path();
        git(p, &["init", "-q", "--template=", "--initial-branch=main"]);
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(p.join("src/lib.rs"), "pub fn a() -> i32 { 1 }\n").unwrap();
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "--no-verify", "-m", "base"]);
        git(p, &["checkout", "-q", "-b", "feature"]);
        std::fs::write(
            p.join("src/lib.rs"),
            "pub fn a() -> i32 { 1 }\npub fn b() -> i32 { 2 }\n",
        )
        .unwrap();
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "--no-verify", "-m", "change"]);
        dir
    }

    #[tokio::test]
    async fn the_library_entry_point_reports_not_measured_instead_of_a_simulated_delta() {
        let repo = repo_with_one_changed_file();
        let out = repo.path().join("report.json");

        handle_analyze_incremental_coverage(
            repo.path().to_path_buf(),
            "main".to_string(),
            None,
            IncrementalCoverageOutputFormat::Json,
            80.0,
            false,
            false,
            Some(out.clone()),
            false,
            None,
            false,
            10,
        )
        .await
        .expect("analysis");

        let text = std::fs::read_to_string(&out).expect("report written");
        let json: serde_json::Value = serde_json::from_str(&text).expect("valid json");

        // The honest shape: absence is a value. The fabricating implementation
        // had no such field — it always had a number.
        assert_eq!(
            json["files_not_measured"].as_u64(),
            Some(1),
            "the one changed file has no coverage artifact, so it is not measured: {text}"
        );
        assert!(
            json["coverage_percentage"].is_null(),
            "an unmeasured project must not report a percentage: {text}"
        );

        // The fabrications, by name. The old implementation reported
        // base_coverage = max(line_coverage, 50.0) - 10.0 and hence a fixed
        // coverage_delta of +10.0 for every file; the honest one has no
        // baseline to report and says so.
        let file = &json["changed_files"][0];
        assert_eq!(file["status"].as_str(), Some("NotMeasured"), "{text}");
        assert!(
            file["coverage_delta"].is_null(),
            "the simulated `max(50.0) - 10.0` baseline delta is back: {text}"
        );
        assert!(
            file["coverage_before"].is_null() && file["coverage_after"].is_null(),
            "a mock coverage constant leaked into the report: {text}"
        );
        assert!(
            !text.contains("base_coverage") && !text.contains("target_coverage"),
            "the fabricating report shape is back: {text}"
        );
    }
}
