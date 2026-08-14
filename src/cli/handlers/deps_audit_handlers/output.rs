#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{DepCategory, DepsAuditReport, ParetoEffort, ParetoEntry};
use crate::cli::colors;

/// Print Pareto analysis report
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn print_pareto_report(entries: &[ParetoEntry]) {
    println!("{}", colors::rule());
    println!(
        "{}📊  Pareto Analysis: 80/20 Dependency Removal{}",
        colors::BOLD,
        colors::RESET
    );
    println!("{}", colors::rule());
    println!();
    println!(
        "{}ROI = Transitive Deps Saved / Effort{}",
        colors::DIM,
        colors::RESET
    );
    println!(
        "{}Higher ROI = Better bang for buck{}",
        colors::DIM,
        colors::RESET
    );
    println!();

    if entries.is_empty() {
        println!("No removable dependencies found.");
        return;
    }

    // Calculate 80% of total savings
    let total_transitive: usize = entries.iter().map(|e| e.transitive_deps).sum();
    let target_80 = (total_transitive as f32 * 0.8) as usize;

    println!("┌─────────────────────┬───────────┬────────┬────────┬─────────────────────────────┐");
    println!("│ Dependency          │ Trans.Deps│ Effort │ ROI    │ Reason                      │");
    println!("├─────────────────────┼───────────┼────────┼────────┼─────────────────────────────┤");

    let mut cumulative = 0;
    let mut marked_80 = false;
    for entry in entries.iter().take(20) {
        cumulative += entry.transitive_deps;
        let marker = if !marked_80 && cumulative >= target_80 {
            marked_80 = true;
            "← 80%"
        } else {
            ""
        };

        let name_str = entry
            .name
            .get(..entry.name.len().min(19))
            .unwrap_or(&entry.name);
        let reason_str = entry
            .reason
            .get(..entry.reason.len().min(21))
            .unwrap_or(&entry.reason);
        println!(
            "│ {}{:<19}{} │ {:>9} │ {:>6} │ {}{:>6.1}{} │ {:<21} {:>5} │",
            colors::CYAN,
            name_str,
            colors::RESET,
            entry.transitive_deps,
            entry.effort.label(),
            colors::BOLD,
            entry.roi,
            colors::RESET,
            reason_str,
            marker
        );
    }
    println!("└─────────────────────┴───────────┴────────┴────────┴─────────────────────────────┘");
    println!();

    // Summary
    let top_5_savings: usize = entries.iter().take(5).map(|e| e.transitive_deps).sum();
    let top_5_pct = if total_transitive > 0 {
        (top_5_savings as f32 / total_transitive as f32 * 100.0) as usize
    } else {
        0
    };

    println!("{}💡 Summary:{}", colors::BOLD, colors::RESET);
    println!(
        "   Total transitive deps from candidates: {}{}{}",
        colors::BOLD_WHITE,
        total_transitive,
        colors::RESET
    );
    println!(
        "   Top 5 removals save: {}{}{} deps ({}{}{}% of total)",
        colors::BOLD_WHITE,
        top_5_savings,
        colors::RESET,
        colors::BOLD_WHITE,
        top_5_pct,
        colors::RESET
    );
    println!();

    // Actionable commands
    println!(
        "{}🔧 Quick Wins (Low Effort, High ROI):{}",
        colors::BOLD,
        colors::RESET
    );
    for entry in entries
        .iter()
        .filter(|e| matches!(e.effort, ParetoEffort::Low) && e.roi > 10.0)
        .take(5)
    {
        println!(
            "   {}cargo rm {}{} # saves {}{}{} transitive deps",
            colors::GREEN,
            entry.name,
            colors::RESET,
            colors::BOLD_WHITE,
            entry.transitive_deps,
            colors::RESET
        );
    }
    println!();
    println!("{}", colors::rule());
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Print text report.
///
/// `show_all` is the `--all` flag. The handler already honours it when it
/// builds `report.dependencies` (it stops filtering out `Core` and
/// `Sovereign`), but this renderer only grouped the four categories `--all`
/// does NOT add — Removable, Heavy, Replaceable, DevOnly — so the extra rows
/// could never be printed for any input, while the footer promised them
/// unconditionally, including on the run that had just supplied `--all`. Text
/// output was byte-identical with and without the flag on a project with
/// `serde` and `rayon` in it; only `-f json` differed.
pub fn print_text_report(report: &DepsAuditReport, show_all: bool) {
    println!("{}", colors::rule());
    println!(
        "{}🔍  Dependency Audit Report (with Graph Analysis){}",
        colors::BOLD,
        colors::RESET
    );
    println!("{}", colors::rule());
    println!();
    println!("{}📊  Summary{}", colors::BOLD, colors::RESET);
    println!(
        "  Direct Dependencies:   {}{}{}",
        colors::BOLD_WHITE,
        report.direct_deps,
        colors::RESET
    );
    println!(
        "  Transitive Deps:       {}{}{}",
        colors::BOLD_WHITE,
        report.transitive_deps,
        colors::RESET
    );
    // Labelled by its source: this is the Cargo.lock package set, and it is
    // exactly direct + transitive (it used to be a third, smaller population).
    println!(
        "  Total (Cargo.lock):    {}{}{}",
        colors::BOLD_WHITE,
        report.total_deps,
        colors::RESET
    );
    println!(
        "  Sovereign Stack:       {}{}{} ✅",
        colors::GREEN,
        report.sovereign_deps,
        colors::RESET
    );
    println!(
        "  Replaceable:           {}{}{} 🔄",
        colors::YELLOW,
        report.replaceable_deps,
        colors::RESET
    );
    println!(
        "  Removable:             {}{}{} ❌",
        colors::RED,
        report.removable_deps,
        colors::RESET
    );
    println!(
        "  Heavy (bloat):         {}{}{} ⚠️",
        colors::YELLOW,
        report.heavy_deps,
        colors::RESET
    );
    println!(
        "  Orphans (easy remove): {}{}{} 🎯",
        colors::BOLD_WHITE,
        report.orphan_deps,
        colors::RESET
    );
    println!(
        "  Bridges (connectors):  {}{}{} 🌉",
        colors::BOLD_WHITE,
        report.bridge_deps,
        colors::RESET
    );
    let savings_color = if report.estimated_savings_kb > 0 {
        colors::YELLOW
    } else {
        colors::BOLD_WHITE
    };
    println!(
        "  Est. Savings:          {}~{}KB (~{}MB){}",
        savings_color,
        report.estimated_savings_kb,
        report.estimated_savings_kb / 1024,
        colors::RESET
    );
    println!();

    // Top critical deps by PageRank
    if !report.top_critical.is_empty() {
        println!(
            "{}📈  Critical Dependencies (by PageRank){}",
            colors::BOLD,
            colors::RESET
        );
        println!("  ┌─────────────────────┬──────────┐");
        println!("  │ Dependency          │ Score    │");
        println!("  ├─────────────────────┼──────────┤");
        for (name, score) in report.top_critical.iter().take(5) {
            let name_str = name.get(..name.len().min(19)).unwrap_or(name);
            println!(
                "  │ {}{:<19}{} │ {}{:.6}{} │",
                colors::CYAN,
                name_str,
                colors::RESET,
                colors::BOLD,
                score,
                colors::RESET
            );
        }
        println!("  └─────────────────────┴──────────┘");
        println!(
            "  {}(Higher = more deps depend on it, harder to remove){}",
            colors::DIM,
            colors::RESET
        );
        println!();
    }

    // Group by category
    let removable: Vec<_> = report
        .dependencies
        .iter()
        .filter(|d| d.category == DepCategory::Removable)
        .collect();
    let heavy: Vec<_> = report
        .dependencies
        .iter()
        .filter(|d| d.category == DepCategory::Heavy)
        .collect();
    let replaceable: Vec<_> = report
        .dependencies
        .iter()
        .filter(|d| d.category == DepCategory::Replaceable)
        .collect();
    let dev_only: Vec<_> = report
        .dependencies
        .iter()
        .filter(|d| d.category == DepCategory::DevOnly)
        .collect();

    if !removable.is_empty() {
        println!(
            "{}❌  Removable Dependencies{}",
            colors::BOLD_RED,
            colors::RESET
        );
        println!("  ┌─────────────────────┬────────────────────────────────────────┐");
        println!("  │ Dependency          │ Reason                                 │");
        println!("  ├─────────────────────┼────────────────────────────────────────┤");
        for dep in &removable {
            let name_str = dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name);
            let reason_str = dep
                .reason
                .get(..dep.reason.len().min(38))
                .unwrap_or(&dep.reason);
            println!(
                "  │ {}{:<19}{} │ {:<38} │",
                colors::CYAN,
                name_str,
                colors::RESET,
                reason_str
            );
        }
        println!("  └─────────────────────┴────────────────────────────────────────┘");
        println!();
    }

    if !heavy.is_empty() {
        println!(
            "{}⚠️   Heavy Dependencies (Bloat){}",
            colors::BOLD_YELLOW,
            colors::RESET
        );
        println!("  ┌─────────────────────┬──────────┬─────────────────────────────┐");
        println!("  │ Dependency          │ Size KB  │ Reason                      │");
        println!("  ├─────────────────────┼──────────┼─────────────────────────────┤");
        for dep in &heavy {
            let name_str = dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name);
            let reason_str = dep
                .reason
                .get(..dep.reason.len().min(27))
                .unwrap_or(&dep.reason);
            println!(
                "  │ {}{:<19}{} │ {:>8} │ {:<27} │",
                colors::CYAN,
                name_str,
                colors::RESET,
                dep.estimated_size_kb,
                reason_str
            );
        }
        println!("  └─────────────────────┴──────────┴─────────────────────────────┘");
        println!();
    }

    if !replaceable.is_empty() {
        println!(
            "{}🔄  Replaceable with Sovereign Stack{}",
            colors::BOLD_YELLOW,
            colors::RESET
        );
        println!("  ┌─────────────────────┬─────────────────────┬───────────────────┐");
        println!("  │ Dependency          │ Replacement         │ Benefit           │");
        println!("  ├─────────────────────┼─────────────────────┼───────────────────┤");
        for dep in &replaceable {
            let replacement = dep.replacement.as_deref().unwrap_or("-");
            let name_str = dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name);
            let repl_str = replacement
                .get(..replacement.len().min(19))
                .unwrap_or(replacement);
            let reason_str = dep
                .reason
                .get(..dep.reason.len().min(17))
                .unwrap_or(&dep.reason);
            println!(
                "  │ {}{:<19}{} │ {}{:<19}{} │ {:<17} │",
                colors::CYAN,
                name_str,
                colors::RESET,
                colors::GREEN,
                repl_str,
                colors::RESET,
                reason_str
            );
        }
        println!("  └─────────────────────┴─────────────────────┴───────────────────┘");
        println!();
    }

    if !dev_only.is_empty() {
        println!(
            "{}🧪  Dev-Only Dependencies ({}){}",
            colors::BOLD,
            dev_only.len(),
            colors::RESET
        );
        let names: Vec<_> = dev_only.iter().map(|d| d.name.as_str()).collect();
        println!("  {}", names.join(", "));
        println!();
    }

    print!("{}", all_sections(report, show_all));

    if !report.recommendations.is_empty() {
        println!("{}💡  Recommendations{}", colors::BOLD, colors::RESET);
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
        println!();
    }

    println!("{}", colors::rule());
    print!("{}", all_footer(show_all));
}

