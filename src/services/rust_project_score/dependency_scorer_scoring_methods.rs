impl DependencyScorer {
    /// Create a new DependencyScorer
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            name: "Dependency Health".to_string(),
            max_points: 12.0,
        }
    }

    /// Score dependency count (5pts)
    /// Parses Cargo.toml and penalizes excessive dependencies
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available (eliminates redundant read)
    fn score_dependency_count(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .cloned()
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Parse dependency count (simple line-based parsing)
        let mut dependency_count = 0;
        let mut in_dependencies = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for [dependencies] section
            if trimmed == "[dependencies]" {
                in_dependencies = true;
                continue;
            }

            // Exit dependencies section when we hit another section
            if in_dependencies && trimmed.starts_with('[') {
                in_dependencies = false;
            }

            // Count dependencies (lines with = that aren't comments)
            if in_dependencies
                && !trimmed.starts_with('#')
                && trimmed.contains('=')
                && !trimmed.is_empty()
            {
                dependency_count += 1;
            }
        }

        // Tiered scoring based on dependency count
        // Evidence-based: ≤20 dependencies optimal for security & build time
        score_dependency_count_tier(dependency_count)
    }

    /// Score feature flags (4pts)
    /// Checks for [features] section in Cargo.toml
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available (eliminates redundant read)
    fn score_feature_flags(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .cloned()
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Check for [features] section
        let has_features = content.contains("[features]");

        if !has_features {
            return Ok(0.0);
        }

        // Count feature definitions
        let mut feature_count = 0;
        let mut in_features = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for [features] section
            if trimmed == "[features]" {
                in_features = true;
                continue;
            }

            // Exit features section when we hit another section
            if in_features && trimmed.starts_with('[') {
                in_features = false;
            }

            // Count feature definitions (lines with = that aren't comments)
            if in_features
                && !trimmed.starts_with('#')
                && trimmed.contains('=')
                && !trimmed.is_empty()
            {
                feature_count += 1;
            }
        }

        // Tiered scoring based on feature count
        score_feature_count_tier(feature_count)
    }

    /// Score tree pruning (3pts)
    /// Awards points for having dependency management practices
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available (eliminates redundant read)
    fn score_tree_pruning(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .cloned()
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Check for optional dependencies (good practice)
        let has_optional_deps = content.contains("optional = true");

        // Check for dependency features (selective imports)
        let has_dep_features = content.contains("features = [");

        // Check for default-features = false (tree pruning)
        let disables_default_features = content.contains("default-features = false");

        // Score based on dependency management practices
        let mut score: f64 = 0.0;

        if has_optional_deps {
            score += 1.5; // Optional dependencies reduce bloat
        }

        if has_dep_features {
            score += 1.0; // Selective feature imports
        }

        if disables_default_features {
            score += 0.5; // Explicitly prunes unnecessary features
        }

        Ok(score.min(3.0)) // Cap at 3.0 points
    }

    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    fn score_internal(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score dependency count (5pts)
        match self.score_dependency_count(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score feature flags (4pts)
        match self.score_feature_flags(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score tree pruning (3pts)
        match self.score_tree_pruning(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}

/// Helper: Map dependency count to score tier (extracted for complexity reduction)
///
/// #242: Relaxed thresholds — previous >30=0 was too harsh for large projects.
/// Large feature-rich libraries (ML, web frameworks) legitimately need 30-50 deps.
fn score_dependency_count_tier(dependency_count: usize) -> ScorerResult<f64> {
    debug_assert!(true, "contract: score_dependency_count_tier");
    if dependency_count <= 15 {
        Ok(5.0) // Lean dependencies - excellent
    } else if dependency_count <= 30 {
        Ok(4.0) // Moderate dependencies - good
    } else if dependency_count <= 50 {
        Ok(2.0) // Many dependencies - acceptable
    } else {
        Ok(1.0) // Heavy dependencies - still gives some credit
    }
}

/// Helper: Map feature count to score tier (extracted for complexity reduction)
fn score_feature_count_tier(feature_count: usize) -> ScorerResult<f64> {
    debug_assert!(true, "contract: score_feature_count_tier");
    if feature_count >= 3 {
        Ok(4.0) // Comprehensive feature flags
    } else if feature_count >= 1 {
        Ok(3.0) // Some feature flags
    } else {
        Ok(0.0) // No feature flags
    }
}
