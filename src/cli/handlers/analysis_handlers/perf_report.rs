//! The single implementation of `--perf` for pmat.
//!
//! `--perf` is documented on thirteen `analyze` subcommands as "Show
//! performance metrics". Before this module each subcommand was expected to
//! honour the flag on its own, and most did not: `analyze graph-metrics`,
//! `analyze symbol-table` and `analyze proof-annotations` bound it as `_perf`,
//! `analyze clippy` destructured it as `perf: _perf`, `analyze
//! incremental-coverage` stored it in a config field nothing read, `analyze
//! defect-prediction` stored it in a field of the live handler while a *second*
//! handler that did honour it sat unrouted, `analyze big-o` routed it through
//! `info!` which the default `warn` log filter discards, and `analyze
//! web-assembly` / `analyze assembly-script` read it only inside their
//! `--format json` branch. Nine of the thirteen subcommands accepted the flag
//! and produced byte-identical output.
//!
//! One rule, one implementation: the analyze router times the command and this
//! module prints the readout, so no subcommand can be wired up wrong. Renderers
//! that fold timings into their own *document* (the JSON `performance` object,
//! the comprehensive breakdown table) keep doing so — that is content on
//! stdout, not a second copy of this readout, which always goes to stderr so it
//! never corrupts a machine-readable document.
//!
//! `--perf` is not only an `analyze` flag, and the router above only covers
//! `AnalyzeCommands`. Four commands outside it declare the flag — `pmat
//! quality-gate`, `pmat test`, `pmat report` and `pmat refactor docs` — and the
//! verifier of the analyze fix found that the last two were still inert:
//! `report` routed its one perf line through `info!`, which the default `warn`
//! EnvFilter discards (big-o's defect verbatim), and `refactor docs` folded its
//! line into the summary/detailed *document* only, so `--format json --perf`
//! was byte-identical (web-assembly's defect verbatim). They now emit the same
//! readout through [`emit_readout`] instead of growing a third and fourth
//! wording. `every_command_with_a_perf_or_metrics_flag_is_routed` walks the
//! whole clap tree so the next sibling cannot be missed.

use crate::cli::AnalyzeCommands;
use std::time::Duration;

/// Render the one-line performance readout.
///
/// Kept separate from [`emit`] so the exact wording is unit-testable.
#[must_use]
pub fn format_line(command: &str, elapsed: Duration) -> String {
    format!(
        "⏱️  perf: {command} completed in {:.1} ms",
        elapsed.as_secs_f64() * 1000.0
    )
}

/// Print the performance readout for a completed analyze subcommand.
pub fn emit(command: &str, elapsed: Duration) {
    eprintln!("{}", format_line(command, elapsed));
}

/// Render an additional `--perf` measurement a subcommand computed itself.
///
/// Kept separate from [`emit_detail`] for the same reason as [`format_line`].
#[must_use]
pub fn format_detail_line(command: &str, name: &str, value: &str) -> String {
    format!("⏱️  perf: {command} {name}: {value}")
}

/// Print an additional `--perf` measurement a subcommand computed itself
/// (e.g. big-o's functions/second), in the same shape as [`emit`].
pub fn emit_detail(command: &str, name: &str, value: &str) {
    eprintln!("{}", format_detail_line(command, name, value));
}

/// The complete readout `--perf` adds for `command`, in emission order: the
/// wall-clock line followed by any measurements only that command can compute.
/// Empty when the flag was not passed.
///
/// Pure, so a command outside the `analyze` router can be *tested* for honouring
/// the flag without capturing stderr — the gap that let `pmat report --perf`
/// ship byte-identical output at every format.
#[must_use]
pub fn readout_lines(
    command: &str,
    elapsed: Duration,
    perf: bool,
    details: &[(&str, String)],
) -> Vec<String> {
    if !perf {
        return Vec::new();
    }

    let mut lines = vec![format_line(command, elapsed)];
    lines.extend(
        details
            .iter()
            .map(|(name, value)| format_detail_line(command, name, value)),
    );
    lines
}

/// Print [`readout_lines`] on stderr — the entry point for commands that are
/// not dispatched by the `analyze` router.
pub fn emit_readout(command: &str, elapsed: Duration, perf: bool, details: &[(&str, String)]) {
    for line in readout_lines(command, elapsed, perf, details) {
        eprintln!("{line}");
    }
}

