//! Analysis command handlers
//!
//! This module extracts all analysis-related handlers from the main CLI module
//! to reduce complexity and improve organization.
#![cfg_attr(coverage_nightly, coverage(off))]

mod advanced_routes;
mod core_routes;
mod entropy_semantic;
pub mod perf_report;
mod platform_routes;

use crate::cli::colors as c;
use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;

// ── Colour for the HUMAN output of reachability, hardcoded-paths, unrun-tests
// and vacuous-tests.
//
// `--color always` did nothing for these four. Measured against this repository:
// 52, 42, 184 and 53 lines of output respectively, and ZERO ANSI escapes under
// both `--color always` and `--color never` — byte-identical. The flag was
// accepted and inert, which is the class this release exists to remove.
//
// Colouring lives HERE, at the print site, and NOT inside `summary()`. The same
// `summary()` string is embedded verbatim into the machine-readable payloads
// (`"summary": report.summary()`), and `colors_enabled()` is a process-wide
// oracle that answers "yes" whenever stdout is a tty — so colour inside
// `summary()` would put raw escapes inside a JSON string field any time the
// tool runs under a pty (CI with tty allocation, `script`, an agent harness).
// That is a silent, environment-dependent corruption no local `| jq` reproduces.
//
// It also would have been a fake fix: `summary()` is one line of 42-184, so
// colouring it alone flips the flag-efficacy verdict to "Effective" on a single
// escape byte while the report still reads monochrome.
//
// Every helper is byte-identical to the old format string when colour is off,
// which the tests pin explicitly — a one-sided "is plain" assertion is satisfied
// by a printer that emits no colour at all, and that is the defect shape this
// module keeps re-having (see src/cli/colors.rs).

/// The report's own headline sentence.
fn h_headline(s: &str) -> String {
    c::label(s)
}

/// `  <path>  (N lines, M tests)` — reachability's orphan rows.
fn h_orphan_row(path: &str, lines: usize, tests: usize) -> String {
    format!(
        "  {}  ({} lines, {} tests)",
        c::path(path),
        c::number(&lines.to_string()),
        c::number(&tests.to_string())
    )
}

/// `  [site] file:line  path  (reason)` — hardcoded-paths findings.
fn h_hardcoded_row(site: &str, file: &str, line: usize, path: &str, reason: &str) -> String {
    format!(
        "  [{}] {}:{}  {}  ({})",
        c::label(site),
        c::path(file),
        c::number(&line.to_string()),
        path,
        c::dim(reason)
    )
}

/// `  [bucket] N test(s)` — unrun-tests bucket headers.
fn h_bucket_header(bucket: &str, n: usize) -> String {
    format!(
        "  [{}] {} test(s)",
        c::label(bucket),
        c::number(&n.to_string())
    )
}

/// `      <path>` — a member row under a bucket.
fn h_member_row(path: &str) -> String {
    format!("      {}", c::path(path))
}

/// `  [kind] file:line  name<detail>` — vacuous-tests rows.
fn h_vacuous_row(kind: &str, file: &str, line: usize, name: &str, detail: &str) -> String {
    format!(
        "  [{}] {}:{}  {}{}",
        c::label(kind),
        c::path(file),
        c::number(&line.to_string()),
        name,
        c::dim(detail)
    )
}

/// A de-emphasised aside: `  skipped: x`, `  unresolved: x`, `  leg: x`.
fn h_aside(text: &str) -> String {
    c::dim(text)
}

/// Refuse an `--ml` flag whose scorer is not wired into the handler (GH-97).
///
/// ONE RULE, TWO COMMANDS. `analyze complexity --ml` and `analyze tdg --ml` both
/// promised "trained ML models instead of heuristic weighted sums" and both
/// destructured the flag into `_`, so each returned the heuristic numbers under
/// an ML banner — a relabelling, not a different result. `complexity` was fixed
/// with a bail! written inline and `tdg` was left behind; the refusal lives in
/// one place now so the next `--ml` cannot drift from it.
///
/// # Errors
/// Always, when `ml` is set. That is the point: an honest refusal beats a
/// silent no-op.
pub(super) fn reject_unimplemented_ml(ml: bool, command: &str, scores: &str) -> Result<()> {
    if !ml {
        return Ok(());
    }
    anyhow::bail!(
        "--ml is not implemented: {scores} are still computed by the heuristic formulas, \
         so this flag would relabel them without changing them. \
         Re-run `{command}` without --ml (see GH-97)."
    )
}

