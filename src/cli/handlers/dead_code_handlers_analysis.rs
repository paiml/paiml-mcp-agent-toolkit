// Dead code analysis logic - included from dead_code_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

/// Dead code analysis output: the report, plus the one figure that describes
/// the PROJECT rather than the reported list.
///
/// `report.summary.dead_percentage` is scoped to the files actually listed —
/// it has to be, a count must agree with the list it heads — so it is not a
/// number a threshold can be enforced against: it shrinks as `--top-files`
/// asks for fewer files. `project_dead_percentage` is measured over every line
/// walked, before `--min-dead-lines` and `--top-files` cut the list down, and
/// is `None` when the analyzer cannot measure one.
struct DeadCodeAnalysisOutcome {
    report: crate::models::dead_code::DeadCodeResult,
    project_dead_percentage: Option<f32>,
    scope: DeadCodeReportScope,
    /// Which of the two engines below produced `report`.
    ///
    /// Set HERE, at the two places that actually construct an outcome, rather
    /// than re-derived by a caller: a second caller asking
    /// `detect_project_language_enhanced` for itself would be a second copy of
    /// the dispatch decision, and the whole point of [`run_dead_code_suite`] is
    /// that one name means one analyzer. `analyze_dead_code`'s MCP payload
    /// publishes this so a client can see which engine answered.
    engine: &'static str,
    /// The language actually READ, for the same reason and set in the same two
    /// places as `engine`.
    ///
    /// Not a synonym for the engine: `multi-language-reachability` reads ONE
    /// language per project and skips every other source file in the tree, so
    /// without this a client cannot tell "no dead Python" from "the Python was
    /// never opened". Nor is it the DETECTED language — the analyzer falls back
    /// to a language that is actually present when the dominant one has no
    /// strategy, and what a reader needs is the one that was read.
    language: String,
    /// Functions the engine walked: the denominator for the dead-function
    /// count, and `None` where the engine does not measure one.
    ///
    /// `None`, never `0`: a zero here reads as "this tree has no functions".
    /// The cargo engine reports what rustc's dead-code pass found dead and
    /// never enumerates the live ones, and counting them some other way would
    /// measure a different file set than the findings it heads (`cargo check`
    /// skips the test, example and bench targets), which is a ratio that is
    /// quietly wrong rather than honestly absent.
    total_functions: Option<usize>,
}

/// `run_dead_code_analysis_with_filters`'s Rust engine: rustc's own dead-code
/// pass, read out of `cargo check`.
pub(crate) const DEAD_CODE_ENGINE_CARGO: &str = "cargo-dead-code";

/// Its engine for every other language: reachability over a parsed call graph.
pub(crate) const DEAD_CODE_ENGINE_MULTI_LANGUAGE: &str = "multi-language-reachability";

/// Everything a report FOUND, in the units its summary counts in.
///
/// #928 EXTENSION. The invariant that fix established — *nothing may be listed
/// that no counter accounts for* — has a mirror image, and the mirror image is
/// what shipped: **nothing may be FOUND that no counter accounts for.** The
/// summary is derived from the listed files (it has to be; a count must agree
/// with the list it heads), so every filter that removes a file also removes its
/// items from every category counter. The only trace left in the payload was
/// `files_with_dead_code_found`, a bare file count sitting next to a
/// `files_with_dead_code: 0` it contradicts, in the same object, with nothing
/// saying why or how much was lost.
///
/// These are the same six figures the summary reports, measured over the files
/// the analyzer found, so `listed + omitted = found` holds field by field and no
/// zero in the summary can stand for a finding that was silently dropped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DeadCodeFindingTotals {
    files: usize,
    dead_lines: usize,
    dead_functions: usize,
    dead_classes: usize,
    dead_modules: usize,
    unreachable_blocks: usize,
}

impl DeadCodeFindingTotals {
    /// Aggregate the rows exactly as `resummarize_from_listed_files` does, so
    /// the found totals and the summary are the same arithmetic over different
    /// sets rather than two estimators that can drift.
    fn of(files: &[crate::models::dead_code::FileDeadCodeMetrics]) -> Self {
        Self {
            files: files.len(),
            dead_lines: files.iter().map(|f| f.dead_lines).sum(),
            dead_functions: files.iter().map(|f| f.dead_functions).sum(),
            dead_classes: files.iter().map(|f| f.dead_classes).sum(),
            dead_modules: files.iter().map(|f| f.dead_modules).sum(),
            unreachable_blocks: files.iter().map(|f| f.unreachable_blocks).sum(),
        }
    }

    /// What `found` holds that `listed` does not.
    ///
    /// Saturating because a subtraction that underflows would panic in debug and
    /// wrap into a colossal "omitted" figure in release; a listed set larger than
    /// the found set is a bug in the caller, and this must not turn it into a
    /// fabricated count.
    fn minus(self, listed: Self) -> Self {
        Self {
            files: self.files.saturating_sub(listed.files),
            dead_lines: self.dead_lines.saturating_sub(listed.dead_lines),
            dead_functions: self.dead_functions.saturating_sub(listed.dead_functions),
            dead_classes: self.dead_classes.saturating_sub(listed.dead_classes),
            dead_modules: self.dead_modules.saturating_sub(listed.dead_modules),
            unreachable_blocks: self
                .unreachable_blocks
                .saturating_sub(listed.unreachable_blocks),
        }
    }

