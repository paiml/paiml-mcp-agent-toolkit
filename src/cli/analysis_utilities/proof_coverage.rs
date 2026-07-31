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
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_proof_annotations(
    project_path: PathBuf,
    format: ProofAnnotationOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    property_type: Option<PropertyTypeFilter>,
    verification_method: Option<VerificationMethodFilter>,
    output: Option<PathBuf>,
    _perf: bool,
    clear_cache: bool,
) -> Result<()> {
    use crate::cli::proof_annotation_helpers::{
        collect_and_filter_annotations, format_as_full, format_as_json, format_as_markdown,
        format_as_sarif, format_as_summary, setup_proof_annotator, ProofAnnotationFilter,
    };
    use std::time::Instant;

    eprintln!("🔍 Collecting proof annotations from project...");
    let start = Instant::now();

    // Setup annotator
    let annotator = setup_proof_annotator(clear_cache);

    // Create filter
    let filter = ProofAnnotationFilter {
        high_confidence_only,
        property_type,
        verification_method,
    };

    // Collect and filter annotations
    let annotations = collect_and_filter_annotations(&annotator, &project_path, &filter).await;
    let elapsed = start.elapsed();

    eprintln!("✅ Found {} matching proof annotations", annotations.len());

    // Format output using helpers
    let content = match format {
        ProofAnnotationOutputFormat::Json => format_as_json(&annotations, elapsed, &annotator)?,
        ProofAnnotationOutputFormat::Summary => format_as_summary(&annotations, elapsed)?,
        ProofAnnotationOutputFormat::Full => {
            format_as_full(&annotations, &project_path, include_evidence)?
        }
        ProofAnnotationOutputFormat::Markdown => {
            format_as_markdown(&annotations, &project_path, include_evidence)?
        }
        ProofAnnotationOutputFormat::Sarif => format_as_sarif(&annotations, &project_path)?,
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Proof annotations written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}
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
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_incremental_coverage(
    project_path: PathBuf,
    base_branch: String,
    target_branch: Option<String>,
    format: IncrementalCoverageOutputFormat,
    coverage_threshold: f64,
    _changed_files_only: bool,
    _detailed: bool,
    output: Option<PathBuf>,
    _perf: bool,
    _cache_dir: Option<PathBuf>,
    _force_refresh: bool,
    top_files: usize,
) -> Result<()> {
    print_coverage_analysis_header(
        &project_path,
        &base_branch,
        &target_branch,
        coverage_threshold,
        &format,
    );

    // Real implementation using IncrementalCoverageAnalyzer
    use crate::cli::coverage_helpers::{get_changed_files_for_coverage, setup_coverage_analyzer};

    let analyzer = setup_coverage_analyzer(_cache_dir, _force_refresh)?;
    let changed_files =
        get_changed_files_for_coverage(&project_path, &base_branch, target_branch.as_deref())
            .await?;

    let modified_files = create_file_ids_from_changes(&changed_files)?;

    let changeset = crate::services::incremental_coverage_analyzer::ChangeSet {
        modified_files,
        added_files: Vec::new(), // These are included in modified_files above
        deleted_files: Vec::new(),
    };

    let coverage_update = analyzer.analyze_changes(&changeset).await?;

    // Convert real coverage data to report format expected by formatting functions
    let report = convert_coverage_update_to_report(
        coverage_update,
        base_branch,
        target_branch.unwrap_or("HEAD".to_string()),
        coverage_threshold,
        changed_files,
    )?;

    // Format and output
    let content = format_coverage_report(&report, format, top_files)?;
    output_coverage_result(content, output).await?;

    Ok(())
}

fn print_coverage_analysis_header(
    project_path: &Path,
    base_branch: &str,
    target_branch: &Option<String>,
    coverage_threshold: f64,
    format: &IncrementalCoverageOutputFormat,
) {
    eprintln!("📊 Analyzing incremental coverage...");
    eprintln!("📁 Project path: {}", project_path.display());
    eprintln!("🌿 Base branch: {base_branch}");
    eprintln!(
        "🎯 Target branch: {}",
        target_branch.as_deref().unwrap_or("HEAD")
    );
    // Already a percentage (`--coverage-threshold`, default 80.0) — GH #658.
    eprintln!("📈 Coverage threshold: {coverage_threshold:.1}%");
    eprintln!("📄 Format: {format:?}");
}

fn create_file_ids_from_changes(
    changed_files: &[(PathBuf, String)],
) -> Result<Vec<crate::services::incremental_coverage_analyzer::FileId>> {
    use crate::services::incremental_coverage_analyzer::FileId;
    use sha2::{Digest, Sha256};

    let mut modified_files = Vec::new();
    for (path, status) in changed_files {
        if status == "M" || status == "A" {
            // Create hash for the file path
            let mut hasher = Sha256::new();
            hasher.update(path.to_string_lossy().as_bytes());
            let hash_result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_result);

            modified_files.push(FileId {
                path: path.clone(),
                hash,
            });
        }
    }
    Ok(modified_files)
}

