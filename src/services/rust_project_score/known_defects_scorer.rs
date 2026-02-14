#![cfg_attr(coverage_nightly, coverage(off))]
//! Known Defects Scorer - Production Defect Pattern Detection (20 points)
//!
//! Detects known defect patterns that have caused production incidents.
//!
//! ## Scoring (20 points total)
//!
//! - **Base Score**: 20 points (perfect - zero production defects)
//! - **unwrap() Penalty**: -5 points per 100 unwrap() calls in production code
//! - **Minimum Score**: 0 points (cannot go negative)
//!
//! ## Defect Patterns Detected
//!
//! ### 1. unwrap() in Production Code (Cloudflare Incident 2025-11-18)
//!
//! **Incident**: Cloudflare's worst outage since 2019 caused by uncaught panic from `.unwrap()`
//!
//! **Root Cause**:
//! ```rust
//! // Cloudflare's code that caused the outage
//! thread fl2_worker_thread panicked: called Result::unwrap() on an Err value
//! ```
//!
//! **Impact**:
//! - Network unavailable from 11:20-14:30 UTC (3+ hours)
//! - HTTP 5xx errors for all customer traffic
//! - Workers KV, Access, Dashboard, Turnstile all impacted
//!
//! **Fix**:
//! ```rust
//! // ❌ BAD - no error context
//! result.unwrap()
//!
//! // ✅ GOOD - descriptive error message
//! result.expect("Bot feature file must be valid and within size limits")
//!
//! // ✅ BEST - proper error handling
//! result.map_err(|e| anyhow!("Failed to load bot features: {}", e))?
//! ```
//!
//! **Academic Foundation**:
//! - Post-Mortem Analysis: Cloudflare Blog (2025-11-18)
//! - Rust RFC 1937: Error handling best practices
//! - "Effective Rust" (2024): Prefer expect() with context over unwrap()
//!
//! ## Test Code Exemption
//!
//! `.unwrap()` is allowed in test code (`#[cfg(test)]`, `tests/` directory)
//! as panics are acceptable for test failures.

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use regex::Regex;
use std::path::Path;

/// Known Defects scorer - detects production defect patterns
#[derive(Debug, Clone)]
pub struct KnownDefectsScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl KnownDefectsScorer {
    /// Create a new KnownDefectsScorer
    pub fn new() -> Self {
        Self {
            name: "Known Defects".to_string(),
            max_points: 20.0,
        }
    }

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
        if path_str.contains("/tests/")
            || path_str.contains("/benches/")
            || path_str.contains("/src/tests/")
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
                || filename_lower.starts_with("coverage_")   // e.g., coverage_part4.rs (include!'d tests)
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

    /// Calculate score based on unwrap count
    ///
    /// Scoring:
    /// - 0-99 unwraps: 20 points (perfect)
    /// - 100-199 unwraps: 15 points (-5)
    /// - 200-299 unwraps: 10 points (-10)
    /// - 300-399 unwraps: 5 points (-15)
    /// - 400+ unwraps: 0 points (-20)
    fn calculate_unwrap_score(&self, production_unwraps: usize) -> f64 {
        let penalty = (production_unwraps / 100) as f64 * 5.0;
        let score = self.max_points - penalty;
        score.max(0.0) // Cannot go negative
    }

    /// Internal scoring logic
    fn score_internal(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let (production_unwraps, _test_unwraps) = self.count_unwraps(project_path, cache)?;
        let score = self.calculate_unwrap_score(production_unwraps);

        // Create category score
        let category_score = CategoryScore::new(score, self.max_points);

        Ok(category_score)
    }
}

impl Default for KnownDefectsScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for KnownDefectsScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        self.score(project_path)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Ok((production_unwraps, _test_unwraps)) = self.count_unwraps(project_path, None) {
            if production_unwraps > 0 {
                recommendations.push(format!(
                    "CRITICAL: {} unwrap() calls in production code - replace with .expect() or proper error handling (Cloudflare-class defect)",
                    production_unwraps
                ));
                recommendations.push(
                    "Run: cargo clippy -- -D clippy::disallowed-methods to enforce unwrap() ban"
                        .to_string(),
                );
                recommendations.push(
                    "See Cloudflare outage 2025-11-18: unwrap() panic caused 3+ hour network outage".to_string()
                );
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for KnownDefectsScorer {}
unsafe impl Sync for KnownDefectsScorer {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = KnownDefectsScorer::new();
        assert_eq!(scorer.name(), "Known Defects");
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_perfect_score_no_unwraps() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // Create src directory with clean code
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
            pub fn safe_function() -> Result<i32, String> {
                Ok(42)
            }
            "#,
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "Perfect score with no unwraps");
    }

    #[test]
    fn test_unwrap_penalty_production_code() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");

        // Production code with 150 unwraps (should lose 5 points)
        let mut code = String::new();
        for i in 0..150 {
            code.push_str(&format!("let x{} = Some(42).unwrap();\n", i));
        }

        fs::write(temp_dir.path().join("src/lib.rs"), code).expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 15.0, "150 unwraps = -5 points");
    }

    #[test]
    fn test_test_code_exemption() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // Tests directory with unwraps (should not count)
        fs::create_dir_all(temp_dir.path().join("tests")).expect("create tests");
        fs::write(
            temp_dir.path().join("tests/integration.rs"),
            "fn test() { Some(42).unwrap(); Some(42).unwrap(); }",
        )
        .expect("write test");

        // Production code - clean
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "Test unwraps don't count against score");
    }

    #[test]
    fn test_src_tests_exemption() {
        // RED test for /src/tests/ pattern (currently fails - false positive bug)
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // src/tests/ directory with unwraps (should not count - common pattern in pmat)
        fs::create_dir_all(temp_dir.path().join("src/tests")).expect("create src/tests");
        fs::write(
            temp_dir.path().join("src/tests/unit_tests.rs"),
            "fn test() { Some(42).unwrap(); Some(42).unwrap(); Some(42).unwrap(); }",
        )
        .expect("write test");

        // Production code - clean
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "src/tests/ unwraps should not count");
    }

    #[test]
    fn test_maximum_penalty() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");

        // 500 unwraps (should max out penalty at 0 points)
        let mut code = String::new();
        for i in 0..500 {
            code.push_str(&format!("let x{} = Some(42).unwrap();\n", i));
        }

        fs::write(temp_dir.path().join("src/lib.rs"), code).expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 0.0, "Maximum penalty capped at 0");
    }

    #[test]
    fn test_recommendations_generated() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "let x = Some(42).unwrap();",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        assert!(
            !recommendations.is_empty(),
            "Should generate recommendations"
        );
        assert!(
            recommendations[0].contains("CRITICAL"),
            "Should be marked critical"
        );
        assert!(
            recommendations[0].contains("Cloudflare"),
            "Should reference incident"
        );
    }
}
