// Claim Extractor: Extract testable claims from commit messages
//
// Specification: Section 3.2 - Claim Categories
// Implements extraction of 8 categories of hallucination-prone claims

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaimCategory {
    TestStatus,        // "all tests passing"
    Documentation,     // "fixed all broken links"
    Coverage,          // "coverage stable at 85%"
    FeatureCompletion, // "complete implementation"
    Migration,         // "migration complete"
    BugFix,            // "fixed bug X"
    Performance,       // "50% faster"
    Security,          // "zero vulnerabilities"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub category: ClaimCategory,
    pub text: String,
    pub is_absolute: bool,          // Contains "all", "zero", "complete"
    pub numeric_value: Option<f64>, // Percentage, count, etc.
    pub issue_number: Option<u32>,  // For bug fix claims
    pub has_scope_qualifier: bool,  // Has "MVP", "Phase N", "Sprint X"
    pub scope: Option<String>,      // The actual scope qualifier
}

pub struct ClaimExtractor {
    // Patterns for each claim category
    test_patterns: Vec<Regex>,
    documentation_patterns: Vec<Regex>,
    coverage_patterns: Vec<Regex>,
    completion_patterns: Vec<Regex>,
    migration_patterns: Vec<Regex>,
    bugfix_patterns: Vec<Regex>,
    performance_patterns: Vec<Regex>,
    security_patterns: Vec<Regex>,

    // Absolute claim keywords
    absolute_keywords: Vec<String>,

    // Scope qualifiers
    scope_patterns: Vec<Regex>,
}