    /// True when nothing at all is being accounted for.
    fn is_empty(self) -> bool {
        self == Self::default()
    }
}

/// What the summary renderer needs in order to describe its own scope, and
/// which `DeadCodeResult` itself does not carry.
///
/// Every field exists because the report made a claim it could not stand
/// behind: the skipped-file line named "tests, examples and benches" on a scan
/// that skips only tests, the omission line blamed `--min-dead-lines` for files
/// `--exclude` had removed, and the percentage was printed unqualified next to a
/// gate that then refused to measure one.
#[derive(Clone, Copy, Default)]
struct DeadCodeReportScope {
    /// What the report's own filters removed from the list, in the summary's
    /// units. `files_with_dead_code_found` said only how many FILES were lost;
    /// a consumer reading `dead_functions: 0` had no way to learn that three
    /// dead functions had been cut from underneath it.
    omitted: DeadCodeFindingTotals,
    /// Dead-code percentage over every line the analyzer walked, `None` when it
    /// cannot measure one. The summary's own percentage covers only the files it
    /// LISTS, so unqualified it read as a project figure.
    project_dead_percentage: Option<f32>,
    /// What the `total_files - analyzed_files` difference is, in the analyzer's
    /// own terms. `None` when the analyzer did not say.
    skipped_kind: Option<&'static str>,
    /// True when `--include`/`--exclude` removed files from the reported list.
    /// They filter the REPORT, not the scan.
    list_filtered: bool,
}

