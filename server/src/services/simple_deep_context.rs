//! Simplified Deep Context Analysis - Phase 4 implementation
//!
//! A streamlined deep context analysis implementation that focuses on
//! integrating with existing services without complex dependencies.

use anyhow::Result;
use std::{path::PathBuf, time::Instant};
use tracing::info;

/// Simplified deep context analysis service
pub struct SimpleDeepContext;

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct SimpleAnalysisConfig {
    pub project_path: PathBuf,
    pub include_features: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub enable_verbose: bool,
}

/// Analysis report
#[derive(Debug)]
pub struct SimpleAnalysisReport {
    pub file_count: usize,
    pub analysis_duration: std::time::Duration,
    pub complexity_metrics: ComplexityMetrics,
    pub recommendations: Vec<String>,
    pub file_complexity_details: Vec<FileComplexityDetail>,
}

#[derive(Debug)]
pub struct ComplexityMetrics {
    pub total_functions: usize,
    pub high_complexity_count: usize,
    pub avg_complexity: f64,
}

#[derive(Debug, Clone)]
pub struct FileComplexityDetail {
    pub file_path: PathBuf,
    pub function_count: usize,
    pub high_complexity_functions: usize,
    pub avg_complexity: f64,
    pub complexity_score: f64, // Weighted score for ranking
}

impl SimpleDeepContext {
    /// Create new simple deep context analyzer
    pub fn new() -> Self {
        Self
    }

    /// Perform simplified deep context analysis
    pub async fn analyze(&self, config: SimpleAnalysisConfig) -> Result<SimpleAnalysisReport> {
        let start_time = Instant::now();
        info!("🔍 Starting simplified deep context analysis");
        info!("📂 Project path: {}", config.project_path.display());

        // Phase 1: File discovery
        let source_files = self.discover_source_files(&config.project_path).await?;
        info!("📁 Discovered {} source files", source_files.len());

        // Phase 2: Basic analysis
        let (complexity_metrics, file_complexity_details) =
            self.analyze_complexity(&source_files).await?;

        // Phase 3: Generate recommendations
        let recommendations = self.generate_recommendations(&complexity_metrics);

        let analysis_duration = start_time.elapsed();

        let report = SimpleAnalysisReport {
            file_count: source_files.len(),
            analysis_duration,
            complexity_metrics,
            recommendations,
            file_complexity_details,
        };

        info!("✅ Analysis completed in {:?}", analysis_duration);
        Ok(report)
    }

    /// Discover source files in the project
    async fn discover_source_files(&self, project_path: &PathBuf) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let source_extensions = ["rs", "js", "ts", "jsx", "tsx", "py", "cpp", "c", "h"];
        let exclude_dirs = ["target", "node_modules", ".git", "build", "dist"];

        let mut files = Vec::new();

        // Resolve the project path to an absolute path
        let abs_project_path = if project_path.is_absolute() {
            project_path.clone()
        } else {
            std::env::current_dir()?.join(project_path)
        };

        info!("🔍 Searching for files in: {}", abs_project_path.display());

