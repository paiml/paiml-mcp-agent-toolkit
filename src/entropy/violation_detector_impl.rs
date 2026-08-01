/// Detects violations from patterns
pub struct ViolationDetector {
    config: EntropyConfig,
}

impl ViolationDetector {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Detect actionable violations from patterns
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn detect_violations(
        &self,
        patterns: &PatternCollection,
        metrics: &EntropyMetrics,
    ) -> Result<Vec<ActionableViolation>> {
        let mut violations = Vec::new();

        // Check for repetitive patterns
        self.detect_repetitive_patterns(patterns, &mut violations)?;

        // Check for low diversity
        self.detect_low_diversity(patterns, metrics, &mut violations)?;

        // Check for cross-file duplication
        self.detect_cross_file_duplication(patterns, &mut violations)?;

        // Check for inconsistent patterns
        self.detect_inconsistent_patterns(patterns, &mut violations)?;

        // Filter by minimum severity
        violations.retain(|v| v.severity >= self.config.min_severity);

        // TOYOTA WAY FIX: Deduplicate violations to prevent false inflation
        // Issue: Same pattern reported by multiple detection methods
        violations = self.deduplicate_violations(violations);

        // Sort by priority, then by message and affected file. Without the
        // tie-breaks a stable sort just preserves whatever order dedup produced,
        // so equal-priority violations came out shuffled between runs.
        violations.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.message.cmp(&b.message))
                .then_with(|| a.affected_files.cmp(&b.affected_files))
        });

        Ok(violations)
    }

    /// Distinct files a pattern occurs in, in a fixed order.
    fn sorted_affected_files(pattern: &AstPattern) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = pattern.locations.iter().map(|l| l.file.clone()).collect();
        files.sort();
        files.dedup();
        files
    }

    /// Detect repetitive pattern violations
    fn detect_repetitive_patterns(
        &self,
        patterns: &PatternCollection,
        violations: &mut Vec<ActionableViolation>,
    ) -> Result<()> {
        for pattern in patterns.patterns.values() {
            if pattern.frequency > self.config.max_pattern_repetition {
                let severity = self.calculate_repetition_severity(pattern.frequency);
                let loc_reduction = self.estimate_loc_reduction(pattern);

                violations.push(ActionableViolation {
                    severity,
                    pattern: PatternSummary {
                        pattern_type: pattern.pattern_type,
                        repetitions: pattern.frequency,
                        variation_score: pattern.variation_score,
                        example_code: pattern.example_code.clone(),
                    },
                    message: format!(
                        "{:?} pattern repeated {} times",
                        pattern.pattern_type, pattern.frequency
                    ),
                    fix_suggestion: self.generate_fix_suggestion(pattern),
                    estimated_loc_reduction: loc_reduction,
                    // Was collected through a HashSet, so the file list (and the
                    // "file" a quality-gate row was attributed to, which is the
                    // first entry) changed between runs.
                    affected_files: Self::sorted_affected_files(pattern),
                    priority_score: self.calculate_priority(severity, loc_reduction),
                });
            }
        }
        Ok(())
    }

    /// Detect low diversity violations
    ///
    /// An unmeasured diversity (`None`) can never be below a threshold — a gate
    /// must not fail on a number that was never computed.
    fn detect_low_diversity(
        &self,
        patterns: &PatternCollection,
        metrics: &EntropyMetrics,
        violations: &mut Vec<ActionableViolation>,
    ) -> Result<()> {
        let Some(diversity) = metrics.pattern_diversity else {
            return Ok(());
        };
        if diversity < self.config.min_pattern_diversity {
            violations.push(ActionableViolation {
                severity: Severity::Medium,
                pattern: PatternSummary {
                    pattern_type: PatternType::ControlFlow,
                    repetitions: 0,
                    variation_score: 1.0 - diversity,
                    example_code: "Various repetitive patterns".to_string(),
                },
                message: format!(
                    "Low pattern diversity: {:.1}% (minimum: {:.1}%)",
                    diversity * 100.0,
                    self.config.min_pattern_diversity * 100.0
                ),
                fix_suggestion: "Consider extracting common patterns into reusable functions"
                    .to_string(),
                estimated_loc_reduction: (metrics.total_loc as f64 * 0.15) as usize,
                affected_files: vec![],
                priority_score: 5.0,
            });
        }
        Ok(())
    }

    /// The most frequent measured pattern type, used to attribute the
    /// project-level diversity finding to real data rather than a hardcoded
    /// `PatternType::ControlFlow`. Only called with a non-empty population.
    fn dominant_pattern_type(metrics: &EntropyMetrics) -> PatternType {
        metrics
            .patterns_by_type
            .iter()
            .max_by_key(|(ty, count)| (**count, format!("{ty:?}")))
            .map_or(PatternType::ControlFlow, |(ty, _)| *ty)
    }

    /// Detect cross-file duplication
    fn detect_cross_file_duplication(
        &self,
        patterns: &PatternCollection,
        violations: &mut Vec<ActionableViolation>,
    ) -> Result<()> {
        // Find patterns that appear in multiple files
        for pattern in patterns.patterns.values() {
            let unique_files = Self::sorted_affected_files(pattern);

            if unique_files.len() > 2 {
                let severity = if unique_files.len() > 5 {
                    Severity::High
                } else {
                    Severity::Medium
                };

                violations.push(ActionableViolation {
                    severity,
                    pattern: PatternSummary {
                        pattern_type: pattern.pattern_type,
                        repetitions: pattern.frequency,
                        variation_score: pattern.variation_score,
                        example_code: pattern.example_code.clone(),
                    },
                    message: format!(
                        "{:?} pattern duplicated across {} files",
                        pattern.pattern_type,
                        unique_files.len()
                    ),
                    fix_suggestion: format!(
                        "Extract to shared module: {}",
                        self.suggest_module_name(pattern.pattern_type)
                    ),
                    estimated_loc_reduction: self.estimate_loc_reduction(pattern),
                    affected_files: unique_files,
                    priority_score: 8.0,
                });
            }
        }
        Ok(())
    }

    /// Detect inconsistent pattern implementations
    fn detect_inconsistent_patterns(
        &self,
        patterns: &PatternCollection,
        violations: &mut Vec<ActionableViolation>,
    ) -> Result<()> {
        for pattern in patterns.patterns.values() {
            if pattern.variation_score > self.config.max_inconsistency_score {
                violations.push(ActionableViolation {
                    severity: Severity::Medium,
                    pattern: PatternSummary {
                        pattern_type: pattern.pattern_type,
                        repetitions: pattern.frequency,
                        variation_score: pattern.variation_score,
                        example_code: pattern.example_code.clone(),
                    },
                    message: format!(
                        "Inconsistent {:?} implementations (variation: {:.1}%)",
                        pattern.pattern_type,
                        pattern.variation_score * 100.0
                    ),
                    fix_suggestion: format!(
                        "Standardize {} pattern across codebase",
                        self.pattern_name(pattern.pattern_type)
                    ),
                    // estimated_loc already covers all `frequency` instances;
                    // multiplying by frequency again counted every line twice over.
                    estimated_loc_reduction: (pattern.estimated_loc as f64 * 0.3) as usize,
                    affected_files: Self::sorted_affected_files(pattern),
                    priority_score: 6.0,
                });
            }
        }
        Ok(())
    }

    /// Calculate severity based on repetition count
    fn calculate_repetition_severity(&self, frequency: usize) -> Severity {
        if frequency > 10 {
            Severity::High
        } else if frequency > 5 {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    /// Estimate LOC reduction from fixing a pattern
    ///
    /// `estimated_loc` is the LOC covered by *all* `frequency` instances, so the
    /// per-instance size has to be divided back out. The old code multiplied the
    /// whole-group figure by `frequency - 1` again, which is where numbers like
    /// "saves 302 lines" for a ten-line pattern came from.
    fn estimate_loc_reduction(&self, pattern: &AstPattern) -> usize {
        if pattern.frequency <= 1 {
            return 0;
        }
        let instances_to_remove = pattern.frequency - 1;
        let per_instance = pattern.estimated_loc as f64 / pattern.frequency as f64;
        let reduction_factor = 0.8; // Assume 80% can be eliminated

        (instances_to_remove as f64 * per_instance * reduction_factor) as usize
    }

    /// Generate fix suggestion for a pattern
    fn generate_fix_suggestion(&self, pattern: &AstPattern) -> String {
        match pattern.pattern_type {
            PatternType::ErrorHandling => {
                format!(
                    "Extract to `handle_{}_error()` function",
                    self.context_name(pattern)
                )
            }
            PatternType::DataValidation => "Create validation trait or module".to_string(),
            PatternType::ResourceManagement => {
                "Implement RAII pattern or use guard types".to_string()
            }
            PatternType::ControlFlow => "Refactor to strategy pattern or polymorphism".to_string(),
            PatternType::DataTransformation => {
                "Extract to data transformation pipeline".to_string()
            }
            PatternType::ApiCall => "Create API client abstraction".to_string(),
        }
    }

    /// Calculate priority score for ordering violations
    fn calculate_priority(&self, severity: Severity, loc_reduction: usize) -> f64 {
        let severity_score = match severity {
            Severity::High => 10.0,
            Severity::Medium => 5.0,
            Severity::Low => 1.0,
        };

        let loc_score = (loc_reduction as f64 / 100.0).min(10.0);

        severity_score + loc_score
    }

    /// Suggest module name for extracted pattern
    fn suggest_module_name(&self, pattern_type: PatternType) -> &'static str {
        match pattern_type {
            PatternType::ErrorHandling => "error_handler",
            PatternType::DataValidation => "validators",
            PatternType::ResourceManagement => "resource_guards",
            PatternType::ControlFlow => "control_flow",
            PatternType::DataTransformation => "transformers",
            PatternType::ApiCall => "api_client",
        }
    }

    /// Get human-readable pattern name
    fn pattern_name(&self, pattern_type: PatternType) -> &'static str {
        match pattern_type {
            PatternType::ErrorHandling => "error handling",
            PatternType::DataValidation => "validation",
            PatternType::ResourceManagement => "resource management",
            PatternType::ControlFlow => "control flow",
            PatternType::DataTransformation => "data transformation",
            PatternType::ApiCall => "API call",
        }
    }

    /// Extract context name from pattern
    fn context_name(&self, _pattern: &AstPattern) -> &'static str {
        // Extract meaningful name from pattern
        // Simplified - would analyze actual AST
        "context"
    }

    /// Deduplicate violations to prevent the same pattern being reported multiple times
    ///
    /// Issue: Same pattern can be detected by multiple methods (repetitive, cross-file, etc.)
    /// causing inflated violation counts. This deduplicates based on pattern type and core message.
    fn deduplicate_violations(
        &self,
        violations: Vec<ActionableViolation>,
    ) -> Vec<ActionableViolation> {
        use std::collections::BTreeMap;

        // BTreeMap: `into_values()` on a HashMap emitted the survivors in a
        // per-process random order, which the stable priority sort below then
        // preserved for equal priorities.
        let mut unique_violations: BTreeMap<String, ActionableViolation> = BTreeMap::new();

        for violation in violations {
            // Create a key based on pattern type and the core pattern identifier
            let key = format!(
                "{}:{}:{}",
                violation.pattern.pattern_type as u8,
                violation.pattern.repetitions,
                violation.pattern.example_code.len() // Use code length as pattern identifier
            );

            // Keep the violation with highest severity/priority
            match unique_violations.get(&key) {
                Some(existing) if existing.priority_score >= violation.priority_score => {
                    // Keep existing
                }
                _ => {
                    // Replace or insert new
                    unique_violations.insert(key, violation);
                }
            }
        }

        unique_violations.into_values().collect()
    }
}
