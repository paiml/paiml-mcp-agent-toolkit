// Metadata scoring: docs.rs, workspace organization, release automation
// Included into rust_tooling_scorer.rs

/// Read Cargo.toml from cache or filesystem
fn read_cargo_toml(
    project_path: &Path,
    cache: Option<&FileCache>,
) -> ScorerResult<String> {
    let cargo_toml_path = project_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(ScorerError::IoError("Cargo.toml not found".to_string()));
    }
    if let Some(cache) = cache {
        cache
            .get(&cargo_toml_path)
            .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))
            .cloned()
    } else {
        std::fs::read_to_string(&cargo_toml_path)
            .map_err(|e| ScorerError::IoError(e.to_string()))
    }
}

impl RustToolingScorer {
    /// Score docs.rs metadata configuration (v2.0 Phase 3)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    /// - +5pts: `[package.metadata.docs.rs]` exists
    /// - +3pts: `all-features = true` (comprehensive docs)
    /// - +2pts: `--generate-link-to-definition` in rustdoc-args
    ///
    /// Total possible: 10 points
    ///
    /// References:
    /// - Aghajani et al. 2019 ICSE: 57% of docs outdated within 6 months
    pub(super) fn score_docs_rs_metadata(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0);
        }

        let cargo_toml_content = read_cargo_toml(project_path, cache)?;
        let mut score = 0.0;

        // Check 1: docs.rs metadata section exists (+5pts)
        if cargo_toml_content.contains("[package.metadata.docs.rs]") {
            score += 5.0;

            // Check 2: all-features = true (+3pts)
            if cargo_toml_content.contains("all-features = true") {
                score += 3.0;
            }

            // Check 3: --generate-link-to-definition in rustdoc-args (+2pts)
            if cargo_toml_content.contains("--generate-link-to-definition") {
                score += 2.0;
            }
        }

        Ok(score)
    }

    /// Score workspace organization (v2.0 Phase 3)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    /// - +6pts: Project uses workspace (for multi-crate projects)
    /// - +3pts: `resolver = "2"` specified
    /// - +2pts: `[workspace.dependencies]` for shared deps
    /// - +2pts: `[workspace.package]` for shared metadata
    ///
    /// Total possible: 13 points
    ///
    /// References:
    /// - Build System Evolution ICSE 2024: Workspace projects have 34% fewer dependency conflicts
    pub(super) fn score_workspace_organization(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0);
        }

        let cargo_toml_content = read_cargo_toml(project_path, cache)?;
        let mut score = 0.0;

        // Check 1: Workspace section exists (+6pts)
        if cargo_toml_content.contains("[workspace]") {
            score += 6.0;

            // Check 2: resolver = "2" (+3pts)
            if cargo_toml_content.contains("resolver = \"2\"")
                || cargo_toml_content.contains("resolver = '2'")
            {
                score += 3.0;
            }

            // Check 3: [workspace.dependencies] (+2pts)
            if cargo_toml_content.contains("[workspace.dependencies]") {
                score += 2.0;
            }

            // Check 4: [workspace.package] (+2pts)
            if cargo_toml_content.contains("[workspace.package]") {
                score += 2.0;
            }
        }

        Ok(score)
    }

    /// Score release automation configuration (v2.0 Phase 3)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    /// - +5pts: `[package.metadata.release]` configured
    /// - +3pts: Automated CHANGELOG.md updates (pre-release-replacements)
    /// - +2pts: Version synchronization across workspace (shared-version)
    /// - +2pts: `.github/workflows/post-release.yml` automation
    ///
    /// Total possible: 12 points
    ///
    /// References:
    /// - FSE 2022: Manual release processes have 3.8x higher error rate
    pub(super) fn score_release_automation(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0);
        }

        let cargo_toml_content = read_cargo_toml(project_path, cache)?;
        let mut score = 0.0;

        // Check 1: [package.metadata.release] exists (+5pts)
        if cargo_toml_content.contains("[package.metadata.release]") {
            score += 5.0;

            // Check 2: CHANGELOG.md automation (+3pts)
            if cargo_toml_content.contains("pre-release-replacements")
                && cargo_toml_content.contains("CHANGELOG.md")
            {
                score += 3.0;
            }

            // Check 3: Version synchronization (+2pts)
            if cargo_toml_content.contains("shared-version") {
                score += 2.0;
            }
        }

        // Check 4: Post-release workflow (+2pts)
        let post_release_path = project_path.join(".github/workflows/post-release.yml");
        if post_release_path.exists() {
            score += 2.0;
        }

        Ok(score)
    }
}
