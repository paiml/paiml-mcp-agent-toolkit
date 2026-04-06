impl ClaimExtractor {
    /// Find the first matching pattern and return its position, matched text, and captures
    fn match_first_pattern<'a>(
        patterns: &[Regex],
        text: &'a str,
    ) -> Option<(usize, String, regex::Captures<'a>)> {
        for pattern in patterns {
            if let Some(captures) = pattern.captures(text) {
                let full_match = captures
                    .get(0)
                    .expect("Match group 0 always exists for successful regex match");
                return Some((
                    full_match.start(),
                    full_match.as_str().to_string(),
                    captures,
                ));
            }
        }
        None
    }

    /// Build a Claim with common fields populated
    fn build_claim(
        &self,
        category: ClaimCategory,
        text: &str,
        numeric_value: Option<f64>,
        issue_number: Option<u32>,
        commit_message: &str,
    ) -> Claim {
        Claim {
            category,
            text: text.to_string(),
            is_absolute: self.is_absolute_claim(text),
            numeric_value,
            issue_number,
            has_scope_qualifier: self.has_scope_qualifier(commit_message),
            scope: self.extract_scope(commit_message),
        }
    }

    fn extract_test_status(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, caps)) = Self::match_first_pattern(&self.test_patterns, msg) {
            let numeric = caps
                .get(4)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .or_else(|| self.extract_numeric_value(&text));
            claims.push((
                pos,
                self.build_claim(ClaimCategory::TestStatus, &text, numeric, None, msg),
            ));
        }
    }

    fn extract_documentation_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, _)) = Self::match_first_pattern(&self.documentation_patterns, msg) {
            let numeric = self.extract_numeric_value(&text);
            claims.push((
                pos,
                self.build_claim(ClaimCategory::Documentation, &text, numeric, None, msg),
            ));
        }
    }

    fn extract_coverage_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, caps)) = Self::match_first_pattern(&self.coverage_patterns, msg) {
            let numeric = caps
                .get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .or_else(|| caps.get(2).and_then(|m| m.as_str().parse::<f64>().ok()));
            claims.push((
                pos,
                self.build_claim(ClaimCategory::Coverage, &text, numeric, None, msg),
            ));
        }
    }

    fn extract_migration_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, _)) = Self::match_first_pattern(&self.migration_patterns, msg) {
            claims.push((
                pos,
                self.build_claim(ClaimCategory::Migration, &text, None, None, msg),
            ));
        }
    }

    fn extract_completion_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, _)) = Self::match_first_pattern(&self.completion_patterns, msg) {
            // Skip if we already have a claim overlapping this position (e.g., migration)
            if claims.iter().any(|(p, _)| *p == pos) {
                return;
            }
            claims.push((
                pos,
                self.build_claim(ClaimCategory::FeatureCompletion, &text, None, None, msg),
            ));
        }
    }

    fn extract_bugfix_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, caps)) = Self::match_first_pattern(&self.bugfix_patterns, msg) {
            let issue_number = caps
                .get(caps.len() - 1)
                .and_then(|m| m.as_str().parse::<u32>().ok());
            claims.push((
                pos,
                self.build_claim(ClaimCategory::BugFix, &text, None, issue_number, msg),
            ));
        }
    }

    fn extract_performance_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, caps)) = Self::match_first_pattern(&self.performance_patterns, msg)
        {
            let numeric = caps.get(1).and_then(|m| m.as_str().parse::<f64>().ok());
            claims.push((
                pos,
                self.build_claim(ClaimCategory::Performance, &text, numeric, None, msg),
            ));
        }
    }

    fn extract_security_claims(&self, msg: &str, claims: &mut Vec<(usize, Claim)>) {
        if let Some((pos, text, _)) = Self::match_first_pattern(&self.security_patterns, msg) {
            claims.push((
                pos,
                self.build_claim(ClaimCategory::Security, &text, None, None, msg),
            ));
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn extract(&self, commit_message: &str) -> Vec<Claim> {
        let mut claims_with_pos: Vec<(usize, Claim)> = Vec::new();

        self.extract_test_status(commit_message, &mut claims_with_pos);
        self.extract_documentation_claims(commit_message, &mut claims_with_pos);
        self.extract_coverage_claims(commit_message, &mut claims_with_pos);
        self.extract_migration_claims(commit_message, &mut claims_with_pos);
        self.extract_completion_claims(commit_message, &mut claims_with_pos);
        self.extract_bugfix_claims(commit_message, &mut claims_with_pos);
        self.extract_performance_claims(commit_message, &mut claims_with_pos);
        self.extract_security_claims(commit_message, &mut claims_with_pos);

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
