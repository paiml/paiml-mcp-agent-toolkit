impl TdgAnalyzerAst {
    #[allow(clippy::cast_possible_truncation)]
    fn score_structural_complexity(
        &self,
        cyclomatic: u32,
        cognitive: u32,
        nesting_depth: usize,
        method_length: usize,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.structural_complexity;

        // Penalize high cyclomatic complexity
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

        // Penalize high cognitive complexity
        if cognitive > 15 {
            let excess = (cognitive - 15) as f32;
            let penalty = (excess * 0.3).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("high_cognitive_{cognitive}"),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cognitive complexity: {cognitive}"),
            ) {
                points -= applied;
            }
        }

        // Penalize deep nesting
        if nesting_depth > self.config.thresholds.max_nesting_depth as usize {
            let excess = (nesting_depth - self.config.thresholds.max_nesting_depth as usize) as f32;
            let penalty = excess.min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_nesting_{nesting_depth}"),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("Deep nesting: {nesting_depth} levels"),
            ) {
                points -= applied;
            }
        }

        // Penalize long methods
        if method_length > 50 {
            let excess = ((method_length - 50) as f32 / 10.0).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("long_method_{method_length}"),
                MetricCategory::StructuralComplexity,
                excess,
                format!("Long method: {method_length} lines"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_semantic_complexity(
        &self,
        max_params: usize,
        type_complexity: u32,
        abstraction_levels: u32,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.semantic_complexity;

        // Penalize too many parameters
        if max_params > 5 {
            let penalty = ((max_params - 5) as f32 * 0.5).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_params_{max_params}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Too many parameters: {max_params}"),
            ) {
                points -= applied;
            }
        }

        // Penalize high type complexity
        if type_complexity > 10 {
            let penalty = ((type_complexity - 10) as f32 * 0.3).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("complex_types_{type_complexity}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Complex type usage: {type_complexity}"),
            ) {
                points -= applied;
            }
        }

        // Penalize too many abstraction levels
        if abstraction_levels > 3 {
            let penalty = ((abstraction_levels - 3) as f32).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_abstraction_{abstraction_levels}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Deep abstraction: {abstraction_levels} levels"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_duplication_ast(
        &self,
        source: &str,
        _language: Language,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Hash-based duplication detection with semantic filtering
        // Excludes comments and blank lines for accurate duplicate detection
        let mut points = self.config.weights.duplication;

        let lines: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
            .collect();

        if lines.len() < 3 {
            return points;
        }

        // Count exact duplicates
        let mut duplicates = 0;
        let mut seen = std::collections::HashSet::new();

        for line in &lines {
            if line.len() > 10 && !seen.insert(line) {
                duplicates += 1;
            }
        }

        let duplication_ratio = duplicates as f32 / lines.len() as f32;

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

    #[allow(clippy::cast_possible_truncation)]
    fn score_coupling(
        &self,
        import_count: u32,
        external_calls: u32,
        _interface_implementations: u32,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.coupling;

        // Penalize too many imports
        if import_count > 20 {
            let penalty = ((import_count - 20) as f32 * 0.2).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("many_imports_{import_count}"),
                MetricCategory::Coupling,
                penalty,
                format!("Too many imports: {import_count}"),
            ) {
                points -= applied;
            }
        }

        // Penalize too many external calls
        if external_calls > 50 {
            let penalty = ((external_calls - 50) as f32 * 0.1).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_external_calls_{external_calls}"),
                MetricCategory::Coupling,
                penalty,
                format!("Too many external calls: {external_calls}"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_documentation(
        &self,
        documented_items: u32,
        total_public_items: u32,
        comment_lines: u32,
        total_lines: u32,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        if total_public_items == 0 {
            return self.config.weights.documentation;
        }

        let coverage = documented_items as f32 / total_public_items as f32;
        let comment_ratio = comment_lines as f32 / total_lines as f32;

        // Weight: 70% API documentation, 30% inline comments
        let score = coverage * 0.7 + comment_ratio * 0.3;

        (score * self.config.weights.documentation).min(self.config.weights.documentation)
    }


}