/// Report tracked `.rs` files that no compilation unit reaches.
async fn route_reachability(cmd: cli::AnalyzeCommands) -> anyhow::Result<()> {
    use crate::services::reachability;
    use crate::services::reachability_ledger as ledger;
    let cli::AnalyzeCommands::Reachability {
        path,
        format,
        fail_on_orphan,
        write_ledger,
        allow_dirty,
        check_ledger,
    } = cmd
    else {
        unreachable!("Expected Reachability command")
    };

    let (roots, tracked) = reachability::discover(&path)?;
    if roots.is_empty() {
        // Refuse rather than report "0 orphans" over a tree we never walked:
        // an unmeasured run must not look like a clean one.
        anyhow::bail!(
            "no cargo targets found under {} — `cargo metadata --no-deps` returned none, \
             so reachability could not be measured (this is not a clean result)",
            path.display()
        );
    }
    let report = reachability::analyze(&path, &roots, &tracked);

    if write_ledger {
        ledger::write(&path, &report, allow_dirty).map_err(anyhow::Error::msg)?;
        println!("wrote {}", ledger::LEDGER_PATH);
        return Ok(());
    }

    if format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "reachable": report.reachable,
                "roots": report.roots,
                "orphan_count": report.orphans.len(),
                "orphan_lines": report.orphan_lines(),
                "orphan_tests": report.orphan_tests(),
                // Quarantined files are their own key, never folded into
                // `reachable` or `orphan_*`. A consumer that only knows the old
                // keys keeps reading exactly what it read before; one that wants
                // the third state has to ask for it by name.
                "quarantined_count": report.quarantined.len(),
                "quarantined_lines": report.quarantined_lines(),
                "quarantined_tests": report.quarantined_tests(),
                "unresolved_mods": report.unresolved.len(),
                "summary": report.summary(),
                "orphans": report.orphans.iter().map(|o| serde_json::json!({
                    "file": o.path, "lines": o.lines, "tests": o.tests
                })).collect::<Vec<_>>(),
                "quarantined": report.quarantined.iter().map(|o| serde_json::json!({
                    "file": o.path, "lines": o.lines, "tests": o.tests
                })).collect::<Vec<_>>(),
            })
        );
    } else {
        println!("{}", h_headline(&report.summary()));
        for o in report.orphans.iter().take(40) {
            println!("{}", h_orphan_row(&o.path, o.lines, o.tests));
        }
        if report.orphans.len() > 40 {
            println!(
                "{}",
                h_aside(&format!("  … and {} more", report.orphans.len() - 40))
            );
        }
        if !report.quarantined.is_empty() {
            println!(
                "{}",
                h_headline(&format!(
                    "quarantined behind `pmat_broken_tests` — declared, never compiled ({} files, {} #[test] fns)",
                    report.quarantined.len(),
                    report.quarantined_tests()
                ))
            );
            for o in report.quarantined.iter().take(40) {
                println!("{}", h_orphan_row(&o.path, o.lines, o.tests));
            }
            if report.quarantined.len() > 40 {
                println!(
                    "{}",
                    h_aside(&format!("  … and {} more", report.quarantined.len() - 40))
                );
            }
        }
        for u in report.unresolved.iter().take(10) {
            println!("{}", h_aside(&format!("  unresolved: {u}")));
        }
    }

    let mut failed = fail_on_orphan && !report.orphans.is_empty();
    if check_ledger {
        let drift = ledger::check(&path, &report);
        if drift.is_clean() {
            println!("ledger is current: {}", ledger::LEDGER_PATH);
        } else {
            report_reachability_ledger_drift(&drift);
            failed = true;
        }
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}

fn report_reachability_ledger_drift(drift: &crate::services::reachability_ledger::Drift) {
    use crate::services::reachability_ledger::LEDGER_PATH;
    eprintln!("{LEDGER_PATH} has drifted from the tree:");
    for p in &drift.added {
        eprintln!("  NEWLY UNREACHABLE  {p} — add a `pending-#<issue>` row, or register the file");
    }
    for p in &drift.removed {
        eprintln!("  STALE ROW  {p} — no longer unreachable; mark it registered-<target> or deleted-<reason>");
    }
    for r in &drift.refuted {
        eprintln!("  REFUTED  {r}");
    }
    for l in &drift.malformed {
        eprintln!("  MALFORMED  {l}");
    }
    if drift.added.is_empty()
        && drift.removed.is_empty()
        && drift.refuted.is_empty()
        && drift.malformed.is_empty()
        && drift.text_differs
    {
        eprintln!("  the rendered ledger differs from the committed copy — re-run --write-ledger");
    }
}

