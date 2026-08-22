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
            // What the walk SAW, not a list of things it might have been.
            //
            // This printed three guesses — "Directory is empty or contains no
            // supported file types", ".gitignore patterns", "Include patterns
            // don't match" — above the error that actually names the cause. The
            // reader hits the speculation first, and on a directory of COBOL it
            // sends them to check their .gitignore.
            //
            // Every one of those guesses is answerable. The walk knows how many
            // files it saw and with which extensions, so it says that, and the
            // include patterns are only mentioned when some were supplied.
            eprintln!(
                "\n⚠️  No files were analyzed under {}",
                config.project_path.display()
            );
            let census = unanalyzed_summary(metrics, config);
            match &census {
                Some(c) => eprintln!("{}", c.note),
                None => eprintln!("   the walk found no files with a file extension"),
            }
            if !config.include.is_empty() {
                eprintln!("   include patterns in effect: {:?}", config.include);
            }
            eprintln!();
            // ...and REFUSE. Printing the diagnosis and returning Ok(vec![]) is
            // the defect this whole branch documents, one level up: a caller
            // that checks the Result sees success, and a zero-length report
            // reads exactly like a clean tree.
            //
            // The clean room found it. `--timeout 1` over this crate's own src/
            // returned `Ok([])` inside the budget having produced metrics for
            // none of 3,991 supported .rs files, so the timeout test got a third
            // outcome its author had not enumerated — neither the timeout error
            // nor a complete result, but an empty success. Locally the same walk
            // takes ~8s and errors correctly, which is why only a slower machine
            // could surface it.
            //
            // Scoped deliberately: this fires only when the walk produced NO
            // metrics at all. A partial result is still Ok, because a tree pmat
            // can read half of is a real report about that half — that is what
            // `unanalyzed_summary` is for, and the Ok branch above prints it.
            // The wording `analyze satd` established, via the shared
            // constructor, so the eight refusals cannot drift apart. It has two
            // branches and they are different events: nothing was there, versus
            // everything that was there was skipped.
            let discovered = census.as_ref().map_or(0, |c| c.total);
            return Err(crate::cli_exit::analysis_error(anyhow::anyhow!(
                crate::services::defect_detector::unmeasured::refusal(
                    "complexity",
                    &config.project_path,
                    discovered,
                    "none produced metrics",
                    "check that the files parse, or raise --timeout if the walk \
                     was cut short",
                )
            )));
        }
        Ok(metrics) => {
            crate::status_eprintln!("✅ Successfully analyzed {} file(s)", metrics.len());
            if let Some(c) = unanalyzed_summary(metrics, config) {
                crate::status_eprintln!("{}", c.note);
            }
        }
        Err(_) => {
            // Error will be returned and handled by caller
        }
    }

    result
}

/// Pick the analysis the flags asked for. Split out of `analyze_files_by_mode`
/// so the whole of it — mode selection included — sits inside the budget.
/// Name the files the walk saw and did not analyze.
///
/// `✅ Successfully analyzed 1 file(s)` is a count with no denominator. Point
/// pmat at a directory holding one `.rs` and eight `.cbl`, and that is exactly
/// what it printed — a confident green, with no hint that eight of nine files
/// were never read. A COBOL codebase with one stray Rust file gets a clean bill
/// of health for the Rust file alone.
///
/// The comment on `analyze_project` already records this defect for a different
/// cause: toolchain auto-detection used to restrict the walk, so "a directory
/// holding a.go, app.ts and main.py therefore reported Files analyzed: 1". That
/// was fixed. The same silence for a file type pmat has no analyzer for was not.
///
/// UNSUPPORTED IS DERIVED, NOT LISTED. The set is "present in the walk, absent
/// from the results" — so this cannot drift from whatever the analyzers actually
/// handle, and it adds no fourth copy of a language list to a repository that
/// already has several that disagree.
/// What the walk saw and did not analyse: the human note, plus the two counts
/// behind it.
///
/// The counts are returned rather than only rendered because the refusal above
/// needs them and must not walk the tree a second time to get them — two walks
/// are two chances to disagree, which is the defect the single-derivation
/// comment below already records.
#[derive(Debug)]
pub(super) struct UnanalyzedCensus {
    pub note: String,
    /// Files with an extension the walk saw, analysed or not.
    pub total: usize,
    /// Of those, how many produced no metrics.
    pub missing: usize,
}

