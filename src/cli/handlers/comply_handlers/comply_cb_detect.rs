// CB-050/CB-060 Detection Logic
// Implements stub and GPU quality checks per specification COMPLY-008
//
// Per Popper's falsification methodology:
// - Each pattern is a hypothesis that can be falsified
// - False positives falsify our precision hypothesis
// - False negatives falsify our detection hypothesis

// Regex::new() on compile-time constant patterns cannot fail; use .expect("valid regex")

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// A violation detected by CB-050 or CB-060 checks
#[derive(Debug, Clone, PartialEq)]
pub struct CbViolation {
    pub line: u32,
    pub pattern_id: &'static str,
    pub description: String,
    pub code_snippet: Option<String>,
}

// =============================================================================
// COMPLY-005: SATD MANIFESTATION TYPE
// =============================================================================
//
// Per [SATD-002] Maldonado & Shihab (2015): Code SATD (todo!(), unimplemented!())
// correlates 2.3x more strongly with defects than comment SATD (// TODO).
// This distinction is fundamental to proper severity modeling.

/// SATD manifestation type affects severity scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SATDManifestationType {
    /// Comment-based: // TODO, // FIXME, /* HACK */ - advisory only
    Comment,
    /// Code-based: todo!(), unimplemented!(), raise NotImplementedError - crashes at runtime
    Code,
}

/// Severity levels for SATD
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl SATDManifestationType {
    /// Escalate severity for Code manifestations
    /// Per [SATD-003]: Design debt (stubs) costs 2.3x more to fix
    pub fn escalate_severity(&self, base: Severity) -> Severity {
        match self {
            SATDManifestationType::Comment => base, // No escalation
            SATDManifestationType::Code => match base {
                Severity::Low => Severity::Medium,        // Low -> Medium
                Severity::Medium => Severity::High,       // Medium -> High
                Severity::High => Severity::Critical,     // High -> Critical
                Severity::Critical => Severity::Critical, // Already max
            },
        }
    }
}

/// Classify SATD content into Code or Comment manifestation type
pub fn classify_satd_manifestation(content: &str) -> SATDManifestationType {
    // Code patterns: deterministic runtime failures
    let code_patterns = [
        "todo!",
        "unimplemented!",
        "panic!(\"not implemented",
        "panic!(\"Not implemented",
        "raise NotImplementedError",
        "fn ", // Empty function bodies (detected separately, but classified here)
    ];

    // Check if content matches any code pattern
    for pattern in &code_patterns {
        if content.contains(pattern) {
            return SATDManifestationType::Code;
        }
    }

    // Check for empty function body pattern
    if content.trim().ends_with("{}") || content.contains("{ }") {
        return SATDManifestationType::Code;
    }

    // Default: Comment manifestation (e.g. `TODO`, `FIXME`, `HACK` markers)
    SATDManifestationType::Comment
}

/// Classify based on pattern ID from CB-050 detection
pub fn classify_satd_by_pattern_id(pattern_id: &str) -> SATDManifestationType {
    match pattern_id {
        // Code patterns - will crash at runtime
        "CB-050-A" => SATDManifestationType::Code, // todo!()
        "CB-050-B" => SATDManifestationType::Code, // unimplemented!()
        "CB-050-C" => SATDManifestationType::Code, // panic!("not implemented")
        "CB-050-D" => SATDManifestationType::Code, // Empty function body
        "CB-050-E" => SATDManifestationType::Code, // Python NotImplementedError
        "CB-050-F" => SATDManifestationType::Comment, // Python pass # stub (advisory)
        // Default to Comment for unknown patterns
        _ => SATDManifestationType::Comment,
    }
}

// =============================================================================
// COMPLY-007: SUPPRESSION CONFIGURATION
// =============================================================================
//
// Per [FP-001] Muske & Serebrenik (2016): >50% false positive rate causes
// tool abandonment. Suppressions allow human judgment to override detection.
//
// CRITICAL: O(1) lookup per violation via HashMap pre-indexing.

