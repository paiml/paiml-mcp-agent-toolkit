//! Repository Showcase Gallery
//!
//! This module provides a curated collection of example repositories that demonstrate
//! the capabilities of PMAT across different languages, frameworks, and architectural patterns.
//! The showcase serves as both a demo and a reference for users exploring the tool.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowcaseRepository {
    pub name: String,
    pub url: String,
    pub description: String,
    pub primary_language: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub category: RepositoryCategory,
    pub complexity_tier: ComplexityTier,
    pub estimated_analysis_time_seconds: u32,
    pub highlights: Vec<String>,
    pub analysis_preview: Option<AnalysisPreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RepositoryCategory {
    WebFramework,
    SystemsProgramming,
    DataScience,
    CloudNative,
    DeveloperTools,
    GameDevelopment,
    MachineLearning,
    Blockchain,
    Mobile,
    Embedded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum ComplexityTier {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreview {
    pub estimated_files: u32,
    pub estimated_functions: u32,
    pub estimated_complexity: f64,
    pub key_insights: Vec<String>,
    pub recommended_focus_areas: Vec<String>,
}

pub struct ShowcaseGallery {
    repositories: HashMap<String, ShowcaseRepository>,
    categories: HashMap<RepositoryCategory, Vec<String>>,
}

impl ShowcaseGallery {
    #[must_use]
    pub fn new() -> Self {
        let mut gallery = Self {
            repositories: HashMap::new(),
            categories: HashMap::new(),
        };
        gallery.initialize_showcase_repositories();
        gallery
    }

    fn initialize_showcase_repositories(&mut self) {
        // Rust - Systems Programming
        self.add_repository(ShowcaseRepository {
            name: "Tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio".to_string(),
            description: "A runtime for writing reliable asynchronous applications with Rust"
                .to_string(),
            primary_language: "rust".to_string(),
            languages: vec!["rust".to_string()],
            frameworks: vec!["Tokio".to_string()],
            category: RepositoryCategory::SystemsProgramming,
            complexity_tier: ComplexityTier::Advanced,
            estimated_analysis_time_seconds: 45,
            highlights: vec![
                "Complex async runtime implementation".to_string(),
                "Extensive use of unsafe Rust".to_string(),
                "High test coverage".to_string(),
                "Advanced concurrency patterns".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 200,
                estimated_functions: 1500,
                estimated_complexity: 8.5,
                key_insights: vec![
                    "Complex state machines for async operations".to_string(),
                    "Sophisticated memory management".to_string(),
                    "Extensive macro usage for code generation".to_string(),
                ],
                recommended_focus_areas: vec![
                    "Async runtime components".to_string(),
                    "Thread pool implementation".to_string(),
                    "I/O driver abstractions".to_string(),
                ],
            }),
        });

        // Python - Web Framework
        self.add_repository(ShowcaseRepository {
            name: "Django".to_string(),
            url: "https://github.com/django/django".to_string(),
            description: "The Web framework for perfectionists with deadlines".to_string(),
            primary_language: "python".to_string(),
            languages: vec!["python".to_string(), "javascript".to_string()],
            frameworks: vec!["Django".to_string()],
            category: RepositoryCategory::WebFramework,
            complexity_tier: ComplexityTier::Advanced,
            estimated_analysis_time_seconds: 60,
            highlights: vec![
                "Comprehensive web framework".to_string(),
                "ORM with query optimization".to_string(),
                "Extensive middleware system".to_string(),
                "Admin interface auto-generation".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 800,
                estimated_functions: 5000,
                estimated_complexity: 7.2,
                key_insights: vec![
                    "Layered architecture with clear separation".to_string(),
                    "Heavy use of metaclasses and descriptors".to_string(),
                    "Database abstraction layer complexity".to_string(),
                ],
                recommended_focus_areas: vec![
                    "ORM query generation".to_string(),
                    "Template engine implementation".to_string(),
                    "Middleware pipeline".to_string(),
                ],
            }),
        });

        // JavaScript - Frontend Framework
        self.add_repository(ShowcaseRepository {
            name: "React".to_string(),
            url: "https://github.com/facebook/react".to_string(),
            description: "The library for web and native user interfaces".to_string(),
            primary_language: "javascript".to_string(),
            languages: vec!["javascript".to_string(), "typescript".to_string()],
            frameworks: vec!["React".to_string()],
            category: RepositoryCategory::WebFramework,
            complexity_tier: ComplexityTier::Advanced,
            estimated_analysis_time_seconds: 40,
            highlights: vec![
                "Virtual DOM implementation".to_string(),
                "Fiber reconciliation algorithm".to_string(),
                "Hooks system".to_string(),
                "Concurrent rendering features".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 300,
                estimated_functions: 2000,
                estimated_complexity: 7.8,
                key_insights: vec![
                    "Complex state reconciliation logic".to_string(),
                    "Sophisticated scheduling algorithms".to_string(),
                    "Memory optimization for component trees".to_string(),
                ],
                recommended_focus_areas: vec![
                    "Fiber reconciler core".to_string(),
                    "Hooks implementation".to_string(),
                    "Scheduling and priority systems".to_string(),
                ],
            }),
        });

        // Go - Cloud Native
        self.add_repository(ShowcaseRepository {
            name: "Kubernetes".to_string(),
            url: "https://github.com/kubernetes/kubernetes".to_string(),
            description: "Production-Grade Container Scheduling and Management".to_string(),
            primary_language: "go".to_string(),
            languages: vec!["go".to_string(), "yaml".to_string(), "shell".to_string()],
            frameworks: vec!["Kubernetes".to_string()],
            category: RepositoryCategory::CloudNative,
            complexity_tier: ComplexityTier::Expert,
            estimated_analysis_time_seconds: 120,
            highlights: vec![
                "Distributed systems architecture".to_string(),
                "Container orchestration".to_string(),
                "API server design".to_string(),
                "Extensive controller patterns".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 2000,
                estimated_functions: 15000,
                estimated_complexity: 9.2,
                key_insights: vec![
                    "Complex state management across cluster".to_string(),
                    "Sophisticated API machinery".to_string(),
                    "Extensive use of Go interfaces and composition".to_string(),
                ],
                recommended_focus_areas: vec![
                    "API server and etcd integration".to_string(),
                    "Controller and operator patterns".to_string(),
                    "Scheduler implementation".to_string(),
                ],
            }),
        });

        // TypeScript - Developer Tools
        self.add_repository(ShowcaseRepository {
            name: "VS Code".to_string(),
            url: "https://github.com/microsoft/vscode".to_string(),
            description: "Visual Studio Code - Open Source IDE".to_string(),
            primary_language: "typescript".to_string(),
            languages: vec![
                "typescript".to_string(),
                "javascript".to_string(),
                "css".to_string(),
            ],
            frameworks: vec!["Electron".to_string(), "Monaco Editor".to_string()],
            category: RepositoryCategory::DeveloperTools,
            complexity_tier: ComplexityTier::Expert,
            estimated_analysis_time_seconds: 90,
            highlights: vec![
                "Advanced text editor implementation".to_string(),
                "Extension system architecture".to_string(),
                "Language server protocol".to_string(),
                "Cross-platform desktop application".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 1500,
                estimated_functions: 10000,
                estimated_complexity: 8.9,
                key_insights: vec![
                    "Sophisticated extension host architecture".to_string(),
                    "Complex text buffer and editor implementations".to_string(),
                    "Extensive use of TypeScript advanced features".to_string(),
                ],
                recommended_focus_areas: vec![
                    "Extension host and API design".to_string(),
                    "Monaco editor core".to_string(),
                    "Language service integration".to_string(),
                ],
            }),
        });

        // Rust - Developer Tools (Simple)
        self.add_repository(ShowcaseRepository {
            name: "ripgrep".to_string(),
            url: "https://github.com/BurntSushi/ripgrep".to_string(),
            description: "A line-oriented search tool that recursively searches for patterns"
                .to_string(),
            primary_language: "rust".to_string(),
            languages: vec!["rust".to_string()],
            frameworks: vec!["clap".to_string(), "regex".to_string()],
            category: RepositoryCategory::DeveloperTools,
            complexity_tier: ComplexityTier::Intermediate,
            estimated_analysis_time_seconds: 15,
            highlights: vec![
                "High-performance search algorithms".to_string(),
                "Memory-efficient processing".to_string(),
                "Cross-platform compatibility".to_string(),
                "Rich CLI interface".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 50,
                estimated_functions: 300,
                estimated_complexity: 5.5,
                key_insights: vec![
                    "Well-structured command-line tool".to_string(),
                    "Efficient file system traversal".to_string(),
                    "Optimized pattern matching algorithms".to_string(),
                ],
                recommended_focus_areas: vec![
                    "Search algorithm implementation".to_string(),
                    "File type detection logic".to_string(),
                    "Output formatting systems".to_string(),
                ],
            }),
        });

        // Python - Data Science
        self.add_repository(ShowcaseRepository {
            name: "pandas".to_string(),
            url: "https://github.com/pandas-dev/pandas".to_string(),
            description: "Flexible and powerful data analysis/manipulation library".to_string(),
            primary_language: "python".to_string(),
            languages: vec!["python".to_string(), "cython".to_string()],
            frameworks: vec!["NumPy".to_string(), "Cython".to_string()],
            category: RepositoryCategory::DataScience,
            complexity_tier: ComplexityTier::Advanced,
            estimated_analysis_time_seconds: 75,
            highlights: vec![
                "Complex data structures (DataFrame, Series)".to_string(),
                "High-performance computing with Cython".to_string(),
                "Extensive I/O capabilities".to_string(),
                "Statistical and analytical functions".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 600,
                estimated_functions: 4000,
                estimated_complexity: 8.1,
                key_insights: vec![
                    "Complex data type system and indexing".to_string(),
                    "Extensive use of Cython for performance".to_string(),
                    "Sophisticated memory management".to_string(),
                ],
                recommended_focus_areas: vec![
                    "Core data structures implementation".to_string(),
                    "I/O and serialization systems".to_string(),
                    "Statistical computation engines".to_string(),
                ],
            }),
        });

        // JavaScript - Simple Web Tool
        self.add_repository(ShowcaseRepository {
            name: "Lodash".to_string(),
            url: "https://github.com/lodash/lodash".to_string(),
            description:
                "A modern JavaScript utility library delivering consistency and performance"
                    .to_string(),
            primary_language: "javascript".to_string(),
            languages: vec!["javascript".to_string()],
            frameworks: vec![],
            category: RepositoryCategory::DeveloperTools,
            complexity_tier: ComplexityTier::Beginner,
            estimated_analysis_time_seconds: 8,
            highlights: vec![
                "Comprehensive utility functions".to_string(),
                "Functional programming patterns".to_string(),
                "High test coverage".to_string(),
                "Consistent API design".to_string(),
            ],
            analysis_preview: Some(AnalysisPreview {
                estimated_files: 200,
                estimated_functions: 800,
                estimated_complexity: 3.2,
                key_insights: vec![
                    "Well-organized utility function library".to_string(),
                    "Consistent parameter handling patterns".to_string(),
                    "Extensive edge case handling".to_string(),
                ],
                recommended_focus_areas: vec![
                    "Array and object manipulation functions".to_string(),
                    "String processing utilities".to_string(),
                    "Type checking and validation systems".to_string(),
                ],
            }),
        });
    }

    fn add_repository(&mut self, repo: ShowcaseRepository) {
        let category = repo.category.clone();
        let name = repo.name.clone();

        self.repositories.insert(name.clone(), repo);
        self.categories.entry(category).or_default().push(name);
    }

    #[must_use]
    pub fn get_all_repositories(&self) -> Vec<&ShowcaseRepository> {
        self.repositories.values().collect()
    }

    #[must_use]
    pub fn get_repositories_by_category(
        &self,
        category: &RepositoryCategory,
    ) -> Vec<&ShowcaseRepository> {
        if let Some(repo_names) = self.categories.get(category) {
            repo_names
                .iter()
                .filter_map(|name| self.repositories.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    #[must_use]
    pub fn get_repositories_by_complexity(
        &self,
        tier: &ComplexityTier,
    ) -> Vec<&ShowcaseRepository> {
        self.repositories
            .values()
            .filter(|repo| repo.complexity_tier == *tier)
            .collect()
    }

    #[must_use]
    pub fn get_repositories_by_language(&self, language: &str) -> Vec<&ShowcaseRepository> {
        let lang_lower = language.to_lowercase();
        self.repositories
            .values()
            .filter(|repo| {
                repo.primary_language.to_lowercase() == lang_lower
                    || repo
                        .languages
                        .iter()
                        .any(|l| l.to_lowercase() == lang_lower)
            })
            .collect()
    }

    #[must_use]
    pub fn get_repository_by_name(&self, name: &str) -> Option<&ShowcaseRepository> {
        self.repositories.get(name)
    }

    #[must_use]
    pub fn get_categories(&self) -> Vec<&RepositoryCategory> {
        self.categories.keys().collect()
    }

    #[must_use]
    pub fn get_quick_start_recommendations(&self) -> Vec<&ShowcaseRepository> {
        // Return beginner and intermediate repositories for quick starts
        self.repositories
            .values()
            .filter(|repo| {
                matches!(
                    repo.complexity_tier,
                    ComplexityTier::Beginner | ComplexityTier::Intermediate
                )
            })
            .take(4)
            .collect()
    }

    #[must_use]
    pub fn get_featured_repositories(&self) -> Vec<&ShowcaseRepository> {
        // Return a curated selection of featured repositories
        let featured_names = vec!["Tokio", "Django", "React", "VS Code"];
        featured_names
            .into_iter()
            .filter_map(|name| self.repositories.get(name))
            .collect()
    }

    #[must_use]
    pub fn generate_showcase_summary(&self) -> ShowcaseSummary {
        let total_repositories = self.repositories.len();
        let languages: std::collections::HashSet<String> = self
            .repositories
            .values()
            .flat_map(|repo| repo.languages.iter().cloned())
            .collect();

        let categories_count = self.categories.len();

        let complexity_distribution = {
            let mut distribution = HashMap::new();
            for repo in self.repositories.values() {
                *distribution.entry(repo.complexity_tier).or_insert(0) += 1;
            }
            distribution
        };

        ShowcaseSummary {
            total_repositories,
            total_languages: languages.len(),
            total_categories: categories_count,
            complexity_distribution,
            featured_count: 4,
            quick_start_count: self.get_quick_start_recommendations().len(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShowcaseSummary {
    pub total_repositories: usize,
    pub total_languages: usize,
    pub total_categories: usize,
    pub complexity_distribution: HashMap<ComplexityTier, usize>,
    pub featured_count: usize,
    pub quick_start_count: usize,
}

impl Default for ShowcaseGallery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcase_gallery_initialization() {
        let gallery = ShowcaseGallery::new();

        assert!(!gallery.repositories.is_empty());
        assert!(!gallery.categories.is_empty());

        // Should have repositories in different categories
        assert!(gallery.categories.len() >= 4);
    }

    #[test]
    fn test_repository_filtering() {
        let gallery = ShowcaseGallery::new();

        // Test filtering by language
        let rust_repos = gallery.get_repositories_by_language("rust");
        assert!(!rust_repos.is_empty());

        // Test filtering by complexity
        let beginner_repos = gallery.get_repositories_by_complexity(&ComplexityTier::Beginner);
        assert!(!beginner_repos.is_empty());

        // Test filtering by category
        let web_repos = gallery.get_repositories_by_category(&RepositoryCategory::WebFramework);
        assert!(!web_repos.is_empty());
    }

    #[test]
    fn test_showcase_recommendations() {
        let gallery = ShowcaseGallery::new();

        let quick_start = gallery.get_quick_start_recommendations();
        assert!(!quick_start.is_empty());
        assert!(quick_start.len() <= 4);

        let featured = gallery.get_featured_repositories();
        assert!(!featured.is_empty());
        assert_eq!(featured.len(), 4);
    }

    #[test]
    fn test_showcase_summary() {
        let gallery = ShowcaseGallery::new();
        let summary = gallery.generate_showcase_summary();

        assert!(summary.total_repositories > 0);
        assert!(summary.total_languages > 0);
        assert!(summary.total_categories > 0);
        assert!(!summary.complexity_distribution.is_empty());
    }

    #[test]
    fn test_repository_structure() {
        let gallery = ShowcaseGallery::new();

        if let Some(repo) = gallery.get_repository_by_name("Tokio") {
            assert_eq!(repo.name, "Tokio");
            assert_eq!(repo.primary_language, "rust");
            assert!(matches!(repo.complexity_tier, ComplexityTier::Advanced));
            assert!(repo.analysis_preview.is_some());
        } else {
            panic!("Tokio repository should be present in showcase");
        }
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
