impl ClaimExtractor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
}
