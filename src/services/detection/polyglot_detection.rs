// Detector trait implementation and PolyglotDetector analysis methods

#[async_trait]
impl Detector for PolyglotDetector {
    type Input = DetectionInput;
    type Output = DetectionOutput;
    type Config = DetectionConfig;

    async fn detect(&self, input: Self::Input, config: Self::Config) -> Result<Self::Output> {
        // Extract polyglot-specific config
        let polyglot_config = match config.detector_specific {
            DetectorSpecificConfig::Polyglot(config) => config,
            _ => PolyglotConfig::default(),
        };

        // Delegate to the existing polyglot analyzer functionality
        let result = match input {
            DetectionInput::SingleFile(_path) => {
                // Single file analysis is limited for polyglot - create minimal result
                PolyglotAnalysis {
                    languages: Vec::new(),
                    cross_language_dependencies: Vec::new(),
                    architecture_pattern: None,
                    integration_points: Vec::new(),
                    recommendation_score: 0.0,
                }
            }
            DetectionInput::MultipleFiles(files) => {
                self.analyze_files(&files, &polyglot_config).await?
            }
            DetectionInput::ProjectDirectory(dir) => {
                self.analyze_project_directory(&dir, &polyglot_config)
                    .await?
            }
            DetectionInput::Content(_content) => {
                // Content-based analysis is limited for polyglot
                PolyglotAnalysis {
                    languages: Vec::new(),
                    cross_language_dependencies: Vec::new(),
                    architecture_pattern: None,
                    integration_points: Vec::new(),
                    recommendation_score: 0.0,
                }
            }
        };

        Ok(DetectionOutput::Polyglot(result))
    }

    fn name(&self) -> &'static str {
        "polyglot"
    }

    fn capabilities(&self) -> DetectorCapabilities {
        DetectorCapabilities {
            supports_batch: true,
            supports_streaming: false,
            language_agnostic: true,
            requires_ast: true,
        }
    }
}

impl PolyglotDetector {
    async fn analyze_files(
        &self,
        files: &[std::path::PathBuf],
        _config: &PolyglotConfig,
    ) -> Result<PolyglotAnalysis> {
        debug_assert!(!files.is_empty(), "files must not be empty");
        // Delegate to the existing polyglot_analyzer module functionality
        let _analyzer = crate::services::polyglot_analyzer::PolyglotAnalyzer::new();

        // Group files by language
        let mut language_files: std::collections::HashMap<String, Vec<&std::path::PathBuf>> =
            std::collections::HashMap::new();

        for file in files {
            if let Some(language) = self.detect_language(file) {
                language_files.entry(language).or_default().push(file);
            }
        }

        // Analyze each language
        let mut languages = Vec::new();
        for (language, files) in language_files {
            let stats = self.analyze_language_files(&language, &files).await?;
            languages.push(stats);
        }

        // Detect cross-language dependencies
        let dependencies = self.detect_cross_language_dependencies(files).await?;

        // Detect architecture pattern
        let architecture_pattern = self.detect_architecture_pattern(&languages, &dependencies);

        // Find integration points
        let integration_points = self.find_integration_points(files).await?;

        // Calculate recommendation score
        let recommendation_score = self.calculate_recommendation_score(&languages, &dependencies);

        Ok(PolyglotAnalysis {
            languages,
            cross_language_dependencies: dependencies,
            architecture_pattern,
            integration_points,
            recommendation_score,
        })
    }

    async fn analyze_project_directory(
        &self,
        dir_path: &Path,
        _config: &PolyglotConfig,
    ) -> Result<PolyglotAnalysis> {
        // Scan directory for all source files
        let files = self.scan_project_directory(dir_path)?;
        self.analyze_files(&files, _config).await
    }