/// Run dead code analysis with include/exclude filters, inside `budget`.
async fn run_dead_code_analysis_with_filters(
    path: &Path,
    filters: DeadCodeAnalysisFilters,
    budget: std::time::Duration,
) -> Result<DeadCodeAnalysisOutcome> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::utils::file_filter::FileFilter;

    // Detect project language to choose the right analyzer
    let detection =
        crate::services::enhanced_language_detection::detect_project_language_enhanced(path);

    // For non-Rust projects, use the multi-language analyzer
    if detection.language != "rust" {
        return run_multi_language_dead_code_within(path, filters, detection.language, budget)
            .await;
    }

    // WHICH CRATE cargo will be asked to compile, decided before anything is
    // measured. rustc's dead-code pass is the only engine this command has for
    // Rust, and rustc cannot type-check less than a whole crate: pointed at a
    // subdirectory it needs the crate above it, and pointed at a tree with no
    // crate at all it has nothing to run.
    //
    // Refusing is the only honest answer to the second case. It used to publish
    // whatever cargo's failure left behind, which on a lib-only crate seen from
    // a subdirectory was `cargo check --bins` matching no target, compiling
    // nothing, and a report of zeros at exit 0.
    //
    // A WORKSPACE ROOT is the second way to have no crate, and it is the one
    // that looks like having one: `[workspace]` with no `[package]` is a
    // Cargo.toml that declares no crate, so stopping the walk there found no
    // `[lib]` and no `src/lib.rs` — a virtual manifest has neither by
    // definition — dropped `--lib`, and `cargo check --bins` over a workspace of
    // libraries matched no target and compiled nothing. Same zero, same exit 0,
    // under a `library_target` calling the workspace root "a binary-only crate".
    let resolution = crate::services::cargo_dead_code_analyzer::resolve_crate_root(path);
    let cargo_root = match resolution.package_root() {
        Some(root) => root.to_path_buf(),
        None => anyhow::bail!(
            "{} holds Rust sources but is inside no cargo PACKAGE: {}. Dead code in Rust \
             is measured by rustc, which needs a crate to compile, so no dead-code \
             measurement was taken. This is not a clean result. {}",
            path.display(),
            resolution
                .no_package_reason(path)
                .unwrap_or_else(|| "no package encloses it".to_string()),
            match &resolution {
                crate::services::cargo_dead_code_analyzer::CrateRootResolution::WorkspaceOnly {
                    ..
                } =>
                    "Point --path at one of those member crates, or at a directory inside \
                      one: a workspace is not a compilation unit, so there is nothing to \
                      measure at its root.",
                _ => "Point --path at a cargo crate, or at a directory inside one.",
            }
        ),
    };
    if cargo_root != crate::services::cargo_dead_code_analyzer::absolutize(path) {
        crate::status_eprintln!(
            "📦 {} is a subtree of the crate at {}: cargo compiles the whole crate \
             (rustc cannot type-check less) and the report is restricted to the subtree",
            path.display(),
            cargo_root.display()
        );
    }

    // Create file filter
    let filter = FileFilter::new(filters.include, filters.exclude)?;

    // Use the accurate cargo-based analyzer for Rust projects
    use crate::services::cargo_dead_code_analyzer::CargoDeadCodeAnalyzer;
    let cargo_analyzer = if filters.include_tests {
        CargoDeadCodeAnalyzer::new(path)
            .include_tests()
            .with_max_depth(filters.max_depth)
    } else {
        CargoDeadCodeAnalyzer::new(path).with_max_depth(filters.max_depth)
    }
    // `--timeout`, enforced by killing the `cargo check` child. Without this the
    // analyzer used its own hardcoded 90s, which both ignored a smaller
    // `--timeout` and silently capped a larger one.
    .with_timeout(budget);

    // Run cargo-based analysis for accurate results
    let accurate_report = cargo_analyzer.analyze().await?;

    // Create config for the result
    let config = DeadCodeAnalysisConfig {
        include_unreachable: filters.include_unreachable,
        include_tests: filters.include_tests,
        min_dead_lines: filters.min_dead_lines,
    };

    let project_total_lines = accurate_report.total_lines;
    // Every .rs file in the project, against the subset actually scanned. A
    // cache entry written before `project_files` existed deserialises it as 0,
    // so fall back to the scanned count rather than claim a project smaller
    // than the scan.
    let project_files = accurate_report
        .project_files
        .max(accurate_report.total_files);
    // Measured over every line the analyzer walked, before any filter. This is
    // the only figure `--fail-on-violation` may compare against; the summary's
    // is scoped to the list that survived `--top-files`/`--min-dead-lines`.
    #[allow(clippy::cast_possible_truncation)]
    let project_dead_percentage = Some(accurate_report.dead_code_percentage as f32);
    // Taken before `accurate_report` is consumed below. A cargo run always has
    // a verdict here; the fallback exists only so an older cached report cannot
    // publish `null` -- which on this engine would mean "no compiler layer",
    // the one thing that is never true of it.
    let compiler_scan = Some(accurate_report.compiler_scan.clone().unwrap_or_else(|| {
        crate::models::dead_code::CompilerScanReport::reduced(
            crate::models::dead_code::COMPILER_SCAN_REASON_LOCKFILE,
            "this report was produced without a record of whether rustc's \
dead-code lint ran, so it cannot be relied on as a full scan"
                .to_string(),
        )
    }));
    // UNFILTERED rows: one per file the analyzer found a reportable item in.
    // The threshold is applied below, against these, so what it removes can be
    // counted instead of vanishing.
    let mut analysis_result = create_dead_code_ranking_result(accurate_report, config);

    // Everything found, before any of this command's three report filters. The
    // summary that follows describes the LIST; this describes the analysis, and
    // the difference between them is what the report now has to declare.
    let found = DeadCodeFindingTotals::of(&analysis_result.ranked_files);
    let files_with_dead_code_found = found.files;

    // `--min-dead-lines`. It used to live inside the conversion, which is how it
    // came to erase findings without a trace: the rows were gone before anything
    // could count them.
    analysis_result
        .ranked_files
        .retain(|file| passes_min_dead_lines(file, filters.min_dead_lines));

    // Apply file filter to results if filters are active.
    //
    // This filters the REPORTED LIST, not the scan: `analyzed_files` and the
    // project-wide percentage still cover every file cargo walked. The report
    // says so rather than letting `--include 'examples/**'` on a two-file crate
    // print "Files analyzed: 2" as though the filter had narrowed the scan.
    let list_filtered = filter.has_filters();
    if list_filtered {
        analysis_result.ranked_files.retain(|file| {
            let path = std::path::Path::new(&file.path);
            filter.should_include(path)
        });
    }

    // Apply top_files limit if specified
    let mut files_truncated = false;
    if let Some(limit) = filters.top_files {
        if limit > 0 && analysis_result.ranked_files.len() > limit {
            analysis_result.ranked_files.truncate(limit);
            files_truncated = true;
        }
    }

    // The summary must describe the list it heads, always — after every filter
    // and after truncation.
    resummarize_from_listed_files(
        &mut analysis_result.summary,
        &analysis_result.ranked_files,
        project_total_lines,
    );

    // …and what the filters took is declared, in the same units, so the two
    // together still account for everything the analyzer found.
    let omitted = found.minus(DeadCodeFindingTotals::of(&analysis_result.ranked_files));

    // Convert to DeadCodeResult
    Ok(DeadCodeAnalysisOutcome {
        report: crate::models::dead_code::DeadCodeResult {
            summary: analysis_result.summary.clone(),
            files: analysis_result.ranked_files,
            // `total_files` is the project; `analyzed_files` is what was read.
            // They differ whenever a tree was excluded, which is what makes the
            // narrowing visible to a reader -- and to a CI gate -- instead of
            // the report reading as a clean bill of health over everything.
            total_files: project_files,
            analyzed_files: analysis_result.summary.total_files_analyzed,
            files_with_dead_code_found,
            files_truncated,
            // Stated even though this engine never had the bug: rustc's
            // dead-code pass already knows the crate's targets, so a `pub fn`
            // in a `--lib` build is reachable to it. Publishing the verdict on
            // BOTH engines is what lets a consumer read one field instead of
            // knowing which engine answered — and `exported_roots: None` says
            // this engine seeded no roots of its own rather than that it looked
            // and found none.
            library_target: Some(cargo_library_target(path)),
            // Whether rustc's dead-code lint contributed to the list above.
            // On this engine it is the difference between "nothing is dead"
            // and "nothing was ADMITTED to be dead"; see #1076.
            compiler_scan,
        },
        project_dead_percentage,
        scope: DeadCodeReportScope {
            omitted,
            project_dead_percentage,
            // Only the test tree is out of scope: `examples/` and `benches/`
            // are scanned with or without `--include-tests` (see
            // `CargoDeadCodeAnalyzer::should_analyze`). The line used to name
            // all three, directly above a Top Files list made of `examples/`.
            skipped_kind: Some("test code; --include-tests scans it too"),
            list_filtered,
        },
        engine: DEAD_CODE_ENGINE_CARGO,
        // This engine is reached only when detection said "rust" (the dispatch
        // at the top of this function), so naming it here is stating the branch
        // we are in, not a second detection.
        language: "rust".to_string(),
        // rustc's dead-code pass names what is dead; it never enumerates what
        // is alive. See the field's own note for why a number is not invented.
        total_functions: None,
    })
}