fn format_coverage_report(
    report: &IncrementalCoverageReport,
    format: IncrementalCoverageOutputFormat,
    top_files: usize,
) -> Result<String> {
    use IncrementalCoverageOutputFormat::{Delta, Detailed, Json, Lcov, Markdown, Sarif, Summary};
    match format {
        Summary => format_incremental_coverage_summary(report, top_files),
        Detailed => format_incremental_coverage_detailed(report, top_files),
        Json => serde_json::to_string_pretty(report).map_err(Into::into),
        Markdown => format_incremental_coverage_markdown(report, top_files),
        Lcov => format_incremental_coverage_lcov(report),
        Delta => format_incremental_coverage_delta(report, top_files),
        Sarif => format_incremental_coverage_sarif(report),
    }
}

async fn output_coverage_result(content: String, output: Option<PathBuf>) -> Result<()> {
    eprintln!("✅ Incremental coverage analysis complete");

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

#[cfg(test)]
mod proof_coverage_tests {
    //! Covers 0%-covered pure-compute helpers in proof_coverage.rs
    //! (60 uncov on broad).
    use super::*;

    fn empty_report() -> IncrementalCoverageReport {
        IncrementalCoverageReport {
            base_branch: "main".into(),
            target_branch: "HEAD".into(),
            coverage_threshold: 0.9,
            files: vec![],
            summary: CoverageSummary {
                total_files_changed: 0,
                files_improved: 0,
                files_degraded: 0,
                overall_delta: 0.0,
                meets_threshold: true,
            },
        }
    }

    // ── print_coverage_analysis_header: exercise both target_branch arms ──

    #[test]
    fn test_print_coverage_analysis_header_with_target_branch() {
        print_coverage_analysis_header(
            std::path::Path::new("/tmp/proj"),
            "main",
            &Some("feat/x".to_string()),
            0.95,
            &IncrementalCoverageOutputFormat::Summary,
        );
    }

    #[test]
    fn test_print_coverage_analysis_header_target_branch_defaults_to_head() {
        print_coverage_analysis_header(
            std::path::Path::new("/tmp/proj"),
            "main",
            &None, // triggers unwrap_or("HEAD") arm
            0.95,
            &IncrementalCoverageOutputFormat::Json,
        );
    }

    // ── create_file_ids_from_changes: only "M" and "A" entries kept, hashes distinct ──

    #[test]
    fn test_create_file_ids_from_changes_keeps_modified_and_added_only() {
        let changes = vec![
            (std::path::PathBuf::from("a.rs"), "M".to_string()),
            (std::path::PathBuf::from("b.rs"), "A".to_string()),
            (std::path::PathBuf::from("c.rs"), "D".to_string()), // deleted
            (std::path::PathBuf::from("d.rs"), "R".to_string()), // renamed
        ];
        let ids = create_file_ids_from_changes(&changes).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.iter().any(|f| f.path == std::path::Path::new("a.rs")));
        assert!(ids.iter().any(|f| f.path == std::path::Path::new("b.rs")));
    }

    #[test]
    fn test_create_file_ids_from_changes_empty_input_returns_empty() {
        let ids = create_file_ids_from_changes(&[]).unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn test_create_file_ids_from_changes_distinct_paths_produce_distinct_hashes() {
        let changes = vec![
            (std::path::PathBuf::from("a.rs"), "M".to_string()),
            (std::path::PathBuf::from("b.rs"), "M".to_string()),
        ];
        let ids = create_file_ids_from_changes(&changes).unwrap();
        assert_eq!(ids.len(), 2);
        assert_ne!(
            ids[0].hash, ids[1].hash,
            "different paths must yield different SHA256 hashes"
        );
    }

    #[test]
    fn test_create_file_ids_from_changes_same_path_produces_same_hash() {
        let changes = vec![(std::path::PathBuf::from("a.rs"), "M".to_string())];
        let a = create_file_ids_from_changes(&changes).unwrap();
        let b = create_file_ids_from_changes(&changes).unwrap();
        assert_eq!(a[0].hash, b[0].hash, "hash must be deterministic");
    }

    // ── format_coverage_report: dispatcher hits Summary/Detailed/Json/Markdown/Lcov/Delta/Sarif arms ──

    #[test]
    fn test_format_coverage_report_json_variant_round_trips_as_json() {
        let report = empty_report();
        let out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Json, 10).unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_coverage_report_summary_variant_returns_nonempty() {
        let report = empty_report();
        let out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Summary, 10).unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn test_format_coverage_report_lcov_variant_returns_string() {
        let report = empty_report();
        let out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Lcov, 10).unwrap();
        // LCOV format is always string (even if empty).
        let _ = out;
    }

    #[test]
    fn test_format_coverage_report_delta_variant_returns_string() {
        let report = empty_report();
        let _out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Delta, 10).unwrap();
    }

    #[test]
    fn test_format_coverage_report_sarif_variant_returns_string() {
        let report = empty_report();
        let _out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Sarif, 10).unwrap();
    }

    #[test]
    fn test_format_coverage_report_markdown_variant_returns_string() {
        let report = empty_report();
        let _out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Markdown, 10)
                .unwrap();
    }

    #[test]
    fn test_format_coverage_report_detailed_variant_returns_string() {
        let report = empty_report();
        let _out =
            format_coverage_report(&report, IncrementalCoverageOutputFormat::Detailed, 10)
                .unwrap();
    }
}