#[cfg(test)]
pub(crate) use advanced_routes::{convert_cache_strategy, convert_deep_context_dag_type};
#[cfg(test)]
pub(crate) use entropy_semantic::{
    create_entropy_config, format_markdown_violations, format_violation_list, get_top_violations,
    output_entropy_results,
};

/// Router for all analysis commands - central dispatch for CLI analyze subcommands.
///
/// This function serves as the main entry point for all `pmat analyze` subcommands,
/// routing each command variant to its specific handler implementation. Critical for
/// API stability as it defines the complete analyze command interface.
///
/// # Parameters
///
/// * `cmd` - The specific analyze command variant with all parsed arguments
///
/// # Returns
///
/// * `Ok(())` - Command completed successfully
/// * `Err(anyhow::Error)` - Command execution failed with detailed error context
///
/// # API Stability Contract
///
/// This router maintains the CLI API contract by:
/// - Ensuring all `AnalyzeCommands` variants are handled
/// - Providing consistent parameter forwarding to handlers
/// - Maintaining backward compatibility for existing commands
/// - Preventing API drift through comprehensive parameter mapping
///
/// # Supported Commands
///
/// ## Core Analysis Commands
/// - `complexity` - Cyclomatic and cognitive complexity analysis
/// - `churn` - Code change frequency analysis over time
/// - `dead-code` - Unused code detection and reporting
/// - `dag` - Dependency graph generation and visualization
/// - `satd` - Self-admitted technical debt detection
///
/// ## Advanced Analysis Commands
/// - `deep-context` - Comprehensive project context analysis
/// - `tdg` - Technical debt gravity calculation
/// - `lint-hotspot` - Linting issue density analysis
/// - `makefile` - Makefile structure and rule analysis
/// - `provability` - Formal verification potential assessment
/// - `duplicates` - Code duplication detection
/// - `defect-prediction` - AI-powered defect probability analysis
/// - `comprehensive` - Full multi-faceted analysis suite
/// - `graph-metrics` - Graph centrality and topology metrics
/// - `name-similarity` - Identifier similarity analysis
/// - `proof-annotations` - Proof annotation extraction
/// - `incremental-coverage` - Differential coverage analysis
/// - `symbol-table` - Symbol visibility and reference analysis
/// - `big-o` - Algorithmic complexity analysis
/// - `assemblyscript` - AssemblyScript-specific analysis
/// - `webassembly` - WebAssembly module analysis
///
/// # Examples
///
/// ```ignore
/// use pmat::cli::handlers::analysis_handlers::route_analyze_command;
/// use pmat::cli::commands::AnalyzeCommands;
/// use std::path::PathBuf;
///
/// # tokio_test::block_on(async {
/// // Complexity analysis command
/// let complexity_cmd = AnalyzeCommands::Complexity {
///     project_path: PathBuf::from("/tmp/project"),
///     file: None,
///     files: vec![],
///     toolchain: None,
///     format: pmat::cli::enums::ComplexityOutputFormat::Summary,
///     output: None,
///     max_cyclomatic: None,
///     max_cognitive: None,
///     include: vec![],
///     watch: false,
///     top_files: 10,
///     fail_on_violation: false,
/// };
///
/// // This would normally execute the command
/// // let result = route_analyze_command(complexity_cmd).await;
/// // assert!(result.is_ok());
///
/// // Dead code analysis command
/// let dead_code_cmd = AnalyzeCommands::DeadCode {
///     path: PathBuf::from("/tmp/project"),
///     format: pmat::cli::enums::DeadCodeOutputFormat::Summary,
///     top_files: None,
///     include_unreachable: false,
///     min_dead_lines: 10,
///     include_tests: false,
///     output: None,
///     fail_on_violation: false,
///     max_percentage: 100.0,
/// };
///
/// // DAG analysis command
/// let dag_cmd = AnalyzeCommands::Dag {
///     dag_type: pmat::cli::enums::DagType::CallGraph,
///     project_path: PathBuf::from("/tmp/project"),
///     output: None,
///     max_depth: Some(5),
///     target_nodes: None,
///     filter_external: false,
///     show_complexity: false,
///     include_duplicates: false,
///     include_dead_code: false,
///     enhanced: false,
/// };
///
/// // All commands follow the same routing pattern
/// // Each command variant maps to a specific handler function
/// # });
/// ```
///
/// # Error Handling
///
/// The router implements comprehensive error handling:
/// - Parameter validation errors are propagated from handlers
/// - I/O errors from file operations are wrapped with context
/// - Parse errors include file location information
/// - Analysis failures preserve original error chains
///
/// # Performance Characteristics
///
/// - Route dispatch: O(1) pattern matching
/// - Parameter forwarding: O(1) move semantics
/// - Memory: Minimal overhead, parameters moved to handlers
/// - Concurrency: Handlers may implement parallel processing internally
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn route_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    // `--perf` has exactly one implementation, and it lives here. Nine of the
    // thirteen subcommands that advertise the flag used to drop it on the floor
    // (see `perf_report` for the roll-call), so `--perf` produced byte-identical
    // output. Timing the dispatch is the only way to make that impossible: a
    // subcommand cannot forget to honour a flag it never sees.
    let perf_label = perf_report::perf_command_label(&cmd);
    let started = std::time::Instant::now();

    let result = dispatch_analyze_command(cmd).await;

    if let Some(label) = perf_label {
        perf_report::emit(label, started.elapsed());
    }

    result
}