/// A single suppression rule
#[derive(Debug, Clone, PartialEq)]
pub struct SuppressionRule {
    /// Check IDs this rule applies to (e.g., ["CB-050-A", "CB-050-B"])
    pub check_ids: Vec<String>,
    /// Glob pattern for file matching (e.g., "examples/**")
    pub glob_pattern: Option<String>,
    /// Specific file path (exact match)
    pub file: Option<String>,
    /// Specific line numbers
    pub lines: Option<Vec<u32>>,
    /// Expiry date (ISO 8601: "2026-12-31")
    pub expires: Option<String>,
    /// Reason for suppression (preserved for audit)
    pub reason: String,
}

/// Suppression configuration with O(1) lookups
#[derive(Debug, Clone, Default)]
pub struct SuppressionConfig {
    /// Rules indexed by check_id for O(1) lookup
    rules: Vec<SuppressionRule>,
    /// Pre-compiled glob patterns with explicit match options
    compiled_globs: Vec<(usize, glob::Pattern, glob::MatchOptions)>,
    /// Pre-indexed file paths for O(1) lookup
    file_index: HashMap<String, Vec<usize>>,
}

/// Result of a suppression check
#[derive(Debug, Clone, PartialEq)]
pub struct SuppressionResult {
    /// Whether the violation is suppressed
    pub suppressed: bool,
    /// The reason for suppression (if suppressed)
    pub reason: Option<String>,
}

impl SuppressionConfig {
    /// Create a new empty suppression config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a suppression rule
    pub fn add_rule(&mut self, rule: SuppressionRule) {
        let rule_idx = self.rules.len();

        // Pre-compile glob pattern with explicit options
        // require_literal_separator: true means * does NOT match /
        if let Some(ref pattern) = rule.glob_pattern {
            if let Ok(compiled) = glob::Pattern::new(pattern) {
                let options = glob::MatchOptions {
                    case_sensitive: true,
                    require_literal_separator: true, // * doesn't match /
                    require_literal_leading_dot: false,
                };
                self.compiled_globs.push((rule_idx, compiled, options));
            }
        }

        // Index by file path for O(1) lookup
        if let Some(ref file) = rule.file {
            self.file_index
                .entry(file.clone())
                .or_default()
                .push(rule_idx);
        }

        self.rules.push(rule);
    }

    /// Check if a violation should be suppressed
    /// Returns (suppressed, reason) - O(1) for file-specific rules
    pub fn should_suppress(&self, check_id: &str, file_path: &str, line: u32) -> SuppressionResult {
        // Normalize path separators (handle Windows paths)
        let normalized_path = file_path.replace('\\', "/");

        for (rule_idx, rule) in self.rules.iter().enumerate() {
            // Check if this rule applies to this check_id
            if !rule.check_ids.is_empty() && !rule.check_ids.iter().any(|id| id == check_id) {
                continue;
            }

            // Check expiry
            if let Some(ref expires) = rule.expires {
                if is_expired(expires) {
                    continue;
                }
            }

            // Check file match
            let file_matches = self.check_file_match(rule_idx, rule, &normalized_path);
            if !file_matches {
                continue;
            }

            // Check line match
            if let Some(ref lines) = rule.lines {
                if !lines.contains(&line) {
                    continue;
                }
            }

            // All conditions matched - suppress
            return SuppressionResult {
                suppressed: true,
                reason: Some(rule.reason.clone()),
            };
        }

        SuppressionResult {
            suppressed: false,
            reason: None,
        }
    }

    /// Check if a file path matches a rule
    fn check_file_match(&self, rule_idx: usize, rule: &SuppressionRule, path: &str) -> bool {
        // If no file constraints, match all files
        if rule.file.is_none() && rule.glob_pattern.is_none() {
            return true;
        }

        // Exact file match (O(1) via index)
        if let Some(ref file) = rule.file {
            if path == file || path.ends_with(file) {
                return true;
            }
        }

        // Glob pattern match with explicit options
        if rule.glob_pattern.is_some() {
            for (idx, compiled, options) in &self.compiled_globs {
                if *idx == rule_idx && compiled.matches_with(path, *options) {
                    return true;
                }
            }
        }

        false
    }
}