/// Does this file survive `--min-dead-lines`?
///
/// A file whose only finding is unreachable code is charged no dead lines at
/// all, so any positive threshold would cut it; the flag that asked for it wins.
fn passes_min_dead_lines(
    file: &crate::models::dead_code::FileDeadCodeMetrics,
    min_dead_lines: usize,
) -> bool {
    file.dead_lines >= min_dead_lines || file.unreachable_blocks > 0
}

/// Is there anything in this row for the report to say?
///
/// The conversion produces a row per file the analyzer flagged, and with
/// `--include-unreachable` off a file whose only finding is an unreachable
/// statement has every counter at zero and an empty item list. It was excluded
/// as a side effect of the old default threshold (`0 >= 10` is false); at the
/// honest default of 0 that accident stops working, and an empty row would be
/// listed as a file with dead code that names none.
fn has_reportable_items(file: &crate::models::dead_code::FileDeadCodeMetrics) -> bool {
    !file.items.is_empty()
}

/// Recompute every summary figure from the files that are actually listed.
///
/// A count must agree with the list it heads (`pmat-no-fabrication-v1`). Before
/// this, `files_with_dead_code` was the pre-filter count (26 over a 4-entry
/// array, and 1 over an EMPTY array on a small fixture) and `total_dead_lines`
/// came from a different estimator than the rows (94 vs a row sum of 76).
fn resummarize_from_listed_files(
    summary: &mut crate::models::dead_code::DeadCodeSummary,
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
    project_total_lines: usize,
) {
    summary.files_with_dead_code = files.len();
    summary.total_dead_lines = files.iter().map(|f| f.dead_lines).sum();
    summary.dead_functions = files.iter().map(|f| f.dead_functions).sum();
    summary.dead_classes = files.iter().map(|f| f.dead_classes).sum();
    summary.dead_modules = files.iter().map(|f| f.dead_modules).sum();
    summary.unreachable_blocks = files.iter().map(|f| f.unreachable_blocks).sum();
    summary.dead_percentage = if project_total_lines > 0 {
        #[allow(clippy::cast_precision_loss)]
        let pct = (summary.total_dead_lines as f32 / project_total_lines as f32) * 100.0;
        pct.min(100.0)
    } else {
        0.0
    };
}

/// Run the multi-language analyzer inside `budget`.
///
/// `analyze_dead_code_multi_language` is synchronous and holds its thread for
/// the whole scan, so a `tokio::time::timeout` wrapped around it directly is
/// exactly the non-enforcement `--timeout` already had: the timer cannot fire
/// while the future never yields. Running it on the blocking pool gives the
/// timer something to fire against, and `--timeout` then reports the same
/// message on both analyzers.
///
/// Caveat, stated rather than hidden: there is no external process to kill
/// here, so the blocking task keeps running after the budget is spent. It dies
/// with the process, which exits immediately on the error this returns.
async fn run_multi_language_dead_code_within(
    path: &Path,
    filters: DeadCodeAnalysisFilters,
    language: String,
    budget: std::time::Duration,
) -> Result<DeadCodeAnalysisOutcome> {
    let owned_path = path.to_path_buf();
    let task = tokio::task::spawn_blocking(move || {
        run_multi_language_dead_code(&owned_path, &filters, &language)
    });

    match tokio::time::timeout(budget, task).await {
        Ok(joined) => {
            use anyhow::Context;
            joined.context("multi-language dead code analysis panicked")?
        }
        Err(_) => anyhow::bail!(
            "Dead code analysis timed out after {} seconds",
            budget.as_secs()
        ),
    }
}

/// Run multi-language dead code analysis for non-Rust projects
/// Dead functions grouped by the file that holds them, honouring
/// `--include-tests`.
///
/// The flag reached this path and was NEVER READ: it was inert for every
/// non-Rust project, while the cargo path honours it by not compiling the test
/// targets at all. MCP `analyze_dead_code` used to apply the filter itself, on
/// top of the analyzer — exactly the duplication [`run_dead_code_suite`] exists
/// to end — so the predicate lives here, where the report is built, and both
/// surfaces get it.
fn group_dead_functions_by_file<'a>(
    ml_result: &'a crate::services::dead_code_multi_language::DeadCodeResult,
    filters: &DeadCodeAnalysisFilters,
) -> std::collections::HashMap<
    String,
    Vec<&'a crate::services::dead_code_multi_language::DeadFunction>,
> {
    let mut file_map: std::collections::HashMap<
        String,
        Vec<&crate::services::dead_code_multi_language::DeadFunction>,
    > = std::collections::HashMap::new();
    for dead_fn in ml_result
        .dead_functions
        .iter()
        .filter(|dead_fn| filters.include_tests || !is_test_path(&dead_fn.file))
    {
        file_map
            .entry(dead_fn.file.clone())
            .or_default()
            .push(dead_fn);
    }
    file_map
}

