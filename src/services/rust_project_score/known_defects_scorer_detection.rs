// Detection logic for known defect patterns
// included from known_defects_scorer.rs - shares parent module scope

impl KnownDefectsScorer {
    /// Count unwrap() calls in production code (excluding tests)
    ///
    /// Returns (production_unwraps, test_unwraps)
    fn count_unwraps(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<(usize, usize)> {
        let unwrap_regex =
            Regex::new(r"\.unwrap\(\)").map_err(|e| ScorerError::IoError(e.to_string()))?;

        let mut production_count = 0;
        let mut test_count = 0;

        // Get all .rs files from cache or filesystem
        if let Some(cache) = cache {
            for (path, content) in cache.iter() {
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        let (prod, test) =
                            Self::count_unwraps_in_file(path, content, &unwrap_regex);
                        production_count += prod;
                        test_count += test;
                    }
                }
            }
        } else {
            // Fallback: walk filesystem (not cached)
            use walkdir::WalkDir;

            for entry in WalkDir::new(project_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let (prod, test) =
                                Self::count_unwraps_in_file(path, &content, &unwrap_regex);
                            production_count += prod;
                            test_count += test;
                        }
                    }
                }
            }
        }

        Ok((production_count, test_count))
    }

    /// Count unwrap() calls in a single file, separating production from test code
    ///
    /// Returns (production_unwraps, test_unwraps)
    ///
    /// **Fixes for GitHub Issues #99 and #100:**
    /// - #99: Excludes doc comments (`///`, `//!`, `//`, `/* */`)
    /// - #100: Properly detects inline `#[cfg(test)]` modules
    fn count_unwraps_in_file(path: &Path, content: &str, unwrap_regex: &Regex) -> (usize, usize) {
        // Check if entire file is test code
        if Self::is_test_file(path) {
            let test_count = unwrap_regex.find_iter(content).count();
            return (0, test_count);
        }

        // Strip comments before counting (fixes #99 - doc comment false positives)
        let code_only = Self::strip_comments(content);

        // Production file - check for #[cfg(test)] module
        // Find any test-related cfg marker: #[cfg(test)], #[cfg(all(test, ...))]
        let test_cfg_regex = Regex::new(r"#\[cfg\((test|all\(test)").ok();
        let test_module_start = test_cfg_regex
            .as_ref()
            .and_then(|re| re.find(&code_only))
            .map(|m| m.start());

        match test_module_start {
            Some(start_pos) => {
                // Split content at test module boundary
                let production_code = &code_only[..start_pos];
                let test_code = &code_only[start_pos..];

                let production_count = unwrap_regex.find_iter(production_code).count();
                let test_count = unwrap_regex.find_iter(test_code).count();

                (production_count, test_count)
            }
            None => {
                // No test module found - all production code
                let production_count = unwrap_regex.find_iter(&code_only).count();
                (production_count, 0)
            }
        }
    }

    /// Strip comments from Rust source code (fixes #99)
    ///
    /// Removes:
    /// - Line comments: `//` (including doc comments `///` and `//!`)
    /// - Block comments: `/* */` (including doc comments `/** */`)
    fn strip_comments(content: &str) -> String {
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut in_block_comment = false;
        let mut in_string = false;
        let mut escape_next = false;

        while let Some(c) = chars.next() {
            if escape_next {
                escape_next = false;
                if !in_block_comment {
                    result.push(c);
                }
                continue;
            }

            if c == '\\' && in_string {
                escape_next = true;
                result.push(c);
                continue;
            }

            if c == '"' && !in_block_comment {
                in_string = !in_string;
                result.push(c);
                continue;
            }

            if in_string {
                result.push(c);
                continue;
            }

            if in_block_comment {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // consume '/'
                    in_block_comment = false;
                }
                continue;
            }

            if c == '/' {
                match chars.peek() {
                    Some(&'/') => {
                        // Line comment - skip to end of line
                        for nc in chars.by_ref() {
                            if nc == '\n' {
                                result.push('\n');
                                break;
                            }
                        }
                    }
                    Some(&'*') => {
                        // Block comment
                        chars.next(); // consume '*'
                        in_block_comment = true;
                    }
                    _ => {
                        result.push(c);
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Determine if a file is a test file
    ///
    /// **Heuristics:**
    /// 1. Path contains `/tests/`, `/benches/`, or `/src/tests/` directory
    /// 2. Filename ends with `_test.rs`, `_tests.rs`, or `tests.rs`
    ///
    /// **Note:** This does NOT check for `#[cfg(test)]` modules within production files.
    /// Trade-off: unwrap() calls inside `#[cfg(test)]` modules in production files
    /// will be counted as production code. This is acceptable because:
    /// - It's rare (best practice is separate test files)
    /// - It's conservative (better to over-count than miss production unwraps)
    /// - It encourages proper test organization
    fn is_test_file(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check 1: Directory structure
        // Note: /src/tests/ is common in pmat (contains test modules)
        // Fixes #234: exclude examples/ and book/ from production unwrap count
        if path_str.contains("/tests/")
            || path_str.contains("/benches/")
            || path_str.contains("/src/tests/")
            || path_str.contains("/examples/")
            || path_str.contains("/book/")
        {
            return true;
        }

        // Check 2: Filename patterns (expanded for common test file naming conventions)
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            let filename_lower = filename_str.to_lowercase();
            if filename_str.ends_with("_test.rs")
                || filename_str.ends_with("_tests.rs")
                || filename_str == "tests.rs"
                || filename_lower.contains("_tests_")        // e.g., *_tests_part1.rs
                || filename_lower.contains("coverage_tests") // e.g., *_coverage_tests*.rs
                || filename_lower.contains("property_tests") // e.g., *_property_tests.rs
                || filename_lower.contains("_coverage_")     // e.g., *_coverage_part*.rs
                || filename_lower.starts_with("test_")       // e.g., test_*.rs
                || filename_lower.starts_with("tests_")      // e.g., tests_core_part3.rs
                || filename_lower.starts_with("coverage_")
            // e.g., coverage_part4.rs (include!'d tests)
            {
                return true;
            }
            // Handle split test files: part1.rs, part2.rs, etc.
            if filename_lower.starts_with("part") && filename_lower.ends_with(".rs") {
                return true;
            }
        }

        false
    }
}