/// Route every `analyze` subcommand to its handler, in one total match.
///
/// Split out of [`route_analyze_command`] so the `--perf` measurement wraps
/// every analyze subcommand exactly once.
///
/// **One match, no catch-all.** This used to be seven matches: this one sorted
/// variants into six families, and each family router re-matched and ended in
/// `_ => unreachable!("Expected <family> analysis command")`. Only this match
/// was checked by the compiler, so listing a variant in the wrong family — or
/// adding it to a family and forgetting its router — compiled cleanly and
/// aborted the process at run time. The families survive as comments; the
/// dispatch decision is made once, by name, so that omitting a variant is a
/// compile error.
async fn dispatch_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    use cli::AnalyzeCommands;

    match cmd {
        // Core analysis commands
        AnalyzeCommands::Bottleneck { .. } => core_routes::route_bottleneck_analysis(cmd).await,
        AnalyzeCommands::Complexity { .. } => core_routes::route_complexity_analysis(cmd).await,
        AnalyzeCommands::Churn { .. } => core_routes::route_churn_analysis(cmd).await,
        AnalyzeCommands::DeadCode { .. } => core_routes::route_dead_code_analysis(cmd).await,
        AnalyzeCommands::Defects { .. } => core_routes::route_defects_analysis(cmd).await,
        AnalyzeCommands::Dag { .. } => core_routes::route_dag_analysis(cmd).await,
        AnalyzeCommands::Satd { .. } => core_routes::route_satd_analysis(cmd).await,

        AnalyzeCommands::Reachability { .. } => route_reachability(cmd).await,

        AnalyzeCommands::HardcodedPaths { .. } => route_hardcoded_paths(cmd).await,

        AnalyzeCommands::UnrunTests { .. } => route_unrun_tests(cmd).await,
        AnalyzeCommands::VacuousTests { .. } => route_vacuous_tests(cmd).await,

        // Advanced analysis commands
        AnalyzeCommands::DeepContext { .. } => {
            advanced_routes::route_deep_context_analysis(cmd).await
        }
        AnalyzeCommands::Tdg { .. } => advanced_routes::route_tdg_analysis(cmd).await,
        AnalyzeCommands::BuildTdg { .. } => advanced_routes::route_build_tdg_analysis(cmd).await,
        AnalyzeCommands::LintHotspot { .. } => {
            advanced_routes::route_lint_hotspot_analysis(cmd).await
        }
        AnalyzeCommands::Comprehensive { .. } => {
            advanced_routes::route_comprehensive_analysis(cmd).await
        }

        // Quality analysis commands
        AnalyzeCommands::Duplicates { .. } => advanced_routes::route_duplicates_analysis(cmd).await,
        AnalyzeCommands::DefectPrediction { .. } => {
            advanced_routes::route_defect_prediction_analysis(cmd).await
        }
        AnalyzeCommands::Provability { .. } => {
            advanced_routes::route_provability_analysis(cmd).await
        }
        AnalyzeCommands::Clippy { .. } => advanced_routes::route_clippy_analysis(cmd).await,
        AnalyzeCommands::Entropy { .. } => entropy_semantic::route_entropy_analysis(cmd).await,

        // Specialized analysis commands
        AnalyzeCommands::GraphMetrics { .. } => {
            platform_routes::route_graph_metrics_analysis(cmd).await
        }
        AnalyzeCommands::NameSimilarity { .. } => {
            platform_routes::route_name_similarity_analysis(cmd).await
        }
        AnalyzeCommands::ProofAnnotations { .. } => {
            platform_routes::route_proof_annotations_analysis(cmd).await
        }
        AnalyzeCommands::IncrementalCoverage { .. } => {
            platform_routes::route_incremental_coverage_analysis(cmd).await
        }
        AnalyzeCommands::CoverageImprove {
            path,
            project_path,
            target,
            max_iterations,
            fast,
            mutation_threshold,
            focus,
            exclude,
            max_targets,
            output,
            format,
        } => {
            let path = project_path.unwrap_or(path);
            crate::cli::handlers::coverage_improve_handler::handle_coverage_improve(
                path,
                target,
                max_iterations,
                fast,
                mutation_threshold,
                focus,
                exclude,
                max_targets,
                output,
                format,
            )
            .await
        }
        AnalyzeCommands::SymbolTable { .. } => {
            platform_routes::route_symbol_table_analysis(cmd).await
        }
        AnalyzeCommands::BigO { .. } => platform_routes::route_big_o_analysis(cmd).await,

        // Language-specific commands
        AnalyzeCommands::AssemblyScript { .. } => {
            platform_routes::route_assemblyscript_analysis(cmd).await
        }
        AnalyzeCommands::WebAssembly { .. } => {
            platform_routes::route_webassembly_analysis(cmd).await
        }
        #[cfg(feature = "wasm-ast")]
        AnalyzeCommands::Wasm { .. } => platform_routes::route_wasm_analysis(cmd).await,
        #[cfg(not(feature = "wasm-ast"))]
        AnalyzeCommands::Wasm { .. } => {
            anyhow::bail!(
                "WASM analysis requires the 'wasm-ast' feature. Build with --features wasm-ast"
            )
        }

        // Deep WASM analysis (feature-gated)
        #[cfg(feature = "deep-wasm")]
        AnalyzeCommands::DeepWasm { .. } => platform_routes::route_deep_wasm_analysis(cmd).await,

        // Mutation testing (feature-gated)
        #[cfg(feature = "mutation-testing")]
        AnalyzeCommands::Mutate { .. } => platform_routes::route_mutation_testing(cmd).await,

        // System commands
        AnalyzeCommands::Makefile { .. } => platform_routes::route_makefile_analysis(cmd).await,

        // Semantic analysis commands (PMAT-SEARCH-011)
        AnalyzeCommands::Cluster { .. } | AnalyzeCommands::Topics { .. } => {
            entropy_semantic::route_semantic_analysis(cmd).await
        }

        // MLOps model analysis (PMAT-500)
        AnalyzeCommands::Models { .. } => platform_routes::route_model_analysis(cmd).await,
    }
}

