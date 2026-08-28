// R21-4 (D98) / R22-2 (D102): Glob expansion + source-tree walking live in
// the shared `crate::services::path_glob` module so the parallel
// `src/handlers/tools/` dispatcher can use the same implementation.
use crate::services::path_glob::expand_paths_to_source_files;

// Re-export for the existing `coverage_tests` suite, which references
// `resolve_paths_with_globs` at module scope (R21-4 test fixture carried
// over from before the service extraction).
#[cfg(test)]
use crate::services::path_glob::resolve_paths_with_globs;

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_complexity(
    paths: &[PathBuf],
    top_files: Option<usize>,
    threshold: Option<u64>,
) -> Result<Value> {
    // `analyze_file_complexity_uncached` used to run the heuristic counter
    // while `pmat analyze complexity` ran the AST one, so this tool reported
    // cyclomatic 10 / cognitive 18 (plus a threshold violation) for the same
    // function the CLI scored 6 / 9 with no violation. Both surfaces now share
    // the analyzer; keep them on one entry point.
    use crate::services::complexity::analyze_file_complexity_uncached;

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let threshold_value = threshold.unwrap_or(10);
    // Issue #1058. NOT `expand_paths_to_source_files`: that admits `.sh`, `.h`
    // and `.hpp`, which `pmat analyze complexity` does not measure, and refuses
    // `.lean`, `.kts` and `.cs`, which it does. Two allow-lists for one
    // question made the two transports report 41 and 39 files for copia. This
    // one is derived from the CLI's own list.
    let files = crate::services::path_glob::expand_paths_to_complexity_files(paths);

    // Analyze all expanded files
    let mut all_functions = Vec::new();
    let mut total_files = 0;
    let mut total_complexity = 0u64;
    let mut violations = Vec::new();

    for path in &files {
        match analyze_file_complexity_uncached(path, None).await {
            Ok(metrics) => {
                total_files += 1;

                for func in &metrics.functions {
                    let cc = func.metrics.cyclomatic as u64;
                    total_complexity += cc;

                    if cc >= threshold_value {
                        violations.push(json!({
                            "file": metrics.path.clone(),
                            "function": func.name.clone(),
                            "complexity": cc,
                            "threshold": threshold_value,
                            "line_start": func.line_start,
                            "line_end": func.line_end,
                        }));
                    }

                    all_functions.push(json!({
                        "file": metrics.path.clone(),
                        "function": func.name.clone(),
                        "cyclomatic_complexity": func.metrics.cyclomatic,
                        "cognitive_complexity": func.metrics.cognitive,
                        "line_start": func.line_start,
                        "line_end": func.line_end,
                    }));
                }
            }
            Err(_) => continue, // Skip files that fail to analyze
        }
    }

    // Sort by complexity and apply top_files limit
    let mut sorted_functions = all_functions;
    if let Some(limit) = top_files {
        sorted_functions.sort_by(|a, b| {
            let a_cc = a["cyclomatic_complexity"].as_u64().unwrap_or(0);
            let b_cc = b["cyclomatic_complexity"].as_u64().unwrap_or(0);
            b_cc.cmp(&a_cc) // Descending order
        });
        sorted_functions.truncate(limit);
    }

    let average_complexity = if total_files > 0 {
        total_complexity / total_files as u64
    } else {
        0
    };

    Ok(json!({
        "status": "completed",
        "message": "Complexity analysis completed",
        "results": {
            "total_files": total_files,
            // Issue #1058: the name `analyze complexity --format json` uses for
            // this same number. One spelling, both transports — a key-name
            // split reads as a measurement disagreement, which is worse than
            // either number being wrong.
            "files_analyzed": total_files,
            "total_complexity": total_complexity,
            "average_complexity": average_complexity,
            "violations": violations,
            "top_files": sorted_functions,
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// `include_tests` exists because this tool used to have no concept of test
/// files at all, while the CLI excludes them by default. For the same fixture
/// the two surfaces answered 3 and 2, and the MCP schema offered no way to ask
/// for the CLI's answer — so neither could be made to agree with the other
/// (#997). It defaults to `false`, matching `analyze satd`'s default and
/// `AnalysisOptions::default()`.
pub async fn analyze_satd(
    paths: &[PathBuf],
    _include_resolved: bool,
    include_tests: bool,
) -> Result<Value> {
    use crate::services::satd_detector::{FileCensus, SATDDetector, SkipReason};

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let detector = SATDDetector::new();

    // The SAME predicate the CLI's walk uses, and the SAME counters. A second
    // opinion about what "not read" means is how these two surfaces drifted
    // apart (#997); a walk that keeps no counters at all is worse still — the
    // payload below used to report `total_satd` with nothing to divide it by,
    // so "this tree is clean" and "almost every file in it was skipped" arrived
    // as the same JSON. That is the defect #1015 fixed on the CLI side.
    //
    // Which files are READ is unchanged: `extract_from_content_with_tests`
    // already returned an empty vec for everything `should_exclude_file`
    // matches, so those files were being opened, scanned and thrown away. They
    // are now declined up front and counted.
    let candidates = expand_paths_to_source_files(paths);
    // The denominator, counted before any rule runs.
    let mut census = FileCensus::over(candidates.len());
    let mut files = Vec::new();
    for path in candidates {
        let reason: Option<SkipReason> = detector
            .skip_reason_for_analysis(&path, include_tests)
            .await;
        if let Some(reason) = reason {
            census.record_skip(&path, reason);
            continue;
        }
        files.push(path);
    }

    let mut total_satd = 0;
    let mut file_results = Vec::new();

    for path in &files {
        match tokio::fs::read_to_string(path).await {
            // `_with_tests`, not the plain wrapper. `include_tests` has TWO
            // effects and they must move together: it selects test FILES above,
            // and it reaches the inline `#[cfg(test)]` skip inside each file
            // here. Wiring only the first made MCP report 3 markers where the
            // CLI reported 4 on the same fixture — two fixes (#995 inline
            // blocks, #997 MCP parity) that each worked alone and did not
            // compose. Caught dogfooding the installed artifact, not by either
            // fix's own tests.
            Ok(content) => {
                match detector.extract_from_content_with_tests(&content, path, include_tests) {
                    Ok(debts) => {
                        // Filter out resolved debt markers (DONE, RESOLVED, FIXED) unless include_resolved
                        let debts: Vec<_> = if _include_resolved {
                            debts
                        } else {
                            debts
                                .into_iter()
                                .filter(|d| {
                                    let upper = d.text.to_uppercase();
                                    !upper.contains("DONE")
                                        && !upper.contains("RESOLVED")
                                        && !upper.contains("FIXED")
                                })
                                .collect()
                        };
                        let satd_count = debts.len();
                        total_satd += satd_count;
                        // Counted from files that were actually decoded and
                        // scanned. It used to be `files.len()` — the size of
                        // the CANDIDATE list — so a file that failed to read
                        // was reported both as read and as having no debt, the
                        // exact "not measured rendered as clean" shape of
                        // #1035, one level below the skip predicate that block
                        // was added to disclose.
                        census.record_analyzed();

                        if satd_count > 0 {
                            file_results.push(json!({
                                "file": path.display().to_string(),
                                "satd_count": satd_count,
                                "debts": debts.iter().map(|debt| json!({
                                    "line": debt.line,
                                    "category": format!("{:?}", debt.category),
                                    "severity": format!("{:?}", debt.severity),
                                    "text": debt.text,
                                })).collect::<Vec<_>>(),
                            }));
                        }
                    }
                    // The scan failed on a file that WAS opened. Not a finding
                    // of zero — a file this run did not measure.
                    Err(_) => census.record_skip(path, SkipReason::Unreadable),
                }
            }
            // I/O error, or content that is not UTF-8.
            Err(_) => census.record_skip(path, SkipReason::Unreadable),
        }
    }

    Ok(json!({
        "status": "completed",
        "message": "SATD analysis completed",
        "results": {
            "total_satd": total_satd,
            "files": file_results,
            // The denominator, in the same shape and the same spelling
            // `analyze satd --format json` emits
            // (`cli::handlers::satd_handler_formatting::format_json`). An MCP
            // consumer could not previously tell how much of the tree this
            // number was measured over, and the key was `files_read` here and
            // `files_analyzed` there — a name split reads as a measurement
            // disagreement, which is worse than either number being wrong
            // (#1058).
            "files_discovered": census.discovered,
            "files_analyzed": census.analyzed,
            "census_balances": census.partitions(),
            "files_unaccounted": census.unaccounted(),
            "files_not_read": {
                "total": census.not_read.total(),
                "tests": census.not_read.tests,
                "out_of_scope": census.not_read.out_of_scope,
                "minified_or_vendor": census.not_read.minified_or_vendor,
                "too_large": census.not_read.too_large,
                "unreadable": census.not_read.unreadable,
                "oversized": census.oversized.iter().map(|f| json!({
                    "path": f.path,
                    "bytes": f.bytes,
                    "limit_bytes": f.limit_bytes,
                })).collect::<Vec<_>>()
            },
            // Always false here, and stated rather than left to be assumed:
            // this surface has no `--top-files` equivalent, so it never elides
            // a finding. A consumer that checks the field before trusting a
            // count gets the right answer from either surface.
            "violations_truncated": false,
        }
    }))
}

/// Find dead code, by running the analysis `pmat analyze dead-code` runs.
///
/// This tool used to call `analyze_dead_code_multi_language` directly, which
/// made "dead code" the name of two different analyses:
///
/// ```text
///   bin crate, 1 dead fn + 2 never-constructed structs
///     CLI  {dead_functions: 1, dead_classes: 2}   MCP  {total_dead_code: 1}
///   lib crate
///     CLI  {never_called_one, never_called_two, dead_method, NeverConstructed}
///     MCP  {entry, never_called_one, never_called_two, dead_method}
///   src/models (this repo)
///     CLI  0                                      MCP  50
/// ```
///
/// The reachability analyzer has no notion of a dead TYPE, so `dead_classes`
/// could not cross to this surface at all; and it reports every un-called `pub`
/// item as dead, which is precisely wrong for a library — a library's public
/// API is its entry point. That is `entry` in the second row and all 50 in the
/// third. A note saying "the two analyzers disagree" would have left both
/// numbers wrong.
///
/// So the dispatch happens once, in
/// [`crate::cli::handlers::dead_code_handlers::run_dead_code_suite`], and both
/// surfaces call it: Rust goes to cargo's own dead-code pass, everything else
/// goes to the reachability analyzer, and neither surface can pick a different
/// engine than the other for the same path.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dead_code(paths: &[PathBuf], include_tests: bool) -> Result<Value> {
    use crate::cli::handlers::dead_code_handlers::run_dead_code_suite;
    use std::collections::{BTreeMap, BTreeSet};

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // BTree, not Hash: the file list is published, and a HashMap ordered it
    // differently on every call for no reason a client could act on.
    let mut dead_by_file: BTreeMap<String, Vec<crate::models::dead_code::DeadCodeItem>> =
        BTreeMap::new();
    let mut engines: BTreeSet<&'static str> = BTreeSet::new();
    // Sorted and de-duplicated, for the same reason `dead_by_file` is a BTree:
    // this is published, and a per-call ordering is noise a client cannot act
    // on. (Master built it with `Vec::contains`, in first-seen order.)
    let mut languages: BTreeSet<String> = BTreeSet::new();
    let mut per_path: Vec<Value> = Vec::new();
    let mut not_analyzed: Vec<Value> = Vec::new();
    let mut files_analyzed = 0usize;
    // Issue #1058. The CLI publishes TWO counts under the names `total_files`
    // (what the walk discovered) and `analyzed_files` (what the engine read);
    // this payload published one, under a third name, `files_analyzed`. On
    // copia that is 38 and 29 against 29, so a parity check that asked both
    // transports for "dead-code files" was answered 38 by one and 29 by the
    // other — and the two agree exactly. A key-name split reads as a
    // measurement disagreement, which is worse than either number being wrong,
    // because both surfaces look right alone.
    let mut total_files = 0usize;
    // The denominator for the dead-function count, summed over the paths that
    // measured one — and only while EVERY analysed path did. A sum over the
    // subset that happened to be countable is not a denominator for a numerator
    // drawn from all of them, so the mixed case is `null`, and so is the case
    // where nothing was analysed at all.
    let mut total_functions: Option<usize> = None;
    let mut every_path_counted_functions = true;

    for path in paths {
        if !path.exists() {
            // NAMED, not skipped. `Err(_) => continue` was the old handling for
            // every failure here, so a path that could not be analysed at all
            // was indistinguishable in the payload from a path with no dead
            // code — the same "a check that did not run has not passed" rule the
            // quality gate's `not_run` list exists for.
            not_analyzed.push(json!({
                "path": path.display().to_string(),
                "reason": "path does not exist",
            }));
            continue;
        }

        // The analysis is rooted at a DIRECTORY (cargo runs in one). A file is
        // answered by analysing its directory and then listing only that file,
        // with both facts stated below rather than left for the caller to infer
        // from a path that is not the one they asked about.
        let requested_file = path.is_file().then(|| canonical_path(path));
        let root = if path.is_file() {
            path.parent().unwrap_or_else(|| std::path::Path::new("."))
        } else {
            path.as_path()
        };

        let run = match run_dead_code_suite(root, include_tests).await {
            Ok(run) => run,
            Err(e) => {
                not_analyzed.push(json!({
                    "path": path.display().to_string(),
                    "reason": format!("{e}"),
                }));
                continue;
            }
        };

        engines.insert(run.engine);
        languages.insert(run.language.clone());
        files_analyzed += run.report.analyzed_files;
        total_files += run.report.total_files;
        match run.total_functions {
            Some(counted) => total_functions = Some(total_functions.unwrap_or(0) + counted),
            None => every_path_counted_functions = false,
        }

        // What the file restriction above removed, in the units the report
        // counts in — so narrowing to one file cannot silently swallow the rest
        // of the directory's findings.
        let mut outside = DeadItemCounts::default();
        let mut listed_files = 0usize;
        for file in &run.report.files {
            let absolute = absolute_report_path(root, &file.path);
            if let Some(wanted) = &requested_file {
                if &canonical_path(std::path::Path::new(&absolute)) != wanted {
                    outside.add_items(&file.items);
                    continue;
                }
            }
            listed_files += 1;
            dead_by_file
                .entry(absolute)
                .or_default()
                .extend(file.items.iter().cloned());
        }

        per_path.push(json!({
            "requested": path.display().to_string(),
            // Equal to `requested` for a directory; a file's enclosing
            // directory otherwise.
            "analysis_root": root.display().to_string(),
            "engine": run.engine,
            // Which language the engine actually READ under this path. Not
            // inferable from `engine`: `multi-language-reachability` reads one
            // language per project and skips the rest of the tree.
            "language": run.language,
            // This path's share of the denominator, `null` where its engine
            // measures none — so a `null` total below can be attributed to the
            // path that could not be counted instead of reading as a failure of
            // the whole call.
            "total_functions": run.total_functions,
            "files_analyzed": run.report.analyzed_files,
            "files_listed": listed_files,
            // Whether the analyzer decided this path was a LIBRARY, and hence
            // whether an un-called export is above or below the line.
            //
            // A library's public API is un-called by construction, so the
            // verdict decides which findings exist — and where it is
            // `undetermined` the list DOES contain exports, reported dead
            // because nothing calls them rather than because they are known to
            // be unreachable. An agent reading this payload has no summary text
            // to fall back on, so the caveat travels with the findings.
            "library_target": run.report.library_target,
            // Whether rustc's dead-code lint contributed to these findings.
            // `null` for engines that have no compiler layer; `reduced` when
            // the compiler layer was refused because compiling the crate would
            // have written a Cargo.lock into a tree pmat was asked to READ
            // (#1076). An agent that cannot read this cannot tell an empty
            // finding list from a search that never ran.
            "compiler_scan": run.report.compiler_scan,
            // null for a directory (nothing was restricted away), a full
            // per-kind count for a file.
            "findings_outside_requested_path": requested_file
                .as_ref()
                .map(|_| outside.to_json()),
        }));
    }

    let mut totals = DeadItemCounts::default();
    let file_results: Vec<Value> = dead_by_file
        .iter()
        .map(|(file, items)| {
            let mut counts = DeadItemCounts::default();
            counts.add_items(items);
            totals.add_items(items);
            // A key per kind, and the SAME six keys `counts` uses, so a reader
            // can check that every counter has a list and every list a counter.
            // The pre-fix payload had `dead_functions` and nothing else, which
            // is why a dead struct had nowhere to go.
            let mut file_json = serde_json::Map::new();
            file_json.insert("file".into(), json!(file));
            file_json.insert("dead_code_count".into(), json!(items.len()));
            for (key, kind) in DeadItemCounts::KINDS {
                file_json.insert(key.to_string(), json!(named_items(items, kind)));
            }
            file_json.insert("counts".into(), counts.to_json());
            Value::Object(file_json)
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": "Dead code analysis completed",
        "results": {
            // Every item listed above, of every kind. It used to be the dead
            // FUNCTION count under a name that promises all dead code, which is
            // how a fixture with one dead function and two dead structs
            // answered 1 while `pmat analyze dead-code` answered 3.
            "total_dead_code": totals.total(),
            // The DENOMINATOR for `by_kind.dead_functions`. Dropped when this
            // tool moved onto the shared analyzer, which left the payload
            // stating a dead count with nothing to divide it by: `3` reads the
            // same whether the tree holds four functions or nine hundred. It is
            // `null` — never `0`, which would read as an empty tree — when an
            // analysed path's engine does not count live functions; `paths[]`
            // below says which one.
            "total_functions": total_functions.filter(|_| every_path_counted_functions),
            // …and the breakdown, so nothing counted here is unaccounted for
            // and nothing listed above is uncounted: the six fields sum to
            // `total_dead_code`.
            "by_kind": totals.to_json(),
            "files_analyzed": files_analyzed,
            // Issue #1058: the CLI's two names for the same two numbers, so one
            // consumer parser reads both transports and gets the same answer.
            // `files_analyzed` above is retained because clients already read
            // it; it is `analyzed_files` by another name and always equal to it.
            "analyzed_files": files_analyzed,
            "total_files": total_files,
            // …and the spelling `analyze complexity` uses on both surfaces, so
            // one reader covers all three commands.
            "files_discovered": total_files,
            "files": file_results,
            // One name, one analyzer. `pmat analyze dead-code -p <path>` at its
            // default flags produces these findings.
            "analyzer": "pmat analyze dead-code",
            "engines": engines.iter().collect::<Vec<_>>(),
            // Which languages were READ. Dropped in the same rewrite, and the
            // engine name does not stand in for it: the multi-language engine
            // reads ONE language per project and skips every other source file
            // under the path, so without this a client cannot tell "no dead
            // Python here" from "the Python was never opened".
            "languages": languages.iter().collect::<Vec<_>>(),
            "analyzer_note": "Same analysis as `pmat analyze dead-code` at its default flags: \
                              rustc's dead-code pass via cargo on Rust projects \
                              (`cargo-dead-code`), call-graph reachability elsewhere \
                              (`multi-language-reachability`). Both surfaces report the same \
                              findings, dead types included, for the same path. \
                              `total_functions` is null where the engine that answered does not \
                              count live functions — rustc's dead-code pass names what is dead, \
                              not what exists — rather than a 0 that would read as an empty tree.",
            "include_tests": include_tests,
            "paths": per_path,
            // Empty means every requested path was analysed — a positive claim,
            // not an absence.
            "paths_not_analyzed": not_analyzed,
        }
    }))
}

/// Dead items by kind, for a file or for the whole run.
///
/// Every kind the report can produce has a field, and [`Self::total`] is their
/// sum: a payload cannot then list an item that no counter covers, which is
/// what `dead_classes` was before this tool shared the CLI's analyzer.
#[derive(Default)]
struct DeadItemCounts {
    functions: usize,
    classes: usize,
    variables: usize,
    modules: usize,
    unreachable_blocks: usize,
    /// Items whose kind the producer could not name. NOT a synonym for
    /// "variable" — see `DeadCodeType::Other`.
    other: usize,
}

impl DeadItemCounts {
    /// The payload key for each kind, in one place, so the per-file lists and
    /// the counters beside them cannot come to use different names — or to
    /// cover different sets of kinds, which is how a listed item ends up with
    /// no counter.
    const KINDS: [(&'static str, crate::models::dead_code::DeadCodeType); 6] = {
        use crate::models::dead_code::DeadCodeType as T;
        [
            ("dead_functions", T::Function),
            ("dead_classes", T::Class),
            ("dead_variables", T::Variable),
            ("dead_modules", T::Module),
            ("unreachable_blocks", T::UnreachableCode),
            ("other", T::Other),
        ]
    };

    fn add_items(&mut self, items: &[crate::models::dead_code::DeadCodeItem]) {
        use crate::models::dead_code::DeadCodeType;
        for item in items {
            // EXHAUSTIVE on purpose: a `_` arm would quietly drop a kind added
            // later out of `total()` while it still appeared in `files`.
            match item.item_type {
                DeadCodeType::Function => self.functions += 1,
                DeadCodeType::Class => self.classes += 1,
                DeadCodeType::Variable => self.variables += 1,
                DeadCodeType::Module => self.modules += 1,
                DeadCodeType::UnreachableCode => self.unreachable_blocks += 1,
                DeadCodeType::Other => self.other += 1,
            }
        }
    }

    fn count_of(&self, kind: crate::models::dead_code::DeadCodeType) -> usize {
        use crate::models::dead_code::DeadCodeType;
        match kind {
            DeadCodeType::Function => self.functions,
            DeadCodeType::Class => self.classes,
            DeadCodeType::Variable => self.variables,
            DeadCodeType::Module => self.modules,
            DeadCodeType::UnreachableCode => self.unreachable_blocks,
            DeadCodeType::Other => self.other,
        }
    }

    fn total(&self) -> usize {
        Self::KINDS
            .iter()
            .map(|(_, kind)| self.count_of(*kind))
            .sum()
    }

    fn to_json(&self) -> Value {
        Value::Object(
            Self::KINDS
                .iter()
                .map(|(key, kind)| ((*key).to_string(), json!(self.count_of(*kind))))
                .collect(),
        )
    }
}

/// The `(name, line)` pairs of one kind, in the order the report listed them.
fn named_items(
    items: &[crate::models::dead_code::DeadCodeItem],
    kind: crate::models::dead_code::DeadCodeType,
) -> Vec<Value> {
    items
        .iter()
        .filter(|item| item.item_type == kind)
        .map(|item| json!({ "name": item.name, "line": item.line, "reason": item.reason }))
        .collect()
}

/// A report path as an absolute path.
///
/// The two engines spell it differently — the reachability analyzer's rows are
/// already absolute, cargo's are RELATIVE, and relative to the crate root rather
/// than to the directory the analysis was asked about. An MCP client has no way
/// to resolve either against a root it did not choose.
///
/// The crate root is not something this function is told, and asking cargo for
/// it would be a second subprocess, so the relative path is anchored at the
/// nearest ancestor of `root` under which it actually exists. Naively joining it
/// to `root` produced `…/fx5/src/src/main.rs` for a `src/main.rs` row analysed
/// as `…/fx5/src` — a path that names no file, which then matched nothing and
/// made a request for one file report `files_listed: 0` beside three findings
/// it called "outside" the file they are inside.
fn absolute_report_path(root: &std::path::Path, file: &str) -> String {
    let path = std::path::Path::new(file);
    if path.is_absolute() {
        return file.to_string();
    }
    for ancestor in root.ancestors() {
        let candidate = ancestor.join(path);
        if candidate.exists() {
            return candidate.display().to_string();
        }
    }
    // Nothing on disk answers to it (a stale cache entry, a deleted file): say
    // where it was looked for rather than publishing a bare relative path whose
    // anchor the client cannot know.
    root.join(path).display().to_string()
}

/// `canonicalize`, falling back to the path itself.
///
/// Used only to compare two spellings of one file; a path that cannot be
/// canonicalised simply compares as written rather than erroring the whole run.
fn canonical_path(path: &std::path::Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_lint_hotspots(paths: &[PathBuf], top_files: Option<usize>) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let top_files_limit = top_files.unwrap_or(10);
    let analyzer = TdgAnalyzer::new()?;
    let project_path = &paths[0];

    // Analyze project with TDG
    let project_score = if project_path.is_dir() {
        analyzer.analyze_project(project_path)?
    } else {
        return Err(anyhow::anyhow!("Path must be a directory"));
    };

    // Sort files by score (lower score = worse quality = hotspot)
    let mut file_scores = project_score.files.clone();
    file_scores.sort_by(|a, b| {
        a.total
            .partial_cmp(&b.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top N hotspots (lowest scores)
    file_scores.truncate(top_files_limit);

    // Build hotspot entries
    let hotspots: Vec<Value> = file_scores
        .iter()
        .filter_map(|file_score| {
            file_score.file_path.as_ref().map(|path| {
                json!({
                    "file": path.display().to_string(),
                    "score": file_score.total,
                    "grade": file_score.grade.to_string(),
                    "violation_count": file_score.penalties_applied.len(),
                    "complexity": file_score.structural_complexity,
                    "satd_count": file_score.penalties_applied.iter()
                        .filter(|p| p.issue.to_lowercase().contains("satd") || p.issue.to_lowercase().contains("todo"))
                        .count(),
                    "total_penalty": file_score.penalties_applied.iter()
                        .map(|p| p.amount)
                        .sum::<f32>(),
                })
            })
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": format!("Lint hotspot analysis completed ({} hotspots found)", hotspots.len()),
        "results": {
            "hotspots": hotspots,
            "total_files_analyzed": project_score.files.len(),
            "top_files_limit": top_files_limit,
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_churn(
    paths: &[PathBuf],
    days: Option<u32>,
    top_files: Option<usize>,
) -> Result<Value> {
    use crate::services::git_analysis::GitAnalysisService;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let days_value = days.unwrap_or(30);
    let top_files_value = top_files.unwrap_or(10);

    // Analyze churn for the first path (typically repository root)
    let repo_path = &paths[0];

    match GitAnalysisService::analyze_code_churn(repo_path, days_value) {
        Ok(mut analysis) => {
            // Apply top_files filtering
            analysis.files.sort_by(|a, b| {
                b.churn_score
                    .partial_cmp(&a.churn_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            analysis.files.truncate(top_files_value);

            // Transform to JSON
            Ok(json!({
                "status": "completed",
                "message": format!("Churn analysis completed for last {days_value} days"),
                "results": {
                    "period_days": analysis.period_days,
                    "total_commits": analysis.summary.total_commits,
                    "total_files_changed": analysis.summary.total_files_changed,
                    "files": analysis.files.iter().map(|f| json!({
                        "path": f.relative_path,
                        "commit_count": f.commit_count,
                        "unique_authors": f.unique_authors.len(),
                        "additions": f.additions,
                        "deletions": f.deletions,
                        "churn_score": f.churn_score,
                        "last_modified": f.last_modified.to_rfc3339(),
                    })).collect::<Vec<_>>(),
                    "hotspot_files": analysis.summary.hotspot_files.len(),
                }
            }))
        }
        Err(e) => Err(anyhow::anyhow!("Churn analysis failed: {e}")),
    }
}

/// The `dag_type` values `analyze_dag` accepts, in the spelling the tool's
/// `inputSchema` advertises.
pub const DAG_TYPES: [&str; 4] = [
    "call-graph",
    "import-graph",
    "inheritance",
    "full-dependency",
];

/// Resolve the `dag_type` argument, REJECTING anything not in [`DAG_TYPES`].
///
/// The match used to end in `_ => DagType::FullDependency`, so `"BOGUS"`, `""`
/// and `"12345"` all came back `status: "completed"` with
/// `results.dag_type: "FullDependency"` and no warning: a client that typo'd the
/// mode got a successful-looking result for a DIFFERENT analysis than the one it
/// asked for, and no way to tell. `generate_context` already rejects an
/// unsupported `format` and `scaffold_project` an unsupported `level`; silently
/// coercing an enum is the one behaviour a schema-declared `enum` promises will
/// not happen.
///
/// The underscore spellings stay accepted — they were accepted before, and
/// removing them would break callers for no gain.
pub fn parse_dag_type(dag_type: Option<&str>) -> Result<crate::services::deep_context::DagType> {
    use crate::services::deep_context::DagType;
    match dag_type.unwrap_or("full-dependency") {
        "call-graph" | "call_graph" => Ok(DagType::CallGraph),
        "import-graph" | "import_graph" => Ok(DagType::ImportGraph),
        "inheritance" => Ok(DagType::Inheritance),
        "full-dependency" | "full_dependency" => Ok(DagType::FullDependency),
        other => Err(anyhow::anyhow!(
            "Unsupported dag_type: {other:?} (expected one of {})",
            DAG_TYPES.join(", ")
        )),
    }
}

// --- DAG analysis (R17-1) ---
//
// Dispatches to `services::deep_context::analysis_functions::analyze_dag`,
// which builds a ProjectContext + DagBuilder graph. This is the analysis
// powering `pmat analyze dag`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag(paths: &[PathBuf], dag_type: Option<String>) -> Result<Value> {
    use crate::services::dag_complexity::{complexity_source, reported_complexity};
    use crate::services::deep_context::analysis_functions::{
        analyze_dag_detailed, dag_type_edge_types,
    };

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let dag_type_parsed = parse_dag_type(dag_type.as_deref())?;
    let dag_type_label = format!("{:?}", dag_type_parsed);
    let edge_types = dag_type_edge_types(dag_type_parsed.clone());

    let project_path = &paths[0];
    let (graph, stats) = analyze_dag_detailed(project_path, dag_type_parsed, None, None)
        .await
        .map_err(|e| anyhow::anyhow!("DAG analysis failed: {e}"))?;

    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();

    // Emit a compact summary plus the raw graph. Callers that want full
    // mermaid output can use the CLI's `pmat analyze dag` path.
    //
    // `complexity` is the McCabe cyclomatic number `analyze_complexity` reports
    // for the same function — it used to be a constant 1 wearing that name, so
    // this tool and `analyze_complexity` answered the same question two ways in
    // one process (#1020). `complexity_source` says which nodes carry a real
    // measurement, because a graph that cannot measure a node must say so
    // instead of quoting a placeholder.
    //
    // And when it says so, `complexity` is `null`. Emitting the struct's
    // neutral weight 1 there left the two fields contradicting each other in one
    // object — the number claimed a measurement the sibling field denied — and a
    // consumer that reads only `complexity` could not tell an unmeasurable
    // trait/module node from a function the analyzer scored 1.
    let top_nodes: Vec<Value> = graph
        .nodes
        .values()
        .take(25)
        .map(|n| {
            json!({
                "id": n.id,
                "label": n.label,
                "node_type": n.node_type,
                "file_path": n.file_path,
                "line_number": n.line_number,
                "complexity": reported_complexity(n),
                "complexity_source": complexity_source(n),
            })
        })
        .collect();

    // An empty graph reported as a completed analysis is absence rendered as
    // success. If nothing could be graphed, say what was seen and why.
    let empty_reason = stats.explain_empty(&graph, edge_types);
    let message = match &empty_reason {
        Some(reason) => format!("DAG analysis produced an empty {dag_type_label} graph: {reason}"),
        None => format!("DAG analysis completed ({node_count} nodes, {edge_count} edges)"),
    };

    Ok(json!({
        "status": "completed",
        "message": message,
        "results": {
            "dag_type": dag_type_label,
            "node_count": node_count,
            "edge_count": edge_count,
            "top_nodes": top_nodes,
            "empty_reason": empty_reason,
            "analyzed": {
                "files": stats.files_analyzed,
                "function_nodes": stats.function_nodes,
                "call_edges": stats.call_edges,
                "total_nodes": stats.total_nodes,
                "total_edges": stats.total_edges,
            },
        }
    }))
}

// --- Big-O analysis (R17-1) ---
//
// Dispatches to `services::deep_context::analysis_functions::analyze_big_o`,
// which classifies function time complexity via BigOAnalyzer. This is the
// analysis powering `pmat analyze big-o`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_big_o(paths: &[PathBuf], top_files: Option<usize>) -> Result<Value> {
    use crate::services::deep_context::analysis_functions::analyze_big_o as svc_analyze_big_o;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];
    let report = svc_analyze_big_o(project_path)
        .await
        .map_err(|e| anyhow::anyhow!("Big-O analysis failed: {e}"))?;

    let top_limit = top_files.unwrap_or(25);
    let high_complexity: Vec<Value> = report
        .high_complexity_functions
        .iter()
        .take(top_limit)
        .map(|f| {
            json!({
                "file_path": f.file_path,
                "function_name": f.function_name,
                "line_number": f.line_number,
                // `time_complexity` / `space_complexity` used to be the raw
                // `#[repr(C)]` ComplexityBound, so the payload carried the
                // struct's alignment padding and its packed flag byte
                // (`{"class":"Quadratic","coefficient":1,"input_var":"N",
                // "confidence":75,"flags":2,"_padding":[0,0]}`) to a reader
                // that cannot tell layout from measurement. The CLI's JSON
                // emits the notation and the confidence; emit the same.
                "time_complexity": f.time_complexity.notation(),
                "time_complexity_confidence": f.time_complexity.confidence,
                "space_complexity": f.space_complexity.notation(),
                "space_complexity_confidence": f.space_complexity.confidence,
                "confidence": f.confidence,
            })
        })
        .collect();

    // A LIST THAT IS SECRETLY A CAP: `top_files` truncated this silently, so
    // a caller asking for 2 saw 2 and had no way to learn there were 8. The
    // CLI names both numbers (`high_complexity_count` / `_found` /
    // `_truncated`); keep the two surfaces telling the same story.
    let listed = high_complexity.len();
    let dist = &report.complexity_distribution;
    let found = (dist.quadratic + dist.cubic + dist.exponential)
        .max(report.high_complexity_functions.len());

    Ok(json!({
        "status": "completed",
        "message": format!("Big-O analysis completed ({} functions analyzed)", report.analyzed_functions),
        "results": {
            "analyzed_functions": report.analyzed_functions,
            "complexity_distribution": report.complexity_distribution,
            "high_complexity_functions": high_complexity,
            "high_complexity_count": listed,
            "high_complexity_found": found,
            "high_complexity_truncated": listed < found,
            "recommendations": report.recommendations,
        }
    }))
}

// --- Deep context analysis (R17-1) ---
//
// Dispatches to `services::deep_context::DeepContextAnalyzer`, which runs the
// full multi-phase deep context pipeline. This is the analysis powering
// `pmat context` / `pmat analyze deep-context`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_deep_context(
    paths: &[PathBuf],
    include_patterns: Option<Vec<String>>,
) -> Result<Value> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    reject_unsupported_include_patterns(include_patterns.as_deref())?;

    let project_path = &paths[0];
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let context = analyzer
        .analyze_project(&project_path.to_path_buf())
        .await
        .map_err(|e| anyhow::anyhow!("Deep context analysis failed: {e}"))?;

    Ok(json!({
        "status": "completed",
        "message": format!("Deep context analysis completed ({} files)", context.file_tree.total_files),
        "results": {
            "metadata": {
                "project_root": context.metadata.project_root,
                "tool_version": context.metadata.tool_version,
                "generated_at": context.metadata.generated_at.to_rfc3339(),
                "analysis_duration_ms": context.metadata.analysis_duration.as_millis(),
            },
            "quality_scorecard": quality_scorecard_json(&context.quality_scorecard),
            "file_count": context.file_tree.total_files,
            "ast_contexts": context.analyses.ast_contexts.len(),
        }
    }))
}

/// Refuse `include_patterns` rather than accept it and ignore it.
///
/// `analyze_deep_context {"paths":[dir]}` and
/// `analyze_deep_context {"paths":[dir],"include_patterns":["*.py"]}` returned
/// the SAME `file_count: 3` over a directory holding `a.go app.ts main.py`: the
/// argument parsed, was bound to `_include_patterns`, and was thrown away. The
/// tool's own schema described it as "accepted but not yet applied as a filter",
/// which is a defect annotated and shipped rather than fixed.
///
/// It cannot be wired up from here. `DeepContextConfig::include_patterns` exists
/// but is READ BY NOBODY — `grep -rn include_patterns src/services/deep_context/`
/// finds the declaration, the `Default` initialiser and nothing else, so setting
/// it would be a second no-op stacked on the first. Only `exclude_patterns` is
/// honoured, and only by the file-tree walk (`analyzer_core/file_tree.rs`), not
/// by the analysis phase that produces `ast_contexts` and the scorecard. Post-
/// filtering the results here would be worse than useless: `quality_scorecard`
/// is computed inside the pipeline over the WHOLE tree, so a filtered
/// `file_count` beside an unfiltered scorecard is a payload that reads as
/// measured and is not.
///
/// So the parameter is refused, loudly and by name, with the operation that
/// actually narrows the analysis. Wiring it for real needs
/// `src/services/deep_context/analyzer_core/` to honour the config field — the
/// CLI's `pmat analyze deep-context --include` sets that same dead field
/// (`src/cli/handlers/advanced_analysis_handlers.rs:272`) and is equally inert.
fn reject_unsupported_include_patterns(patterns: Option<&[String]>) -> Result<()> {
    let Some(patterns) = patterns.filter(|p| !p.is_empty()) else {
        return Ok(());
    };
    Err(anyhow::anyhow!(
        "include_patterns is not supported by analyze_deep_context: the deep-context \
         pipeline has no file filter, so applying {patterns:?} would change nothing about \
         the analysis. Rejecting instead of silently ignoring it. Narrow the analysis by \
         passing the subdirectory you want in `paths`, or use analyze_complexity / \
         analyze_satd, which do filter."
    ))
}

/// Serialise a quality scorecard for MCP consumers.
///
/// Unmeasured fields are `null` AND listed by name in `not_measured`, because
/// this payload is read by models that cannot tell a measurement from a
/// placeholder. Every one of these was a constant: six wildly different code
/// bases — a 5-file toy fixture and pmat's own 3891-file tree among them — all
/// returned maintainability_index 70.0, modularity_score 85.0 and
/// technical_debt_hours 40.0, while file_count and ast_contexts varied
/// correctly, which is exactly what made the scorecard read as measured
/// (GH #667, same root cause as #643 on the CLI side).
///
/// `test_coverage` is included here; the MCP payload used to omit it entirely
/// while the CLI reported it.
fn quality_scorecard_json(
    scorecard: &crate::services::deep_context::QualityScorecard,
) -> serde_json::Value {
    let fields = [
        ("overall_health", scorecard.overall_health),
        ("complexity_score", scorecard.complexity_score),
        ("maintainability_index", scorecard.maintainability_index),
        ("modularity_score", scorecard.modularity_score),
        ("test_coverage", scorecard.test_coverage),
        ("technical_debt_hours", scorecard.technical_debt_hours),
    ];

    let mut object = serde_json::Map::new();
    let mut not_measured = Vec::new();
    for (name, value) in fields {
        if value.is_none() {
            not_measured.push(serde_json::Value::from(name));
        }
        object.insert(name.to_string(), json!(value));
    }
    object.insert(
        "not_measured".to_string(),
        serde_json::Value::Array(not_measured),
    );
    serde_json::Value::Object(object)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_coupling(paths: &[PathBuf], threshold: Option<f64>) -> Result<Value> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
    use std::collections::HashMap;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];
    let threshold_value = threshold.unwrap_or(0.5);

    // Use deep context analyzer to get AST contexts
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let context = analyzer.analyze_project(project_path).await?;

    // Analyze coupling from AST contexts
    let mut file_metrics: HashMap<String, (usize, usize, f64)> = HashMap::new();

    // Build import map for afferent coupling calculation
    let mut all_imports: HashMap<String, Vec<String>> = HashMap::new();
    for ast_context in &context.analyses.ast_contexts {
        let file_path = ast_context.base.path.clone();
        let imports: Vec<String> = ast_context
            .base
            .items
            .iter()
            .filter_map(|item| match item {
                crate::services::context::AstItem::Use { path, .. } => Some(path.clone()),
                crate::services::context::AstItem::Import { module, .. } => Some(module.clone()),
                _ => None,
            })
            .collect();
        all_imports.insert(file_path, imports);
    }

    // Calculate metrics
    for (file, imports) in &all_imports {
        let efferent = imports.len();
        let afferent = all_imports
            .values()
            .filter(|deps| deps.iter().any(|d| d.contains(file) || file.contains(d)))
            .count();
        let total = afferent + efferent;
        let instability = if total > 0 {
            efferent as f64 / total as f64
        } else {
            0.0
        };

        file_metrics.insert(file.clone(), (afferent, efferent, instability));
    }

    // Filter by threshold and build coupling entries
    let couplings: Vec<Value> = file_metrics
        .iter()
        .filter(|(_, (_, _, instability))| *instability >= threshold_value)
        .map(|(file, (afferent, efferent, instability))| {
            json!({
                "file": file,
                "afferent_coupling": afferent,
                "efferent_coupling": efferent,
                "instability": instability,
                "strength": afferent + efferent,
            })
        })
        .collect();

    // Calculate project-level metrics
    let avg_afferent = if !file_metrics.is_empty() {
        file_metrics.values().map(|(a, _, _)| *a).sum::<usize>() as f64 / file_metrics.len() as f64
    } else {
        0.0
    };
    let avg_efferent = if !file_metrics.is_empty() {
        file_metrics.values().map(|(_, e, _)| *e).sum::<usize>() as f64 / file_metrics.len() as f64
    } else {
        0.0
    };
    let max_afferent = file_metrics.values().map(|(a, _, _)| *a).max().unwrap_or(0);
    let max_efferent = file_metrics.values().map(|(_, e, _)| *e).max().unwrap_or(0);

    Ok(json!({
        "status": "completed",
        "message": format!("Coupling analysis completed ({} files analyzed)", file_metrics.len()),
        "results": {
            "couplings": couplings,
            "total_files": file_metrics.len(),
            "threshold": threshold_value,
            "project_metrics": {
                "avg_afferent": avg_afferent,
                "avg_efferent": avg_efferent,
                "max_afferent": max_afferent,
                "max_efferent": max_efferent,
            }
        }
    }))
}

#[cfg(test)]
mod big_o_payload_tests {
    //! The tool serialised the raw `#[repr(C)]` ComplexityBound and truncated
    //! its function list without saying so.
    use super::*;

    /// Three functions with a doubly-nested loop each: enough for `top_files`
    /// to bite.
    fn quadratic_fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut src = String::new();
        for i in 0..3 {
            src.push_str(&format!(
                "pub fn pairs{i}(xs: &[usize]) -> usize {{\n    let mut acc = 0;\n    for a in xs {{\n        for b in xs {{\n            acc += a * b;\n        }}\n    }}\n    acc\n}}\n\n"
            ));
        }
        std::fs::write(dir.path().join("lib.rs"), src).expect("write lib.rs");
        dir
    }

    #[tokio::test]
    async fn test_big_o_payload_has_no_struct_layout_fields() {
        let dir = quadratic_fixture();
        let value = analyze_big_o(&[dir.path().to_path_buf()], Some(1))
            .await
            .expect("analysis");
        let payload = serde_json::to_string(&value).expect("serialise");

        assert!(
            !payload.contains("_padding"),
            "alignment filler leaked into the MCP payload: {payload}"
        );
        let functions = value["results"]["high_complexity_functions"]
            .as_array()
            .expect("array");
        assert!(
            !functions.is_empty(),
            "the fixture must produce a high-complexity function, or this asserts nothing"
        );
        for func in functions {
            // A notation string ("O(n²)"), not the packed bound struct.
            let time = func["time_complexity"]
                .as_str()
                .unwrap_or_else(|| panic!("time_complexity must be a notation string: {func}"));
            assert!(time.starts_with("O("), "unexpected notation {time:?}");
            assert!(func["time_complexity_confidence"].is_number());
            assert!(func["space_complexity"].is_string());
        }
    }

    /// `top_files` capped the list silently: asking for 1 of 3 returned 1,
    /// with nothing in the payload naming the other two.
    #[tokio::test]
    async fn test_big_o_payload_discloses_truncation() {
        let dir = quadratic_fixture();
        let results = analyze_big_o(&[dir.path().to_path_buf()], Some(1))
            .await
            .expect("analysis");
        let results = &results["results"];

        let listed = results["high_complexity_functions"]
            .as_array()
            .expect("array")
            .len();
        let count = results["high_complexity_count"].as_u64().expect("count");
        let found = results["high_complexity_found"].as_u64().expect("found");

        assert_eq!(count as usize, listed, "count must be the listed length");
        // The fixture holds three quadratic functions and asks for one, so the
        // cap must be visible rather than inferred.
        assert_eq!(count, 1, "top_files=1 must list exactly one");
        assert!(
            found > count,
            "found ({found}) must exceed the listed count ({count}) for this fixture"
        );
        assert_eq!(
            results["high_complexity_truncated"],
            serde_json::Value::Bool(count < found),
            "the truncation flag must follow the two counts"
        );
    }
}

#[cfg(test)]
mod dead_code_include_tests_tests {
    //! `include_tests` used to be an unused parameter (`_include_tests`).
    use super::*;

    /// Passing include_tests:true returned a byte-identical response, because
    /// the tool signature was `analyze_dead_code(paths, _include_tests: bool)`.
    #[tokio::test]
    async fn test_include_tests_changes_the_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::create_dir_all(dir.path().join("tests")).expect("mkdir tests");
        // A REAL crate. Without a manifest there is no cargo target to check,
        // and the analysis this tool now shares with the CLI is rooted in one.
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname=\"mcp_include_tests\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .expect("write Cargo.toml");
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn used() {}\npub fn never_called() {}\nfn main() { used(); }\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            dir.path().join("tests/helper.rs"),
            "fn only_in_tests() {}\n",
        )
        .expect("write tests/helper.rs");
        crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(dir.path());

        let paths = vec![dir.path().to_path_buf()];
        let without = analyze_dead_code(&paths, false).await.expect("analysis");
        let with = analyze_dead_code(&paths, true).await.expect("analysis");

        let count = |v: &Value| v["results"]["total_dead_code"].as_u64().unwrap_or(0);
        assert!(
            count(&with) > count(&without),
            "include_tests made no difference: {} vs {}",
            count(&without),
            count(&with)
        );
        // `pub fn never_called` is a LIBRARY's public API and must not be
        // called dead by either surface — the false positive that made this
        // tool report all 50 items in `src/models` dead.
        let named: Vec<String> = with["results"]["files"]
            .as_array()
            .expect("files array")
            .iter()
            .flat_map(|f| {
                f["dead_functions"]
                    .as_array()
                    .expect("dead_functions")
                    .iter()
            })
            .map(|i| i["name"].as_str().expect("name").to_string())
            .collect();
        assert!(
            !named.contains(&"never_called".to_string()),
            "a library's public API was reported dead: {named:?}"
        );
        assert_eq!(
            without["results"]["analyzer"].as_str(),
            Some("pmat analyze dead-code"),
            "the payload must name which analyzer produced these numbers"
        );
        assert_eq!(
            without["results"]["engines"]
                .as_array()
                .expect("engines")
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>(),
            vec!["cargo-dead-code"],
            "a Rust crate must be answered by cargo's dead-code pass"
        );
    }

    /// The other engine, and the same flag.
    ///
    /// `--include-tests` reached the multi-language path and was never read
    /// there: this tool applied a test-path filter of its own ON TOP of the
    /// analyzer, so the flag worked over MCP and was inert on the CLI for every
    /// non-Rust project. One predicate now, inside the shared runner.
    #[tokio::test]
    async fn test_include_tests_changes_the_result_without_cargo() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("tests")).expect("mkdir tests");
        std::fs::write(
            dir.path().join("main.py"),
            "def used():\n    return 1\n\n\ndef never_called():\n    return 2\n\n\nused()\n",
        )
        .expect("write main.py");
        std::fs::write(
            dir.path().join("tests/test_helper.py"),
            "def only_in_tests():\n    return 3\n",
        )
        .expect("write tests/test_helper.py");

        let paths = vec![dir.path().to_path_buf()];
        let without = analyze_dead_code(&paths, false).await.expect("analysis");
        let with = analyze_dead_code(&paths, true).await.expect("analysis");

        let names = |v: &Value| {
            let mut found: Vec<String> = v["results"]["files"]
                .as_array()
                .expect("files array")
                .iter()
                .flat_map(|f| {
                    f["dead_functions"]
                        .as_array()
                        .expect("dead_functions")
                        .iter()
                })
                .map(|i| i["name"].as_str().expect("name").to_string())
                .collect();
            found.sort();
            found
        };

        assert_eq!(
            names(&without),
            vec!["never_called".to_string()],
            "the test tree is out of scope by default"
        );
        assert_eq!(
            names(&with),
            vec!["never_called".to_string(), "only_in_tests".to_string()],
            "--include-tests must add the test tree's findings"
        );
        assert_eq!(
            with["results"]["engines"]
                .as_array()
                .expect("engines")
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>(),
            vec!["multi-language-reachability"],
            "a Python tree must be answered by the reachability analyzer"
        );
    }
}

#[cfg(test)]
mod satd_surface_agreement_tests {
    //! REGRESSION (#997): the MCP tool and `analyze satd` answered the same
    //! question differently, and the MCP schema offered no way to ask for the
    //! CLI's answer — so neither surface could be made to agree with the other.
    //!
    //! ```text
    //! MCP  analyze_satd {"paths":[fixture]}      -> 3
    //! CLI  analyze satd -p fixture               -> 2
    //! CLI  analyze satd -p fixture --include-tests -> 3
    //! ```
    //!
    //! This tool had no concept of a test file at all: it walked
    //! `expand_paths_to_source_files` and scanned whatever came back. It is the
    //! THIRD independent SATD implementation — the CLI goes through
    //! `SatdFacade::analyze_directory_with_tests`, and
    //! `analysis_service_analyzers::analyze_satd` hardcodes `true` while
    //! ignoring its own options argument.
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("src");
        std::fs::create_dir_all(d.path().join("tests")).expect("tests");
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname=\"f\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(
            d.path().join("src/lib.rs"),
            "// TODO: production marker\n// FIXME: production fixme\npub fn f() -> i32 { 1 }\n",
        )
        .expect("lib");
        std::fs::write(
            d.path().join("tests/it.rs"),
            "// TODO: integration-test marker\n#[test] fn t() { assert_eq!(1,1); }\n",
        )
        .expect("it");
        d
    }

    fn total(v: &serde_json::Value) -> u64 {
        v.get("results")
            .and_then(|r| r.get("total_satd"))
            .and_then(serde_json::Value::as_u64)
            .expect("total_satd in the payload")
    }

    /// The default must match the CLI's default: production only.
    #[tokio::test]
    async fn default_excludes_test_files_like_the_cli() {
        let d = fixture();
        let out = analyze_satd(&[d.path().to_path_buf()], false, false)
            .await
            .expect("analysis");
        assert_eq!(
            total(&out),
            2,
            "MCP default must agree with `analyze satd` (2 production markers): {out}"
        );
    }

    /// And the flag must be able to reach the CLI's `--include-tests` answer.
    #[tokio::test]
    async fn include_tests_reaches_the_cli_include_tests_answer() {
        let d = fixture();
        let out = analyze_satd(&[d.path().to_path_buf()], false, true)
            .await
            .expect("analysis");
        assert_eq!(
            total(&out),
            3,
            "include_tests must agree with `--include-tests` (3 markers): {out}"
        );
    }

    /// REGRESSION: `include_tests` has TWO effects and they must move together.
    ///
    /// It selects test FILES, and it reaches the inline `#[cfg(test)]` skip
    /// inside each file. #997 wired the first; #995 had already fixed the
    /// second on the CLI path. Neither test noticed that the MCP path still
    /// called the plain `extract_from_content`, so on a fixture with BOTH kinds
    /// of test debt the surfaces disagreed again — MCP 3, CLI 4 — and both
    /// PRs were green.
    ///
    /// This fixture deliberately carries both kinds, which is what the two
    /// earlier fixtures each lacked.
    #[tokio::test]
    async fn include_tests_reaches_both_test_files_and_inline_blocks() {
        let d = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("src");
        std::fs::create_dir_all(d.path().join("tests")).expect("tests");
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname=\"f\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .expect("manifest");
        // 1 production marker + 1 INLINE test marker
        std::fs::write(
            d.path().join("src/lib.rs"),
            "// TODO: production\npub fn f() -> i32 { 1 }\n\n#[cfg(test)]\nmod tests {\n    // TODO: inline test debt\n    #[test] fn t() { assert_eq!(1, 1); }\n}\n",
        )
        .expect("lib");
        // 1 marker in a test FILE
        std::fs::write(
            d.path().join("tests/it.rs"),
            "// TODO: test-file debt\n#[test] fn w() { assert_eq!(2, 2); }\n",
        )
        .expect("it");

        let off = total(
            &analyze_satd(&[d.path().to_path_buf()], false, false)
                .await
                .unwrap(),
        );
        let on = total(
            &analyze_satd(&[d.path().to_path_buf()], false, true)
                .await
                .unwrap(),
        );
        assert_eq!(off, 1, "default is production only");
        assert_eq!(
            on, 3,
            "include_tests must reach the test FILE and the INLINE block — \
             1 production + 1 inline + 1 test-file"
        );
    }

    /// The two must never be the same number, or the test above proves nothing
    /// about the flag.
    #[tokio::test]
    async fn the_flag_actually_moves_the_answer() {
        let d = fixture();
        let off = total(
            &analyze_satd(&[d.path().to_path_buf()], false, false)
                .await
                .unwrap(),
        );
        let on = total(
            &analyze_satd(&[d.path().to_path_buf()], false, true)
                .await
                .unwrap(),
        );
        assert!(on > off, "include_tests changed nothing: {off} vs {on}");
    }
}

#[cfg(test)]
mod dag_tool_surface_tests {
    //! REGRESSION (#1020): the MCP `analyze_dag` tool answered two questions
    //! wrongly, and the second one it answered twice.
    //!
    //! ```text
    //! analyze_dag {paths:[src/services], dag_type:"call-graph"}
    //!   -> node_count 0, edge_count 0, top_nodes []
    //! analyze_dag {paths:[src/services], dag_type:"full-dependency"}
    //!   -> node_count 369, edge_count 400        (same path, same process)
    //!
    //! analyze_dag        top_nodes[branchy].complexity -> 1
    //! analyze_complexity branchy cyclomatic            -> 7
    //! ```
    use super::*;

    fn branchy_fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("branchy.rs"),
            r"
pub fn caller() {
    branchy(1, 2);
}

pub fn branchy(a: i32, b: i32) -> i32 {
    if a > 0 && b > 0 {
        return 1;
    }
    if a < 0 || b < 0 {
        return 2;
    }
    for i in 0..a {
        if i == b {
            return 3;
        }
    }
    match a {
        0 => 4,
        _ => 5,
    }
}
",
        )
        .expect("write fixture");
        dir
    }

    /// The two tools must not answer the same question two ways in one process.
    #[tokio::test]
    async fn dag_node_complexity_matches_analyze_complexity_in_the_same_process() {
        let dir = branchy_fixture();
        let paths = vec![dir.path().to_path_buf()];

        let complexity = analyze_complexity(&paths, Some(50), Some(1000))
            .await
            .expect("analyze_complexity must succeed");
        let expected = complexity["results"]["top_files"]
            .as_array()
            .expect("top_files")
            .iter()
            .find(|f| f["function"] == "branchy")
            .expect("analyze_complexity must see branchy")["cyclomatic_complexity"]
            .as_u64()
            .expect("cyclomatic_complexity");
        assert!(
            expected > 1,
            "fixture must be branchy enough to tell a measurement from the old constant 1"
        );

        let dag = analyze_dag(&paths, Some("call-graph".to_string()))
            .await
            .expect("analyze_dag must succeed");
        let node = dag["results"]["top_nodes"]
            .as_array()
            .expect("top_nodes")
            .iter()
            .find(|n| n["label"].as_str().is_some_and(|l| l.ends_with("branchy")))
            .expect("branchy must appear in the call graph");

        assert_eq!(
            node["complexity"].as_u64(),
            Some(expected),
            "analyze_dag says {} for branchy, analyze_complexity says {expected}",
            node["complexity"]
        );
        assert_eq!(
            node["complexity_source"].as_str(),
            Some(crate::services::dag_complexity::SOURCE_CYCLOMATIC),
            "a node must say where its number came from"
        );
    }

    /// An empty graph must explain itself instead of being reported as a
    /// completed analysis of nothing.
    #[tokio::test]
    async fn an_empty_graph_is_explained_rather_than_reported_as_success() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("readme.txt"), "not code\n").expect("write");

        let dag = analyze_dag(&[dir.path().to_path_buf()], Some("call-graph".to_string()))
            .await
            .expect("analyze_dag must succeed");

        assert_eq!(dag["results"]["node_count"].as_u64(), Some(0));
        let reason = dag["results"]["empty_reason"]
            .as_str()
            .expect("an empty graph must carry empty_reason");
        assert!(!reason.is_empty(), "empty_reason must say something");
        assert!(
            dag["message"].as_str().unwrap_or_default().contains(reason),
            "the message must carry the reason, got {}",
            dag["message"]
        );
    }

    /// A tree whose full-dependency graph holds all four node kinds: a branchy
    /// function (measurable), plus a struct, a trait and a module (nothing the
    /// complexity analyzer is asked about).
    fn mixed_node_kinds_fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("shapes.rs"),
            r"
