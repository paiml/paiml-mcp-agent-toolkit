//! Generic Description Detector
//!
//! TICKET: PMAT-7001 Phase 2 (GREEN)
//!
//! This module detects generic, placeholder, or low-quality documentation
//! text that doesn't provide useful information to users.
//!
//! ## Detection Strategy
//!
//! A description is considered "generic" if it:
//! 1. Is too short (<15 characters)
//! 2. Matches common generic patterns
//! 3. Has too few words (<3 words)
//! 4. Just repeats parameter names
//! 5. Lacks specific details
//!
//! ## Examples
//!
//! **Generic (Forbidden)**:
//! - "The name parameter"
//! - "Project name"
//! - "Input value"
//! - "Path to file"
//! - "Template"
//!
//! **Good (Descriptive)**:
//! - "Agent project name (lowercase, alphanumeric, hyphens only)"
//! - "Quality level: standard (fast), high (thorough), extreme (comprehensive)"
//! - "Path to ROADMAP.md file for validation (default: ./ROADMAP.md)"

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Generic patterns that indicate placeholder text
    static ref GENERIC_PATTERNS: Vec<Regex> = vec![
        // "The X parameter"
        Regex::new(r"^The .+ parameter").unwrap(),

        // "X parameter" (just noun + parameter)
        Regex::new(r"^\w+ parameter$").unwrap(),

        // "X value" patterns
        Regex::new(r"^\w+ value$").unwrap(),
        Regex::new(r"^Input (for|value)").unwrap(),
        Regex::new(r"^Output (for|value)").unwrap(),

        // Single word descriptions
        Regex::new(r"^[A-Z][a-z]+$").unwrap(),

        // "Path to X" without details
        Regex::new(r"^Path to \w+$").unwrap(),

        // "Name for X" without details
        Regex::new(r"^Name for \w+$").unwrap(),

        // "X for Y" without context (less than 3 words after)
        Regex::new(r"^\w+ for \w+$").unwrap(),
    ];

    /// Words that often indicate lazy documentation
    static ref LAZY_WORDS: Vec<&'static str> = vec![
        "parameter",
        "value",
        "input",
        "output",
        "option",
        "setting",
    ];
}

/// Check if a description is generic/placeholder
///
/// Returns `true` if the description is generic and should be rejected.
/// Returns `false` if the description is specific and informative.
///
/// ## Algorithm
///
/// 1. **Length check**: <15 chars = generic
/// 2. **Pattern check**: Matches any generic pattern = generic
/// 3. **Word count**: <3 words = generic
/// 4. **Lazy word ratio**: >50% lazy words = generic
/// 5. **Detail indicators**: Has examples, constraints, or defaults = specific
///
/// ## Examples
///
/// ```rust
/// use pmat::docs_enforcement::generic_detector::is_generic_description;
///
/// // Generic
/// assert!(is_generic_description("The name parameter"));
/// assert!(is_generic_description("Project name"));
/// assert!(is_generic_description("Name"));
///
/// // Specific
/// assert!(!is_generic_description("Agent name (lowercase, alphanumeric)"));
/// assert!(!is_generic_description("Quality level: standard, high, extreme"));
/// ```
pub fn is_generic_description(desc: &str) -> bool {
    // Empty is generic
    if desc.is_empty() {
        return true;
    }

    let desc = desc.trim();

    // Basic checks: length and pattern matching
    if is_too_short_or_matches_pattern(desc) {
        return true;
    }

    let words: Vec<&str> = desc.split_whitespace().collect();
    if words.len() < 3 {
        return true;
    }

    // Check for lazy words and repetitive patterns
    if has_too_many_lazy_words(&words) || has_low_word_uniqueness(&words) {
        return true;
    }

    // If has detail indicators and decent length, it's specific
    if has_detail_indicators(desc) && desc.len() > 30 {
        return false;
    }

    // Default: generic unless proven specific
    false
}

/// Check if description is too short or matches a generic pattern
fn is_too_short_or_matches_pattern(desc: &str) -> bool {
    if desc.len() < 15 {
        return true;
    }

    for pattern in GENERIC_PATTERNS.iter() {
        if pattern.is_match(desc) {
            return true;
        }
    }

    false
}

/// Check if more than 50% of words are lazy/generic words
fn has_too_many_lazy_words(words: &[&str]) -> bool {
    let lazy_count = words.iter()
        .filter(|w| LAZY_WORDS.contains(&w.to_lowercase().as_str()))
        .count();

    (lazy_count as f64) / (words.len() as f64) > 0.5
}

