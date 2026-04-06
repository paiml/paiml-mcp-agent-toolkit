impl TicketFile {
    /// Parse ticket from file
    ///
    /// # Complexity
    /// - Time: O(n) where n is file size
    /// - Cyclomatic: 2
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_file(path: &Path) -> Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = std::fs::read_to_string(path)?;
        let mut ticket = Self::parse_content(&content)?;
        ticket.file_path = path.to_path_buf();
        Ok(ticket)
    }

    /// Parse ticket from string
    ///
    /// # Complexity
    /// - Time: O(n) where n is content length
    /// - Cyclomatic: 8
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse_content(content: &str) -> Result<Self> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let lines: Vec<&str> = content.lines().collect();

        // Extract header (first line)
        let header = lines
            .first()
            .ok_or_else(|| TicketError::ParseError("Empty ticket file".into()))?;

        let (id, title) = parse_header(header)?;

        // Extract metadata
        let status = extract_metadata(&lines, "**Status**")?;
        let priority = extract_metadata(&lines, "**Priority**")?;
        let complexity_str = extract_metadata(&lines, "**Complexity**")?;
        let estimated_time = extract_metadata(&lines, "**Estimated Time**")?;
        let dependencies_str = extract_metadata(&lines, "**Dependencies**")?;
        let sprint = extract_metadata(&lines, "**Sprint**")?;

        // Parse values
        let status = parse_status(&status)?;
        let priority = parse_priority(&priority)?;
        let complexity = parse_complexity(&complexity_str)?;
        let dependencies = parse_dependencies(&dependencies_str);

        // Extract sections
        let objective = extract_section(&lines, "## Objective")?;
        let success_criteria = extract_checklist(&lines, "## Success Criteria")?;

        Ok(TicketFile {
            id,
            title,
            status,
            priority,
            complexity,
            estimated_time,
            dependencies,
            sprint,
            objective,
            success_criteria,
            file_path: PathBuf::new(),
        })
    }

    /// Validate ticket structure
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 5
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate(&self) -> Result<()> {
        // Validate ID format
        if !self.id.starts_with("TICKET-PMAT-") {
            return Err(TicketError::ParseError(format!(
                "Invalid ticket ID: {}",
                self.id
            )));
        }

        // Validate complexity range
        if self.complexity < 1 || self.complexity > 10 {
            return Err(TicketError::InvalidComplexity(self.complexity));
        }

        // Validate has objective
        if self.objective.trim().is_empty() {
            return Err(TicketError::MissingField("Objective".into()));
        }

        // Validate has success criteria
        if self.success_criteria.is_empty() {
            return Err(TicketError::MissingField("Success Criteria".into()));
        }

        Ok(())
    }
}