fn run_multi_language_dead_code(
    path: &Path,
    filters: &DeadCodeAnalysisFilters,
    language: &str,
) -> Result<DeadCodeAnalysisOutcome> {
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeSummary, DeadCodeType, FileDeadCodeMetrics,
    };
    use crate::services::dead_code_multi_language::analyze_dead_code_multi_language;

    crate::status_eprintln!("🌐 Using multi-language analyzer for {language}");

    let ml_result = analyze_dead_code_multi_language(path)?;

    let file_map = group_dead_functions_by_file(&ml_result, filters);

    let mut files: Vec<FileDeadCodeMetrics> = file_map
        .into_iter()
        .map(|(file_path, dead_fns)| {
            // MEASURED (was the literal `100` for every file). `0` only when
            // the file cannot be read, in which case `update_percentage` leaves
            // the percentage at 0.0 rather than dividing by a made-up total.
            let total_lines = count_lines_of(path, &file_path);
            let mut metrics = FileDeadCodeMetrics::new(file_path);
            metrics.total_lines = total_lines;
            for dead_fn in &dead_fns {
                metrics.add_item(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: dead_fn.name.clone(),
                    line: dead_fn.line as u32,
                    reason: dead_fn.reason.clone(),
                });
            }
            // `add_item` bills a flat 10 lines per dead function, which is an
            // estimate, while `total_lines` above is measured. Unbounded, the
            // estimate exceeded the file it describes: a 2-line h.py with one
            // dead function reported dead_lines 10 / total_lines 2 = 500.0%,
            // and a 10-line m.py reported 20 / 10 = 200.0%. Dead code cannot
            // occupy more lines than the file physically has, so the estimate is
            // held to the measured file length before any percentage is taken.
            if total_lines > 0 {
                metrics.dead_lines = metrics.dead_lines.min(total_lines);
            }
            // Lua has dynamic dispatch, so Medium confidence for non-local functions
            metrics.confidence = ConfidenceLevel::Medium;
            metrics.update_percentage();
            metrics.calculate_score();
            metrics
        })
        .collect();

    // Sort by score descending, path ascending as the tie-break: the map above
    // is a HashMap, so equal-score files would otherwise swap between runs.
    files.sort_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });

    // Found first, then trimmed — same order as the cargo path, for the same
    // reason: a row removed before it is counted cannot be declared.
    //
    // The threshold is also the SAME predicate as the cargo path's now. This
    // path used to add `|| f.dead_functions > 0`, and every row here is built
    // out of dead functions, so the escape hatch admitted all of them and
    // `--min-dead-lines` could not affect a Python or Lua project at any value —
    // one flag, one meaning per language was not one of the options. Whatever it
    // trims is declared in `omitted` below, on this path as on the other.
    let found = DeadCodeFindingTotals::of(&files);
    let files_with_dead_code_found = found.files;
    files.retain(|f| passes_min_dead_lines(f, filters.min_dead_lines));

    let mut files_truncated = false;
    if let Some(limit) = filters.top_files {
        if limit > 0 && files.len() > limit {
            files.truncate(limit);
            files_truncated = true;
        }
    }

    // from_files() derives every figure from the files it is given, so the
    // summary always agrees with the list that follows it.
    let mut summary = DeadCodeSummary::from_files(&files);
    // ...but `total_files_analyzed` is not a figure about the LISTED files: it
    // is how many files were read. `from_files` had it counting the files with
    // dead code, so one run of `analyze dead-code` reported three different
    // numbers for it -- stdout "Files analyzed: 2" (from `analyzed_files`),
    // stderr "0 files analyzed" and JSON `summary.total_files_analyzed: 0`
    // (both from here).
    summary.total_files_analyzed = ml_result.total_files;

    // Source files under `path` in a language this run did not read. The
    // multi-language analyzer picks ONE language per project, so on a 19-file /
    // 12-language tree it silently dropped 17 files and still headed the report
    // "Files analyzed: 2" with no skip line at all.
    let source_files = count_source_files(path);
    let total_files = source_files.max(ml_result.total_files);

    let omitted = found.minus(DeadCodeFindingTotals::of(&files));

    Ok(DeadCodeAnalysisOutcome {
        report: crate::models::dead_code::DeadCodeResult {
            summary,
            // MEASURED file count (#720). These were `total_functions.max(1)` --
            // a FUNCTION count under a FILE label, which made a 2-file Python
            // fixture with 4 functions print "Files Analyzed | 4" directly above
            // a summary that correctly said 2, and `.max(1)` invented one file
            // for an empty project.
            total_files,
            analyzed_files: ml_result.total_files,
            files,
            files_with_dead_code_found,
            files_truncated,
            // WHICH WAY the library question was answered, and — when it could
            // not be — what could not be decided. This engine has no compiler to
            // ask, so the verdict changes the finding list: with it, a Python
            // package's `__all__` is kept; without it, the same tree reported
            // its entire public API as dead at 100%. A reader cannot weigh the
            // list without it.
            library_target: Some(library_target_report(
                &ml_result.library_target,
                ml_result.exported_roots,
            )),
            // This engine has no compiler layer to report on: reachability
            // here is computed from the source, never from rustc. `None` says
            // "there was none", which is the only honest value -- a `full`
            // here would claim a compile that never happened.
            compiler_scan: None,
        },
        // The multi-language analyzer never counts the project's total lines,
        // only the lines of the files it flagged, so there is no project-wide
        // ratio to report. `--fail-on-violation` refuses rather than comparing
        // the list-scoped figure and calling the result a pass.
        project_dead_percentage: None,
        scope: DeadCodeReportScope {
            omitted,
            project_dead_percentage: None,
            skipped_kind: Some(
                "source in languages this run did not read; the multi-language \
                 analyzer reads one language per project",
            ),
            // `--include`/`--exclude` are not wired into this path at all, so
            // nothing here was list-filtered.
            list_filtered: false,
        },
        engine: DEAD_CODE_ENGINE_MULTI_LANGUAGE,
        // The analyzer's OWN answer, not the `language` argument: detection
        // reports one dominant language and the analyzer falls back to a
        // language that is actually present when that one has no strategy, so
        // the two disagree on exactly the mixed trees where it matters most.
        language: ml_result.language.clone(),
        // This engine walks a call graph, so it has the live functions in hand
        // and has always counted them; the count simply had nowhere to go once
        // the report became `DeadCodeResult`. It is the denominator for the
        // dead-function list above.
        total_functions: Some(ml_result.total_functions),
    })
}

