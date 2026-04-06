// =============================================================================
// CB-050: STUB DETECTION PATTERNS
// =============================================================================

/// Compiled regex patterns for CB-050 stub detection
/// Using LazyLock for thread-safe one-time initialization
static CB050_PATTERNS: LazyLock<Vec<(Regex, &'static str, &'static str)>> = LazyLock::new(|| {
    vec![
        // CB-050-A: `todo!()` macro - handles spacing variations
        (
            Regex::new(r"todo\s*!\s*\(").expect("valid regex"),
            "CB-050-A",
            "todo!() macro - will panic at runtime",
        ),
        // CB-050-B: unimplemented!() macro
        (
            Regex::new(r"unimplemented\s*!\s*\(").expect("valid regex"),
            "CB-050-B",
            "unimplemented!() macro - will panic at runtime",
        ),
        // CB-050-C: panic! with "not implemented" message
        (
            Regex::new(r#"panic\s*!\s*\(\s*"[^"]*not\s+implemented[^"]*""#).expect("valid regex"),
            "CB-050-C",
            "panic!() with 'not implemented' message",
        ),
        // CB-050-E: Python NotImplementedError
        (
            Regex::new(r"raise\s+NotImplementedError").expect("valid regex"),
            "CB-050-E",
            "Python NotImplementedError - will raise at runtime",
        ),
        // CB-050-F: Python pass with stub/placeholder comment
        (
            Regex::new(r"pass\s*#\s*(?i:stub|todo|fixme)").expect("valid regex"),
            "CB-050-F",
            "Python pass with stub comment",
        ),
    ]
});

/// Pattern for detecting empty function bodies (CB-050-D)
/// This requires special handling to avoid trait defaults and test functions
static EMPTY_BODY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match: fn name(...) { } or fn name(...) -> Type { }
    // Allows for whitespace and newlines inside braces
    Regex::new(r"fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*[^{]+)?\s*\{\s*\}").expect("valid regex")
});

/// Pattern to detect if we're inside a trait block
static TRAIT_BLOCK_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"trait\s+\w+[^{]*\{").expect("valid regex"));

/// Pattern to detect if line is inside a string literal
#[allow(dead_code)] // Reserved for future string literal detection
static STRING_LITERAL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*(?:let\s+\w+\s*=\s*)?"[^"]*$|^\s*r#*""#).expect("valid regex")
});

/// Pattern to detect comment lines
/// Note: Cannot use negative lookahead (?!\[) - regex crate doesn't support it
/// Instead we check for # not followed by [ in the detection logic
static COMMENT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?://|/\*|\*|///|//!)").expect("valid regex"));

/// Pattern to detect Python comments (# not followed by [)
static PYTHON_COMMENT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*#[^\[]").expect("valid regex"));

/// Pattern to detect doc test blocks (``` in doc comments)
static DOC_TEST_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*///.*```").expect("valid regex"));

// =============================================================================
// CB-050 DETECTION FUNCTIONS
// =============================================================================

/// Detect CB-050 code stubs in a string of source code
/// Returns list of (line_number, pattern_id, description)
///
/// # Arguments
/// * `code` - The source code to analyze
///
/// # Returns
/// Vector of violations found
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn detect_cb050_code_stubs_in_str(code: &str) -> Vec<(u32, &'static str, String)> {
    detect_cb050_code_stubs_in_str_with_path(code, "")
}

/// Detect CB-050 code stubs with file path context for filtering
/// Returns list of (line_number, pattern_id, description)
///
/// # Arguments
/// * `code` - The source code to analyze
/// * `path` - The file path (used to identify test files)
///
/// # Returns
/// Vector of violations found
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn detect_cb050_code_stubs_in_str_with_path(
    code: &str,
    path: &str,
) -> Vec<(u32, &'static str, String)> {
    let mut violations = Vec::new();

    // Check if this is a test file - stubs in tests are acceptable
    let is_test_file = is_test_path(path);
    if is_test_file {
        return violations;
    }

    // Pre-compute which lines are in doc test blocks, string literals, or comments
    let lines: Vec<&str> = code.lines().collect();
    let skip_mask = compute_skip_mask(&lines);
    let trait_lines = compute_trait_block_lines(&lines);

    // Check each line for violations
    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;

        // Skip lines that are comments, strings, or doc tests
        if skip_mask[line_idx] {
            continue;
        }

        // Check for macro-based stubs (CB-050-A, B, C, E, F)
        for (pattern, id, desc) in CB050_PATTERNS.iter() {
            if pattern.is_match(line) {
                // Additional check: make sure it's not in a string literal on this line
                if !is_in_string_literal(line, pattern) {
                    violations.push((line_num, *id, desc.to_string()));
                }
            }
        }
    }

    // Check for empty function bodies (CB-050-D)
    // This needs multi-line matching since the body might span lines
    for cap in EMPTY_BODY_PATTERN.captures_iter(code) {
        let match_start = cap.get(0).expect("capture group 0 always exists").start();
        let fn_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

        // Find the line number for this match
        let line_num = code
            .get(..match_start)
            .unwrap_or_default()
            .matches('\n')
            .count() as u32
            + 1;

        // Skip if in a trait block (trait default methods are intentionally empty)
        if trait_lines.contains(&(line_num as usize)) {
            continue;
        }

        // Skip marker/sentinel functions (const fn marker() {} is often intentional)
        if is_marker_function(fn_name) {
            continue;
        }

        violations.push((
            line_num,
            "CB-050-D",
            format!("Empty function body: {}()", fn_name),
        ));
    }

    violations
}

