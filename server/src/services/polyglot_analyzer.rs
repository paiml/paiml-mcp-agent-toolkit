use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub file_count: usize,
    pub line_count: usize,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotAnalysis {
    pub languages: Vec<LanguageStats>,
    pub cross_language_dependencies: Vec<CrossLanguageDependency>,
    pub architecture_pattern: Option<ArchitecturePattern>,
    pub integration_points: Vec<IntegrationPoint>,
    pub recommendation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub file_count: usize,
    pub line_count: usize,
    pub complexity_score: f64,
    pub test_coverage: f64,
    pub primary_frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLanguageDependency {
    pub from_language: String,
    pub to_language: String,
    pub dependency_type: DependencyType,
    pub coupling_strength: f64,
    pub files_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    FFI,
    ProcessCommunication,
    SharedDataStructure,
    ConfigurationFile,
    BuildSystem,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturePattern {
    Microservices,
    Monolithic,
    LayeredArchitecture,
    EventDriven,
    PluginArchitecture,
    ClientServer,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPoint {
    pub name: String,
    pub languages: Vec<String>,
    pub integration_type: IntegrationType,
    pub risk_level: RiskLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    API,
    Database,
    FileSystem,
    Memory,
    Network,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct PolyglotAnalyzer {
    language_patterns: HashMap<String, LanguagePattern>,
    architecture_signatures: Vec<ArchitectureSignature>,
}

#[derive(Debug, Clone)]
struct LanguagePattern {
    file_extensions: Vec<String>,
    _build_files: Vec<String>,
    _config_files: Vec<String>,
    _dependency_files: Vec<String>,
}

#[derive(Debug, Clone)]
struct ArchitectureSignature {
    pattern: ArchitecturePattern,
    _indicators: Vec<String>,
    required_languages: usize,
    confidence_threshold: f64,
}

#[derive(Debug, Clone)]
struct ArchitectureIndicators {
    has_microservice_indicators: bool,
    has_layered_indicators: bool,
    has_event_indicators: bool,
    has_plugin_indicators: bool,
    directory_structure: Vec<String>,
    config_files: Vec<String>,
}

impl PolyglotAnalyzer {
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
            let full_ext = format!(".{}", ext);
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
        framework_map
            .iter()
            .filter(|(search_term, _)| content.contains(search_term))
            .map(|(_, name)| name.to_string())
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
            ("javascript", "python")
            | ("python", "javascript")
            | ("typescript", "python")
            | ("python", "typescript") => {
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
            files_involved.push(format!("{} JS + {} TS files", js_files, ts_files));
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
                files_involved.push(api_file.to_string());
            }
        }

        // Look for shared data schemas
        for schema_file in &["schema.json", "types.json", "models.json"] {
            if project_path.join(schema_file).exists() {
                coupling_strength += 0.3;
                files_involved.push(schema_file.to_string());
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
                                from_language: lang1.to_string(),
                                to_language: lang2.to_string(),
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
        let mut dependencies = Vec::new();

        // Check for shared configuration files
        for config_file in &["config.json", "settings.yaml", ".env", "app.config"] {
            if project_path.join(config_file).exists() {
                let languages: Vec<_> = language_info.keys().collect();
                if languages.len() >= 2 {
                    for (i, &lang1) in languages.iter().enumerate() {
                        for &lang2 in languages.iter().skip(i + 1) {
                            dependencies.push(CrossLanguageDependency {
                                from_language: lang1.to_string(),
                                to_language: lang2.to_string(),
                                dependency_type: DependencyType::ConfigurationFile,
                                coupling_strength: 0.4,
                                files_involved: vec![config_file.to_string()],
                            });
                        }
                    }
                }
                break; // Only count one shared config file
            }
        }

        Ok(dependencies)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn count_files_recursive(
        &self,
        dir_path: &Path,
        extensions: &[String],
        count: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(
                            dir_name,
                            "node_modules"
                                | "target"
                                | "build"
                                | ".git"
                                | "__pycache__"
                                | ".venv"
                                | "venv"
                        ) {
                            continue;
                        }
                    }
                    self.count_files_recursive(&path, extensions, count)?;
                } else if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if extensions.contains(&ext.to_string()) {
                            *count += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn has_potential_integration(&self, lang1: &str, lang2: &str) -> bool {
        matches!(
            (lang1, lang2),
            ("rust", "python")
                | ("python", "rust")
                | ("typescript", "rust")
                | ("rust", "typescript")
                | ("javascript", "python")
                | ("python", "javascript")
                | ("typescript", "javascript")
                | ("javascript", "typescript")
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
            insights.push(format!("Architecture pattern: {:?}", pattern));
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

/// Toyota Way: Extract Method - Check if directory should be skipped (complexity ≤3)
fn should_skip_directory(path: &Path) -> bool {
    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        matches!(
            dir_name,
            "node_modules"
                | "target"
                | "build"
                | ".git"
                | "__pycache__"
                | ".venv"
                | "venv"
        )
    } else {
        false
    }
}

impl Default for PolyglotAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polyglot_analyzer_creation() {
        let analyzer = PolyglotAnalyzer::new();
        assert!(!analyzer.language_patterns.is_empty());
        assert!(!analyzer.architecture_signatures.is_empty());
    }

    #[test]
    fn test_language_pattern_initialization() {
        let analyzer = PolyglotAnalyzer::new();
        let rust_pattern = analyzer.language_patterns.get("rust").unwrap();
        assert!(rust_pattern.file_extensions.contains(&".rs".to_string()));
        assert!(rust_pattern
            ._build_files
            .contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_has_potential_integration() {
        let analyzer = PolyglotAnalyzer::new();
        assert!(analyzer.has_potential_integration("rust", "python"));
        assert!(analyzer.has_potential_integration("typescript", "javascript"));
        assert!(!analyzer.has_potential_integration("rust", "rust"));
    }

    #[test]
    fn test_dependency_type_inference() {
        let analyzer = PolyglotAnalyzer::new();
        let dep_type = analyzer.infer_dependency_type("rust", "python");
        assert!(matches!(dep_type, DependencyType::FFI));
    }

    #[test]
    fn test_risk_level_assessment() {
        let analyzer = PolyglotAnalyzer::new();
        assert!(matches!(
            analyzer.assess_risk_level(0.9),
            RiskLevel::Critical
        ));
        assert!(matches!(analyzer.assess_risk_level(0.7), RiskLevel::High));
        assert!(matches!(analyzer.assess_risk_level(0.5), RiskLevel::Medium));
        assert!(matches!(analyzer.assess_risk_level(0.2), RiskLevel::Low));
    }

    #[test]
    fn test_complexity_score_calculation() {
        let analyzer = PolyglotAnalyzer::new();
        let language_info = LanguageInfo {
            name: "rust".to_string(),
            file_count: 10,
            line_count: 1000,
            frameworks: vec![],
        };

        let score = analyzer.calculate_language_complexity_score(&language_info);
        assert!((1.0..=10.0).contains(&score));
    }

    #[test]
    fn test_integration_point_mapping() {
        let analyzer = PolyglotAnalyzer::new();
        let integration_type = analyzer.map_dependency_to_integration(&DependencyType::FFI);
        assert!(matches!(integration_type, IntegrationType::Memory));
    }

    #[test]
    fn test_recommendation_score_calculation() {
        let analyzer = PolyglotAnalyzer::new();
        let language_stats = vec![LanguageStats {
            language: "rust".to_string(),
            file_count: 10,
            line_count: 2000,
            complexity_score: 5.0,
            test_coverage: 0.8,
            primary_frameworks: vec![],
        }];

        let score = analyzer.calculate_recommendation_score(
            &language_stats,
            &[],
            &Some(ArchitecturePattern::Monolithic),
        );
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_polyglot_insights_generation() {
        let analyzer = PolyglotAnalyzer::new();
        let analysis = PolyglotAnalysis {
            languages: vec![
                LanguageStats {
                    language: "rust".to_string(),
                    file_count: 5,
                    line_count: 1000,
                    complexity_score: 5.0,
                    test_coverage: 0.8,
                    primary_frameworks: vec![],
                },
                LanguageStats {
                    language: "python".to_string(),
                    file_count: 3,
                    line_count: 500,
                    complexity_score: 3.0,
                    test_coverage: 0.7,
                    primary_frameworks: vec![],
                },
                LanguageStats {
                    language: "typescript".to_string(),
                    file_count: 2,
                    line_count: 300,
                    complexity_score: 4.0,
                    test_coverage: 0.6,
                    primary_frameworks: vec![],
                },
            ],
            cross_language_dependencies: vec![],
            architecture_pattern: Some(ArchitecturePattern::Mixed),
            integration_points: vec![],
            recommendation_score: 0.8,
        };

        let insights = analyzer.generate_polyglot_insights(&analysis);
        assert!(!insights.is_empty());
        assert!(insights.iter().any(|i| i.contains("polyglot project")));
        assert!(insights
            .iter()
            .any(|i| i.contains("Primary language: rust")));
    }
}