pub struct Widget {
    pub id: u32,
    pub name: String,
}

pub trait Draw {
    fn draw(&self);
}

pub mod inner {
    pub fn helper() {}
}

pub fn branchy(a: i32, b: i32) -> i32 {
    if a > 0 && b > 0 {
        return 1;
    }
    if a < 0 || b < 0 {
        return 2;
    }
    for i in 0..a {
        if i == b {
            return 3;
        }
    }
    match a {
        0 => 4,
        _ => 5,
    }
}
",
        )
        .expect("write fixture");
        dir
    }

    /// REGRESSION: a node said "nobody measured this" and quoted a number in
    /// the same breath.
    ///
    /// `NodeInfo::complexity` is a non-optional `u32`, so a struct/trait/module
    /// node — which the complexity annotator never touches — carried the
    /// neutral weight 1, and the payload serialised it verbatim:
    ///
    /// ```text
    /// {"label":"Widget","complexity":1,"complexity_source":"not-measured"}
    /// ```
    ///
    /// A consumer reading `complexity` gets the number 1, which is a
    /// measurement; the sibling field says no measurement exists. Two fields in
    /// one object contradicting each other is the shape #928 fixed. Absence has
    /// to be REPRESENTABLE — `null` — not disguised as a plausible value.
    #[tokio::test]
    async fn unmeasured_nodes_report_null_complexity_not_the_placeholder_one() {
        let dir = mixed_node_kinds_fixture();

        let dag = analyze_dag(
            &[dir.path().to_path_buf()],
            Some("full-dependency".to_string()),
        )
        .await
        .expect("analyze_dag must succeed");

        let nodes = dag["results"]["top_nodes"]
            .as_array()
            .expect("top_nodes")
            .clone();
        assert!(!nodes.is_empty(), "fixture produced no graph: {dag}");

        let unmeasured: Vec<&Value> = nodes
            .iter()
            .filter(|n| n["complexity_source"] == json!("not-measured"))
            .collect();
        let measured: Vec<&Value> = nodes
            .iter()
            .filter(|n| n["complexity_source"] != json!("not-measured"))
            .collect();

        // Both halves must be present, or the assertions below are vacuous:
        // an all-null payload would pass a "nothing is 1" check by accident.
        assert!(
            !unmeasured.is_empty(),
            "fixture must contain a node nobody measured (struct/trait/module): {nodes:#?}"
        );
        assert!(
            !measured.is_empty(),
            "fixture must contain a measured function, or `null` proves nothing: {nodes:#?}"
        );

        for node in &unmeasured {
            assert!(
                node["complexity"].is_null(),
                "{} says complexity_source=not-measured and complexity={} in the same object",
                node["label"],
                node["complexity"]
            );
        }
        for node in &measured {
            let n = node["complexity"]
                .as_u64()
                .unwrap_or_else(|| panic!("a measured node must carry a number: {node}"));
            assert!(n >= 1, "a cyclomatic measurement is at least 1: {node}");
        }

        // And the number that IS reported is still the analyzer's, not a
        // constant — the #1020 half of the contract, restated here so a fix
        // that nulls everything cannot pass.
        let branchy = measured
            .iter()
            .find(|n| n["label"].as_str().is_some_and(|l| l.ends_with("branchy")))
            .expect("branchy must be measured");
        assert!(
            branchy["complexity"].as_u64().is_some_and(|c| c > 1),
            "branchy is not branchy in this payload: {branchy}"
        );
    }
}