/// The `--all`-only sections, or the empty string.
///
/// Returned rather than printed so the flag's whole effect on the text report
/// is one testable value: the defect was that this effect did not exist and
/// `diff` of the two runs was empty.
fn all_sections(report: &DepsAuditReport, show_all: bool) -> String {
    if !show_all {
        // Without the flag the handler has already filtered these two
        // categories out of `report.dependencies`, so the sections would
        // always be empty.
        return String::new();
    }
    let mut out = kept_category_section(
        report,
        DepCategory::Core,
        "🧱  Core Dependencies",
        colors::BOLD,
    );
    out.push_str(&kept_category_section(
        report,
        DepCategory::Sovereign,
        "✅  Sovereign Stack Dependencies",
        colors::BOLD_GREEN,
    ));
    out
}

/// The footer hint, which must not advertise `--all` to a run that used it.
fn all_footer(show_all: bool) -> String {
    if show_all {
        return String::new();
    }
    format!(
        "{}Run with --all to see Core and Sovereign deps{}\n",
        colors::DIM,
        colors::RESET
    )
}

/// Render one `--all` section: the dependencies in `category`.
///
/// These two categories are the whole content of `--all`, so a run that
/// supplies the flag and gets no section back has been told the flag did
/// nothing. An empty section is rendered as an empty section.
fn kept_category_section(
    report: &DepsAuditReport,
    category: DepCategory,
    heading: &str,
    color: colors::Sgr,
) -> String {
    let deps: Vec<_> = report
        .dependencies
        .iter()
        .filter(|d| d.category == category)
        .collect();

    let mut out = format!("{}{}  ({}){}\n", color, heading, deps.len(), colors::RESET);
    if deps.is_empty() {
        out.push_str(&format!(
            "  {}none in this project{}\n\n",
            colors::DIM,
            colors::RESET
        ));
        return out;
    }
    out.push_str("  ┌─────────────────────┬──────────┬─────────────────────────────┐\n");
    out.push_str("  │ Dependency          │ Version  │ Reason                      │\n");
    out.push_str("  ├─────────────────────┼──────────┼─────────────────────────────┤\n");
    for dep in &deps {
        let name_str = dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name);
        let version_str = dep
            .version
            .get(..dep.version.len().min(8))
            .unwrap_or(&dep.version);
        let reason_str = dep
            .reason
            .get(..dep.reason.len().min(27))
            .unwrap_or(&dep.reason);
        out.push_str(&format!(
            "  │ {}{:<19}{} │ {:<8} │ {:<27} │\n",
            colors::CYAN,
            name_str,
            colors::RESET,
            version_str,
            reason_str
        ));
    }
    out.push_str("  └─────────────────────┴──────────┴─────────────────────────────┘\n\n");
    out
}

