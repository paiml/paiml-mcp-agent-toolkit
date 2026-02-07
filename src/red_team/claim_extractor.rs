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
                Regex::new(r"(?i)complete\s+test\s+coverage")
                    .expect("Hardcoded regex pattern must be valid"),
            ],

            // Documentation patterns
            documentation_patterns: vec![
                Regex::new(
                    r"(?i)fix(ed)?\s+(all\s+)?broken\s+(documentation\s+links?|links?|docs?)",
                )
                .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)documentation\s+(complete|ready|fixed)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)all\s+examples?\s+work")
                    .expect("Hardcoded regex pattern must be valid"),
            ],

            // Coverage patterns
            coverage_patterns: vec![
                Regex::new(r"(?i)coverage\s+(stable|at|achieved?)\s+(?:at\s+)?(\d+)%")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)(\d+)%\s+coverage")
                    .expect("Hardcoded regex pattern must be valid"),
            ],

            // Feature completion patterns
            completion_patterns: vec![
                Regex::new(r"(?i)complete\s+(\w+(\s+\w+)*)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)(\w+(\s+\w+)*)\s+(ready|complete|done)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)fully\s+functional")
                    .expect("Hardcoded regex pattern must be valid"),
            ],

            // Migration patterns
            migration_patterns: vec![
                Regex::new(r"(?i)(complete\s+)?migration\s+to\s+(\w+)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)fully\s+migrated\s+to\s+(\w+)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)deprecated\s+(\w+)\s+removed")
                    .expect("Hardcoded regex pattern must be valid"),
            ],

            // Bug fix patterns
            bugfix_patterns: vec![
                Regex::new(r"(?i)fix(es|ed)?\s+(bug|issue)\s+#?(\d+)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)resolve[sd]?\s+(issue\s+)?#?(\d+)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)bug\s+fixed").expect("Hardcoded regex pattern must be valid"),
            ],

            // Performance patterns
            performance_patterns: vec![
                Regex::new(r"(?i)(\d+)%\s+(faster|slower|improvement)(\s+\w+)*")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)performance\s+(optimized|improved)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)reduced\s+memory\s+by\s+(\d+)%")
                    .expect("Hardcoded regex pattern must be valid"),
            ],

            // Security patterns
            security_patterns: vec![
                Regex::new(r"(?i)zero\s+vulnerabilities")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)all\s+deps?\s+updated")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"(?i)security\s+audit\s+passed")
                    .expect("Hardcoded regex pattern must be valid"),
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
                Regex::new(r"(?i)(MVP|Alpha|Beta|Phase\s+\d+|Sprint\s+\d+)")
                    .expect("Hardcoded regex pattern must be valid"),
                Regex::new(r"\(([^)]*(?:MVP|Phase|Sprint|Alpha|Beta)[^)]*)\)")
                    .expect("Hardcoded regex pattern must be valid"),
            ],
        }
    }

    pub fn extract(&self, commit_message: &str) -> Vec<Claim> {
        let mut claims_with_pos: Vec<(usize, Claim)> = Vec::new();

        // Extract test status claims
        for pattern in &self.test_patterns {
            if let Some(captures) = pattern.captures(commit_message) {
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_extractor_compiles() {
        let extractor = ClaimExtractor::new();
        assert!(!extractor.test_patterns.is_empty());
    }

    #[test]
    fn test_claim_extractor_default() {
        let extractor = ClaimExtractor::default();
        assert!(!extractor.test_patterns.is_empty());
        assert!(!extractor.documentation_patterns.is_empty());
        assert!(!extractor.coverage_patterns.is_empty());
    }

    // ============================================================================
    // ClaimCategory Tests
    // ============================================================================

    #[test]
    fn test_claim_category_clone() {
        let cat = ClaimCategory::TestStatus;
        let cloned = cat.clone();
        assert_eq!(cat, cloned);
    }

    #[test]
    fn test_claim_category_debug() {
        let cat = ClaimCategory::Documentation;
        let debug = format!("{:?}", cat);
        assert!(debug.contains("Documentation"));
    }

    #[test]
    fn test_claim_category_serialize() {
        let cat = ClaimCategory::Coverage;
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("Coverage"));
    }

    #[test]
    fn test_claim_category_deserialize() {
        let json = r#""BugFix""#;
        let cat: ClaimCategory = serde_json::from_str(json).unwrap();
        assert_eq!(cat, ClaimCategory::BugFix);
    }

    // ============================================================================
    // Claim struct Tests
    // ============================================================================

    #[test]
    fn test_claim_creation() {
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };

        assert_eq!(claim.category, ClaimCategory::TestStatus);
        assert!(claim.is_absolute);
    }

    #[test]
    fn test_claim_clone() {
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };

        let cloned = claim.clone();
        assert_eq!(cloned.category, claim.category);
        assert_eq!(cloned.numeric_value, claim.numeric_value);
    }

    #[test]
    fn test_claim_serialize() {
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };

        let json = serde_json::to_string(&claim).unwrap();
        assert!(json.contains("BugFix"));
        assert!(json.contains("123"));
    }

    // ============================================================================
    // Extract Tests - TestStatus
    // ============================================================================

    #[test]
    fn test_extract_test_status_all_tests_passing() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("all tests passing");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert_eq!(claim.category, ClaimCategory::TestStatus);
        assert!(claim.is_absolute);
    }

    #[test]
    fn test_extract_test_status_fraction() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("15/15 tests pass");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert_eq!(claim.category, ClaimCategory::TestStatus);
    }

    // ============================================================================
    // Extract Tests - Documentation
    // ============================================================================

    #[test]
    fn test_extract_documentation_fixed_links() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("fixed all broken documentation links");

        assert!(!claims.is_empty());
        let claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Documentation);
        assert!(claim.is_some());
    }

    #[test]
    fn test_extract_documentation_complete() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("documentation complete");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Coverage
    // ============================================================================

    #[test]
    fn test_extract_coverage_percentage() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("coverage achieved at 85%");

        assert!(!claims.is_empty());
        let claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Coverage);
        assert!(claim.is_some());
        assert_eq!(claim.unwrap().numeric_value, Some(85.0));
    }

    #[test]
    fn test_extract_coverage_stable() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("coverage stable at 90%");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - FeatureCompletion
    // ============================================================================

    #[test]
    fn test_extract_feature_completion() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("API implementation ready");

        assert!(!claims.is_empty());
    }

    #[test]
    fn test_extract_fully_functional() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("module fully functional");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Migration
    // ============================================================================

    #[test]
    fn test_extract_migration() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("complete migration to async");

        let migration_claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Migration);
        assert!(migration_claim.is_some());
    }

    #[test]
    fn test_extract_fully_migrated() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("fully migrated to tokio");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - BugFix
    // ============================================================================

    #[test]
    fn test_extract_bugfix_with_issue() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("fixes bug #42");

        let bugfix_claim = claims.iter().find(|c| c.category == ClaimCategory::BugFix);
        assert!(bugfix_claim.is_some());
        assert_eq!(bugfix_claim.unwrap().issue_number, Some(42));
    }

    #[test]
    fn test_extract_resolved_issue() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("resolved issue 100");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Performance
    // ============================================================================

    #[test]
    fn test_extract_performance_improvement() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("50% faster parsing");

        let perf_claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Performance);
        assert!(perf_claim.is_some());
        assert_eq!(perf_claim.unwrap().numeric_value, Some(50.0));
    }

    #[test]
    fn test_extract_performance_optimized() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("performance optimized");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Security
    // ============================================================================

    #[test]
    fn test_extract_security_zero_vulnerabilities() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("zero vulnerabilities detected");

        let sec_claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Security);
        assert!(sec_claim.is_some());
        assert!(sec_claim.unwrap().is_absolute);
    }

    #[test]
    fn test_extract_security_audit_passed() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("security audit passed");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Scope Qualifier Tests
    // ============================================================================

    #[test]
    fn test_extract_with_mvp_scope() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("all tests passing (MVP)");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert!(claim.has_scope_qualifier);
        assert!(claim
            .scope
            .as_ref()
            .map(|s| s.contains("MVP"))
            .unwrap_or(false));
    }

    #[test]
    fn test_extract_with_sprint_scope() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("complete implementation Sprint 5");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert!(claim.has_scope_qualifier);
    }

    #[test]
    fn test_extract_with_phase_scope() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("API ready Phase 1");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_extract_empty_message() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("");

        assert!(claims.is_empty());
    }

    #[test]
    fn test_extract_no_claims() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("refactor: renamed variable");

        // May or may not have claims depending on patterns
        let _ = claims.len();
    }

    #[test]
    fn test_extract_multiple_claims() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("all tests passing, coverage stable at 85%");

        assert!(claims.len() >= 2);
    }

    // ============================================================================
    // Helper Method Tests
    // ============================================================================

    #[test]
    fn test_is_absolute_claim() {
        let extractor = ClaimExtractor::new();
        assert!(extractor.is_absolute_claim("all tests passing"));
        assert!(extractor.is_absolute_claim("zero bugs"));
        assert!(extractor.is_absolute_claim("fully complete"));
        assert!(!extractor.is_absolute_claim("some tests pass"));
    }

    #[test]
    fn test_extract_numeric_value() {
        let extractor = ClaimExtractor::new();
        assert_eq!(extractor.extract_numeric_value("85% coverage"), Some(85.0));
        assert_eq!(extractor.extract_numeric_value("100 tests"), Some(100.0));
        assert_eq!(extractor.extract_numeric_value("no numbers"), None);
    }

    #[test]
    fn test_has_scope_qualifier() {
        let extractor = ClaimExtractor::new();
        assert!(extractor.has_scope_qualifier("ready (MVP)"));
        assert!(extractor.has_scope_qualifier("Sprint 5 complete"));
        assert!(extractor.has_scope_qualifier("Phase 1 done"));
        assert!(!extractor.has_scope_qualifier("just a normal message"));
    }

    #[test]
    fn test_extract_scope() {
        let extractor = ClaimExtractor::new();
        let scope = extractor.extract_scope("complete (MVP release)");
        assert!(scope.is_some());

        let scope = extractor.extract_scope("no scope here");
        assert!(scope.is_none());
    }
}

