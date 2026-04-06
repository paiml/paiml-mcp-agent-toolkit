// TDG formatter helper functions for markdown generation.
// Extracted from tdg_formatter.rs: header/summary, file percentages, hotspots, and components sections.

/// Toyota Way: Extract Method - Add header and summary (complexity <=8)
fn add_header_and_summary(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    md.push_str("# Technical Debt Gradient Analysis\n\n");
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Total Files**: {}\n", summary.total_files));

    if summary.total_files > 0 {
        add_file_percentages(md, summary);
    }

    md.push_str(&format!("- **Average TDG**: {:.2}\n", summary.average_tdg));
    md.push_str(&format!("- **95th Percentile**: {:.2}\n", summary.p95_tdg));
    md.push_str(&format!("- **99th Percentile**: {:.2}\n", summary.p99_tdg));
    md.push_str(&format!(
        "- **Estimated Technical Debt**: {:.1} hours\n\n",
        summary.estimated_debt_hours
    ));
}

/// Toyota Way: Extract Method - Add file percentages (complexity <=3)
fn add_file_percentages(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    let critical_pct = (summary.critical_files as f64 / summary.total_files as f64) * 100.0;
    let warning_pct = (summary.warning_files as f64 / summary.total_files as f64) * 100.0;

    md.push_str(&format!(
        "- **Critical Files**: {} ({:.1}%)\n",
        summary.critical_files, critical_pct
    ));
    md.push_str(&format!(
        "- **Warning Files**: {} ({:.1}%)\n",
        summary.warning_files, warning_pct
    ));
}

/// Toyota Way: Extract Method - Add hotspots section (complexity <=6)
fn add_hotspots_section(md: &mut String, hotspots: &[crate::models::tdg::TDGHotspot]) {
    md.push_str("## Hotspots\n\n");

    for (i, hotspot) in hotspots.iter().enumerate() {
        md.push_str(&format!("### {}. {}\n\n", i + 1, hotspot.path));
        md.push_str(&format!("- **TDG Score**: {:.2}\n", hotspot.tdg_score));
        md.push_str(&format!(
            "- **Primary Factor**: {}\n",
            hotspot.primary_factor
        ));
        md.push_str(&format!(
            "- **Estimated Refactoring Time**: {:.1} hours\n\n",
            hotspot.estimated_hours
        ));
    }
}

/// Toyota Way: Extract Method - Add components section (complexity <=2)
fn add_components_section(md: &mut String) {
    md.push_str("## TDG Components\n\n");
    md.push_str(
        "The Technical Debt Gradient is calculated using the following weighted components:\n\n",
    );
    md.push_str("- **Complexity** (30%): Cyclomatic and cognitive complexity\n");
    md.push_str("- **Code Churn** (35%): Frequency of changes over time\n");
    md.push_str("- **Coverage** (20%): Test coverage and quality\n");
    md.push_str("- **Maintainability** (15%): Code quality metrics\n\n");
}
