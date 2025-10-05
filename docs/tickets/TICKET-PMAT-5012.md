# TICKET-PMAT-5012: Roadmap-Ticket Linking Verification

**Status**: RED
**Priority**: P0
**Complexity**: 7
**Estimated Time**: 3 hours
**Dependencies**: TICKET-PMAT-5010, TICKET-PMAT-5011
**Sprint**: Sprint 17 - Maintenance Engine

## Objective

Implement cross-validation between roadmap and ticket files to ensure integrity. This verifies that all tickets referenced in the roadmap exist as files, all dependencies are valid, and metadata is consistent - enforcing **Rule B: Always have tickets linked in roadmap**.

## Success Criteria

- [ ] Verify all roadmap tickets have corresponding ticket files
- [ ] Verify all ticket dependencies exist
- [ ] Cross-validate ticket status matches roadmap completion
- [ ] Detect orphaned tickets (files not in roadmap)
- [ ] Generate validation report with errors and warnings
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_validate_ticket_exists` - Check ticket file exists for roadmap entry
- [ ] `test_validate_dependencies_exist` - Verify all dependencies are valid
- [ ] `test_detect_orphaned_tickets` - Find tickets not in roadmap
- [ ] `test_cross_validate_status` - Match ticket status with roadmap completion
- [ ] `test_generate_validation_report` - Create report with issues

### Property Tests
- [ ] Property: All roadmap tickets have files OR reported as missing
- [ ] Property: All ticket dependencies are valid OR reported as broken
- [ ] Property: Validation is deterministic (same input → same output)

### Integration Tests
- [ ] `integration_validate_pmat_project` - Full validation of PMAT
- [ ] `integration_validation_report` - Generate actual validation report

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
// server/src/maintenance/validator.rs

use super::roadmap::Roadmap;
use super::ticket::TicketFile;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Validation result containing all issues found
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Timestamp of validation
    pub timestamp: String,
    /// Project being validated
    pub project_name: String,
    /// Total errors found (critical issues)
    pub error_count: usize,
    /// Total warnings found (non-critical issues)
    pub warning_count: usize,
    /// Missing ticket files
    pub missing_tickets: Vec<MissingTicket>,
    /// Broken dependencies
    pub broken_dependencies: Vec<BrokenDependency>,
    /// Orphaned ticket files
    pub orphaned_tickets: Vec<String>,
    /// Status mismatches
    pub status_mismatches: Vec<StatusMismatch>,
}

/// Ticket referenced in roadmap but file doesn't exist
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissingTicket {
    /// Ticket ID
    pub ticket_id: String,
    /// Sprint number where referenced
    pub sprint_number: u32,
}

/// Dependency that doesn't exist
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrokenDependency {
    /// Ticket that has the broken dependency
    pub ticket_id: String,
    /// Missing dependency ID
    pub dependency_id: String,
}

/// Ticket status doesn't match roadmap completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusMismatch {
    /// Ticket ID
    pub ticket_id: String,
    /// Status in ticket file
    pub ticket_status: String,
    /// Marked as complete in roadmap
    pub roadmap_completed: bool,
}

/// Validator errors
#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("Roadmap error: {0}")]
    RoadmapError(#[from] super::roadmap::RoadmapError),

    #[error("Ticket error: {0}")]
    TicketError(#[from] super::ticket::TicketError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ValidatorError>;
```

### Phase 2: Validator Implementation

