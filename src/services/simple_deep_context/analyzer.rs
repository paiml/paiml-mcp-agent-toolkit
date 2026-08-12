#![cfg_attr(coverage_nightly, coverage(off))]
//! Core analysis methods for `SimpleDeepContext`.
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

use super::types::{
    ComplexityMetrics, FileComplexityDetail, FileComplexityMetrics, SimpleAnalysisConfig,
    SimpleAnalysisReport, SimpleDeepContext,
};

impl SimpleDeepContext {
    /// Create new simple deep context analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }

    /// Perform simplified deep context analysis
    ///
    /// This function analyzes a Rust project to identify complexity patterns and
    /// provide refactoring recommendations. After fixing issue #33, it now uses
    /// proper AST-based complexity analysis instead of heuristic estimation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisConfig};
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let analyzer = SimpleDeepContext::new();
    /// let config = SimpleAnalysisConfig {
    ///     project_path: PathBuf::from("./my-rust-project"),
    ///     include_features: vec![],
    ///     include_patterns: vec![],
    ///     exclude_patterns: vec![],
    ///     enable_verbose: false,
    /// };
    ///
    /// let report = analyzer.analyze(config).await?;
    ///
    /// // Issue #33 fix: Complexity values are now accurate, not fixed at 1.0
    /// assert!(report.complexity_metrics.total_functions > 0);
    /// assert!(report.complexity_metrics.avg_complexity >= 1.0);
    ///
    /// // High complexity functions are properly detected
    /// if report.complexity_metrics.high_complexity_count > 0 {
    ///     println!("Found {} high-complexity functions",
    ///         report.complexity_metrics.high_complexity_count);
    /// }
    ///
    /// // File-level complexity details are accurate
    /// for detail in &report.file_complexity_details {
    ///     println!("File: {} - {} functions, avg complexity: {:.2}",
    ///         detail.file_path.display(),
    ///         detail.function_count,
    ///         detail.avg_complexity);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze(&self, config: SimpleAnalysisConfig) -> Result<SimpleAnalysisReport> {
        let start_time = Instant::now();
        info!("🔍 Starting simplified deep context analysis");
        info!("📂 Project path: {}", config.project_path.display());

        // Phase 1: File discovery
        let source_files = self.discover_source_files(&config).await?;
        info!("📁 Discovered {} source files", source_files.len());
        for file in &source_files {
            info!("📄 File: {}", file.display());
        }

        // Phase 2: Basic analysis
        let (complexity_metrics, file_complexity_details) =
            self.analyze_complexity(&source_files).await?;

        // Phase 3: Generate recommendations
        //
        // `analyze deep-context` is documented as "deep context analysis with
        // defect detection", but only `--format sarif` ran a defect detector:
        // json/markdown came through here and reported "Code complexity looks
        // good! No immediate recommendations." for the very corpus whose SARIF
        // run listed a High-severity self-admitted-debt finding. The same
        // detector now runs for every format.
        let debts = Self::detect_self_admitted_debt(&source_files);
        let recommendations = Self::with_debt_recommendations(
            self.generate_recommendations(&complexity_metrics),
            &debts,
        );

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

