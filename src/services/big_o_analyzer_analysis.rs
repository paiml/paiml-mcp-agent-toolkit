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

        // #1015: with no source file discovered this went on to print
        // "Total Functions Analyzed: 0" over a complexity distribution of eight
        // zeros and exit 0 — the identical document a tree of nothing but O(1)
        // functions would produce, and the identical document a `--include`
        // pattern that matches nothing produces. A distribution over an empty
        // population is not a measurement. Refused here rather than in the CLI
        // handler so every consumer of the analyzer (the handler, the unified
        // context builder) is told the same thing.
        // "big-O" and not "big-O complexity": the binary's `categorize_error`
        // greps the rendered message for the substring "complexity" and would
        // turn this one refusal into exit 5 while the other seven — and
        // `analyze satd`, which this sentence is copied from — exit 1.
        crate::cli::ensure_source_files_were_analyzed(
            "big-O",
            &config.project_path,
            source_files.len(),
        )?;

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

    /// Source extensions big-O analysis knows how to read.
    const SOURCE_EXTENSIONS: [&str; 10] = [
        "rs", "js", "ts", "jsx", "tsx", "py", "cpp", "c", "java", "go",
    ];

    /// Discover source files based on patterns
    ///
    /// This used to be a bare `walkdir::WalkDir` with a six-entry exclude
    /// list (target/node_modules/.git/build/dist/__pycache__). It therefore
    /// descended into hidden and gitignored trees that `analyze complexity`
    /// never sees — `.claude/worktrees` above all — and reported 2,650,897
    /// functions on a repo where complexity reported 39,907, and 6 functions on
    /// a fixture holding exactly one. File discovery is now the shared
    /// `ProjectFileDiscovery` (gitignore-aware, hidden directories skipped), so
    /// every analyzer is asked about the same corpus.
    async fn discover_source_files(&self, config: &BigOAnalysisConfig) -> Result<Vec<PathBuf>> {
        let discovery =
            crate::services::file_discovery::ProjectFileDiscovery::new(config.project_path.clone());

        // `--include`/`--exclude` were carried in the config and never read, so
        // `--exclude '**/*.rs'` on a Rust-only crate still analysed every function
        // and `--include '**/*.py'` still returned the Rust ones. They are filters now.
        let include_set = Self::build_pattern_set(&config.include_patterns)?;
        let exclude_set = Self::build_pattern_set(&config.exclude_patterns)?;

        let mut files: Vec<PathBuf> = discovery
            .discover_files()?
            .into_iter()
            .filter(|path| {
                path.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| Self::SOURCE_EXTENSIONS.contains(&ext))
            })
            .filter(|path| {
                if let Some(ref set) = exclude_set {
                    if Self::pattern_matches(set, path, &config.project_path) {
                        return false;
                    }
                }
                match include_set {
                    Some(ref set) => Self::pattern_matches(set, path, &config.project_path),
                    None => true,
                }
            })
            .collect();

        files.sort_unstable();
        Ok(files)
    }

    /// Compile user-supplied globs, or `None` when no pattern was given.
    fn build_pattern_set(patterns: &[String]) -> Result<Option<globset::GlobSet>> {
        if patterns.is_empty() {
            return Ok(None);
        }
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(globset::Glob::new(pattern)?);
        }
        Ok(Some(builder.build()?))
    }

    /// Match a discovered file against a pattern set.
    ///
    /// Tested against the project-relative path, the full path and the bare file
    /// name, so `src/**`, `**/*.rs` and `lib.rs` all mean what a user expects.
    fn pattern_matches(
        set: &globset::GlobSet,
        path: &std::path::Path,
        project_path: &std::path::Path,
    ) -> bool {
        let relative = path.strip_prefix(project_path).unwrap_or(path);
        set.is_match(relative)
            || set.is_match(path)
            || path
                .file_name()
                .is_some_and(|name| set.is_match(std::path::Path::new(name)))
    }

    /// Analyze single file for complexity
    ///
    /// #655/#661: this used to run a bare `fn\s+(\w+)` regex over the raw file
    /// and then hand `content[name_start..]` — *the entire rest of the file* —
    /// to the body analyzer. Two consequences:
    ///   * every function in a file inherited the deepest loop nest that
    ///     appeared anywhere below it, so a loop-free `a + b` was reported
    ///     O(n^2) with the same confidence as the nested-loop function under it;
    ///   * the regex matched comment text, inventing functions named "2" and "3"
    ///     out of `/// fn 2: ...` doc lines.
    ///
    /// Function boundaries now come from the same `LanguageAnalyzer` the
    /// complexity path uses, and each function sees only its own body.
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

        let (_pattern, lang) = Self::detect_language_pattern(extension);
        let language = crate::cli::language_analyzer::Language::from_path(file_path);

        // Unknown file type, skip analysis
        if lang.is_empty() || language == crate::cli::language_analyzer::Language::Unknown {
            return Ok(functions);
        }

        let extractor = crate::cli::language_analyzer::create_analyzer(language);
        let lines: Vec<&str> = content.lines().collect();

        for info in extractor.extract_functions(&content) {
            let Some(body) = Self::function_body(&lines, info.line_start, info.line_end) else {
                continue;
            };

            let complexity = self.analyze_function_complexity(&info.name, &body, lang);

            if complexity.confidence >= config.confidence_threshold {
                functions.push(FunctionComplexity {
                    file_path: file_path.clone(),
                    function_name: info.name,
                    // 1-based real source line, not a prefix line count that
                    // rendered a function at the top of a file as "line 0".
                    line_number: info.line_start + 1,
                    time_complexity: complexity.time_complexity,
                    space_complexity: complexity.space_complexity,
                    confidence: complexity.confidence,
                    notes: complexity.notes,
                });
            }
        }

        Ok(functions)
    }

    /// Join the source lines a function actually occupies (0-based, inclusive).
    fn function_body(lines: &[&str], start: usize, end: usize) -> Option<String> {
        let last = lines.len().checked_sub(1)?;
        let end = end.min(last);
        if start > end {
            return None;
        }
        Some(lines[start..=end].join("\n"))
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
        // No `.take(100)` any more: `function_body` is now this function's body,
        // not the rest of the file, so there is nothing to truncate away.
        let lines: Vec<&str> = function_body.lines().collect();

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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod discovery_scope_tests {
    //! Big-O discovery must see the same corpus as `analyze complexity`:
    //! a raw WalkDir walked hidden tool caches such as `.claude/worktrees`
    //! and reported 6 functions for a fixture holding exactly one.
    use super::*;

    fn config(root: &std::path::Path) -> BigOAnalysisConfig {
        BigOAnalysisConfig {
            project_path: root.to_path_buf(),
            include_patterns: vec![],
            exclude_patterns: vec![],
            confidence_threshold: 0,
            analyze_space_complexity: true,
        }
    }

    #[tokio::test]
    async fn discovery_skips_hidden_tool_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join(".claude/junk")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn a() -> i32 { 1 }\n").unwrap();
        for i in 0..5 {
            std::fs::write(
                root.join(format!(".claude/junk/h{i}.rs")),
                format!("pub fn h{i}() -> i32 {{ {i} }}\n"),
            )
            .unwrap();
        }

        let analyzer = BigOAnalyzer::new();
        let files = analyzer
            .discover_source_files(&config(root))
            .await
            .expect("discovery");

        assert_eq!(
            files.len(),
            1,
            "only src/lib.rs is project source; got {files:?}"
        );
        assert!(files[0].ends_with("src/lib.rs"));

        let report = analyzer.analyze(config(root)).await.expect("analysis");
        assert_eq!(report.analyzed_functions, 1, "one file, one function");
    }

    #[tokio::test]
    async fn discovery_skips_gitignored_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("generated")).unwrap();
        // `.git` must exist for gitignore rules to apply, exactly as in a checkout.
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join(".gitignore"), "generated/\n").unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn a() -> i32 { 1 }\n").unwrap();
        std::fs::write(root.join("generated/g.rs"), "pub fn g() -> i32 { 2 }\n").unwrap();

        let files = BigOAnalyzer::new()
            .discover_source_files(&config(root))
            .await
            .expect("discovery");

        assert_eq!(files.len(), 1, "gitignored trees are not project source: {files:?}");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod pattern_filter_tests {
    //! `--include`/`--exclude` were stored in the config and never read, so
    //! `--exclude '**/*.rs'` on a Rust-only crate still analysed every function.
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn a() -> i32 { 1 }\n").unwrap();
        dir
    }

    fn config_with(
        root: &std::path::Path,
        include: Vec<&str>,
        exclude: Vec<&str>,
    ) -> BigOAnalysisConfig {
        BigOAnalysisConfig {
            project_path: root.to_path_buf(),
            include_patterns: include.into_iter().map(String::from).collect(),
            exclude_patterns: exclude.into_iter().map(String::from).collect(),
            confidence_threshold: 0,
            analyze_space_complexity: false,
        }
    }

    async fn discovered(config: &BigOAnalysisConfig) -> usize {
        BigOAnalyzer::new()
            .discover_source_files(config)
            .await
            .expect("discovery")
            .len()
    }

    #[tokio::test]
    async fn exclude_patterns_remove_files() {
        let dir = fixture();
        let root = dir.path();

        assert_eq!(discovered(&config_with(root, vec![], vec![])).await, 1);

        for pattern in ["**/*.rs", "*.rs", "lib.rs", "src/**", "src/lib.rs"] {
            assert_eq!(
                discovered(&config_with(root, vec![], vec![pattern])).await,
                0,
                "--exclude '{pattern}' must drop the only Rust file"
            );
        }
    }

    #[tokio::test]
    async fn include_patterns_restrict_files() {
        let dir = fixture();
        let root = dir.path();

        for pattern in ["**/*.py", "nothing_matches_zzz", "*.py"] {
            assert_eq!(
                discovered(&config_with(root, vec![pattern], vec![])).await,
                0,
                "--include '{pattern}' must match nothing in a Rust-only crate"
            );
        }

        for pattern in ["**/*.rs", "lib.rs", "src/lib.rs"] {
            assert_eq!(
                discovered(&config_with(root, vec![pattern], vec![])).await,
                1,
                "--include '{pattern}' must keep the matching file"
            );
        }
    }
}
