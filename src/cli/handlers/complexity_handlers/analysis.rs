//! Complexity analysis logic: single file, multi-file, project, filtering, and violation checks.

use crate::services::complexity::FileComplexityMetrics;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::ComplexityConfig;

/// Analyze a single file and return its complexity metrics
///
/// This helper function handles single file analysis with proper error handling
/// and maintains consistency with the Issue #42 fix for multi-language support.
///
/// **Issue #67 Fix**: When analyzing a single file with `--file` parameter,
/// we ALWAYS use uncached analysis to ensure line numbers reflect the CURRENT
/// file location, not stale cached data from when the function was in a different file.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_single_file(
    file_path: &Path,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    crate::status_eprintln!("🔍 Analyzing complexity of file: {}", file_path.display());

    // Ensure file exists and resolve absolute path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        config.project_path.join(file_path)
    };

    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }

    // Issue #67 Fix: Use UNCACHED analysis for single file operations
    // This ensures line numbers are accurate for extracted/moved functions
    // When functions are extracted from one file to another, the TDG cache
    // (keyed by content hash) returns stale line numbers from the old location.
    // By using uncached analysis, we always report line numbers from the CURRENT file.
    let metrics = crate::services::complexity::analyze_file_complexity_uncached(&full_path, None)
        .await
        .context(format!(
            "Failed to analyze file complexity: {}",
            full_path.display()
        ))?;

    // #702: a Rust file that `include!`s fragments has most of its code in
    // OTHER files, and the parser only ever sees the includer's own body.
    // `--file src/cli/handlers/lint_hotspot_handlers/clippy.rs` reported
    // "total_functions": 4 and listed only the 4 functions written in clippy.rs,
    // silently omitting the 15 in clippy_parsing.rs and the 16 in
    // clippy_file_analysis.rs that its two `include!` lines pull in — a partial
    // count that reads exactly like a complete one. The fragments are analysed
    // and reported as their OWN entries rather than folded into the includer,
    // so every function keeps the file and line it actually lives at.
    let mut analyzed = vec![metrics];
    analyzed.extend(analyze_included_fragments(&full_path).await?);

    Ok(analyzed)
}

/// Analyze every file reachable from `root` through top-level `include!("…")`.
///
/// See #702. Returns one `FileComplexityMetrics` per included fragment, in
/// breadth-first include order. Anything that cannot be resolved is reported on
/// stderr instead of being dropped, because a silently short function list is
/// indistinguishable from a genuinely small file.
async fn analyze_included_fragments(root: &Path) -> Result<Vec<FileComplexityMetrics>> {
    let (included, unresolved) = collect_included_files(root);
    report_include_expansion(root, &included, &unresolved);

    let mut fragments = Vec::with_capacity(included.len());
    for path in &included {
        fragments.push(
            crate::services::complexity::analyze_file_complexity_uncached(path, None)
                .await
                .context(format!(
                    "Failed to analyze included fragment: {}",
                    path.display()
                ))?,
        );
    }
    Ok(fragments)
}

/// Includes nest (a fragment may include another); this bounds a pathological
/// or hand-written cycle that the visited set alone would not.
const MAX_INCLUDE_DEPTH: usize = 8;

/// Breadth-first walk of the `include!` graph rooted at `root`.
///
/// Returns the files pulled in (never including `root` itself) and one message
/// per include that could NOT be followed.
fn collect_included_files(root: &Path) -> (Vec<PathBuf>, Vec<String>) {
    let mut visited: std::collections::HashSet<PathBuf> =
        std::collections::HashSet::from([canonical_key(root)]);
    let mut queue: std::collections::VecDeque<(PathBuf, usize)> =
        std::collections::VecDeque::from([(root.to_path_buf(), 0usize)]);
    let mut included = Vec::new();
    let mut unresolved = Vec::new();

    while let Some((path, depth)) = queue.pop_front() {
        let Some(source) = read_rust_source(&path) else {
            continue;
        };
        let (targets, opaque) = scan_rust_includes(&source);
        for raw in opaque {
            unresolved.push(format!("{}: include!({raw})", path.display()));
        }

        for next in resolve_include_targets(&path, targets, &mut visited, &mut unresolved) {
            if depth + 1 < MAX_INCLUDE_DEPTH {
                queue.push_back((next.clone(), depth + 1));
            } else {
                unresolved.push(format!(
                    "{} (include nesting deeper than {MAX_INCLUDE_DEPTH} was not followed)",
                    next.display()
                ));
            }
            included.push(next);
        }
    }

    (included, unresolved)
}

