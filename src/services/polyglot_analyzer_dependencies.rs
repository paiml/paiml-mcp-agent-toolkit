const SKIPPED_DIRS: &[&str] = &[
    "node_modules", "target", "build", ".git", "__pycache__", ".venv", "venv",
];

fn is_skipped_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map_or(false, |name| SKIPPED_DIRS.contains(&name))
}

fn build_config_dependency_pairs(languages: &[&String], config_file: &str) -> Vec<CrossLanguageDependency> {
    let mut deps = Vec::new();
    for (i, &lang1) in languages.iter().enumerate() {
        for &lang2 in languages.iter().skip(i + 1) {
            deps.push(CrossLanguageDependency {
                from_language: lang1.clone(),
                to_language: lang2.clone(),
                dependency_type: DependencyType::ConfigurationFile,
                coupling_strength: 0.4,
                files_involved: vec![config_file.to_string()],
            });
        }
    }
    deps
}

impl PolyglotAnalyzer {
    async fn analyze_cross_language_dependencies(
        &self,
        project_path: &Path,
        language_info: &HashMap<String, LanguageInfo>,
    ) -> Result<Vec<CrossLanguageDependency>, Box<dyn std::error::Error>> {
        let mut dependencies = Vec::new();
        let languages: Vec<_> = language_info.keys().collect();

        for (i, lang1) in languages.iter().enumerate() {
            for lang2 in languages.iter().skip(i + 1) {
                if let Some(dep) = self
                    .analyze_language_pair(project_path, lang1, lang2)
                    .await?
                {
                    dependencies.push(dep);
                }
            }
        }

        // Also analyze build system and configuration dependencies
        dependencies.extend(
            self.analyze_build_system_dependencies(project_path, language_info)
                .await?,
        );
        dependencies.extend(
            self.analyze_configuration_dependencies(project_path, language_info)
                .await?,
        );

        Ok(dependencies)
    }

    async fn analyze_language_pair(
        &self,
        project_path: &Path,
        lang1: &str,
        lang2: &str,
    ) -> Result<Option<CrossLanguageDependency>, Box<dyn std::error::Error>> {
        if !self.has_potential_integration(lang1, lang2) {
            return Ok(None);
        }

        let mut files_involved = Vec::new();
        let dependency_type = self.infer_dependency_type(lang1, lang2);

        // Analyze actual file interactions
        let coupling_strength = match (lang1, lang2) {
            ("rust", "python") | ("python", "rust") => {
                // Look for PyO3 bindings or ctypes usage
                self.analyze_rust_python_integration(project_path, &mut files_involved)
                    .await?
            }
            ("typescript", "javascript") | ("javascript", "typescript") => {
                // Look for shared configurations and imports
                self.analyze_js_ts_integration(project_path, &mut files_involved)
                    .await?
            }
            ("javascript" | "typescript", "python") | ("python", "javascript" | "typescript") => {
                // Look for API boundaries and shared data formats
                self.analyze_api_integration(project_path, &mut files_involved)
                    .await?
            }
            _ => {
                files_involved.push("shared config".to_string());
                0.3 // Default moderate coupling
            }
        };

        if coupling_strength > 0.1 {
            Ok(Some(CrossLanguageDependency {
                from_language: lang1.to_string(),
                to_language: lang2.to_string(),
                dependency_type,
                coupling_strength,
                files_involved,
            }))
        } else {
            Ok(None)
        }
    }

    async fn analyze_rust_python_integration(
        &self,
        project_path: &Path,
        files_involved: &mut Vec<String>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let mut coupling_strength: f64 = 0.0;

        // Check Cargo.toml for PyO3
        if let Ok(cargo_toml) = std::fs::read_to_string(project_path.join("Cargo.toml")) {
            if cargo_toml.contains("pyo3") {
                coupling_strength += 0.7;
                files_involved.push("Cargo.toml (PyO3)".to_string());
            }
        }

        // Check for Python setup files that reference Rust
        if let Ok(setup_py) = std::fs::read_to_string(project_path.join("setup.py")) {
            if setup_py.contains("rust") || setup_py.contains("cargo") {
                coupling_strength += 0.5;
                files_involved.push("setup.py".to_string());
            }
        }

        // Check for .so files or build artifacts
        if project_path.join("target").exists() && project_path.join("__pycache__").exists() {
            coupling_strength += 0.3;
            files_involved.push("build artifacts".to_string());
        }

        Ok(coupling_strength.min(1.0))
    }

