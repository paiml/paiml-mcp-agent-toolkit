use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Type of metadata file detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaFileType {
    Makefile,
    Readme,
}

/// Detected metadata file with its content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaFile {
    pub path: PathBuf,
    pub file_type: MetaFileType,
    pub content: String,
}

/// Compressed representation of a Makefile
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompressedMakefile {
    pub variables: Vec<String>,
    pub targets: Vec<MakeTarget>,
    pub detected_toolchain: Option<String>,
    pub key_dependencies: Vec<String>,
}

/// A makefile target with dependencies and recipe summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeTarget {
    pub name: String,
    pub deps: Vec<String>,
    pub recipe_summary: String,
}

/// Compressed representation of a README
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompressedReadme {
    pub sections: Vec<CompressedSection>,
    pub project_description: Option<String>,
    pub key_features: Vec<String>,
}

/// A compressed section from README
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedSection {
    pub title: String,
    pub content: String,
}

/// Build information extracted from Makefile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub toolchain: String,
    pub targets: Vec<String>,
    pub dependencies: Vec<String>,
    pub primary_command: Option<String>,
}

impl BuildInfo {
    pub fn from_makefile(compressed: CompressedMakefile) -> Self {
        Self {
            toolchain: compressed
                .detected_toolchain
                .unwrap_or_else(|| "unknown".to_string()),
            targets: compressed.targets.iter().map(|t| t.name.clone()).collect(),
            dependencies: compressed.key_dependencies,
            primary_command: compressed
                .targets
                .iter()
                .find(|t| t.name == "all" || t.name == "build")
                .map(|t| t.recipe_summary.clone()),
        }
    }
}

/// Project overview extracted from README
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOverview {
    pub compressed_description: String,
    pub key_features: Vec<String>,
    pub architecture_summary: Option<String>,
    pub api_summary: Option<String>,
}

impl CompressedReadme {
    pub fn to_summary(&self) -> ProjectOverview {
        let mut overview = ProjectOverview {
            compressed_description: self.project_description.clone().unwrap_or_default(),
            key_features: self.key_features.clone(),
            architecture_summary: None,
            api_summary: None,
        };

        // Extract architecture and API summaries from sections
        for section in &self.sections {
            let title_lower = section.title.to_lowercase();
            if title_lower.contains("architecture") && overview.architecture_summary.is_none() {
                overview.architecture_summary = Some(section.content.clone());
            } else if (title_lower.contains("api") || title_lower.contains("interface"))
                && overview.api_summary.is_none()
            {
                overview.api_summary = Some(section.content.clone());
            }
        }

        overview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_meta_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