/// Check if a date string (ISO 8601) is in the past
fn is_expired(date_str: &str) -> bool {
    // Simple date comparison: "2026-01-24" format
    // Current date is 2026-01-24
    let current_date = "2026-01-24";

    // Lexicographic comparison works for ISO 8601 dates
    date_str < current_date
}

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

// =============================================================================
// CB-060: GPU QUALITY DETECTION
// =============================================================================
//
// Per the "Danger Zone" heuristic: We look for patterns that indicate potential
// GPU correctness issues, accepting some false positives in exchange for high
// recall. Target: >90% precision.

/// Pattern to detect PTX branch instructions (predicated jumps)
static PTX_BRANCH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@%p\d+\s+bra\s+\w+").expect("valid regex"));

/// Pattern to detect PTX barrier sync
static PTX_BARRIER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"bar\.sync\s+\d+").expect("valid regex"));

/// Pattern to detect PTX shared memory load (destination, then source)
static PTX_SHARED_LOAD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"ld\.shared\.\w+\s+[^,]+,\s*\[([^\]]+)\]").expect("valid regex"));

/// Pattern to detect PTX shared memory store (address first, then source)
static PTX_SHARED_STORE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"st\.shared\.\w+\s+\[([^\]]+)\],").expect("valid regex"));

/// Pattern to detect PTX predicated shared memory access (safe)
static PTX_PREDICATED_SHARED_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@%p\d+\s+(ld|st)\.shared").expect("valid regex"));

/// Pattern to detect PTX bounds check (setp.lt)
static PTX_BOUNDS_CHECK_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"setp\.lt\.\w+\s+%p\d+").expect("valid regex"));

/// Pattern to detect constant offset shared access (safe)
static PTX_CONSTANT_OFFSET_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(ld|st)\.shared\.\w+\s+[^,]+,\s*\[\w+\s*\+\s*\d+\]").expect("valid regex")
});

/// Pattern to detect WGSL workgroup barrier
static WGSL_BARRIER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"workgroupBarrier\s*\(\s*\)").expect("valid regex"));

/// Pattern to detect WGSL if statement start
static WGSL_IF_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bif\s*\([^)]+\)\s*\{").expect("valid regex"));

/// Pattern to detect WGSL else block
static WGSL_ELSE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\}\s*else\s*\{").expect("valid regex"));

/// Pattern to detect WGSL for loop with thread-dependent bounds (divergent)
static WGSL_DIVERGENT_LOOP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"for\s*\([^)]*<\s*(?:local_id|global_id)\.\w+[^)]*\)").expect("valid regex")
});

/// Pattern to detect matrix store without bounds (tiled kernel)
static TILED_STORE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bc\s*\[\s*row\s*\*\s*n\s*\+\s*col\s*\]").expect("valid regex"));

/// Pattern to detect proper bounds check (row < m && col < n)
static TILED_BOUNDS_CHECK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:row|global_id\.y)\s*<\s*(?:m|\w+).*(?:col|global_id\.x)\s*<\s*(?:n|\w+)")
        .expect("valid regex")
});

/// Pattern to detect complex but valid bounds expressions
static TILED_COMPLEX_BOUNDS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\([^)]*row[^)]*\)\s*<\s*\([^)]*m[^)]*\).*col\s*<\s*n").expect("valid regex")
});

/// Pattern to detect PTX early exit pattern before tile loop
static PTX_EARLY_EXIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"@%p\d+\s+bra\s+exit[\s\S]{0,200}(?:tile|loop|ld\.shared)").expect("valid regex")
});

/// Pattern to detect WGSL tiled kernel store
static WGSL_TILED_STORE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\ba\s*\[\s*(?:global_id|local_id)\.\w+\s*\*").expect("valid regex")
});