    async fn analyze_js_ts_integration(
        &self,
        project_path: &Path,
        files_involved: &mut Vec<String>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let mut coupling_strength: f64 = 0.0;

        // Check for TypeScript config
        if project_path.join("tsconfig.json").exists() {
            coupling_strength += 0.6;
            files_involved.push("tsconfig.json".to_string());
        }

        // Check package.json for TypeScript dependencies
        if let Ok(package_json) = std::fs::read_to_string(project_path.join("package.json")) {
            if package_json.contains("typescript") {
                coupling_strength += 0.4;
                files_involved.push("package.json (TypeScript)".to_string());
            }
        }

        // Check for mixed .js and .ts files
        let mut js_files = 0;
        let mut ts_files = 0;
        self.count_files_recursive(project_path, &["js".to_string()], &mut js_files)?;
        self.count_files_recursive(project_path, &["ts".to_string()], &mut ts_files)?;

        if js_files > 0 && ts_files > 0 {
            coupling_strength += 0.3;
            files_involved.push(format!("{js_files} JS + {ts_files} TS files"));
        }

        Ok(coupling_strength.min(1.0))
    }

    async fn analyze_api_integration(
        &self,
        project_path: &Path,
        files_involved: &mut Vec<String>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let mut coupling_strength: f64 = 0.0;

        // Look for API specification files
        for api_file in &[
            "openapi.yaml",
            "openapi.json",
            "swagger.yaml",
            "swagger.json",
            "api.yaml",
        ] {
            if project_path.join(api_file).exists() {
                coupling_strength += 0.5;
                files_involved.push((*api_file).to_string());
            }
        }

        // Look for shared data schemas
        for schema_file in &["schema.json", "types.json", "models.json"] {
            if project_path.join(schema_file).exists() {
                coupling_strength += 0.3;
                files_involved.push((*schema_file).to_string());
            }
        }

        // Look for Docker Compose indicating microservices
        if project_path.join("docker-compose.yml").exists()
            || project_path.join("docker-compose.yaml").exists()
        {
            coupling_strength += 0.4;
            files_involved.push("docker-compose".to_string());
        }

        Ok(coupling_strength.min(1.0))
    }

    async fn analyze_build_system_dependencies(
        &self,
        project_path: &Path,
        language_info: &HashMap<String, LanguageInfo>,
    ) -> Result<Vec<CrossLanguageDependency>, Box<dyn std::error::Error>> {
        let mut dependencies = Vec::new();

        // Check for multi-language build systems
        if project_path.join("Makefile").exists() {
            if let Ok(makefile) = std::fs::read_to_string(project_path.join("Makefile")) {
                let languages_in_make: Vec<_> = language_info
                    .keys()
                    .filter(|&lang| makefile.contains(lang))
                    .collect();

                if languages_in_make.len() >= 2 {
                    for (i, &lang1) in languages_in_make.iter().enumerate() {
                        for &lang2 in languages_in_make.iter().skip(i + 1) {
                            dependencies.push(CrossLanguageDependency {
                                from_language: lang1.clone(),
                                to_language: lang2.clone(),
                                dependency_type: DependencyType::BuildSystem,
                                coupling_strength: 0.6,
                                files_involved: vec!["Makefile".to_string()],
                            });
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    async fn analyze_configuration_dependencies(
        &self,
        project_path: &Path,
        language_info: &HashMap<String, LanguageInfo>,
    ) -> Result<Vec<CrossLanguageDependency>, Box<dyn std::error::Error>> {
        let config_file = ["config.json", "settings.yaml", ".env", "app.config"]
            .iter()
            .find(|f| project_path.join(f).exists());

        let Some(config_file) = config_file else {
            return Ok(Vec::new());
        };

        let languages: Vec<_> = language_info.keys().collect();
        if languages.len() < 2 {
            return Ok(Vec::new());
        }

        let dependencies = build_config_dependency_pairs(&languages, config_file);
        Ok(dependencies)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn count_files_recursive(
        &self,
        dir_path: &Path,
        extensions: &[String],
        count: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entries = match std::fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if !is_skipped_dir(&path) {
                    self.count_files_recursive(&path, extensions, count)?;
                }
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_string()) {
                    *count += 1;
                }
            }
        }
        Ok(())
    }

    fn has_potential_integration(&self, lang1: &str, lang2: &str) -> bool {
        matches!(
            (lang1, lang2),
            ("rust" | "javascript", "python" | "typescript")
                | ("python" | "typescript", "rust" | "javascript")
        )
    }

    fn infer_dependency_type(&self, lang1: &str, lang2: &str) -> DependencyType {
        match (lang1, lang2) {
            ("rust", "python") | ("python", "rust") => DependencyType::FFI,
            ("typescript", "javascript") | ("javascript", "typescript") => {
                DependencyType::SharedDataStructure
            }
            _ => DependencyType::ProcessCommunication,
        }
    }

}