// Tests re-unified via Operation Logical Atomism
#[cfg(test)]
#[path = "../analysis_handlers_tests.rs"]
mod tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
#[path = "dispatch_totality_tests.rs"]
mod dispatch_totality_tests;

/// Find machine-specific absolute paths baked into source.
async fn route_hardcoded_paths(cmd: cli::AnalyzeCommands) -> anyhow::Result<()> {
    use crate::services::hardcoded_paths::{self, Site};
    let cli::AnalyzeCommands::HardcodedPaths {
        path,
        format,
        fail_on_shipped,
        fail_on_any,
    } = cmd
    else {
        unreachable!("Expected HardcodedPaths command")
    };

    let files = hardcoded_paths::tracked_files(&path)?;
    if files.is_empty() {
        // Refuse rather than print "0 findings" over a tree we never opened.
        // An unmeasured run must not be indistinguishable from a clean one
        // (#1015) — that confusion is the exact defect this command hunts.
        anyhow::bail!(
            "no scannable tracked files under {} — `git ls-files` returned none, so no path \
             scan was performed (this is not a clean result)",
            path.display()
        );
    }
    let report = hardcoded_paths::analyze(&path, &files);

    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "summary": report.summary(),
                "files_scanned": report.files_scanned,
                "literals_scanned": report.literals_scanned,
                "finding_count": report.findings.len(),
                "shipped_count": report.shipped(),
                "by_kind": report.by_kind(),
                "skipped": report.skipped,
                "findings": report.findings,
            }))?
        );
    } else {
        println!("{}", h_headline(&report.summary()));
        const MAX_SHOWN: usize = 40;
        for f in report.findings.iter().take(MAX_SHOWN) {
            println!(
                "{}",
                h_hardcoded_row(f.site.as_str(), &f.file, f.line, &f.path, f.kind.reason())
            );
        }
        if let Some(hidden) = report.findings.len().checked_sub(MAX_SHOWN) {
            if hidden > 0 {
                println!("{}", h_aside(&format!("  … and {hidden} more")));
            }
        }
        for s in report
            .skipped
            .unreadable
            .iter()
            .chain(&report.skipped.not_utf8)
        {
            println!("{}", h_aside(&format!("  skipped: {s}")));
        }
    }

    let shipped = report
        .findings
        .iter()
        .filter(|f| f.site == Site::Shipped)
        .count();
    if (fail_on_any && !report.findings.is_empty()) || (fail_on_shipped && shipped > 0) {
        std::process::exit(1);
    }
    Ok(())
}