/// The multi-language engine's own library verdict, in the shape the report
/// publishes.
fn library_target_report(
    target: &crate::services::dead_code_multi_language::LibraryTarget,
    exported_roots: usize,
) -> crate::models::dead_code::LibraryTargetReport {
    crate::models::dead_code::LibraryTargetReport {
        verdict: target.verdict().to_string(),
        detail: target.detail().to_string(),
        exported_roots: Some(exported_roots),
    }
}

/// The cargo engine's library verdict, taken from the ENCLOSING CRATE.
///
/// It is the same `project_has_library` that decides whether this run asks
/// cargo for `--lib` or `--bins`, and it is asked about the same directory, so
/// the report cannot claim a target shape the scan did not use.
///
/// Asked about the requested path instead, it published a statement of fact
/// that was false: `--path <crate>/src/inner` reported `"not-a-library"` with
/// the detail "Cargo.toml declares no [lib] and there is no src/lib.rs" about a
/// crate whose `src/lib.rs` was two directories up. A subdirectory declares no
/// cargo target because subdirectories never do; that is a fact about
/// subdirectories, not about the crate.
///
/// A path with no enclosing PACKAGE is `undetermined` — the analyzer cannot tell
/// a library from a program without a manifest that declares a crate, and
/// "not-a-library" is a DECISION, not a synonym for "did not find one".
/// `exported_roots` is `None` because this engine seeds none: rustc's dead-code
/// pass resolves a library's public API for itself.
///
/// The manifest is NAMED in every branch. The verdict is a checkable claim about
/// a specific file, and it was published without saying which file: "the
/// Cargo.toml at <workspace root> declares no [lib]" is true of every workspace
/// manifest ever written and says nothing whatever about a crate, because a
/// workspace manifest declares none.
fn cargo_library_target(path: &Path) -> crate::models::dead_code::LibraryTargetReport {
    use crate::models::dead_code::LibraryTargetReport;
    use crate::services::cargo_dead_code_analyzer::{
        absolutize, project_has_library, resolve_crate_root, CrateRootResolution,
    };

    let resolution = resolve_crate_root(path);
    let CrateRootResolution::Package {
        root,
        manifest,
        name,
    } = &resolution
    else {
        return LibraryTargetReport {
            verdict: "undetermined".to_string(),
            detail: format!(
                "cargo: {}, so there is no crate whose target shape could be read — \
                 whether these `pub` items are a library's public API or a program's \
                 internals was not decided",
                resolution
                    .no_package_reason(path)
                    .unwrap_or_else(|| "no package encloses this path".to_string())
            ),
            exported_roots: None,
        };
    };
    let package = name
        .as_ref()
        .map_or_else(|| "a package".to_string(), |n| format!("package `{n}`"));
    let viewed_from_inside = *root != absolutize(path);
    let scope_note = if viewed_from_inside {
        " (the analysed path is a subtree of that crate; cargo compiled the whole crate \
         and the report is restricted to the subtree)"
    } else {
        ""
    };

    if project_has_library(root) {
        LibraryTargetReport {
            verdict: "library".to_string(),
            detail: format!(
                "cargo: {} declares {package}, which has a library target, and rustc's \
                 dead-code pass treats its public API as reachable{scope_note}",
                manifest.display()
            ),
            exported_roots: None,
        }
    } else {
        LibraryTargetReport {
            verdict: "not-a-library".to_string(),
            detail: format!(
                "cargo: {} declares {package}, which declares no [lib] and has no \
                 src/lib.rs beside it — a binary-only crate, whose `pub` items rustc \
                 reports as dead when nothing calls them{scope_note}",
                manifest.display()
            ),
            exported_roots: None,
        }
    }
}

/// Does this path look like test code?
///
/// The reachability analyzer has no test-awareness of its own, so this is what
/// `--include-tests` means on the multi-language path. Kept deliberately
/// conventional — `tests/`, `test_x`, `x_test`, `x_tests` — because it has to
/// agree across languages that have no cargo to ask.
fn is_test_path(file: &str) -> bool {
    let path = std::path::Path::new(file);
    let in_test_dir = path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("tests") | Some("test") | Some("testing")
        )
    });
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();

    in_test_dir || name.starts_with("test_") || name.ends_with("_test") || name.ends_with("_tests")
}

