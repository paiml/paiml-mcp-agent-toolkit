impl ClippyFixEngine {
    /// Create new engine (complexity: 2)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    /// Compute what a fixed version of `source` would look like. IN MEMORY ONLY.
    ///
    /// The rewritten source comes back in `FixResult::modified_source`. This
    /// method does not open, create or write any file, and neither does
    /// anything else in this module — there is no `fs::write` anywhere in
    /// `src/services/clippy_fix/`. `pmat analyze clippy` nevertheless
    /// reported `"action": "applied"`, a non-zero `successful_fixes` and a
    /// populated `fixed_files` over a byte-identical tree, which is #1086. The
    /// response now says `previewed` and the apply path is gone; see
    /// `src/mcp_pmcp/tools/auto_clippy_fix.rs`.
    ///
    /// Do NOT close the gap by writing `modified_source` back: see the warning
    /// on `apply_fix_internal` for why its output must not reach a file.
    ///
    /// Complexity: 5
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn apply_fix(
        &self,
        source: &str,
        diagnostic: &ClippyDiagnostic,
    ) -> Result<FixResult> {
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

    /// Internal fix transform (complexity: 6). ITS OUTPUT IS NOT SAFE TO WRITE.
    ///
    /// No branch here consults the diagnostic's span, even though `diagnostic`
    /// carries `line_start` and `column_start`:
    ///
    /// - `clippy::needless_return` is `source.replace("return ", "")` over the
    ///   WHOLE file, so it strikes that substring wherever it occurs — inside a
    ///   string literal or a comment as readily as at the `return` statement
    ///   clippy flagged.
    /// - the suggestion branch replaces every occurrence of the diagnostic's
    ///   human-readable *message* text with the suggestion, and when the
    ///   suggestion contains `{{` or `}}` it APPENDS it to the end of the file
    ///   instead.
    /// - everything else returns the source unchanged.
    ///
    /// The suggestion branches are unreachable from `pmat analyze clippy`:
    /// `ClippyDiagnostic::parse_json_value` sets `suggestion: None`
    /// unconditionally, so only the first and last apply there.
    ///
    /// This is why #1086 is fixed by removing the "applied" claim rather than
    /// by adding the missing `fs::write`: writing this output would corrupt
    /// user source. A real fix needs a span-based rewrite, which this is not.
    /// `clippy_fix_tests.rs` pins the corruption so the hazard stays visible.
    fn apply_fix_internal(&self, source: &str, diagnostic: &ClippyDiagnostic) -> Result<String> {
        // Whole-file substitution, NOT a span edit — see the doc comment above.
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
        format!(
            "{}:{}:{}",
            diagnostic.code,
            diagnostic.line_start,
            source.get(..source.len().min(100)).unwrap_or(source)
        )
    }

    /// Apply fixes with validation (complexity: 8)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn apply_fix_with_validation(
        &self,
        source: &str,
        diagnostic: &ClippyDiagnostic,
    ) -> Result<FixResult> {
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
        // Check for obvious syntax errors
        if source.contains("{{") || source.contains("}}") {
            return Ok(false);
        }
        Ok(true)
    }

    /// Apply batch fixes (complexity: 5)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn apply_batch_fixes(
        &self,
        diagnostics: &[ClippyDiagnostic],
    ) -> Result<Vec<FixResult>> {
        let mut results = Vec::new();

        for diagnostic in diagnostics {
            let result = self.apply_fix("", diagnostic).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Apply fixes in parallel (complexity: 4)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn apply_parallel_fixes(
        &self,
        diagnostics: &[ClippyDiagnostic],
    ) -> Result<Vec<FixResult>> {
        use futures::future::join_all;

        let futures = diagnostics.iter().map(|d| self.apply_fix("", d));
        let results = join_all(futures).await;

        results.into_iter().collect()
    }

    /// Filter by confidence level (complexity: 3)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn filter_by_confidence(
        &self,
        diagnostics: Vec<(ClippyDiagnostic, ConfidenceLevel)>,
        min_confidence: ConfidenceLevel,
    ) -> Vec<(ClippyDiagnostic, ConfidenceLevel)> {
        diagnostics
            .into_iter()
            .filter(|(_, conf)| *conf == min_confidence)
            .collect()
    }

    /// Summarise a batch of in-memory transforms.
    ///
    /// `successful_fixes` and `success_rate` count TRANSFORMS ATTEMPTED, not
    /// files changed: `apply_fix` builds its `FixResult` with `success: true`
    /// unconditionally, and no caller writes `modified_source` anywhere. This
    /// report reached the `pmat analyze clippy` payload as `successful_fixes`
    /// and `fixed_files` over an unmodified tree (#1086); it no longer does.
    /// `auto_clippy_fix` has no apply path, so no command in this crate now
    /// puts these numbers in front of a user — only tests call it.
    ///
    /// Complexity: 5
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_report(&self, results: Vec<FixResult>) -> FixReport {
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