#[cfg(test)]
mod deps_audit_output_tests {
    //! Covers print_pareto_report + print_text_report in
    //! deps_audit_handlers/output.rs (12 uncov on broad, 0% cov; the rest
    //! comes from the dispatch handler.rs).
    use super::*;
    use crate::cli::handlers::deps_audit_handlers::types::{DepAnalysis, ParetoEntry};

    fn pareto(name: &str, trans_deps: usize, effort: ParetoEffort, roi: f32) -> ParetoEntry {
        ParetoEntry {
            name: name.into(),
            transitive_deps: trans_deps,
            effort,
            roi,
            reason: "test".into(),
            category: DepCategory::Removable,
        }
    }

    fn empty_report() -> DepsAuditReport {
        DepsAuditReport {
            total_deps: 0,
            direct_deps: 0,
            transitive_deps: 0,
            sovereign_deps: 0,
            replaceable_deps: 0,
            removable_deps: 0,
            heavy_deps: 0,
            orphan_deps: 0,
            bridge_deps: 0,
            estimated_savings_kb: 0,
            dependencies: vec![],
            recommendations: vec![],
            top_critical: vec![],
            removal_candidates: vec![],
        }
    }

    #[test]
    fn test_print_pareto_report_empty_returns_early() {
        // Empty entries → "No removable dependencies found." early-return.
        print_pareto_report(&[]);
    }

