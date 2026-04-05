//! Asset layout validation service (R-10 remediation).
//!
//! Provides a clean API for validating non-code assets (README, Dockerfile,
//! SVG, CHANGELOG, badges, mdBook, forjar). Used by `pmat comply asset-validate`
//! and CB-1320..1326 checks.
//!
//! Spec: commit-level-contract-enforcement.md Phase 3, Asset Layout Contracts.

use std::path::Path;

/// Result of validating a single asset.
#[derive(Debug, Clone)]
pub struct AssetValidationResult {
    pub asset_type: AssetType,
    pub name: String,
    pub status: AssetStatus,
    pub message: String,
    pub issues: Vec<String>,
}

/// Supported asset types for layout contract validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Readme,
    Dockerfile,
    Svg,
    Changelog,
    Badges,
    MdBook,
    Forjar,
}

impl AssetType {
    /// CB check ID for this asset type.
    pub fn cb_id(&self) -> &'static str {
        match self {
            Self::Readme => "CB-1320",
            Self::Dockerfile => "CB-1321",
            Self::Svg => "CB-1322",
            Self::Forjar => "CB-1323",
            Self::MdBook => "CB-1324",
            Self::Changelog => "CB-1325",
            Self::Badges => "CB-1326",
        }
    }

    /// All asset types in spec order.
    pub fn all() -> &'static [AssetType] {
        &[
            Self::Readme,
            Self::Dockerfile,
            Self::Svg,
            Self::Forjar,
            Self::MdBook,
            Self::Changelog,
            Self::Badges,
        ]
    }

    /// Parse from string (for --asset CLI flag).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "readme" => Some(Self::Readme),
            "dockerfile" => Some(Self::Dockerfile),
            "svg" => Some(Self::Svg),
            "changelog" => Some(Self::Changelog),
            "badges" => Some(Self::Badges),
            "book" | "mdbook" => Some(Self::MdBook),
            "forjar" => Some(Self::Forjar),
            _ => None,
        }
    }
}

/// Status of asset validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetStatus {
    Pass,
    Warn,
    Skip,
}

/// Validate all assets in a project and return results.
pub fn validate_all_assets(project_path: &Path) -> Vec<AssetValidationResult> {
    AssetType::all()
        .iter()
        .map(|t| validate_asset(project_path, *t))
        .collect()
}

/// Validate a specific asset type.
pub fn validate_asset(project_path: &Path, asset_type: AssetType) -> AssetValidationResult {
    match asset_type {
        AssetType::Readme => validate_readme(project_path),
        AssetType::Dockerfile => validate_dockerfile(project_path),
        AssetType::Svg => validate_svg(project_path),
        AssetType::Changelog => validate_changelog(project_path),
        AssetType::Badges => validate_badges(project_path),
        AssetType::MdBook => validate_mdbook(project_path),
        AssetType::Forjar => validate_forjar(project_path),
    }
}

fn validate_readme(project_path: &Path) -> AssetValidationResult {
    let readme = project_path.join("README.md");
    if !readme.exists() {
        return AssetValidationResult {
            asset_type: AssetType::Readme,
            name: "README.md".into(),
            status: AssetStatus::Warn,
            message: "No README.md found".into(),
            issues: vec!["missing".into()],
        };
    }
    let content = std::fs::read_to_string(&readme).unwrap_or_default();
    let mut issues = Vec::new();

    if !content.lines().any(|l| l.starts_with("# ")) {
        issues.push("missing title (# heading)".into());
    }
    let lower = content.to_lowercase();
    for (name, patterns) in &[
        ("install", &["install", "getting started", "setup"][..]),
        ("usage", &["usage", "examples"][..]),
        ("license", &["license"][..]),
    ] {
        if !patterns
            .iter()
            .any(|p| lower.contains(&format!("## {}", p)))
        {
            issues.push(format!("missing section: {}", name));
        }
    }

    if issues.is_empty() {
        AssetValidationResult {
            asset_type: AssetType::Readme,
            name: "README.md".into(),
            status: AssetStatus::Pass,
            message: "README.md has required sections".into(),
            issues: vec![],
        }
    } else {
        AssetValidationResult {
            asset_type: AssetType::Readme,
            name: "README.md".into(),
            status: AssetStatus::Warn,
            message: format!("{} issue(s)", issues.len()),
            issues,
        }
    }
}

