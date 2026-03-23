// analyzer_simple_metrics.rs — Metric analysis methods for TdgAnalyzer
// Included by analyzer_simple.rs — shares parent module scope

impl TdgAnalyzer {
    fn analyze_structural_complexity(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.structural_complexity;

        let lines: Vec<&str> = source.lines().collect();
        let cyclomatic = self.estimate_cyclomatic_complexity(&lines);

        if cyclomatic > self.config.thresholds.max_cyclomatic_complexity {
            let excess = (cyclomatic - self.config.thresholds.max_cyclomatic_complexity) as f32;
            let penalty = (excess * 0.5).min(15.0);

            if let Some(applied) = tracker.apply(
                format!("high_cyclomatic_{cyclomatic}"),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cyclomatic complexity: {cyclomatic}"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_semantic_complexity(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.semantic_complexity;

        let nesting_depth = self.estimate_nesting_depth(source);
        if nesting_depth > self.config.thresholds.max_nesting_depth as usize {
            let penalty = ((nesting_depth - self.config.thresholds.max_nesting_depth as usize)
                as f32)
                .min(10.0);

            if let Some(applied) = tracker.apply(
                format!("deep_nesting_{nesting_depth}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Deep nesting: {nesting_depth} levels"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_duplication(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.duplication;

        let duplication_ratio = self.estimate_duplication_ratio(source);
        if duplication_ratio > 0.1 {
            let penalty = (duplication_ratio * 20.0).min(20.0);

            if let Some(applied) = tracker.apply(
                format!("duplication_{duplication_ratio:.2}"),
                MetricCategory::Duplication,
                penalty,
                format!("Code duplication: {:.1}%", duplication_ratio * 100.0),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    fn analyze_coupling(&self, source: &str, _tracker: &mut PenaltyTracker) -> f32 {
        let import_count = source
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("use ")
                    || trimmed.starts_with("import ")
                    || trimmed.starts_with("from ")
                    || trimmed.starts_with("#include ")
                    || trimmed.starts_with("open ")
            })
            .count();

        let base_score = self.config.weights.coupling;
        if import_count > 20 {
            base_score - ((import_count - 20) as f32 * 0.2).min(10.0)
        } else {
            base_score
        }
        .max(0.0)
    }

    fn analyze_documentation(
        &self,
        source: &str,
        language: Language,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        let total_lines = source.lines().count();
        if total_lines == 0 {
            return self.config.weights.documentation;
        }

        let doc_lines = count_doc_lines(source, language);

        let coverage = doc_lines as f32 / total_lines as f32;
        (coverage * self.config.weights.documentation).min(self.config.weights.documentation)
    }

    fn analyze_consistency(
        &self,
        source: &str,
        _language: Language,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        // Simple consistency check based on indentation style
        let lines: Vec<&str> = source.lines().collect();
        if lines.is_empty() {
            return self.config.weights.consistency;
        }

        let mut tab_count = 0;
        let mut space_count = 0;

        for line in &lines {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("    ") || line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if total_indented == 0 {
            return self.config.weights.consistency;
        }

        let consistency = if tab_count > space_count {
            tab_count as f32 / total_indented as f32
        } else {
            space_count as f32 / total_indented as f32
        };

        consistency * self.config.weights.consistency
    }
}

/// Count documentation lines for a given language.
/// Extracted as free function to reduce method complexity.
fn count_doc_lines(source: &str, language: Language) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            match language {
                Language::Rust => trimmed.starts_with("///") || trimmed.starts_with("//!"),
                Language::Python => trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''"),
                Language::JavaScript | Language::TypeScript => {
                    trimmed.starts_with("/**") || trimmed.starts_with('*')
                }
                Language::Lean => {
                    trimmed.starts_with("/--") // Lean doc comments
                        || trimmed.starts_with("/-!")  // Lean module doc comments
                        || trimmed.starts_with("--")   // Lean line comments (doc-like)
                }
                _ => trimmed.starts_with("//") || trimmed.starts_with("/*"),
            }
        })
        .count()
}
