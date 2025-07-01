use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::{Mutex, Semaphore};

use crate::models::tdg::{
    RecommendationType, TDGAnalysis, TDGBucket, TDGComponents, TDGConfig, TDGDistribution,
    TDGHotspot, TDGRecommendation, TDGScore, TDGSeverity, TDGSummary,
};
use crate::models::unified_ast::{AstKind, UnifiedAstNode};
use crate::services::file_discovery::ProjectFileDiscovery;
use crate::services::git_analysis::GitAnalysisService;
use crate::services::lightweight_provability_analyzer::{
    FunctionId, LightweightProvabilityAnalyzer,
};
use crate::services::unified_ast_engine::UnifiedAstEngine;
use crate::services::verified_complexity::VerifiedComplexityAnalyzer;

/// Complexity variance metrics for TDG calculation
#[derive(Debug, Clone)]
pub struct ComplexityVariance {
    pub mean: f64,
    pub variance: f64,
    pub gini: f64,
    pub percentile_90: f64,
}

/// Coupling metrics for files
#[derive(Debug, Clone)]
pub struct CouplingMetrics {
    pub afferent: usize,  // Incoming dependencies
    pub efferent: usize,  // Outgoing dependencies
    pub instability: f64, // efferent / (afferent + efferent)
}

/// TDG (Code Quality Gradient) Calculator
/// Primary service for calculating TDG scores to replace defect probability
pub struct TDGCalculator {
    config: TDGConfig,
    /// Cache for TDG scores
    cache: Arc<DashMap<PathBuf, TDGScore>>,
    semaphore: Arc<Semaphore>,
    /// Lightweight provability analyzer
    provability_analyzer: Arc<LightweightProvabilityAnalyzer>,
    /// AST engine for parsing
    ast_engine: Arc<UnifiedAstEngine>,
    /// Project root for git analysis
    project_root: PathBuf,
    /// Cached churn analysis for the entire project
    cached_churn_analysis: Arc<Mutex<Option<crate::models::churn::CodeChurnAnalysis>>>,
}

impl TDGCalculator {
    pub fn new() -> Self {
        Self::with_config(TDGConfig::default())
    }

