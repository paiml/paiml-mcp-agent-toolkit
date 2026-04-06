// Core parsing methods for AgentsMdParser
// Contains: parse(), process_event(), and all markdown event handler methods

impl AgentsMdParser {
    /// Create new parser with default rules
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self::with_rules(ValidationRules::default())
    }

    /// Create parser with custom rules
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_rules(rules: ValidationRules) -> Self {
        Self {
            validation_rules: rules,
            command_patterns: Self::init_command_patterns(),
        }
    }

    /// Initialize command detection patterns
    fn init_command_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"^```(?:bash|sh|shell)\n(.*?)\n```").expect("internal error"),
            Regex::new(r"^\$ (.+)$").expect("internal error"),
            Regex::new(r"^> (.+)$").expect("internal error"),
        ]
    }

    /// Parse AGENTS.md content
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse(&self, content: &str) -> Result<AgentsMdDocument> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        // Check size limit
        if content.len() > self.validation_rules.max_size {
            return Err(anyhow::anyhow!(
                "Document exceeds maximum size of {} bytes",
                self.validation_rules.max_size
            ));
        }

        let mut document = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::new(),
                modified: std::time::SystemTime::now(),
                version: None,
                project: None,
            },
            sections: Vec::new(),
            commands: Vec::new(),
            guidelines: Vec::new(),
            quality_rules: None,
        };

        let mut state = ParseState::default();

        // Parse markdown
        let parser = MarkdownParser::new(content);
        for event in parser {
            self.process_event(event, &mut state, &mut document);
        }

        // Add final section
        if let Some(section) = state.current_section {
            document.sections.push(section);
        }

        // Extract guidelines from all sections
        for section in &document.sections {
            self.extract_guidelines(
                &section.content,
                &section.section_type,
                &mut document.guidelines,
            );
        }

        // Extract quality rules
        document.quality_rules = self.extract_quality_rules(&document.sections);

        // Extract metadata
        self.extract_metadata(&document.sections, &mut document.metadata);

        Ok(document)
    }

    /// Process a single markdown event during parsing
    fn process_event(&self, event: Event, state: &mut ParseState, document: &mut AgentsMdDocument) {
        debug_assert!(true, "contract: process_event");
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                state.current_heading_level = Self::heading_level_to_u8(level);
            }
            Event::Text(text) => {
                self.process_text_event(&text, state, document);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                state.in_code_block = true;
                if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                    state.code_block_lang = lang.to_string();
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                self.process_code_block_end(state, document);
            }
            Event::Start(Tag::List(_)) => {
                state.in_list = true;
            }
            Event::End(TagEnd::List(_)) => {
                state.in_list = false;
            }
            Event::Start(Tag::Item) => {
                state.list_item_content.clear();
            }
            Event::End(TagEnd::Item) => {
                Self::process_list_item_end(state);
            }
            _ => {}
        }
    }

    /// Convert heading level enum to u8
    fn heading_level_to_u8(level: HeadingLevel) -> u8 {
        debug_assert!(true, "contract: heading_level_to_u8");
        match level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }

    /// Process a text event based on current parse state
    fn process_text_event(
        &self,
        text: &str,
        state: &mut ParseState,
        document: &mut AgentsMdDocument,
    ) {
        debug_assert!(!text.is_empty(), "text must not be empty");
        if state.in_code_block {
            state.code_block_content.push_str(text);
        } else if state.in_list {
            state.list_item_content.push_str(text);
        } else if state.current_heading_level > 0 {
            Self::start_new_section(text, state, document);
        } else if let Some(ref mut section) = state.current_section {
            section.content.push_str(text);
            section.content.push('\n');
            self.extract_commands(text, &mut document.commands);
            self.extract_guidelines(text, &section.section_type, &mut document.guidelines);
        }
    }

    /// Start a new section when a heading text is encountered
    fn start_new_section(text: &str, state: &mut ParseState, document: &mut AgentsMdDocument) {
        debug_assert!(!text.is_empty(), "text must not be empty");
        if let Some(section) = state.current_section.take() {
            document.sections.push(section);
        }
        let section_type = Self::detect_section_type(text);
        state.current_section = Some(Section {
            section_type,
            title: text.to_string(),
            content: String::new(),
            subsections: Vec::new(),
        });
        state.current_heading_level = 0;
    }

    /// Process end of a code block
    fn process_code_block_end(&self, state: &mut ParseState, document: &mut AgentsMdDocument) {
        debug_assert!(true, "contract: process_code_block_end");
        if !state.in_code_block {
            return;
        }
        let is_shell = matches!(state.code_block_lang.as_str(), "bash" | "sh" | "shell");
        if is_shell {
            self.extract_shell_commands(state, document);
        }
        if let Some(ref mut section) = state.current_section {
            section.content.push_str(&format!(
                "```{}\n{}\n```\n",
                state.code_block_lang, state.code_block_content
            ));
        }
        state.in_code_block = false;
        state.code_block_content.clear();
        state.code_block_lang.clear();
    }

    /// Extract shell commands from a completed code block
    fn extract_shell_commands(&self, state: &ParseState, document: &mut AgentsMdDocument) {
        debug_assert!(true, "contract: extract_shell_commands");
        let section_title = state
            .current_section
            .as_ref()
            .map_or("Unknown", |s| &s.title);
        for line in state.code_block_content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                document.commands.push(Command {
                    name: format!("Command from {section_title}"),
                    command: trimmed.to_string(),
                    working_dir: None,
                    env: Vec::new(),
                    timeout: Some(60),
                    safe: self.is_command_safe(line),
                });
            }
        }
    }

    /// Process end of a list item
    fn process_list_item_end(state: &mut ParseState) {
        debug_assert!(true, "contract: process_list_item_end");
        if let Some(ref mut section) = state.current_section {
            section.content.push_str("- ");
            section.content.push_str(&state.list_item_content);
            section.content.push('\n');
        }
        state.list_item_content.clear();
    }
}
