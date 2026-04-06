/// Specification parser
pub struct SpecParser {
    /// Regex for YAML frontmatter
    frontmatter_regex: Regex,
    /// Regex for checkbox items
    checkbox_regex: Regex,
    /// Regex for issue references
    issue_ref_regex: Regex,
    /// Regex for claims (numbered items, MUST/SHALL/SHOULD)
    claim_regex: Regex,
}

impl Default for SpecParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SpecParser {
    pub fn new() -> Self {
        Self {
            frontmatter_regex: Regex::new(r"(?s)^---\n(.*?)\n---").expect("internal error"),
            checkbox_regex: Regex::new(r"^\s*-\s*\[([ xX])\]\s*(.+)$").expect("internal error"),
            issue_ref_regex: Regex::new(r"(?:#(\d+)|GH-(\d+)|Issue\s+#?(\d+))")
                .expect("internal error"),
            claim_regex: Regex::new(r"(?i)(must|shall|should|will)\s+(.+)")
                .expect("internal error"),
        }
    }

    /// Parse a specification file
    pub fn parse_file(&self, path: &Path) -> Result<ParsedSpec> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read specification: {}", path.display()))?;

        self.parse_content(&content, path)
    }

    /// Finalize a code block and add it to the spec
    fn finalize_code_block(
        code_lang: &str,
        code_content: &str,
        code_start_line: usize,
        spec: &mut ParsedSpec,
    ) {
        let is_executable = matches!(
            code_lang,
            "bash" | "sh" | "shell" | "rust" | "python" | "typescript" | "javascript"
        );
        spec.code_examples.push(CodeExample {
            language: code_lang.to_string(),
            code: code_content.trim().to_string(),
            line: code_start_line,
            executable: is_executable,
        });

        if is_executable && !code_content.trim().is_empty() {
            let claim_text = format!(
                "Code example ({}) at line {} compiles/runs correctly",
                code_lang, code_start_line
            );
            spec.claims.push(ValidationClaim {
                id: format!("CODE-{}", spec.code_examples.len()),
                text: claim_text,
                line: code_start_line,
                category: ClaimCategory::Falsifiability,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            });
        }
    }

    /// Extract checkbox items as acceptance criteria and claims
    fn extract_checkbox(&self, line: &str, line_num: usize, section: &str, spec: &mut ParsedSpec) {
        if let Some(caps) = self.checkbox_regex.captures(line) {
            let checked = caps.get(1).map(|m| m.as_str()) == Some("x")
                || caps.get(1).map(|m| m.as_str()) == Some("X");
            let text = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            spec.acceptance_criteria.push(AcceptanceCriterion {
                text: text.clone(),
                complete: checked,
                line: line_num,
            });

            let category =
                ClaimCategory::from_section(section).unwrap_or(ClaimCategory::Implementation);
            spec.claims.push(ValidationClaim {
                id: format!("AC-{}", spec.acceptance_criteria.len()),
                text,
                line: line_num,
                category,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            });
        }
    }

    /// Extract falsification condition claims from bullet points
    fn extract_falsification(line: &str, line_num: usize, spec: &mut ParsedSpec) {
        if line.starts_with("- ") && line.to_lowercase().contains("falsified") {
            let claim_text = line.trim_start_matches("- ").trim().to_string();
            spec.claims.push(ValidationClaim {
                id: format!("FC-{}", spec.claims.len() + 1),
                text: claim_text,
                line: line_num,
                category: ClaimCategory::Falsifiability,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            });
        }
    }

    /// Extract documentation requirement claims from doc sections
    fn extract_doc_requirement(line: &str, line_num: usize, section: &str, spec: &mut ParsedSpec) {
        let section_lower = section.to_lowercase();
        let is_doc_section =
            section_lower.contains("documentation") || section_lower.contains("open science");
        if is_doc_section && line.starts_with("- ") && !line.contains("[ ]") {
            let claim_text = line.trim_start_matches("- ").trim().to_string();
            if !claim_text.is_empty() {
                spec.claims.push(ValidationClaim {
                    id: format!("DOC-{}", spec.claims.len() + 1),
                    text: claim_text,
                    line: line_num,
                    category: ClaimCategory::Documentation,
                    automatable: false,
                    validation_cmd: None,
                    expected_pattern: None,
                });
            }
        }
    }

    /// Check if a claim text is automatable
    fn is_automatable(claim_text: &str) -> bool {
        let lower = claim_text.to_lowercase();
        lower.contains("pmat ")
            || lower.contains("cargo ")
            || lower.contains("test")
            || lower.contains("coverage")
            || lower.contains("compile")
            || lower.contains("build")
            || lower.contains("pass")
            || lower.contains("fail")
            || lower.contains('%')
            || lower.contains("< ")
            || lower.contains("> ")
            || lower.contains("≥")
            || lower.contains("≤")
    }

    /// Extract MUST/SHALL/SHOULD claims from a line
    fn extract_formal_claims(
        &self,
        line: &str,
        line_num: usize,
        section: &str,
        spec: &mut ParsedSpec,
    ) {
        if let Some(caps) = self.claim_regex.captures(line) {
            let verb = caps
                .get(1)
                .map(|m| m.as_str().to_uppercase())
                .unwrap_or_default();
            let claim_text = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let category = Self::categorize_claim(&claim_text, section);
            let automatable = Self::is_automatable(&claim_text);
            let validation_cmd = if automatable {
                self.extract_validation_command(&claim_text)
            } else {
                None
            };

            spec.claims.push(ValidationClaim {
                id: format!("{}-{}", &verb[..1], spec.claims.len() + 1),
                text: format!("{} {}", verb, claim_text),
                line: line_num,
                category,
                automatable,
                validation_cmd,
                expected_pattern: None,
            });
        }
    }

    /// Classify test type from line content
    fn classify_test_type(lower: &str) -> &'static str {
        if lower.contains("unit") {
            "unit"
        } else if lower.contains("integration") {
            "integration"
        } else if lower.contains("property") || lower.contains("proptest") {
            "property"
        } else if lower.contains("e2e") || lower.contains("end-to-end") {
            "e2e"
        } else {
            "general"
        }
    }

    /// Extract test requirements from a line
    fn extract_test_req(&self, line: &str, spec: &mut ParsedSpec) {
        let lower = line.to_lowercase();
        let has_test = lower.contains("test");
        let has_obligation =
            lower.contains("must") || lower.contains("should") || lower.contains("require");
        if has_test && has_obligation {
            spec.test_requirements.push(TestRequirement {
                text: line.trim().to_string(),
                test_type: Self::classify_test_type(&lower).to_string(),
                code_path: self.extract_code_path(line),
            });
        }
    }

    /// Parse specification content
    pub fn parse_content(&self, content: &str, path: &Path) -> Result<ParsedSpec> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let mut spec = ParsedSpec {
            path: path.to_path_buf(),
            title: String::new(),
            issue_refs: Vec::new(),
            status: None,
            claims: Vec::new(),
            code_examples: Vec::new(),
            acceptance_criteria: Vec::new(),
            test_requirements: Vec::new(),
            raw_content: content.to_string(),
        };

        // Extract frontmatter
        if let Some(caps) = self.frontmatter_regex.captures(content) {
            let frontmatter = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            self.parse_frontmatter(frontmatter, &mut spec);
        }

        // Extract title from first H1 if not in frontmatter
        if spec.title.is_empty() {
            for line in content.lines() {
                if line.starts_with("# ") {
                    spec.title = line.trim_start_matches("# ").to_string();
                    break;
                }
            }
        }

        // Extract issue references
        for caps in self.issue_ref_regex.captures_iter(content) {
            let issue_num = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map(|m| m.as_str());
            if let Some(num) = issue_num {
                let ref_str = format!("#{}", num);
                if !spec.issue_refs.contains(&ref_str) {
                    spec.issue_refs.push(ref_str);
                }
            }
        }

        // Parse line by line for structured content
        let lines: Vec<&str> = content.lines().collect();
        let mut current_section = String::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();
        let mut code_start_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            // Track code blocks
            if line.starts_with("```") {
                if in_code_block {
                    Self::finalize_code_block(
                        &code_lang,
                        &code_content,
                        code_start_line,
                        &mut spec,
                    );
                    in_code_block = false;
                    code_content.clear();
                } else {
                    in_code_block = true;
                    code_lang = line.trim_start_matches("```").to_string();
                    code_start_line = line_num;
                }
                continue;
            }

            if in_code_block {
                code_content.push_str(line);
                code_content.push('\n');
                continue;
            }

            // Track sections
            if line.starts_with("## ") || line.starts_with("### ") {
                current_section = line.trim_start_matches('#').trim().to_string();
            }

            self.extract_checkbox(line, line_num, &current_section, &mut spec);
            Self::extract_falsification(line, line_num, &mut spec);
            Self::extract_doc_requirement(line, line_num, &current_section, &mut spec);
            self.extract_formal_claims(line, line_num, &current_section, &mut spec);
            self.extract_test_req(line, &mut spec);
        }

        Ok(spec)
    }

    /// Parse YAML frontmatter
    fn parse_frontmatter(&self, frontmatter: &str, spec: &mut ParsedSpec) {
        // Simple key: value parsing (not full YAML)
        for line in frontmatter.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                match key.as_str() {
                    "title" => spec.title = value,
                    "status" => spec.status = Some(value),
                    "issue" | "issues" | "related" | "issue_refs" | "issue-refs" => {
                        // Parse issue references from frontmatter
                        // Handle YAML array syntax: ["#75", "#96", "#223"]
                        let cleaned = value
                            .trim_start_matches('[')
                            .trim_end_matches(']');
                        for part in cleaned.split(',') {
                            let part = part.trim().trim_matches('"').trim_matches('\'').trim();
                            if !part.is_empty() && !spec.issue_refs.contains(&part.to_string()) {
                                spec.issue_refs.push(part.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Extract a validation command from claim text
    fn extract_validation_command(&self, text: &str) -> Option<String> {
        // Look for `command` patterns
        let cmd_regex = Regex::new(r"`([^`]+)`").ok()?;
        for caps in cmd_regex.captures_iter(text) {
            let cmd = caps.get(1)?.as_str();
            if cmd.starts_with("pmat ") || cmd.starts_with("cargo ") {
                return Some(cmd.to_string());
            }
        }

        // Look for common patterns (case-insensitive)
        let lower = text.to_lowercase();
        if lower.contains("coverage") && text.contains("95%") {
            return Some("pmat analyze coverage --format json".to_string());
        }
        if lower.contains("complexity") {
            return Some("pmat analyze complexity --format json".to_string());
        }
        if lower.contains("test") && lower.contains("pass") {
            return Some("cargo test".to_string());
        }

        None
    }

    /// Extract code path from text
    fn extract_code_path(&self, text: &str) -> Option<String> {
        let path_regex = Regex::new(r"(?:`([^`]+\.[a-z]+)`|(\S+\.[a-z]+))").ok()?;
        for caps in path_regex.captures_iter(text) {
            let path = caps.get(1).or_else(|| caps.get(2))?.as_str();
            if path.ends_with(".rs") || path.ends_with(".py") || path.ends_with(".ts") {
                return Some(path.to_string());
            }
        }
        None
    }

    /// Content-based claim categorization (more accurate than section-based)
    fn categorize_claim(claim_text: &str, section: &str) -> ClaimCategory {
        let lower = claim_text.to_lowercase();
        let section_lower = section.to_lowercase();

        // Falsifiability: claims with concrete metrics, thresholds, or testable assertions
        if lower.contains('%')
            || lower.contains("≥")
            || lower.contains("≤")
            || lower.contains("< ")
            || lower.contains("> ")
            || lower.contains("within")
            || lower.contains("at least")
            || lower.contains("at most")
            || lower.contains("exactly")
            || lower.contains("zero ")
            || lower.contains("no ")
            || lower.contains("all ")
            || lower.contains("none ")
            || lower.contains("compile")
            || lower.contains("pass")
            || lower.contains("fail")
            || section_lower.contains("falsif")
            || section_lower.contains("testab")
            || section_lower.contains("acceptance")
        {
            return ClaimCategory::Falsifiability;
        }

        // Testing: test-related claims
        if lower.contains("test")
            || lower.contains("coverage")
            || lower.contains("mutation")
            || lower.contains("property")
            || section_lower.contains("test")
        {
            return ClaimCategory::Testing;
        }

        // Documentation: doc-related claims
        if lower.contains("document")
            || lower.contains("readme")
            || lower.contains("example")
            || lower.contains("changelog")
            || section_lower.contains("doc")
        {
            return ClaimCategory::Documentation;
        }

        // Integration: external system claims
        if lower.contains("api")
            || lower.contains("integrat")
            || lower.contains("github")
            || lower.contains("ci/cd")
            || lower.contains("deploy")
            || section_lower.contains("integrat")
        {
            return ClaimCategory::Integration;
        }

        // Default to Implementation
        ClaimCategory::from_section(section).unwrap_or(ClaimCategory::Implementation)
    }

    /// Find all specifications in a directory
    pub fn find_specs(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let mut specs = Vec::new();

        if dir.is_file() {
            if dir.extension().map(|e| e == "md").unwrap_or(false) {
                specs.push(dir.to_path_buf());
            }
        } else if dir.is_dir() {
            let pattern = dir.join("**/*.md");
            for path in glob::glob(pattern.to_str().unwrap_or(""))?.flatten() {
                specs.push(path);
            }
        }

        Ok(specs)
    }
}