#[cfg(test)]
mod satd_skip_accounting_tests {
    //! REGRESSION: `analyze_satd` over MCP reported a count with no
    //! denominator.
    //!
    //! ```text
    //! CLI  analyze satd -p fixture --format json
    //!        total_violations: 2
    //!        files_not_read: {total: 14, tests: 1, examples_…: 13, …}
    //!        violations_truncated: false
    //! MCP  analyze_satd {"paths":[fixture]}
    //!        total_satd: 2
    //!        (nothing at all about the 14 files it declined to read)
    //! ```
    //!
    //! Both surfaces agreed on 2, and only one of them could say what 2 was
    //! measured over — so over MCP "this tree is clean" and "almost every file
    //! in this tree was skipped" were the same payload. This is the defect
    //! #1015 fixed for `analyze satd`, one surface later.
    //!
    //! #1035 then moved `examples/` OUT of the declined population on both
    //! surfaces — it is shipped, compiled code — so the fixture's 13 example
    //! markers are findings now rather than a skip bucket, and the payload
    //! carries `files_discovered`/`files_analyzed` so the buckets can be checked
    //! to partition instead of merely listed.
    use super::*;

    /// 2 production markers, 1 in a test file, 13 in `examples/`.
    fn fixture() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("src");
        std::fs::create_dir_all(d.path().join("tests")).expect("tests");
        std::fs::create_dir_all(d.path().join("examples")).expect("examples");
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname=\"f\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(
            d.path().join("src/lib.rs"),
            "// TODO: production marker\n// FIXME: production fixme\npub fn f() -> i32 { 1 }\n",
        )
        .expect("lib");
        std::fs::write(
            d.path().join("tests/it.rs"),
            "// TODO: integration-test marker\n#[test] fn t() { assert_eq!(1,1); }\n",
        )
        .expect("it");
        for i in 0..13 {
            std::fs::write(
                d.path().join(format!("examples/e{i}.rs")),
                format!("// TODO: example marker {i}\nfn main() {{}}\n"),
            )
            .expect("example");
        }
        d
    }

    async fn mcp_payload(dir: &std::path::Path, include_tests: bool) -> Value {
        analyze_satd(&[dir.to_path_buf()], false, include_tests)
            .await
            .expect("analysis")
    }

    /// The accounting must be there, and it must be right.
    #[tokio::test]
    async fn the_payload_discloses_what_it_declined_to_read() {
        let d = fixture();
        let out = mcp_payload(d.path(), false).await;
        let results = &out["results"];

        assert_eq!(
            results["total_satd"], 15,
            "2 in src/ and 13 in examples/, which is shipped code (#1035): {out}"
        );

        let not_read = &results["files_not_read"];
        assert!(
            !not_read.is_null(),
            "a count with no denominator: the payload says nothing about the \
             files it did not read: {out}"
        );
        assert_eq!(not_read["total"], 1, "{out}");
        assert_eq!(
            not_read["tests"], 1,
            "the one file in tests/ was declined, and must be disclosed: {out}"
        );
        assert_eq!(
            not_read["out_of_scope"], 0,
            "nothing here is vendored, generated or a fuzz harness: {out}"
        );
        assert_eq!(not_read["minified_or_vendor"], 0, "{out}");
        assert_eq!(not_read["too_large"], 0, "{out}");
        assert_eq!(not_read["unreadable"], 0, "{out}");
        assert_eq!(
            not_read["oversized"],
            json!([]),
            "nothing was declined for size, so nothing may be named: {out}"
        );
        assert_eq!(
            results["files_analyzed"], 14,
            "and how many WERE read — 1 declined out of 15: {out}"
        );
        assert_eq!(
            results["files_discovered"], 15,
            "the denominator the buckets partition: {out}"
        );
        assert_eq!(
            results["census_balances"],
            Value::Bool(true),
            "analysed + not read must equal walked: {out}"
        );
        assert_eq!(results["files_unaccounted"], 0, "{out}");
        assert_eq!(
            results["violations_truncated"],
            Value::Bool(false),
            "this surface never elides a finding, and must say so rather than \
             leave the caller to assume it: {out}"
        );
    }

    /// The numbers are not decoration: `--include-tests` moves them.
    ///
    /// Without this, a hardcoded `files_not_read` block would pass the test
    /// above.
    #[tokio::test]
    async fn the_accounting_tracks_what_the_flag_actually_did() {
        let d = fixture();
        let without = mcp_payload(d.path(), false).await;
        let with = mcp_payload(d.path(), true).await;

        assert_eq!(without["results"]["files_not_read"]["tests"], 1);
        assert_eq!(
            with["results"]["files_not_read"]["tests"], 0,
            "with --include-tests nothing is declined for being a test: {with}"
        );
        assert_eq!(
            with["results"]["total_satd"], 16,
            "and the test file's marker is now counted: {with}"
        );
        assert_eq!(
            with["results"]["files_analyzed"], 15,
            "one more file was read: {with}"
        );
        assert_eq!(
            with["results"]["files_discovered"], without["results"]["files_discovered"],
            "a flag that only widens the scan must not move the denominator: {with}"
        );
    }

    /// The whole point of the shape: an MCP consumer must be able to compare
    /// it with the CLI's. Same fixture, same tree, same numbers.
    #[tokio::test]
    async fn the_accounting_matches_the_cli_field_for_field() {
        use crate::services::facades::satd_facade::{SatdAnalysisRequest, SatdFacade};
        use crate::services::service_registry::ServiceRegistry;

        let d = fixture();
        let mcp = mcp_payload(d.path(), false).await;

        let cli = SatdFacade::new(std::sync::Arc::new(ServiceRegistry::new()))
            .analyze_project(SatdAnalysisRequest {
                path: d.path().to_path_buf(),
                strict_mode: false,
                include_tests: false,
                extended: false,
            })
            .await
            .expect("cli analysis");

        assert_eq!(
            mcp["results"]["total_satd"].as_u64().expect("total_satd") as usize,
            cli.violations.len(),
            "the two surfaces must still agree on the count"
        );
        assert_eq!(
            mcp["results"]["files_not_read"],
            json!({
                "total": cli.census.not_read.total(),
                "tests": cli.census.not_read.tests,
                "out_of_scope": cli.census.not_read.out_of_scope,
                "minified_or_vendor": cli.census.not_read.minified_or_vendor,
                "too_large": cli.census.not_read.too_large,
                "unreadable": cli.census.not_read.unreadable,
                "oversized": []
            }),
            "…and now also on the denominator: {mcp}"
        );
        assert_eq!(
            mcp["results"]["files_analyzed"].as_u64().expect("files_analyzed") as usize,
            cli.census.analyzed,
            "one spelling, both transports (#1058): {mcp}"
        );
        assert_eq!(
            mcp["results"]["files_discovered"].as_u64().expect("files_discovered") as usize,
            cli.census.discovered,
            "…over the same population: {mcp}"
        );
    }
}

