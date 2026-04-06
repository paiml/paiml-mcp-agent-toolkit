// BuildPerfScorer - Build Performance Category (15 points)
//
// Analyzes Rust project build configuration for performance optimization:
// - LTO Enabled (2pts): Link-Time Optimization in release profile
// - Target Dir Size (2pts): Reasonable target directory size (<10GB)
// - Cargo.lock Present (2pts): Reproducible builds via lockfile
// - Cargo Config (2pts): .cargo/config.toml exists with build settings
// - Incremental Builds (2pts): Incremental compilation enabled
// - Codegen Units (2pts): Optimized codegen-units for release
// - Build System (3pts): Makefile/justfile/build.rs automation
//
// Evidence-based design: LTO provides 10-20% binary size reduction and
// 5-10% runtime performance improvement (LLVM benchmarks 2024).

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;

/// Build Performance scorer
#[derive(Debug, Clone)]
pub struct BuildPerfScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl BuildPerfScorer {
    /// Create a new BuildPerfScorer
    pub fn new() -> Self {
        Self {
            name: "Build Performance".to_string(),
            max_points: 15.0,
        }
    }

    /// Score LTO configuration (2pts)
    /// Checks for Link-Time Optimization in release profile
    ///
    /// Validates:
    /// - `lto = true` or `lto = "thin"` or `lto = "fat"` in [profile.release]
    fn score_lto(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cargo_toml_path = project_path.join("Cargo.toml");

        let content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .cloned()
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Look for LTO in [profile.release] section
        let mut in_release_profile = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for [profile.release] section
            if trimmed == "[profile.release]" {
                in_release_profile = true;
                continue;
            }

            // Exit section when we hit another section
            if in_release_profile && trimmed.starts_with('[') {
                in_release_profile = false;
            }

            // Check for lto setting
            if in_release_profile && trimmed.starts_with("lto") {
                // Accept: lto = true, lto = "thin", lto = "fat"
                if trimmed.contains("true")
                    || trimmed.contains("\"thin\"")
                    || trimmed.contains("\"fat\"")
                {
                    return Ok(2.0);
                }
            }
        }

