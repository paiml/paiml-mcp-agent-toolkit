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
    #[must_use]
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
    #[must_use]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ========================================================================
    // MetaFileType Tests
    // ========================================================================

    #[test]
    fn test_meta_file_type_variants() {
        let makefile = MetaFileType::Makefile;
        let readme = MetaFileType::Readme;

        assert_ne!(makefile, readme);
        assert_eq!(makefile, MetaFileType::Makefile);
        assert_eq!(readme, MetaFileType::Readme);
    }

    #[test]
    fn test_meta_file_type_clone() {
        let original = MetaFileType::Makefile;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_meta_file_type_serialization() {
        let makefile = MetaFileType::Makefile;
        let json = serde_json::to_string(&makefile).unwrap();
        assert!(json.contains("Makefile"));

        let deserialized: MetaFileType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, makefile);
    }

    // ========================================================================
    // MetaFile Tests
    // ========================================================================

    #[test]
    fn test_meta_file_creation() {
        let meta_file = MetaFile {
            path: PathBuf::from("./Makefile"),
            file_type: MetaFileType::Makefile,
            content: "all: build".to_string(),
        };

        assert_eq!(meta_file.path, PathBuf::from("./Makefile"));
        assert_eq!(meta_file.file_type, MetaFileType::Makefile);
        assert_eq!(meta_file.content, "all: build");
    }

    #[test]
    fn test_meta_file_serialization() {
        let meta_file = MetaFile {
            path: PathBuf::from("./README.md"),
            file_type: MetaFileType::Readme,
            content: "# Project Title".to_string(),
        };

        let json = serde_json::to_string(&meta_file).unwrap();
        let deserialized: MetaFile = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, meta_file.path);
        assert_eq!(deserialized.file_type, meta_file.file_type);
        assert_eq!(deserialized.content, meta_file.content);
    }

    // ========================================================================
    // MakeTarget Tests
    // ========================================================================

    #[test]
    fn test_make_target_creation() {
        let target = MakeTarget {
            name: "build".to_string(),
            deps: vec!["clean".to_string(), "prepare".to_string()],
            recipe_summary: "cargo build --release".to_string(),
        };

        assert_eq!(target.name, "build");
        assert_eq!(target.deps.len(), 2);
        assert!(target.deps.contains(&"clean".to_string()));
    }

    #[test]
    fn test_make_target_empty_deps() {
        let target = MakeTarget {
            name: "clean".to_string(),
            deps: vec![],
            recipe_summary: "rm -rf target/".to_string(),
        };

        assert!(target.deps.is_empty());
    }

    // ========================================================================
    // CompressedMakefile Tests
    // ========================================================================

    #[test]
    fn test_compressed_makefile_default() {
        let compressed = CompressedMakefile::default();

        assert!(compressed.variables.is_empty());
        assert!(compressed.targets.is_empty());
        assert!(compressed.detected_toolchain.is_none());
        assert!(compressed.key_dependencies.is_empty());
    }

    #[test]
    fn test_compressed_makefile_with_data() {
        let compressed = CompressedMakefile {
            variables: vec!["CC=gcc".to_string(), "CFLAGS=-Wall".to_string()],
            targets: vec![MakeTarget {
                name: "all".to_string(),
                deps: vec!["build".to_string()],
                recipe_summary: "Build all".to_string(),
            }],
            detected_toolchain: Some("rust".to_string()),
            key_dependencies: vec!["serde".to_string(), "tokio".to_string()],
        };

        assert_eq!(compressed.variables.len(), 2);
        assert_eq!(compressed.targets.len(), 1);
        assert_eq!(compressed.detected_toolchain, Some("rust".to_string()));
        assert_eq!(compressed.key_dependencies.len(), 2);
    }

    // ========================================================================
    // BuildInfo Tests
    // ========================================================================

    #[test]
    fn test_build_info_from_makefile_with_all() {
        let compressed = CompressedMakefile {
            variables: vec![],
            targets: vec![MakeTarget {
                name: "all".to_string(),
                deps: vec!["build".to_string()],
                recipe_summary: "cargo build --release".to_string(),
            }],
            detected_toolchain: Some("rust".to_string()),
            key_dependencies: vec!["serde".to_string()],
        };

        let build_info = BuildInfo::from_makefile(compressed);

        assert_eq!(build_info.toolchain, "rust");
        assert_eq!(build_info.targets, vec!["all"]);
        assert_eq!(build_info.dependencies, vec!["serde"]);
        assert_eq!(
            build_info.primary_command,
            Some("cargo build --release".to_string())
        );
    }

    #[test]
    fn test_build_info_from_makefile_with_build() {
        let compressed = CompressedMakefile {
            variables: vec![],
            targets: vec![
                MakeTarget {
                    name: "clean".to_string(),
                    deps: vec![],
                    recipe_summary: "rm -rf target/".to_string(),
                },
                MakeTarget {
                    name: "build".to_string(),
                    deps: vec!["clean".to_string()],
                    recipe_summary: "cargo build".to_string(),
                },
            ],
            detected_toolchain: Some("rust".to_string()),
            key_dependencies: vec![],
        };

        let build_info = BuildInfo::from_makefile(compressed);

        // Should find "build" as primary command
        assert_eq!(build_info.primary_command, Some("cargo build".to_string()));
    }

    #[test]
    fn test_build_info_from_makefile_unknown_toolchain() {
        let compressed = CompressedMakefile {
            variables: vec![],
            targets: vec![],
            detected_toolchain: None,
            key_dependencies: vec![],
        };

        let build_info = BuildInfo::from_makefile(compressed);

        assert_eq!(build_info.toolchain, "unknown");
        assert!(build_info.targets.is_empty());
        assert!(build_info.primary_command.is_none());
    }

    #[test]
    fn test_build_info_no_primary_command() {
        let compressed = CompressedMakefile {
            variables: vec![],
            targets: vec![
                MakeTarget {
                    name: "clean".to_string(),
                    deps: vec![],
                    recipe_summary: "rm -rf target/".to_string(),
                },
                MakeTarget {
                    name: "test".to_string(),
                    deps: vec![],
                    recipe_summary: "cargo test".to_string(),
                },
            ],
            detected_toolchain: Some("rust".to_string()),
            key_dependencies: vec![],
        };

        let build_info = BuildInfo::from_makefile(compressed);

        // No "all" or "build" target, so no primary command
        assert!(build_info.primary_command.is_none());
    }

    // ========================================================================
    // CompressedSection Tests
    // ========================================================================

    #[test]
    fn test_compressed_section() {
        let section = CompressedSection {
            title: "Installation".to_string(),
            content: "Run npm install".to_string(),
        };

        assert_eq!(section.title, "Installation");
        assert_eq!(section.content, "Run npm install");
    }

    // ========================================================================
    // CompressedReadme Tests
    // ========================================================================

    #[test]
    fn test_compressed_readme_default() {
        let readme = CompressedReadme::default();

        assert!(readme.sections.is_empty());
        assert!(readme.project_description.is_none());
        assert!(readme.key_features.is_empty());
    }

    #[test]
    fn test_compressed_readme_to_summary_basic() {
        let readme = CompressedReadme {
            sections: vec![],
            project_description: Some("A test project".to_string()),
            key_features: vec!["Fast".to_string(), "Reliable".to_string()],
        };

        let summary = readme.to_summary();

        assert_eq!(summary.compressed_description, "A test project");
        assert_eq!(summary.key_features, vec!["Fast", "Reliable"]);
        assert!(summary.architecture_summary.is_none());
        assert!(summary.api_summary.is_none());
    }

    #[test]
    fn test_compressed_readme_extracts_architecture() {
        let readme = CompressedReadme {
            sections: vec![CompressedSection {
                title: "Architecture Overview".to_string(),
                content: "Uses microservices".to_string(),
            }],
            project_description: None,
            key_features: vec![],
        };

        let summary = readme.to_summary();

        assert_eq!(
            summary.architecture_summary,
            Some("Uses microservices".to_string())
        );
    }

    #[test]
    fn test_compressed_readme_extracts_api() {
        let readme = CompressedReadme {
            sections: vec![CompressedSection {
                title: "API Reference".to_string(),
                content: "GET /users returns list".to_string(),
            }],
            project_description: None,
            key_features: vec![],
        };

        let summary = readme.to_summary();

        assert_eq!(
            summary.api_summary,
            Some("GET /users returns list".to_string())
        );
    }

    #[test]
    fn test_compressed_readme_extracts_interface() {
        let readme = CompressedReadme {
            sections: vec![CompressedSection {
                title: "User Interface".to_string(),
                content: "React-based UI".to_string(),
            }],
            project_description: None,
            key_features: vec![],
        };

        let summary = readme.to_summary();

        assert_eq!(summary.api_summary, Some("React-based UI".to_string()));
    }

    #[test]
    fn test_compressed_readme_multiple_sections() {
        let readme = CompressedReadme {
            sections: vec![
                CompressedSection {
                    title: "Getting Started".to_string(),
                    content: "Install dependencies".to_string(),
                },
                CompressedSection {
                    title: "Architecture".to_string(),
                    content: "MVC pattern".to_string(),
                },
                CompressedSection {
                    title: "API Documentation".to_string(),
                    content: "RESTful API".to_string(),
                },
            ],
            project_description: Some("Full project".to_string()),
            key_features: vec!["Feature 1".to_string()],
        };

        let summary = readme.to_summary();

        assert_eq!(summary.compressed_description, "Full project");
        assert_eq!(
            summary.architecture_summary,
            Some("MVC pattern".to_string())
        );
        assert_eq!(summary.api_summary, Some("RESTful API".to_string()));
    }

    #[test]
    fn test_compressed_readme_first_match_wins() {
        let readme = CompressedReadme {
            sections: vec![
                CompressedSection {
                    title: "System Architecture".to_string(),
                    content: "First architecture".to_string(),
                },
                CompressedSection {
                    title: "Code Architecture".to_string(),
                    content: "Second architecture".to_string(),
                },
            ],
            project_description: None,
            key_features: vec![],
        };

        let summary = readme.to_summary();

        // First match wins
        assert_eq!(
            summary.architecture_summary,
            Some("First architecture".to_string())
        );
    }

    // ========================================================================
    // ProjectOverview Tests
    // ========================================================================

    #[test]
    fn test_project_overview_serialization() {
        let overview = ProjectOverview {
            compressed_description: "Test project".to_string(),
            key_features: vec!["Fast".to_string()],
            architecture_summary: Some("Monolith".to_string()),
            api_summary: None,
        };

        let json = serde_json::to_string(&overview).unwrap();
        let deserialized: ProjectOverview = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.compressed_description,
            overview.compressed_description
        );
        assert_eq!(deserialized.key_features, overview.key_features);
        assert_eq!(
            deserialized.architecture_summary,
            overview.architecture_summary
        );
        assert_eq!(deserialized.api_summary, overview.api_summary);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_make_target_roundtrip(
            name in "[a-z_]+",
            recipe in "[a-z -]+",
        ) {
            let target = MakeTarget {
                name: name.clone(),
                deps: vec![],
                recipe_summary: recipe.clone(),
            };

            let json = serde_json::to_string(&target).unwrap();
            let deserialized: MakeTarget = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.recipe_summary, recipe);
        }

        #[test]
        fn test_compressed_section_roundtrip(
            title in "[A-Za-z ]+",
            content in "[A-Za-z ]+",
        ) {
            let section = CompressedSection {
                title: title.clone(),
                content: content.clone(),
            };

            let json = serde_json::to_string(&section).unwrap();
            let deserialized: CompressedSection = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(deserialized.title, title);
            prop_assert_eq!(deserialized.content, content);
        }

        #[test]
        fn test_build_info_toolchain_fallback(
            toolchain_opt in prop::option::of("[a-z]+"),
        ) {
            let compressed = CompressedMakefile {
                variables: vec![],
                targets: vec![],
                detected_toolchain: toolchain_opt.clone(),
                key_dependencies: vec![],
            };

            let build_info = BuildInfo::from_makefile(compressed);

            match toolchain_opt {
                Some(t) => prop_assert_eq!(build_info.toolchain, t),
                None => prop_assert_eq!(build_info.toolchain, "unknown"),
            }
        }
    }
}