impl ClaimExtractor {
    pub fn new() -> Self {
        Self {
            // Test status patterns
            test_patterns: vec![
                Regex::new(r"(?i)(all|every|\d+/\d+)\s+tests?\s+(passing|pass|work|succeed)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)(most|some)?\s*tests?\s+(all\s+)?passing(\s+\((\d+)/\d+\))?")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)complete\s+test\s+coverage").expect("Hardcoded regex pattern must be valid"),
            ],

            // Documentation patterns
            documentation_patterns: vec![
                Regex::new(
                    r"(?i)fix(ed)?\s+(all\s+)?broken\s+(documentation\s+links?|links?|docs?)",
                )
                .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)documentation\s+(complete|ready|fixed)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)all\s+examples?\s+work").expect("Hardcoded regex pattern must be valid"),
            ],

            // Coverage patterns
            coverage_patterns: vec![
                Regex::new(r"(?i)coverage\s+(stable|at|achieved?)\s+(?:at\s+)?(\d+)%").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)(\d+)%\s+coverage").expect("Hardcoded regex pattern must be valid"),
            ],

            // Feature completion patterns
            completion_patterns: vec![
                Regex::new(r"(?i)complete\s+(\w+(\s+\w+)*)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)(\w+(\s+\w+)*)\s+(ready|complete|done)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)fully\s+functional").expect("Hardcoded regex pattern must be valid"),
            ],

            // Migration patterns
            migration_patterns: vec![
                Regex::new(r"(?i)(complete\s+)?migration\s+to\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)fully\s+migrated\s+to\s+(\w+)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)deprecated\s+(\w+)\s+removed").expect("Hardcoded regex pattern must be valid"),
            ],

            // Bug fix patterns
            bugfix_patterns: vec![
                Regex::new(r"(?i)fix(es|ed)?\s+(bug|issue)\s+#?(\d+)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)resolve[sd]?\s+(issue\s+)?#?(\d+)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)bug\s+fixed").expect("Hardcoded regex pattern must be valid"),
            ],

            // Performance patterns
            performance_patterns: vec![
                Regex::new(r"(?i)(\d+)%\s+(faster|slower|improvement)(\s+\w+)*").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)performance\s+(optimized|improved)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)reduced\s+memory\s+by\s+(\d+)%").expect("Hardcoded regex pattern must be valid"),
            ],

            // Security patterns
            security_patterns: vec![
                Regex::new(r"(?i)zero\s+vulnerabilities").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)all\s+deps?\s+updated").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)security\s+audit\s+passed").expect("Hardcoded regex pattern must be valid"),
            ],

            // Absolute claim keywords
            absolute_keywords: vec![
                "all".to_string(),
                "every".to_string(),
                "zero".to_string(),
                "complete".to_string(),
                "fully".to_string(),
                "entirely".to_string(),
            ],

            // Scope qualifier patterns
            scope_patterns: vec![
                Regex::new(r"(?i)(MVP|Alpha|Beta|Phase\s+\d+|Sprint\s+\d+)").expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"\(([^)]*(?:MVP|Phase|Sprint|Alpha|Beta)[^)]*)\)").expect("Hardcoded regex pattern must be valid"),
            ],
        }
    }

    pub fn extract(&self, commit_message: &str) -> Vec<Claim> {
        let mut claims_with_pos: Vec<(usize, Claim)> = Vec::new();

        // Extract test status claims
        for pattern in &self.test_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                // Try to extract numeric value from capture group 4 (fraction numerator), then fallback to regex
                let numeric_value = captures
                    .get(4)
                    .and_then(|m| m.as_str().parse::<f64>().ok())
                    .or_else(|| self.extract_numeric_value(&text));

                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::TestStatus,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value,
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break; // Only one claim per category
            }
        }

        // Extract documentation claims
        for pattern in &self.documentation_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::Documentation,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value: self.extract_numeric_value(&text),
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Extract coverage claims
        for pattern in &self.coverage_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                let numeric_value = captures
                    .get(1)
                    .and_then(|m| m.as_str().parse::<f64>().ok())
                    .or_else(|| captures.get(2).and_then(|m| m.as_str().parse::<f64>().ok()));

                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::Coverage,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value,
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Extract migration claims (check before feature completion to avoid conflicts)
        for pattern in &self.migration_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::Migration,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value: None,
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Extract feature completion claims
        for pattern in &self.completion_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();

                // Skip if we already have a claim overlapping this position (e.g., migration)
                if claims_with_pos.iter().any(|(pos, _)| *pos == position) {
                    break;
                }

                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::FeatureCompletion,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value: None,
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Extract bug fix claims
        for pattern in &self.bugfix_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                let issue_number = if let Some(issue_match) = captures.get(captures.len() - 1) {
                    issue_match.as_str().parse::<u32>().ok()
                } else {
                    None
                };

                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::BugFix,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value: None,
                        issue_number,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Extract performance claims
        for pattern in &self.performance_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                let numeric_value = captures.get(1).and_then(|m| m.as_str().parse::<f64>().ok());

                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::Performance,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value,
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Extract security claims
        for pattern in &self.security_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures.get(0).expect("Match group 0 always exists for successful regex match");
                let text = full_match.as_str().to_string();
                let position = full_match.start();
                claims_with_pos.push((
                    position,
                    Claim {
                        category: ClaimCategory::Security,
                        text: text.clone(),
                        is_absolute: self.is_absolute_claim(&text),
                        numeric_value: None,
                        issue_number: None,
                        has_scope_qualifier: self.has_scope_qualifier(commit_message),
                        scope: self.extract_scope(commit_message),
                    },
                ));
                break;
            }
        }

        // Sort claims by position in message
        claims_with_pos.sort_by_key(|(pos, _)| *pos);

        // Return claims without position
        claims_with_pos
            .into_iter()
            .map(|(_, claim)| claim)
            .collect()
    }

    fn is_absolute_claim(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.absolute_keywords
            .iter()
            .any(|keyword| text_lower.contains(keyword))
    }

    fn extract_numeric_value(&self, text: &str) -> Option<f64> {
        let num_pattern = Regex::new(r"(\d+)").expect("Hardcoded regex pattern must be valid");
        num_pattern
            .captures(text)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
    }

    fn has_scope_qualifier(&self, commit_message: &str) -> bool {
        self.scope_patterns
            .iter()
            .any(|pattern| pattern.is_match(commit_message))
    }

    fn extract_scope(&self, commit_message: &str) -> Option<String> {
        for pattern in &self.scope_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                if let Some(scope_match) = captures.get(1) {
                    return Some(scope_match.as_str().to_string());
                }
            }
        }
        None
    }
}

impl Default for ClaimExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_extractor_compiles() {
        let extractor = ClaimExtractor::new();
        assert!(!extractor.test_patterns.is_empty());
    }
}