        Ok(0.0)
    }

    /// Score target directory size (2pts)
    /// Penalizes excessively large target directories (>10GB)
    ///
    /// Note: Only checks if target/ exists, doesn't require it
    fn score_target_dir_size(
        &self,
        project_path: &Path,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let target_path = project_path.join("target");

        // If no target directory, give full points (clean state)
        if !target_path.exists() {
            return Ok(2.0);
        }

        // Calculate directory size (best-effort, don't fail on errors)
        let size_bytes = dir_size(&target_path).unwrap_or(0);
        let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        // Tiered scoring
        if size_gb <= 2.0 {
            Ok(2.0) // Excellent: ≤2GB
        } else if size_gb <= 5.0 {
            Ok(1.5) // Good: ≤5GB
        } else if size_gb <= 10.0 {
            Ok(1.0) // Acceptable: ≤10GB
        } else {
            Ok(0.0) // Poor: >10GB
        }
    }

    /// Score Cargo.lock presence (2pts)
    /// Ensures reproducible builds via lockfile
    fn score_cargo_lock(
        &self,
        project_path: &Path,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cargo_lock_path = project_path.join("Cargo.lock");

        if cargo_lock_path.exists() {
            Ok(2.0)
        } else {
            Ok(0.0)
        }
    }

    /// Score .cargo/config.toml presence (2pts)
    /// Checks for build configuration file
    fn score_cargo_config(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let config_toml = project_path.join(".cargo").join("config.toml");
        let config_old = project_path.join(".cargo").join("config");

        // Check for config.toml (preferred) or legacy config
        let config_path = if config_toml.exists() {
            config_toml
        } else if config_old.exists() {
            config_old
        } else {
            return Ok(0.0);
        };

        // Read content and validate it has useful settings
        let content = if let Some(cache) = cache {
            cache.get(&config_path).cloned()
        } else {
            std::fs::read_to_string(&config_path).ok()
        };

        if let Some(content) = content {
            // Check for common useful settings
            let has_build_settings = content.contains("[build]")
                || content.contains("[target")
                || content.contains("[env]")
                || content.contains("[alias]");

            if has_build_settings {
                return Ok(2.0); // Has meaningful configuration
            }
            return Ok(1.0); // Config exists but minimal
        }

        Ok(0.0)
    }

    /// Score incremental builds (2pts)
    /// Checks for incremental = true in config or default profile
    fn score_incremental_builds(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // Check .cargo/config.toml first
        let config_path = project_path.join(".cargo").join("config.toml");

        if config_path.exists() {
            let content = if let Some(cache) = cache {
                cache.get(&config_path).cloned()
            } else {
                std::fs::read_to_string(&config_path).ok()
            };

            if let Some(content) = content {
                // Check for incremental = true in [build] section
                if content.contains("incremental = true") {
                    return Ok(2.0);
                }
            }
        }

        // Check Cargo.toml for [profile.dev] incremental setting
        let cargo_toml_path = project_path.join("Cargo.toml");
        let content = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).cloned()
        } else {
            std::fs::read_to_string(&cargo_toml_path).ok()
        };

        if let Some(content) = content {
            // Default is incremental for dev, so check if explicitly disabled
            if content.contains("incremental = false") {
                return Ok(0.0);
            }
        }

        // Default: incremental is enabled by default in Rust
        Ok(1.5) // Partial credit for default behavior
    }

    /// Score codegen-units configuration (2pts)
    /// Checks for optimized codegen-units in release profile
    fn score_codegen_units(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cargo_toml_path = project_path.join("Cargo.toml");

        let content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .cloned()
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Look for codegen-units in [profile.release] section
        let mut in_release_profile = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "[profile.release]" {
                in_release_profile = true;
                continue;
            }

            if in_release_profile && trimmed.starts_with('[') {
                in_release_profile = false;
            }

            if in_release_profile && trimmed.starts_with("codegen-units") {
                // codegen-units = 1 is optimal for release
                if trimmed.contains("= 1") {
                    return Ok(2.0);
                }
                // Any explicit setting shows awareness
                return Ok(1.0);
            }
        }

        Ok(0.0)
    }

    /// Score build system presence (3pts)
    /// Checks for Makefile, justfile, or build.rs automation
    fn score_build_system(
        &self,
        project_path: &Path,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut score: f64 = 0.0;

        // Check for Makefile (most common)
        if project_path.join("Makefile").exists() {
            score += 1.5;
        }

        // Check for justfile (modern alternative)
        if project_path.join("justfile").exists() || project_path.join("Justfile").exists() {
            score += 1.5;
        }

        // Check for build.rs (Cargo build script)
        if project_path.join("build.rs").exists() {
            score += 1.0;
        }

        Ok(score.min(3.0)) // Cap at 3.0 points
    }

    /// Internal scoring logic that accepts optional cache
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

        // LTO (2pts)
        match self.score_lto(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        // Target dir size (2pts)
        match self.score_target_dir_size(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        // Cargo.lock (2pts)
        match self.score_cargo_lock(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        // Cargo config (2pts)
        match self.score_cargo_config(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        // Incremental builds (2pts)
        match self.score_incremental_builds(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        // Codegen units (2pts)
        match self.score_codegen_units(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        // Build system (3pts)
        match self.score_build_system(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(_) => {} // Non-fatal, continue
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    fn recommend_lto(&self, path: &Path, recs: &mut Vec<String>) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if self.score_lto(path, None).unwrap_or(0.0) < 2.0 {
            recs.push("Enable LTO: Add `lto = true` or `lto = \"thin\"` to [profile.release] for 10-20% smaller binaries".into());
        }
    }

    fn recommend_cargo_lock(&self, path: &Path, recs: &mut Vec<String>) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if self.score_cargo_lock(path, None).unwrap_or(0.0) < 2.0 {
            recs.push("Add Cargo.lock: Commit Cargo.lock for reproducible builds (required for binaries)".into());
        }
    }

    fn recommend_cargo_config(&self, path: &Path, recs: &mut Vec<String>) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if self.score_cargo_config(path, None).unwrap_or(0.0) < 2.0 {
            recs.push("Add .cargo/config.toml: Configure build settings for consistent builds across machines".into());
        }
    }

    fn recommend_codegen_units(&self, path: &Path, recs: &mut Vec<String>) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if self.score_codegen_units(path, None).unwrap_or(0.0) < 2.0 {
            recs.push("Optimize codegen-units: Add `codegen-units = 1` to [profile.release] for better optimization".into());
        }
    }

    fn recommend_build_system(&self, path: &Path, recs: &mut Vec<String>) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let score = self.score_build_system(path, None).unwrap_or(0.0);
        if score == 0.0 {
            recs.push("Add build automation: Create a Makefile or justfile for common build tasks".into());
        } else if score < 2.0 {
            recs.push("Enhance build automation: Add justfile alongside Makefile for full credit".into());
        }
    }
}

impl Default for BuildPerfScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for BuildPerfScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        self.score_internal(project_path, None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // This scorer doesn't have expensive operations
        self.score(project_path)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut recs = Vec::new();
        self.recommend_lto(project_path, &mut recs);
        self.recommend_cargo_lock(project_path, &mut recs);
        self.recommend_cargo_config(project_path, &mut recs);
        self.recommend_codegen_units(project_path, &mut recs);
        self.recommend_build_system(project_path, &mut recs);
        recs
    }
}

// SAFETY: BuildPerfScorer holds only a PathBuf (owned, Send+Sync) and no interior mutability,
// making it safe to send between and share across threads for parallel scoring.
unsafe impl Send for BuildPerfScorer {}
unsafe impl Sync for BuildPerfScorer {}

/// Calculate directory size recursively (best-effort)
fn dir_size(path: &Path) -> std::io::Result<u64> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut total = 0u64;

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path).unwrap_or(0);
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }

    Ok(total)
}