    #[test]
    fn test_print_pareto_report_single_entry_no_panic() {
        let entries = vec![pareto("foo", 5, ParetoEffort::Low, 5.0)];
        print_pareto_report(&entries);
    }

    #[test]
    fn test_print_pareto_report_emits_80_marker_when_cumulative_threshold_hit() {
        // Build entries where the second one pushes cumulative past 80% threshold.
        let entries = vec![
            pareto("a", 100, ParetoEffort::Low, 100.0),
            pareto("b", 50, ParetoEffort::Medium, 25.0),
            pareto("c", 10, ParetoEffort::High, 3.3),
        ];
        print_pareto_report(&entries);
    }

    #[test]
    fn test_print_pareto_report_more_than_20_entries_takes_top_20() {
        let entries: Vec<ParetoEntry> = (0..25)
            .map(|i| pareto(&format!("dep{i}"), i, ParetoEffort::Low, i as f32))
            .collect();
        print_pareto_report(&entries);
    }

    #[test]
    fn test_print_pareto_report_quick_wins_section_with_low_effort_high_roi() {
        let entries = vec![
            pareto("quick", 50, ParetoEffort::Low, 50.0), // qualifies (Low + ROI > 10)
            pareto("hard", 50, ParetoEffort::High, 50.0), // not Low → skipped
            pareto("low_roi", 5, ParetoEffort::Low, 5.0), // ROI ≤ 10 → skipped
        ];
        print_pareto_report(&entries);
    }

    // ── print_text_report ──

    #[test]
    fn test_print_text_report_empty_no_panic() {
        let r = empty_report();
        print_text_report(&r, false);
    }