/// Check if description has detail indicators like examples, defaults, constraints
fn has_detail_indicators(desc: &str) -> bool {
    desc.contains("(")      // Examples: "(default: ...)", "(e.g., ...)"
        || desc.contains("[")   // Constraints: "[required]"
        || desc.contains(":")   // Enumerations: "level: standard, high"
        || desc.contains("e.g.")
        || desc.contains("default")
        || desc.contains("example")
}

/// Check if description has low word uniqueness (repetitive)
fn has_low_word_uniqueness(words: &[&str]) -> bool {
    let lowercase_words: Vec<String> = words.iter()
        .map(|w| w.to_lowercase())
        .collect();
    let unique_words: std::collections::HashSet<&String> = lowercase_words.iter().collect();

    // If <40% unique words, probably generic
    (unique_words.len() as f64) / (words.len() as f64) < 0.4
}

/// Suggest improvements for generic descriptions
///
/// Returns suggested improvements if a description is generic.
pub fn suggest_improvements(desc: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    if desc.len() < 15 {
        suggestions.push("Make description at least 15 characters with specific details".to_string());
    }

    if desc.split_whitespace().count() < 3 {
        suggestions.push("Add more context - aim for at least 3 words with details".to_string());
    }

    if !desc.contains("(") && !desc.contains("[") && !desc.contains(":") {
        suggestions.push("Add examples, defaults, or constraints: '(default: ...)', '[required]', 'level: A, B, C'".to_string());
    }

    // Check for common patterns we can suggest fixes for
    if desc.starts_with("The ") && desc.ends_with(" parameter") {
        suggestions.push(format!(
            "Instead of '{}', explain what it does and any constraints",
            desc
        ));
    }

    if desc.split_whitespace().count() < 5 {
        suggestions.push("Explain: What is it? What does it do? What are valid values?".to_string());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_is_generic() {
        assert!(is_generic_description(""));
    }

    #[test]
    fn test_short_is_generic() {
        assert!(is_generic_description("Name"));
        assert!(is_generic_description("Template"));
        assert!(is_generic_description("Short"));
    }

    #[test]
    fn test_the_parameter_pattern() {
        assert!(is_generic_description("The name parameter"));
        assert!(is_generic_description("The template parameter"));
        assert!(is_generic_description("The output parameter"));
    }

    #[test]
    fn test_noun_parameter_pattern() {
        assert!(is_generic_description("Name parameter"));
        assert!(is_generic_description("Template parameter"));
    }

    #[test]
    fn test_value_patterns() {
        assert!(is_generic_description("Name value"));
        assert!(is_generic_description("Input value"));
        assert!(is_generic_description("Output value"));
        assert!(is_generic_description("Input for testing"));
        assert!(is_generic_description("Output for results"));
    }

    #[test]
    fn test_path_patterns() {
        assert!(is_generic_description("Path to file"));
        assert!(is_generic_description("Path to directory"));
        assert!(is_generic_description("Name for project"));
    }

    #[test]
    fn test_specific_descriptions_not_generic() {
        // Has parentheses with details
        assert!(!is_generic_description(
            "Agent project name (lowercase, alphanumeric, hyphens only)"
        ));

        // Has colon enumeration
        assert!(!is_generic_description(
            "Quality level: standard (fast), high (thorough), extreme (comprehensive)"
        ));

        // Has default value
        assert!(!is_generic_description(
            "Path to ROADMAP.md file for validation (default: ./ROADMAP.md)"
        ));

        // Has example
        assert!(!is_generic_description(
            "Output directory where the agent project will be created (default: current directory)"
        ));

        // Has detailed explanation
        assert!(!is_generic_description(
            "Dry-run mode: preview changes without creating files"
        ));
    }

    #[test]
    fn test_domain_specific_terms_allowed() {
        // Domain terms with sufficient context are not generic
        assert!(!is_generic_description(
            "ROADMAP.md file path (default: ./ROADMAP.md in project root)"
        ));

        assert!(!is_generic_description(
            "Cyclomatic complexity threshold (default: 8)"
        ));

        assert!(!is_generic_description(
            "SATD annotation pattern (e.g., TODO, FIXME, HACK)"
        ));
    }

    #[test]
    fn test_suggest_improvements() {
        let suggestions = suggest_improvements("Name");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("15 characters")));
    }

    #[test]
    fn test_suggest_improvements_for_the_parameter() {
        let suggestions = suggest_improvements("The name parameter");
        assert!(suggestions.iter().any(|s| s.contains("Instead of")));
    }
}
