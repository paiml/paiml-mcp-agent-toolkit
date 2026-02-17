#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared Known Defects Detection Module
//!
//! Provides defect detection capabilities for:
//! - rust-project-score (KnownDefectsScorer)
//! - TDG analyzer (auto-fail on critical defects)
//! - analyze defects command (project-wide scanning)
//!
//! Based on specification: docs/specifications/known-defects-languages-spec.md

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Defect severity levels (based on production impact)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Auto-fail in TDG, exit code 1 in analyze
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// A detected defect instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectInstance {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code_snippet: String,
}

/// A known defect pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectPattern {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub fix_recommendation: String,
    pub bad_example: String,
    pub good_example: String,
    pub evidence_description: String,
    pub evidence_url: Option<String>,
    pub instances: Vec<DefectInstance>,
}

/// Defect detector for Rust code
pub struct RustDefectDetector {
    unwrap_regex: Regex,
}

impl RustDefectDetector {
    pub fn new() -> Self {
        Self {
            unwrap_regex: Regex::new(r"\.unwrap\(\)").expect("internal error"),
        }
    }

    /// Check if a file should be excluded from defect detection
    fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Exclude test directories
        if path_str.contains("/tests/")
            || path_str.starts_with("tests/")
            || path_str.contains("/benches/")
            || path_str.starts_with("benches/")
        {
            return true;
        }

        // Exclude examples directory (demos and samples often use .expect("internal error") for brevity)
        if path_str.contains("/examples/")
            || path_str.starts_with("examples/")
            || path_str.starts_with("./examples/")
        {
            return true;
        }

        // Exclude fuzz targets (fuzz tests typically use .expect("internal error") for simplicity)
        if path_str.contains("/fuzz/")
            || path_str.starts_with("fuzz/")
            || path_str.starts_with("./fuzz/")
        {
            return true;
        }

        // Exclude test file patterns
        if file_name.ends_with("_tests.rs")
            || file_name.ends_with("_test.rs")
            || file_name.starts_with("test_")
        {
            return true;
        }

        false
    }

    /// Check if content contains test-related markers
    fn has_test_markers(&self, content: &str) -> bool {
        // Check for test cfg attributes
        let has_cfg_test = content.contains("#[cfg(test)]")
            || content.contains("#[cfg(all(test,")
            || content.contains("#[cfg(any(test,");

        // Check for test function attributes
        let has_test_attr = content.contains("#[test]")
            || content.contains("#[tokio::test]")
            || content.contains("#[async_test]");

        has_cfg_test || has_test_attr
    }

    /// Detect all defects in Rust source code
    /// Returns vector of detected defect patterns with instances
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        let mut defects = Vec::new();

        // Exclude test files entirely
        if self.should_exclude_file(file_path) {
            return defects;
        }

        // Exclude files with test markers
        if self.has_test_markers(content) {
            return defects;
        }

        // Detect .unwrap() calls
        let unwrap_instances = self.detect_unwraps(content, file_path);
        if !unwrap_instances.is_empty() {
            defects.push(DefectPattern {
                id: "RUST-UNWRAP-001".to_string(),
                name: ".unwrap() calls".to_string(),
                severity: Severity::Critical,
                fix_recommendation:
                    "Use .expect() with descriptive messages or proper error handling with ?"
                        .to_string(),
                bad_example: "let x = result.unwrap();".to_string(),
                good_example: "let x = result.expect(\"Bot feature file must be valid\");"
                    .to_string(),
                evidence_description: "Cloudflare outage 2025-11-18 (3+ hour network outage)"
                    .to_string(),
                evidence_url: Some("https://blog.cloudflare.com/2025-01-18-outage".to_string()),
                instances: unwrap_instances,
            });
        }

        defects
    }

    fn detect_unwraps(&self, content: &str, file_path: &Path) -> Vec<DefectInstance> {
        let mut instances = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip doc comments - they contain examples, not production code
            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                continue;
            }
            for mat in self.unwrap_regex.find_iter(line) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: mat.start() + 1,
                    code_snippet: line.trim().to_string(),
                });
            }
        }

        instances
    }

    /// Count unwrap() calls (used by rust-project-score)
    pub fn count_unwraps(&self, content: &str) -> usize {
        self.unwrap_regex.find_iter(content).count()
    }
}