    #[test]
    fn test_print_text_report_with_savings_uses_yellow_color() {
        let mut r = empty_report();
        r.estimated_savings_kb = 4096; // > 0 → yellow color branch
        print_text_report(&r, false);
    }

    #[test]
    fn test_print_text_report_with_critical_deps_emits_table() {
        let mut r = empty_report();
        r.top_critical = vec![("serde".into(), 0.123456), ("tokio".into(), 0.098765)];
        print_text_report(&r, false);
    }

    #[test]
    fn test_print_text_report_with_recommendations_filters_heavy_deps() {
        let mut r = empty_report();
        r.dependencies = vec![DepAnalysis {
            name: "huge_dep".into(),
            version: "1.0".into(),
            category: DepCategory::Heavy,
            replacement: None,
            reason: "5MB binary bloat".into(),
            transitive_count: 50,
            estimated_size_kb: 5000,
            pagerank_score: 0.5,
            in_degree: 0,
            out_degree: 0,
            is_bridge: false,
            is_orphan: false,
        }];
        print_text_report(&r, false);
    }
}

#[cfg(test)]
mod deps_audit_all_flag_tests {
    //! `--all` must add rows to the TEXT report, not only to JSON.
    //!
    //! The handler honoured `--all` when it built `report.dependencies`
    //! (dropping the `Core | Sovereign` filter), but `print_text_report`
    //! grouped only Removable / Heavy / Replaceable / DevOnly — precisely the
    //! four categories `--all` does NOT add. So on a project with `serde` and
    //! `rayon` in it, `diff <(deps-audit) <(deps-audit --all)` was empty while
    //! `-f json` differed, and the footer printed "Run with --all to see Core
    //! and Sovereign deps" on the run that had just supplied `--all`.
    use super::*;
    use crate::cli::handlers::deps_audit_handlers::types::DepAnalysis;

    fn dep(name: &str, category: DepCategory) -> DepAnalysis {
        DepAnalysis {
            name: name.into(),
            version: "1.0".into(),
            category,
            replacement: None,
            reason: "fixture".into(),
            transitive_count: 1,
            estimated_size_kb: 10,
            pagerank_score: 0.1,
            in_degree: 0,
            out_degree: 0,
            is_bridge: false,
            is_orphan: false,
        }
    }

    fn report_with_kept_deps() -> DepsAuditReport {
        DepsAuditReport {
            total_deps: 3,
            direct_deps: 3,
            transitive_deps: 0,
            sovereign_deps: 1,
            replaceable_deps: 0,
            removable_deps: 0,
            heavy_deps: 0,
            orphan_deps: 0,
            bridge_deps: 0,
            estimated_savings_kb: 0,
            dependencies: vec![
                dep("serde", DepCategory::Core),
                dep("trueno", DepCategory::Sovereign),
                dep("bloaty", DepCategory::Heavy),
            ],
            recommendations: vec![],
            top_critical: vec![],
            removal_candidates: vec![],
        }
    }

    /// The two categories the flag unlocks reach the text report.
    #[test]
    fn all_adds_core_and_sovereign_sections_to_text() {
        let report = report_with_kept_deps();

        let plain = all_sections(&report, false);
        let all = all_sections(&report, true);

        assert!(
            plain.is_empty(),
            "without --all these categories are filtered out upstream:\n{plain}"
        );
        assert_ne!(plain, all, "--all must change the text report");
        for expected in [
            "Core Dependencies",
            "serde",
            "Sovereign Stack Dependencies",
            "trueno",
        ] {
            assert!(all.contains(expected), "--all must name {expected}:\n{all}");
        }
    }

    /// A `--all` section with nothing in it says so, rather than vanishing and
    /// reading as "the flag did nothing".
    #[test]
    fn empty_all_section_is_still_printed() {
        let mut report = report_with_kept_deps();
        report
            .dependencies
            .retain(|d| d.category != DepCategory::Core);

        let all = all_sections(&report, true);
        assert!(all.contains("Core Dependencies"), "{all}");
        assert!(all.contains("none in this project"), "{all}");
    }

    /// The footer must not advertise a flag the caller has already used.
    #[test]
    fn footer_stops_promising_all_once_all_is_supplied() {
        assert!(all_footer(false).contains("Run with --all"));
        assert_eq!(
            all_footer(true),
            "",
            "--all printed 'Run with --all to see Core and Sovereign deps'"
        );
    }
}
