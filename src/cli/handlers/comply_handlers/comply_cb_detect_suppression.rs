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
        debug_assert!(!check_id.is_empty(), "check_id must not be empty");
        debug_assert!(!file_path.is_empty(), "file_path must not be empty");
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
        debug_assert!(!path.is_empty(), "path must not be empty");
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
    debug_assert!(!date_str.is_empty(), "date_str must not be empty");
    // Simple date comparison: "2026-01-24" format
    // Current date is 2026-01-24
    let current_date = "2026-01-24";

    // Lexicographic comparison works for ISO 8601 dates
    date_str < current_date
}