/// The label to report for `cmd`, or `None` when `--perf` was not requested
/// (or the subcommand has no `--perf` flag at all).
///
/// Every `AnalyzeCommands` variant that declares `perf: bool` must appear here;
/// `perf_flag_coverage_tests` fails if one is added without being listed.
#[must_use]
pub fn perf_command_label(cmd: &AnalyzeCommands) -> Option<&'static str> {
    let (perf, label) = match cmd {
        AnalyzeCommands::LintHotspot { perf, .. } => (*perf, "analyze lint-hotspot"),
        AnalyzeCommands::Duplicates { perf, .. } => (*perf, "analyze duplicates"),
        AnalyzeCommands::DefectPrediction { perf, .. } => (*perf, "analyze defect-prediction"),
        AnalyzeCommands::Comprehensive { perf, .. } => (*perf, "analyze comprehensive"),
        AnalyzeCommands::GraphMetrics { perf, .. } => (*perf, "analyze graph-metrics"),
        AnalyzeCommands::NameSimilarity { perf, .. } => (*perf, "analyze name-similarity"),
        AnalyzeCommands::ProofAnnotations { perf, .. } => (*perf, "analyze proof-annotations"),
        AnalyzeCommands::IncrementalCoverage { perf, .. } => {
            (*perf, "analyze incremental-coverage")
        }
        AnalyzeCommands::SymbolTable { perf, .. } => (*perf, "analyze symbol-table"),
        AnalyzeCommands::BigO { perf, .. } => (*perf, "analyze big-o"),
        AnalyzeCommands::AssemblyScript { perf, .. } => (*perf, "analyze assembly-script"),
        AnalyzeCommands::WebAssembly { perf, .. } => (*perf, "analyze web-assembly"),
        AnalyzeCommands::Clippy { perf, .. } => (*perf, "analyze clippy"),
        _ => return None,
    };

    if perf {
        Some(label)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readout_names_the_command_and_a_duration() {
        let line = format_line("analyze big-o", Duration::from_millis(1234));
        assert!(line.contains("analyze big-o"), "got: {line}");
        assert!(line.contains("1234.0 ms"), "got: {line}");
        assert!(line.contains("perf"), "got: {line}");
    }

    /// The flag must be inert only when it was not passed.
    #[test]
    fn label_is_none_without_the_flag_and_some_with_it() {
        let make = |perf: bool| AnalyzeCommands::BigO {
            path: std::path::PathBuf::from("."),
            project_path: None,
            format: crate::cli::BigOOutputFormat::Summary,
            confidence_threshold: 50,
            analyze_space: false,
            include: vec![],
            exclude: vec![],
            high_complexity_only: false,
            output: None,
            perf,
            top_files: 0,
        };

        assert_eq!(perf_command_label(&make(false)), None);
        assert_eq!(perf_command_label(&make(true)), Some("analyze big-o"));
    }

    #[test]
    fn readout_is_empty_without_the_flag_and_carries_every_detail_with_it() {
        assert!(
            readout_lines("report", Duration::from_millis(5), false, &[]).is_empty(),
            "a command must stay silent when --perf was not passed"
        );

        let lines = readout_lines(
            "report",
            Duration::from_millis(5),
            true,
            &[("throughput", "42 files/second".to_string())],
        );
        assert_eq!(lines.len(), 2, "got: {lines:?}");
        assert!(lines[0].contains("report"), "got: {lines:?}");
        assert!(lines[0].contains("5.0 ms"), "got: {lines:?}");
        assert!(
            lines[1].contains("throughput: 42 files/second"),
            "got: {lines:?}"
        );
    }

    /// The whole-CLI roll call.
    ///
    /// `every_subcommand_with_a_perf_flag_has_a_label` only covers `analyze`,
    /// which is how `pmat report --perf` and `pmat refactor docs --format json
    /// --perf` stayed byte-identical after the thirteen analyze subcommands were
    /// fixed. Every command anywhere in the clap tree that declares `--perf` or
    /// `--metrics` must appear here together with the source that honours it,
    /// and that source must still contain the marker that does so. Deleting the
    /// emission, or adding the flag to a fourteenth command, fails here.
    #[test]
    fn every_command_with_a_perf_or_metrics_flag_is_routed() {
        // (command path, source that honours the flag, marker proving it does)
        let routed: &[(&str, &str, &str)] = &[
            // The analyze router times every subcommand it dispatches, so all
            // thirteen `analyze … --perf` flags share this one emission.
            ("analyze", include_str!("mod.rs"), "perf_report::emit("),
            (
                "analyze satd",
                include_str!("../satd_handler_formatting.rs"),
                "fn print_metrics(",
            ),
            (
                "quality-gate",
                include_str!("../../analysis_utilities/quality_gate_project.rs"),
                "print_check_performance(",
            ),
            (
                "test",
                include_str!("../../command_dispatcher/test_commands.rs"),
                "fn print_performance_summary_if_requested(",
            ),
            (
                "report",
                include_str!("../enhanced_reporting_handlers.rs"),
                "perf_report::readout_lines(",
            ),
            (
                "refactor docs",
                include_str!("../refactor_docs_orchestration.rs"),
                "perf_report::emit_readout(",
            ),
        ];

        let mut declared: Vec<String> = crate::cli::commands::on_big_stack(|| {
            fn walk(cmd: &clap::Command, prefix: &str, out: &mut Vec<String>) {
                let path = if prefix.is_empty() {
                    cmd.get_name().to_string()
                } else {
                    format!("{prefix} {}", cmd.get_name())
                };
                if cmd
                    .get_arguments()
                    .any(|a| a.get_id() == "perf" || a.get_id() == "metrics")
                {
                    out.push(path.clone());
                }
                for sub in cmd.get_subcommands() {
                    walk(sub, &path, out);
                }
            }

            let mut out = Vec::new();
            let cli = <crate::cli::Cli as clap::CommandFactory>::command();
            for sub in cli.get_subcommands() {
                walk(sub, "", &mut out);
            }
            out
        });
        declared.sort();

        assert!(
            !declared.is_empty(),
            "no command declares --perf/--metrics; the probe is broken"
        );

        for path in &declared {
            // Longest match wins: `analyze satd`'s `--metrics` is honoured by
            // the satd printer, not by the analyze router's `--perf` timing.
            let owner = routed
                .iter()
                .filter(|(cmd, _, _)| path == cmd || path.starts_with(&format!("{cmd} ")))
                .max_by_key(|(cmd, _, _)| cmd.len());
            let Some((cmd, src, marker)) = owner else {
                panic!(
                    "`pmat {path}` accepts --perf/--metrics but no entry in this roll call \
                     says where the flag is honoured, so it would parse and change nothing"
                );
            };
            assert!(
                src.contains(marker),
                "`pmat {path}` is recorded as honouring the flag via `{cmd}`'s source, \
                 but `{marker}` is gone from it — the flag is inert again"
            );
        }
    }

    /// Guard against the defect this module exists to remove: a subcommand that
    /// declares `--perf` but is never wired into the router's single reader.
    ///
    /// The list is derived from the clap definition, so adding a `--perf` to a
    /// new subcommand without extending `perf_command_label` fails here rather
    /// than shipping another flag that parses and changes nothing.
    #[test]
    fn every_subcommand_with_a_perf_flag_has_a_label() {
        use clap::Subcommand;

        let with_perf: Vec<String> = crate::cli::commands::on_big_stack(|| {
            let cmd = crate::cli::commands::AnalyzeCommands::augment_subcommands(
                clap::Command::new("analyze"),
            );
            cmd.get_subcommands()
                .filter(|s| s.get_arguments().any(|a| a.get_id() == "perf"))
                .map(|s| format!("analyze {}", s.get_name()))
                .collect()
        });

        assert!(
            !with_perf.is_empty(),
            "no analyze subcommand declares --perf; the probe is broken"
        );

        let known: Vec<&str> = vec![
            "analyze lint-hotspot",
            "analyze duplicates",
            "analyze defect-prediction",
            "analyze comprehensive",
            "analyze graph-metrics",
            "analyze name-similarity",
            "analyze proof-annotations",
            "analyze incremental-coverage",
            "analyze symbol-table",
            "analyze big-o",
            "analyze assembly-script",
            "analyze web-assembly",
            "analyze clippy",
        ];

        for name in &with_perf {
            assert!(
                known.contains(&name.as_str()),
                "{name} accepts --perf but perf_command_label() does not know it, \
                 so the flag would parse and change nothing"
            );
        }
    }
}
