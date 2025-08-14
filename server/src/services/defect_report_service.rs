//! Comprehensive defect report service
//!
//! This service aggregates defects from all analyzers and generates
//! reports in multiple formats (JSON, CSV, Markdown, Text).

use crate::models::defect_report::{
    Defect, DefectCategory, DefectReport, DefectSummary, FileHotspot, ReportMetadata,
};
use crate::services::defect_analyzer::DefectAnalyzer;
use anyhow::Result;
use chrono::Utc;
use csv::Writer;
use serde_json;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Service for generating comprehensive defect reports
pub struct DefectReportService {
    /// Concurrent analysis limit
    semaphore: Arc<Semaphore>,
}

/// Output format for reports
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    /// JSON format (default)
    Json,
    /// CSV format
    Csv,
    /// Markdown format
    Markdown,
    /// Plain text format
    Text,
}

impl DefectReportService {
    /// Create a new defect report service
    pub fn new() -> Self {
        let cpus = num_cpus::get();
        Self {
            semaphore: Arc::new(Semaphore::new(cpus * 2)),
        }
    }

    /// Generate a comprehensive defect report
    pub async fn generate_report(&self, project_path: &Path) -> Result<DefectReport> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting comprehensive defect analysis for: {}",
            project_path.display()
        );

        // Collect defects from all analyzers in parallel
        let defects = self.collect_all_defects(project_path).await?;

        // Build file index
        let mut file_index = BTreeMap::new();
        for defect in &defects {
            file_index
                .entry(defect.file_path.clone())
                .or_insert_with(Vec::new)
                .push(defect.id.clone());
        }

        // Compute summary statistics
        let summary = self.compute_summary(&defects);

        // Generate report
        let report = DefectReport {
            metadata: ReportMetadata {
                tool: "pmat".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                generated_at: Utc::now(),
                project_root: project_path.to_path_buf(),
                total_files_analyzed: file_index.len(),
                analysis_duration_ms: start_time.elapsed().as_millis() as u64,
            },
            defects,
            summary,
            file_index,
        };

        info!(
            "Defect analysis completed: {} defects found in {} files",
            report.defects.len(),
            report.file_index.len()
        );

        Ok(report)
    }

    /// Collect defects from all analyzers
    async fn collect_all_defects(&self, project_path: &Path) -> Result<Vec<Defect>> {
        let semaphore = self.semaphore.clone();
        let project_path = project_path.to_path_buf();

        // Project path is ready for analyzers

        // Run all analyzers in parallel
        let (complexity, satd, dead_code, duplication, perf, arch) = tokio::join!(
            self.analyze_complexity_defects(&project_path, &semaphore),
            self.analyze_satd_defects(&project_path, &semaphore),
            self.analyze_dead_code_defects(&project_path, &semaphore),
            self.analyze_duplication_defects(&project_path, &semaphore),
            self.analyze_performance_defects(&project_path, &semaphore),
            self.analyze_architecture_defects(&project_path, &semaphore),
        );

        // Merge all defects
        let mut all_defects = Vec::with_capacity(10_000);
        all_defects.extend(complexity?);
        all_defects.extend(satd?);
        all_defects.extend(dead_code?);
        all_defects.extend(duplication?);
        all_defects.extend(perf?);
        all_defects.extend(arch?);

        // Sort by severity, then file, then line
        all_defects.sort_by_key(|d| (d.severity, d.file_path.clone(), d.line_start));

        Ok(all_defects)
    }

    /// Analyze complexity defects
    async fn analyze_complexity_defects(
        &self,
        project_path: &Path,
        semaphore: &Arc<Semaphore>,
    ) -> Result<Vec<Defect>> {
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing complexity defects");

        use crate::services::defect_analyzers::{ComplexityConfig, ComplexityDefectAnalyzer};

        let analyzer = ComplexityDefectAnalyzer;
        let config = ComplexityConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Analyze SATD defects
    async fn analyze_satd_defects(
        &self,
        project_path: &Path,
        semaphore: &Arc<Semaphore>,
    ) -> Result<Vec<Defect>> {
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing SATD defects");

        use crate::services::defect_analyzers::{SATDConfig, SATDDefectAnalyzer};

        let analyzer = SATDDefectAnalyzer::new();
        let config = SATDConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Analyze dead code defects
    async fn analyze_dead_code_defects(
        &self,
        project_path: &Path,
        semaphore: &Arc<Semaphore>,
    ) -> Result<Vec<Defect>> {
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing dead code defects");

        use crate::services::defect_analyzers::{DeadCodeConfig, DeadCodeDefectAnalyzer};

        let analyzer = DeadCodeDefectAnalyzer::new();
        let config = DeadCodeConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Analyze duplication defects
    async fn analyze_duplication_defects(
        &self,
        project_path: &Path,
        semaphore: &Arc<Semaphore>,
    ) -> Result<Vec<Defect>> {
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing duplication defects");

        use crate::services::defect_analyzers::{DuplicationConfig, DuplicationDefectAnalyzer};

        let analyzer = DuplicationDefectAnalyzer::new();
        let config = DuplicationConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Analyze performance defects
    async fn analyze_performance_defects(
        &self,
        project_path: &Path,
        semaphore: &Arc<Semaphore>,
    ) -> Result<Vec<Defect>> {
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing performance defects");

        use crate::services::defect_analyzers::{PerformanceConfig, PerformanceDefectAnalyzer};

        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Analyze architecture defects
    async fn analyze_architecture_defects(
        &self,
        project_path: &Path,
        semaphore: &Arc<Semaphore>,
    ) -> Result<Vec<Defect>> {
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing architecture defects");

        use crate::services::defect_analyzers::{ArchitectureConfig, ArchitectureDefectAnalyzer};

        let analyzer = ArchitectureDefectAnalyzer::new();
        let config = ArchitectureConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Compute summary statistics
    pub fn compute_summary(&self, defects: &[Defect]) -> DefectSummary {
        let mut by_severity = BTreeMap::new();
        let mut by_category = BTreeMap::new();
        let mut file_defect_counts: HashMap<PathBuf, (usize, f64)> = HashMap::new();

        for defect in defects {
            // Count by severity
            *by_severity
                .entry(format!("{:?}", defect.severity).to_lowercase())
                .or_insert(0) += 1;

            // Count by category
            *by_category
                .entry(format!("{:?}", defect.category))
                .or_insert(0) += 1;

            // Track file defect counts and scores
            let (count, score) = file_defect_counts
                .entry(defect.file_path.clone())
                .or_insert((0, 0.0));
            *count += 1;
            *score += defect.severity_weight();
        }

        // Find hotspot files
        let mut hotspots: Vec<_> = file_defect_counts
            .into_iter()
            .map(|(path, (count, score))| FileHotspot {
                path,
                defect_count: count,
                severity_score: score,
            })
            .collect();

        // Sort by severity score descending
        hotspots.sort_by(|a, b| {
            b.severity_score
                .partial_cmp(&a.severity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep top 10 hotspots
        hotspots.truncate(10);

        DefectSummary {
            total_defects: defects.len(),
            by_severity,
            by_category,
            hotspot_files: hotspots,
        }
    }

    /// Format report as JSON
    pub fn format_json(&self, report: &DefectReport) -> Result<String> {
        serde_json::to_string_pretty(report).map_err(Into::into)
    }

    /// Format report as CSV
    pub fn format_csv(&self, report: &DefectReport) -> Result<String> {
        let mut wtr = Writer::from_writer(vec![]);

        // Write headers
        wtr.write_record([
            "id",
            "severity",
            "category",
            "file_path",
            "line_start",
            "line_end",
            "message",
            "rule_id",
            "cyclomatic",
            "cognitive",
        ])?;

        // Write defects
        for defect in &report.defects {
            wtr.write_record([
                &defect.id,
                &format!("{:?}", defect.severity).to_lowercase(),
                &format!("{:?}", defect.category),
                &defect.file_path.display().to_string(),
                &defect.line_start.to_string(),
                &defect.line_end.map(|l| l.to_string()).unwrap_or_default(),
                &defect.message,
                &defect.rule_id,
                &defect
                    .metrics
                    .get("cyclomatic")
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                &defect
                    .metrics
                    .get("cognitive")
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ])?;
        }

        let data = wtr.into_inner()?;
        Ok(String::from_utf8(data)?)
    }

    /// Format report as Markdown
    pub fn format_markdown(&self, report: &DefectReport) -> Result<String> {
        let mut md = String::with_capacity(100_000);

        // Header
        md.push_str("# Code Quality Report\n\n");
        md.push_str(&format!(
            "Generated: {}\n\n",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Executive Summary
        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!(
            "- **Total Defects**: {}\n",
            report.summary.total_defects
        ));
        md.push_str(&format!(
            "- **Files Analyzed**: {}\n",
            report.metadata.total_files_analyzed
        ));
        md.push_str(&format!(
            "- **Analysis Duration**: {}ms\n\n",
            report.metadata.analysis_duration_ms
        ));

        // Severity Distribution
        md.push_str("### Severity Distribution\n\n");
        md.push_str("```\n");

        let total = report.summary.total_defects as f64;
        for (severity, count) in &report.summary.by_severity {
            let percentage = (*count as f64 / total) * 100.0;
            let bar_length = (percentage / 5.0) as usize;
            let progress_bar = "█".repeat(bar_length);
            let empty = "░".repeat(20 - bar_length);
            md.push_str(&format!(
                "{:<8} {}{} {} ({:.1}%)\n",
                severity, progress_bar, empty, count, percentage
            ));
        }
        md.push_str("```\n\n");

        // Top Hotspot Files
        if !report.summary.hotspot_files.is_empty() {
            md.push_str("### Top 10 Hotspot Files\n\n");
            md.push_str("| Rank | File | Defects | Severity Score |\n");
            md.push_str("|------|------|---------|----------------|\n");

            for (i, hotspot) in report.summary.hotspot_files.iter().enumerate() {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.1} |\n",
                    i + 1,
                    hotspot.path.display(),
                    hotspot.defect_count,
                    hotspot.severity_score
                ));
            }
            md.push('\n');
        }

        // Detailed Findings by Category
        md.push_str("## Detailed Findings\n\n");

        for category in DefectCategory::all() {
            let category_defects: Vec<_> = report
                .defects
                .iter()
                .filter(|d| d.category == category)
                .collect();

            if !category_defects.is_empty() {
                md.push_str(&format!(
                    "### {} ({} issues)\n\n",
                    category,
                    category_defects.len()
                ));

                for defect in category_defects.iter().take(10) {
                    md.push_str(&format!(
                        "#### {}:{}-{}\n\n",
                        defect.file_path.display(),
                        defect.line_start,
                        defect.line_end.unwrap_or(defect.line_start)
                    ));
                    md.push_str(&format!("**{}** - {}\n\n", defect.severity, defect.message));

                    if let Some(fix) = &defect.fix_suggestion {
                        md.push_str(&format!("> 💡 **Suggestion**: {}\n\n", fix));
                    }
                }

                if category_defects.len() > 10 {
                    md.push_str(&format!(
                        "_...and {} more {}_\n\n",
                        category_defects.len() - 10,
                        category
                    ));
                }
            }
        }

        Ok(md)
    }

    /// Format report as plain text
    pub fn format_text(&self, report: &DefectReport) -> Result<String> {
        let mut txt = String::with_capacity(50_000);

        // Header
        txt.push_str("CODE QUALITY REPORT\n");
        txt.push_str("===================\n\n");
        txt.push_str(&format!(
            "Generated: {}\n",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        txt.push_str(&format!(
            "Project: {}\n",
            report.metadata.project_root.display()
        ));
        txt.push_str(&format!(
            "Total Defects: {}\n",
            report.summary.total_defects
        ));
        txt.push_str(&format!(
            "Files Analyzed: {}\n\n",
            report.metadata.total_files_analyzed
        ));

        // Summary by severity
        txt.push_str("SEVERITY BREAKDOWN\n");
        txt.push_str("------------------\n");
        for (severity, count) in &report.summary.by_severity {
            txt.push_str(&format!("{:<10} {}\n", severity, count));
        }
        txt.push('\n');

        // Summary by category
        txt.push_str("CATEGORY BREAKDOWN\n");
        txt.push_str("------------------\n");
        for (category, count) in &report.summary.by_category {
            txt.push_str(&format!("{:<20} {}\n", category, count));
        }
        txt.push('\n');

        // Top hotspot files
        if !report.summary.hotspot_files.is_empty() {
            txt.push_str("TOP HOTSPOT FILES\n");
            txt.push_str("-----------------\n");
            for (i, hotspot) in report.summary.hotspot_files.iter().enumerate() {
                txt.push_str(&format!(
                    "{}. {} ({} defects, score: {:.1})\n",
                    i + 1,
                    hotspot.path.display(),
                    hotspot.defect_count,
                    hotspot.severity_score
                ));
            }
            txt.push('\n');
        }

        // List defects
        txt.push_str("DEFECTS\n");
        txt.push_str("-------\n");
        for defect in &report.defects {
            txt.push_str(&format!(
                "[{}] {} - {}:{}",
                defect.severity,
                defect.category,
                defect.file_path.display(),
                defect.line_start
            ));
            if let Some(end) = defect.line_end {
                txt.push_str(&format!("-{}", end));
            }
            txt.push_str(&format!("\n  {}\n", defect.message));
            if let Some(fix) = &defect.fix_suggestion {
                txt.push_str(&format!("  Fix: {}\n", fix));
            }
            txt.push('\n');
        }

        Ok(txt)
    }

    /// Generate filename with timestamp
    pub fn generate_filename(&self, format: ReportFormat) -> String {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        match format {
            ReportFormat::Json => format!("defect-report-{}.json", timestamp),
            ReportFormat::Csv => format!("defect-report-{}.csv", timestamp),
            ReportFormat::Markdown => format!("defect-report-{}.md", timestamp),
            ReportFormat::Text => format!("defect-report-{}.txt", timestamp),
        }
    }
}

impl Default for DefectReportService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::defect_report::Severity;

    #[test]
    fn test_report_format_filename() {
        let service = DefectReportService::new();

        let json_name = service.generate_filename(ReportFormat::Json);
        assert!(json_name.starts_with("defect-report-"));
        assert!(json_name.ends_with(".json"));

        let csv_name = service.generate_filename(ReportFormat::Csv);
        assert!(csv_name.ends_with(".csv"));

        let md_name = service.generate_filename(ReportFormat::Markdown);
        assert!(md_name.ends_with(".md"));

        let txt_name = service.generate_filename(ReportFormat::Text);
        assert!(txt_name.ends_with(".txt"));
    }

    #[test]
    fn test_summary_computation() {
        let service = DefectReportService::new();
        let defects = vec![
            Defect {
                id: "TEST-001".to_string(),
                severity: Severity::Critical,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file1.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
            Defect {
                id: "TEST-002".to_string(),
                severity: Severity::High,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file1.rs"),
                line_start: 10,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test 2".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
        ];

        let summary = service.compute_summary(&defects);
        assert_eq!(summary.total_defects, 2);
        assert_eq!(summary.by_severity.get("critical"), Some(&1));
        assert_eq!(summary.by_severity.get("high"), Some(&1));
        assert_eq!(summary.hotspot_files.len(), 1);
        assert_eq!(summary.hotspot_files[0].defect_count, 2);
        assert_eq!(summary.hotspot_files[0].severity_score, 15.0);
    }
}

// Include additional integration tests
#[cfg(test)]
#[path = "defect_report_service_tests.rs"]
mod integration_tests;
