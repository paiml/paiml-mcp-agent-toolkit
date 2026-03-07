#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG markdown output formatting

use crate::models::tdg::{TDGHotspot, TDGSummary};
use anyhow::Result;

/// Format TDG results as Markdown
pub fn format_tdg_markdown(
    summary: &TDGSummary,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<String> {
    let mut output = String::new();

    write_tdg_header(&mut output)?;
    write_tdg_summary(&mut output, summary)?;

    if !hotspots.is_empty() {
        write_tdg_hotspots(&mut output, hotspots, include_components)?;
    }

    Ok(output)
}

/// Write TDG markdown header
pub(super) fn write_tdg_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Technical Debt Gradient Analysis\n")?;
    Ok(())
}

/// Write TDG summary section
pub(super) fn write_tdg_summary(output: &mut String, summary: &TDGSummary) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary\n")?;
    writeln!(output, "- **Total Files**: {}", summary.total_files)?;
    writeln!(
        output,
        "- **Critical Files**: {} (TDG > 2.5)",
        summary.critical_files
    )?;
    writeln!(
        output,
        "- **Warning Files**: {} (TDG > 1.5)",
        summary.warning_files
    )?;
    writeln!(output, "- **Average TDG**: {:.3}", summary.average_tdg)?;
    writeln!(output, "- **95th Percentile**: {:.3}", summary.p95_tdg)?;
    writeln!(
        output,
        "- **Estimated Debt**: {:.1} hours\n",
        summary.estimated_debt_hours
    )?;

    Ok(())
}

/// Write TDG hotspots section
pub(super) fn write_tdg_hotspots(
    output: &mut String,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Top Hotspots\n")?;

    for (i, hotspot) in hotspots.iter().enumerate() {
        write_single_hotspot(output, i + 1, hotspot, include_components)?;
    }

    Ok(())
}

/// Write a single hotspot entry
pub(super) fn write_single_hotspot(
    output: &mut String,
    index: usize,
    hotspot: &TDGHotspot,
    include_components: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "### {}. {}\n", index, hotspot.path)?;
    write_hotspot_basic_info(output, hotspot)?;

    if include_components {
        write_component_breakdown(output, hotspot)?;
    }

    Ok(())
}

/// Write basic hotspot information
pub(super) fn write_hotspot_basic_info(output: &mut String, hotspot: &TDGHotspot) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "- **TDG Score**: {:.3}", hotspot.tdg_score)?;
    writeln!(output, "- **Primary Factor**: {}", hotspot.primary_factor)?;
    writeln!(
        output,
        "- **Estimated Hours**: {:.1}\n",
        hotspot.estimated_hours
    )?;

    Ok(())
}

/// Write component breakdown for a hotspot
pub(super) fn write_component_breakdown(output: &mut String, hotspot: &TDGHotspot) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "#### Component Breakdown:")?;
    writeln!(
        output,
        "- Complexity: {:.3}",
        calculate_component_score(hotspot.tdg_score, 0.4)
    )?;
    writeln!(
        output,
        "- Churn: {:.3}",
        calculate_component_score(hotspot.tdg_score, 0.3)
    )?;
    writeln!(
        output,
        "- Duplication: {:.3}",
        calculate_component_score(hotspot.tdg_score, 0.2)
    )?;
    writeln!(
        output,
        "- Coupling: {:.3}\n",
        calculate_component_score(hotspot.tdg_score, 0.1)
    )?;

    Ok(())
}

/// Calculate component score with given weight
pub(super) fn calculate_component_score(tdg_score: f64, weight: f64) -> f64 {
    tdg_score * weight
}
