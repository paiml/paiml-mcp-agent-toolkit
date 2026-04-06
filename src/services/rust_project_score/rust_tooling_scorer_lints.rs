// Workspace lint configuration scoring
// Included into rust_tooling_scorer.rs

impl RustToolingScorer {
    /// Score workspace-level lint configuration (v2.0 feature)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    /// - +5pts: Workspace-level lints configured ([workspace.lints])
    /// - +4pts: High-value lint categories enabled (correctness, suspicious, perf)
    /// - +3pts: .clippy.toml with disallowed-methods
    ///
    /// Total possible: 12 points
    ///
    /// References:
    /// - Johnson et al. 2013 ICSE: Quality over quantity (avoid warning blindness)
    /// - Bacchelli & Bird 2013 ICSE: Automated style enforcement reduces review waste
    pub(super) fn score_workspace_lints(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let mut score = 0.0;

        // Read Cargo.toml
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0); // No Cargo.toml, can't have workspace lints
        }

        // Use cache if available, otherwise read file
        let cargo_toml_content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
                .clone()
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Check 1: Workspace-level lints configured (+5pts)
        let has_workspace_rust_lints = cargo_toml_content.contains("[workspace.lints.rust]");
        let has_workspace_clippy_lints = cargo_toml_content.contains("[workspace.lints.clippy]");

        if has_workspace_rust_lints || has_workspace_clippy_lints {
            score += 5.0;
        }

        // Check 2: High-value lint categories (+4pts)
        // Look for key lints that indicate quality focus (not just quantity)
        let has_high_value_lints = cargo_toml_content.contains("unsafe_op_in_unsafe_fn") || // Safety-critical
            cargo_toml_content.contains("unreachable_pub") ||        // API clarity
            cargo_toml_content.contains("unused_lifetimes") ||       // Code quality
            cargo_toml_content.contains("checked_conversions") ||    // Correctness
            cargo_toml_content.contains("fallible_impl_from"); // Correctness

        if has_high_value_lints {
            score += 4.0;
        }

        // Check 3: .clippy.toml with disallowed-methods (+3pts)
        let clippy_toml_path = project_path.join(".clippy.toml");
        if clippy_toml_path.exists() {
            // Use cache if available
            let clippy_toml_content = if let Some(cache) = cache {
                cache
                    .get(&clippy_toml_path)
                    .ok_or_else(|| {
                        ScorerError::IoError(".clippy.toml not in cache".to_string())
                    })?
                    .clone()
            } else {
                std::fs::read_to_string(&clippy_toml_path)
                    .map_err(|e| ScorerError::IoError(e.to_string()))?
            };

            // Check for disallowed-methods section with actual content
            if clippy_toml_content.contains("disallowed-methods") {
                score += 3.0;
            }
        }

        Ok(score)
    }

    /// Generate workspace lint recommendations based on Cargo.toml content
    pub(super) fn lint_recommendations(project_path: &Path) -> Vec<String> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut recs = Vec::new();
        let cargo_path = project_path.join("Cargo.toml");
        if !cargo_path.exists() {
            return recs;
        }
        let content = match std::fs::read_to_string(&cargo_path) {
            Ok(c) => c,
            Err(_) => return recs,
        };
        if !content.contains("[workspace.lints") {
            recs.push("Add [workspace.lints.rust] and [workspace.lints.clippy] to Cargo.toml for consistent linting across all crates".to_string());
        }
        if !content.contains("unsafe_op_in_unsafe_fn")
            && !content.contains("checked_conversions")
        {
            recs.push("Enable high-value lint categories (unsafe_op_in_unsafe_fn, unreachable_pub, checked_conversions) for better code quality".to_string());
        }
        if !project_path.join(".clippy.toml").exists() {
            recs.push("Create .clippy.toml with disallowed-methods to enforce project-specific style preferences".to_string());
        }
        recs
    }
}