    pub fn with_config(config: TDGConfig) -> Self {
        Self {
            config,
            cache: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(num_cpus::get() * 2)),
            provability_analyzer: Arc::new(LightweightProvabilityAnalyzer::new()),
            ast_engine: Arc::new(UnifiedAstEngine::new()),
            project_root: PathBuf::from("."),
            cached_churn_analysis: Arc::new(Mutex::new(None)),
        }
    }

    /// Calculate TDG score for a single file
    pub async fn calculate_file(&self, path: &Path) -> Result<TDGScore> {
        // Check cache first
        if let Some(cached) = self.cache.get(&path.to_path_buf()) {
            return Ok(cached.clone());
        }

        let score = self.calculate_file_uncached(path).await?;
        self.cache.insert(path.to_path_buf(), score.clone());
        Ok(score)
    }

    /// Calculate TDG score without caching
    async fn calculate_file_uncached(&self, path: &Path) -> Result<TDGScore> {
        // Gather all metrics in parallel
        let (complexity, churn, coupling, duplication, provability) = tokio::try_join!(
            self.calculate_complexity_factor(path),
            self.calculate_churn_factor(path),
            self.calculate_coupling_factor(path),
            self.calculate_duplication_factor(path),
            self.calculate_provability_factor(path),
        )?;

        let domain_risk = self.calculate_domain_risk(path).await?;

        // Calculate weighted TDG value
        let components = TDGComponents {
            complexity,
            churn,
            coupling,
            domain_risk,
            duplication,
        };

        let value = self.calculate_weighted_tdg(&components, provability);
        let severity = TDGSeverity::from(value);

        Ok(TDGScore {
            value,
            components,
            severity,
            percentile: 0.0, // Will be calculated in batch analysis
            confidence: self.calculate_confidence(&components),
        })
    }

    /// Calculate TDG scores for multiple files with parallelization
    pub async fn calculate_batch(&self, files: Vec<PathBuf>) -> Result<Vec<TDGScore>> {
        let tasks: Vec<_> = files
            .into_iter()
            .map(|file| {
                let calculator = self.clone();
                tokio::spawn(async move {
                    let _permit = calculator.semaphore.acquire().await?;
                    calculator.calculate_file(&file).await
                })
            })
            .collect();

        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            results.push(task.await??);
        }

        // Calculate percentiles
        self.calculate_percentiles(&mut results);

        Ok(results)
    }

    /// Analyze directory and generate TDG summary
    pub async fn analyze_directory(&self, path: &Path) -> Result<TDGSummary> {
        let files = self.discover_files(path).await?;
        let scores = self.calculate_batch(files.clone()).await?;

        let mut critical_files = 0;
        let mut warning_files = 0;
        let mut tdg_values: Vec<f64> = Vec::with_capacity(scores.len());

        for score in &scores {
            tdg_values.push(score.value);
            match score.severity {
                TDGSeverity::Critical => critical_files += 1,
                TDGSeverity::Warning => warning_files += 1,
                TDGSeverity::Normal => {}
            }
        }

        // Sort for percentile calculation
        tdg_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let average_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
        };

        let p95_tdg = self.percentile(&tdg_values, 0.95);
        let p99_tdg = self.percentile(&tdg_values, 0.99);

        // Find hotspots
        let mut indexed_scores: Vec<(usize, &TDGScore, PathBuf)> = scores
            .iter()
            .enumerate()
            .zip(files.iter())
            .map(|((idx, score), path)| (idx, score, path.clone()))
            .collect();

        indexed_scores.sort_by(|a, b| b.1.value.partial_cmp(&a.1.value).unwrap());

        let hotspots = indexed_scores
            .iter()
            .take(10)
            .map(|(_, score, path)| {
                let primary_factor = self.identify_primary_factor(&score.components);
                TDGHotspot {
                    path: path.display().to_string(),
                    tdg_score: score.value,
                    primary_factor,
                    estimated_hours: self.estimate_refactoring_hours(score.value),
                }
            })
            .collect();

        let estimated_debt_hours = scores
            .iter()
            .map(|s| self.estimate_refactoring_hours(s.value))
            .sum();

        Ok(TDGSummary {
            total_files: scores.len(),
            critical_files,
            warning_files,
            average_tdg,
            p95_tdg,
            p99_tdg,
            estimated_debt_hours,
            hotspots,
        })
    }

    /// Generate detailed TDG analysis with recommendations
    pub async fn analyze_path(&self, path: &Path) -> Result<TDGAnalysis> {
        let score = self.calculate_file(path).await?;
        let explanation = self.generate_explanation(&score);
        let recommendations = self.generate_recommendations(&score, path).await?;

        Ok(TDGAnalysis {
            score,
            explanation,
            recommendations,
        })
    }

    /// Compute complexity gradient with variance analysis
    #[allow(dead_code)]
    fn compute_complexity_gradient(&self, ast: &UnifiedAstNode) -> ComplexityVariance {
        let mut analyzer = VerifiedComplexityAnalyzer::new();
        // For now, analyze the node itself if it's a function
        let complexities: Vec<u32> = if matches!(ast.kind, AstKind::Function(_)) {
            vec![analyzer.analyze_function(ast).cyclomatic]
        } else {
            // In real implementation, would traverse children via first_child/next_sibling
            vec![]
        };

        if complexities.is_empty() {
            return ComplexityVariance {
                mean: 0.0,
                variance: 0.0,
                gini: 0.0,
                percentile_90: 0.0,
            };
        }

        // Calculate mean
        let sum: u32 = complexities.iter().sum();
        let mean = sum as f64 / complexities.len() as f64;

        // Calculate variance
        let squared_diff_sum: f64 = complexities
            .iter()
            .map(|&c| (c as f64 - mean).powi(2))
            .sum();
        let variance = squared_diff_sum / complexities.len() as f64;

        // Calculate Gini coefficient
        let mut sorted = complexities.clone();
        sorted.sort_unstable();

        let mut gini_sum = 0.0;
        for (i, &value) in sorted.iter().enumerate() {
            gini_sum += (2.0 * (i + 1) as f64 - sorted.len() as f64 - 1.0) * value as f64;
        }
        let gini = gini_sum / (sorted.len() as f64 * sum as f64);

        // Calculate 90th percentile
        let percentile_idx = ((sorted.len() as f64 * 0.9) as usize).min(sorted.len() - 1);
        let percentile_90 = sorted[percentile_idx] as f64;

        ComplexityVariance {
            mean,
            variance,
            gini,
            percentile_90,
        }
    }

    /// Calculate complexity factor (normalized 0-5)
    async fn calculate_complexity_factor(&self, path: &Path) -> Result<f64> {
        // For now, use heuristic analysis until we have proper AST integration
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content.lines().collect();

        // Basic complexity heuristics
        let mut complexity = 0usize;
        let mut nesting_level = 0usize;
        let mut function_complexities = Vec::<usize>::new();

        for line in &lines {
            let trimmed = line.trim();

            // Track function boundaries for variance
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("func ")
            {
                if complexity > 0 {
                    function_complexities.push(complexity);
                }
                complexity = 1; // Reset for new function
                nesting_level = 0;
            }

            // Count control flow statements
            if trimmed.starts_with("if ")
                || trimmed.starts_with("elif ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("match ")
                || trimmed.starts_with("case ")
            {
                complexity += 1 + nesting_level;
            }

            // Track nesting
            nesting_level += trimmed.matches('{').count();
            nesting_level = nesting_level.saturating_sub(trimmed.matches('}').count());
        }

        // Add last function
        if complexity > 0 {
            function_complexities.push(complexity);
        }

        // Calculate variance to ensure different TDG values
        if function_complexities.is_empty() {
            return Ok(0.5); // Low complexity for files without functions
        }

        let mean =
            function_complexities.iter().sum::<usize>() as f64 / function_complexities.len() as f64;
        let variance = function_complexities
            .iter()
            .map(|&c| (c as f64 - mean).powi(2))
            .sum::<f64>()
            / function_complexities.len() as f64;

        // Multi-factor score with variance
        let base_complexity = mean / 5.0; // More sensitive to complexity
        let variance_factor = variance.sqrt() / 3.0; // Higher weight for variance
        let max_complexity = function_complexities.iter().max().copied().unwrap_or(0) as f64;
        let hotspot_factor = (max_complexity / 10.0).min(1.0);

        // Add file length factor
        let loc_factor = (lines.len() as f64 / 100.0).min(1.0);

        let score =
            base_complexity * 0.4 + variance_factor * 0.2 + hotspot_factor * 0.2 + loc_factor * 0.2;
        Ok(score.min(5.0))
    }

    /// Calculate churn factor based on git history
    async fn calculate_churn_factor(&self, path: &Path) -> Result<f64> {
        // Get cached churn analysis or compute it once
        let analysis = self.get_or_compute_churn_analysis().await?;

        // Find this file in the analysis
        let relative_path = path.strip_prefix(&self.project_root).unwrap_or(path);

        if let Some(file_metrics) = analysis
            .files
            .iter()
            .find(|f| f.path == relative_path || f.relative_path == relative_path.to_string_lossy())
        {
            let monthly_rate = file_metrics.commit_count as f64 / 3.0; // 90 days = 3 months

            // Apply logarithmic normalization
            // log(1 + monthly_rate) scales nicely for typical rates
            let normalized = (1.0 + monthly_rate).ln() / 2.0;

            Ok(normalized.min(5.0))
        } else {
            // File not found in git history
            self.calculate_churn_fallback(path).await
        }
    }

    /// Get cached churn analysis or compute it once for the entire project
    pub async fn get_or_compute_churn_analysis(
        &self,
    ) -> Result<crate::models::churn::CodeChurnAnalysis> {
        let mut cache = self.cached_churn_analysis.lock().await;

        if let Some(ref analysis) = *cache {
            return Ok(analysis.clone());
        }

        // Compute churn analysis once for the entire project
        tracing::info!("Computing churn analysis for project (this should only happen once)...");
        match GitAnalysisService::analyze_code_churn(&self.project_root, 90) {
            Ok(analysis) => {
                tracing::info!("Churn analysis computed successfully and cached");
                *cache = Some(analysis.clone());
                Ok(analysis)
            }
            Err(e) => {
                // Create empty analysis on error to avoid repeated git failures
                let empty_analysis = crate::models::churn::CodeChurnAnalysis {
                    generated_at: chrono::Utc::now(),
                    period_days: 90,
                    repository_root: self.project_root.clone(),
                    files: vec![],
                    summary: crate::models::churn::ChurnSummary {
                        total_commits: 0,
                        total_files_changed: 0,
                        hotspot_files: vec![],
                        stable_files: vec![],
                        author_contributions: HashMap::new(),
                    },
                };
                *cache = Some(empty_analysis.clone());
                Err(e.into())
            }
        }
    }

    /// Fallback churn calculation based on file modification time
    async fn calculate_churn_fallback(&self, path: &Path) -> Result<f64> {
        match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let days_old = elapsed.as_secs() / 86400;
                        // Files modified recently have higher churn factor
                        if days_old < 7 {
                            Ok(3.0)
                        } else if days_old < 30 {
                            Ok(2.0)
                        } else if days_old < 90 {
                            Ok(1.0)
                        } else {
                            Ok(0.5)
                        }
                    } else {
                        Ok(1.0)
                    }
                } else {
                    Ok(1.0)
                }
            }
            Err(_) => Ok(1.0),
        }
    }

    /// Analyze coupling metrics for a file
    #[allow(dead_code)]
    fn analyze_coupling(&self, _file: &Path, ast: &UnifiedAstNode) -> CouplingMetrics {
        // Extract imports/exports from AST
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        self.extract_dependencies(ast, &mut imports, &mut exports);

        // For now, simplified coupling calculation
        // In full implementation, would track actual dependency graph
        let efferent = imports.len();
        let afferent = exports.len(); // Simplified - would need project-wide analysis

        let instability = if afferent + efferent == 0 {
            0.0
        } else {
            efferent as f64 / (afferent + efferent) as f64
        };

        CouplingMetrics {
            afferent,
            efferent,
            instability,
        }
    }

    /// Extract imports and exports from AST
    #[allow(dead_code)]
    fn extract_dependencies(
        &self,
        node: &UnifiedAstNode,
        imports: &mut Vec<String>,
        exports: &mut Vec<String>,
    ) {
        match &node.kind {
            AstKind::Import(_) => {
                // Simplified - would extract actual import paths
                imports.push("import".to_string());
            }
            AstKind::Function(_) => {
                // Simplified - would check export status
                exports.push("function".to_string());
            }
            AstKind::Class(_) => {
                // Simplified - would check export status
                exports.push("class".to_string());
            }
            _ => {}
        }

        // In real implementation would traverse children via first_child/next_sibling
    }

    /// Calculate coupling factor
    async fn calculate_coupling_factor(&self, path: &Path) -> Result<f64> {
        // Use import counting heuristic
        let content = tokio::fs::read_to_string(path).await?;
        let import_count = self.count_imports(&content);

        // Also count exported items for better coupling analysis
        let export_count = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("pub fn")
                    || trimmed.starts_with("pub struct")
                    || trimmed.starts_with("pub enum")
                    || trimmed.starts_with("export ")
                    || trimmed.contains("module.exports")
            })
            .count();

        // Calculate instability metric
        let total = import_count + export_count;
        let instability = if total == 0 {
            0.0
        } else {
            import_count as f64 / total as f64
        };

        // Multi-factor coupling score
        let import_factor = (import_count as f64 / 15.0).min(2.0);
        let instability_factor = instability * 2.0;
        let complexity_penalty = if import_count > 20 { 1.0 } else { 0.0 };

        let score = import_factor + instability_factor + complexity_penalty;
        Ok(score.min(5.0))
    }

    /// Calculate code duplication factor
    async fn calculate_duplication_factor(&self, path: &Path) -> Result<f64> {
        // Simplified duplication detection
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#"))
            .collect();

        if lines.len() < 10 {
            return Ok(0.0);
        }

        // Count duplicate lines (very basic)
        let mut line_counts = HashMap::new();
        for line in &lines {
            if line.len() > 10 {
                // Only count substantial lines
                *line_counts.entry(*line).or_insert(0) += 1;
            }
        }

        let duplicate_lines: usize = line_counts
            .values()
            .filter(|&&count| count > 1)
            .map(|&count| count - 1)
            .sum();

        let duplication_percentage = (duplicate_lines as f64 / lines.len() as f64) * 100.0;

        // Normalize: >30% duplication is critical
        Ok((duplication_percentage / 30.0).min(5.0))
    }

    /// Calculate domain-specific risk factors
    async fn calculate_domain_risk(&self, path: &Path) -> Result<f64> {
        let mut risk: f64 = 0.0;

        // High-risk domain paths
        if path.to_string_lossy().contains("auth")
            || path.to_string_lossy().contains("crypto")
            || path.to_string_lossy().contains("security")
        {
            risk += 2.0;
        }

        // Database/persistence layer
        if path.to_string_lossy().contains("database")
            || path.to_string_lossy().contains("migration")
        {
            risk += 1.5;
        }

        // External API integrations
        if path.to_string_lossy().contains("api") || path.to_string_lossy().contains("integration")
        {
            risk += 1.0;
        }

        Ok(risk.min(5.0))
    }

    /// Calculate weighted TDG value from components
    fn calculate_weighted_tdg(&self, components: &TDGComponents, provability_factor: f64) -> f64 {
        let base_weighted = components.complexity * self.config.complexity_weight
            + components.churn * self.config.churn_weight
            + components.coupling * self.config.coupling_weight
            + components.domain_risk * self.config.domain_risk_weight
            + components.duplication * self.config.duplication_weight;

        // Apply provability factor (higher provability reduces TDG)
        let adjusted = base_weighted * (1.0 - provability_factor * 0.2);

        // Ensure result is in 0-5 range
        adjusted.clamp(0.0, 5.0)
    }

    /// Calculate confidence level based on data availability
    fn calculate_confidence(&self, components: &TDGComponents) -> f64 {
        let mut confidence = 1.0;

        // Reduce confidence for zero values (likely missing data)
        if components.churn == 0.0 {
            confidence *= 0.8;
        }
        if components.coupling == 0.0 {
            confidence *= 0.9;
        }
        if components.duplication == 0.0 {
            confidence *= 0.95;
        }

        confidence
    }

    /// Calculate percentiles for a batch of scores
    fn calculate_percentiles(&self, scores: &mut [TDGScore]) {
        let mut values: Vec<f64> = scores.iter().map(|s| s.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for score in scores.iter_mut() {
            let position = values
                .binary_search_by(|&v| v.partial_cmp(&score.value).unwrap())
                .unwrap_or_else(|i| i);
            score.percentile = (position as f64 / values.len() as f64) * 100.0;
        }
    }

    /// Calculate specific percentile value
    fn percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (sorted_values.len() as f64 * percentile) as usize;
        let index = index.min(sorted_values.len() - 1);
        sorted_values[index]
    }

    /// Identify primary contributing factor
    fn identify_primary_factor(&self, components: &TDGComponents) -> String {
        let mut factors = [
            (
                components.complexity * self.config.complexity_weight,
                "High Complexity",
            ),
            (
                components.churn * self.config.churn_weight,
                "Frequent Changes",
            ),
            (
                components.coupling * self.config.coupling_weight,
                "High Coupling",
            ),
            (
                components.domain_risk * self.config.domain_risk_weight,
                "Domain Risk",
            ),
            (
                components.duplication * self.config.duplication_weight,
                "Code Duplication",
            ),
        ];

        factors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        factors[0].1.to_string()
    }

    /// Estimate refactoring hours based on TDG score
    fn estimate_refactoring_hours(&self, tdg_score: f64) -> f64 {
        // Empirical formula: hours = base * multiplier^tdg
        let base_hours: f64 = 2.0;
        let multiplier: f64 = 1.8;

        base_hours * multiplier.powf(tdg_score)
    }

    /// Generate human-readable explanation
    fn generate_explanation(&self, score: &TDGScore) -> String {
        let mut explanation = format!(
            "Code Quality Gradient: {:.2} ({})\n\n",
            score.value,
            score.severity.as_str()
        );

        explanation.push_str("Component Breakdown:\n");

        let components = [
            (
                score.components.complexity,
                "Complexity",
                self.config.complexity_weight,
            ),
            (
                score.components.churn,
                "Code Churn",
                self.config.churn_weight,
            ),
            (
                score.components.coupling,
                "Coupling",
                self.config.coupling_weight,
            ),
            (
                score.components.domain_risk,
                "Domain Risk",
                self.config.domain_risk_weight,
            ),
            (
                score.components.duplication,
                "Duplication",
                self.config.duplication_weight,
            ),
        ];

        for (value, name, weight) in components {
            let contribution = value * weight;
            explanation.push_str(&format!(
                "- {name}: {value:.2} (contributes {contribution:.2} to total)\n"
            ));
        }

        explanation.push_str(&format!("\nConfidence: {:.0}%", score.confidence * 100.0));

        explanation
    }

    /// Generate actionable recommendations
    async fn generate_recommendations(
        &self,
        score: &TDGScore,
        _path: &Path,
    ) -> Result<Vec<TDGRecommendation>> {
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if score.components.complexity > 3.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::ReduceComplexity,
                action: "Extract complex logic into smaller, focused functions".to_string(),
                expected_reduction: score.components.complexity
                    * 0.3
                    * self.config.complexity_weight,
                estimated_hours: 4.0,
                priority: 5,
            });
        }

        // Churn recommendations
        if score.components.churn > 3.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::StabilizeChurn,
                action: "Add comprehensive tests to stabilize frequently changing code".to_string(),
                expected_reduction: score.components.churn * 0.4 * self.config.churn_weight,
                estimated_hours: 8.0,
                priority: 4,
            });
        }

        // Coupling recommendations
        if score.components.coupling > 3.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::ReduceCoupling,
                action: "Introduce abstractions to reduce direct dependencies".to_string(),
                expected_reduction: score.components.coupling * 0.35 * self.config.coupling_weight,
                estimated_hours: 6.0,
                priority: 3,
            });
        }

        // Duplication recommendations
        if score.components.duplication > 2.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::RemoveDuplication,
                action: "Extract duplicated code into shared utilities".to_string(),
                expected_reduction: score.components.duplication
                    * 0.5
                    * self.config.duplication_weight,
                estimated_hours: 3.0,
                priority: 2,
            });
        }

        // Sort by priority
        recommendations.sort_by_key(|r| std::cmp::Reverse(r.priority));

        Ok(recommendations)
    }

    /// Discover files for analysis
    async fn discover_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        // Use the proper file discovery service with external dependency filtering
        let discovery = ProjectFileDiscovery::new(path.to_path_buf());
        discovery.discover_files()
    }

    fn count_imports(&self, content: &str) -> usize {
        let patterns = [
            r"^use\s+",             // Rust
            r"^import\s+",          // Python/JS/TS
            r"^from\s+.*\s+import", // Python
            r"^require\(",          // Node.js
        ];

        content
            .lines()
            .filter(|line| {
                patterns
                    .iter()
                    .any(|p| regex::Regex::new(p).unwrap().is_match(line.trim()))
            })
            .count()
    }

    /// Calculate provability factor using lightweight analysis
    async fn calculate_provability_factor(&self, path: &Path) -> Result<f64> {
        // Extract function information from file
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let func_id = FunctionId {
            file_path: path.to_string_lossy().to_string(),
            function_name: file_name.to_string(),
            line_number: 1,
        };

        let summaries = self
            .provability_analyzer
            .analyze_incrementally(&[func_id])
            .await;

        if let Some(summary) = summaries.first() {
            Ok(summary.provability_score)
        } else {
            Ok(0.0) // Default to no provability
        }
    }

    /// Generate TDG distribution for visualization
    pub fn calculate_distribution(&self, scores: &[TDGScore]) -> TDGDistribution {
        let bucket_size = 0.5;
        let max_value = 5.0;
        let num_buckets = (max_value / bucket_size) as usize;

        let mut buckets = Vec::with_capacity(num_buckets);

        for i in 0..num_buckets {
            let min = i as f64 * bucket_size;
            let max = (i + 1) as f64 * bucket_size;

            let count = scores
                .iter()
                .filter(|s| s.value >= min && s.value < max)
                .count();

            let percentage = if scores.is_empty() {
                0.0
            } else {
                (count as f64 / scores.len() as f64) * 100.0
            };

            buckets.push(TDGBucket {
                min,
                max,
                count,
                percentage,
            });
        }

        TDGDistribution {
            buckets,
            total_files: scores.len(),
        }
    }
}

