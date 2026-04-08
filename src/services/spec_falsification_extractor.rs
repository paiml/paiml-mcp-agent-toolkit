/// Extracts falsifiable claims from markdown specification documents
pub struct SpecClaimExtractor {
    path_pattern: Regex,
    entity_pattern: Regex,
    numeric_pattern: Regex,
    rfc2119_must: Regex,
    rfc2119_should: Regex,
    rfc2119_may: Regex,
    absolute_pattern: Regex,
    command_pattern: Regex,
    absence_pattern: Regex,
}

/// Extracted signals from a single line during claim extraction
struct LineSignals {
    path_refs: Vec<String>,
    entity_refs: Vec<String>,
    numeric_value: Option<f64>,
    numeric_comparator: Option<String>,
    has_command: bool,
    has_absence: bool,
}

impl SpecClaimExtractor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            path_pattern: Regex::new(
                r#"(?:^|[\s`"])((?:src|docs|tests|server|crates|\.)/[a-zA-Z0-9_./-]+\.[a-z]+)"#,
            )
            .expect("internal regex"),
            entity_pattern: Regex::new(
                r#"`([A-Z][a-zA-Z0-9]+(?:::[a-z_][a-zA-Z0-9_]*)?)`|`([a-z_][a-z0-9_]+(?:::[a-z_][a-z0-9_]*)+)`"#,
            )
            .expect("internal regex"),
            numeric_pattern: Regex::new(
                r#"([><=]+)\s*(\d+(?:\.\d+)?)\s*(%|ms|s|min|seconds|minutes|lines|functions|files|points|pts)?"#,
            )
            .expect("internal regex"),
            rfc2119_must: Regex::new(r#"\b(MUST|SHALL|REQUIRED|MUST NOT|SHALL NOT)\b"#)
                .expect("internal regex"),
            rfc2119_should: Regex::new(r#"\b(SHOULD|RECOMMENDED|SHOULD NOT)\b"#)
                .expect("internal regex"),
            rfc2119_may: Regex::new(r#"\b(MAY|OPTIONAL)\b"#).expect("internal regex"),
            absolute_pattern: Regex::new(
                r#"\b(all|every|zero|no|none|always|never|complete|entirely|fully)\b"#,
            )
            .expect("internal regex"),
            command_pattern: Regex::new(
                r#"`(pmat\s+[a-z][\w-]*(?:\s+[\w-]+)*)`|`(cargo\s+[a-z][\w-]*(?:\s+[\w-]+)*)`"#,
            )
            .expect("internal regex"),
            absence_pattern: Regex::new(
                r#"(?i)\b(no\s+(?:new\s+)?(?:unsafe|panic|unwrap|todo|fixme|dead.?code)|zero\s+\w+|without\s+any|does not (?:exist|contain|have))\b"#,
            )
            .expect("internal regex"),
        }
    }

    /// Extract all falsifiable claims from a specification document
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn extract(&self, content: &str, source_file: &Path) -> Vec<SpecClaim> {
        let mut claims = Vec::new();
        let mut in_code_block = false;
        let mut claim_counter = 0usize;
        let mut current_section = String::new();

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Track section headers for context
            if trimmed.starts_with('#') {
                current_section = trimmed.trim_start_matches('#').trim().to_string();
                continue;
            }

            // Skip empty lines and table separators
            if trimmed.is_empty() || trimmed.chars().all(|c| c == '-' || c == '|' || c == ' ') {
                continue;
            }

            // Extract claims from this line
            if let Some(claim) = self.extract_claim_from_line(
                trimmed,
                line_idx + 1,
                &mut claim_counter,
                &current_section,
                source_file,
            ) {
                claims.push(claim);
            }
        }

        claims
    }

    fn extract_claim_from_line(
        &self,
        line: &str,
        line_number: usize,
        counter: &mut usize,
        _section: &str,
        _source: &Path,
    ) -> Option<SpecClaim> {
        let priority = self.classify_priority(line);
        let is_absolute = self.absolute_pattern.is_match(&line.to_lowercase());
        let signals = self.extract_signals(line);
        let category = Self::categorize(&signals, priority, is_absolute)?;

        *counter += 1;
        Some(SpecClaim {
            id: format!("claim-{:03}", counter),
            original_text: line.to_string(),
            source_line: line_number,
            category,
            priority,
            is_absolute,
            path_refs: signals.path_refs,
            entity_refs: signals.entity_refs,
            numeric_value: signals.numeric_value,
            numeric_comparator: signals.numeric_comparator,
        })
    }

    fn classify_priority(&self, line: &str) -> ClaimPriority {
        if self.rfc2119_must.is_match(line) {
            ClaimPriority::P0Critical
        } else if self.rfc2119_should.is_match(line) {
            ClaimPriority::P1High
        } else if self.rfc2119_may.is_match(line) {
            ClaimPriority::P2Low
        } else {
            ClaimPriority::P3Default
        }
    }

    fn extract_signals(&self, line: &str) -> LineSignals {
        let path_refs: Vec<String> = self
            .path_pattern
            .captures_iter(line)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .filter(|p| !p.is_empty())
            .collect();

        let entity_refs: Vec<String> = self
            .entity_pattern
            .captures_iter(line)
            .filter_map(|c| c.get(1).or(c.get(2)).map(|m| m.as_str().to_string()))
            .collect();

        let (numeric_value, numeric_comparator) = self
            .numeric_pattern
            .captures(line)
            .and_then(|c| {
                let comp = c.get(1)?.as_str().to_string();
                let val = c.get(2)?.as_str().parse::<f64>().ok()?;
                Some((Some(val), Some(comp)))
            })
            .unwrap_or((None, None));

        LineSignals {
            path_refs,
            entity_refs,
            numeric_value,
            numeric_comparator,
            has_command: self.command_pattern.is_match(line),
            has_absence: self.absence_pattern.is_match(line),
        }
    }

    fn categorize(
        signals: &LineSignals,
        priority: ClaimPriority,
        is_absolute: bool,
    ) -> Option<SpecClaimCategory> {
        if signals.has_absence {
            return Some(SpecClaimCategory::AbsenceClaim);
        }
        if !signals.path_refs.is_empty() {
            return Some(SpecClaimCategory::PathReference);
        }
        if signals.has_command {
            return Some(SpecClaimCategory::CommandClaim);
        }
        if signals.numeric_value.is_some() {
            return Some(SpecClaimCategory::MetricClaim);
        }
        if !signals.entity_refs.is_empty() {
            return Some(SpecClaimCategory::CodeEntity);
        }
        if is_absolute || priority != ClaimPriority::P3Default {
            return Some(SpecClaimCategory::ArchitecturalClaim);
        }
        None
    }
}

impl Default for SpecClaimExtractor {
    fn default() -> Self {
        Self::new()
    }
}