/// Turn one file's `include!` string literals into paths on disk, skipping ones
/// already seen and recording ones that do not exist.
fn resolve_include_targets(
    from: &Path,
    targets: Vec<String>,
    visited: &mut std::collections::HashSet<PathBuf>,
    unresolved: &mut Vec<String>,
) -> Vec<PathBuf> {
    // `include!` resolves relative to the directory of the file it appears in.
    let dir = from.parent().unwrap_or_else(|| Path::new("."));
    let mut resolved_paths = Vec::new();

    for target in targets {
        let resolved = dir.join(&target);
        if !resolved.exists() {
            unresolved.push(format!(
                "{}: include!(\"{target}\") -> {} (not found)",
                from.display(),
                resolved.display()
            ));
        } else if visited.insert(canonical_key(&resolved)) {
            resolved_paths.push(resolved);
        }
    }

    resolved_paths
}

/// Read a Rust file's text, or `None` when it is not Rust or cannot be read.
fn read_rust_source(path: &Path) -> Option<String> {
    if path.extension().and_then(|e| e.to_str()) != Some("rs") {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

/// Tell the user which files the reported metrics actually cover.
fn report_include_expansion(root: &Path, included: &[PathBuf], unresolved: &[String]) {
    if !included.is_empty() {
        crate::status_eprintln!(
            "📎 {} also analyzed via include!(): {}",
            root.display(),
            included
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !unresolved.is_empty() {
        eprintln!(
            "⚠️  {} of {}'s include!() target(s) could not be analyzed, so its \
             function count is INCOMPLETE: {}",
            unresolved.len(),
            root.display(),
            unresolved.join("; ")
        );
    }
}

/// Identity of a path for cycle detection; falls back to the literal path when
/// it cannot be canonicalized.
fn canonical_key(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Split a Rust source's `include!` invocations into resolvable string-literal
/// targets and everything else.
///
/// The second vector holds arguments this scanner cannot resolve — typically
/// `include!(concat!(env!("OUT_DIR"), "/x.rs"))` — and exists so those are
/// REPORTED rather than quietly missing from the function list (#702).
fn scan_rust_includes(source: &str) -> (Vec<String>, Vec<String>) {
    let mut targets = Vec::new();
    let mut opaque = Vec::new();
    for line in source.lines() {
        scan_include_line(line, &mut targets, &mut opaque);
    }
    (targets, opaque)
}

/// One `include!` argument as this scanner understands it.
enum IncludeArg {
    /// A plain string literal, resolvable against the includer's directory.
    Literal(String),
    /// Anything else (`concat!`, `env!`, a macro), reported rather than dropped.
    Opaque(String),
}

fn scan_include_line(line: &str, targets: &mut Vec<String>, opaque: &mut Vec<String>) {
    // Only the code part of the line; a commented-out include is not an include.
    let code = line.split("//").next().unwrap_or("");
    let bytes = code.as_bytes();
    let mut from = 0usize;

    while let Some(hit) = code[from..].find("include!") {
        let at = from + hit;
        from = at + "include!".len();
        // `my_include!` / `nested_include!` are different macros.
        if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
            continue;
        }
        match include_argument(&code[from..]) {
            Some(IncludeArg::Literal(path)) => targets.push(path),
            Some(IncludeArg::Opaque(raw)) => opaque.push(raw),
            None => {}
        }
    }
}

/// Classify the text following an `include!` token.
fn include_argument(after_bang: &str) -> Option<IncludeArg> {
    let args = after_bang.trim_start().strip_prefix('(')?.trim_start();

    if let Some(rest) = args.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            if end > 0 {
                return Some(IncludeArg::Literal(rest[..end].to_string()));
            }
        }
    }

    Some(IncludeArg::Opaque(
        args.trim_end_matches([')', ';']).chars().take(80).collect(),
    ))
}

/// Analyze multiple files and return aggregated complexity metrics
///
/// This helper function processes a list of files, maintaining consistency
/// with single file analysis and proper error handling for missing files.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_multiple_files(
    files: &[PathBuf],
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    crate::status_eprintln!("🔍 Analyzing complexity of {} files...", files.len());

    let mut all_metrics = Vec::new();
    for file_path in files {
        let full_path = if file_path.is_absolute() {
            file_path.clone()
        } else {
            config.project_path.join(file_path)
        };

        if !full_path.exists() {
            eprintln!("⚠️  Skipping missing file: {}", full_path.display());
            continue;
        }

        // Use same analyzer as single file mode (Issue #42 consistency)
        let file_content = std::fs::read_to_string(&full_path)
            .context(format!("Failed to read file: {}", full_path.display()))?;

        let metrics =
            crate::cli::language_analyzer::analyze_file_complexity(&full_path, &file_content)
                .await?;
        all_metrics.push(metrics);
    }

    Ok(all_metrics)
}

/// Analyze entire project directory based on toolchain detection
///
/// This helper function handles project-wide analysis with proper toolchain
/// detection and maintains the Issue #42 fix for multi-language projects.
pub(super) async fn analyze_project(
    detected_toolchain: Option<String>,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    // Auto-detection used to RESTRICT the walk to the one language it guessed.
    // A directory holding a.go, app.ts and main.py therefore reported
    // "Files analyzed: 1 / Total functions: 1" — whichever toolchain detection
    // happened to win that run — and printed the summary as if it covered the
    // project, with no hint that two of three source files were skipped.
    // Detection is only a label now; an explicit `--toolchain` still restricts.
    let explicit_toolchain = config.toolchain.as_deref();

    if let Some(toolchain) = explicit_toolchain {
        crate::status_eprintln!("🔍 Analyzing {toolchain} files only (--toolchain {toolchain})...");
        crate::cli::analysis_utilities::analyze_project_files(
            &config.project_path,
            Some(toolchain),
            &config.include,
            config.max_cyclomatic,
            config.max_cognitive,
        )
        .await
    } else {
        match detected_toolchain {
            Some(toolchain) => {
                crate::status_eprintln!(
                    "🔍 Analyzing {toolchain} project complexity (all languages)..."
                );
            }
            None => crate::status_eprintln!("🔍 Analyzing project complexity (multi-language)..."),
        }
        crate::cli::analysis_utilities::analyze_project_files(
            &config.project_path,
            None, // Analyze every supported language, not just the detected one
            &config.include,
            config.max_cyclomatic,
            config.max_cognitive,
        )
        .await
    }
}

/// Apply complexity threshold filtering to metrics
///
/// Filters files to only include those with functions exceeding the specified
/// cyclomatic or cognitive complexity thresholds.
///
/// Returns the count of files that were filtered out for better UX reporting.
pub(super) fn apply_complexity_filters(
    file_metrics: &mut Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> usize {
    if max_cyclomatic.is_none() && max_cognitive.is_none() {
        return 0;
    }

    let original_count = file_metrics.len();

    file_metrics.retain(|file| {
        file.functions.iter().any(|func| {
            let exceeds_cyclomatic =
                max_cyclomatic.is_some_and(|threshold| func.metrics.cyclomatic > threshold);
            let exceeds_cognitive =
                max_cognitive.is_some_and(|threshold| func.metrics.cognitive > threshold);
            exceeds_cyclomatic || exceeds_cognitive
        })
    });

    let filtered_count = original_count - file_metrics.len();

    if filtered_count > 0 {
        crate::status_eprintln!(
            "ℹ️  Filtered {} file(s) with no functions exceeding thresholds ({})",
            filtered_count,
            describe_thresholds(max_cyclomatic, max_cognitive)
        );
    }

    filtered_count
}

/// Name the thresholds that were actually in force.
///
/// An unset threshold used to be printed as its saturating sentinel —
/// "cognitive > 65535" — which reads as a real limit that no function can ever
/// exceed, and told the user a gate was running that was not. A threshold that
/// was never set is simply not named.
fn describe_thresholds(max_cyclomatic: Option<u16>, max_cognitive: Option<u16>) -> String {
    let mut in_force = Vec::new();
    if let Some(threshold) = max_cyclomatic {
        in_force.push(format!("cyclomatic > {threshold}"));
    }
    if let Some(threshold) = max_cognitive {
        in_force.push(format!("cognitive > {threshold}"));
    }
    if in_force.is_empty() {
        return "no thresholds set".to_string();
    }
    in_force.join(", ")
}

/// Aggregate over every analyzed file, then list only the top-N slice.
///
/// The summary and the list are built here together so they cannot drift: the
/// handler used to aggregate AFTER truncation and then overwrite
/// `summary.total_files` with the project count, which is how one unchanged
/// 1070-file tree reported `total_files: 1070` next to `total_functions: 159`
/// (true value 10148) and `technical_debt_hours: 388.75` (true 1644.25).
///
/// `analyzed` is consumed for the aggregate; `listed` is what the renderer
/// prints. See `contracts/pmat-no-fabrication-v1.yaml` — a cap must never be
/// presented as a total.
pub(super) fn build_report_over_analyzed_files(
    analyzed: Vec<FileComplexityMetrics>,
    listed: Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> crate::services::complexity::ComplexityReport {
    let analyzed_count = analyzed.len();
    let mut report = crate::services::complexity::aggregate_results_with_thresholds(
        analyzed,
        max_cyclomatic,
        max_cognitive,
    );
    report.summary.total_files = analyzed_count;
    report.files = listed;
    report
}

/// Apply top files limit by sorting and truncating results
///
/// Sorts files by total complexity (cyclomatic + cognitive) in descending order
/// and keeps only the top N most complex files.
pub(super) fn apply_top_files_limit(
    file_metrics: &mut Vec<FileComplexityMetrics>,
    top_files: usize,
) {
    if top_files > 0 && !file_metrics.is_empty() {
        // Sort files by complexity (descending)
        file_metrics.sort_by(|a, b| {
            let a_complexity =
                f64::from(a.total_complexity.cyclomatic) + f64::from(a.total_complexity.cognitive);
            let b_complexity =
                f64::from(b.total_complexity.cyclomatic) + f64::from(b.total_complexity.cognitive);
            b_complexity
                .partial_cmp(&a_complexity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only top N files
        file_metrics.truncate(top_files);
    }
}

/// Analyze files based on the specified mode (single, multiple, or project)
///
/// The banner below used to be the ENTIRETY of `--timeout`: nothing in this
/// module ever built a `Duration` from `config.timeout`, so `analyze complexity
/// -p . --timeout 1` printed "⏰ Analysis timeout set to 1 seconds", ran for
/// 8.1s and exited 0. Same shape as #929 in `analyze dead-code`, and the fix is
/// deliberately not a third private copy: the budget is enforced by
/// `run_within_analysis_budget`, which lives beside the `--timeout` flag
/// declarations that promise it.
pub(super) async fn analyze_files_by_mode(
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    crate::status_eprintln!("⏰ Analysis timeout set to {} seconds", config.timeout);

    // Owned so the work can run on its own task: a budget enforced from the
    // caller's own task is the non-enforcement it replaces (see the helper).
    let owned = config.clone();
    let result = crate::cli::commands::analyze_commands::run_within_analysis_budget(
        "Complexity analysis",
        config.timeout,
        async move { analyze_by_mode(file, files, &owned).await },
    )
    .await;

    // Provide feedback on analysis results
    match &result {
        Ok(metrics) if metrics.is_empty() => {
            eprintln!("\n⚠️  Warning: No files were found or analyzed");
            eprintln!("   Possible reasons:");
            eprintln!("   - Directory is empty or contains no supported file types");
            eprintln!("   - Files are excluded by .gitignore patterns");
            eprintln!("   - Include patterns don't match any files");
            if !config.include.is_empty() {
                eprintln!("   - Current include patterns: {:?}", config.include);
            }
            eprintln!();
        }
        Ok(metrics) => {
            crate::status_eprintln!("✅ Successfully analyzed {} file(s)", metrics.len());
        }
        Err(_) => {
            // Error will be returned and handled by caller
        }
    }

    result
}

/// Pick the analysis the flags asked for. Split out of `analyze_files_by_mode`
/// so the whole of it — mode selection included — sits inside the budget.
async fn analyze_by_mode(
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    if let Some(single_file) = file {
        analyze_single_file(&single_file, config).await
    } else if !files.is_empty() {
        analyze_multiple_files(&files, config).await
    } else {
        let detected_toolchain = config.detect_toolchain();
        analyze_project(detected_toolchain, config).await
    }
}

/// Check for complexity violations and exit if required
pub(super) fn check_complexity_violations(
    file_metrics: &[FileComplexityMetrics],
    fail_on_violation: bool,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) {
    if !fail_on_violation {
        return;
    }

    let has_violations = has_complexity_violations(file_metrics, max_cyclomatic, max_cognitive);

    if has_violations {
        eprintln!("\n❌ Complexity violations found");
        std::process::exit(1);
    }
}

/// Check if any files have complexity violations
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn has_complexity_violations(
    file_metrics: &[FileComplexityMetrics],
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> bool {
    file_metrics.iter().any(|file| {
        file.functions.iter().any(|func| {
            let cyclomatic_exceeded = func.metrics.cyclomatic > max_cyclomatic.unwrap_or(20);
            let cognitive_exceeded = func.metrics.cognitive > max_cognitive.unwrap_or(15);
            cyclomatic_exceeded || cognitive_exceeded
        })
    })
}

#[cfg(test)]
mod multi_language_tests {
    //! Regression tests for two defects in this module: a detected toolchain
    //! silently restricted the project walk to one language, and an unset
    //! threshold was reported as the unreachable sentinel 65535.
    use super::{analyze_project, describe_thresholds};
    use crate::cli::handlers::complexity_handlers::ComplexityConfig;

    fn write_polyglot(dir: &std::path::Path) {
        std::fs::write(
            dir.join("a.go"),
            "package main\nfunc Add(a int, b int) int { if a > b { return a }\n return b }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("app.ts"),
            "export function add(a: number, b: number): number { return a > b ? a : b; }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("main.py"),
            "def add(a, b):\n    if a > b:\n        return a\n    return b\n",
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_detected_toolchain_does_not_drop_other_languages() {
        let temp = tempfile::TempDir::new().unwrap();
        write_polyglot(temp.path());

        // No `--toolchain` flag: detection may name any one language, but it
        // must not become the whole project.
        let config = ComplexityConfig::from_args(
            temp.path().to_path_buf(),
            None,
            None,
            None,
            Vec::new(),
            60,
            0,
        );
        let metrics = analyze_project(Some("typescript".to_string()), &config)
            .await
            .unwrap();

        let mut extensions: Vec<String> = metrics
            .iter()
            .filter_map(|m| {
                std::path::Path::new(&m.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_string)
            })
            .collect();
        extensions.sort();
        extensions.dedup();

        assert!(
            extensions.len() >= 2,
            "detecting one toolchain must not restrict the walk to it; analyzed {extensions:?}"
        );
    }

    #[tokio::test]
    async fn test_explicit_toolchain_still_restricts() {
        let temp = tempfile::TempDir::new().unwrap();
        write_polyglot(temp.path());

        let config = ComplexityConfig::from_args(
            temp.path().to_path_buf(),
            Some("go".to_string()),
            None,
            None,
            Vec::new(),
            60,
            0,
        );
        let metrics = analyze_project(Some("go".to_string()), &config)
            .await
            .unwrap();

        assert!(
            metrics.iter().all(|m| m.path.ends_with(".go")),
            "--toolchain go must analyze only Go files"
        );
    }

    #[test]
    fn test_unset_threshold_is_not_reported_as_65535() {
        let described = describe_thresholds(Some(20), None);
        assert_eq!(described, "cyclomatic > 20");
        assert!(
            !described.contains("65535"),
            "an unset cognitive threshold must not be printed as u16::MAX"
        );
        assert_eq!(
            describe_thresholds(Some(20), Some(15)),
            "cyclomatic > 20, cognitive > 15"
        );
    }
}

#[cfg(test)]
mod include_expansion_tests {
    //! Regression tests for #702 — `analyze complexity --file X.rs` reported a
    //! function count that covered only X.rs's own body when X.rs pulls most of
    //! its code in with `include!()`. Observed on this repo:
    //! `--file src/cli/handlers/lint_hotspot_handlers/clippy.rs` returned
    //! `"total_functions": 4` while a directory scan of the same code found 4 +
    //! 15 + 16 across clippy.rs and its two included fragments.
    use super::ComplexityConfig;
    use super::{analyze_included_fragments, analyze_single_file, scan_rust_includes};

    /// The defect as the user meets it: `--file parent.rs` must not report a
    /// function count that stops at the includer's own body.
    #[tokio::test]
    async fn test_single_file_mode_counts_functions_pulled_in_by_include() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("frag.rs"),
            "fn frag_one() -> u32 { 1 }\nfn frag_two(x: u32) -> u32 { if x > 0 { 1 } else { 0 } }\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("parent.rs"),
            "include!(\"frag.rs\");\nfn parent_one() -> u32 { 2 }\n",
        )
        .unwrap();

        let config = ComplexityConfig::from_args(
            dir.path().to_path_buf(),
            None,
            None,
            None,
            Vec::new(),
            60,
            0,
        );
        let metrics = analyze_single_file(std::path::Path::new("parent.rs"), &config)
            .await
            .unwrap();

        let counted: usize = metrics.iter().map(|m| m.functions.len()).sum();
        assert_eq!(
            counted,
            3,
            "PRE-FIX this was 1: only parent.rs's own body was measured, and the \
             two functions its include!() pulls in were reported to nobody. Got {:?}",
            metrics
                .iter()
                .map(|m| (m.path.clone(), m.functions.len()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_scan_finds_include_targets() {
        let (targets, opaque) = scan_rust_includes(
            "use std::io;\ninclude!(\"clippy_parsing.rs\");\ninclude!( \"sub/dir.rs\" );\n",
        );
        assert_eq!(targets, vec!["clippy_parsing.rs", "sub/dir.rs"]);
        assert!(opaque.is_empty(), "{opaque:?}");
    }

    #[test]
    fn test_scan_ignores_lookalikes_and_comments() {
        let (targets, opaque) = scan_rust_includes(
            "let s = include_str!(\"x.txt\");\n\
             let b = include_bytes!(\"y.bin\");\n\
             // include!(\"commented_out.rs\");\n\
             my_include!(\"other.rs\");\n",
        );
        assert!(targets.is_empty(), "{targets:?}");
        assert!(opaque.is_empty(), "{opaque:?}");
    }

    #[test]
    fn test_scan_reports_an_unresolvable_include_instead_of_dropping_it() {
        // A generated include cannot be resolved from source alone; it must be
        // REPORTED, because a short function list otherwise reads as complete.
        let (targets, opaque) =
            scan_rust_includes("include!(concat!(env!(\"OUT_DIR\"), \"/gen.rs\"));\n");
        assert!(targets.is_empty(), "{targets:?}");
        assert_eq!(opaque.len(), 1, "{opaque:?}");
        assert!(opaque[0].contains("concat!"), "{opaque:?}");
    }

    #[tokio::test]
    async fn test_included_fragments_are_analyzed_not_skipped() {
        // PRE-FIX this returned nothing: the includer's 1 function was the whole
        // report and the fragment's 2 functions were invisible.
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("frag.rs"),
            "fn frag_one() -> u32 { 1 }\nfn frag_two(x: u32) -> u32 { if x > 0 { 1 } else { 0 } }\n",
        )
        .unwrap();
        let parent = dir.path().join("parent.rs");
        std::fs::write(
            &parent,
            "include!(\"frag.rs\");\nfn parent_one() -> u32 { 2 }\n",
        )
        .unwrap();

        let fragments = analyze_included_fragments(&parent).await.unwrap();
        assert_eq!(fragments.len(), 1, "the included fragment must be analyzed");
        assert!(
            fragments[0].path.ends_with("frag.rs"),
            "a fragment keeps its own path so its line numbers stay truthful: {:?}",
            fragments[0].path
        );
        assert_eq!(
            fragments[0].functions.len(),
            2,
            "both functions in the fragment must be counted: {:?}",
            fragments[0].functions
        );
    }

    #[tokio::test]
    async fn test_include_cycle_terminates() {
        let dir = tempfile::TempDir::new().unwrap();
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "include!(\"b.rs\");\nfn a_one() {}\n").unwrap();
        std::fs::write(&b, "include!(\"a.rs\");\nfn b_one() {}\n").unwrap();

        let fragments = analyze_included_fragments(&a).await.unwrap();
        assert_eq!(fragments.len(), 1, "a.rs must not be analyzed twice");
    }

    #[tokio::test]
    async fn test_a_file_without_includes_gains_nothing() {
        let dir = tempfile::TempDir::new().unwrap();
        let solo = dir.path().join("solo.rs");
        std::fs::write(&solo, "fn only() {}\n").unwrap();
        assert!(analyze_included_fragments(&solo).await.unwrap().is_empty());
    }
}

#[cfg(test)]
mod timeout_is_a_bound_tests {
    //! `analyze complexity --timeout N` printed "⏰ Analysis timeout set to N
    //! seconds" and enforced nothing: measured at HEAD 1ac9feb5a,
    //! `pmat analyze complexity -p . --timeout 1` walked 4400 files in 8.1s and
    //! exited 0. Same shape as #929 in `analyze dead-code`.
    use super::analyze_files_by_mode;
    use crate::cli::handlers::complexity_handlers::ComplexityConfig;
    use std::path::{Path, PathBuf};

    fn config_for(path: PathBuf, timeout: u64) -> ComplexityConfig {
        ComplexityConfig::from_args(path, None, None, None, vec![], timeout, 10)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_walk_that_outruns_the_budget_fails_instead_of_reporting_success() {
        // This crate's own src/ tree — the measurement above. One second cannot
        // buy 8.1 seconds of walking, so the ONLY honest outcomes are an error
        // naming the budget or (if this machine were ~10x faster) a complete
        // result inside 1s; a result that took longer than the budget is the
        // defect. Elapsed is asserted so a future "always Ok" cannot pass.
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        assert!(src.is_dir(), "fixture is this crate's own source tree");

        let started = std::time::Instant::now();
        let err = analyze_files_by_mode(None, vec![], &config_for(src, 1))
            .await
            .expect_err("a 1s budget must not report success for an 8s walk");
        let elapsed = started.elapsed();

        assert!(
            err.to_string().contains("timed out after 1 seconds"),
            "the error must name the budget the banner promised, got: {err}"
        );
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "the budget must actually cut the walk short, took {elapsed:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_budget_that_is_ample_still_returns_the_analysis() {
        // The other half of the contract: enforcement must not turn every run
        // into a timeout.
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("small.rs"),
            "fn one(a: bool) -> u8 { if a { 1 } else { 2 } }\n",
        )
        .unwrap();

        let metrics = analyze_files_by_mode(None, vec![], &config_for(dir.path().into(), 300))
            .await
            .expect("one small file cannot exhaust a 300s budget");
        assert_eq!(metrics.len(), 1, "the one file must still be analyzed");
    }
}