fn unanalyzed_summary(
    metrics: &[FileComplexityMetrics],
    config: &ComplexityConfig,
) -> Option<UnanalyzedCensus> {
    use crate::services::ast::strategy::StrategySelector;
    use std::collections::{BTreeMap, HashSet};

    // Single-file and explicit-file modes have no population to compare against.
    if !config.project_path.is_dir() {
        return None;
    }

    // `m.path` is relative to the PROCESS CWD, not to `project_path`. Joining it
    // onto `project_path` built `src/tdg/src/tdg/foo.rs` — which canonicalizes to
    // nothing — so `analyzed` came back EMPTY and every file in the walk was
    // counted as unanalyzed. Try the path as given first, then as a child of the
    // scanned root, and keep whichever actually resolves.
    let resolve = |p: &Path| -> Option<PathBuf> {
        std::fs::canonicalize(p)
            .or_else(|_| std::fs::canonicalize(config.project_path.join(p)))
            .ok()
    };
    let analyzed: HashSet<PathBuf> = metrics
        .iter()
        .filter_map(|m| resolve(Path::new(&m.path)))
        .collect();

    // Which extensions pmat can analyse is not a guess: `supported_extensions`
    // is compiled under the same feature flags as this binary, so it cannot
    // drift from what the build can actually parse.
    let supported: HashSet<String> = StrategySelector::supported_extensions()
        .into_iter()
        .map(str::to_ascii_lowercase)
        .collect();

    let mut no_analyzer: BTreeMap<String, usize> = BTreeMap::new();
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0usize;
    for entry in ignore::WalkBuilder::new(&config.project_path)
        .hidden(true)
        .git_ignore(true)
        .build()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let ext = ext.to_ascii_lowercase();
        total += 1;
        if resolve(entry.path()).is_some_and(|c| analyzed.contains(&c)) {
            continue;
        }
        if supported.contains(&ext) {
            *skipped.entry(ext).or_default() += 1;
        } else {
            *no_analyzer.entry(ext).or_default() += 1;
        }
    }

    // ONE derivation of the count. It used to be `total - metrics.len()` while
    // the census was built separately from the walk, so when the path match
    // broke the two contradicted each other inside a single sentence:
    //   "0 of 220 file(s) were not analyzed — pmat has no complexity analyzer
    //    for: .rs (220)"
    // Both halves now come from the same tally, which cannot disagree with
    // itself, and a wrong tally shows up as a wrong number instead of hiding
    // behind a plausible zero.
    let missing: usize = no_analyzer.values().chain(skipped.values()).sum();
    if missing == 0 {
        return None;
    }

    let census = |m: &BTreeMap<String, usize>| -> String {
        m.iter()
            .map(|(e, n)| format!(".{e} ({n})"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = format!("   {missing} of {total} file(s) were not analyzed");
    if !no_analyzer.is_empty() {
        out.push_str(&format!(
            "\n   no complexity analyzer for: {}",
            census(&no_analyzer)
        ));
    }
    // A supported extension that still produced no metrics is a DIFFERENT fact
    // from an unsupported one — reporting it as "no analyzer" told users to stop
    // expecting Rust support because three files failed to parse.
    if !skipped.is_empty() {
        out.push_str(&format!(
            "\n   supported, but no metrics were produced: {}",
            census(&skipped)
        ));
    }
    Some(UnanalyzedCensus {
        note: out,
        total,
        missing,
    })
}

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

    /// A tree whose files pmat cannot analyse is REFUSED, not returned empty.
    ///
    /// The clean room found this one. `analyze complexity --timeout 1` over this
    /// crate's own `src/` came back `Ok([])` inside the budget having produced
    /// metrics for none of 3,991 supported `.rs` files, so the timeout test above
    /// got a third outcome its author had not enumerated: neither the timeout
    /// error nor a complete result, but an EMPTY SUCCESS. Locally the same walk
    /// takes ~8s and errors correctly, which is why only a slower machine could
    /// surface it.
    ///
    /// `analyze_files_by_mode` printed a careful diagnosis of what it had failed
    /// to analyse and then returned `Ok(vec![])`. A caller that checks the
    /// `Result` sees success, and a zero-length report reads exactly like a clean
    /// tree — the defect the diagnosis itself documents, one level up.
    ///
    /// The wording comes from the shared `unmeasured::refusal` constructor, the
    /// one `analyze satd` established, so the refusals cannot drift apart.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_tree_that_yields_no_metrics_is_refused_not_returned_empty() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        for n in 0..3 {
            std::fs::write(
                dir.path().join(format!("legacy{n}.cbl")),
                "IDENTIFICATION DIVISION.\n",
            )
            .expect("write cobol fixture");
        }

        let err = analyze_files_by_mode(None, vec![], &config_for(dir.path().into(), 60))
            .await
            .expect_err("a tree pmat can analyse no file of is not a clean zero");
        let message = format!("{err:#}");

        assert!(
            message.contains("This is not a clean result"),
            "the refusal must say the zero is not clean, got: {message}"
        );
        assert!(
            message.contains(&dir.path().display().to_string()),
            "the refusal must name the path it refused, got: {message}"
        );
        // discovered > 0, so it must be the "all N were skipped" branch and not
        // the "nothing was there" one. These are different events.
        assert!(
            message.contains("were skipped"),
            "three files WERE found; the refusal must not claim the tree was \
             empty, got: {message}"
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

#[cfg(test)]
mod unanalyzed_tests {
    /// The note alone, which is all these tests assert on.
    ///
    /// `unanalyzed_summary` returns the two counts as well, because the
    /// refusal in `analyze_files_by_mode` needs them and must not walk the
    /// tree a second time to get them.
    fn unanalyzed_note(
        metrics: &[FileComplexityMetrics],
        config: &ComplexityConfig,
    ) -> Option<String> {
        super::unanalyzed_summary(metrics, config).map(|c| c.note)
    }

    use super::*;
    use crate::services::complexity::types::ComplexityMetrics;

    fn cfg(dir: &std::path::Path) -> ComplexityConfig {
        ComplexityConfig::from_args(dir.to_path_buf(), None, None, None, vec![], 300, 10)
    }

    fn metric(path: &str) -> FileComplexityMetrics {
        FileComplexityMetrics {
            path: path.to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    /// The count and the census are one tally, on the path shape PRODUCTION uses.
    ///
    /// `m.path` is relative to the process CWD, not to the scanned root. The
    /// first version of this function joined it onto `project_path` — building
    /// `src/tdg/src/tdg/foo.rs`, which resolves to nothing — so the analyzed set
    /// came back empty and every file was counted as skipped. It went unseen
    /// because the count was derived a SECOND way (`total - metrics.len()`),
    /// which still read 0, so `pmat analyze complexity --path src/tdg` printed
    /// one self-contradicting sentence:
    ///
    ///     0 of 220 file(s) were not analyzed — pmat has no complexity analyzer
    ///     for: .rs (220)
    ///
    /// The tests above missed it entirely by passing `"ok.rs"` — relative to the
    /// temp dir, a shape the walker never emits. The fixture agreed with the
    /// author instead of with production. This one builds the root under the CWD
    /// and names the file the way the walker really does.
    #[test]
    fn the_count_and_the_census_are_one_tally() {
        let dir = tempfile::TempDir::with_prefix_in("pmat-unanalyzed-", ".").expect("temp dir");
        // RELATIVE, like `--path src/tdg`. `dir.path()` is absolute, and
        // `Path::join(absolute)` returns the absolute unchanged — which silently
        // repairs the very bug under test, so an absolute fixture proves nothing.
        let root = std::path::PathBuf::from(".").join(dir.path().file_name().expect("dir name"));
        std::fs::write(root.join("ok.rs"), "fn main(){}\n").expect("write rs");
        for ext in ["cbl", "f90", "vhd"] {
            std::fs::write(root.join(format!("thing.{ext}")), "x\n").expect("write");
        }

        // Exactly what the walker hands back: the root as given, plus the child.
        let as_walked = format!("{}/ok.rs", root.display());
        let note = unanalyzed_note(&[metric(&as_walked)], &cfg(&root))
            .expect("three unanalyzable files must be reported");

        let stated: usize = note
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or_default();
        assert!(stated > 0, "no leading count in: {note}");
        let counted: usize = note
            .match_indices('(')
            .filter_map(|(i, _)| note[i + 1..].split(')').next())
            .filter_map(|n| n.parse::<usize>().ok())
            .sum();
        assert_eq!(
            stated, counted,
            "the number and the per-extension census disagree: {note}"
        );
        assert_eq!(
            stated, 3,
            "one .rs was analyzed, three files were not: {note}"
        );
    }

    /// A supported extension is never reported as unsupported.
    ///
    /// The broken path match put all 220 analyzed `.rs` files into the skipped
    /// census, so pmat told a Rust project it had "no complexity analyzer for
    /// .rs" — while printing the metrics it had just computed for them.
    #[test]
    fn a_supported_extension_is_never_called_unsupported() {
        let dir = tempfile::TempDir::with_prefix_in("pmat-supported-", ".").expect("temp dir");
        // Relative for the same reason as above.
        let root = std::path::PathBuf::from(".").join(dir.path().file_name().expect("dir name"));
        std::fs::write(root.join("ok.rs"), "fn main(){}\n").expect("write rs");
        std::fs::write(root.join("thing.cbl"), "x\n").expect("write cbl");

        let as_walked = format!("{}/ok.rs", root.display());
        let note =
            unanalyzed_note(&[metric(&as_walked)], &cfg(&root)).expect("the .cbl must be reported");

        let claim = note
            .lines()
            .find(|l| l.contains("no complexity analyzer for"))
            .unwrap_or_default();
        assert!(!claim.is_empty(), "expected an unsupported line in: {note}");
        assert!(
            !claim.contains(".rs"),
            "pmat analyzed the .rs and still called it unsupported: {note}"
        );
        assert!(claim.contains(".cbl"), "the .cbl must be named: {note}");
    }

    /// A file type pmat cannot analyze must be NAMED, not silently dropped.
    ///
    /// `✅ Successfully analyzed 1 file(s)` is a count with no denominator: one
    /// `.rs` beside eight `.cbl` printed exactly that, and a COBOL codebase with
    /// one stray Rust file got a clean bill of health for the Rust file alone.
    #[test]
    fn unanalyzed_file_types_are_named_with_a_denominator() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("ok.rs"), "fn main(){}\n").expect("write rs");
        for ext in ["cbl", "f90", "vhd"] {
            std::fs::write(dir.path().join(format!("thing.{ext}")), "x\n").expect("write");
        }

        let note = unanalyzed_note(&[metric("ok.rs")], &cfg(dir.path()))
            .expect("skipped files must be reported");

        assert!(note.contains("3 of 4"), "needs a denominator, got: {note}");
        for ext in [".cbl", ".f90", ".vhd"] {
            assert!(note.contains(ext), "{ext} must be named, got: {note}");
        }
    }

    /// The counter-test. A tree pmat fully analyzed must produce NO note —
    /// otherwise the fix is a nag on every clean run, and a warning everyone
    /// learns to ignore is worse than the silence it replaced.
    #[test]
    fn a_fully_analyzed_tree_is_not_nagged() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("a.rs"), "fn a(){}\n").expect("write a");
        std::fs::write(dir.path().join("b.rs"), "fn b(){}\n").expect("write b");

        assert_eq!(
            unanalyzed_note(&[metric("a.rs"), metric("b.rs")], &cfg(dir.path())),
            None,
            "a fully analyzed tree must produce no note"
        );
    }

    /// Single-file mode has no population to compare against, so it must not
    /// invent one from the file's parent directory.
    #[test]
    fn single_file_mode_reports_nothing() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let file = dir.path().join("only.rs");
        std::fs::write(&file, "fn f(){}\n").expect("write");
        std::fs::write(dir.path().join("other.cbl"), "x\n").expect("write");

        let c = ComplexityConfig::from_args(file, None, None, None, vec![], 300, 10);
        assert_eq!(unanalyzed_note(&[metric("only.rs")], &c), None);
    }
}