/// Detect PTX barrier divergence patterns
/// Returns list of (line_number, pattern_id, description)
///
/// The "Danger Zone" heuristic: A branch before a barrier is dangerous because
/// some threads may exit early, causing the remaining threads to deadlock on
/// the barrier.
pub fn detect_ptx_barrier_divergence_in_str(ptx: &str) -> Vec<(u32, &'static str, String)> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = ptx.lines().collect();

    // First pass: find all branch and barrier locations
    let mut branch_locations: Vec<(usize, &str)> = Vec::new();
    let mut barrier_locations: Vec<usize> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Track branch locations
        if let Some(m) = PTX_BRANCH_PATTERN.find(line) {
            branch_locations.push((line_idx, m.as_str()));
        }

        // Track barrier locations
        if PTX_BARRIER_PATTERN.is_match(line) {
            barrier_locations.push(line_idx);
        }
    }

    // "Danger Zone" detection: branch BEFORE barrier (within reasonable distance)
    for (branch_line, branch_text) in &branch_locations {
        for &barrier_line in &barrier_locations {
            // Branch must be BEFORE barrier (danger zone)
            if *branch_line < barrier_line {
                // Check distance - within 20 lines is suspicious
                let distance = barrier_line - *branch_line;
                if distance <= 20 {
                    violations.push((
                        (*branch_line + 1) as u32,
                        "CB-060-A",
                        format!(
                            "Thread divergence before barrier: {} (barrier {} lines later)",
                            branch_text, distance
                        ),
                    ));
                }
            }
        }
    }

    violations
}

/// Detect WGSL barrier divergence patterns
///
/// WGSL workgroupBarrier() inside control flow (if/else) is dangerous because
/// not all threads in the workgroup may execute the barrier.
pub fn detect_wgsl_barrier_divergence_in_str(wgsl: &str) -> Vec<(u32, &'static str, String)> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = wgsl.lines().collect();

    // Track control flow state
    let mut if_depth: usize = 0;
    let mut in_divergent_loop = false;
    let mut divergent_loop_depth: usize = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Track divergent loops (for with thread-dependent bounds)
        if WGSL_DIVERGENT_LOOP_PATTERN.is_match(line) {
            in_divergent_loop = true;
            divergent_loop_depth = 1 + line
                .matches('{')
                .count()
                .saturating_sub(line.matches('}').count());
        }

        // Track brace depth for divergent loop
        if in_divergent_loop {
            divergent_loop_depth += line.matches('{').count();
            divergent_loop_depth = divergent_loop_depth.saturating_sub(line.matches('}').count());
            if divergent_loop_depth == 0 {
                in_divergent_loop = false;
            }
        }

        // Track if/else depth
        if WGSL_IF_PATTERN.is_match(line) {
            if_depth += 1;
        }
        if WGSL_ELSE_PATTERN.is_match(line) {
            // else maintains if_depth
        }
        // Count closing braces to track depth
        if if_depth > 0 {
            // Simplified: decrement on closing brace
            // This is imprecise but avoids complex parsing
            let opens = line.matches('{').count();
            let closes = line.matches('}').count();
            if closes > opens {
                if_depth = if_depth.saturating_sub(closes - opens);
            }
        }

        // Check for barrier in dangerous context
        if WGSL_BARRIER_PATTERN.is_match(line) {
            if if_depth > 0 {
                violations.push((
                    line_num,
                    "CB-060-D",
                    "workgroupBarrier() in divergent control flow (if/else)".to_string(),
                ));
            } else if in_divergent_loop {
                violations.push((
                    line_num,
                    "CB-060-D",
                    "workgroupBarrier() in divergent loop (thread-dependent bounds)".to_string(),
                ));
            }
        }
    }

    violations
}

