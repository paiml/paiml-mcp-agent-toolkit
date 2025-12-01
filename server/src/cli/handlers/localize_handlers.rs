// Fault localization CLI handlers (GH-103)
// Toyota Way: Genchi Genbutsu - Use actual coverage data for debugging

use anyhow::{Context, Result};
use std::path::Path;

use crate::services::fault_localization::{
    FaultLocalizer, LcovParser, ReportFormat, SbflFormula,
};

/// Handle the `pmat localize` command
#[allow(clippy::too_many_arguments)]
pub async fn handle_localize(
    passed_coverage: &Path,
    failed_coverage: &Path,
    passed_count: usize,
    failed_count: usize,
    formula: &str,
    top_n: usize,
    output: Option<&Path>,
    format: &str,
) -> Result<()> {
    println!("\n🔍 Tarantula Fault Localization");
    println!("   Formula: {}", formula);
    println!("   Passed tests: {}", passed_count);
    println!("   Failed tests: {}", failed_count);
    println!("   Top-N: {}", top_n);
    println!();

    // Check tool availability
    if !FaultLocalizer::is_coverage_tool_available() {
        println!("⚠️  Note: cargo-llvm-cov not detected. Install with: cargo install cargo-llvm-cov");
    }

    // Parse LCOV files
    let passed_data = LcovParser::parse_file(passed_coverage)
        .context("Failed to parse passed coverage LCOV file")?;
    let failed_data = LcovParser::parse_file(failed_coverage)
        .context("Failed to parse failed coverage LCOV file")?;

    println!(
        "📊 Parsed coverage: {} passed statements, {} failed statements",
        passed_data.len(),
        failed_data.len()
    );

    // Parse formula
    let sbfl_formula: SbflFormula = formula.parse().unwrap_or(SbflFormula::Tarantula);

    // Run localization
    let result = FaultLocalizer::run_localization(
        &passed_data,
        &failed_data,
        passed_count,
        failed_count,
        sbfl_formula,
        top_n,
    );

    // Determine output format
    let report_format: ReportFormat = format.parse().unwrap_or_else(|_| {
        // Try to infer from output file extension
        if let Some(path) = output {
            match path.extension().and_then(|e| e.to_str()) {
                Some("json") => ReportFormat::Json,
                Some("yaml" | "yml") => ReportFormat::Yaml,
                _ => ReportFormat::Terminal,
            }
        } else {
            ReportFormat::Terminal
        }
    });

    // Generate report
    let report = FaultLocalizer::generate_report(&result, report_format)?;

    // Output
    if let Some(out_path) = output {
        std::fs::write(out_path, &report).context("Failed to write output file")?;
        println!("📄 Report written to: {:?}", out_path);
    } else {
        println!("{}", report);
    }

    // Summary
    if !result.rankings.is_empty() {
        println!("\n💡 Recommendation: Start debugging from the top-ranked locations.");
        println!("   The most suspicious statement is: {}", result.rankings[0].statement);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_handle_localize_basic() {
        // Create temp LCOV files
        let mut passed_file = NamedTempFile::new().unwrap();
        writeln!(
            passed_file,
            "SF:src/main.rs\nDA:1,10\nDA:2,10\nend_of_record"
        )
        .unwrap();

        let mut failed_file = NamedTempFile::new().unwrap();
        writeln!(
            failed_file,
            "SF:src/main.rs\nDA:1,1\nDA:2,0\nend_of_record"
        )
        .unwrap();

        let result = handle_localize(
            passed_file.path(),
            failed_file.path(),
            10,
            1,
            "tarantula",
            5,
            None,
            "terminal",
        )
        .await;

        assert!(result.is_ok());
    }
}