    fn detect_language(&self, file_path: &Path) -> Option<String> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some("Rust".to_string()),
                "ts" => Some("TypeScript".to_string()),
                "js" => Some("JavaScript".to_string()),
                "py" => Some("Python".to_string()),
                "c" => Some("C".to_string()),
                "cpp" | "cxx" | "cc" | "cu" | "cuh" => Some("C++".to_string()),
                "java" => Some("Java".to_string()),
                "kt" => Some("Kotlin".to_string()),
                "go" => Some("Go".to_string()),
                _ => None,
            })
    }

    async fn analyze_language_files(
        &self,
        language: &str,
        files: &[&std::path::PathBuf],
    ) -> Result<LanguageStats> {
        debug_assert!(!language.is_empty(), "language must not be empty");
        debug_assert!(!files.is_empty(), "files must not be empty");
        let mut total_lines = 0;
        let mut complexity_scores = Vec::new();
        let mut frameworks = std::collections::HashSet::new();

        for file in files {
            if let Ok(content) = std::fs::read_to_string(file) {
                total_lines += content.lines().count();

                // Basic complexity estimation (placeholder)
                let complexity = self.estimate_file_complexity(&content);
                complexity_scores.push(complexity);

                // Detect frameworks (basic pattern matching)
                frameworks.extend(self.detect_frameworks_in_content(&content, language));
            }
        }

        let avg_complexity = if complexity_scores.is_empty() {
            0.0
        } else {
            complexity_scores.iter().sum::<f64>() / complexity_scores.len() as f64
        };

        Ok(LanguageStats {
            language: language.to_string(),
            file_count: files.len(),
            line_count: total_lines,
            complexity_score: avg_complexity,
            test_coverage: 0.0, // Placeholder - would need actual coverage analysis
            primary_frameworks: frameworks.into_iter().collect(),
        })
    }

    async fn detect_cross_language_dependencies(
        &self,
        files: &[std::path::PathBuf],
    ) -> Result<Vec<CrossLanguageDependency>> {
        debug_assert!(!files.is_empty(), "files must not be empty");
        let mut dependencies = Vec::new();

        // This is a simplified implementation
        // A full implementation would analyze imports, FFI calls, build files, etc.

        // For now, detect some common patterns
        for file in files {
            if let Ok(content) = std::fs::read_to_string(file) {
                let from_language = self
                    .detect_language(file)
                    .unwrap_or_else(|| "Unknown".to_string());

                // Look for common cross-language patterns
                if content.contains("extern \"C\"") && from_language == "Rust" {
                    dependencies.push(CrossLanguageDependency {
                        from_language: "Rust".to_string(),
                        to_language: "C".to_string(),
                        dependency_type: DependencyType::FFI,
                        coupling_strength: 0.8,
                        files_involved: vec![file.to_string_lossy().to_string()],
                    });
                }

                if content.contains("import") && from_language == "TypeScript" {
                    // Simplified - would need more sophisticated import analysis
                    dependencies.push(CrossLanguageDependency {
                        from_language: "TypeScript".to_string(),
                        to_language: "JavaScript".to_string(),
                        dependency_type: DependencyType::SharedDataStructure,
                        coupling_strength: 0.6,
                        files_involved: vec![file.to_string_lossy().to_string()],
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn detect_architecture_pattern(
        &self,
        languages: &[LanguageStats],
        dependencies: &[CrossLanguageDependency],
    ) -> Option<ArchitecturePattern> {
        debug_assert!(!languages.is_empty(), "languages must not be empty");
        // Simplified pattern detection logic
        if languages.len() == 1 {
            Some(ArchitecturePattern::Monolithic)
        } else if dependencies.len() > languages.len() {
            Some(ArchitecturePattern::Microservices)
        } else if dependencies
            .iter()
            .any(|d| d.dependency_type == DependencyType::FFI)
        {
            Some(ArchitecturePattern::LayeredWithFFI)
        } else {
            Some(ArchitecturePattern::Modular)
        }
    }

    async fn find_integration_points(
        &self,
        files: &[std::path::PathBuf],
    ) -> Result<Vec<IntegrationPoint>> {
        debug_assert!(!files.is_empty(), "files must not be empty");
        let mut integration_points = Vec::new();

        // Look for common integration patterns
        for file in files {
            if let Ok(content) = std::fs::read_to_string(file) {
                // API endpoints
                if content.contains("@app.route")
                    || content.contains("app.get")
                    || content.contains("express")
                {
                    integration_points.push(IntegrationPoint {
                        point_type: IntegrationPointType::RestAPI,
                        location: file.to_string_lossy().to_string(),
                        technologies: vec!["HTTP".to_string(), "REST".to_string()],
                        complexity_score: 0.7,
                    });
                }

                // Database connections
                if content.contains("SELECT")
                    || content.contains("INSERT")
                    || content.contains("database")
                {
                    integration_points.push(IntegrationPoint {
                        point_type: IntegrationPointType::Database,
                        location: file.to_string_lossy().to_string(),
                        technologies: vec!["SQL".to_string()],
                        complexity_score: 0.5,
                    });
                }
            }
        }

        Ok(integration_points)
    }

}
