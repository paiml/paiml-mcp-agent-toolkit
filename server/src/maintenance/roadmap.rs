//! Roadmap parsing and validation - TICKET-PMAT-5010
//!
//! Parses ROADMAP.md files into structured data for analysis and validation.

use std::path::Path;
use serde::{Deserialize, Serialize};

/// Represents a parsed roadmap
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Roadmap {
    /// Project version from roadmap title
    pub version: String,
    /// List of sprints in the roadmap
    pub sprints: Vec<Sprint>,
}

/// Represents a sprint in the roadmap
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprint {
    /// Sprint number (e.g., 16, 17)
    pub number: u32,
    /// Sprint name (e.g., "Scaffolding Foundation")
    pub name: String,
    /// Sprint focus area
    pub focus: String,
    /// Sprint status (Complete, In Progress, Planned)
    pub status: SprintStatus,
    /// Estimated duration
    pub duration: String,
    /// Tickets in this sprint
    pub tickets: Vec<Ticket>,
    /// Quality gates for this sprint
    pub quality_gates: Vec<String>,
}

/// Sprint completion status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SprintStatus {
    Complete,
    InProgress,
    Planned,
}

/// Represents a ticket in a sprint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    /// Ticket ID (e.g., "TICKET-PMAT-5001")
    pub id: String,
    /// Ticket description
    pub description: String,
    /// Completion status
    pub completed: bool,
    /// Git commit reference (if completed)
    pub commit: Option<String>,
}

/// Roadmap parsing errors
#[derive(Debug, thiserror::Error)]
pub enum RoadmapError {
    #[error("Failed to read roadmap file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid roadmap format: {0}")]
    ParseError(String),

    #[error("Invalid ticket ID format: {0}")]
    InvalidTicketId(String),

    #[error("Sprint {0} has no tickets")]
    EmptySprint(u32),
}

pub type Result<T> = std::result::Result<T, RoadmapError>;