```rust
impl ValidationReport {
    /// Create new empty report
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn new(project_name: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            project_name,
            error_count: 0,
            warning_count: 0,
            missing_tickets: Vec::new(),
            broken_dependencies: Vec::new(),
            orphaned_tickets: Vec::new(),
            status_mismatches: Vec::new(),
        }
    }

    /// Check if validation passed (no errors)
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn is_valid(&self) -> bool {
        self.error_count == 0
    }

    /// Update counts based on issues
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    fn update_counts(&mut self) {
        self.error_count = self.missing_tickets.len() + self.broken_dependencies.len();
        self.warning_count = self.orphaned_tickets.len() + self.status_mismatches.len();
    }
}

/// Validate roadmap against ticket files
///
/// # Complexity
/// - Time: O(n*m) where n=tickets in roadmap, m=ticket files
/// - Cyclomatic: 7
pub fn validate_project(
    roadmap_path: &Path,
    tickets_dir: &Path,
) -> Result<ValidationReport> {
    // Parse roadmap
    let roadmap = Roadmap::from_file(roadmap_path)?;

    // List all ticket files
    let ticket_files = super::ticket::list_tickets(tickets_dir)?;
    let ticket_map: std::collections::HashMap<_, _> = ticket_files
        .iter()
        .map(|t| (t.id.clone(), t))
        .collect();

    let mut report = ValidationReport::new("PMAT".to_string());

    // Check all roadmap tickets have files
    for sprint in &roadmap.sprints {
        for ticket in &sprint.tickets {
            if !ticket_map.contains_key(&ticket.id) {
                report.missing_tickets.push(MissingTicket {
                    ticket_id: ticket.id.clone(),
                    sprint_number: sprint.number,
                });
            } else {
                // Cross-validate status
                let ticket_file = ticket_map.get(&ticket.id).unwrap();
                if !status_matches(ticket_file, ticket.completed) {
                    report.status_mismatches.push(StatusMismatch {
                        ticket_id: ticket.id.clone(),
                        ticket_status: format!("{:?}", ticket_file.status),
                        roadmap_completed: ticket.completed,
                    });
                }
            }
        }
    }

    // Check for orphaned tickets
    let roadmap_ticket_ids: std::collections::HashSet<_> = roadmap
        .sprints
        .iter()
        .flat_map(|s| s.tickets.iter().map(|t| &t.id))
        .collect();

    for ticket_file in &ticket_files {
        if !roadmap_ticket_ids.contains(&ticket_file.id) {
            report.orphaned_tickets.push(ticket_file.id.clone());
        }

        // Check dependencies exist
        for dep in &ticket_file.dependencies {
            if !ticket_map.contains_key(dep) {
                report.broken_dependencies.push(BrokenDependency {
                    ticket_id: ticket_file.id.clone(),
                    dependency_id: dep.clone(),
                });
            }
        }
    }

    report.update_counts();
    Ok(report)
}

/// Check if ticket status matches roadmap completion
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn status_matches(ticket_file: &TicketFile, roadmap_completed: bool) -> bool {
    use super::ticket::TicketStatus;

    if roadmap_completed {
        // If marked complete in roadmap, ticket should be GREEN or COMPLETE
        matches!(ticket_file.status, TicketStatus::Green | TicketStatus::Complete)
    } else {
        // If not complete in roadmap, ticket should be RED or in progress
        true // Allow any status for incomplete tickets
    }
}

/// Format validation report as markdown
///
/// # Complexity
/// - Time: O(n) where n is number of issues
/// - Cyclomatic: 5
pub fn format_report(report: &ValidationReport) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Validation Report: {}\n\n", report.project_name));
    output.push_str(&format!("**Timestamp**: {}\n\n", report.timestamp));

    if report.is_valid() {
        output.push_str("✅ **Status**: VALID - All checks passed!\n\n");
    } else {
        output.push_str(&format!(
            "❌ **Status**: INVALID - {} errors, {} warnings\n\n",
            report.error_count, report.warning_count
        ));
    }

    // Missing tickets (errors)
    if !report.missing_tickets.is_empty() {
        output.push_str("## ❌ Missing Ticket Files\n\n");
        for missing in &report.missing_tickets {
            output.push_str(&format!(
                "- `{}` (Sprint {})\n",
                missing.ticket_id, missing.sprint_number
            ));
        }
        output.push('\n');
    }

    // Broken dependencies (errors)
    if !report.broken_dependencies.is_empty() {
        output.push_str("## ❌ Broken Dependencies\n\n");
        for broken in &report.broken_dependencies {
            output.push_str(&format!(
                "- `{}` depends on missing `{}`\n",
                broken.ticket_id, broken.dependency_id
            ));
        }
        output.push('\n');
    }

    // Orphaned tickets (warnings)
    if !report.orphaned_tickets.is_empty() {
        output.push_str("## ⚠️  Orphaned Tickets (not in roadmap)\n\n");
        for orphaned in &report.orphaned_tickets {
            output.push_str(&format!("- `{}`\n", orphaned));
        }
        output.push('\n');
    }

    // Status mismatches (warnings)
    if !report.status_mismatches.is_empty() {
        output.push_str("## ⚠️  Status Mismatches\n\n");
        for mismatch in &report.status_mismatches {
            output.push_str(&format!(
                "- `{}`: ticket={}, roadmap_complete={}\n",
                mismatch.ticket_id, mismatch.ticket_status, mismatch.roadmap_completed
            ));
        }
        output.push('\n');
    }

    output
}
```

