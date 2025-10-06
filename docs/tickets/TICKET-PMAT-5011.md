# TICKET-PMAT-5011: Ticket Management System

**Status**: GREEN ✅
**Priority**: P0
**Complexity**: 8
**Estimated Time**: 4 hours
**Dependencies**: TICKET-PMAT-5010
**Sprint**: Sprint 17 - Maintenance Engine

## Objective

Implement a ticket management system that can read, parse, and validate ticket files from `docs/tickets/`. This enables automatic verification that tickets referenced in the roadmap actually exist and have the correct format and metadata.

## Success Criteria

- [ ] Parse ticket files into structured data
- [ ] Extract ticket metadata (ID, status, priority, complexity, dependencies)
- [ ] Validate ticket structure and completeness
- [ ] List all tickets in a directory
- [ ] Check if ticket file exists for a given ID
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_parse_ticket_header` - Extract ticket metadata
- [ ] `test_parse_ticket_sections` - Extract sections (Objective, Success Criteria, etc.)
- [ ] `test_validate_ticket_has_objective` - Validate required sections
- [ ] `test_validate_ticket_has_success_criteria` - Validate checklist exists
- [ ] `test_list_tickets_in_directory` - Scan directory for ticket files
- [ ] `test_ticket_exists` - Check if ticket file exists

### Property Tests
- [ ] Property: All parsed tickets have valid TICKET-PMAT-XXXX IDs
- [ ] Property: Ticket status is one of (RED, GREEN, REFACTOR, COMPLETE)
- [ ] Property: Priority is one of (P0, P1, P2)
- [ ] Property: Complexity is 1-10

### Integration Tests
- [ ] `integration_parse_ticket_5010` - Parse TICKET-PMAT-5010.md
- [ ] `integration_list_all_tickets` - List all tickets in docs/tickets/
- [ ] `integration_validate_all_tickets` - Validate all existing tickets

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Data Structures

```rust
// server/src/maintenance/ticket.rs

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Represents a parsed ticket file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketFile {
    /// Ticket ID (e.g., "TICKET-PMAT-5011")
    pub id: String,
    /// Ticket title
    pub title: String,
    /// Current status (RED, GREEN, REFACTOR, COMPLETE)
    pub status: TicketStatus,
    /// Priority level
    pub priority: Priority,
    /// Complexity estimate (1-10)
    pub complexity: u8,
    /// Estimated time
    pub estimated_time: String,
    /// Dependencies (other ticket IDs)
    pub dependencies: Vec<String>,
    /// Sprint number
    pub sprint: String,
    /// Objective section content
    pub objective: String,
    /// Success criteria checklist
    pub success_criteria: Vec<String>,
    /// File path
    pub file_path: PathBuf,
}

/// Ticket status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TicketStatus {
    Red,
    Green,
    Refactor,
    Complete,
}

/// Priority level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    P0,
    P1,
    P2,
}

/// Ticket management errors
#[derive(Debug, thiserror::Error)]
pub enum TicketError {
    #[error("Failed to read ticket file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid ticket format: {0}")]
    ParseError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Ticket not found: {0}")]
    NotFound(String),

    #[error("Invalid ticket status: {0}")]
    InvalidStatus(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(String),

    #[error("Invalid complexity: {0}")]
    InvalidComplexity(u8),
}

pub type Result<T> = std::result::Result<T, TicketError>;
```

### Phase 2: Parser Implementation

```rust
impl TicketFile {
    /// Parse ticket from file
    ///
    /// # Complexity
    /// - Time: O(n) where n is file size
    /// - Cyclomatic: 2
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut ticket = Self::from_str(&content)?;
        ticket.file_path = path.to_path_buf();
        Ok(ticket)
    }

