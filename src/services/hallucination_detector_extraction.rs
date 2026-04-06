// ClaimExtractor implementation: claim extraction from documentation text,
// entity recognition (languages, capabilities), and regex-based pattern matching.

impl ClaimExtractor {
    /// Create new claim extractor with default patterns
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let capability_patterns = vec![
            // Positive capabilities: "PMAT can analyze X"
            Regex::new(r"(?i)PMAT can ([a-z]+)\s+(.+?)(?:\.|$)").expect("internal error"),
            // Negative capabilities: "PMAT cannot compile"
            Regex::new(r"(?i)PMAT cannot ([a-z]+)\s+(.+?)(?:\.|$)").expect("internal error"),
            // Alternative patterns: "PMAT supports X"
            Regex::new(r"(?i)PMAT supports? (.+?)(?:\.|$)").expect("internal error"),
        ];

        let known_languages = vec![
            "Rust",
            "TypeScript",
            "JavaScript",
            "Python",
            "C",
            "C++",
            "Go",
            "Java",
            "Kotlin",
            "Ruby",
            "PHP",
            "Swift",
            "C#",
            "Bash",
            "WASM",
            "Haskell",
            "Elixir",
            "Erlang",
            "OCaml",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            capability_patterns,
            known_languages,
        }
    }

    /// Extract all claims from documentation text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn extract_claims(&self, documentation: &str) -> Vec<Claim> {
        debug_assert!(!documentation.is_empty(), "documentation must not be empty");
        let mut claims = Vec::new();
        let mut in_code_block = false;

        for (line_number, line) in documentation.lines().enumerate() {
            let trimmed = line.trim();

            // Track markdown fenced code blocks (```)
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip lines inside code blocks
            if in_code_block {
                continue;
            }

            // Skip empty lines and headers
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Try to extract capability claims
            if let Some(claim) = self.extract_capability_claim(line, line_number + 1) {
                claims.push(claim);
            }
        }

        claims
    }

    /// Extract capability claim from a line of text
    fn extract_capability_claim(&self, line: &str, line_number: usize) -> Option<Claim> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Check for "PMAT can" pattern
        if let Some(caps) = self.capability_patterns[0].captures(line) {
            let verb = caps.get(1)?.as_str();
            let object = caps.get(2)?.as_str();
            let text = format!("PMAT can {} {}", verb, object);

            let entities = self.extract_entities(&text);

            return Some(Claim {
                source_file: PathBuf::from(""),
                line_number,
                text: text.trim_end_matches('.').to_string(),
                claim_type: ClaimType::Capability,
                entities,
                is_negative: false,
            });
        }

        // Check for "PMAT cannot" pattern (negative capability)
        if let Some(caps) = self.capability_patterns[1].captures(line) {
            let verb = caps.get(1)?.as_str();
            let object = caps.get(2)?.as_str();
            let text = format!("PMAT cannot {} {}", verb, object);

            let entities = self.extract_entities(&text);

            return Some(Claim {
                source_file: PathBuf::from(""),
                line_number,
                text: text.trim_end_matches('.').to_string(),
                claim_type: ClaimType::Capability,
                entities,
                is_negative: true,
            });
        }

        // Check for "PMAT supports" pattern
        if let Some(caps) = self.capability_patterns[2].captures(line) {
            let object = caps.get(1)?.as_str();
            let text = format!("PMAT supports {}", object);

            let entities = self.extract_entities(&text);

            return Some(Claim {
                source_file: PathBuf::from(""),
                line_number,
                text: text.trim_end_matches('.').to_string(),
                claim_type: ClaimType::Capability,
                entities,
                is_negative: false,
            });
        }

        None
    }

    /// Extract entities (languages, capabilities) from claim text
    fn extract_entities(&self, text: &str) -> Vec<Entity> {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let mut entities = Vec::new();

        // Extract language entities
        for language in &self.known_languages {
            if text.contains(language) {
                entities.push(Entity::Language(language.clone()));
            }
        }

        // Extract capability entities (verbs)
        let capability_verbs = vec![
            "analyze", "compile", "support", "detect", "generate", "validate", "parse", "extract",
            "format", "refactor",
        ];

        for verb in capability_verbs {
            if text.to_lowercase().contains(verb) {
                entities.push(Entity::Capability(verb.to_string()));
            }
        }

        entities
    }
}

impl Default for ClaimExtractor {
    fn default() -> Self {
        Self::new()
    }
}
