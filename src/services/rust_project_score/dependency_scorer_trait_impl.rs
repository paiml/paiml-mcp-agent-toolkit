impl Default for DependencyScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for DependencyScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        // Backward compatibility: call with no cache
        self.score_internal(project_path, None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        // This scorer doesn't have expensive operations, so mode doesn't affect it
        self.score(project_path)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Kaizen Round 4: Use FileCache to eliminate 3 redundant Cargo.toml reads
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check dependency count (no cache - backward compatibility)
        if let Ok(score) = self.score_dependency_count(project_path, None) {
            if score < 5.0 {
                recommendations.push(
                    "Reduce dependency count: Aim for ≤20 dependencies to improve security and build times".to_string(),
                );
            }
        }

        // Check feature flags (no cache - backward compatibility)
        if let Ok(score) = self.score_feature_flags(project_path, None) {
            if score < 4.0 {
                recommendations.push(
                    "Add feature flags: Use [features] to make dependencies optional and enable modular builds".to_string(),
                );
            }
        }

        // Check tree pruning (no cache - backward compatibility)
        if let Ok(score) = self.score_tree_pruning(project_path, None) {
            if score < 3.0 {
                recommendations.push(
                    "Optimize dependency tree: Use optional dependencies and disable default features to reduce bloat".to_string(),
                );
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for DependencyScorer {}
unsafe impl Sync for DependencyScorer {}
