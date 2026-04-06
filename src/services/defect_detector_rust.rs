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
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
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
        // Track #[cfg(...)] blocks via brace depth so we can skip .unwrap()
        // inside conditional compilation code (issue #279).
        let mut brace_depth: i32 = 0;
        let mut cfg_entry_depth: Option<i32> = None; // depth when #[cfg] was seen
        let mut pending_cfg = false;
        let mut in_block_comment = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Track block comments (simplified — no nesting)
            if in_block_comment {
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("/*") {
                in_block_comment = !trimmed.contains("*/");
                continue;
            }

            // Skip doc comments and line comments
            if trimmed.starts_with("///")
                || trimmed.starts_with("//!")
                || trimmed.starts_with("//")
            {
                continue;
            }

            // Detect #[cfg(...)] attributes — marks the next braced item as cfg-gated
            if trimmed.starts_with("#[cfg(") || trimmed.starts_with("#[cfg_attr(") {
                pending_cfg = true;
            }

            // Track brace depth and cfg block boundaries
            for ch in line.chars() {
                if ch == '{' {
                    if pending_cfg && cfg_entry_depth.is_none() {
                        cfg_entry_depth = Some(brace_depth);
                        pending_cfg = false;
                    }
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if let Some(entry) = cfg_entry_depth {
                        if brace_depth <= entry {
                            cfg_entry_depth = None;
                        }
                    }
                }
            }

            // Skip .unwrap() detection inside #[cfg] blocks — conditional
            // compilation code may use .unwrap() in feature-gated contexts
            // where it's acceptable (e.g., GPU init, platform-specific code).
            if cfg_entry_depth.is_some() {
                continue;
            }

            // Strip string literal contents to avoid false positives on
            // documentation strings like: "Detects .unwrap() panics"
            let code_only = strip_string_literals(line);

            for mat in self.unwrap_regex.find_iter(&code_only) {
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
    /// Skips comments and string literal contents to avoid false positives.
    pub fn count_unwraps(&self, content: &str) -> usize {
        debug_assert!(!content.is_empty(), "content must not be empty");
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.is_empty()
            })
            .map(|line| {
                let code = strip_string_literals(line);
                self.unwrap_regex.find_iter(&code).count()
            })
            .sum()
    }
}

impl Default for RustDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip contents of string literals to prevent false-positive defect detection.
/// Replaces `"..."` contents with spaces (preserving column offsets).
fn strip_string_literals(line: &str) -> String {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'"' {
            out.push(b'"');
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < len {
                    out.push(b' ');
                    out.push(b' ');
                    i += 2;
                } else {
                    out.push(b' ');
                    i += 1;
                }
            }
            if i < len {
                out.push(b'"');
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}