/// Check if a path indicates a test file
fn is_test_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    // Check path components
    path.contains("/tests/")
        || path.contains("/test/")
        || path.starts_with("tests/")
        || path.starts_with("test/")
        || path.contains("_test.rs")
        || path.contains("_tests.rs")
        || path.ends_with("/tests.rs")
        || path.ends_with("/test.rs")
        || path.contains("src/tests/")
}

/// Compute a mask of lines to skip (comments, strings, doc tests)
fn compute_skip_mask(lines: &[&str]) -> Vec<bool> {
    let mut skip = vec![false; lines.len()];
    let mut in_doc_test = false;
    let mut in_multiline_string = false;

    for (i, line) in lines.iter().enumerate() {
        // Track doc test blocks
        if DOC_TEST_PATTERN.is_match(line) {
            in_doc_test = !in_doc_test;
            skip[i] = true;
            continue;
        }

        if in_doc_test {
            skip[i] = true;
            continue;
        }

        // Skip comment lines (Rust style: //, ///, //!, /*, *)
        if COMMENT_PATTERN.is_match(line) {
            skip[i] = true;
            continue;
        }

        // Skip Python comment lines (# but not #[attribute])
        if PYTHON_COMMENT_PATTERN.is_match(line) {
            skip[i] = true;
            continue;
        }

        // Track multiline strings (raw strings with r#" ... "#)
        // Use string matching instead of raw string literals to avoid syntax issues
        let raw_start_marker = "r#\"";
        let raw_end_marker = "\"#";
        let raw_string_starts = line.matches(raw_start_marker).count();
        let raw_string_ends = line.matches(raw_end_marker).count();
        if raw_string_starts > raw_string_ends {
            in_multiline_string = true;
        } else if raw_string_ends > raw_string_starts {
            in_multiline_string = false;
        }

        if in_multiline_string {
            skip[i] = true;
        }
    }

    skip
}

/// Compute which lines are inside trait blocks
fn compute_trait_block_lines(lines: &[&str]) -> std::collections::HashSet<usize> {
    let mut trait_lines = std::collections::HashSet::new();
    let mut brace_depth = 0;
    let mut in_trait = false;

    for (i, line) in lines.iter().enumerate() {
        // Check for trait definition start
        if TRAIT_BLOCK_PATTERN.is_match(line) {
            in_trait = true;
        }

        if in_trait {
            trait_lines.insert(i + 1); // 1-indexed line numbers

            // Track brace depth
            brace_depth += line.matches('{').count();
            brace_depth = brace_depth.saturating_sub(line.matches('}').count());

            if brace_depth == 0 && line.contains('}') {
                in_trait = false;
            }
        }
    }

    trait_lines
}

/// Check if the pattern match is inside a string literal on this line
fn is_in_string_literal(line: &str, pattern: &Regex) -> bool {
    // Find where the pattern matches
    if let Some(m) = pattern.find(line) {
        let before = line.get(..m.start()).unwrap_or_default();
        // Count unescaped quotes before the match
        let quote_count = before
            .chars()
            .filter(|&c| c == '"')
            .count()
            .saturating_sub(before.matches(r#"\""#).count());
        // If odd number of quotes before, we're inside a string
        quote_count % 2 == 1
    } else {
        false
    }
}

/// Check if a function name suggests it's an intentional marker/sentinel
fn is_marker_function(name: &str) -> bool {
    let lower_name = name.to_lowercase();

    // Exact matches
    let exact_markers = [
        "marker", "sentinel", "phantom", "noop", "no_op", "dummy", "_",
    ];
    if exact_markers.iter().any(|&m| lower_name == m) {
        return true;
    }

    // Suffix/prefix patterns (e.g., "type_marker", "phantom_data")
    let pattern_markers = ["_marker", "_sentinel", "_phantom", "_noop", "_dummy"];
    if pattern_markers.iter().any(|m| lower_name.ends_with(m)) {
        return true;
    }

    let prefix_markers = ["marker_", "sentinel_", "phantom_", "noop_", "dummy_"];
    if prefix_markers.iter().any(|m| lower_name.starts_with(m)) {
        return true;
    }

    false
}