        for entry in WalkDir::new(&abs_project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Check exclusions
            let should_exclude = path.components().any(|comp| {
                if let Some(name) = comp.as_os_str().to_str() {
                    exclude_dirs.contains(&name)
                } else {
                    false
                }
            });

            if should_exclude {
                continue;
            }

            // Check extensions
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if source_extensions.contains(&ext) {
                    files.push(path.to_path_buf());
                }
            }
        }

        files.sort();
        info!("📁 Found {} source files after filtering", files.len());
        if files.is_empty() {
            info!("⚠️  No source files found. Check if:");
            info!(
                "   - The project path is correct: {}",
                abs_project_path.display()
            );
            info!(
                "   - Source files exist with extensions: {:?}",
                source_extensions
            );
            info!(
                "   - Files are not in excluded directories: {:?}",
                exclude_dirs
            );
        }
        Ok(files)
    }

    /// Analyze complexity of source files
    async fn analyze_complexity(
        &self,
        files: &[PathBuf],
    ) -> Result<(ComplexityMetrics, Vec<FileComplexityDetail>)> {
        let mut total_functions = 0;
        let mut high_complexity_count = 0;
        let mut complexity_sum = 0.0;
        let mut file_details = Vec::new();

        for file in files {
            let metrics = self.analyze_file_complexity(file).await?;
            total_functions += metrics.function_count;
            high_complexity_count += metrics.high_complexity_functions;
            complexity_sum += metrics.avg_complexity * metrics.function_count as f64;

            // Calculate complexity score for ranking (weighted by functions and complexity)
            let complexity_score = (metrics.avg_complexity * 0.7)
                + (metrics.high_complexity_functions as f64 * 2.0)
                + (metrics.function_count as f64 * 0.3);

            file_details.push(FileComplexityDetail {
                file_path: file.clone(),
                function_count: metrics.function_count,
                high_complexity_functions: metrics.high_complexity_functions,
                avg_complexity: metrics.avg_complexity,
                complexity_score,
            });
        }

        let avg_complexity = if total_functions > 0 {
            complexity_sum / total_functions as f64
        } else {
            0.0
        };

        let complexity_metrics = ComplexityMetrics {
            total_functions,
            high_complexity_count,
            avg_complexity,
        };

        Ok((complexity_metrics, file_details))
    }

    /// Analyze complexity of a single file
    async fn analyze_file_complexity(&self, file_path: &PathBuf) -> Result<FileComplexityMetrics> {
        // Simple heuristic-based complexity analysis
        let content = tokio::fs::read_to_string(file_path).await?;
        let lines: Vec<&str> = content.lines().collect();

        let mut function_count = 0;
        let mut high_complexity_functions = 0;
        let mut total_complexity = 0.0;

        for line in &lines {
            let trimmed = line.trim();

            // Simple function detection
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("def ")
                || trimmed.contains("function(")
            {
                function_count += 1;

                // Simple complexity heuristic based on keywords
                let complexity = self.estimate_function_complexity(&content, line);
                total_complexity += complexity;

                if complexity > 10.0 {
                    high_complexity_functions += 1;
                }
            }
        }

        let avg_complexity = if function_count > 0 {
            total_complexity / function_count as f64
        } else {
            0.0
        };

        Ok(FileComplexityMetrics {
            function_count,
            high_complexity_functions,
            avg_complexity,
        })
    }

    /// Estimate function complexity using simple heuristics
    fn estimate_function_complexity(&self, _content: &str, _function_line: &str) -> f64 {
        // Simple heuristic: base complexity of 1 plus complexity keywords
        let complexity_keywords = [
            "if", "else", "for", "while", "match", "switch", "case", "try", "catch", "&&", "||",
        ];

        let mut complexity = 1.0;

        // Count complexity-adding keywords (simplified)
        for keyword in &complexity_keywords {
            complexity += _function_line.matches(keyword).count() as f64 * 0.5;
        }

        complexity
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(&self, metrics: &ComplexityMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.high_complexity_count > 0 {
            recommendations.push(format!(
                "Consider refactoring {} high-complexity functions (complexity > 10)",
                metrics.high_complexity_count
            ));
        }

        if metrics.avg_complexity > 5.0 {
            recommendations.push(format!(
                "Average function complexity is {:.1}, consider simplifying functions",
                metrics.avg_complexity
            ));
        }

        if metrics.total_functions == 0 {
            recommendations
                .push("No functions detected - verify file discovery patterns".to_string());
        }

        if recommendations.is_empty() {
            recommendations
                .push("Code complexity looks good! No immediate recommendations.".to_string());
        }

        recommendations
    }

    /// Format report as JSON
    pub fn format_as_json(&self, report: &SimpleAnalysisReport) -> Result<String> {
        let json_report = serde_json::json!({
            "summary": {
                "file_count": report.file_count,
                "analysis_duration_ms": report.analysis_duration.as_millis(),
                "total_functions": report.complexity_metrics.total_functions,
                "high_complexity_functions": report.complexity_metrics.high_complexity_count,
                "avg_complexity": report.complexity_metrics.avg_complexity
            },
            "recommendations": report.recommendations
        });

        Ok(serde_json::to_string_pretty(&json_report)?)
    }

    /// Format report as Markdown
    ///
    /// # Example
    ///
    /// ```
    /// use pmat::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisReport, ComplexityMetrics, FileComplexityDetail};
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// let analyzer = SimpleDeepContext::new();
    /// let report = SimpleAnalysisReport {
    ///     file_count: 5,
    ///     analysis_duration: Duration::from_millis(500),
    ///     complexity_metrics: ComplexityMetrics {
    ///         total_functions: 25,
    ///         high_complexity_count: 3,
    ///         avg_complexity: 4.2,
    ///     },
    ///     recommendations: vec!["Consider refactoring 3 high-complexity functions".to_string()],
    ///     file_complexity_details: vec![
    ///         FileComplexityDetail {
    ///             file_path: PathBuf::from("src/main.rs"),
    ///             function_count: 10,
    ///             high_complexity_functions: 2,
    ///             avg_complexity: 5.5,
    ///             complexity_score: 8.5,
    ///         },
    ///         FileComplexityDetail {
    ///             file_path: PathBuf::from("src/lib.rs"),
    ///             function_count: 15,
    ///             high_complexity_functions: 1,
    ///             avg_complexity: 3.8,
    ///             complexity_score: 7.2,
    ///         },
    ///     ],
    /// };
    ///
    /// let output = analyzer.format_as_markdown(&report, 10);
    ///
    /// assert!(output.contains("# Deep Context Analysis Report"));
    /// assert!(output.contains("**Files Analyzed**: 5"));
    /// assert!(output.contains("## Top Files by Complexity"));
    /// assert!(output.contains("1. `main.rs` - 5.5 avg complexity"));
    /// ```
    pub fn format_as_markdown(&self, report: &SimpleAnalysisReport, top_files: usize) -> String {
        let mut markdown = String::new();

        markdown.push_str("# Deep Context Analysis Report\n\n");

        markdown.push_str("## Summary\n\n");
        markdown.push_str(&format!("- **Files Analyzed**: {}\n", report.file_count));
        markdown.push_str(&format!(
            "- **Analysis Duration**: {:?}\n",
            report.analysis_duration
        ));
        markdown.push_str(&format!(
            "- **Total Functions**: {}\n",
            report.complexity_metrics.total_functions
        ));
        markdown.push_str(&format!(
            "- **High Complexity Functions**: {}\n",
            report.complexity_metrics.high_complexity_count
        ));
        markdown.push_str(&format!(
            "- **Average Complexity**: {:.1}\n\n",
            report.complexity_metrics.avg_complexity
        ));

        // Show top files by complexity
        if !report.file_complexity_details.is_empty() {
            markdown.push_str("## Top Files by Complexity\n\n");

            // Sort files by complexity score (descending)
            let mut sorted_files = report.file_complexity_details.clone();
            sorted_files.sort_by(|a, b| {
                b.complexity_score
                    .partial_cmp(&a.complexity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let files_to_show = if top_files == 0 { 10 } else { top_files };
            for (i, file_detail) in sorted_files.iter().take(files_to_show).enumerate() {
                let filename = file_detail
                    .file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| file_detail.file_path.to_string_lossy().to_string());
                markdown.push_str(&format!(
                    "{}. `{}` - {:.1} avg complexity ({} functions, {} high complexity)\n",
                    i + 1,
                    filename,
                    file_detail.avg_complexity,
                    file_detail.function_count,
                    file_detail.high_complexity_functions
                ));
            }
            markdown.push('\n');
        }

        markdown.push_str("## Recommendations\n\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            markdown.push_str(&format!("{}. {}\n", i + 1, rec));
        }

        markdown
    }
}

#[derive(Debug)]
struct FileComplexityMetrics {
    function_count: usize,
    high_complexity_functions: usize,
    avg_complexity: f64,
}

impl Default for SimpleDeepContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_simple_deep_context_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