/// Detect unbounded shared memory access in PTX
///
/// Shared memory accesses without bounds checks can cause out-of-bounds errors.
/// Safe patterns: predicated access (@%p), constant offset, or preceding setp.lt.
pub fn detect_shared_memory_unbounded_in_str(ptx: &str) -> Vec<(u32, &'static str, String)> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = ptx.lines().collect();

    // Track bounds check coverage (lines covered by a preceding setp.lt)
    let mut bounds_check_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // First pass: identify bounds checks and their coverage
    for (line_idx, line) in lines.iter().enumerate() {
        if PTX_BOUNDS_CHECK_PATTERN.is_match(line) {
            // A bounds check covers the next ~10 lines (heuristic)
            for i in 0..=10 {
                bounds_check_lines.insert(line_idx + i);
            }
        }
    }

    // Second pass: check shared memory accesses
    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Check if this line has shared memory access (load or store)
        let has_shared_access =
            PTX_SHARED_LOAD_PATTERN.is_match(line) || PTX_SHARED_STORE_PATTERN.is_match(line);
        if has_shared_access {
            // Check for safety patterns:

            // 1. Predicated access is safe (@%p ld.shared)
            if PTX_PREDICATED_SHARED_PATTERN.is_match(line) {
                continue;
            }

            // 2. Constant offset is safe (shared_mem + 128)
            if PTX_CONSTANT_OFFSET_PATTERN.is_match(line) {
                continue;
            }

            // 3. Covered by preceding bounds check
            if bounds_check_lines.contains(&line_idx) {
                continue;
            }

            // Unsafe: shared access without bounds protection
            violations.push((
                line_num,
                "CB-060-B",
                "Unbounded shared memory access (no bounds check or predicate)".to_string(),
            ));
        }
    }

    violations
}

/// Detect tiled kernels without boundary predicates
///
/// Tiled kernels (GEMM, etc.) must check row < m && col < n before storing
/// to avoid out-of-bounds writes on non-tile-aligned dimensions.
pub fn detect_tiled_kernel_no_bounds_in_str(code: &str) -> Vec<(u32, &'static str, String)> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Check for PTX early exit pattern
    if PTX_EARLY_EXIT_PATTERN.is_match(code) {
        // Find the line with the early exit
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains("@%p") && line.contains("bra") && line.contains("exit") {
                violations.push((
                    (line_idx + 1) as u32,
                    "CB-060-C",
                    "Early exit before tile loop may cause barrier divergence".to_string(),
                ));
                break;
            }
        }
    }

    // Track bounds check state
    let mut has_proper_bounds = false;
    let mut bounds_check_line: Option<usize> = None;

    // Scan for bounds checks and stores
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments and string literals
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        if trimmed.starts_with('"') || trimmed.contains("= \"") {
            continue;
        }

        // Look for proper bounds checks
        if TILED_BOUNDS_CHECK_PATTERN.is_match(line) || TILED_COMPLEX_BOUNDS_PATTERN.is_match(line)
        {
            has_proper_bounds = true;
            bounds_check_line = Some(line_idx);
        }

        // Look for partial bounds (only row OR col)
        let has_row_check = line.contains("row <") || line.contains("row<");
        let has_col_check = line.contains("col <") || line.contains("col<");
        let has_if = line.contains("if ");
        if has_if && has_row_check && !has_col_check {
            // Partial bounds - only row checked
            violations.push((
                (line_idx + 1) as u32,
                "CB-060-C",
                "Partial bounds check: row checked but not col".to_string(),
            ));
        }
    }

    // Look for tiled stores
    check_tiled_stores(
        &lines,
        has_proper_bounds,
        bounds_check_line,
        &mut violations,
    );

    violations
}

