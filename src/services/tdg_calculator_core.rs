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
    pub(crate) config: TDGConfig,
    /// Cache for TDG scores
    pub(crate) cache: Arc<DashMap<PathBuf, TDGScore>>,
    pub(crate) semaphore: Arc<Semaphore>,
    /// Lightweight provability analyzer
    pub(crate) provability_analyzer: Arc<LightweightProvabilityAnalyzer>,
    /// AST engine for parsing
    pub(crate) ast_engine: Arc<UnifiedAstEngine>,
    /// Project root for git analysis
    pub(crate) project_root: PathBuf,
    /// Cached churn analysis for the entire project
    pub(crate) cached_churn_analysis: Arc<Mutex<Option<crate::models::churn::CodeChurnAnalysis>>>,
}

impl TDGCalculator {
    /// Creates a new `TDGCalculator` with default configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::tdg_calculator::TDGCalculator;
    ///
    /// let calculator = TDGCalculator::new();
    /// // Calculator ready to compute Technical Debt Gradient scores
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(TDGConfig::default())
    }

    #[must_use]
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

    /// Set the project root for git-based analysis (churn, etc.)
    #[must_use]
    pub fn with_project_root(mut self, root: PathBuf) -> Self {
        debug_assert!(root.exists(), "root must exist: {}", root.display());
        self.project_root = root;
        self
    }

    /// Calculate TDG score for a single file
    pub async fn calculate_file(&self, path: &Path) -> Result<TDGScore> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
        // CB-128: dead_code component added but set to 0.0 for now
        // Full integration requires calling CargoDeadCodeAnalyzer
        let components = TDGComponents {
            complexity,
            churn,
            coupling,
            domain_risk,
            duplication,
            dead_code: 0.0, // TODO(CB-128): Integrate with CargoDeadCodeAnalyzer
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
        debug_assert!(!files.is_empty(), "files must not be empty");
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
        values.sort_by(|a, b| a.total_cmp(b));

        for score in scores.iter_mut() {
            let position = values
                .binary_search_by(|&v| v.total_cmp(&score.value))
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
            cached_churn_analysis: Arc::clone(&self.cached_churn_analysis),
        }
    }
}
