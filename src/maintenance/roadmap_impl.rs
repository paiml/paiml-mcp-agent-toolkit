impl Roadmap {
    /// Parse roadmap from file
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of lines
    /// - Cyclomatic: 2
    pub fn from_file(path: &Path) -> Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = std::fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    /// Parse roadmap from string
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of lines
    /// - Cyclomatic: 9
    pub fn parse_content(content: &str) -> Result<Self> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut state = ParseState {
            sprints: Vec::new(),
            current_sprint: None,
            next_line_is_focus: false,
        };
        let version = extract_version(content);

        for line in content.lines() {
            process_roadmap_line(&mut state, line);
        }

        // Save last sprint
        if let Some(sprint) = state.current_sprint {
            state.sprints.push(sprint);
        }

        Ok(Roadmap {
            version,
            sprints: state.sprints,
        })
    }

    /// Calculate sprint completion percentage
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of sprints
    /// - Cyclomatic: 2
    pub fn completion_percentage(&self, sprint_number: u32) -> Option<f64> {
        self.sprints
            .iter()
            .find(|s| s.number == sprint_number)
            .map(|s| s.completion_percentage())
    }

    /// Validate roadmap structure
    ///
    /// # Complexity
    /// - Time: O(n*m) where n=sprints, m=tickets
    /// - Cyclomatic: 4
    ///
    /// # Note
    /// Modern sprints (38+) may not use ticket format - they use narrative structure.
    /// This is acceptable as roadmap evolved. Only validate ticket IDs if tickets exist.
    pub fn validate(&self) -> Result<()> {
        for sprint in &self.sprints {
            // Validate ticket IDs only if sprint uses ticket format
            for ticket in &sprint.tickets {
                validate_ticket_id(&ticket.id)?;
            }
        }
        Ok(())
    }
}

impl Sprint {
    /// Calculate completion percentage for this sprint
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of tickets
    /// - Cyclomatic: 2
    pub fn completion_percentage(&self) -> f64 {
        if self.tickets.is_empty() {
            return 0.0;
        }

        let completed = self.tickets.iter().filter(|t| t.completed).count();
        (completed as f64 / self.tickets.len() as f64) * 100.0
    }

    /// Check if sprint is complete
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of tickets
    /// - Cyclomatic: 1
    pub fn is_complete(&self) -> bool {
        !self.tickets.is_empty() && self.tickets.iter().all(|t| t.completed)
    }
}

/// Mutable state for roadmap parsing
struct ParseState {
    sprints: Vec<Sprint>,
    current_sprint: Option<Sprint>,
    next_line_is_focus: bool,
}

/// Process a single line during roadmap parsing
fn process_roadmap_line(state: &mut ParseState, line: &str) {
    if let Some(sprint_info) = parse_sprint_header(line) {
        if let Some(sprint) = state.current_sprint.take() {
            state.sprints.push(sprint);
        }
        state.current_sprint = Some(sprint_info);
        state.next_line_is_focus = true;
        return;
    }

    if state.next_line_is_focus && line.starts_with("**Focus:**") {
        if let Some(ref mut sprint) = state.current_sprint {
            sprint.focus = line
                .strip_prefix("**Focus:**")
                .unwrap_or("")
                .trim()
                .to_string();
        }
        state.next_line_is_focus = false;
        return;
    }

    if let Some(ticket) = parse_ticket_line(line) {
        if let Some(ref mut sprint) = state.current_sprint {
            sprint.tickets.push(ticket);
        }
        return;
    }

    if is_quality_gate_section(line) {
        return;
    }

    if state.current_sprint.is_some()
        && line.trim().starts_with("- ")
        && !line.contains("TICKET-")
    {
        if let Some(gate) = parse_quality_gate(line) {
            if let Some(ref mut sprint) = state.current_sprint {
                sprint.quality_gates.push(gate);
            }
        }
    }
}
