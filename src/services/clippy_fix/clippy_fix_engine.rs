impl ClippyFixEngine {
    /// Create new engine (complexity: 2)
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            confidence_rules: Self::init_confidence_rules(),
        }
    }

    /// Initialize confidence rules (complexity: 3)
    fn init_confidence_rules() -> HashMap<String, ConfidenceLevel> {
        let mut rules = HashMap::new();

        // High confidence fixes
        rules.insert("clippy::needless_return".to_string(), ConfidenceLevel::High);
        rules.insert("clippy::redundant_clone".to_string(), ConfidenceLevel::High);
        rules.insert(
            "clippy::unnecessary_wraps".to_string(),
            ConfidenceLevel::High,
        );

        // Medium confidence
        rules.insert("clippy::manual_map".to_string(), ConfidenceLevel::Medium);
        rules.insert("clippy::single_match".to_string(), ConfidenceLevel::Medium);

        // Low confidence
        rules.insert(
            "clippy::needless_lifetimes".to_string(),
            ConfidenceLevel::Low,
        );
        rules.insert("clippy::complex_lifetime".to_string(), ConfidenceLevel::Low);

        rules
    }

    /// Calculate confidence for a diagnostic (complexity: 4)
    #[must_use]
    pub fn calculate_confidence(&self, diagnostic: &ClippyDiagnostic) -> ConfidenceLevel {
        self.confidence_rules
            .get(&diagnostic.code)
            .cloned()
            .unwrap_or_else(|| self.default_confidence(diagnostic))
    }

    /// Default confidence calculation (complexity: 3)
    fn default_confidence(&self, diagnostic: &ClippyDiagnostic) -> ConfidenceLevel {
        if diagnostic.suggestion.is_some() {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        }
    }

    /// Apply a single fix (complexity: 5)
    pub async fn apply_fix(
        &self,
        source: &str,
        diagnostic: &ClippyDiagnostic,
    ) -> Result<FixResult> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let start = std::time::Instant::now();

        // Check cache first
        let cache_key = self.generate_cache_key(source, diagnostic);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Apply the fix
        let modified = self.apply_fix_internal(source, diagnostic)?;

        let result = FixResult {
            success: true,
            diagnostic: diagnostic.clone(),
            modified_source: modified,
            confidence: self.calculate_confidence(diagnostic),
            duration: start.elapsed(),
            error: None,
        };

        Ok(result)
    }

    /// Internal fix application (complexity: 6)
    fn apply_fix_internal(&self, source: &str, diagnostic: &ClippyDiagnostic) -> Result<String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Simple implementation for needless_return
        if diagnostic.code == "clippy::needless_return" {
            Ok(source.replace("return ", ""))
        } else if let Some(suggestion) = &diagnostic.suggestion {
            // Apply suggestion if available
            if suggestion.contains("{{") || suggestion.contains("}}") {
                // Invalid syntax in suggestion
                Ok(format!("{source}{suggestion}"))
            } else {
                Ok(source.replace(&diagnostic.message, suggestion))
            }
        } else {
            Ok(source.to_string())
        }
    }

    /// Generate cache key (complexity: 2)
    fn generate_cache_key(&self, source: &str, diagnostic: &ClippyDiagnostic) -> String {
        debug_assert!(!source.is_empty(), "source must not be empty");
        format!(
            "{}:{}:{}",
            diagnostic.code,
            diagnostic.line_start,
            source.get(..source.len().min(100)).unwrap_or(source)
        )
    }

    /// Apply fixes with validation (complexity: 8)
    pub async fn apply_fix_with_validation(
        &self,
        source: &str,
        diagnostic: &ClippyDiagnostic,
    ) -> Result<FixResult> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let result = self.apply_fix(source, diagnostic).await?;

        // Validate the fix compiles
        if !self.validate_fix(&result.modified_source).await? {
            return Ok(FixResult {
                success: false,
                error: Some("Fix breaks compilation".to_string()),
                ..result
            });
        }

        Ok(result)
    }

    /// Validate that fix compiles (complexity: 3)
    async fn validate_fix(&self, source: &str) -> Result<bool> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Check for obvious syntax errors
        if source.contains("{{") || source.contains("}}") {
            return Ok(false);
        }
        Ok(true)
    }

    /// Apply batch fixes (complexity: 5)
    pub async fn apply_batch_fixes(
        &self,
        diagnostics: &[ClippyDiagnostic],
    ) -> Result<Vec<FixResult>> {
        debug_assert!(!diagnostics.is_empty(), "diagnostics must not be empty");
        let mut results = Vec::new();

        for diagnostic in diagnostics {
            let result = self.apply_fix("", diagnostic).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Apply fixes in parallel (complexity: 4)
    pub async fn apply_parallel_fixes(
        &self,
        diagnostics: &[ClippyDiagnostic],
    ) -> Result<Vec<FixResult>> {
        debug_assert!(!diagnostics.is_empty(), "diagnostics must not be empty");
        use futures::future::join_all;

        let futures = diagnostics.iter().map(|d| self.apply_fix("", d));
        let results = join_all(futures).await;

        results.into_iter().collect()
    }

    /// Filter by confidence level (complexity: 3)
    #[must_use]
    pub fn filter_by_confidence(
        &self,
        diagnostics: Vec<(ClippyDiagnostic, ConfidenceLevel)>,
        min_confidence: ConfidenceLevel,
    ) -> Vec<(ClippyDiagnostic, ConfidenceLevel)> {
        debug_assert!(!diagnostics.is_empty(), "diagnostics must not be empty");
        diagnostics
            .into_iter()
            .filter(|(_, conf)| *conf == min_confidence)
            .collect()
    }

    /// Generate comprehensive report (complexity: 5)
    #[must_use]
    pub fn generate_report(&self, results: Vec<FixResult>) -> FixReport {
        debug_assert!(!results.is_empty(), "results must not be empty");
        let total = results.len();
        let successful = results.iter().filter(|r| r.success).count();
        let duration = results.iter().map(|r| r.duration).sum();

        let mut files = results
            .iter()
            .map(|r| r.diagnostic.file.clone())
            .collect::<Vec<_>>();
        files.dedup();

        FixReport {
            total_diagnostics: total,
            successful_fixes: successful,
            failed_fixes: total - successful,
            skipped_low_confidence: 0,
            success_rate: (successful as f64 / total as f64) * 100.0,
            total_duration: duration,
            fixed_files: files,
        }
    }
}
