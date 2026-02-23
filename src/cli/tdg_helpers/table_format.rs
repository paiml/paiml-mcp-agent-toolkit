#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG table output formatting

use crate::models::tdg::TDGHotspot;
use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Format TDG results as table
pub fn format_tdg_table(hotspots: &[TDGHotspot], verbose: bool) -> Result<String> {
    let mut output = String::new();

    writeln!(
        &mut output,
        "| File | TDG Score | Primary Factor | Est. Hours |"
    )?;
    writeln!(
        &mut output,
        "|------|-----------|----------------|------------|"
    )?;

    for hotspot in hotspots {
        let filename = Path::new(&hotspot.path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| hotspot.path.clone());

        writeln!(
            &mut output,
            "| {} | {:.2} | {} | {:.1} |",
            filename, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
        )?;

        if verbose {
            writeln!(
                &mut output,
                "|   Components: complexity={:.2}, churn={:.2}, dup={:.2}, coupling={:.2} ||||",
                hotspot.tdg_score * 0.4,
                hotspot.tdg_score * 0.3,
                hotspot.tdg_score * 0.2,
                hotspot.tdg_score * 0.1
            )?;
        }
    }

    Ok(output)
}