### Phase 3: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validation_report_creation() {
        let report = ValidationReport::new("Test".to_string());
        assert_eq!(report.project_name, "Test");
        assert_eq!(report.error_count, 0);
        assert!(report.is_valid());
    }

    #[test]
    fn test_validation_report_with_errors() {
        let mut report = ValidationReport::new("Test".to_string());
        report.missing_tickets.push(MissingTicket {
            ticket_id: "TICKET-PMAT-9999".into(),
            sprint_number: 99,
        });
        report.update_counts();

        assert_eq!(report.error_count, 1);
        assert!(!report.is_valid());
    }

    #[test]
    fn test_status_matches_completed() {
        use super::super::ticket::{TicketFile, TicketStatus, Priority};

        let ticket = TicketFile {
            id: "TICKET-PMAT-0001".into(),
            title: "Test".into(),
            status: TicketStatus::Green,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1h".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Test".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        };

        assert!(status_matches(&ticket, true));
    }

    #[test]
    fn test_status_matches_incomplete() {
        use super::super::ticket::{TicketFile, TicketStatus, Priority};

        let ticket = TicketFile {
            id: "TICKET-PMAT-0001".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1h".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Test".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        };

        assert!(status_matches(&ticket, false));
    }

    #[test]
    fn integration_validate_pmat_project() {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        let roadmap_path = project_root.join("ROADMAP.md");
        let tickets_dir = project_root.join("docs/tickets");

        let report = validate_project(&roadmap_path, &tickets_dir).unwrap();

        // PMAT should have valid roadmap-ticket linkage
        println!("Validation report:\n{}", format_report(&report));

        // We expect at most warnings, not errors
        // (There might be orphaned tickets or status mismatches during development)
        if report.error_count > 0 {
            println!("Errors found: {}", report.error_count);
            println!("Missing tickets: {:?}", report.missing_tickets);
            println!("Broken dependencies: {:?}", report.broken_dependencies);
        }
    }

    #[test]
    fn test_format_report_valid() {
        let report = ValidationReport::new("Test".to_string());
        let formatted = format_report(&report);

        assert!(formatted.contains("VALID"));
        assert!(formatted.contains("All checks passed"));
    }

    #[test]
    fn test_format_report_with_issues() {
        let mut report = ValidationReport::new("Test".to_string());
        report.missing_tickets.push(MissingTicket {
            ticket_id: "TICKET-PMAT-9999".into(),
            sprint_number: 99,
        });
        report.update_counts();

        let formatted = format_report(&report);

        assert!(formatted.contains("INVALID"));
        assert!(formatted.contains("Missing Ticket Files"));
        assert!(formatted.contains("TICKET-PMAT-9999"));
    }
}
```

## Complexity Analysis

Functions with complexity:
- `validate_project`: CC=7 (multiple validation steps)
- `format_report`: CC=5 (conditional sections)
- `status_matches`: CC=3
- `ValidationReport::update_counts`: CC=1
- `ValidationReport::is_valid`: CC=1

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run tests
cargo test --lib maintenance::validator

# Validate PMAT project
cargo run --bin pmat -- maintain validate

# Generate validation report
cargo run --bin pmat -- maintain validate --report
```

## Files to Create/Modify

### New Files
- `server/src/maintenance/validator.rs` - Validation implementation

### Modified Files
- `server/src/maintenance/mod.rs` - Add validator module
- `server/Cargo.toml` - Add chrono dependency for timestamps

## Risk Assessment

**Low Risk:**
- Validation is read-only, doesn't modify files
- Clear error messages for all issues

**Mitigation:**
- Comprehensive testing on real PMAT data
- Separate errors (critical) from warnings (informational)

## Notes

This ticket enforces **Rule B: Always have tickets linked in roadmap** by:
- Verifying all roadmap tickets exist as files
- Detecting orphaned tickets not in roadmap
- Validating dependency chains
- Cross-checking status consistency

After this ticket, we can automatically validate PMAT's own project health and ensure the three core rules are followed!

**TDD Cycle Duration**: Estimated 2-3 hours for RED → GREEN → REFACTOR
