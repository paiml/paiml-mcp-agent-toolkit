#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{DepCategory, DepsAuditReport, ParetoEffort, ParetoEntry};

/// Print Pareto analysis report
pub fn print_pareto_report(entries: &[ParetoEntry]) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊  Pareto Analysis: 80/20 Dependency Removal");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("ROI = Transitive Deps Saved / Effort");
    println!("Higher ROI = Better bang for buck");
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

        println!(
            "│ {:<19} │ {:>9} │ {:>6} │ {:>6.1} │ {:<21} {:>5} │",
            entry
                .name
                .get(..entry.name.len().min(19))
                .unwrap_or(&entry.name),
            entry.transitive_deps,
            entry.effort.label(),
            entry.roi,
            entry
                .reason
                .get(..entry.reason.len().min(21))
                .unwrap_or(&entry.reason),
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

    println!("💡 Summary:");
    println!(
        "   Total transitive deps from candidates: {}",
        total_transitive
    );
    println!(
        "   Top 5 removals save: {} deps ({}% of total)",
        top_5_savings, top_5_pct
    );
    println!();

    // Actionable commands
    println!("🔧 Quick Wins (Low Effort, High ROI):");
    for entry in entries
        .iter()
        .filter(|e| matches!(e.effort, ParetoEffort::Low) && e.roi > 10.0)
        .take(5)
    {
        println!(
            "   cargo rm {} # saves {} transitive deps",
            entry.name, entry.transitive_deps
        );
    }
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

pub fn print_text_report(report: &DepsAuditReport) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍  Dependency Audit Report (with Graph Analysis)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📊  Summary");
    println!("  Direct Dependencies:   {}", report.direct_deps);
    println!("  Transitive Deps:       {}", report.transitive_deps);
    println!("  Total (graph nodes):   {}", report.total_deps);
    println!("  Sovereign Stack:       {} ✅", report.sovereign_deps);
    println!("  Replaceable:           {} 🔄", report.replaceable_deps);
    println!("  Removable:             {} ❌", report.removable_deps);
    println!("  Heavy (bloat):         {} ⚠️", report.heavy_deps);
    println!("  Orphans (easy remove): {} 🎯", report.orphan_deps);
    println!("  Bridges (connectors):  {} 🌉", report.bridge_deps);
    println!(
        "  Est. Savings:          ~{}KB (~{}MB)",
        report.estimated_savings_kb,
        report.estimated_savings_kb / 1024
    );
    println!();

    // Top critical deps by PageRank
    if !report.top_critical.is_empty() {
        println!("📈  Critical Dependencies (by PageRank)");
        println!("  ┌─────────────────────┬──────────┐");
        println!("  │ Dependency          │ Score    │");
        println!("  ├─────────────────────┼──────────┤");
        for (name, score) in report.top_critical.iter().take(5) {
            println!(
                "  │ {:<19} │ {:.6} │",
                name.get(..name.len().min(19)).unwrap_or(name),
                score
            );
        }
        println!("  └─────────────────────┴──────────┘");
        println!("  (Higher = more deps depend on it, harder to remove)");
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
        println!("❌  Removable Dependencies");
        println!("  ┌─────────────────────┬────────────────────────────────────────┐");
        println!("  │ Dependency          │ Reason                                 │");
        println!("  ├─────────────────────┼────────────────────────────────────────┤");
        for dep in &removable {
            println!(
                "  │ {:<19} │ {:<38} │",
                dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name),
                dep.reason
                    .get(..dep.reason.len().min(38))
                    .unwrap_or(&dep.reason)
            );
        }
        println!("  └─────────────────────┴────────────────────────────────────────┘");
        println!();
    }

    if !heavy.is_empty() {
        println!("⚠️   Heavy Dependencies (Bloat)");
        println!("  ┌─────────────────────┬──────────┬─────────────────────────────┐");
        println!("  │ Dependency          │ Size KB  │ Reason                      │");
        println!("  ├─────────────────────┼──────────┼─────────────────────────────┤");
        for dep in &heavy {
            println!(
                "  │ {:<19} │ {:>8} │ {:<27} │",
                dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name),
                dep.estimated_size_kb,
                dep.reason
                    .get(..dep.reason.len().min(27))
                    .unwrap_or(&dep.reason)
            );
        }
        println!("  └─────────────────────┴──────────┴─────────────────────────────┘");
        println!();
    }

    if !replaceable.is_empty() {
        println!("🔄  Replaceable with Sovereign Stack");
        println!("  ┌─────────────────────┬─────────────────────┬───────────────────┐");
        println!("  │ Dependency          │ Replacement         │ Benefit           │");
        println!("  ├─────────────────────┼─────────────────────┼───────────────────┤");
        for dep in &replaceable {
            let replacement = dep.replacement.as_deref().unwrap_or("-");
            println!(
                "  │ {:<19} │ {:<19} │ {:<17} │",
                dep.name.get(..dep.name.len().min(19)).unwrap_or(&dep.name),
                replacement
                    .get(..replacement.len().min(19))
                    .unwrap_or(replacement),
                dep.reason
                    .get(..dep.reason.len().min(17))
                    .unwrap_or(&dep.reason)
            );
        }
        println!("  └─────────────────────┴─────────────────────┴───────────────────┘");
        println!();
    }

    if !dev_only.is_empty() {
        println!("🧪  Dev-Only Dependencies ({})", dev_only.len());
        let names: Vec<_> = dev_only.iter().map(|d| d.name.as_str()).collect();
        println!("  {}", names.join(", "));
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("💡  Recommendations");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Run with --all to see Core and Sovereign deps");
}
