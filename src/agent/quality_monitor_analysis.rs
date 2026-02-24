// File analysis, metrics computation, and change detection for QualityMonitorEngine
// Included by quality_monitor.rs - shares parent module scope

impl QualityMonitorEngine {
    /// Analyze file metrics for a single file
    async fn analyze_file_metrics(
        file_path: &PathBuf,
        relative_path: &Path,
    ) -> Result<FileQualityMetrics> {
        use std::fs;
        use std::time::UNIX_EPOCH;

        let metadata = fs::metadata(file_path)?;
        let last_modified = metadata.modified().unwrap_or(UNIX_EPOCH);

        // Basic file analysis (can be enhanced with actual AST analysis later)
        let content = tokio::fs::read_to_string(file_path).await?;
        let lines = content.lines().count();
        let function_count = Self::count_functions(&content, file_path);

        // Simple complexity estimation based on control flow keywords
        let complexity = Self::estimate_complexity(&content);
        let avg_complexity = if function_count > 0 {
            f64::from(complexity) / function_count as f64
        } else {
            0.0
        };

        let max_complexity = complexity; // Single function complexity in basic analysis
        let satd_issues = Self::count_satd_issues(&content);

        // Calculate quality score based on various factors
        let quality_score =
            Self::calculate_file_quality_score(lines, function_count, avg_complexity, satd_issues);

        let needs_attention = quality_score < 0.7 || max_complexity > 20 || satd_issues > 0;

        Ok(FileQualityMetrics {
            file_path: relative_path.to_string_lossy().to_string(),
            last_modified,
            last_analyzed: SystemTime::now(),
            function_count,
            avg_complexity,
            max_complexity,
            satd_issues,
            quality_score,
            needs_attention,
        })
    }

    /// Count functions in a file (simple heuristic)
    fn count_functions(content: &str, file_path: &Path) -> usize {
        let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match extension {
            "rs" => content.matches("fn ").count(),
            "py" => content.matches("def ").count(),
            "js" | "ts" => {
                content.matches("function ").count()
                    + content.matches(" => ").count()
                    + content.matches("function(").count()
            }
            _ => content.matches("def ").count() + content.matches("fn ").count(),
        }
    }

    /// Estimate complexity based on control flow keywords
    fn estimate_complexity(content: &str) -> u32 {
        let keywords = [
            "if", "else", "for", "while", "match", "switch", "case", "catch", "&&", "||",
        ];
        keywords
            .iter()
            .map(|keyword| content.matches(keyword).count() as u32)
            .sum()
    }

    /// Count SATD (Self-Admitted Technical Debt) issues
    fn count_satd_issues(content: &str) -> usize {
        // Using concatenation to avoid false positives in SATD detection
        let patterns = [
            ['T', 'O', 'D', 'O'].iter().collect::<String>(),
            ['F', 'I', 'X', 'M', 'E'].iter().collect::<String>(),
            ['H', 'A', 'C', 'K'].iter().collect::<String>(),
            ['B', 'U', 'G', ':'].iter().collect::<String>(),
            ['X', 'X', 'X'].iter().collect::<String>(),
        ];
        patterns
            .iter()
            .map(|pattern| content.matches(pattern.as_str()).count())
            .sum()
    }