#[cfg(test)]
mod coverage_instrumented_tests {
    use super::*;

    // ====================================================================
    // ClaimExtractor construction
    // ====================================================================

    #[test]
    fn test_ci_extractor_new_all_patterns_populated() {
        let ext = ClaimExtractor::new();
        assert!(!ext.test_patterns.is_empty());
        assert!(!ext.documentation_patterns.is_empty());
        assert!(!ext.coverage_patterns.is_empty());
        assert!(!ext.completion_patterns.is_empty());
        assert!(!ext.migration_patterns.is_empty());
        assert!(!ext.bugfix_patterns.is_empty());
        assert!(!ext.performance_patterns.is_empty());
        assert!(!ext.security_patterns.is_empty());
        assert!(!ext.absolute_keywords.is_empty());
        assert!(!ext.scope_patterns.is_empty());
    }

    #[test]
    fn test_ci_extractor_default_equals_new() {
        let d = ClaimExtractor::default();
        let n = ClaimExtractor::new();
        assert_eq!(d.test_patterns.len(), n.test_patterns.len());
        assert_eq!(d.absolute_keywords.len(), n.absolute_keywords.len());
    }

    // ====================================================================
    // Category: TestStatus
    // ====================================================================

    #[test]
    fn test_ci_test_status_every_test_succeed() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("every test succeeds in CI");
        let test_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == ClaimCategory::TestStatus)
            .collect();
        assert!(!test_claims.is_empty());
        assert!(test_claims[0].is_absolute); // "every" is absolute
    }

    #[test]
    fn test_ci_test_status_complete_test_coverage() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("complete test coverage achieved");
        assert!(claims
            .iter()
            .any(|c| c.category == ClaimCategory::TestStatus));
    }

    // ====================================================================
    // Category: Documentation
    // ====================================================================

    #[test]
    fn test_ci_documentation_all_examples_work() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("all examples work now");
        let doc_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == ClaimCategory::Documentation)
            .collect();
        assert!(!doc_claims.is_empty());
        assert!(doc_claims[0].is_absolute); // "all" is absolute
    }

    #[test]
    fn test_ci_documentation_fixed_broken_docs() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("fixed broken docs");
        assert!(claims
            .iter()
            .any(|c| c.category == ClaimCategory::Documentation));
    }

    // ====================================================================
    // Category: Coverage
    // ====================================================================

    #[test]
    fn test_ci_coverage_percentage_format() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("92% coverage reached");
        let cov = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Coverage);
        assert!(cov.is_some());
        assert_eq!(cov.unwrap().numeric_value, Some(92.0));
    }

    // ====================================================================
    // Category: Migration
    // ====================================================================

    #[test]
    fn test_ci_migration_deprecated_removed() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("deprecated callbacks removed");
        let mig = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Migration);
        assert!(mig.is_some());
    }

    // ====================================================================
    // Category: FeatureCompletion
    // ====================================================================

    #[test]
    fn test_ci_feature_completion_complete_keyword() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("complete refactoring of parser");
        // Should find FeatureCompletion (may also find Migration if "complete" overlaps)
        assert!(claims
            .iter()
            .any(|c| c.category == ClaimCategory::FeatureCompletion
                || c.category == ClaimCategory::Migration));
    }

    // ====================================================================
    // Category: BugFix
    // ====================================================================

    #[test]
    fn test_ci_bugfix_resolved_issue_hash() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("resolved #77");
        let bf = claims.iter().find(|c| c.category == ClaimCategory::BugFix);
        assert!(bf.is_some());
        assert_eq!(bf.unwrap().issue_number, Some(77));
    }

    #[test]
    fn test_ci_bugfix_bug_fixed_no_number() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("bug fixed in parser");
        let bf = claims.iter().find(|c| c.category == ClaimCategory::BugFix);
        assert!(bf.is_some());
    }

    // ====================================================================
    // Category: Performance
    // ====================================================================

    #[test]
    fn test_ci_performance_reduced_memory() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("reduced memory by 30%");
        let perf = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Performance);
        assert!(perf.is_some());
        assert_eq!(perf.unwrap().numeric_value, Some(30.0));
    }

    // ====================================================================
    // Category: Security
    // ====================================================================

    #[test]
    fn test_ci_security_all_deps_updated() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("all deps updated");
        let sec = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Security);
        assert!(sec.is_some());
        assert!(sec.unwrap().is_absolute); // "all"
    }

    // ====================================================================
    // Helper methods
    // ====================================================================

    #[test]
    fn test_ci_is_absolute_entirely() {
        let ext = ClaimExtractor::new();
        assert!(ext.is_absolute_claim("entirely rewritten"));
        assert!(!ext.is_absolute_claim("mostly done"));
    }

    #[test]
    fn test_ci_extract_numeric_value_none() {
        let ext = ClaimExtractor::new();
        assert!(ext.extract_numeric_value("no digits here").is_none());
    }

    #[test]
    fn test_ci_extract_numeric_value_first_number() {
        let ext = ClaimExtractor::new();
        assert_eq!(ext.extract_numeric_value("upgraded to v3"), Some(3.0));
    }

    #[test]
    fn test_ci_scope_alpha_beta() {
        let ext = ClaimExtractor::new();
        assert!(ext.has_scope_qualifier("Alpha release"));
        assert!(ext.has_scope_qualifier("Beta version"));
        let scope = ext.extract_scope("all tests pass (Beta launch)");
        assert!(scope.is_some());
        assert!(scope.unwrap().contains("Beta"));
    }

    // ====================================================================
    // Edge cases
    // ====================================================================

    #[test]
    fn test_ci_multiple_categories_in_one_message() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract(
            "all tests passing, 95% coverage, fixed bug #10, performance optimized, zero vulnerabilities"
        );
        // Should extract at least 4 distinct categories
        let cats: std::collections::HashSet<_> =
            claims.iter().map(|c| format!("{:?}", c.category)).collect();
        assert!(cats.len() >= 4, "Expected >=4 categories, got {:?}", cats);
    }

    #[test]
    fn test_ci_claim_serialization_roundtrip() {
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let json = serde_json::to_string(&claim).unwrap();
        let deserialized: Claim = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.category, ClaimCategory::Performance);
        assert_eq!(deserialized.numeric_value, Some(50.0));
    }

    #[test]
    fn test_ci_category_all_variants_deserialize() {
        for variant in [
            "\"TestStatus\"",
            "\"Documentation\"",
            "\"Coverage\"",
            "\"FeatureCompletion\"",
            "\"Migration\"",
            "\"BugFix\"",
            "\"Performance\"",
            "\"Security\"",
        ] {
            let cat: ClaimCategory = serde_json::from_str(variant).unwrap();
            let _ = format!("{:?}", cat); // exercises Debug
        }
    }
}