fn validate_dockerfile(project_path: &Path) -> AssetValidationResult {
    let dockerfile = project_path.join("Dockerfile");
    if !dockerfile.exists() {
        return AssetValidationResult {
            asset_type: AssetType::Dockerfile,
            name: "Dockerfile".into(),
            status: AssetStatus::Skip,
            message: "No Dockerfile found".into(),
            issues: vec![],
        };
    }
    let content = std::fs::read_to_string(&dockerfile).unwrap_or_default();
    let mut issues = Vec::new();
    if content.contains(":latest") {
        issues.push(":latest tag used".into());
    }
    if content.contains("curl") && content.contains("bash") {
        issues.push("curl|bash pattern detected".into());
    }

    AssetValidationResult {
        asset_type: AssetType::Dockerfile,
        name: "Dockerfile".into(),
        status: if issues.is_empty() {
            AssetStatus::Pass
        } else {
            AssetStatus::Warn
        },
        message: if issues.is_empty() {
            "Dockerfile passes security checks".into()
        } else {
            format!("{} issue(s)", issues.len())
        },
        issues,
    }
}

fn validate_svg(project_path: &Path) -> AssetValidationResult {
    let search_dirs = ["assets", "docs", "static", "."];
    let mut count = 0usize;
    let mut issues = Vec::new();

    for dir_name in &search_dirs {
        let dir = project_path.join(dir_name);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(true, |e| e != "svg") || !path.is_file() {
                    continue;
                }
                count += 1;
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if !content.contains("viewBox") {
                        issues.push(format!("{}: missing viewBox", name));
                    }
                    if !content.contains("<title") && !content.contains("aria-label") {
                        issues.push(format!("{}: no accessibility", name));
                    }
                }
            }
        }
    }

    if count == 0 {
        AssetValidationResult {
            asset_type: AssetType::Svg,
            name: "SVG".into(),
            status: AssetStatus::Skip,
            message: "No SVG files found".into(),
            issues: vec![],
        }
    } else {
        AssetValidationResult {
            asset_type: AssetType::Svg,
            name: "SVG".into(),
            status: if issues.is_empty() {
                AssetStatus::Pass
            } else {
                AssetStatus::Warn
            },
            message: format!("{} file(s), {} issue(s)", count, issues.len()),
            issues,
        }
    }
}

fn validate_changelog(project_path: &Path) -> AssetValidationResult {
    let path = project_path.join("CHANGELOG.md");
    if !path.exists() {
        return AssetValidationResult {
            asset_type: AssetType::Changelog,
            name: "CHANGELOG.md".into(),
            status: AssetStatus::Skip,
            message: "No CHANGELOG.md".into(),
            issues: vec![],
        };
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let lower = content.to_lowercase();
    let has_format = lower.contains("## [") || lower.contains("# changelog");

    AssetValidationResult {
        asset_type: AssetType::Changelog,
        name: "CHANGELOG.md".into(),
        status: if has_format {
            AssetStatus::Pass
        } else {
            AssetStatus::Warn
        },
        message: if has_format {
            "Keep-a-Changelog format".into()
        } else {
            "Missing version headings".into()
        },
        issues: if has_format {
            vec![]
        } else {
            vec!["no version headings".into()]
        },
    }
}

fn validate_badges(project_path: &Path) -> AssetValidationResult {
    let readme = project_path.join("README.md");
    if !readme.exists() {
        return AssetValidationResult {
            asset_type: AssetType::Badges,
            name: "Badges".into(),
            status: AssetStatus::Skip,
            message: "No README.md".into(),
            issues: vec![],
        };
    }
    let content = std::fs::read_to_string(&readme).unwrap_or_default();
    let mut missing = Vec::new();
    // Match the same patterns as CB-1326 inline check
    let ci_patterns = [
        "actions/workflows",
        "github.com/",
        "ci.svg",
        "build.svg",
        "passing",
        "ci.yml",
    ];
    if !ci_patterns.iter().any(|p| content.contains(p)) {
        missing.push("CI status".into());
    }
    let version_patterns = ["crates.io", "version", "crate-", "crate"];
    if !version_patterns.iter().any(|p| content.contains(p)) {
        missing.push("version/crate".into());
    }
    if !content.to_lowercase().contains("license") {
        missing.push("license".into());
    }

    AssetValidationResult {
        asset_type: AssetType::Badges,
        name: "Badges".into(),
        status: if missing.is_empty() {
            AssetStatus::Pass
        } else {
            AssetStatus::Warn
        },
        message: if missing.is_empty() {
            "Required badges present".into()
        } else {
            format!("Missing: {}", missing.join(", "))
        },
        issues: missing,
    }
}

fn validate_mdbook(project_path: &Path) -> AssetValidationResult {
    let summary = project_path.join("book/src/SUMMARY.md");
    if !summary.exists() {
        return AssetValidationResult {
            asset_type: AssetType::MdBook,
            name: "mdBook".into(),
            status: AssetStatus::Skip,
            message: "No book/src/SUMMARY.md".into(),
            issues: vec![],
        };
    }
    let content = std::fs::read_to_string(&summary).unwrap_or_default();
    let book_src = project_path.join("book/src");
    let mut broken = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("](") {
            if let Some(end) = line[start + 2..].find(')') {
                let link = &line[start + 2..start + 2 + end];
                if link.starts_with("http") || link.starts_with('#') {
                    continue;
                }
                let link_path = book_src.join(link.split('#').next().unwrap_or(link));
                if !link_path.exists() {
                    broken.push(link.to_string());
                }
            }
        }
    }

    AssetValidationResult {
        asset_type: AssetType::MdBook,
        name: "mdBook".into(),
        status: if broken.is_empty() {
            AssetStatus::Pass
        } else {
            AssetStatus::Warn
        },
        message: if broken.is_empty() {
            "SUMMARY.md links valid".into()
        } else {
            format!("{} broken link(s)", broken.len())
        },
        issues: broken,
    }
}