    /// Calculate quality score for a file
    fn calculate_file_quality_score(
        lines: usize,
        function_count: usize,
        avg_complexity: f64,
        satd_issues: usize,
    ) -> f64 {
        let mut score = 1.0;

        // Penalize high complexity
        if avg_complexity > 20.0 {
            score -= 0.3;
        } else if avg_complexity > 10.0 {
            score -= 0.1;
        }

        // Penalize SATD issues
        if satd_issues > 0 {
            score -= (satd_issues as f64 * 0.1).min(0.5);
        }

        // Penalize very large files
        if lines > 500 {
            score -= 0.1;
        }

        // Penalize files with no functions (might be just comments/imports)
        if function_count == 0 && lines > 10 {
            score -= 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    /// Update aggregate metrics for a project
    fn update_aggregate_metrics(metrics: &mut QualityMetrics) {
        let files_analyzed = metrics.file_metrics.len();
        let functions_analyzed: usize = metrics
            .file_metrics
            .values()
            .map(|f| f.function_count)
            .sum();

        let total_complexity: f64 = metrics
            .file_metrics
            .values()
            .map(|f| f.avg_complexity * f.function_count as f64)
            .sum();

        let avg_complexity = if functions_analyzed > 0 {
            total_complexity / functions_analyzed as f64
        } else {
            0.0
        };

        let max_complexity = metrics
            .file_metrics
            .values()
            .map(|f| f.max_complexity)
            .max()
            .unwrap_or(0);

        let satd_issues: usize = metrics.file_metrics.values().map(|f| f.satd_issues).sum();

        let quality_scores: Vec<f64> = metrics
            .file_metrics
            .values()
            .map(|f| f.quality_score)
            .collect();

        let quality_score = if quality_scores.is_empty() {
            0.0
        } else {
            quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
        };

        // Update complexity distribution
        let mut distribution = ComplexityDistribution {
            low: 0,
            medium: 0,
            high: 0,
            very_high: 0,
            violations: 0,
        };

        for file_metrics in metrics.file_metrics.values() {
            let complexity = file_metrics.max_complexity;
            match complexity {
                0..=5 => distribution.low += 1,
                6..=10 => distribution.medium += 1,
                11..=15 => distribution.high += 1,
                16..=20 => distribution.very_high += 1,
                _ => distribution.violations += 1,
            }
        }

        // Update metrics
        metrics.files_analyzed = files_analyzed;
        metrics.functions_analyzed = functions_analyzed;
        metrics.avg_complexity = avg_complexity;
        metrics.max_complexity = max_complexity;
        metrics.satd_issues = satd_issues;
        metrics.quality_score = quality_score;
        metrics.complexity_distribution = distribution;
        metrics.hotspot_functions = metrics
            .file_metrics
            .values()
            .filter(|f| f.max_complexity > 20)
            .count();
    }

    /// Detect quality changes between old and new metrics
    fn detect_quality_changes(
        old: &FileQualityMetrics,
        new: &FileQualityMetrics,
        file_path: &str,
    ) -> Vec<QualityChange> {
        let mut changes = Vec::new();

        // Check complexity changes
        if (new.avg_complexity - old.avg_complexity).abs() > 0.1 {
            if new.avg_complexity > old.avg_complexity {
                changes.push(QualityChange::ComplexityIncrease {
                    file: file_path.to_string(),
                    old_complexity: old.avg_complexity,
                    new_complexity: new.avg_complexity,
                });
            } else {
                changes.push(QualityChange::ComplexityDecrease {
                    file: file_path.to_string(),
                    old_complexity: old.avg_complexity,
                    new_complexity: new.avg_complexity,
                });
            }
        }

        // Check SATD changes
        match new.satd_issues.cmp(&old.satd_issues) {
            std::cmp::Ordering::Greater => {
                changes.push(QualityChange::SatdAdded {
                    file: file_path.to_string(),
                    count: new.satd_issues - old.satd_issues,
                });
            }
            std::cmp::Ordering::Less => {
                changes.push(QualityChange::SatdRemoved {
                    file: file_path.to_string(),
                    count: old.satd_issues - new.satd_issues,
                });
            }
            std::cmp::Ordering::Equal => {}
        }

        // Check quality score changes
        if (new.quality_score - old.quality_score).abs() > 0.1 {
            if new.quality_score > old.quality_score {
                changes.push(QualityChange::QualityImproved {
                    old_score: old.quality_score,
                    new_score: new.quality_score,
                });
            } else {
                changes.push(QualityChange::QualityDegraded {
                    old_score: old.quality_score,
                    new_score: new.quality_score,
                });
            }
        }

        changes
    }
}
