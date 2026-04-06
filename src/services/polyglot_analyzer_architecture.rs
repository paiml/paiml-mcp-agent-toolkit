impl PolyglotAnalyzer {
    async fn detect_architecture_pattern(
        &self,
        project_path: &Path,
        language_info: &HashMap<String, LanguageInfo>,
    ) -> Result<Option<ArchitecturePattern>, Box<dyn std::error::Error>> {
        let language_count = language_info.len();

        // Analyze project structure for architecture indicators
        let architecture_indicators = self.analyze_architecture_indicators(project_path).await?;

        for signature in &self.architecture_signatures {
            if language_count >= signature.required_languages {
                let confidence = self.calculate_architecture_confidence(
                    signature,
                    language_info,
                    &architecture_indicators,
                );
                if confidence >= signature.confidence_threshold {
                    return Ok(Some(signature.pattern.clone()));
                }
            }
        }

        // Fallback detection based on project structure
        if architecture_indicators.has_microservice_indicators {
            Ok(Some(ArchitecturePattern::Microservices))
        } else if architecture_indicators.has_layered_indicators {
            Ok(Some(ArchitecturePattern::LayeredArchitecture))
        } else if architecture_indicators.has_event_indicators {
            Ok(Some(ArchitecturePattern::EventDriven))
        } else if language_count >= 3 {
            Ok(Some(ArchitecturePattern::Mixed))
        } else if language_count >= 2 {
            Ok(Some(ArchitecturePattern::ClientServer))
        } else {
            Ok(Some(ArchitecturePattern::Monolithic))
        }
    }

    // Helper to check if any directory contains specified patterns
    fn has_directory_pattern(directories: &[String], patterns: &[&str]) -> bool {
        directories
            .iter()
            .any(|d| patterns.iter().any(|p| d.contains(p)))
    }

    async fn analyze_architecture_indicators(
        &self,
        project_path: &Path,
    ) -> Result<ArchitectureIndicators, Box<dyn std::error::Error>> {
        // Analyze directory structure
        let directory_structure = self.analyze_directory_structure(project_path).await?;

        // Check for microservice indicators
        let microservice_files = [
            "docker-compose.yml",
            "docker-compose.yaml",
            "kubernetes",
            "k8s",
        ];
        let has_microservice_indicators = microservice_files
            .iter()
            .any(|file| project_path.join(file).exists());

        // Check for layered architecture indicators
        let has_layered_indicators = self.check_layered_architecture(&directory_structure);

        // Check for event-driven indicators
        let event_patterns = ["event", "message", "msg", "queue"];
        let has_event_indicators =
            Self::has_directory_pattern(&directory_structure, &event_patterns);

        // Check for plugin architecture indicators
        let plugin_patterns = ["plugin", "extension"];
        let has_plugin_indicators =
            Self::has_directory_pattern(&directory_structure, &plugin_patterns);

        Ok(ArchitectureIndicators {
            has_microservice_indicators,
            has_layered_indicators,
            has_event_indicators,
            has_plugin_indicators,
            directory_structure,
            config_files: Vec::new(),
        })
    }

    fn check_layered_architecture(&self, directories: &[String]) -> bool {
        let has_controller = Self::has_directory_pattern(directories, &["controller"]);
        let has_service = Self::has_directory_pattern(directories, &["service"]);
        let has_repository = Self::has_directory_pattern(directories, &["repository", "dao"]);
        let has_model = Self::has_directory_pattern(directories, &["model", "entity"]);

        has_service && (has_controller || has_repository || has_model)
    }

    async fn analyze_directory_structure(
        &self,
        project_path: &Path,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut directories = Vec::new();
        self.collect_directories_recursive(project_path, &mut directories, 0)?;
        Ok(directories)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_directories_recursive(
        &self,
        dir_path: &Path,
        directories: &mut Vec<String>,
        depth: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if depth > 3 {
            // Limit recursion depth
            return Ok(());
        }

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip common non-architecture directories
                        if matches!(
                            dir_name,
                            "node_modules"
                                | "target"
                                | "build"
                                | ".git"
                                | "__pycache__"
                                | ".venv"
                                | "venv"
                                | "dist"
                        ) {
                            continue;
                        }
                        directories.push(dir_name.to_lowercase());
                        self.collect_directories_recursive(&path, directories, depth + 1)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn calculate_architecture_confidence(
        &self,
        signature: &ArchitectureSignature,
        language_info: &HashMap<String, LanguageInfo>,
        indicators: &ArchitectureIndicators,
    ) -> f64 {
        let mut confidence: f64 = 0.0;

        match signature.pattern {
            ArchitecturePattern::Microservices => {
                if indicators.has_microservice_indicators {
                    confidence += 0.6;
                }
                if language_info.len() >= 2 {
                    confidence += 0.2;
                }
                // Docker/container indicators
                if indicators.config_files.iter().any(|f| f.contains("docker")) {
                    confidence += 0.2;
                }
            }
            ArchitecturePattern::LayeredArchitecture => {
                if indicators.has_layered_indicators {
                    confidence += 0.7;
                }
                // Common in enterprise applications
                if language_info.contains_key("java") || language_info.contains_key("csharp") {
                    confidence += 0.1;
                }
            }
            ArchitecturePattern::EventDriven => {
                if indicators.has_event_indicators {
                    confidence += 0.6;
                }
                // Message queues, event systems
                if indicators
                    .directory_structure
                    .iter()
                    .any(|d| d.contains("kafka") || d.contains("rabbitmq"))
                {
                    confidence += 0.2;
                }
            }
            ArchitecturePattern::PluginArchitecture => {
                if indicators.has_plugin_indicators {
                    confidence += 0.7;
                }
            }
            _ => {
                confidence = 0.5; // Default confidence
            }
        }

        confidence.clamp(0.0, 1.0)
    }

    async fn identify_integration_points(
        &self,
        _project_path: &Path,
        cross_deps: &[CrossLanguageDependency],
    ) -> Result<Vec<IntegrationPoint>, Box<dyn std::error::Error>> {
        let mut integration_points = Vec::new();

        for dep in cross_deps {
            integration_points.push(IntegrationPoint {
                name: format!("{} ↔ {}", dep.from_language, dep.to_language),
                languages: vec![dep.from_language.clone(), dep.to_language.clone()],
                integration_type: self.map_dependency_to_integration(&dep.dependency_type),
                risk_level: self.assess_risk_level(dep.coupling_strength),
                description: format!(
                    "Integration between {} and {} via {:?}",
                    dep.from_language, dep.to_language, dep.dependency_type
                ),
            });
        }

        Ok(integration_points)
    }

    fn map_dependency_to_integration(&self, dep_type: &DependencyType) -> IntegrationType {
        match dep_type {
            DependencyType::FFI => IntegrationType::Memory,
            DependencyType::ProcessCommunication => IntegrationType::Network,
            DependencyType::SharedDataStructure => IntegrationType::Memory,
            DependencyType::ConfigurationFile => IntegrationType::Configuration,
            DependencyType::BuildSystem => IntegrationType::FileSystem,
            DependencyType::Testing => IntegrationType::API,
        }
    }

    fn assess_risk_level(&self, coupling_strength: f64) -> RiskLevel {
        if coupling_strength >= 0.8 {
            RiskLevel::Critical
        } else if coupling_strength >= 0.6 {
            RiskLevel::High
        } else if coupling_strength >= 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn calculate_recommendation_score(
        &self,
        language_stats: &[LanguageStats],
        cross_deps: &[CrossLanguageDependency],
        architecture: &Option<ArchitecturePattern>,
    ) -> f64 {
        let mut score: f64 = 0.0;

        let total_lines: usize = language_stats.iter().map(|s| s.line_count).sum();
        if total_lines > 1000 {
            score += 0.3;
        }

        let language_count = language_stats.len();
        if language_count >= 2 {
            score += 0.2;
        }
        if language_count >= 3 {
            score += 0.2;
        }

        if !cross_deps.is_empty() {
            score += 0.2;
        }

        if architecture.is_some() {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_polyglot_insights(&self, analysis: &PolyglotAnalysis) -> Vec<String> {
        let mut insights = Vec::new();

        if analysis.languages.len() >= 3 {
            insights
                .push("This is a polyglot project with multiple programming languages".to_string());
        }

        let primary_language = analysis.languages.first();
        if let Some(lang) = primary_language {
            insights.push(format!(
                "Primary language: {} ({} files, {} lines)",
                lang.language, lang.file_count, lang.line_count
            ));
        }

        if !analysis.cross_language_dependencies.is_empty() {
            insights.push(format!(
                "Found {} cross-language integration points",
                analysis.cross_language_dependencies.len()
            ));
        }

        if let Some(pattern) = &analysis.architecture_pattern {
            insights.push(format!("Architecture pattern: {pattern:?}"));
        }

        let high_risk_points: Vec<_> = analysis
            .integration_points
            .iter()
            .filter(|p| matches!(p.risk_level, RiskLevel::High | RiskLevel::Critical))
            .collect();

        if !high_risk_points.is_empty() {
            insights.push(format!(
                "⚠️  {} high-risk integration points identified",
                high_risk_points.len()
            ));
        }

        insights.push(format!(
            "Overall recommendation score: {:.2}/1.0",
            analysis.recommendation_score
        ));

        insights
    }
}