impl Default for RustDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Defect detector for Lua code
pub struct LuaDefectDetector {
    global_assign_re: Regex,
    nil_chain_re: Regex,
    unchecked_pcall_re: Regex,
    dangerous_api_re: Regex,
}

impl LuaDefectDetector {
    pub fn new() -> Self {
        Self {
            global_assign_re: Regex::new(r"^([a-zA-Z_]\w*)\s*=").expect("internal error"),
            nil_chain_re: Regex::new(r"\)\s*[.:]\w+").expect("internal error"),
            unchecked_pcall_re: Regex::new(r"^\s*x?pcall\s*\(").expect("internal error"),
            dangerous_api_re: Regex::new(
                r"\b(?:os\.execute|io\.popen|loadstring|setfenv|getfenv|debug\.setlocal)\s*\(",
            )
            .expect("internal error"),
        }
    }

    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        let mut defects = Vec::new();
        if self.should_exclude_file(file_path) {
            return defects;
        }

        self.detect_implicit_globals(content, file_path, &mut defects);
        self.detect_nil_unsafe(content, file_path, &mut defects);
        self.detect_unchecked_pcall(content, file_path, &mut defects);
        self.detect_dangerous_apis(content, file_path, &mut defects);

        defects
    }

    fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.contains("/spec/")
            || file_name.starts_with("test_")
            || file_name.ends_with("_test.lua")
            || file_name.ends_with("_spec.lua")
    }

    fn detect_implicit_globals(
        &self,
        content: &str,
        file_path: &Path,
        defects: &mut Vec<DefectPattern>,
    ) {
        let lua_keywords = [
            "if", "then", "else", "elseif", "end", "do", "while", "repeat", "until", "for", "in",
            "function", "return", "break", "goto", "not", "and", "or",
        ];
        let mut instances = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("local ") {
                continue;
            }
            let Some(caps) = self.global_assign_re.captures(trimmed) else {
                continue;
            };
            let name = caps.get(1).map_or("", |m| m.as_str());
            if lua_keywords.contains(&name) || name.starts_with('_') {
                continue;
            }
            instances.push(DefectInstance {
                file: file_path.to_string_lossy().to_string(),
                line: line_num + 1,
                column: 1,
                code_snippet: trimmed.to_string(),
            });
        }
        if !instances.is_empty() {
            let severity = if instances.len() > 10 {
                Severity::Critical
            } else {
                Severity::High
            };
            defects.push(DefectPattern {
                id: "LUA-GLOBAL-001".to_string(),
                name: "Implicit global assignment".to_string(),
                severity,
                fix_recommendation: "Add `local` keyword to variable declarations".to_string(),
                bad_example: "count = 0".to_string(),
                good_example: "local count = 0".to_string(),
                evidence_description:
                    "Global namespace pollution is Lua's #1 defect source (Maidl et al. 2014)"
                        .to_string(),
                evidence_url: None,
                instances,
            });
        }
    }

    fn detect_nil_unsafe(&self, content: &str, file_path: &Path, defects: &mut Vec<DefectPattern>) {
        let mut instances = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if self.nil_chain_re.is_match(trimmed) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: 1,
                    code_snippet: trimmed.to_string(),
                });
            }
        }
        if !instances.is_empty() {
            defects.push(DefectPattern {
                id: "LUA-NIL-001".to_string(),
                name: "Nil-unsafe chained access".to_string(),
                severity: Severity::High,
                fix_recommendation:
                    "Store function return in local variable and nil-check before accessing"
                        .to_string(),
                bad_example: "get_player():set_health(100)".to_string(),
                good_example: "local p = get_player()\nif p then p:set_health(100) end".to_string(),
                evidence_description:
                    "Chained access on nil return causes runtime crash (LuaTaint analysis)"
                        .to_string(),
                evidence_url: None,
                instances,
            });
        }
    }

    fn detect_unchecked_pcall(
        &self,
        content: &str,
        file_path: &Path,
        defects: &mut Vec<DefectPattern>,
    ) {
        let mut instances = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            // Uncaptured: pcall( without assignment
            if self.unchecked_pcall_re.is_match(trimmed) && !trimmed.contains('=') {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: 1,
                    code_snippet: trimmed.to_string(),
                });
            }
        }
        if !instances.is_empty() {
            defects.push(DefectPattern {
                id: "LUA-PCALL-001".to_string(),
                name: "Unchecked pcall/xpcall return".to_string(),
                severity: Severity::High,
                fix_recommendation: "Capture and check pcall return: local ok, err = pcall(fn); if not ok then ... end".to_string(),
                bad_example: "pcall(dangerous_function)".to_string(),
                good_example: "local ok, err = pcall(dangerous_function)\nif not ok then error(err) end".to_string(),
                evidence_description: "Swallowed errors hide crashes (FLuaScan, Zhang et al. 2020)".to_string(),
                evidence_url: None,
                instances,
            });
        }
    }

    fn detect_dangerous_apis(
        &self,
        content: &str,
        file_path: &Path,
        defects: &mut Vec<DefectPattern>,
    ) {
        let mut instances = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if self.dangerous_api_re.is_match(trimmed) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: 1,
                    code_snippet: trimmed.to_string(),
                });
            }
        }
        if !instances.is_empty() {
            defects.push(DefectPattern {
                id: "LUA-DANGER-001".to_string(),
                name: "Dangerous/deprecated API usage".to_string(),
                severity: Severity::High,
                fix_recommendation: "Avoid os.execute/io.popen with user input; use structured APIs instead of loadstring".to_string(),
                bad_example: "os.execute('rm -rf ' .. user_input)".to_string(),
                good_example: "os.execute('make clean')  -- hardcoded commands only".to_string(),
                evidence_description: "Command injection via string concatenation in shell APIs (LuaTaint)".to_string(),
                evidence_url: None,
                instances,
            });
        }
    }
}