    /// Parse ticket from string
    ///
    /// # Complexity
    /// - Time: O(n) where n is content length
    /// - Cyclomatic: 8
    pub fn from_str(content: &str) -> Result<Self> {
        let lines: Vec<&str> = content.lines().collect();

        // Extract header (first line)
        let header = lines.first()
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
    pub fn validate(&self) -> Result<()> {
        // Validate ID format
        if !self.id.starts_with("TICKET-PMAT-") {
            return Err(TicketError::ParseError(format!("Invalid ticket ID: {}", self.id)));
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

/// Parse ticket header line
///
/// # Example
/// "# TICKET-PMAT-5011: Ticket Management System"
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn parse_header(line: &str) -> Result<(String, String)> {
    if !line.starts_with("# TICKET-") {
        return Err(TicketError::ParseError("Invalid header format".into()));
    }

    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return Err(TicketError::ParseError("Header missing title".into()));
    }

    let id = parts[0].trim_start_matches("# ").trim().to_string();
    let title = parts[1].trim().to_string();

    Ok((id, title))
}

/// Extract metadata value
///
/// # Complexity
/// - Time: O(n) where n is number of lines
/// - Cyclomatic: 3
fn extract_metadata(lines: &[&str], key: &str) -> Result<String> {
    for line in lines {
        if line.starts_with(key) {
            let value = line.strip_prefix(key)
                .and_then(|s| s.strip_prefix(":"))
                .map(|s| s.trim())
                .ok_or_else(|| TicketError::ParseError(format!("Invalid metadata format for {}", key)))?;
            return Ok(value.to_string());
        }
    }
    Err(TicketError::MissingField(key.to_string()))
}

/// Extract section content
///
/// # Complexity
/// - Time: O(n) where n is number of lines
/// - Cyclomatic: 4
fn extract_section(lines: &[&str], header: &str) -> Result<String> {
    let mut in_section = false;
    let mut content = String::new();

    for line in lines {
        if line.starts_with(header) {
            in_section = true;
            continue;
        }

        if in_section {
            if line.starts_with("## ") {
                // Next section started
                break;
            }
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(line);
        }
    }

    if content.is_empty() {
        Err(TicketError::MissingField(header.to_string()))
    } else {
        Ok(content.trim().to_string())
    }
}

/// Extract checklist items
///
/// # Complexity
/// - Time: O(n) where n is number of lines
/// - Cyclomatic: 4
fn extract_checklist(lines: &[&str], header: &str) -> Result<Vec<String>> {
    let mut in_section = false;
    let mut items = Vec::new();

    for line in lines {
        if line.starts_with(header) {
            in_section = true;
            continue;
        }

        if in_section {
            if line.starts_with("## ") {
                break;
            }
            if line.trim().starts_with("- [ ]") {
                let item = line.trim()
                    .strip_prefix("- [ ]")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                items.push(item);
            }
        }
    }

    Ok(items)
}

/// Parse status string
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 5
fn parse_status(s: &str) -> Result<TicketStatus> {
    match s.to_uppercase().as_str() {
        "RED" => Ok(TicketStatus::Red),
        "GREEN" => Ok(TicketStatus::Green),
        "REFACTOR" => Ok(TicketStatus::Refactor),
        "COMPLETE" => Ok(TicketStatus::Complete),
        _ => Err(TicketError::InvalidStatus(s.to_string())),
    }
}

/// Parse priority string
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 4
fn parse_priority(s: &str) -> Result<Priority> {
    match s.to_uppercase().as_str() {
        "P0" => Ok(Priority::P0),
        "P1" => Ok(Priority::P1),
        "P2" => Ok(Priority::P2),
        _ => Err(TicketError::InvalidPriority(s.to_string())),
    }
}

/// Parse complexity number
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn parse_complexity(s: &str) -> Result<u8> {
    s.parse::<u8>()
        .map_err(|_| TicketError::ParseError(format!("Invalid complexity: {}", s)))
}

/// Parse dependencies list
///
/// # Complexity
/// - Time: O(n) where n is number of dependencies
/// - Cyclomatic: 2
fn parse_dependencies(s: &str) -> Vec<String> {
    if s.to_lowercase() == "none" {
        return Vec::new();
    }

    s.split(',')
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .collect()
}

/// List all tickets in a directory
///
/// # Complexity
/// - Time: O(n) where n is number of files
/// - Cyclomatic: 4
pub fn list_tickets(dir: &Path) -> Result<Vec<TicketFile>> {
    let mut tickets = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if name.starts_with("TICKET-PMAT-") {
                    match TicketFile::from_file(&path) {
                        Ok(ticket) => tickets.push(ticket),
                        Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
                    }
                }
            }
        }
    }

    Ok(tickets)
}