/// Source files under `root`, by extension.
///
/// The multi-language analyzer reports only the files of the ONE language it
/// chose, so its count cannot say how much of the tree went unread. This is the
/// denominator that makes the gap visible.
fn count_source_files(root: &Path) -> usize {
    const SOURCE_EXTENSIONS: &[&str] = &[
        "rs", "ruchy", "rh", "c", "h", "cpp", "cc", "cxx", "hpp", "hxx", "py", "pyi", "lua", "go",
        "java", "kt", "kts", "scala", "swift", "cs", "rb", "php", "js", "jsx", "mjs", "cjs", "ts",
        "tsx", "sh", "bash", "zig", "zsh", "pl", "pm", "ex", "exs", "erl", "hs", "ml", "dart",
        "vim", "r", "jl",
    ];

    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
        })
        .count()
}

/// Physical line count for a file discovered by the multi-language analyzer.
///
/// The reported path may be absolute or relative to the analyzed root; `0` is
/// returned only when neither resolves to a readable file.
fn count_lines_of(root: &Path, file_path: &str) -> usize {
    let candidate = Path::new(file_path);
    let full = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    std::fs::read_to_string(&full)
        .or_else(|_| std::fs::read_to_string(candidate))
        .map(|content| content.lines().count())
        .unwrap_or(0)
}

/// Create dead code ranking result from cargo analysis report.
///
/// The rows come back UNFILTERED — `--min-dead-lines` is applied by the caller,
/// after the found totals have been taken. It used to be applied in here, which
/// meant the rows a threshold removed were gone before any counter saw them: the
/// report could only ever say how many FILES it had lost, never what was in
/// them, and a summary of zeros was indistinguishable from a clean project.
fn create_dead_code_ranking_result(
    accurate_report: crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    config: crate::models::dead_code::DeadCodeAnalysisConfig,
) -> crate::models::dead_code::DeadCodeRankingResult {
    use crate::models::dead_code::DeadCodeRankingResult;
    use chrono::Utc;

    DeadCodeRankingResult {
        ranked_files: cargo_files_to_metric_rows(
            accurate_report.files_with_dead_code.clone(),
            config.include_unreachable,
        ),
        summary: create_dead_code_summary(&accurate_report),
        analysis_timestamp: Utc::now(),
        config,
    }
}

/// Convert cargo dead code files to metrics format, then apply
/// `--min-dead-lines`.
///
/// `include_unreachable` is the ONLY thing that lets an unreachable finding into
/// the report: with it off the rows are exactly what they were before the
/// analyzer started collecting rustc's `unreachable_code` lint, down to the
/// `unreachable_blocks: 0`.
fn convert_cargo_files_to_metrics(
    cargo_files: Vec<crate::services::cargo_dead_code_analyzer::FileDeadCode>,
    min_dead_lines: usize,
    include_unreachable: bool,
) -> Vec<crate::models::dead_code::FileDeadCodeMetrics> {
    let mut rows = cargo_files_to_metric_rows(cargo_files, include_unreachable);
    rows.retain(|file| passes_min_dead_lines(file, min_dead_lines));
    rows
}

/// One metrics row per file the analyzer flagged, before any report filter.
fn cargo_files_to_metric_rows(
    cargo_files: Vec<crate::services::cargo_dead_code_analyzer::FileDeadCode>,
    include_unreachable: bool,
) -> Vec<crate::models::dead_code::FileDeadCodeMetrics> {
    use crate::models::dead_code::{ConfidenceLevel, FileDeadCodeMetrics};

    cargo_files
        .into_iter()
        .map(|file| {
            let dead_functions_count = count_dead_items_by_kind(
                &file,
                &[
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Function,
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Method,
                ],
            );
            let dead_classes_count = count_dead_items_by_kind(
                &file,
                &[
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Struct,
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Enum,
                ],
            );

            let dead_modules_count = count_dead_items_by_kind(
                &file,
                &[crate::services::cargo_dead_code_analyzer::DeadCodeKind::Module],
            );

            FileDeadCodeMetrics {
                path: file.file_path.display().to_string(),
                // Same estimator as the project total (5 lines per fn/method,
                // 3 per struct/enum, 2 otherwise). It used to be
                // `dead_items.len() * 4`, which disagreed with the summary.
                dead_lines: crate::services::cargo_dead_code_analyzer::estimated_dead_lines_bounded(
                    &file.dead_items,
                    file.total_lines,
                ),
                // MEASURED (was the literal `100` for every file, which
                // contradicted the dead_percentage printed beside it: a
                // 370-line file reported dead_lines 24 / total_lines 100 and
                // dead_percentage 6.49). `0` when the file could not be read;
                // the percentage is then 0.0 too, so the row never claims a
                // ratio it did not compute.
                total_lines: file.total_lines.unwrap_or(0),
                dead_percentage: file.file_dead_percentage as f32,
                dead_functions: dead_functions_count,
                dead_classes: dead_classes_count,
                dead_modules: dead_modules_count,
                unreachable_blocks: if include_unreachable {
                    file.unreachable_items.len()
                } else {
                    0
                },
                dead_score: file.file_dead_percentage as f32,
                confidence: ConfidenceLevel::High, // Cargo-based detection is high confidence
                items: {
                    let mut items = dead_items_to_report_items(&file.dead_items);
                    if include_unreachable {
                        items.extend(unreachable_items_to_report_items(&file.unreachable_items));
                    }
                    items
                },
            }
        })
        // A row with no items names nothing and counts nothing: it is what a
        // file whose only finding is unreachable code becomes when
        // `--include-unreachable` is off. `--min-dead-lines` used to exclude it
        // by accident (its dead_lines are 0), which stopped being true when the
        // threshold stopped defaulting to 10.
        .filter(has_reportable_items)
        .collect()
}

