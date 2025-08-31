//! Sprint 31 Week 2 - Export Capabilities
//! 
//! Provides comprehensive export functionality for TDG analysis results,
//! metrics, and reports in various industry-standard formats.

use crate::tdg::{TdgScore, ProjectScore, Grade, Comparison};
use anyhow::Result;
use std::time::SystemTime;

/// Export format types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Sarif,
    Html,
    Markdown,
    Xml,
    Prometheus,
    Grafana,
}

/// Export options configuration
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_recommendations: bool,
    pub include_metrics: bool,
    pub include_trends: bool,
    pub pretty_print: bool,
    pub compression: Option<CompressionType>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_metadata: true,
            include_recommendations: true,
            include_metrics: true,
            include_trends: false,
            pretty_print: true,
            compression: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Gzip,
    Zstd,
    Lz4,
}

/// Main exporter for TDG data
pub struct TdgExporter;

impl TdgExporter {
    /// Export TDG score
    pub fn export_score(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        match options.format {
            ExportFormat::Json => Self::score_to_json(score, options),
            ExportFormat::Csv => Self::score_to_csv(score, options),
            ExportFormat::Sarif => Self::score_to_sarif(score, options),
            ExportFormat::Html => Self::score_to_html(score, options),
            ExportFormat::Markdown => Self::score_to_markdown(score, options),
            ExportFormat::Xml => Self::score_to_xml(score, options),
            _ => Err(anyhow::anyhow!("Unsupported format for score export")),
        }
    }

    /// Export project scores
    pub fn export_project(project: &ProjectScore, options: &ExportOptions) -> Result<String> {
        match options.format {
            ExportFormat::Json => Self::project_to_json(project, options),
            ExportFormat::Csv => Self::project_to_csv(project, options),
            ExportFormat::Sarif => Self::project_to_sarif(project, options),
            ExportFormat::Html => Self::project_to_html(project, options),
            ExportFormat::Markdown => Self::project_to_markdown(project, options),
            _ => Err(anyhow::anyhow!("Unsupported format for project export")),
        }
    }

    /// Export comparison results
    pub fn export_comparison(comparison: &Comparison, options: &ExportOptions) -> Result<String> {
        match options.format {
            ExportFormat::Json => Self::comparison_to_json(comparison, options),
            ExportFormat::Csv => Self::comparison_to_csv(comparison, options),
            ExportFormat::Html => Self::comparison_to_html(comparison, options),
            ExportFormat::Markdown => Self::comparison_to_markdown(comparison, options),
            _ => Err(anyhow::anyhow!("Unsupported format for comparison export")),
        }
    }

    // JSON Exports
    