/// Report tests that no CI leg executes.
async fn route_unrun_tests(cmd: cli::AnalyzeCommands) -> anyhow::Result<()> {
    use crate::services::unrun_tests::{self, ledger};
    let cli::AnalyzeCommands::UnrunTests {
        path,
        format,
        executed,
        write_ledger,
        allow_dirty,
        check_ledger,
        fail_on_any,
    } = cmd
    else {
        unreachable!("Expected UnrunTests command")
    };

    let report = unrun_tests::analyze(&path, &executed).map_err(anyhow::Error::msg)?;
    if write_ledger {
        ledger::write(&path, &report, allow_dirty).map_err(anyhow::Error::msg)?;
        println!("wrote {}", ledger::LEDGER_PATH);
        return Ok(());
    }
    print_unrun_report(&report, &format)?;

    let mut failed = fail_on_any && !report.unrun.is_empty();
    if check_ledger {
        let drift = ledger::check(&path, &report);
        if drift.is_clean() {
            println!("ledger is current: {}", ledger::LEDGER_PATH);
        } else {
            report_ledger_drift(&drift);
            failed = true;
        }
    }
    // A predicate this analysis cannot decide is a finding, never a pass.
    if !report.undeterminable.is_empty() || !report.unparsed.is_empty() {
        failed = true;
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}

fn print_unrun_report(
    report: &crate::services::unrun_tests::Report,
    format: &str,
) -> anyhow::Result<()> {
    use crate::services::unrun_tests::ledger;
    match format {
        "ledger" => print!("{}", ledger::render(report)),
        "json" => println!("{}", serde_json::to_string_pretty(&json_of(report))?),
        _ => {
            println!("{}", h_headline(&report.summary()));
            for leg in &report.legs {
                println!("{}", h_aside(&format!("  leg: {leg}")));
            }
            for (bucket, members) in report.buckets() {
                println!("{}", h_bucket_header(&bucket, members.len()));
                for m in members.iter().take(5) {
                    println!("{}", h_member_row(&m.path));
                }
                if members.len() > 5 {
                    println!(
                        "{}",
                        h_aside(&format!("      … and {} more", members.len() - 5))
                    );
                }
            }
            for f in &report.undeterminable {
                println!(
                    "{}",
                    h_aside(&format!("  [undeterminable] {}  cfg: {}", f.path, f.cfg))
                );
            }
        }
    }
    Ok(())
}

fn json_of(report: &crate::services::unrun_tests::Report) -> serde_json::Value {
    let rows = |v: &[crate::services::unrun_tests::Finding]| {
        v.iter()
            .map(|f| {
                serde_json::json!({
                    "path": f.path, "file": f.file, "bucket": f.bucket, "cfg": f.cfg,
                })
            })
            .collect::<Vec<_>>()
    };
    serde_json::json!({
        "legs": report.legs,
        "total_tests": report.total_tests,
        "executed": report.executed,
        "ignored": report.ignored,
        "unrun": rows(&report.unrun),
        "undeterminable": rows(&report.undeterminable),
        "unresolved": report.unresolved,
        "unparsed": report.unparsed,
        "summary": report.summary(),
    })
}

fn report_ledger_drift(drift: &crate::services::unrun_tests::ledger::Drift) {
    use crate::services::unrun_tests::ledger::LEDGER_PATH;
    eprintln!("{LEDGER_PATH} has drifted from the tree:");
    for p in &drift.added {
        eprintln!("  NEWLY UNRUN  {p}");
    }
    for p in &drift.removed {
        eprintln!("  NO LONGER UNRUN  {p}");
    }
    for b in &drift.unexplained {
        eprintln!("  NO RECORDED REASON for bucket `{b}` — add one to src/services/unrun_tests/reasons.rs");
    }
    for b in &drift.stale_reasons {
        eprintln!("  STALE REASON for bucket `{b}` — nothing is unrun for it any more");
    }
    if drift.added.is_empty() && drift.removed.is_empty() && drift.text_differs {
        eprintln!("  the rendered ledger differs from the committed copy");
    }
}

/// Find `#[test]` functions that cannot fail.
async fn route_vacuous_tests(cmd: cli::AnalyzeCommands) -> anyhow::Result<()> {
    use crate::services::vacuous_tests;
    let cli::AnalyzeCommands::VacuousTests {
        path,
        format,
        max_rate,
        fail_on_any,
    } = cmd
    else {
        unreachable!("Expected VacuousTests command")
    };

    let files = vacuous_tests::tracked_rust_files(&path)?;
    if files.is_empty() {
        anyhow::bail!(
            "no tracked .rs files under {} — `git ls-files` returned none, so no test was \
             examined (this is not a clean result)",
            path.display()
        );
    }
    let report = vacuous_tests::analyze(&path, &files);
    if report.tests_examined == 0 {
        // Zero vacuous tests out of zero tests is not a pass. Say so.
        anyhow::bail!(
            "no #[test] functions found in {} parsed file(s) under {} — nothing was judged, \
             so this is not a clean result",
            report.files_parsed,
            path.display()
        );
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", h_headline(&report.summary()));
        for v in report.vacuous.iter().take(40) {
            let detail = v
                .detail
                .as_deref()
                .map(|d| format!(" — {d}"))
                .unwrap_or_default();
            println!(
                "{}",
                h_vacuous_row(v.kind.as_str(), &v.file, v.line, &v.name, &detail)
            );
        }
        if report.vacuous.len() > 40 {
            println!(
                "{}",
                h_aside(&format!("  … and {} more", report.vacuous.len() - 40))
            );
        }
        for s in report.conditional_skips.iter().take(10) {
            println!(
                "{}",
                h_vacuous_row(
                    "silent-skip",
                    &s.file,
                    s.line,
                    &s.name,
                    &format!("  if {}", s.guard)
                )
            );
        }
        if report.conditional_skips.len() > 10 {
            println!(
                "{}",
                h_aside(&format!(
                    "  … and {} more silent skips",
                    report.conditional_skips.len() - 10
                ))
            );
        }
    }

    let over_rate = max_rate.is_some_and(|m| report.rate() > m);
    if (fail_on_any && !report.vacuous.is_empty()) || over_rate {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod ml_refusal_tests {
    //! `analyze complexity --ml` and `analyze tdg --ml` are the same defect:
    //! both promised "trained ML models instead of heuristic weighted sums" and
    //! both threw the flag away, so each printed the heuristic numbers under an
    //! ML banner. The refusal is one function so the two cannot drift.
    use super::reject_unimplemented_ml;

    #[test]
    fn an_unset_flag_is_not_refused() {
        assert!(reject_unimplemented_ml(false, "analyze tdg", "TDG scores").is_ok());
    }

    #[test]
    fn tdg_ml_is_refused_rather_than_relabelled() {
        let err = reject_unimplemented_ml(true, "analyze tdg", "TDG scores")
            .expect_err("--ml returned heuristic scores, so it must not be accepted");
        let err = err.to_string();
        assert!(err.contains("--ml is not implemented"), "{err}");
        assert!(err.contains("TDG scores"), "{err}");
        assert!(err.contains("analyze tdg"), "{err}");
    }

    #[test]
    fn complexity_ml_keeps_its_own_wording() {
        let err = reject_unimplemented_ml(true, "analyze complexity", "complexity scores")
            .unwrap_err()
            .to_string();
        assert!(err.contains("complexity scores"), "{err}");
        assert!(err.contains("analyze complexity"), "{err}");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod human_colour_tests {
    use super::*;
    use crate::cli::colors::{ForcedColor, ESC};

    /// Every helper, under both colour states, in one table.
    ///
    /// `(what it renders, the exact bytes it must produce with colour OFF)`.
    /// The expected strings are pinned literally so a future refactor cannot
    /// quietly change what a piped or redirected run prints — that output is
    /// what `| grep`, `| wc` and every script downstream actually consume.
    fn cases() -> Vec<(&'static str, String, String)> {
        let on = |f: &dyn Fn() -> String| {
            let _g = ForcedColor::on();
            f()
        };
        let off = |f: &dyn Fn() -> String| {
            let _g = ForcedColor::off();
            f()
        };
        let mk = |name: &'static str, f: &dyn Fn() -> String| (name, on(f), off(f));
        vec![
            mk("headline", &|| h_headline("3 of 4 files are reachable")),
            mk("orphan_row", &|| h_orphan_row("src/dead.rs", 12, 3)),
            mk("hardcoded_row", &|| {
                h_hardcoded_row("shipped", "src/a.rs", 42, "/home/x", "absolute path")
            }),
            mk("bucket_header", &|| h_bucket_header("cfg(feature)", 7)),
            mk("member_row", &|| h_member_row("src/t.rs")),
            mk("vacuous_row", &|| {
                h_vacuous_row("no-assert", "src/t.rs", 9, "test_thing", " — nothing")
            }),
            mk("aside", &|| h_aside("  skipped: src/x.rs")),
        ]
    }

    /// With `--color always`, every human row must actually carry colour.
    ///
    /// Measured at HEAD before this change: `pmat --color always analyze
    /// {reachability,hardcoded-paths,unrun-tests,vacuous-tests} --path .`
    /// produced 52, 42, 184 and 53 lines and ZERO escapes — the flag was
    /// accepted and inert.
    #[test]
    fn every_human_row_is_coloured_when_colour_is_on() {
        for (name, coloured, _) in cases() {
            assert!(
                coloured.contains(ESC),
                "{name} emits no escape with colour forced ON: {coloured:?}"
            );
        }
    }

    /// ...and none of them carries colour when it is off. This is the half that
    /// keeps `--format json`, pipes and redirects parseable.
    #[test]
    fn no_human_row_is_coloured_when_colour_is_off() {
        for (name, _, plain) in cases() {
            assert!(
                !plain.contains(ESC),
                "{name} emitted an escape with colour forced OFF: {plain:?}"
            );
        }
    }

    /// The plain rendering is byte-for-byte what it was before colour existed.
    ///
    /// Without this, "add colour" is free to reformat the piped output, and the
    /// two assertions above would both still pass.
    #[test]
    fn the_plain_rendering_is_unchanged() {
        let _g = ForcedColor::off();
        assert_eq!(
            h_headline("3 of 4 files are reachable"),
            "3 of 4 files are reachable"
        );
        assert_eq!(
            h_orphan_row("src/dead.rs", 12, 3),
            "  src/dead.rs  (12 lines, 3 tests)"
        );
        assert_eq!(
            h_hardcoded_row("shipped", "src/a.rs", 42, "/home/x", "absolute path"),
            "  [shipped] src/a.rs:42  /home/x  (absolute path)"
        );
        assert_eq!(
            h_bucket_header("cfg(feature)", 7),
            "  [cfg(feature)] 7 test(s)"
        );
        assert_eq!(h_member_row("src/t.rs"), "      src/t.rs");
        assert_eq!(
            h_vacuous_row("no-assert", "src/t.rs", 9, "test_thing", " — nothing"),
            "  [no-assert] src/t.rs:9  test_thing — nothing"
        );
        assert_eq!(h_aside("  skipped: src/x.rs"), "  skipped: src/x.rs");
    }
}
