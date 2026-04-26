impl PolyglotAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_project(
        &self,
        project_path: &Path,
    ) -> Result<PolyglotAnalysis, Box<dyn std::error::Error>> {
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

    // Helper function to check frameworks in content.
    // Wave 39 PR23: contract added — output length is bounded by map length
    // (`check_compliance` invariant, no unbounded allocations).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn check_frameworks(content: &str, framework_map: &[(&str, &str)]) -> Vec<String> {
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

        stats.sort_by_key(|b| std::cmp::Reverse(b.line_count));
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

#[cfg(test)]
mod polyglot_detection_tests {
    //! Wave 39 PR23 — pure-helper coverage for polyglot_analyzer_detection.rs
    //! (185 missed at 30% pre-wave). Async framework-detect methods do
    //! filesystem reads (testable with tempdir). The static `check_frameworks`
    //! helper is purely computational.
    use super::*;

    // ── check_frameworks (static, pure) ─────────────────────────────────────

    #[test]
    fn test_check_frameworks_finds_listed_terms() {
        let map: &[(&str, &str)] = &[("tokio", "Tokio"), ("serde", "Serde"), ("clap", "Clap")];
        let content = "[dependencies]\ntokio = \"1\"\nserde = \"1\"\n";
        let found = PolyglotAnalyzer::check_frameworks(content, map);
        assert!(found.contains(&"Tokio".to_string()));
        assert!(found.contains(&"Serde".to_string()));
        assert!(!found.contains(&"Clap".to_string()));
    }

    #[test]
    fn test_check_frameworks_empty_content_returns_empty() {
        let map: &[(&str, &str)] = &[("tokio", "Tokio")];
        let found = PolyglotAnalyzer::check_frameworks("", map);
        assert!(found.is_empty());
    }

    #[test]
    fn test_check_frameworks_empty_map_returns_empty() {
        // PIN: empty framework map → empty result regardless of content.
        let found = PolyglotAnalyzer::check_frameworks("tokio = \"1\"", &[]);
        assert!(found.is_empty());
    }

    #[test]
    fn test_check_frameworks_substring_match_no_word_boundary() {
        // PIN: contains() is substring match (not word-boundary). So
        // "tokio" matches inside "tokio_metrics" too. Document this so
        // future contributors don't tighten without intent.
        let map: &[(&str, &str)] = &[("tokio", "Tokio")];
        let content = "tokio_metrics = \"0.1\"";
        let found = PolyglotAnalyzer::check_frameworks(content, map);
        assert_eq!(found, vec!["Tokio".to_string()]);
    }

    #[test]
    fn test_check_frameworks_preserves_order_of_map() {
        // PIN: results are produced by filter_map over the map iter →
        // order is the order of the map argument, not content order.
        let map: &[(&str, &str)] = &[("axum", "Axum"), ("tokio", "Tokio")];
        let content = "tokio = \"1\"\naxum = \"0.7\"";
        let found = PolyglotAnalyzer::check_frameworks(content, map);
        // Map order: Axum first, then Tokio. Despite content order being reversed.
        assert_eq!(found, vec!["Axum".to_string(), "Tokio".to_string()]);
    }

    #[test]
    fn test_check_frameworks_duplicate_names_each_emit_once() {
        // PIN: filter+map doesn't dedupe; each map entry that matches emits one entry.
        let map: &[(&str, &str)] = &[("tokio", "Tokio"), ("tokio", "Tokio")];
        let found = PolyglotAnalyzer::check_frameworks("tokio", map);
        assert_eq!(found.len(), 2, "each map entry contributes independently");
    }

    // ── PolyglotAnalyzer::new (constructor) ─────────────────────────────────

    #[test]
    fn test_polyglot_analyzer_new_initializes_patterns() {
        let analyzer = PolyglotAnalyzer::new();
        // Constructor calls initialize_patterns + initialize_architecture_signatures.
        // Verify language_patterns has at least the major languages.
        assert!(analyzer.language_patterns.contains_key("rust"));
        assert!(!analyzer.architecture_signatures.is_empty());
    }

    #[test]
    fn test_polyglot_analyzer_new_returns_non_empty_state() {
        let a = PolyglotAnalyzer::new();
        // Sanity: > 1 language pattern, > 0 architecture signatures.
        assert!(a.language_patterns.len() > 1);
        assert!(!a.architecture_signatures.is_empty());
    }

    // ── detect_rust_frameworks (async, tempdir-fixture testable) ────────────

    #[tokio::test]
    async fn test_detect_rust_frameworks_with_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[dependencies]\ntokio = \"1\"\nserde = \"1\"\naxum = \"0.7\"\n",
        )
        .unwrap();
        let analyzer = PolyglotAnalyzer::new();
        let frameworks = analyzer.detect_rust_frameworks(tmp.path()).await.unwrap();
        assert!(frameworks.contains(&"Tokio".to_string()));
        assert!(frameworks.contains(&"Serde".to_string()));
        assert!(frameworks.contains(&"Axum".to_string()));
    }

    #[tokio::test]
    async fn test_detect_rust_frameworks_missing_cargo_returns_empty() {
        // PIN: missing Cargo.toml is graceful → empty Vec, NOT error.
        let tmp = tempfile::tempdir().unwrap();
        let analyzer = PolyglotAnalyzer::new();
        let frameworks = analyzer.detect_rust_frameworks(tmp.path()).await.unwrap();
        assert!(frameworks.is_empty());
    }

    #[tokio::test]
    async fn test_detect_rust_frameworks_no_known_frameworks() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[dependencies]\nfoo = \"1\"\nbar = \"2\"\n",
        )
        .unwrap();
        let analyzer = PolyglotAnalyzer::new();
        let frameworks = analyzer.detect_rust_frameworks(tmp.path()).await.unwrap();
        assert!(frameworks.is_empty());
    }

    #[tokio::test]
    async fn test_detect_python_frameworks_via_requirements_txt() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("requirements.txt"),
            "django==4.0\nfastapi==0.100\nrequests==2.0\n",
        )
        .unwrap();
        let analyzer = PolyglotAnalyzer::new();
        let frameworks = analyzer.detect_python_frameworks(tmp.path()).await.unwrap();
        assert!(frameworks.contains(&"Django".to_string()));
        assert!(frameworks.contains(&"FastAPI".to_string()));
    }

    #[tokio::test]
    async fn test_detect_language_frameworks_routes_by_language() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[dependencies]\nclap = \"4\"\n",
        )
        .unwrap();
        let analyzer = PolyglotAnalyzer::new();
        let result = analyzer
            .detect_language_frameworks(tmp.path(), "rust")
            .await
            .unwrap();
        assert!(result.contains(&"Clap".to_string()));
    }

    #[tokio::test]
    async fn test_detect_language_frameworks_unknown_language_returns_empty() {
        // PIN: unknown languages route to "_" arm → empty Vec (no error).
        let tmp = tempfile::tempdir().unwrap();
        let analyzer = PolyglotAnalyzer::new();
        let result = analyzer
            .detect_language_frameworks(tmp.path(), "klingon")
            .await
            .unwrap();
        assert!(result.is_empty());
    }
}