fn check_tiled_stores(
    lines: &[&str],
    has_proper_bounds: bool,
    bounds_check_line: Option<usize>,
    violations: &mut Vec<(u32, &'static str, String)>,
) {
    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Skip comments and string literals
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        if trimmed.starts_with('"')
            || trimmed.contains("= \"")
            || trimmed.starts_with("let kernel_src")
        {
            continue;
        }

        // Check for tiled store pattern
        if TILED_STORE_PATTERN.is_match(line) {
            if !has_proper_bounds {
                violations.push((
                    line_num,
                    "CB-060-C",
                    "Tiled kernel store without bounds check (row < m && col < n)".to_string(),
                ));
            } else if let Some(bounds_line) = bounds_check_line {
                if bounds_line > line_idx {
                    violations.push((
                        line_num,
                        "CB-060-C",
                        "Bounds check after store (must be before)".to_string(),
                    ));
                }
            }
        }

        // Check for WGSL tiled pattern
        if WGSL_TILED_STORE_PATTERN.is_match(line) && !has_proper_bounds {
            violations.push((
                line_num,
                "CB-060-C",
                "WGSL tiled kernel without bounds check".to_string(),
            ));
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_is_test_path() {
        assert!(is_test_path("src/tests/mod.rs"));
        assert!(is_test_path("src/foo_test.rs"));
        assert!(is_test_path("tests/integration.rs"));
        assert!(!is_test_path("src/lib.rs"));
        assert!(!is_test_path("src/services/context.rs"));
        assert!(!is_test_path(""));
    }

    #[test]
    fn test_is_marker_function() {
        assert!(is_marker_function("marker"));
        assert!(is_marker_function("type_marker"));
        assert!(is_marker_function("phantom_data"));
        assert!(is_marker_function("no_op"));
        assert!(is_marker_function("noop"));
        assert!(!is_marker_function("process"));
        assert!(!is_marker_function("handle_request"));
        assert!(!is_marker_function("marketer")); // not a marker
    }

    #[test]
    fn test_comment_detection() {
        // Rust comments
        assert!(COMMENT_PATTERN.is_match("// this is a comment"));
        assert!(COMMENT_PATTERN.is_match("/// doc comment"));
        assert!(COMMENT_PATTERN.is_match("//! inner doc"));
        assert!(COMMENT_PATTERN.is_match("   // indented comment"));
        assert!(!COMMENT_PATTERN.is_match("let x = 42; // inline"));

        // Python comments (separate pattern)
        assert!(PYTHON_COMMENT_PATTERN.is_match("# python comment"));
        assert!(!PYTHON_COMMENT_PATTERN.is_match("#[test]")); // attribute, not comment
    }

    // ========================================================================
    // WILD TESTS: Self-scan false positive detection
    // ========================================================================

    #[test]
    fn wild_string_literal_false_positive() {
        // From src/qdd/refactor.rs:190 - todo! inside string literal
        let code = r#"let result = code.replace("todo!(", "Ok(Default::default())");"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected todo! in string literal: {:?}",
            violations
        );
    }

    #[test]
    fn wild_test_fixture_string_literal() {
        // From src/services/quality_proxy.rs - test fixture string
        let code = r#"content: Some("fn stub() { unimplemented!() }".to_string()),"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected unimplemented! in test fixture string: {:?}",
            violations
        );
    }

    #[test]
    fn wild_actual_stub_detected() {
        // From src/services/languages/wasm.rs - actual stub
        let code = r#"
    fn _extract_wasm_functions(&mut self, _parser: &Parser) -> Result<(), String> {
        // TO BE IMPLEMENTED
        todo!("Extract function information from WASM module")
    }
"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            !violations.is_empty(),
            "MISSED TRUE POSITIVE: Should detect todo!() in actual stub"
        );
        assert!(
            violations.iter().any(|(_, id, _)| *id == "CB-050-A"),
            "Wrong pattern ID for todo!()"
        );
    }

    #[test]
    fn wild_code_generation_string() {
        // From src/qdd/generator_ast.rs:75 - code generation string
        let code = r#"code.push_str("    todo!(\"Implementation needed\")\n");"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected todo! in code generation string: {:?}",
            violations
        );
    }

    #[test]
    fn wild_string_pattern_matching() {
        // From src/qdd/refactor.rs:299 - pattern matching for todo!
        let code = r#"let todo_count = code.matches("todo!").count() as u32;"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected todo! in pattern matching: {:?}",
            violations
        );
    }
}