/// Check if ticket exists
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
pub fn ticket_exists(tickets_dir: &Path, ticket_id: &str) -> bool {
    let path = tickets_dir.join(format!("{}.md", ticket_id));
    path.exists()
}
```

### Phase 3: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let header = "# TICKET-PMAT-5011: Ticket Management System";
        let (id, title) = parse_header(header).unwrap();

        assert_eq!(id, "TICKET-PMAT-5011");
        assert_eq!(title, "Ticket Management System");
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(parse_status("RED").unwrap(), TicketStatus::Red);
        assert_eq!(parse_status("Green").unwrap(), TicketStatus::Green);
        assert_eq!(parse_status("COMPLETE").unwrap(), TicketStatus::Complete);
        assert!(parse_status("INVALID").is_err());
    }

    #[test]
    fn test_parse_priority() {
        assert_eq!(parse_priority("P0").unwrap(), Priority::P0);
        assert_eq!(parse_priority("p1").unwrap(), Priority::P1);
        assert!(parse_priority("P3").is_err());
    }

    #[test]
    fn test_parse_complexity() {
        assert_eq!(parse_complexity("8").unwrap(), 8);
        assert!(parse_complexity("invalid").is_err());
    }

    #[test]
    fn test_parse_dependencies() {
        let deps = parse_dependencies("TICKET-PMAT-5010, TICKET-PMAT-5009");
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0], "TICKET-PMAT-5010");

        let no_deps = parse_dependencies("None");
        assert_eq!(no_deps.len(), 0);
    }

    #[test]
    fn test_validate_ticket_valid() {
        let ticket = TicketFile {
            id: "TICKET-PMAT-5011".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 8,
            estimated_time: "4 hours".into(),
            dependencies: vec![],
            sprint: "Sprint 17".into(),
            objective: "Test objective".into(),
            success_criteria: vec!["Criterion 1".into()],
            file_path: PathBuf::new(),
        };

        assert!(ticket.validate().is_ok());
    }

    #[test]
    fn test_validate_ticket_invalid_complexity() {
        let ticket = TicketFile {
            id: "TICKET-PMAT-5011".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 15,
            estimated_time: "4 hours".into(),
            dependencies: vec![],
            sprint: "Sprint 17".into(),
            objective: "Test objective".into(),
            success_criteria: vec!["Criterion 1".into()],
            file_path: PathBuf::new(),
        };

        assert!(ticket.validate().is_err());
    }

    #[test]
    fn integration_parse_ticket_5010() {
        let ticket_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("docs/tickets/TICKET-PMAT-5010.md");

        let ticket = TicketFile::from_file(&ticket_path).unwrap();

        assert_eq!(ticket.id, "TICKET-PMAT-5010");
        assert_eq!(ticket.title, "Roadmap Parsing and Validation");
        assert_eq!(ticket.priority, Priority::P0);
        assert!(ticket.complexity <= 10);
        assert!(!ticket.objective.is_empty());
        assert!(!ticket.success_criteria.is_empty());

        // Validate structure
        assert!(ticket.validate().is_ok());
    }

    #[test]
    fn integration_list_all_tickets() {
        let tickets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("docs/tickets");

        let tickets = list_tickets(&tickets_dir).unwrap();

        // Should have at least TICKET-PMAT-5001 through 5011
        assert!(!tickets.is_empty());
        assert!(tickets.len() >= 11);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_valid_complexity_range(complexity in 1u8..=10) {
            let ticket = TicketFile {
                id: "TICKET-PMAT-0001".into(),
                title: "Test".into(),
                status: TicketStatus::Red,
                priority: Priority::P0,
                complexity,
                estimated_time: "1 hour".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec!["Test".into()],
                file_path: PathBuf::new(),
            };

            prop_assert!(ticket.validate().is_ok());
        }

        #[test]
        fn prop_invalid_complexity_rejected(complexity in 11u8..=255) {
            let ticket = TicketFile {
                id: "TICKET-PMAT-0001".into(),
                title: "Test".into(),
                status: TicketStatus::Red,
                priority: Priority::P0,
                complexity,
                estimated_time: "1 hour".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec!["Test".into()],
                file_path: PathBuf::new(),
            };

            prop_assert!(ticket.validate().is_err());
        }
    }
}
```

## Complexity Analysis

Functions with complexity:
- `TicketFile::from_str`: CC=8 (extracting multiple fields)
- `TicketFile::validate`: CC=5 (validation checks)
- `extract_metadata`: CC=3
- `extract_section`: CC=4
- `extract_checklist`: CC=4
- `parse_status`: CC=5
- `parse_priority`: CC=4
- `list_tickets`: CC=4

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run tests
cargo test --lib maintenance::ticket

# List all tickets
cargo run --bin pmat -- maintain tickets list

# Validate specific ticket
cargo run --bin pmat -- maintain tickets validate TICKET-PMAT-5011
```

## Files to Create/Modify

### New Files
- `server/src/maintenance/ticket.rs` - Ticket parsing implementation

### Modified Files
- `server/src/maintenance/mod.rs` - Add ticket module

## Risk Assessment

**Low Risk:**
- Ticket format is well-defined and consistent
- Parsing is straightforward key-value extraction

**Mitigation:**
- Graceful error handling for malformed tickets
- Property tests for validation invariants
- Integration tests on real ticket files

## Notes

This ticket enables automatic verification that:
- All tickets referenced in roadmap exist as files
- All ticket files have proper structure and metadata
- Dependencies between tickets are valid

Next ticket (TICKET-PMAT-5012) will link roadmap and tickets together for full validation.

**TDD Cycle Duration**: Estimated 2-3 hours for RED → GREEN → REFACTOR