impl Roadmap {
    /// Parse roadmap from file
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of lines
    /// - Cyclomatic: 2
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    /// Parse roadmap from string
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of lines
    /// - Cyclomatic: 9
    pub fn parse_content(content: &str) -> Result<Self> {
        let mut sprints = Vec::new();
        let mut current_sprint: Option<Sprint> = None;
        let mut next_line_is_focus = false;
        let version = extract_version(content);

        for line in content.lines() {
            if let Some(sprint_info) = parse_sprint_header(line) {
                // Save previous sprint if exists
                if let Some(sprint) = current_sprint.take() {
                    sprints.push(sprint);
                }
                current_sprint = Some(sprint_info);
                next_line_is_focus = true;
            } else if next_line_is_focus && line.starts_with("**Focus:**") {
                if let Some(ref mut sprint) = current_sprint {
                    sprint.focus = line.strip_prefix("**Focus:**").unwrap_or("").trim().to_string();
                }
                next_line_is_focus = false;
            } else if let Some(ticket) = parse_ticket_line(line) {
                if let Some(ref mut sprint) = current_sprint {
                    sprint.tickets.push(ticket);
                }
            } else if is_quality_gate_section(line) {
                // Skip the "**Quality Gates:**" header
                continue;
            } else if current_sprint.is_some() && line.trim().starts_with("- ") && !line.contains("TICKET-") {
                if let Some(gate) = parse_quality_gate(line) {
                    if let Some(ref mut sprint) = current_sprint {
                        sprint.quality_gates.push(gate);
                    }
                }
            }
        }

        // Save last sprint
        if let Some(sprint) = current_sprint {
            sprints.push(sprint);
        }

        Ok(Roadmap { version, sprints })
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
    /// - Cyclomatic: 5
    pub fn validate(&self) -> Result<()> {
        for sprint in &self.sprints {
            // Validate sprint has tickets
            if sprint.tickets.is_empty() {
                return Err(RoadmapError::EmptySprint(sprint.number));
            }

            // Validate ticket IDs
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

/// Extract version from roadmap title
///
/// # Complexity
/// - Time: O(n) where n is content length
/// - Cyclomatic: 3
fn extract_version(content: &str) -> String {
    content
        .lines()
        .find(|line| line.contains("v2."))
        .and_then(|line| {
            line.split("v2.")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
        })
        .map(|s| format!("v2.{}", s))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Parse sprint header line
///
/// # Example
/// "### Sprint 16: Scaffolding Foundation (2-3 days) - COMPLETE ✅"
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 6
fn parse_sprint_header(line: &str) -> Option<Sprint> {
    if !line.starts_with("### Sprint ") {
        return None;
    }

    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    // Extract sprint number
    let number = parts[0]
        .split_whitespace()
        .nth(2)?
        .parse::<u32>()
        .ok()?;

    // Extract name and status
    let rest = parts[1].trim();
    let (name, status) = if rest.contains(" - COMPLETE") {
        (rest.split(" (").next()?.trim(), SprintStatus::Complete)
    } else if rest.contains(" - IN PROGRESS") {
        (rest.split(" (").next()?.trim(), SprintStatus::InProgress)
    } else {
        (rest.split(" (").next()?.trim(), SprintStatus::Planned)
    };

    // Extract duration
    let duration = if let Some(start) = rest.find('(') {
        if let Some(end) = rest.find(')') {
            rest[start + 1..end].to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    Some(Sprint {
        number,
        name: name.to_string(),
        focus: String::new(),
        status,
        duration,
        tickets: Vec::new(),
        quality_gates: Vec::new(),
    })
}

/// Parse ticket line
///
/// # Example
/// "- [x] TICKET-PMAT-5001: Core ScaffoldEngine implementation (commit: 1adfcd7)"
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 5
fn parse_ticket_line(line: &str) -> Option<Ticket> {
    let line = line.trim();

    if !line.starts_with("- [") {
        return None;
    }

    let completed = line.contains("[x]");

    // Extract ticket ID and description
    let content = if completed {
        line.strip_prefix("- [x]")?.trim()
    } else {
        line.strip_prefix("- [ ]")?.trim()
    };

    let parts: Vec<&str> = content.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let id = parts[0].trim().to_string();
    let description = parts[1].split('(').next()?.trim().to_string();

    // Extract commit if present
    let commit = if let Some(commit_start) = content.find("(commit: ") {
        let commit_end = content[commit_start..].find(')')?;
        Some(content[commit_start + 9..commit_start + commit_end].to_string())
    } else {
        None
    };

    Some(Ticket {
        id,
        description,
        completed,
        commit,
    })
}

/// Check if line is quality gate section header
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn is_quality_gate_section(line: &str) -> bool {
    line.trim() == "**Quality Gates:**"
}

/// Parse quality gate line
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn parse_quality_gate(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with("- ") {
        Some(line.strip_prefix("- ")?.to_string())
    } else {
        None
    }
}

/// Validate ticket ID format
///
/// # Format
/// TICKET-PMAT-XXXX where XXXX is 4 digits
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn validate_ticket_id(id: &str) -> Result<()> {
    if !id.starts_with("TICKET-PMAT-") {
        return Err(RoadmapError::InvalidTicketId(id.to_string()));
    }

    let number_part = id.strip_prefix("TICKET-PMAT-")
        .ok_or_else(|| RoadmapError::InvalidTicketId(id.to_string()))?;

    if number_part.len() != 4 || !number_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(RoadmapError::InvalidTicketId(id.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_roadmap() {
        let roadmap = Roadmap::parse_content("").unwrap();
        assert_eq!(roadmap.sprints.len(), 0);
    }

    #[test]
    fn test_parse_sprint_header() {
        let line = "### Sprint 16: Scaffolding Foundation (2-3 days) - COMPLETE ✅";
        let sprint = parse_sprint_header(line).unwrap();

        assert_eq!(sprint.number, 16);
        assert_eq!(sprint.name, "Scaffolding Foundation");
        assert_eq!(sprint.status, SprintStatus::Complete);
        assert_eq!(sprint.duration, "2-3 days");
    }

    #[test]
    fn test_parse_ticket_line_completed() {
        let line = "- [x] TICKET-PMAT-5001: Core ScaffoldEngine implementation (commit: 1adfcd7)";
        let ticket = parse_ticket_line(line).unwrap();

        assert_eq!(ticket.id, "TICKET-PMAT-5001");
        assert_eq!(ticket.description, "Core ScaffoldEngine implementation");
        assert!(ticket.completed);
        assert_eq!(ticket.commit, Some("1adfcd7".to_string()));
    }

    #[test]
    fn test_parse_ticket_line_incomplete() {
        let line = "- [ ] TICKET-PMAT-5010: Roadmap parsing and validation";
        let ticket = parse_ticket_line(line).unwrap();

        assert_eq!(ticket.id, "TICKET-PMAT-5010");
        assert_eq!(ticket.description, "Roadmap parsing and validation");
        assert!(!ticket.completed);
        assert_eq!(ticket.commit, None);
    }

    #[test]
    fn test_validate_ticket_id_valid() {
        assert!(validate_ticket_id("TICKET-PMAT-5001").is_ok());
        assert!(validate_ticket_id("TICKET-PMAT-0001").is_ok());
    }

    #[test]
    fn test_validate_ticket_id_invalid() {
        assert!(validate_ticket_id("TICKET-5001").is_err());
        assert!(validate_ticket_id("TICKET-PMAT-501").is_err());
        assert!(validate_ticket_id("TICKET-PMAT-ABCD").is_err());
    }

    #[test]
    fn test_sprint_completion_percentage() {
        let sprint = Sprint {
            number: 16,
            name: "Test".to_string(),
            focus: "".to_string(),
            status: SprintStatus::InProgress,
            duration: "2 days".to_string(),
            tickets: vec![
                Ticket { id: "TICKET-PMAT-5001".into(), description: "".into(), completed: true, commit: None },
                Ticket { id: "TICKET-PMAT-5002".into(), description: "".into(), completed: true, commit: None },
                Ticket { id: "TICKET-PMAT-5003".into(), description: "".into(), completed: false, commit: None },
            ],
            quality_gates: vec![],
        };

        assert_eq!(sprint.completion_percentage(), 66.66666666666666);
    }

    #[test]
    fn test_sprint_is_complete() {
        let complete_sprint = Sprint {
            number: 16,
            name: "Test".to_string(),
            focus: "".to_string(),
            status: SprintStatus::Complete,
            duration: "2 days".to_string(),
            tickets: vec![
                Ticket { id: "TICKET-PMAT-5001".into(), description: "".into(), completed: true, commit: None },
            ],
            quality_gates: vec![],
        };

        assert!(complete_sprint.is_complete());
    }

    #[test]
    fn integration_parse_real_roadmap() {
        // Parse the actual PMAT ROADMAP.md file
        let roadmap_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("ROADMAP.md");

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();

        // Verify we parsed something
        assert!(!roadmap.sprints.is_empty(), "Should have parsed sprints from ROADMAP.md");

        // Find Sprint 16 (should be complete)
        let sprint16 = roadmap.sprints.iter().find(|s| s.number == 16);
        assert!(sprint16.is_some(), "Sprint 16 should exist in roadmap");

        let sprint16 = sprint16.unwrap();
        assert_eq!(sprint16.name, "Scaffolding Foundation");
        assert_eq!(sprint16.status, SprintStatus::Complete);
        assert_eq!(sprint16.tickets.len(), 5);

        // All Sprint 16 tickets should be complete
        assert!(sprint16.is_complete(), "Sprint 16 should be marked complete");
        assert_eq!(sprint16.completion_percentage(), 100.0);
    }

    #[test]
    fn integration_validate_pmat_roadmap() {
        // Validate the actual PMAT ROADMAP.md structure
        let roadmap_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("ROADMAP.md");

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();

        // Validate structure
        let result = roadmap.validate();
        assert!(result.is_ok(), "PMAT roadmap should pass validation: {:?}", result);
    }

    #[test]
    fn test_parse_complete_roadmap() {
        let content = r#"# PMAT Agent System Roadmap

## 📋 Planned: v2.139.0 - Project Scaffolding

### Sprint 16: Scaffolding Foundation (2-3 days) - COMPLETE ✅
**Focus:** Core scaffolding engine and template system
- [x] TICKET-PMAT-5001: Core ScaffoldEngine implementation (commit: 1adfcd7)
- [x] TICKET-PMAT-5002: Template system (pforge-based agents) (commit: a7cc051)
- [ ] TICKET-PMAT-5003: Template system (wasm-labs-based WASM)

**Quality Gates:**
- Complexity <10 for all functions
- Coverage >80%

### Sprint 17: Maintenance Engine (2-3 days) - TICKET-PMAT-5010
**Focus:** Roadmap and ticket management
- [ ] TICKET-PMAT-5010: Roadmap parsing and validation
- [ ] TICKET-PMAT-5011: Ticket management system
"#;

        let roadmap = Roadmap::parse_content(content).unwrap();

        assert_eq!(roadmap.version, "v2.139.0");
        assert_eq!(roadmap.sprints.len(), 2);

        let sprint16 = &roadmap.sprints[0];
        assert_eq!(sprint16.number, 16);
        assert_eq!(sprint16.name, "Scaffolding Foundation");
        assert_eq!(sprint16.focus, "Core scaffolding engine and template system");
        assert_eq!(sprint16.status, SprintStatus::Complete);
        assert_eq!(sprint16.tickets.len(), 3);
        assert_eq!(sprint16.quality_gates.len(), 2);

        assert!(sprint16.tickets[0].completed);
        assert!(!sprint16.tickets[2].completed);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_ticket_id_validation(num in 0u32..=9999) {
            let id = format!("TICKET-PMAT-{:04}", num);
            prop_assert!(validate_ticket_id(&id).is_ok());
        }

        #[test]
        fn prop_completion_percentage_bounded(
            completed in 0usize..=10,
            total in 1usize..=10
        ) {
            let completed = completed.min(total);
            let tickets: Vec<Ticket> = (0..total)
                .map(|i| Ticket {
                    id: format!("TICKET-PMAT-{:04}", i),
                    description: "test".into(),
                    completed: i < completed,
                    commit: None,
                })
                .collect();

            let sprint = Sprint {
                number: 1,
                name: "test".into(),
                focus: "".into(),
                status: SprintStatus::InProgress,
                duration: "1 day".into(),
                tickets,
                quality_gates: vec![],
            };

            let pct = sprint.completion_percentage();
            prop_assert!(pct >= 0.0 && pct <= 100.0);
        }
    }
}
