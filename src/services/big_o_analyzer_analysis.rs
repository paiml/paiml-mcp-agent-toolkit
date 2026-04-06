// Analysis methods for BigOAnalyzer
// This file is include!()'d into big_o_analyzer.rs scope.
// NO use imports or #! inner attributes allowed.

impl BigOAnalyzer {
    /// Analyze project for algorithmic complexity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze(&self, config: BigOAnalysisConfig) -> Result<BigOAnalysisReport> {
        info!("Starting Big-O complexity analysis");
        info!("Project path: {}", config.project_path.display());

        // Discover source files
        let source_files = self.discover_source_files(&config).await?;
        info!("Found {} source files", source_files.len());

        // Analyze each file
        let mut all_functions = Vec::new();
        let mut pattern_counts = rustc_hash::FxHashMap::default();

        for file in &source_files {
            let functions = self.analyze_file(file, &config).await?;

            // Count pattern matches
            for func in &functions {
                for pattern in &func.notes {
                    if pattern.starts_with("Pattern: ") {
                        let pattern_name = pattern.trim_start_matches("Pattern: ");
                        *pattern_counts.entry(pattern_name.to_string()).or_insert(0) += 1;
                    }
                }
            }

            all_functions.extend(functions);
        }

        // Build report
        let report = self.build_report(all_functions, pattern_counts);

        info!("Big-O analysis completed");
        info!("Analyzed {} functions", report.analyzed_functions);

        Ok(report)
    }

    /// Discover source files based on patterns
    async fn discover_source_files(&self, config: &BigOAnalysisConfig) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let extensions = [
            "rs", "js", "ts", "jsx", "tsx", "py", "cpp", "c", "java", "go",
        ];
        let exclude_dirs = [
            "target",
            "node_modules",
            ".git",
            "build",
            "dist",
            "__pycache__",
        ];

        let mut files = Vec::new();

        for entry in WalkDir::new(&config.project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Check exclusions
            let should_exclude = path.components().any(|comp| {
                if let Some(name) = comp.as_os_str().to_str() {
                    exclude_dirs.contains(&name)
                } else {
                    false
                }
            });

            if should_exclude {
                continue;
            }

            // Check extensions
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    files.push(path.to_path_buf());
                }
            }
        }

        files.sort_unstable();
        Ok(files)
    }

    /// Analyze single file for complexity
    async fn analyze_file(
        &self,
        file_path: &PathBuf,
        config: &BigOAnalysisConfig,
    ) -> Result<Vec<FunctionComplexity>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let mut functions = Vec::new();

        // Detect language based on file extension
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let (pattern, lang) = Self::detect_language_pattern(extension);

        // Unknown file type, skip analysis
        if pattern.is_empty() {
            return Ok(functions);
        }

        // Use only the appropriate pattern for this file's language
        let regex = regex::Regex::new(pattern)?;
        for cap in regex.captures_iter(&content) {
            if let Some(name_match) = cap.get(cap.len() - 1) {
                let function_name = name_match.as_str().to_string();
                let line_number = content
                    .get(..name_match.start())
                    .unwrap_or_default()
                    .lines()
                    .count();

                // Analyze function complexity
                let complexity = self.analyze_function_complexity(
                    &function_name,
                    content.get(name_match.start()..).unwrap_or_default(),
                    lang,
                );

                if complexity.confidence >= config.confidence_threshold {
                    functions.push(FunctionComplexity {
                        file_path: file_path.clone(),
                        function_name,
                        line_number,
                        time_complexity: complexity.time_complexity,
                        space_complexity: complexity.space_complexity,
                        confidence: complexity.confidence,
                        notes: complexity.notes,
                    });
                }
            }
        }

        Ok(functions)
    }

    /// Detect language pattern and name from file extension
    fn detect_language_pattern(extension: &str) -> (&'static str, &'static str) {
        match extension {
            "rs" => (r"fn\s+(\w+)", "rust"),
            "js" | "jsx" => (r"function\s+(\w+)", "javascript"),
            "ts" | "tsx" => (r"function\s+(\w+)", "typescript"),
            "py" => (r"def\s+(\w+)", "python"),
            "go" => (r"func\s+(\w+)", "go"),
            "java" => (
                r"(public|private|protected)?\s*(static)?\s*\w+\s+(\w+)\s*\(",
                "java",
            ),
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "cu" | "cuh" => (
                r"(?:int|void|bool|char|float|double|long|short|unsigned|static|const)*\s+(\w+)\s*\([^)]*\)\s*\{",
                "c",
            ),
            _ => ("", ""),
        }
    }

    /// Analyze function complexity using patterns and heuristics
    fn analyze_function_complexity(
        &self,
        function_name: &str,
        function_body: &str,
        language: &str,
    ) -> ComplexityAnalysisResult {
        let mut notes = Vec::new();
        let lines: Vec<&str> = function_body.lines().take(100).collect();

        // Get language-specific loop keywords
        let loop_keywords = Self::get_loop_keywords(language);

        // Calculate loop depth
        let max_loop_depth = Self::calculate_loop_depth(&lines, &loop_keywords);

        // Check for iterator patterns in Rust (these are linear operations)
        let mut has_iterator_pattern = false;
        if language == "rust" {
            has_iterator_pattern = Self::detect_rust_iterator_patterns(function_body);
            if has_iterator_pattern {
                notes.push("Iterator pattern detected (linear)".to_string());
            }
        }

        // Check for patterns
        let mut has_recursion = false;
        let mut has_sorting = false;

        for line in &lines {
            if Self::detect_recursive_call(line, function_name) {
                has_recursion = true;
                notes.push("Recursive function detected".to_string());
            }

            if Self::detect_sorting_operation(line) {
                has_sorting = true;
                notes.push("Pattern: Sorting operation".to_string());
            }

            if Self::detect_binary_search(line) {
                notes.push("Pattern: Binary search".to_string());
            }
        }

        // Determine time complexity
        let mut time_complexity = Self::determine_time_complexity(max_loop_depth, has_recursion);

        // Adjust for iterator patterns - these are linear operations
        if has_iterator_pattern
            && (time_complexity.class == BigOClass::Constant
                || time_complexity.class == BigOClass::Unknown)
        {
            time_complexity = ComplexityBound::linear().with_confidence(75);
        }

        // Adjust for sorting operations
        if has_sorting
            && time_complexity
                .class
                .is_better_than(&BigOClass::Linearithmic)
        {
            time_complexity = ComplexityBound::linearithmic();
        }

        // Determine space complexity
        let (space_complexity, has_allocation) = Self::detect_space_complexity(function_body);
        if has_allocation {
            notes.push("Dynamic memory allocation detected".to_string());
        }

        ComplexityAnalysisResult {
            time_complexity,
            space_complexity,
            matched_patterns: Vec::new(),
            confidence: (time_complexity.confidence + space_complexity.confidence) / 2,
            notes,
        }
    }
}
