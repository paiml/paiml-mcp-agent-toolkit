impl Default for QualityAssistant {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityAssistant {
    /// Create a new quality assistant
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            pattern_db: Self::initialize_patterns(),
            feedback: FeedbackCollector::new(),
            scorer: ConfidenceScorer::new(),
        }
    }

    /// Suggest fixes for a violation
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn suggest(
        &self,
        violation: &crate::unified_quality::metrics::Violation,
    ) -> Vec<Suggestion> {
        self.pattern_db
            .get(&violation.violation_type)
            .map(|patterns| {
                patterns
                    .iter()
                    .map(|p| {
                        let confidence = self.scorer.score(p, violation);
                        Suggestion {
                            pattern: p.clone(),
                            confidence,
                            preview: self.generate_diff(violation, p),
                            impact: self.estimate_impact(p),
                        }
                    })
                    .filter(|s| s.confidence > 0.6)
                    .take(3)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Record feedback on a suggestion
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_feedback(
        &mut self,
        suggestion_id: &str,
        accepted: bool,
        outcome: Option<String>,
    ) {
        self.feedback.record(suggestion_id, accepted, outcome);
    }

    /// Get suggestion success rate
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_success_rate(&self) -> f64 {
        self.feedback.metrics.success_rate
    }

    /// Initialize pattern database with common refactorings
    fn initialize_patterns() -> HashMap<ViolationType, Vec<Pattern>> {
        let mut patterns = HashMap::new();

        // Complexity reduction patterns
        patterns.insert(
            ViolationType::Complexity,
            vec![
                Pattern {
                    id: "extract_method".to_string(),
                    name: "Extract Method".to_string(),
                    description: "Extract complex logic into separate functions".to_string(),
                    template: "fn extracted_logic() { ... }".to_string(),
                    success_rate: 0.85,
                    contexts: vec!["high_complexity".to_string()],
                    example: Example {
                        before: "if a && b && c { /* complex */ }".to_string(),
                        after: "if should_process() { process() }".to_string(),
                        improvement: "Reduced complexity from 15 to 5".to_string(),
                    },
                },
                Pattern {
                    id: "early_return".to_string(),
                    name: "Early Return".to_string(),
                    description: "Use early returns to reduce nesting".to_string(),
                    template: "if !condition { return }".to_string(),
                    success_rate: 0.75,
                    contexts: vec!["nested_conditions".to_string()],
                    example: Example {
                        before: "if valid { /* nested */ }".to_string(),
                        after: "if !valid { return } /* flat */".to_string(),
                        improvement: "Reduced nesting by 2 levels".to_string(),
                    },
                },
            ],
        );

        // SATD removal patterns
        patterns.insert(
            ViolationType::Satd,
            vec![Pattern {
                id: "implement_todo".to_string(),
                name: "Implement TODO".to_string(),
                description: "Complete the TODO implementation".to_string(),
                template: "// Completed implementation".to_string(),
                success_rate: 0.70,
                contexts: vec!["todo_comment".to_string()],
                example: Example {
                    before: "// Add validation".to_string(),
                    after: "validate_input(&input)?;".to_string(),
                    improvement: "Removed technical debt".to_string(),
                },
            }],
        );

        // Dead code removal patterns
        patterns.insert(
            ViolationType::DeadCode,
            vec![Pattern {
                id: "remove_dead_code".to_string(),
                name: "Remove Dead Code".to_string(),
                description: "Remove unreachable or unused code".to_string(),
                template: "// Code removed".to_string(),
                success_rate: 0.95,
                contexts: vec!["unused".to_string()],
                example: Example {
                    before: " fn unused() {}".to_string(),
                    after: "// Removed".to_string(),
                    improvement: "Removed 10 lines of dead code".to_string(),
                },
            }],
        );

        patterns
    }

    /// Generate diff preview for a suggestion
    fn generate_diff(&self, violation: &Violation, pattern: &Pattern) -> String {
        format!(
            "--- {}\n+++ {}\n@@ -1,1 +1,1 @@\n-{}\n+{}",
            violation.file, violation.file, pattern.example.before, pattern.example.after
        )
    }

    /// Estimate impact of applying a pattern
    fn estimate_impact(&self, pattern: &Pattern) -> Impact {
        Impact {
            complexity_reduction: match pattern.id.as_str() {
                "extract_method" => 10,
                "early_return" => 5,
                _ => 2,
            },
            loc_change: match pattern.id.as_str() {
                "remove_dead_code" => -10,
                "extract_method" => 5,
                _ => 0,
            },
            coverage_impact: 0.0,
            risk: match pattern.success_rate {
                r if r > 0.8 => RiskLevel::Low,
                r if r > 0.6 => RiskLevel::Medium,
                _ => RiskLevel::High,
            },
        }
    }

    /// Analyze a file and generate suggestions
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file(
        &self,
        file_path: &std::path::Path,
    ) -> Result<Vec<Suggestion>, anyhow::Error> {
        // Read file content and analyze for violations
        let content = std::fs::read_to_string(file_path)?;

        // Simple violation detection for demonstration
        let mut suggestions = Vec::new();

        // Check for TODO comments (SATD)
        if content.contains("TODO") || content.contains("FIXME") {
            let violation = crate::unified_quality::metrics::Violation {
                file: file_path.to_string_lossy().to_string(),
                violation_type: crate::unified_quality::metrics::ViolationType::Satd,
                severity: crate::unified_quality::metrics::Severity::Medium,
                value: 1.0,
                threshold: 0.0,
            };
            suggestions.extend(self.suggest(&violation));
        }

        Ok(suggestions)
    }

    /// Generate suggestions for a file (synchronous version)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn generate_suggestions(
        &self,
        file_path: &std::path::Path,
    ) -> Result<Vec<Suggestion>, anyhow::Error> {
        // Synchronous version of analyze_file
        let content = std::fs::read_to_string(file_path)?;
        let mut suggestions = Vec::new();

        // Check for various violations
        if content.contains("TODO") || content.contains("FIXME") {
            let violation = crate::unified_quality::metrics::Violation {
                file: file_path.to_string_lossy().to_string(),
                violation_type: crate::unified_quality::metrics::ViolationType::Satd,
                severity: crate::unified_quality::metrics::Severity::Medium,
                value: 1.0,
                threshold: 0.0,
            };
            suggestions.extend(self.suggest(&violation));
        }

        Ok(suggestions)
    }
}
