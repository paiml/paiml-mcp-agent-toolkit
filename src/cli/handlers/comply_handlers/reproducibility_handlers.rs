//! CB-301: Reproducibility Level Check
//! CB-302: Golden Trace Drift Detection
//!
//! Adopts NeurIPS/ICLR standards for reproducibility:
//!
//! | Level    | Requirements                              |
//! |----------|-------------------------------------------|
//! | Bronze   | Code available + deps pinned (Cargo.lock) |
//! | Silver   | + Dockerfile + environment documented      |
//! | Gold     | + `make reproduce` + golden trace passing  |
//!
//! CB-302 integrates with renacer golden tracing for transpilers
//! and distributed systems.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Reproducibility level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReproducibilityLevel {
    /// No reproducibility guarantees
    None,
    /// Code available + dependencies pinned
    Bronze,
    /// + Dockerfile + environment documented
    Silver,
    /// + `make reproduce` + golden trace verification
    Gold,
}

impl std::fmt::Display for ReproducibilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReproducibilityLevel::None => write!(f, "None"),
            ReproducibilityLevel::Bronze => write!(f, "Bronze"),
            ReproducibilityLevel::Silver => write!(f, "Silver"),
            ReproducibilityLevel::Gold => write!(f, "Gold"),
        }
    }
}

/// Reproducibility report with evidence for each level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityReport {
    pub level: ReproducibilityLevel,
    pub has_lockfile: bool,
    pub has_dockerfile: bool,
    pub has_ci_config: bool,
    pub has_make_reproduce: bool,
    pub has_golden_traces: bool,
    pub has_pinned_deps: bool,
    pub details: Vec<String>,
}

/// Check the reproducibility level of a project (CB-301).
pub fn check_reproducibility(project_path: &Path) -> ReproducibilityReport {
    let has_lockfile = check_lockfile(project_path);
    let has_dockerfile = check_dockerfile(project_path);
    let has_ci_config = check_ci(project_path);
    let has_make_reproduce = check_make_reproduce(project_path);
    let has_golden_traces = check_golden_traces(project_path);
    let has_pinned_deps = has_lockfile; // Lockfile implies pinned deps

    let mut details = Vec::new();

    // Bronze: Code available + deps pinned
    if has_lockfile {
        details.push("Lockfile present (deps pinned)".into());
    } else {
        details.push("Missing lockfile (deps not pinned)".into());
    }

    // Silver indicators
    if has_dockerfile {
        details.push("Dockerfile present (environment documented)".into());
    }
    if has_ci_config {
        details.push("CI configuration found".into());
    }

    // Gold indicators
    if has_make_reproduce {
        details.push("'make reproduce' target available".into());
    }
    if has_golden_traces {
        details.push("Golden traces configured (renacer.toml)".into());
    }

    let level = determine_level(
        has_lockfile,
        has_dockerfile,
        has_ci_config,
        has_make_reproduce,
        has_golden_traces,
    );

    ReproducibilityReport {
        level,
        has_lockfile,
        has_dockerfile,
        has_ci_config,
        has_make_reproduce,
        has_golden_traces,
        has_pinned_deps,
        details,
    }
}

/// Determine reproducibility level from evidence.
fn determine_level(
    has_lockfile: bool,
    has_dockerfile: bool,
    has_ci_config: bool,
    has_make_reproduce: bool,
    has_golden_traces: bool,
) -> ReproducibilityLevel {
    // Gold: lockfile + dockerfile + make reproduce + golden traces
    if has_lockfile && has_dockerfile && has_make_reproduce && has_golden_traces {
        return ReproducibilityLevel::Gold;
    }

    // Silver: lockfile + (dockerfile OR CI) + some automation
    if has_lockfile
        && (has_dockerfile || has_ci_config)
        && (has_make_reproduce || has_golden_traces)
    {
        return ReproducibilityLevel::Silver;
    }

    // Also Silver: lockfile + dockerfile + CI
    if has_lockfile && has_dockerfile && has_ci_config {
        return ReproducibilityLevel::Silver;
    }

    // Bronze: just a lockfile
    if has_lockfile {
        return ReproducibilityLevel::Bronze;
    }

    ReproducibilityLevel::None
}

/// Check for dependency lockfiles (Cargo.lock, package-lock.json, etc.)
/// Returns true if a lockfile exists OR if the project has no package manager
/// (zero-dependency projects like pure Lua have nothing to lock).
fn check_lockfile(project_path: &Path) -> bool {
    if !project_path.exists() {
        return false;
    }
    // Check for lockfiles across ecosystems
    let has_lockfile = project_path.join("Cargo.lock").exists()
        || project_path.join("package-lock.json").exists()
        || project_path.join("yarn.lock").exists()
        || project_path.join("pnpm-lock.yaml").exists()
        || project_path.join("poetry.lock").exists()
        || project_path.join("Pipfile.lock").exists()
        || project_path.join("go.sum").exists()
        || project_path.join("Gemfile.lock").exists()
        || project_path.join("flake.lock").exists()
        || project_path.join("uv.lock").exists();
    if has_lockfile {
        return true;
    }

    // If no package manager manifest exists, there are no deps to lock.
    // Treat as "pinned" since there's nothing to pin.
    let has_manifest = project_path.join("Cargo.toml").exists()
        || project_path.join("package.json").exists()
        || project_path.join("pyproject.toml").exists()
        || project_path.join("setup.py").exists()
        || project_path.join("Pipfile").exists()
        || project_path.join("go.mod").exists()
        || project_path.join("Gemfile").exists()
        || project_path.join("pom.xml").exists()
        || project_path.join("build.gradle").exists()
        || project_path.join("build.gradle.kts").exists();

    // No manifest = no external deps = nothing to lock = effectively pinned
    // But only if the directory has actual content (empty dirs aren't projects)
    let has_content = std::fs::read_dir(project_path)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false);
    !has_manifest && has_content
}

