//! Report Generator
//!
//! Generates comprehensive deep WASM context reports in markdown and HTML formats.

use crate::services::deep_wasm::{DeepWasmReport, DeepWasmResult};

/// Report generator for deep WASM analysis
pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_markdown(&self, report: &DeepWasmReport) -> DeepWasmResult<String> {
        let mut output = String::new();

        output.push_str(&format!("# Deep WASM Context: {}\n\n", report.project_name));
        output.push_str(&format!("Generated: {}\n", report.timestamp));
        output.push_str(&format!("PMAT Version: {}\n\n", report.pmat_version));

        output.push_str("## Pipeline Overview\n\n");
        output.push_str(&format!(
            "- Source Language: {:?}\n",
            report.pipeline_overview.source_language
        ));
        output.push_str(&format!("- Target: {}\n", report.pipeline_overview.target));
        output.push_str(&format!(
            "- Optimization Level: {}\n\n",
            report.pipeline_overview.optimization_level
        ));

        output.push_str("## Source Metrics\n\n");
        output.push_str(&format!("- LOC: {}\n", report.source_metrics.lines_of_code));
        output.push_str(&format!(
            "- Functions: {}\n",
            report.source_metrics.function_count
        ));
        output.push_str(&format!(
            "- Max Complexity: {}\n\n",
            report.source_metrics.max_complexity
        ));

        output.push_str("## WASM Module Analysis\n\n");
        output.push_str(&format!(
            "- Module Size: {} bytes\n",
            report.wasm_module_analysis.module_size_bytes
        ));
        output.push_str(&format!(
            "- Functions: {}\n",
            report.wasm_module_analysis.function_count
        ));
        output.push_str(&format!(
            "- Exported: {}\n\n",
            report.wasm_module_analysis.exported_functions
        ));

        output.push_str("## Source-to-WASM Correlations\n\n");
        if report.correlations.is_empty() {
            output.push_str("No correlations available (DWARF/source map data not provided)\n\n");
        } else {
            output.push_str(&format!("Found {} source-to-WASM mappings:\n\n", report.correlations.len()));

            // Show top 10 highest confidence mappings
            let top_mappings = report.correlations.iter().take(10);
            output.push_str("### Top Confidence Mappings\n\n");
            output.push_str("| Source | Line:Col | WASM Fn | Confidence |\n");
            output.push_str("|--------|----------|---------|------------|\n");

            for mapping in top_mappings {
                let source_file = mapping.source_file.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                output.push_str(&format!(
                    "| {} | {}:{} | {} | {:.0}% |\n",
                    source_file,
                    mapping.source_location.line,
                    mapping.source_location.column,
                    mapping.wasm_function_idx,
                    mapping.confidence * 100.0
                ));
            }
            output.push('\n');

            // Summary statistics
            let avg_confidence = report.correlations.iter()
                .map(|m| m.confidence)
                .sum::<f64>() / report.correlations.len() as f64;

            let high_confidence = report.correlations.iter()
                .filter(|m| m.confidence >= 0.9)
                .count();

            let medium_confidence = report.correlations.iter()
                .filter(|m| m.confidence >= 0.7 && m.confidence < 0.9)
                .count();

            let low_confidence = report.correlations.iter()
                .filter(|m| m.confidence < 0.7)
                .count();

            output.push_str("### Correlation Statistics\n\n");
            output.push_str(&format!("- Average Confidence: {:.1}%\n", avg_confidence * 100.0));
            output.push_str(&format!("- High Confidence (≥90%): {} mappings\n", high_confidence));
            output.push_str(&format!("- Medium Confidence (70-90%): {} mappings\n", medium_confidence));
            output.push_str(&format!("- Low Confidence (<70%): {} mappings\n\n", low_confidence));

            // DWARF vs Source Map breakdown
            let dwarf_count = report.correlations.iter()
                .filter(|m| m.dwarf_die.is_some())
                .count();
            let source_map_count = report.correlations.iter()
                .filter(|m| m.source_map_entry.is_some())
                .count();
            let both_count = report.correlations.iter()
                .filter(|m| m.dwarf_die.is_some() && m.source_map_entry.is_some())
                .count();

            output.push_str("### Data Sources\n\n");
            output.push_str(&format!("- DWARF entries: {} mappings\n", dwarf_count));
            output.push_str(&format!("- Source map entries: {} mappings\n", source_map_count));
            output.push_str(&format!("- Both DWARF + Source Map: {} mappings\n\n", both_count));
        }

        output.push_str("## Quality Gates\n\n");
        if report.quality_gate_results.passed {
            output.push_str("✅ All quality gates passed\n");
        } else {
            output.push_str("❌ Quality gate violations:\n\n");
            for violation in &report.quality_gate_results.violations {
                output.push_str(&format!("- {}: {}\n", violation.rule, violation.message));
            }
        }

        Ok(output)
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::deep_wasm::*;

    #[test]
    fn test_generator_creation() {
        let _gen = ReportGenerator::new();
    }

    #[test]
    fn test_generate_minimal_report() {
        let gen = ReportGenerator::new();
        let report = DeepWasmReport {
            project_name: "test".to_string(),
            timestamp: "2025-10-02".to_string(),
            pmat_version: "2.109.0".to_string(),
            pipeline_overview: PipelineOverview {
                source_language: SourceLanguage::Rust,
                source_version: "1.75.0".to_string(),
                target: "wasm32-unknown-unknown".to_string(),
                optimization_level: "3".to_string(),
                debug_symbols: None,
            },
            source_metrics: SourceMetrics {
                lines_of_code: 100,
                function_count: 5,
                max_complexity: 10,
                wasm_boundary_functions: 2,
            },
            wasm_module_analysis: WasmModuleAnalysis {
                module_size_bytes: 1000,
                function_count: 5,
                exported_functions: 2,
                max_complexity: 10,
                has_dwarf: false,
                has_source_map: false,
            },
            correlations: vec![],
            type_flows: vec![],
            hotspots: vec![],
            quality_gate_results: QualityGateResults {
                passed: true,
                violations: vec![],
            },
            bytecode_analysis: None,
            disassembled_functions: None,
            suspicious_patterns: None,
        };

        let result = gen.generate_markdown(&report);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Deep WASM Context"));
        assert!(markdown.contains("test"));
    }
}
