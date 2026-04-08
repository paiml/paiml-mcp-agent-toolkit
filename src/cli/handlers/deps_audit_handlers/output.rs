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
pub fn print_text_report(report: &DepsAuditReport) {
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
    println!(
        "  Total (graph nodes):   {}{}{}",
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

    if !report.recommendations.is_empty() {
        println!("{}💡  Recommendations{}", colors::BOLD, colors::RESET);
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
        println!();
    }

    println!("{}", colors::rule());
    println!(
        "{}Run with --all to see Core and Sovereign deps{}",
        colors::DIM,
        colors::RESET
    );
}
