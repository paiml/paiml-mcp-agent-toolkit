impl RustDefectDetector {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    /// True if a single trimmed line is a test-function attribute (`#[test]`,
    /// `#[tokio::test]`, …). The item it precedes is test code even when it is
    /// not wrapped in a `#[cfg(test)]` module.
    fn is_test_attr_line(trimmed: &str) -> bool {
        trimmed.starts_with("#[test]")
            || trimmed.starts_with("#[tokio::test")
            || trimmed.starts_with("#[async_test")
            || trimmed.starts_with("#[async_std::test")
    }

    /// Detect all defects in Rust source code
    /// Returns vector of detected defect patterns with instances
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        let mut defects = Vec::new();

        // Exclude test files entirely
        if self.should_exclude_file(file_path) {
            return defects;
        }

        // NOTE: there used to be a whole-file bail here —
        // `if self.has_test_markers(content) { return defects; }` — where
        // has_test_markers matched the bare substrings "#[cfg(test)]" / "#[test]"
        // ANYWHERE in the file. Rust's idiomatic co-located unit-test module
        // therefore blanked out every line of production code above it: a crate
        // whose src/lib.rs was one `.unwrap()`-bearing function reported 1 defect,
        // and reported 0 the moment an unrelated `#[cfg(test)] mod tests` was
        // appended to the same file. Scanning this repo it left 44 defects in 5
        // files against ~19,500 `.unwrap()` calls in ~1,300 files.
        //
        // Exclusion is now per-range, not per-file: detect_unwraps already skips
        // the body of any `#[cfg(...)]` item by brace depth (which covers
        // `#[cfg(test)] mod tests`), and additionally skips `#[test]`-attributed
        // items, so test code is dropped while the production code around it is
        // still reported.

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
                // The slug used to say 2025-01-18 while the description said
                // 2025-11-18: a citation for an outage that never happened, and
                // the URL 404ed. The post about the November 18 outage — the one
                // an `.unwrap()` on a bot-features file caused — is this one.
                evidence_url: Some(
                    "https://blog.cloudflare.com/18-november-2025-outage/".to_string(),
                ),
                instances: unwrap_instances,
            });
        }

        defects
    }

    fn detect_unwraps(&self, content: &str, file_path: &Path) -> Vec<DefectInstance> {
        let mut instances = Vec::new();

        // Honor a file-level (inner) suppression attribute. A developer who has
        // written `#![allow(clippy::unwrap_used)]` (or the broader
        // `#![allow(clippy::restriction)]`, the lint group that contains
        // `unwrap_used`) has explicitly opted these unwraps out of the lint that
        // owns this policy. Auto-failing such a file would be a false positive,
        // so mirror clippy's suppression semantics and skip the whole file.
        if file_allows_unwrap(content) {
            return instances;
        }

        // Track #[cfg(...)] blocks via brace depth so we can skip .unwrap()
        // inside conditional compilation code (issue #279). The same
        // brace-depth machinery is reused to honor item-level (outer)
        // `#[allow(clippy::unwrap_used)]` attributes.
        let mut brace_depth: i32 = 0;
        let mut suppress_entry_depth: Option<i32> = None; // depth when #[cfg]/#[allow] was seen
        let mut pending_suppress = false;
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

            // Detect #[cfg(...)] attributes — marks the next braced item as cfg-gated.
            // Also honor item-level (outer) #[allow(clippy::unwrap_used)] / clippy::restriction.
            // `#[test]`-attributed items are skipped the same way, so a test
            // function that is not inside a `#[cfg(test)]` module is still
            // excluded without discarding the rest of the file.
            if trimmed.starts_with("#[cfg(")
                || trimmed.starts_with("#[cfg_attr(")
                || attr_line_allows_unwrap(trimmed)
                || Self::is_test_attr_line(trimmed)
            {
                pending_suppress = true;
            }

            // Track brace depth and suppressed-block boundaries
            for ch in line.chars() {
                if ch == '{' {
                    if pending_suppress && suppress_entry_depth.is_none() {
                        suppress_entry_depth = Some(brace_depth);
                        pending_suppress = false;
                    }
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if let Some(entry) = suppress_entry_depth {
                        if brace_depth <= entry {
                            suppress_entry_depth = None;
                        }
                    }
                }
            }

            // Skip .unwrap() detection inside #[cfg] blocks (conditional
            // compilation, e.g. GPU init / platform-specific code) or inside
            // an item explicitly annotated with #[allow(clippy::unwrap_used)].
            if suppress_entry_depth.is_some() {
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_unwraps(&self, content: &str) -> usize {
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

/// True if `attrs` (a clippy lint list, e.g. `clippy::unwrap_used, clippy::all`)
/// suppresses the `unwrap_used` lint. Honors both the specific lint and the
/// `clippy::restriction` group that contains it. Note: `clippy::all` does NOT
/// include `unwrap_used` (it lives in the `restriction` group), so it is
/// intentionally not treated as a suppression here.
fn allow_list_suppresses_unwrap(attrs: &str) -> bool {
    attrs.contains("clippy::unwrap_used") || attrs.contains("clippy::restriction")
}

/// True if the file carries a module/crate-level (inner) attribute
/// `#![allow(clippy::unwrap_used)]` / `#![allow(clippy::restriction)]`,
/// which suppresses the unwrap lint for the entire file.
fn file_allows_unwrap(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        // Stop scanning once we reach real code: inner attributes must appear
        // before the first item. Bail out on the first non-attribute,
        // non-comment, non-blank line for a cheap early exit.
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }
        if trimmed.starts_with("#![allow(") {
            if allow_list_suppresses_unwrap(trimmed) {
                return true;
            }
            continue;
        }
        if trimmed.starts_with("#!") || trimmed.starts_with("#[") {
            // Other inner/outer attributes (e.g. #![feature], #![deny]) — keep scanning.
            continue;
        }
        // First line of real code reached without finding the allow — done.
        break;
    }
    false
}

/// True if a single trimmed line is an item-level (outer) attribute
/// `#[allow(clippy::unwrap_used)]` / `#[allow(clippy::restriction)]` that
/// suppresses the unwrap lint for the item it precedes.
fn attr_line_allows_unwrap(trimmed: &str) -> bool {
    trimmed.starts_with("#[allow(") && allow_list_suppresses_unwrap(trimmed)
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

/// Regression tests for the whole-file test-marker bail.
///
/// `detect` used to return an empty vec for any file containing the substring
/// "#[cfg(test)]" or "#[test]", so a co-located unit-test module — the Rust
/// idiom — hid every defect in the production code above it.
#[cfg(test)]
mod colocated_test_module_regression_tests {
    use super::*;

    const PRODUCTION_THEN_TEST_MODULE: &str = "pub fn dangerous(v: &[i32]) -> i32 {\n    \
         *v.first().unwrap()\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        \
         assert_eq!(1, 1);\n    }\n}\n";

    #[test]
    fn production_unwrap_survives_a_colocated_test_module() {
        let detector = RustDefectDetector::new();
        let defects = detector.detect(PRODUCTION_THEN_TEST_MODULE, Path::new("src/lib.rs"));

        assert_eq!(
            defects.len(),
            1,
            "the production .unwrap() was discarded because the file also contains a test module"
        );
        assert_eq!(defects[0].instances.len(), 1);
        assert_eq!(defects[0].instances[0].line, 2);
    }

    #[test]
    fn unwrap_inside_the_colocated_test_module_is_still_excluded() {
        let detector = RustDefectDetector::new();
        let code = "pub fn safe(v: &[i32]) -> i32 {\n    v.len() as i32\n}\n\n#[cfg(test)]\n\
                    mod tests {\n    #[test]\n    fn t() {\n        \
                    let _ = Some(1).unwrap();\n    }\n}\n";
        let defects = detector.detect(code, Path::new("src/lib.rs"));

        assert!(
            defects.is_empty(),
            "test-module .unwrap() must stay excluded: {defects:?}"
        );
    }

    /// The citation used to point at `/2025-01-18-outage` while claiming the
    /// outage of 2025-11-18: two different dates in one piece of evidence, and
    /// the URL 404ed. The date in the slug must match the date in the prose.
    #[test]
    fn unwrap_evidence_url_matches_its_own_description() {
        let detector = RustDefectDetector::new();
        let defects = detector.detect(PRODUCTION_THEN_TEST_MODULE, Path::new("src/lib.rs"));
        let unwrap = defects
            .iter()
            .find(|d| d.id == "RUST-UNWRAP-001")
            .expect("RUST-UNWRAP-001 must be reported");
        let url = unwrap
            .evidence_url
            .as_deref()
            .expect("RUST-UNWRAP-001 cites a URL");

        assert!(
            unwrap.evidence_description.contains("2025-11-18"),
            "description names the November 2025 outage: {}",
            unwrap.evidence_description
        );
        assert!(
            url.contains("18-november-2025"),
            "the cited URL must be the November 2025 outage post, not {url}"
        );
    }

    #[test]
    fn bare_test_function_is_excluded_without_hiding_the_file() {
        let detector = RustDefectDetector::new();
        let code = "pub fn dangerous(v: &[i32]) -> i32 {\n    *v.first().unwrap()\n}\n\n\
                    #[test]\nfn stray() {\n    let _ = Some(2).unwrap();\n}\n";
        let defects = detector.detect(code, Path::new("src/lib.rs"));

        assert_eq!(defects.len(), 1);
        assert_eq!(
            defects[0].instances.len(),
            1,
            "only the production .unwrap() should be reported: {:?}",
            defects[0].instances
        );
        assert_eq!(defects[0].instances[0].line, 2);
    }
}
