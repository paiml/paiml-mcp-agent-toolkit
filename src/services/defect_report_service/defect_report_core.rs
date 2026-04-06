// Core orchestration: constructor, report generation, analyzer dispatch, summary computation

impl DefectReportService {
    /// Create a new defect report service
    #[must_use]
    pub fn new() -> Self {
        let cpus = num_cpus::get();
        Self {
            semaphore: Arc::new(Semaphore::new(cpus * 2)),
        }
    }

    /// Generate a comprehensive defect report
    pub async fn generate_report(&self, project_path: &Path) -> Result<DefectReport> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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

    /// Collect defects from all analyzers (with per-analyzer timeout)
    async fn collect_all_defects(&self, project_path: &Path) -> Result<Vec<Defect>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        use std::time::Duration;

        let semaphore = self.semaphore.clone();
        let project_path = project_path.to_path_buf();

        let analyzer_timeout = Duration::from_secs(30);

        // Run all analyzers in parallel with per-analyzer timeouts
        let (complexity, satd, dead_code, duplication, perf, arch) = tokio::join!(
            async {
                match tokio::time::timeout(analyzer_timeout, self.analyze_complexity_defects(&project_path, &semaphore)).await {
                    Ok(result) => result,
                    Err(_) => { warn!("Complexity analyzer timed out after 30s"); Ok(Vec::new()) }
                }
            },
            async {
                match tokio::time::timeout(analyzer_timeout, self.analyze_satd_defects(&project_path, &semaphore)).await {
                    Ok(result) => result,
                    Err(_) => { warn!("SATD analyzer timed out after 30s"); Ok(Vec::new()) }
                }
            },
            async {
                match tokio::time::timeout(analyzer_timeout, self.analyze_dead_code_defects(&project_path, &semaphore)).await {
                    Ok(result) => result,
                    Err(_) => { warn!("Dead code analyzer timed out after 30s"); Ok(Vec::new()) }
                }
            },
            async {
                match tokio::time::timeout(analyzer_timeout, self.analyze_duplication_defects(&project_path, &semaphore)).await {
                    Ok(result) => result,
                    Err(_) => { warn!("Duplication analyzer timed out after 30s"); Ok(Vec::new()) }
                }
            },
            async {
                match tokio::time::timeout(analyzer_timeout, self.analyze_performance_defects(&project_path, &semaphore)).await {
                    Ok(result) => result,
                    Err(_) => { warn!("Performance analyzer timed out after 30s"); Ok(Vec::new()) }
                }
            },
            async {
                match tokio::time::timeout(analyzer_timeout, self.analyze_architecture_defects(&project_path, &semaphore)).await {
                    Ok(result) => result,
                    Err(_) => { warn!("Architecture analyzer timed out after 30s"); Ok(Vec::new()) }
                }
            },
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let _permit = semaphore.acquire().await?;
        debug!("Analyzing architecture defects");

        use crate::services::defect_analyzers::{ArchitectureConfig, ArchitectureDefectAnalyzer};

        let analyzer = ArchitectureDefectAnalyzer::new();
        let config = ArchitectureConfig::default();

        analyzer.analyze(project_path, config).await
    }

    /// Compute summary statistics
    #[must_use]
    pub fn compute_summary(&self, defects: &[Defect]) -> DefectSummary {
        debug_assert!(!defects.is_empty(), "defects must not be empty");
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
}

impl Default for DefectReportService {
    fn default() -> Self {
        Self::new()
    }
}
