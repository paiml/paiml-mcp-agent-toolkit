#![cfg_attr(coverage_nightly, coverage(off))]
//! Deployment & Release Scorer (15 points)
//!
//! DR-01 (5pts): Nightly/release workflow exists
//! DR-02 (3pts): Cross-platform builds (>=2 targets in matrix)
//! DR-03 (3pts): Automated release notes (action-gh-release or equivalent)
//! DR-04 (2pts): Published to registry (Cargo.toml [package] with version)
//! DR-05 (2pts): Semantic versioning

use super::{read_cargo_toml, read_workflow_files, InfraScorer};
use crate::services::infra_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct DeploymentReleaseScorer;

impl DeploymentReleaseScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeploymentReleaseScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfraScorer for DeploymentReleaseScorer {
    fn category_name(&self) -> &str {
        "Deployment & Release"
    }

    fn max_score(&self) -> f64 {
        15.0
    }

    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore> {
        let workflows = read_workflow_files(repo_path);
        let all_content: String = workflows
            .iter()
            .map(|(_, c)| c.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let cargo_toml = read_cargo_toml(repo_path);

        let mut checks = Vec::new();
        let mut findings = Vec::new();

        // DR-01 (5pts): Nightly/release workflow
        let dr01 = check_release_workflow(&workflows);
        if !dr01.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "DR-01".to_string(),
                message: "No nightly or release workflow found.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(dr01);

        // DR-02 (3pts): Cross-platform builds
        let dr02 = check_cross_platform(&all_content);
        if !dr02.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "DR-02".to_string(),
                message: "No cross-platform matrix found (need >=2 targets).".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(dr02);

        // DR-03 (3pts): Automated release notes
        let dr03 = check_release_automation(&all_content);
        if !dr03.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "DR-03".to_string(),
                message: "No automated release tool found (action-gh-release, etc.).".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(dr03);

        // DR-04 (2pts): Published to registry
        let dr04 = check_registry_publishing(cargo_toml.as_deref());
        if !dr04.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "DR-04".to_string(),
                message: "No Cargo.toml [package] with version found.".to_string(),
                location: Some("Cargo.toml".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(dr04);

        // DR-05 (2pts): Semantic versioning
        let dr05 = check_semver(cargo_toml.as_deref());
        if !dr05.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "DR-05".to_string(),
                message: "Version does not follow semver (x.y.z).".to_string(),
                location: Some("Cargo.toml".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(dr05);

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// DR-01: Check for nightly/release workflow
fn check_release_workflow(workflows: &[(String, String)]) -> InfraCheck {
    let release_names = ["nightly", "release", "deploy", "publish"];

    for (name, content) in workflows {
        let name_lower = name.to_lowercase();
        for pattern in &release_names {
            if name_lower.contains(pattern)
                || content
                    .to_lowercase()
                    .contains(&format!("name: {}", pattern))
            {
                return InfraCheck::pass(
                    "DR-01",
                    "Release workflow",
                    5.0,
                    vec![format!("Found release workflow: {}", name)],
                );
            }
        }
        // Also check for schedule trigger (nightly builds)
        if content.contains("schedule:") || content.contains("cron:") {
            return InfraCheck::pass(
                "DR-01",
                "Release workflow",
                5.0,
                vec![format!("Found scheduled workflow: {}", name)],
            );
        }
    }

    InfraCheck::fail(
        "DR-01",
        "Release workflow",
        5.0,
        vec!["No nightly/release workflow found".to_string()],
    )
}

/// DR-02: Cross-platform builds (>=2 targets in matrix)
fn check_cross_platform(content: &str) -> InfraCheck {
    // Count distinct platform indicators
    let mut targets = 0u32;

    let platform_indicators = [
        "ubuntu",
        "linux",
        "x86_64-unknown-linux",
        "macos",
        "darwin",
        "x86_64-apple",
        "aarch64-apple",
        "windows",
        "x86_64-pc-windows",
    ];

    let content_lower = content.to_lowercase();
    let mut seen_platforms: Vec<&str> = Vec::new();
    for indicator in &platform_indicators {
        if content_lower.contains(indicator) {
            // Group by OS family
            let family = if indicator.contains("linux") || indicator.contains("ubuntu") {
                "linux"
            } else if indicator.contains("macos")
                || indicator.contains("darwin")
                || indicator.contains("apple")
            {
                "macos"
            } else {
                "windows"
            };
            if !seen_platforms.contains(&family) {
                seen_platforms.push(family);
                targets += 1;
            }
        }
    }

    // Also check for matrix strategy
    let has_matrix = content.contains("matrix:");

    if targets >= 2 || (has_matrix && targets >= 1) {
        InfraCheck::pass(
            "DR-02",
            "Cross-platform builds",
            3.0,
            vec![format!(
                "Found {} platform targets: {:?}",
                targets, seen_platforms
            )],
        )
    } else {
        InfraCheck::fail(
            "DR-02",
            "Cross-platform builds",
            3.0,
            vec![format!(
                "Only {} platform target(s) found, need >=2",
                targets
            )],
        )
    }
}

/// DR-03: Automated release notes (action-gh-release, etc.)
fn check_release_automation(content: &str) -> InfraCheck {
    let release_actions = [
        "action-gh-release",
        "softprops/action-gh-release",
        "ncipollo/release-action",
        "release-drafter",
        "semantic-release",
        "cargo publish",
        "npm publish",
    ];

    for pattern in &release_actions {
        if content.contains(pattern) {
            return InfraCheck::pass(
                "DR-03",
                "Release automation",
                3.0,
                vec![format!("Found release automation: {}", pattern)],
            );
        }
    }

    InfraCheck::fail(
        "DR-03",
        "Release automation",
        3.0,
        vec!["No release automation found".to_string()],
    )
}

/// DR-04: Published to registry (Cargo.toml [package] with version)
fn check_registry_publishing(cargo_toml: Option<&str>) -> InfraCheck {
    if let Some(content) = cargo_toml {
        let has_package = content.contains("[package]");
        let has_version = content.lines().any(|l| {
            let t = l.trim();
            t.starts_with("version") && t.contains('=')
        });

        if has_package && has_version {
            return InfraCheck::pass(
                "DR-04",
                "Registry publishing",
                2.0,
                vec!["Found [package] with version in Cargo.toml".to_string()],
            );
        }
    }

    InfraCheck::fail(
        "DR-04",
        "Registry publishing",
        2.0,
        vec!["No Cargo.toml [package] with version found".to_string()],
    )
}

/// Result of searching Cargo.toml for a version declaration.
enum VersionFound {
    /// A direct version string like `version = "1.2.3"`.
    Direct(String),
    /// Version inherited from workspace (`version.workspace = true`).
    Workspace,
    /// No version declaration found.
    None,
}

/// Scan Cargo.toml content for a version declaration.
fn find_version_declaration(content: &str) -> VersionFound {
    // First pass: look for a direct version assignment (not workspace-inherited).
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("version") || !trimmed.contains('=') {
            continue;
        }
        if trimmed.contains(".workspace") {
            continue;
        }
        if let Some(version_str) = extract_version_string(trimmed) {
            return VersionFound::Direct(version_str);
        }
    }

    // Second pass: check for workspace inheritance.
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") && trimmed.contains(".workspace") {
            return VersionFound::Workspace;
        }
    }

    VersionFound::None
}

/// DR-05: Semantic versioning (x.y.z pattern)
fn check_semver(cargo_toml: Option<&str>) -> InfraCheck {
    let Some(content) = cargo_toml else {
        return semver_fail("No version found");
    };

    match find_version_declaration(content) {
        VersionFound::Direct(ref v) if is_semver(v) => InfraCheck::pass(
            "DR-05",
            "Semantic versioning",
            2.0,
            vec![format!("Version {} follows semver", v)],
        ),
        VersionFound::Direct(ref v) => {
            semver_fail(&format!("Version {} does not follow semver (x.y.z)", v))
        }
        VersionFound::Workspace => InfraCheck::pass(
            "DR-05",
            "Semantic versioning",
            2.0,
            vec!["Version inherited from workspace (version.workspace = true)".to_string()],
        ),
        VersionFound::None => semver_fail("No version found"),
    }
}

/// Helper to construct a DR-05 failure.
fn semver_fail(reason: &str) -> InfraCheck {
    InfraCheck::fail(
        "DR-05",
        "Semantic versioning",
        2.0,
        vec![reason.to_string()],
    )
}

/// Extract version string from a TOML line like `version = "1.2.3"`
fn extract_version_string(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        let val = parts[1].trim().trim_matches('"').trim_matches('\'');
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

/// Check if a version string follows semver (x.y.z with optional pre-release)
fn is_semver(version: &str) -> bool {
    let parts: Vec<&str> = version
        .split('-')
        .next()
        .unwrap_or(version)
        .split('.')
        .collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_empty_repo() {
        let tmp = TempDir::new().unwrap();
        let scorer = DeploymentReleaseScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn test_dr01_nightly_workflow() {
        let workflows = vec![(
            "nightly.yml".to_string(),
            "name: Nightly\non: schedule".to_string(),
        )];
        let check = check_release_workflow(&workflows);
        assert!(check.passed);
    }

    #[test]
    fn test_dr01_schedule_trigger() {
        let workflows = vec![(
            "build.yml".to_string(),
            "on:\n  schedule:\n    - cron: '0 4 * * *'".to_string(),
        )];
        let check = check_release_workflow(&workflows);
        assert!(check.passed);
    }

    #[test]
    fn test_dr01_no_release() {
        let workflows = vec![("ci.yml".to_string(), "on: push\njobs:\n  test:".to_string())];
        let check = check_release_workflow(&workflows);
        assert!(!check.passed);
    }

    #[test]
    fn test_dr02_cross_platform_pass() {
        let content = "matrix:\n  os: [ubuntu-latest, macos-latest, windows-latest]";
        let check = check_cross_platform(content);
        assert!(check.passed);
    }

    #[test]
    fn test_dr02_single_platform() {
        let content = "runs-on: ubuntu-latest";
        let check = check_cross_platform(content);
        assert!(!check.passed);
    }

    #[test]
    fn test_dr02_rust_targets() {
        let content =
            "target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc]";
        let check = check_cross_platform(content);
        assert!(check.passed);
    }

    #[test]
    fn test_dr03_gh_release_pass() {
        let check = check_release_automation("- uses: softprops/action-gh-release@v2");
        assert!(check.passed);
    }

    #[test]
    fn test_dr03_cargo_publish_pass() {
        let check = check_release_automation("run: cargo publish");
        assert!(check.passed);
    }

    #[test]
    fn test_dr03_no_release_automation() {
        let check = check_release_automation("run: cargo build");
        assert!(!check.passed);
    }

    #[test]
    fn test_dr04_package_with_version() {
        let toml = "[package]\nname = \"my-crate\"\nversion = \"1.2.3\"";
        let check = check_registry_publishing(Some(toml));
        assert!(check.passed);
    }

    #[test]
    fn test_dr04_no_package() {
        let check = check_registry_publishing(Some("[dependencies]\nserde = \"1.0\""));
        assert!(!check.passed);
    }

    #[test]
    fn test_dr04_no_cargo_toml() {
        let check = check_registry_publishing(None);
        assert!(!check.passed);
    }

    #[test]
    fn test_dr05_semver_pass() {
        let toml = "[package]\nversion = \"1.2.3\"";
        let check = check_semver(Some(toml));
        assert!(check.passed);
    }

    #[test]
    fn test_dr05_semver_prerelease() {
        let toml = "[package]\nversion = \"0.1.0-alpha\"";
        let check = check_semver(Some(toml));
        assert!(check.passed);
    }

    #[test]
    fn test_dr05_no_version() {
        let check = check_semver(None);
        assert!(!check.passed);
    }

    #[test]
    fn test_dr05_workspace_inherited_version() {
        let toml = "[package]\nname = \"my-crate\"\nversion.workspace = true";
        let check = check_semver(Some(toml));
        assert!(
            check.passed,
            "workspace-inherited version should pass DR-05"
        );
        assert!(
            check.evidence[0].contains("workspace"),
            "should mention workspace inheritance"
        );
    }

    #[test]
    fn test_extract_version_string() {
        assert_eq!(
            extract_version_string("version = \"1.2.3\""),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            extract_version_string("version = \"0.1.0-alpha\""),
            Some("0.1.0-alpha".to_string())
        );
    }

    #[test]
    fn test_is_semver() {
        assert!(is_semver("1.2.3"));
        assert!(is_semver("0.1.0"));
        assert!(is_semver("0.1.0-alpha"));
        assert!(is_semver("1.0"));
        assert!(!is_semver("abc"));
        assert!(!is_semver("1.2.3.4"));
    }

    #[tokio::test]
    async fn test_perfect_deployment() {
        let tmp = TempDir::new().unwrap();
        let wf_dir = tmp.path().join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        fs::write(
            wf_dir.join("nightly.yml"),
            r#"name: Nightly
on:
  schedule:
    - cron: '0 4 * * *'
jobs:
  build:
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc]
    runs-on: ubuntu-latest
    steps:
      - uses: softprops/action-gh-release@v2
"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.2.3\"",
        )
        .unwrap();

        let scorer = DeploymentReleaseScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 15.0).abs() < f64::EPSILON);
    }
}