/// Issue #1058 — the transport-parity gate's first real finding.
///
/// Two transports, one binary, one repository, two answers:
///
/// ```text
///   repo    probe                     CLI   MCP stdio
///   copia   analyze complexity files   41          39
///   copia   analyze dead-code files    38          29
///   pzsh    analyze dead-code files    29          28
/// ```
///
/// The filed hypothesis was sub-crate traversal (copia has two `Cargo.toml`
/// files and shows the largest gap). Both halves of it are FALSIFIED here, on
/// single-crate fixtures:
///
/// * complexity — a real population split, from two hand-maintained extension
///   allow-lists that had drifted in BOTH directions. See
///   [`crate::services::path_glob::expand_paths_to_complexity_files`].
/// * dead-code — not a population split at all. The two surfaces agree
///   exactly; they published the same number under different key names while
///   the CLI ALSO published a second, larger number (`total_files`, the walk's
///   denominator) that this payload had no counterpart for. A parity check
///   reading "files" got 38 from one and 29 from the other, and both were
///   right about their own field.
#[cfg(test)]
mod transport_parity_1058_tests {
    use super::*;

    /// Every extension in one fixture, so a drift in either direction shows.
    fn mixed_language_fixture() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("src");
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname=\"ef\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(
            d.path().join("src/lib.rs"),
            "pub fn a(x: i32) -> i32 { if x > 0 { x } else { -x } }\n",
        )
        .expect("lib");
        // Admitted by the MCP list, refused by the CLI's.
        std::fs::write(
            d.path().join("run.sh"),
            "#!/bin/bash\nf(){ if [ 1 ]; then echo a; else echo b; fi; }\n",
        )
        .expect("sh");
        std::fs::write(d.path().join("hdr.h"), "int h(int x){ return x; }\n").expect("h");
        // Admitted by the CLI's list, refused by the MCP one.
        std::fs::write(d.path().join("proof.lean"), "theorem t : 1 = 1 := rfl\n").expect("lean");
        d
    }

    /// RED CONTROL: pointing `analyze_complexity` back at
    /// `expand_paths_to_source_files` fails this with 3 against 2 — the exact
    /// numbers the fixture produced against the shipped 3.32.0 binary.
    #[tokio::test]
    async fn complexity_measures_the_same_population_as_the_cli() {
        let d = mixed_language_fixture();
        let root = d.path().to_path_buf();

        // The CLI's own population builder — not a restatement of its list.
        let cli = crate::cli::analysis_utilities::analyze_project_files(&root, None, &[], 10, 10)
            .await
            .expect("cli walk");

        let mcp = analyze_complexity(std::slice::from_ref(&root), Some(50), Some(10))
            .await
            .expect("mcp analysis");
        let mcp_total = mcp["results"]["total_files"]
            .as_u64()
            .expect("total_files in the payload") as usize;

        assert_eq!(
            mcp_total,
            cli.len(),
            "same repo, same question, two answers: CLI measured {} file(s), MCP {mcp_total} — {mcp}",
            cli.len()
        );
    }

    /// COUNTER-TEST. The lazy over-correction — admitting everything, or
    /// admitting nothing — would pass the equality above by making both sides
    /// wrong together. The population has to be the RIGHT one: the `.lean`
    /// file is measured, the `.sh` and `.h` files are not.
    #[test]
    fn the_complexity_population_is_the_cli_one_not_merely_a_matching_one() {
        let d = mixed_language_fixture();
        let files =
            crate::services::path_glob::expand_paths_to_complexity_files(&[d.path().to_path_buf()]);
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();

        assert!(
            names.contains(&"proof.lean".to_string()),
            "the CLI measures .lean; this surface must too, got {names:?}"
        );
        assert!(
            names.contains(&"lib.rs".to_string()),
            "the ordinary case must not be lost, got {names:?}"
        );
        assert!(
            !names.contains(&"run.sh".to_string()),
            "the CLI does not measure .sh; admitting it is the old split with \
             the sign flipped, got {names:?}"
        );
        assert!(
            !names.contains(&"hdr.h".to_string()),
            "the CLI does not measure .h, got {names:?}"
        );
    }

    /// The other MCP tools keep their own, wider list — this fix must not
    /// quietly narrow `analyze_satd` and friends off shell scripts.
    #[test]
    fn the_shared_source_walk_is_unchanged() {
        let d = mixed_language_fixture();
        let files =
            crate::services::path_glob::expand_paths_to_source_files(&[d.path().to_path_buf()]);
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        assert!(
            names.contains(&"run.sh".to_string()),
            "the general source walk still admits shell, got {names:?}"
        );
    }

    /// RED CONTROL: deleting the `total_files` entry from the dead-code
    /// payload fails this — which is the state the gate measured, where the
    /// CLI's 38 had no counterpart on this transport at all.
    #[tokio::test]
    async fn dead_code_publishes_both_counts_under_the_cli_names() {
        let d = mixed_language_fixture();
        let root = d.path().to_path_buf();

        let cli = crate::cli::handlers::dead_code_handlers::run_dead_code_suite(&root, false)
            .await
            .expect("cli dead-code run");

        let mcp = analyze_dead_code(std::slice::from_ref(&root), false)
            .await
            .expect("mcp analysis");
        let results = &mcp["results"];

        assert_eq!(
            results["total_files"].as_u64(),
            Some(cli.report.total_files as u64),
            "the CLI's `total_files` (the walk's denominator) must exist here \
             and match: {mcp}"
        );
        assert_eq!(
            results["analyzed_files"].as_u64(),
            Some(cli.report.analyzed_files as u64),
            "the CLI's `analyzed_files` must exist here and match: {mcp}"
        );
        // The legacy name is kept, and it is the SAME number — two spellings of
        // one measurement, never two measurements.
        assert_eq!(
            results["files_analyzed"], results["analyzed_files"],
            "the retained legacy key must not drift from the canonical one: {mcp}"
        );
    }
}