/// Carry rustc's `unreachable_code` findings into the report.
///
/// `DeadCodeType::UnreachableCode` already existed and already had a summary
/// counter (`unreachable_blocks`); nothing on the CLI path ever produced one.
fn unreachable_items_to_report_items(
    items: &[crate::services::cargo_dead_code_analyzer::DeadItem],
) -> Vec<crate::models::dead_code::DeadCodeItem> {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    items
        .iter()
        .map(|item| DeadCodeItem {
            item_type: DeadCodeType::UnreachableCode,
            name: item.name.clone(),
            line: u32::try_from(item.line).unwrap_or(u32::MAX),
            reason: item.message.clone(),
        })
        .collect()
}

/// Carry the dead items cargo actually reported into the result.
///
/// They were dropped (`items: Vec::new()`), which left the counts in the
/// summary unverifiable — a reader could not see WHICH items produced
/// `total_dead_lines`.
fn dead_items_to_report_items(
    items: &[crate::services::cargo_dead_code_analyzer::DeadItem],
) -> Vec<crate::models::dead_code::DeadCodeItem> {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    use crate::services::cargo_dead_code_analyzer::DeadCodeKind;

    items
        .iter()
        .map(|item| DeadCodeItem {
            // EXHAUSTIVE on purpose: no `_` arm. The wildcard here is what typed
            // every suppressed function as `"variable"` in a record whose own
            // `reason` said `fn`, and it would silently swallow any kind added
            // later the same way.
            //
            // #928: `Module` and `Other` used to land on `Variable` because the
            // target enum had no way to say either one — a dead module was
            // published as `"item_type": "variable"` beside a `reason` reading
            // "module `x` is never used", and `union `U` is never used` (which
            // the parser classifies as `Other`) was published the same way.
            // Both now map to the variant that names them.
            item_type: match &item.kind {
                DeadCodeKind::Function | DeadCodeKind::Method => DeadCodeType::Function,
                DeadCodeKind::Struct
                | DeadCodeKind::Enum
                | DeadCodeKind::Trait
                | DeadCodeKind::TypeAlias => DeadCodeType::Class,
                DeadCodeKind::Variant
                | DeadCodeKind::Field
                | DeadCodeKind::Constant
                | DeadCodeKind::Static => DeadCodeType::Variable,
                DeadCodeKind::Module => DeadCodeType::Module,
                DeadCodeKind::UnreachableCode => DeadCodeType::UnreachableCode,
                DeadCodeKind::Other(_) => DeadCodeType::Other,
            },
            name: item.name.clone(),
            line: u32::try_from(item.line).unwrap_or(u32::MAX),
            reason: item.message.clone(),
        })
        .collect()
}

/// Count dead items of specific kinds
fn count_dead_items_by_kind(
    file: &crate::services::cargo_dead_code_analyzer::FileDeadCode,
    kinds: &[crate::services::cargo_dead_code_analyzer::DeadCodeKind],
) -> usize {
    file.dead_items
        .iter()
        .filter(|i| kinds.contains(&i.kind))
        .count()
}

/// Create dead code summary from cargo report.
///
/// Every per-list figure here is overwritten by `resummarize_from_listed_files`
/// once the reported list is final; only `total_files_analyzed` (the .rs files
/// walked) describes the project rather than the list.
fn create_dead_code_summary(
    accurate_report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
) -> crate::models::dead_code::DeadCodeSummary {
    use crate::models::dead_code::DeadCodeSummary;

    DeadCodeSummary {
        total_files_analyzed: accurate_report.total_files, // actual .rs files walked
        files_with_dead_code: accurate_report.files_with_dead_code.len(),
        total_dead_lines: accurate_report.dead_lines,
        dead_percentage: accurate_report.dead_code_percentage as f32,
        dead_functions: get_dead_count_by_types(accurate_report, &["function", "method"]),
        dead_classes: get_dead_count_by_types(accurate_report, &["struct", "enum"]),
        dead_modules: get_dead_count_by_types(accurate_report, &["module"]),
        // Counted, not assumed. The literal `0` here carried the comment "Not
        // tracked by cargo", which stopped being true when the analyzer started
        // collecting rustc's `unreachable_code` lint — and a comment is not
        // something a JSON consumer can read anyway. Like every other figure in
        // this summary it is recomputed from the listed files by
        // `resummarize_from_listed_files`, which is what honours
        // `--include-unreachable`.
        unreachable_blocks: accurate_report
            .files_with_dead_code
            .iter()
            .map(|f| f.unreachable_items.len())
            .sum(),
    }
}

/// Get total dead count for specific types
fn get_dead_count_by_types(
    report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    types: &[&str],
) -> usize {
    types
        .iter()
        .map(|type_name| report.dead_by_type.get(*type_name).copied().unwrap_or(0))
        .sum()
}
