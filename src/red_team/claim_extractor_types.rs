#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Category classification for claim.
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
/// Claim.
pub struct Claim {
    pub category: ClaimCategory,
    pub text: String,
    pub is_absolute: bool,          // Contains "all", "zero", "complete"
    pub numeric_value: Option<f64>, // Percentage, count, etc.
    pub issue_number: Option<u32>,  // For bug fix claims
    pub has_scope_qualifier: bool,  // Has "MVP", "Phase N", "Sprint X"
    pub scope: Option<String>,      // The actual scope qualifier
}

/// Claim extractor.
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