impl Default for LuaDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_unwrap() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn main() {
                let x = Some(42).unwrap();
            }
        "#;

        let path = PathBuf::from("src/main.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].id, "RUST-UNWRAP-001");
        assert_eq!(defects[0].severity, Severity::Critical);
        assert_eq!(defects[0].instances.len(), 1);
    }

    #[test]
    fn test_excludes_doc_comments() {
        let detector = RustDefectDetector::new();
        let code = r#"
            /// # Examples
            ///
            /// ```
            /// let result = something.unwrap();
            /// ```
            pub fn something() -> Option<i32> {
                Some(42)
            }

            //! Module doc with example
            //! let x = foo.unwrap();
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "Doc comments should be excluded (issue #131)"
        );
    }

    #[test]
    fn test_excludes_test_code() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg_attr(coverage_nightly, coverage(off))]
            #[cfg(test)]
            mod tests {
                fn test_foo() {
                    let x = Some(42).unwrap();
                }
            }
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Test code should be excluded");
    }

    #[test]
    fn test_excludes_test_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn test_helper() {
                let x = Some(42).expect("internal error");
            }
        "#;

        let path = PathBuf::from("tests/integration_test.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Tests directory should be excluded");
    }

    #[test]
    fn test_excludes_examples_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn main() {
                let x = Some(42).expect("internal error");
            }
        "#;

        // Test various examples path patterns
        for path in &[
            "examples/demo.rs",
            "./examples/demo.rs",
            "server/examples/demo.rs",
        ] {
            let path = PathBuf::from(path);
            let defects = detector.detect(code, &path);
            assert_eq!(
                defects.len(),
                0,
                "Examples directory should be excluded: {}",
                path.display()
            );
        }
    }

    #[test]
    fn test_excludes_fuzz_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn fuzz_target() {
                let x = Some(42).expect("internal error");
            }
        "#;

        // Test various fuzz path patterns
        for path in &[
            "fuzz/fuzz_targets/target.rs",
            "./fuzz/fuzz_targets/target.rs",
            "server/fuzz/target.rs",
        ] {
            let path = PathBuf::from(path);
            let defects = detector.detect(code, &path);
            assert_eq!(
                defects.len(),
                0,
                "Fuzz directory should be excluded: {}",
                path.display()
            );
        }
    }
}