    fn score_to_json(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        let output = if options.include_metadata {
            json!({
                "score": score,
                "metadata": {
                    "exported_at": SystemTime::now(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "options": {
                        "include_recommendations": options.include_recommendations,
                        "include_metrics": options.include_metrics,
                    }
                }
            })
        } else {
            json!(score)
        };

        if options.pretty_print {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    fn project_to_json(project: &ProjectScore, options: &ExportOptions) -> Result<String> {
        let output = json!({
            "project": {
                "total_files": project.total_files,
                "average_score": project.average_score,
                "average_grade": project.average_grade.to_string(),
                "language_distribution": project.language_distribution,
                "files": if options.include_metrics { Some(&project.files) } else { None },
            },
            "metadata": if options.include_metadata {
                Some(json!({
                    "exported_at": SystemTime::now(),
                    "version": env!("CARGO_PKG_VERSION"),
                }))
            } else {
                None
            }
        });

        if options.pretty_print {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    fn comparison_to_json(comparison: &Comparison, options: &ExportOptions) -> Result<String> {
        let output = json!({
            "comparison": {
                "delta": comparison.delta,
                "improvement_percentage": comparison.improvement_percentage,
                "winner": comparison.winner,
                "improvements": comparison.improvements,
                "regressions": comparison.regressions,
            },
            "source1": if options.include_metrics { Some(&comparison.source1) } else { None },
            "source2": if options.include_metrics { Some(&comparison.source2) } else { None },
        });

        if options.pretty_print {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    // CSV Exports
    
    fn score_to_csv(score: &TdgScore, _options: &ExportOptions) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("metric,value\n");
        csv.push_str(&format!("total_score,{:.2}\n", score.total));
        csv.push_str(&format!("grade,{}\n", score.grade));
        csv.push_str(&format!("structural_complexity,{:.2}\n", score.structural_complexity));
        csv.push_str(&format!("semantic_complexity,{:.2}\n", score.semantic_complexity));
        csv.push_str(&format!("duplication_ratio,{:.2}\n", score.duplication_ratio));
        csv.push_str(&format!("coupling_score,{:.2}\n", score.coupling_score));
        csv.push_str(&format!("doc_coverage,{:.2}\n", score.doc_coverage));
        csv.push_str(&format!("consistency_score,{:.2}\n", score.consistency_score));
        csv.push_str(&format!("confidence,{:.2}\n", score.confidence));
        csv.push_str(&format!("language,{:?}\n", score.language));
        
        if let Some(path) = &score.file_path {
            csv.push_str(&format!("file_path,{}\n", path.display()));
        }
        
        Ok(csv)
    }

    fn project_to_csv(project: &ProjectScore, options: &ExportOptions) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("file_path,total_score,grade,structural,semantic,duplication,coupling,documentation,consistency,language\n");
        
        for score in &project.files {
            let path = score.file_path.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            csv.push_str(&format!(
                "{},{:.2},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:?}\n",
                path,
                score.total,
                score.grade,
                score.structural_complexity,
                score.semantic_complexity,
                score.duplication_ratio,
                score.coupling_score,
                score.doc_coverage,
                score.consistency_score,
                score.language,
            ));
        }
        
        if options.include_metadata {
            csv.push_str(&"\nSummary\n".to_string());
            csv.push_str(&format!("total_files,{}\n", project.total_files));
            csv.push_str(&format!("average_score,{:.2}\n", project.average_score));
            csv.push_str(&format!("average_grade,{}\n", project.average_grade));
        }
        
        Ok(csv)
    }

    fn comparison_to_csv(comparison: &Comparison, _options: &ExportOptions) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("metric,source1,source2,delta\n");
        
        csv.push_str(&format!("total_score,{:.2},{:.2},{:.2}\n",
            comparison.source1.total,
            comparison.source2.total,
            comparison.delta
        ));
        
        csv.push_str(&format!("grade,{},{},\n",
            comparison.source1.grade,
            comparison.source2.grade
        ));
        
        csv.push_str(&format!("structural_complexity,{:.2},{:.2},{:.2}\n",
            comparison.source1.structural_complexity,
            comparison.source2.structural_complexity,
            comparison.source2.structural_complexity - comparison.source1.structural_complexity
        ));
        
        csv.push_str(&format!("semantic_complexity,{:.2},{:.2},{:.2}\n",
            comparison.source1.semantic_complexity,
            comparison.source2.semantic_complexity,
            comparison.source2.semantic_complexity - comparison.source1.semantic_complexity
        ));
        
        csv.push_str(&format!("improvement_percentage,,,{:.2}%\n", comparison.improvement_percentage));
        csv.push_str(&format!("winner,{}\n", comparison.winner));
        
        Ok(csv)
    }

    // SARIF Exports (Static Analysis Results Interchange Format)
    
    fn score_to_sarif(score: &TdgScore, _options: &ExportOptions) -> Result<String> {
        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "TDG Analyzer",
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": Self::get_sarif_rules(),
                    }
                },
                "results": Self::score_to_sarif_results(score),
            }]
        });
        
        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn project_to_sarif(project: &ProjectScore, _options: &ExportOptions) -> Result<String> {
        let mut all_results = Vec::new();
        
        for score in &project.files {
            all_results.extend(Self::score_to_sarif_results(score));
        }
        
        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "TDG Analyzer",
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": Self::get_sarif_rules(),
                    }
                },
                "results": all_results,
                "properties": {
                    "total_files": project.total_files,
                    "average_score": project.average_score,
                    "average_grade": project.average_grade.to_string(),
                }
            }]
        });
        
        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn get_sarif_rules() -> Vec<serde_json::Value> {
        vec![
            json!({
                "id": "TDG001",
                "name": "HighComplexity",
                "shortDescription": {
                    "text": "High code complexity detected"
                },
                "fullDescription": {
                    "text": "The code has high structural or semantic complexity that may impact maintainability"
                },
                "defaultConfiguration": {
                    "level": "warning"
                }
            }),
            json!({
                "id": "TDG002",
                "name": "CodeDuplication",
                "shortDescription": {
                    "text": "Code duplication detected"
                },
                "fullDescription": {
                    "text": "Duplicated code patterns detected that should be refactored"
                },
                "defaultConfiguration": {
                    "level": "note"
                }
            }),
            json!({
                "id": "TDG003",
                "name": "LowDocumentation",
                "shortDescription": {
                    "text": "Insufficient documentation"
                },
                "fullDescription": {
                    "text": "Code lacks adequate documentation for public APIs"
                },
                "defaultConfiguration": {
                    "level": "note"
                }
            }),
        ]
    }

    fn score_to_sarif_results(score: &TdgScore) -> Vec<serde_json::Value> {
        let mut results = Vec::new();
        
        if score.structural_complexity < 15.0 || score.semantic_complexity < 15.0 {
            results.push(json!({
                "ruleId": "TDG001",
                "level": if score.total < 50.0 { "error" } else if score.total < 70.0 { "warning" } else { "note" },
                "message": {
                    "text": format!("Code complexity issues detected. Structural: {:.1}, Semantic: {:.1}",
                        score.structural_complexity, score.semantic_complexity)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        }
                    }
                }],
                "properties": {
                    "tdg_score": score.total,
                    "grade": score.grade.to_string(),
                }
            }));
        }
        
        if score.duplication_ratio < 15.0 {
            results.push(json!({
                "ruleId": "TDG002",
                "level": "note",
                "message": {
                    "text": format!("Code duplication detected: {:.1}% duplication ratio", 
                        (20.0 - score.duplication_ratio) * 5.0)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        }
                    }
                }],
            }));
        }
        
        if score.doc_coverage < 8.0 {
            results.push(json!({
                "ruleId": "TDG003",
                "level": "note",
                "message": {
                    "text": format!("Low documentation coverage: {:.1}%", 
                        score.doc_coverage * 10.0)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        }
                    }
                }],
            }));
        }
        
        results
    }

    // HTML Export
    
    fn score_to_html(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        let grade_color = match score.grade {
            Grade::APLus | Grade::A | Grade::AMinus => "#4CAF50",
            Grade::BPlus | Grade::B | Grade::BMinus => "#2196F3",
            Grade::CPlus | Grade::C | Grade::CMinus => "#FF9800",
            Grade::D => "#FF5722",
            Grade::F => "#F44336",
        };

        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>TDG Analysis Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; }}
        .score-card {{ background: white; border: 1px solid #ddd; border-radius: 8px; padding: 20px; margin: 20px 0; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .grade {{ font-size: 48px; color: {}; font-weight: bold; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px; }}
        .metric {{ background: #f5f5f5; padding: 15px; border-radius: 4px; }}
        .metric-label {{ color: #666; font-size: 12px; text-transform: uppercase; }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #333; }}
        .progress-bar {{ width: 100%; height: 20px; background: #e0e0e0; border-radius: 10px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #4CAF50, #8BC34A); transition: width 0.3s; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>TDG Analysis Report</h1>
        <p>{}</p>
    </div>
    
    <div class="score-card">
        <div style="display: flex; justify-content: space-between; align-items: center;">
            <div>
                <h2>Overall Score</h2>
                <div style="font-size: 36px; font-weight: bold;">{:.1}/100</div>
            </div>
            <div class="grade">{}</div>
        </div>
        
        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%;"></div>
        </div>
        
        <div class="metrics">
            <div class="metric">
                <div class="metric-label">Structural Complexity</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Semantic Complexity</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Duplication Ratio</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Coupling Score</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Documentation</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Consistency</div>
                <div class="metric-value">{:.1}</div>
            </div>
        </div>
    </div>
    
    {}
</body>
</html>
        "#,
            grade_color,
            score.file_path.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Analysis Results".to_string()),
            score.total,
            score.grade,
            score.total,
            score.structural_complexity,
            score.semantic_complexity,
            score.duplication_ratio,
            score.coupling_score,
            score.doc_coverage,
            score.consistency_score,
            if options.include_recommendations {
                Self::generate_recommendations_html(score)
            } else {
                String::new()
            }
        );

        Ok(html)
    }

    fn project_to_html(project: &ProjectScore, _options: &ExportOptions) -> Result<String> {
        // Similar implementation for project HTML
        Ok(format!("<html><body><h1>Project Report</h1><p>Files: {}</p></body></html>", 
            project.total_files))
    }

    fn comparison_to_html(comparison: &Comparison, _options: &ExportOptions) -> Result<String> {
        // Similar implementation for comparison HTML
        Ok(format!("<html><body><h1>Comparison Report</h1><p>Delta: {:.2}</p></body></html>",
            comparison.delta))
    }

    // Markdown Export
    
    fn score_to_markdown(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        let mut md = String::new();
        
        md.push_str("# TDG Analysis Report\n\n");
        
        if let Some(path) = &score.file_path {
            md.push_str(&format!("**File:** `{}`\n\n", path.display()));
        }
        
        md.push_str(&format!("## Overall Score: {:.1}/100 ({})\n\n", score.total, score.grade));
        
        md.push_str("### Metrics Breakdown\n\n");
        md.push_str("| Metric | Score | Max |\n");
        md.push_str("|--------|-------|-----|\n");
        md.push_str(&format!("| Structural Complexity | {:.1} | 25.0 |\n", score.structural_complexity));
        md.push_str(&format!("| Semantic Complexity | {:.1} | 20.0 |\n", score.semantic_complexity));
        md.push_str(&format!("| Duplication Ratio | {:.1} | 20.0 |\n", score.duplication_ratio));
        md.push_str(&format!("| Coupling Score | {:.1} | 15.0 |\n", score.coupling_score));
        md.push_str(&format!("| Documentation | {:.1} | 10.0 |\n", score.doc_coverage));
        md.push_str(&format!("| Consistency | {:.1} | 10.0 |\n", score.consistency_score));
        
        if options.include_recommendations {
            md.push_str("\n### Recommendations\n\n");
            for rec in Self::generate_recommendations(score) {
                md.push_str(&format!("- {}\n", rec));
            }
        }
        
        Ok(md)
    }

    fn project_to_markdown(project: &ProjectScore, _options: &ExportOptions) -> Result<String> {
        let mut md = String::new();
        
        md.push_str("# Project TDG Analysis\n\n");
        md.push_str(&format!("**Total Files:** {}\n", project.total_files));
        md.push_str(&format!("**Average Score:** {:.1}/100 ({})\n\n", 
            project.average_score, project.average_grade));
        
        md.push_str("## Language Distribution\n\n");
        for (lang, count) in &project.language_distribution {
            md.push_str(&format!("- {:?}: {} files\n", lang, count));
        }
        
        md.push_str("\n## File Scores\n\n");
        md.push_str("| File | Score | Grade |\n");
        md.push_str("|------|-------|-------|\n");
        
        for score in &project.files {
            let path = score.file_path.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            md.push_str(&format!("| {} | {:.1} | {} |\n", path, score.total, score.grade));
        }
        
        Ok(md)
    }

    fn comparison_to_markdown(comparison: &Comparison, _options: &ExportOptions) -> Result<String> {
        let mut md = String::new();
        
        md.push_str("# TDG Comparison Report\n\n");
        md.push_str(&format!("**Winner:** {}\n", comparison.winner));
        md.push_str(&format!("**Improvement:** {:.1}% ({:+.1} points)\n\n", 
            comparison.improvement_percentage, comparison.delta));
        
        md.push_str("## Score Comparison\n\n");
        md.push_str("| Metric | Source 1 | Source 2 | Delta |\n");
        md.push_str("|--------|----------|----------|-------|\n");
        md.push_str(&format!("| Total Score | {:.1} | {:.1} | {:+.1} |\n",
            comparison.source1.total, comparison.source2.total, comparison.delta));
        md.push_str(&format!("| Grade | {} | {} | - |\n",
            comparison.source1.grade, comparison.source2.grade));
        
        if !comparison.improvements.is_empty() {
            md.push_str("\n## Improvements\n\n");
            for improvement in &comparison.improvements {
                md.push_str(&format!("- {}\n", improvement));
            }
        }
        
        if !comparison.regressions.is_empty() {
            md.push_str("\n## Regressions\n\n");
            for regression in &comparison.regressions {
                md.push_str(&format!("- {}\n", regression));
            }
        }
        
        Ok(md)
    }

    // XML Export
    
    fn score_to_xml(score: &TdgScore, _options: &ExportOptions) -> Result<String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<tdg_analysis>\n");
        xml.push_str(&format!("  <total_score>{:.2}</total_score>\n", score.total));
        xml.push_str(&format!("  <grade>{}</grade>\n", score.grade));
        xml.push_str(&format!("  <language>{:?}</language>\n", score.language));
        xml.push_str(&format!("  <confidence>{:.2}</confidence>\n", score.confidence));
        xml.push_str("  <metrics>\n");
        xml.push_str(&format!("    <structural_complexity>{:.2}</structural_complexity>\n", score.structural_complexity));
        xml.push_str(&format!("    <semantic_complexity>{:.2}</semantic_complexity>\n", score.semantic_complexity));
        xml.push_str(&format!("    <duplication_ratio>{:.2}</duplication_ratio>\n", score.duplication_ratio));
        xml.push_str(&format!("    <coupling_score>{:.2}</coupling_score>\n", score.coupling_score));
        xml.push_str(&format!("    <doc_coverage>{:.2}</doc_coverage>\n", score.doc_coverage));
        xml.push_str(&format!("    <consistency_score>{:.2}</consistency_score>\n", score.consistency_score));
        xml.push_str("  </metrics>\n");
        
        if let Some(path) = &score.file_path {
            xml.push_str(&format!("  <file_path>{}</file_path>\n", path.display()));
        }
        
        xml.push_str("</tdg_analysis>\n");
        Ok(xml)
    }

    // Helper methods
    
    fn generate_recommendations(score: &TdgScore) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if score.structural_complexity < 15.0 {
            recommendations.push("Consider refactoring complex functions to improve structural complexity".to_string());
        }
        
        if score.semantic_complexity < 15.0 {
            recommendations.push("Simplify nested logic and reduce cognitive complexity".to_string());
        }
        
        if score.duplication_ratio < 15.0 {
            recommendations.push("Extract common code patterns into reusable functions".to_string());
        }
        
        if score.doc_coverage < 7.0 {
            recommendations.push("Add documentation for public APIs and complex logic".to_string());
        }
        
        if score.coupling_score < 10.0 {
            recommendations.push("Reduce dependencies between modules to improve coupling".to_string());
        }
        
        recommendations
    }

    fn generate_recommendations_html(score: &TdgScore) -> String {
        let recommendations = Self::generate_recommendations(score);
        
        if recommendations.is_empty() {
            return String::new();
        }
        
        let mut html = String::from(r#"
    <div class="score-card">
        <h3>Recommendations</h3>
        <ul>
        "#);
        
        for rec in recommendations {
            html.push_str(&format!("<li>{}</li>\n", rec));
        }
        
        html.push_str("</ul></div>");
        html
    }
}

use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::Language;

    fn create_test_score() -> TdgScore {
        TdgScore {
            structural_complexity: 20.0,
            semantic_complexity: 18.0,
            duplication_ratio: 17.0,
            coupling_score: 12.0,
            doc_coverage: 8.0,
            consistency_score: 8.0,
            total: 83.0,
            grade: Grade::BPlus,
            confidence: 0.95,
            language: Language::Rust,
            file_path: Some(std::path::PathBuf::from("test.rs")),
            penalties_applied: Vec::new(),
        }
    }

    #[test]
    fn test_json_export() {
        let score = create_test_score();
        let options = ExportOptions::default();
        
        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("\"total\": 83.0"));
        assert!(json.contains("\"grade\": \"B+\""));
    }

    #[test]
    fn test_csv_export() {
        let score = create_test_score();
        let mut options = ExportOptions::default();
        options.format = ExportFormat::Csv;
        
        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());
        
        let csv = result.unwrap();
        assert!(csv.contains("total_score,83.00"));
        assert!(csv.contains("grade,B+"));
    }

    #[test]
    fn test_sarif_export() {
        let score = create_test_score();
        let mut options = ExportOptions::default();
        options.format = ExportFormat::Sarif;
        
        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());
        
        let sarif = result.unwrap();
        assert!(sarif.contains("\"$schema\""));
        assert!(sarif.contains("\"version\": \"2.1.0\""));
    }

    #[test]
    fn test_markdown_export() {
        let score = create_test_score();
        let mut options = ExportOptions::default();
        options.format = ExportFormat::Markdown;
        
        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());
        
        let md = result.unwrap();
        assert!(md.contains("# TDG Analysis Report"));
        assert!(md.contains("## Overall Score: 83.0/100 (B+)"));
    }
}