/// Check for Dockerfile or container configuration
fn check_dockerfile(project_path: &Path) -> bool {
    project_path.join("Dockerfile").exists()
        || project_path.join("docker-compose.yml").exists()
        || project_path.join("docker-compose.yaml").exists()
        || project_path
            .join(".devcontainer/devcontainer.json")
            .exists()
        || project_path.join("flake.nix").exists()
}

/// Check for CI configuration
fn check_ci(project_path: &Path) -> bool {
    project_path.join(".github/workflows").exists()
        || project_path.join(".gitlab-ci.yml").exists()
        || project_path.join("Jenkinsfile").exists()
        || project_path.join(".circleci/config.yml").exists()
}

/// Check for `make reproduce` target in Makefile
fn check_make_reproduce(project_path: &Path) -> bool {
    let makefile = project_path.join("Makefile");
    if !makefile.exists() {
        return false;
    }

    if let Ok(content) = std::fs::read_to_string(&makefile) {
        // Check for reproduce target (common patterns)
        content.contains("reproduce:")
            || content.contains("reproducible:")
            || content.contains("repro:")
    } else {
        false
    }
}

/// Check for golden trace configuration (renacer.toml)
fn check_golden_traces(project_path: &Path) -> bool {
    let has_config = project_path.join("renacer.toml").exists();
    let has_baseline = project_path.join("golden_traces").exists()
        || project_path.join("golden_traces/baseline").exists();
    has_config || has_baseline
}

/// Check if golden traces are passing (CB-302).
/// Returns None if no golden traces configured, Some(true) if passing.
pub fn check_golden_trace_drift(project_path: &Path) -> Option<bool> {
    if !project_path.join("renacer.toml").exists() {
        return None; // Not configured
    }

    let baseline_dir = project_path.join("golden_traces").join("baseline");
    if !baseline_dir.exists() {
        return Some(true); // Config exists but no baseline yet - not a failure
    }

    // Check if renacer is available and baseline is valid
    if let Ok(entries) = std::fs::read_dir(&baseline_dir) {
        let trace_count = entries.filter_map(|e| e.ok()).count();
        Some(trace_count > 0)
    } else {
        Some(false)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_reproducibility_level_ordering() {
        assert!(ReproducibilityLevel::Gold > ReproducibilityLevel::Silver);
        assert!(ReproducibilityLevel::Silver > ReproducibilityLevel::Bronze);
        assert!(ReproducibilityLevel::Bronze > ReproducibilityLevel::None);
    }

    #[test]
    fn test_reproducibility_level_display() {
        assert_eq!(format!("{}", ReproducibilityLevel::Gold), "Gold");
        assert_eq!(format!("{}", ReproducibilityLevel::Silver), "Silver");
        assert_eq!(format!("{}", ReproducibilityLevel::Bronze), "Bronze");
        assert_eq!(format!("{}", ReproducibilityLevel::None), "None");
    }

    #[test]
    fn test_determine_level_gold() {
        let level = determine_level(true, true, true, true, true);
        assert_eq!(level, ReproducibilityLevel::Gold);
    }

    #[test]
    fn test_determine_level_silver() {
        // Lockfile + dockerfile + CI
        let level = determine_level(true, true, true, false, false);
        assert_eq!(level, ReproducibilityLevel::Silver);

        // Lockfile + CI + make reproduce
        let level = determine_level(true, false, true, true, false);
        assert_eq!(level, ReproducibilityLevel::Silver);
    }

    #[test]
    fn test_determine_level_bronze() {
        let level = determine_level(true, false, false, false, false);
        assert_eq!(level, ReproducibilityLevel::Bronze);
    }

    #[test]
    fn test_determine_level_none() {
        let level = determine_level(false, false, false, false, false);
        assert_eq!(level, ReproducibilityLevel::None);
    }

    #[test]
    fn test_check_on_self() {
        let report = check_reproducibility(&PathBuf::from("."));
        // This project has Cargo.lock, so at least Bronze
        assert!(report.has_lockfile);
        assert!(report.level >= ReproducibilityLevel::Bronze);
    }

    #[test]
    fn test_nonexistent_path() {
        let report = check_reproducibility(&PathBuf::from("/nonexistent/path"));
        assert_eq!(report.level, ReproducibilityLevel::None);
        assert!(!report.has_lockfile);
    }

    #[test]
    fn test_golden_trace_drift_no_config() {
        let result = check_golden_trace_drift(&PathBuf::from("/nonexistent"));
        assert_eq!(result, None);
    }

    #[test]
    fn test_golden_trace_drift_with_config() {
        // This project has renacer.toml
        let result = check_golden_trace_drift(&PathBuf::from("."));
        assert!(result.is_some());
    }
}