impl Default for TDGCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TDGCalculator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache: self.cache.clone(),
            semaphore: self.semaphore.clone(),
            provability_analyzer: self.provability_analyzer.clone(),
            ast_engine: self.ast_engine.clone(),
            project_root: self.project_root.clone(),
            cached_churn_analysis: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for TDGComponents {
    fn default() -> Self {
        Self {
            complexity: 0.0,
            churn: 0.0,
            coupling: 0.0,
            domain_risk: 0.0,
            duplication: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_tdg_calculation() {
        let calculator = TDGCalculator::new();
        let temp_dir = TempDir::new().unwrap();

        let test_file = temp_dir.path().join("test.rs");
        tokio::fs::write(
            &test_file,
            r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        if x > 20 {
                            x * 2
                        } else {
                            x + 10
                        }
                    } else {
                        x + 5
                    }
                } else {
                    0
                }
            }
        "#,
        )
        .await
        .unwrap();

        // Try to calculate the score - it may fail if no git repo
        let score_result = calculator.calculate_file(&test_file).await;

        // If it fails due to no git repo, that's expected in test environment
        if let Err(e) = score_result {
            if e.to_string().contains("git repository") {
                // Skip test in non-git environment
                return;
            }
            panic!("Unexpected error: {}", e);
        }

        let score = score_result.unwrap();

        assert!(score.value > 0.0);
        assert!(score.value <= 5.0);
        assert!(score.components.complexity > 0.0);
    }

    #[test]
    fn test_tdg_distribution() {
        let calculator = TDGCalculator::new();

        let scores = vec![
            TDGScore {
                value: 0.5,
                components: TDGComponents::default(),
                severity: TDGSeverity::Normal,
                percentile: 0.0,
                confidence: 1.0,
            },
            TDGScore {
                value: 1.8,
                components: TDGComponents::default(),
                severity: TDGSeverity::Warning,
                percentile: 0.0,
                confidence: 1.0,
            },
            TDGScore {
                value: 3.2,
                components: TDGComponents::default(),
                severity: TDGSeverity::Critical,
                percentile: 0.0,
                confidence: 1.0,
            },
        ];

        let distribution = calculator.calculate_distribution(&scores);

        assert_eq!(distribution.total_files, 3);
        assert!(!distribution.buckets.is_empty());

        let total_percentage: f64 = distribution.buckets.iter().map(|b| b.percentage).sum();
        assert!((total_percentage - 100.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_tdg_variance() {
        let calculator = TDGCalculator::new();
        let temp_dir = TempDir::new().unwrap();

        // Create files with different complexity levels
        let simple_file = temp_dir.path().join("simple.rs");
        tokio::fs::write(
            &simple_file,
            r#"
            fn simple() -> i32 {
                42
            }
            "#,
        )
        .await
        .unwrap();

        let complex_file = temp_dir.path().join("complex.rs");
        tokio::fs::write(
            &complex_file,
            r#"
            fn complex(items: &[i32]) -> i32 {
                let mut result = 0;
                for item in items {
                    if *item > 0 {
                        if *item % 2 == 0 {
                            result += item;
                        } else {
                            result -= item;
                        }
                    } else if *item < -10 {
                        for i in 0..*item.abs() {
                            result *= 2;
                        }
                    }
                }
                result
            }
            "#,
        )
        .await
        .unwrap();

        let medium_file = temp_dir.path().join("medium.rs");
        tokio::fs::write(
            &medium_file,
            r#"
            fn medium(x: i32, y: i32) -> i32 {
                if x > y {
                    x - y
                } else {
                    y - x
                }
            }
            "#,
        )
        .await
        .unwrap();

        // Calculate TDG for each file
        let simple_result = calculator.calculate_file(&simple_file).await;
        let complex_result = calculator.calculate_file(&complex_file).await;
        let medium_result = calculator.calculate_file(&medium_file).await;

        // If any fail due to no git repo, that's expected in test environment
        if let Err(e) = &simple_result {
            if e.to_string().contains("git repository") {
                return;
            }
        }

        let simple_tdg = simple_result.unwrap();
        let complex_tdg = complex_result.unwrap();
        let medium_tdg = medium_result.unwrap();

        // Verify variance - values should be different
        assert_ne!(
            simple_tdg.value, complex_tdg.value,
            "Simple and complex files should have different TDG values"
        );
        assert_ne!(
            simple_tdg.value, medium_tdg.value,
            "Simple and medium files should have different TDG values"
        );
        assert_ne!(
            complex_tdg.value, medium_tdg.value,
            "Complex and medium files should have different TDG values"
        );

        // Verify ordering - complex should be highest
        assert!(
            complex_tdg.value > medium_tdg.value,
            "Complex file should have higher TDG than medium"
        );
        assert!(
            medium_tdg.value > simple_tdg.value,
            "Medium file should have higher TDG than simple"
        );

        // Calculate variance
        let values = [simple_tdg.value, complex_tdg.value, medium_tdg.value];
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        println!(
            "TDG values: simple={:.3}, medium={:.3}, complex={:.3}",
            simple_tdg.value, medium_tdg.value, complex_tdg.value
        );
        println!("Variance: {variance:.3}");
        // With multi-factor TDG, variance will be lower but should still be non-zero
        assert!(
            variance > 0.01,
            "TDG variance {variance:.3} too low - values too similar"
        );
    }
}