fn validate_forjar(project_path: &Path) -> AssetValidationResult {
    let yaml = project_path.join("forjar.yaml");
    let toml = project_path.join("forjar.toml");
    let path = if yaml.exists() {
        yaml
    } else if toml.exists() {
        toml
    } else {
        return AssetValidationResult {
            asset_type: AssetType::Forjar,
            name: "forjar".into(),
            status: AssetStatus::Skip,
            message: "No forjar.yaml or forjar.toml".into(),
            issues: vec![],
        };
    };

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut issues = Vec::new();
    let secret_patterns = ["password:", "secret:", "api_key:", "token:"];
    for pattern in &secret_patterns {
        if content.contains(pattern) && !content.contains("${") && !content.contains("env(") {
            issues.push(format!(
                "possible plaintext {}",
                pattern.trim_end_matches(':')
            ));
        }
    }

    AssetValidationResult {
        asset_type: AssetType::Forjar,
        name: "forjar".into(),
        status: if issues.is_empty() {
            AssetStatus::Pass
        } else {
            AssetStatus::Warn
        },
        message: if issues.is_empty() {
            "Forjar config passes checks".into()
        } else {
            format!("{} issue(s)", issues.len())
        },
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_all_empty_project() {
        let dir = tempdir().unwrap();
        let results = validate_all_assets(dir.path());
        assert_eq!(results.len(), 7);
        // Most should be Skip or Warn (no files)
        let skips = results
            .iter()
            .filter(|r| r.status == AssetStatus::Skip)
            .count();
        assert!(skips >= 4); // SVG, book, forjar, changelog should skip
    }

    #[test]
    fn test_validate_readme_with_sections() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("README.md"),
            "# Project\n\n## Installation\n\n## Usage\n\n## License\nMIT\n",
        )
        .unwrap();
        let result = validate_asset(dir.path(), AssetType::Readme);
        assert_eq!(result.status, AssetStatus::Pass);
    }

    #[test]
    fn test_validate_dockerfile_latest() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Dockerfile"), "FROM ubuntu:latest\n").unwrap();
        let result = validate_asset(dir.path(), AssetType::Dockerfile);
        assert_eq!(result.status, AssetStatus::Warn);
        assert!(result.issues.iter().any(|i| i.contains(":latest")));
    }

    #[test]
    fn test_asset_type_from_str() {
        assert_eq!(AssetType::parse("readme"), Some(AssetType::Readme));
        assert_eq!(AssetType::parse("DOCKERFILE"), Some(AssetType::Dockerfile));
        assert_eq!(AssetType::parse("mdbook"), Some(AssetType::MdBook));
        assert_eq!(AssetType::parse("unknown"), None);
    }

    #[test]
    fn test_asset_type_cb_ids() {
        assert_eq!(AssetType::Readme.cb_id(), "CB-1320");
        assert_eq!(AssetType::Badges.cb_id(), "CB-1326");
    }
}
