// Showcase repository definitions — data-only initialization
// Included from showcase.rs — no `use` imports or `#!` attributes allowed

impl ShowcaseGallery {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn initialize_showcase_repositories(&mut self) {
        self.add_systems_and_web_repositories();
        self.add_cloud_and_tools_repositories();
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn add_systems_and_web_repositories(&mut self) {
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
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn add_cloud_and_tools_repositories(&mut self) {
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn add_repository(&mut self, repo: ShowcaseRepository) {
        let category = repo.category.clone();
        let name = repo.name.clone();

        self.repositories.insert(name.clone(), repo);
        self.categories.entry(category).or_default().push(name);
    }
}