    /// Check if a path matches any include pattern
    fn matches_include_patterns(path: &Path, ext: &str, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        patterns.iter().any(|pattern| {
            // Glob pattern: extract extension from "**/*.rs"
            if let Some(ext_from_pattern) = pattern
                .strip_prefix("**/")
                .and_then(|p| p.strip_prefix("*."))
            {
                return ext == ext_from_pattern;
            }
            // Simple pattern: check if filename or path contains the pattern
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.contains(pattern) || path_str.contains(pattern))
        })
    }

    /// Build the matcher for `--exclude-pattern` (and the common exclusions the
    /// handler appends).
    ///
    /// `analyze deep-context --exclude-pattern '**/*.py'` used to produce a
    /// byte-identical report to a run without it — `SimpleAnalysisConfig` stored
    /// `exclude_patterns` and nothing ever read them. An unparseable pattern is
    /// an error rather than a silently dropped filter, for the same reason.
    pub(super) fn build_exclude_matcher(patterns: &[String]) -> Result<Option<globset::GlobSet>> {
        if patterns.is_empty() {
            return Ok(None);
        }
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            let glob = globset::Glob::new(pattern)
                .map_err(|e| anyhow::anyhow!("invalid --exclude-pattern '{pattern}': {e}"))?;
            builder.add(glob);
        }
        Ok(Some(builder.build()?))
    }

    /// Discover source files in the project
    pub(super) async fn discover_source_files(
        &self,
        config: &SimpleAnalysisConfig,
    ) -> Result<Vec<PathBuf>> {
        use ignore::WalkBuilder;

        let source_extensions = [
            "rs", "js", "ts", "jsx", "tsx", "py", "cpp", "c", "h", "wasm", "wat", "rb", "ruchy",
            "go", "java", "cs", "kt", "sh", "bash", "php", "swift", "lua", "lean",
        ];
        let exclude_dirs = ["target", "node_modules", ".git", "build", "dist"];

        let abs_project_path = if config.project_path.is_absolute() {
            config.project_path.clone()
        } else {
            std::env::current_dir()?.join(&config.project_path)
        };

        let excludes = Self::build_exclude_matcher(&config.exclude_patterns)?;

        let mut files = Vec::new();
        // Gitignore-aware walk. A bare `WalkDir` here counted every file under
        // gitignored paths: on this repo the ephemeral `.claude/worktrees/`
        // duplicate checkouts took `file_count` to 222022 for a tree with 5,497
        // tracked files (and `total_functions` to 1,012,834 against 25,593).
        // Duplicating a checkout must not change what the project measures.
        for entry in WalkBuilder::new(&abs_project_path)
            .follow_links(false)
            .hidden(false)
            .parents(true)
            .ignore(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        {
            let path = entry.path();

            let should_exclude = path.components().any(|comp| {
                comp.as_os_str()
                    .to_str()
                    .is_some_and(|name| exclude_dirs.contains(&name))
            });
            if should_exclude {
                continue;
            }

            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !source_extensions.contains(&ext) {
                continue;
            }

            if let Some(excludes) = &excludes {
                let relative = path.strip_prefix(&abs_project_path).unwrap_or(path);
                if excludes.is_match(path) || excludes.is_match(relative) {
                    continue;
                }
            }

            if config.include_patterns.is_empty()
                || Self::matches_include_patterns(path, ext, &config.include_patterns)
            {
                files.push(path.to_path_buf());
            }
        }

        files.sort();
        Ok(files)
    }

    /// Analyze complexity of source files
    pub(super) async fn analyze_complexity(
        &self,
        files: &[PathBuf],
    ) -> Result<(ComplexityMetrics, Vec<FileComplexityDetail>)> {
        let mut total_functions = 0;
        let mut high_complexity_count = 0;
        let mut complexity_sum = 0.0;
        let mut file_details = Vec::new();

        for file in files {
            let metrics = self.analyze_file_complexity(file.as_path()).await?;
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
                function_names: metrics.function_names.clone(),
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

    /// Analyze complexity of a single file using proper AST-based analysis
    ///
    /// This method uses the unified AST-based complexity analyzer instead of heuristics,
    /// ensuring accurate complexity measurements across all analysis commands.
    ///
    /// # Example
    ///
    /// ```compile_fail
    /// use pmat::services::simple_deep_context::{SimpleDeepContext, FileComplexityMetrics};
    /// use std::path::Path;
    ///
    /// # tokio_test::block_on(async {
    /// let analyzer = SimpleDeepContext::new();
    /// // This is a private method and cannot be called from outside the module
    /// let metrics = analyzer.analyze_file_complexity(Path::new("src/main.rs")).await.unwrap();
    ///
    /// // Metrics now contain accurate AST-based complexity values
    /// assert!(metrics.avg_complexity > 0.0);
    /// # });
    /// ```
    pub(super) async fn analyze_file_complexity(
        &self,
        file_path: &Path,
    ) -> Result<FileComplexityMetrics> {
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        info!(
            "🔍 Analyzing file: {} with extension: {}",
            file_path.display(),
            extension
        );

        // Dispatch complexity analysis to per-language helpers
        let (function_count, high_complexity_functions, avg_complexity) = match extension {
            "rs" => self.complexity_for_rust(file_path).await,
            "ts" | "tsx" | "js" | "jsx" => {
                self.complexity_for_typescript(file_path, extension).await
            }
            "wasm" | "wat" => self.complexity_for_wasm(file_path, extension).await,
            "rb" | "ruchy" => {
                self.complexity_heuristic_fallback(file_path, extension)
                    .await
            }
            "go" => self.complexity_for_go(file_path).await,
            "cs" => self.complexity_for_csharp(file_path).await,
            "kt" => self.complexity_for_kotlin(file_path).await,
            "sh" | "bash" => self.complexity_for_bash(file_path).await,
            "php" => self.complexity_for_php(file_path).await,
            "swift" => self.complexity_for_swift(file_path).await,
            "lua" => self.complexity_for_lua(file_path).await,
            _ => {
                self.complexity_heuristic_fallback(file_path, extension)
                    .await
            }
        };

        // Dispatch function name extraction to per-language helpers
        let function_names = match extension {
            "rs" => self.function_names_for_rust(file_path).await,
            "ts" | "tsx" | "js" | "jsx" => self
                .extract_function_names_heuristic(file_path, extension)
                .await
                .unwrap_or_default(),
            "wasm" | "wat" => self.function_names_for_wasm(file_path, extension).await,
            "go" => self.function_names_for_go(file_path).await,
            "cs" => self.function_names_for_csharp(file_path).await,
            "kt" => self.function_names_for_kotlin(file_path).await,
            "sh" | "bash" => self.function_names_for_bash(file_path).await,
            "php" => self.function_names_for_php(file_path).await,
            "swift" => self.function_names_for_swift(file_path).await,
            "lua" => self.function_names_for_lua(file_path).await,
            _ => self
                .extract_function_names_heuristic(file_path, extension)
                .await
                .unwrap_or_default(),
        };

        // The per-language analyzers above return a MEASURED triple: how many
        // functions they parsed, how many of those exceed the cyclomatic
        // threshold, and the mean cyclomatic over them. Those three agree with
        // each other by construction.
        //
        // This block used to throw two of them away whenever function-name
        // extraction found anything, substituting `names.len() / 4` for the
        // high-complexity count and `2.5` for a zero average. Both are
        // constants wearing a measurement's clothes, and the ratio was visible
        // in the report: `analyze deep-context` on this repo listed
        //
        //   1. comprehensive_assert_cmd_coverage.rs - 1.0 avg complexity
        //      (134 functions, 33 high complexity)
        //
        // — 134/4 = 33 "functions above complexity 10" in a file whose measured
        // mean complexity is 1.0, which is arithmetically impossible. Repo-wide
        // it fabricated 4352 high-complexity functions where the AST had found
        // a small fraction of that.
        //
        // A measurement is now reported only when it was taken. The fix is
        // scoped to the two values that were being invented: name extraction
        // can say nothing about complexity, so it no longer sets
        // `high_complexity_functions` or `avg_complexity`.
        //
        // It remains the better source for the *count*. For languages whose
        // AST complexity pass is a stub in this build — Java, Ruby — the pass
        // reports one function for a file that plainly declares two, while
        // name extraction finds both. Preferring the pass's count there traded
        // a fabrication for an undercount; both are wrong answers, and only
        // the fabrication was the defect being fixed.
        let function_count = if function_names.is_empty() {
            function_count
        } else {
            function_names.len()
        };

        Ok(FileComplexityMetrics {
            function_count,
            high_complexity_functions,
            avg_complexity,
            function_names,
        })
    }

    /// Detect self-admitted technical debt in the discovered source files.
    ///
    /// Uses the same `SATDDetector` the SARIF path reaches through
    /// `DeepContextAnalyzer`, so one command cannot report a defect in one
    /// format and "no immediate recommendations" in another.
    pub(super) fn detect_self_admitted_debt(
        source_files: &[PathBuf],
    ) -> Vec<crate::services::satd_detector::TechnicalDebt> {
        let detector = crate::services::satd_detector::SATDDetector::new();
        let mut debts = Vec::new();
        for file in source_files {
            let Ok(content) = std::fs::read_to_string(file) else {
                continue;
            };
            if let Ok(found) = detector.extract_from_content(&content, file) {
                debts.extend(found);
            }
        }
        // File-discovery order is not guaranteed; the report must be.
        debts.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
        debts
    }

    /// Fold detected debt into the recommendation list.
    ///
    /// "Code complexity looks good! No immediate recommendations." must not
    /// survive alongside a detected defect — that sentence was the whole
    /// contradiction users saw between `--format json` and `--format sarif`.
    pub(super) fn with_debt_recommendations(
        mut recommendations: Vec<String>,
        debts: &[crate::services::satd_detector::TechnicalDebt],
    ) -> Vec<String> {
        if debts.is_empty() {
            return recommendations;
        }
        recommendations.retain(|r| !r.starts_with("Code complexity looks good"));
        let sample: Vec<String> = debts
            .iter()
            .take(3)
            .map(|d| format!("{}:{} {}", d.file.display(), d.line, d.text.trim()))
            .collect();
        recommendations.push(format!(
            "Resolve {} self-admitted technical debt comment(s) (e.g. {})",
            debts.len(),
            sample.join("; ")
        ));
        recommendations
    }

    /// Generate recommendations based on analysis
    pub(super) fn generate_recommendations(&self, metrics: &ComplexityMetrics) -> Vec<String> {
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_json(&self, report: &SimpleAnalysisReport) -> Result<String> {
        let json_report = serde_json::json!({
            "summary": {
                "file_count": report.file_count,
                "analysis_duration_ms": report.analysis_duration.as_millis(),
                "total_functions": report.complexity_metrics.total_functions,
                "high_complexity_functions": report.complexity_metrics.high_complexity_count,
                "avg_complexity": report.complexity_metrics.avg_complexity
            },
            "files": report.file_complexity_details.iter().map(|file| {
                serde_json::json!({
                    "path": file.file_path.to_string_lossy(),
                    "function_count": file.function_count,
                    "high_complexity_functions": file.high_complexity_functions,
                    "avg_complexity": file.avg_complexity,
                    "complexity_score": file.complexity_score
                })
            }).collect::<Vec<_>>(),
            "recommendations": report.recommendations
        });

        Ok(serde_json::to_string_pretty(&json_report)?)
    }

    /// Format report as Markdown
    ///
    /// # Example
    ///
    /// ```rust
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
    ///             function_names: vec![],
    ///         },
    ///         FileComplexityDetail {
    ///             file_path: PathBuf::from("src/lib.rs"),
    ///             function_count: 15,
    ///             high_complexity_functions: 1,
    ///             avg_complexity: 3.8,
    ///             complexity_score: 7.2,
    ///             function_names: vec![],
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
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

            // `if top_files == 0 { 10 }` contradicted the flag's own help text
            // ("0 = all"): `--top-files 0` listed ten files while `--top-files
            // 50` listed fifty. One authority: crate::cli::top_files_count.
            let files_to_show = crate::cli::top_files_count(sorted_files.len(), top_files);
            for (i, file_detail) in sorted_files.iter().take(files_to_show).enumerate() {
                let filename = file_detail
                    .file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map_or_else(
                        || file_detail.file_path.to_string_lossy().to_string(),
                        std::string::ToString::to_string,
                    );
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

#[cfg(test)]
mod top_files_zero_regression_tests {
    //! `--top-files 0` is documented as "0 = all". This renderer read it as
    //! "0 = ten", so `--top-files 0` listed FEWER files than `--top-files 50`.
    use super::*;

    fn report(files: usize) -> SimpleAnalysisReport {
        SimpleAnalysisReport {
            file_count: files,
            analysis_duration: std::time::Duration::from_millis(1),
            complexity_metrics: crate::services::simple_deep_context::types::ComplexityMetrics {
                total_functions: files,
                high_complexity_count: 0,
                avg_complexity: 1.0,
            },
            recommendations: vec![],
            file_complexity_details: (0..files)
                .map(
                    |i| crate::services::simple_deep_context::types::FileComplexityDetail {
                        file_path: PathBuf::from(format!("src/f{i}.rs")),
                        function_count: 1,
                        high_complexity_functions: 0,
                        avg_complexity: 1.0,
                        complexity_score: (files - i) as f64,
                        function_names: vec![],
                    },
                )
                .collect(),
        }
    }

    #[test]
    fn zero_lists_every_file_not_ten() {
        let report = report(25);
        let analyzer = SimpleDeepContext::new();
        let zero = analyzer.format_as_markdown(&report, 0);
        let fifty = analyzer.format_as_markdown(&report, 50);
        assert_eq!(zero, fifty, "0 must mean all, exactly as --help says");
        assert!(
            zero.contains("src/f24.rs") || zero.contains("f24.rs"),
            "--top-files 0 stopped at ten files: {zero}"
        );
        // And a real limit still bites.
        let three = analyzer.format_as_markdown(&report, 3);
        assert!(!three.contains("f3.rs"), "--top-files 3 listed a 4th file");
    }
}

#[cfg(test)]
mod discovery_regression_tests {
    //! What `analyze deep-context` counts as "the project".
    use super::*;

    fn config(dir: &Path, exclude_patterns: Vec<String>) -> SimpleAnalysisConfig {
        SimpleAnalysisConfig {
            project_path: dir.to_path_buf(),
            include_features: vec![],
            include_patterns: vec![],
            exclude_patterns,
            enable_verbose: false,
        }
    }

    #[tokio::test]
    async fn gitignored_paths_are_not_analyzed() {
        let dir = tempfile::tempdir().unwrap();
        // `ignore` applies .gitignore only inside a git repository.
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".gitignore"), "worktrees/\n").unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::create_dir_all(dir.path().join("worktrees/copy")).unwrap();
        std::fs::write(dir.path().join("worktrees/copy/main.rs"), "fn main() {}\n").unwrap();

        let files = SimpleDeepContext::new()
            .discover_source_files(&config(dir.path(), vec![]))
            .await
            .unwrap();

        assert_eq!(
            files.len(),
            1,
            "a gitignored duplicate checkout must not double the file count: {files:?}"
        );
        assert!(files[0].ends_with("main.rs"));
    }

    #[tokio::test]
    async fn exclude_patterns_drop_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.py"), "def main():\n    pass\n").unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        let all = SimpleDeepContext::new()
            .discover_source_files(&config(dir.path(), vec![]))
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        let filtered = SimpleDeepContext::new()
            .discover_source_files(&config(dir.path(), vec!["**/*.py".to_string()]))
            .await
            .unwrap();
        assert_eq!(
            filtered.len(),
            1,
            "--exclude-pattern '**/*.py' must drop the python file: {filtered:?}"
        );
        assert!(filtered[0].ends_with("main.rs"));
    }

    #[tokio::test]
    async fn an_unparseable_exclude_pattern_is_reported_not_dropped() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        let err = SimpleDeepContext::new()
            .discover_source_files(&config(dir.path(), vec!["[".to_string()]))
            .await
            .expect_err("an unparseable pattern must not silently filter nothing");
        assert!(err.to_string().contains("--exclude-pattern"), "{err}");
    }

    #[tokio::test]
    async fn a_detected_defect_reaches_the_json_and_markdown_reports() {
        // corpus/poly reproduced this: `--format sarif` listed a High-severity
        // "FIXME: broken" in main.py while `--format json` reported "Code
        // complexity looks good! No immediate recommendations."
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("main.py"),
            "def f():\n    # FIXME: broken\n    return 1\n",
        )
        .unwrap();

        let report = SimpleDeepContext::new()
            .analyze(config(dir.path(), vec![]))
            .await
            .expect("analysis succeeds");

        let recommendations = report.recommendations.join("\n");
        assert!(
            recommendations.contains("FIXME"),
            "a detected defect must appear in the default report, got: {recommendations}"
        );
        assert!(
            !recommendations.contains("Code complexity looks good"),
            "must not claim all-clear next to a detected defect: {recommendations}"
        );
    }
}
