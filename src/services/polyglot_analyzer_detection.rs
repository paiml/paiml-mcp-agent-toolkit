impl PolyglotAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        let mut analyzer = Self {
            language_patterns: HashMap::new(),
            architecture_signatures: Vec::new(),
        };
        analyzer.initialize_patterns();
        analyzer.initialize_architecture_signatures();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        self.language_patterns.insert(
            "rust".to_string(),
            LanguagePattern {
                file_extensions: vec![".rs".to_string()],
                _build_files: vec!["Cargo.toml".to_string(), "Cargo.lock".to_string()],
                _config_files: vec!["rust-toolchain".to_string(), ".rustfmt.toml".to_string()],
                _dependency_files: vec!["Cargo.toml".to_string()],
            },
        );

        self.language_patterns.insert(
            "python".to_string(),
            LanguagePattern {
                file_extensions: vec![".py".to_string(), ".pyw".to_string()],
                _build_files: vec!["setup.py".to_string(), "pyproject.toml".to_string()],
                _config_files: vec!["setup.cfg".to_string(), "tox.ini".to_string()],
                _dependency_files: vec!["requirements.txt".to_string(), "Pipfile".to_string()],
            },
        );

        self.language_patterns.insert(
            "typescript".to_string(),
            LanguagePattern {
                file_extensions: vec![".ts".to_string(), ".tsx".to_string()],
                _build_files: vec!["package.json".to_string(), "tsconfig.json".to_string()],
                _config_files: vec![
                    "webpack.config.js".to_string(),
                    ".eslintrc.json".to_string(),
                ],
                _dependency_files: vec!["package.json".to_string(), "yarn.lock".to_string()],
            },
        );

        self.language_patterns.insert(
            "javascript".to_string(),
            LanguagePattern {
                file_extensions: vec![".js".to_string(), ".jsx".to_string()],
                _build_files: vec!["package.json".to_string(), "webpack.config.js".to_string()],
                _config_files: vec![".babelrc".to_string(), ".eslintrc.js".to_string()],
                _dependency_files: vec![
                    "package.json".to_string(),
                    "package-lock.json".to_string(),
                ],
            },
        );
    }

    fn initialize_architecture_signatures(&mut self) {
        self.architecture_signatures.push(ArchitectureSignature {
            pattern: ArchitecturePattern::Microservices,
            _indicators: vec![
                "docker-compose".to_string(),
                "kubernetes".to_string(),
                "service".to_string(),
                "api".to_string(),
            ],
            required_languages: 2,
            confidence_threshold: 0.7,
        });

        self.architecture_signatures.push(ArchitectureSignature {
            pattern: ArchitecturePattern::LayeredArchitecture,
            _indicators: vec![
                "controller".to_string(),
                "service".to_string(),
                "repository".to_string(),
                "model".to_string(),
            ],
            required_languages: 1,
            confidence_threshold: 0.8,
        });

        self.architecture_signatures.push(ArchitectureSignature {
            pattern: ArchitecturePattern::EventDriven,
            _indicators: vec![
                "event".to_string(),
                "message".to_string(),
                "queue".to_string(),
                "pub_sub".to_string(),
            ],
            required_languages: 1,
            confidence_threshold: 0.6,
        });
    }

    pub async fn analyze_project(
        &self,
        project_path: &Path,
    ) -> Result<PolyglotAnalysis, Box<dyn std::error::Error>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let language_info = self.detect_languages(project_path).await?;
        let language_stats = self.calculate_language_stats(&language_info).await?;
        let cross_deps = self
            .analyze_cross_language_dependencies(project_path, &language_info)
            .await?;
        let architecture = self
            .detect_architecture_pattern(project_path, &language_info)
            .await?;
        let integration_points = self
            .identify_integration_points(project_path, &cross_deps)
            .await?;
        let recommendation_score =
            self.calculate_recommendation_score(&language_stats, &cross_deps, &architecture);

        Ok(PolyglotAnalysis {
            languages: language_stats,
            cross_language_dependencies: cross_deps,
            architecture_pattern: architecture,
            integration_points,
            recommendation_score,
        })
    }

    async fn detect_languages(
        &self,
        project_path: &Path,
    ) -> Result<HashMap<String, LanguageInfo>, Box<dyn std::error::Error>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut languages = HashMap::new();

        for (lang_name, pattern) in &self.language_patterns {
            let mut file_count = 0;
            let mut total_lines = 0;
            // Recursively scan project directory for language files
            self.scan_directory_recursive(
                project_path,
                &pattern.file_extensions,
                &mut file_count,
                &mut total_lines,
            )?;

            // Detect frameworks for this language
            let frameworks = self
                .detect_language_frameworks(project_path, lang_name)
                .await?;

            if file_count > 0 {
                languages.insert(
                    lang_name.clone(),
                    LanguageInfo {
                        name: lang_name.clone(),
                        file_count,
                        line_count: total_lines,
                        frameworks,
                    },
                );
            }
        }

        Ok(languages)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn scan_directory_recursive(
        &self,
        dir_path: &Path,
        extensions: &[String],
        file_count: &mut usize,
        total_lines: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug_assert!(dir_path.exists(), "dir_path must exist: {}", dir_path.display());
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    self.handle_directory(&path, extensions, file_count, total_lines)?;
                } else if path.is_file() {
                    self.handle_file(&path, extensions, file_count, total_lines);
                }
            }
        }
        Ok(())
    }

    /// Toyota Way: Extract Method - Handle directory processing (complexity ≤5)
    fn handle_directory(
        &self,
        path: &Path,
        extensions: &[String],
        file_count: &mut usize,
        total_lines: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if should_skip_directory(path) {
            return Ok(());
        }

        self.scan_directory_recursive(path, extensions, file_count, total_lines)
    }

    /// Toyota Way: Extract Method - Handle file processing (complexity ≤5)
    fn handle_file(
        &self,
        path: &Path,
        extensions: &[String],
        file_count: &mut usize,
        total_lines: &mut usize,
    ) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let full_ext = format!(".{ext}");
            if extensions.contains(&full_ext) {
                *file_count += 1;
                if let Ok(content) = std::fs::read_to_string(path) {
                    *total_lines += content.lines().count();
                }
            }
        }
    }

    // Helper function to check frameworks in content
    fn check_frameworks(content: &str, framework_map: &[(&str, &str)]) -> Vec<String> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        framework_map
            .iter()
            .filter(|(search_term, _)| content.contains(search_term))
            .map(|(_, name)| (*name).to_string())
            .collect()
    }

    async fn detect_language_frameworks(
        &self,
        project_path: &Path,
        language: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        debug_assert!(!language.is_empty(), "language must not be empty");
        match language {
            "rust" => self.detect_rust_frameworks(project_path).await,
            "python" => self.detect_python_frameworks(project_path).await,
            "typescript" | "javascript" => self.detect_js_frameworks(project_path).await,
            _ => Ok(Vec::new()),
        }
    }

    async fn detect_rust_frameworks(
        &self,
        project_path: &Path,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let frameworks_map = [
            ("tokio", "Tokio"),
            ("actix-web", "Actix Web"),
            ("axum", "Axum"),
            ("diesel", "Diesel"),
            ("serde", "Serde"),
            ("clap", "Clap"),
        ];

        if let Ok(cargo_toml) = std::fs::read_to_string(project_path.join("Cargo.toml")) {
            Ok(Self::check_frameworks(&cargo_toml, &frameworks_map))
        } else {
            Ok(Vec::new())
        }
    }

    async fn detect_python_frameworks(
        &self,
        project_path: &Path,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let frameworks_map = [
            ("django", "Django"),
            ("flask", "Flask"),
            ("fastapi", "FastAPI"),
            ("pandas", "Pandas"),
            ("numpy", "NumPy"),
        ];

        let mut frameworks = Vec::new();

        // Check requirements.txt
        if let Ok(reqs) = std::fs::read_to_string(project_path.join("requirements.txt")) {
            frameworks.extend(Self::check_frameworks(&reqs, &frameworks_map));
        }

        // Check pyproject.toml (only for web frameworks to avoid duplicates)
        if let Ok(pyproject) = std::fs::read_to_string(project_path.join("pyproject.toml")) {
            let web_frameworks = &frameworks_map[..3]; // Only Django, Flask, FastAPI
            for framework in Self::check_frameworks(&pyproject, web_frameworks) {
                if !frameworks.contains(&framework) {
                    frameworks.push(framework);
                }
            }
        }

        Ok(frameworks)
    }

    async fn detect_js_frameworks(
        &self,
        project_path: &Path,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let frameworks_map = [
            ("react", "React"),
            ("vue", "Vue.js"),
            ("angular", "Angular"),
            ("express", "Express.js"),
            ("next", "Next.js"),
            ("svelte", "Svelte"),
        ];

        if let Ok(package_json) = std::fs::read_to_string(project_path.join("package.json")) {
            Ok(Self::check_frameworks(&package_json, &frameworks_map))
        } else {
            Ok(Vec::new())
        }
    }

    async fn calculate_language_stats(
        &self,
        language_info: &HashMap<String, LanguageInfo>,
    ) -> Result<Vec<LanguageStats>, Box<dyn std::error::Error>> {
        let mut stats = Vec::new();

        for (lang_name, info) in language_info {
            let complexity_score = self.calculate_language_complexity_score(info);
            let test_coverage = self.estimate_test_coverage(info);

            stats.push(LanguageStats {
                language: lang_name.clone(),
                file_count: info.file_count,
                line_count: info.line_count,
                complexity_score,
                test_coverage,
                primary_frameworks: info.frameworks.clone(),
            });
        }

        stats.sort_by(|a, b| b.line_count.cmp(&a.line_count));
        Ok(stats)
    }

    fn calculate_language_complexity_score(&self, info: &LanguageInfo) -> f64 {
        let base_score = (info.line_count as f64).ln() / (info.file_count as f64).ln();
        base_score.clamp(1.0, 10.0)
    }

    fn estimate_test_coverage(&self, _info: &LanguageInfo) -> f64 {
        0.75
    }
}
