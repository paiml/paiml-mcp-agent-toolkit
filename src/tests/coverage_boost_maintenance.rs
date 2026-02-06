#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for maintenance module (roadmap, ticket, validator)
//! Tests: type construction, serde, parse_content, validation

use crate::maintenance::roadmap::{Roadmap, RoadmapError, Sprint, SprintStatus, Ticket};
use crate::maintenance::ticket::{Priority, TicketError, TicketFile, TicketStatus};
use crate::maintenance::validator::{
    BrokenDependency, MissingTicket, StatusMismatch, ValidationReport,
};

// ============ SprintStatus Tests ============

#[test]
fn test_sprint_status_serde_complete() {
    let status = SprintStatus::Complete;
    let json = serde_json::to_string(&status).unwrap();
    let back: SprintStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back, SprintStatus::Complete);
}

#[test]
fn test_sprint_status_serde_in_progress() {
    let status = SprintStatus::InProgress;
    let json = serde_json::to_string(&status).unwrap();
    let back: SprintStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back, SprintStatus::InProgress);
}

#[test]
fn test_sprint_status_serde_planned() {
    let status = SprintStatus::Planned;
    let json = serde_json::to_string(&status).unwrap();
    let back: SprintStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back, SprintStatus::Planned);
}

#[test]
fn test_sprint_status_clone_eq() {
    let status = SprintStatus::Complete;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

// ============ Ticket Tests ============

#[test]
fn test_ticket_construction_serde() {
    let ticket = Ticket {
        id: "TICKET-PMAT-5001".to_string(),
        description: "Test ticket".to_string(),
        completed: true,
        commit: Some("abc123".to_string()),
    };
    let json = serde_json::to_string(&ticket).unwrap();
    let back: Ticket = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "TICKET-PMAT-5001");
    assert!(back.completed);
    assert_eq!(back.commit, Some("abc123".to_string()));
}

#[test]
fn test_ticket_incomplete() {
    let ticket = Ticket {
        id: "TICKET-PMAT-5002".to_string(),
        description: "Pending ticket".to_string(),
        completed: false,
        commit: None,
    };
    assert!(!ticket.completed);
    assert!(ticket.commit.is_none());
}

// ============ Sprint Tests ============

#[test]
fn test_sprint_construction_serde() {
    let sprint = Sprint {
        number: 16,
        name: "Foundation".to_string(),
        focus: "Core infrastructure".to_string(),
        status: SprintStatus::Complete,
        duration: "2 weeks".to_string(),
        tickets: vec![Ticket {
            id: "TICKET-PMAT-5001".to_string(),
            description: "Setup".to_string(),
            completed: true,
            commit: None,
        }],
        quality_gates: vec!["All tests pass".to_string()],
    };
    let json = serde_json::to_string(&sprint).unwrap();
    let back: Sprint = serde_json::from_str(&json).unwrap();
    assert_eq!(back.number, 16);
    assert_eq!(back.tickets.len(), 1);
}

// ============ Roadmap Parsing Tests ============

#[test]
fn test_roadmap_parse_empty_content() {
    let roadmap = Roadmap::parse_content("").unwrap();
    assert!(roadmap.sprints.is_empty());
}

#[test]
fn test_roadmap_parse_no_sprints() {
    let content = "# PMAT Roadmap v1.0\n\nSome description text.\n";
    let roadmap = Roadmap::parse_content(content).unwrap();
    assert!(roadmap.sprints.is_empty());
}

#[test]
fn test_roadmap_parse_with_sprint() {
    let content = r#"# PMAT Roadmap v2.0

### Sprint 16: Foundation (2 days) - COMPLETE ✅
**Focus:** Core infrastructure

- [x] TICKET-PMAT-5001: Setup project (abc123)
- [ ] TICKET-PMAT-5002: Add tests
"#;
    let roadmap = Roadmap::parse_content(content).unwrap();
    assert!(!roadmap.sprints.is_empty());
}

#[test]
fn test_roadmap_completion_percentage_no_sprint() {
    let roadmap = Roadmap {
        version: "1.0".to_string(),
        sprints: vec![],
    };
    assert!(roadmap.completion_percentage(1).is_none());
}

#[test]
fn test_roadmap_completion_percentage_found() {
    let roadmap = Roadmap {
        version: "1.0".to_string(),
        sprints: vec![Sprint {
            number: 1,
            name: "Test".to_string(),
            focus: "Testing".to_string(),
            status: SprintStatus::Complete,
            duration: "1 week".to_string(),
            tickets: vec![
                Ticket {
                    id: "T-1".to_string(),
                    description: "Done".to_string(),
                    completed: true,
                    commit: None,
                },
                Ticket {
                    id: "T-2".to_string(),
                    description: "Not done".to_string(),
                    completed: false,
                    commit: None,
                },
            ],
            quality_gates: vec![],
        }],
    };
    let pct = roadmap.completion_percentage(1).unwrap();
    assert!((pct - 50.0).abs() < 0.1);
}

#[test]
fn test_roadmap_serde() {
    let roadmap = Roadmap {
        version: "1.0".to_string(),
        sprints: vec![],
    };
    let json = serde_json::to_string(&roadmap).unwrap();
    let back: Roadmap = serde_json::from_str(&json).unwrap();
    assert_eq!(back.version, "1.0");
}

// ============ RoadmapError Tests ============

#[test]
fn test_roadmap_error_parse_display() {
    let err = RoadmapError::ParseError("bad format".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid roadmap format"));
}

#[test]
fn test_roadmap_error_invalid_ticket_id() {
    let err = RoadmapError::InvalidTicketId("BAD-001".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid ticket ID"));
}

#[test]
fn test_roadmap_error_empty_sprint() {
    let err = RoadmapError::EmptySprint(5);
    let msg = format!("{}", err);
    assert!(msg.contains("Sprint 5"));
}

// ============ TicketStatus Tests ============

#[test]
fn test_ticket_status_serde_all() {
    let statuses = vec![
        TicketStatus::Red,
        TicketStatus::Green,
        TicketStatus::Refactor,
        TicketStatus::Complete,
    ];
    for status in &statuses {
        let json = serde_json::to_string(status).unwrap();
        let back: TicketStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*status, back);
    }
}

// ============ Priority Tests ============

#[test]
fn test_priority_serde_all() {
    let priorities = vec![Priority::P0, Priority::P1, Priority::P2];
    for priority in &priorities {
        let json = serde_json::to_string(priority).unwrap();
        let back: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(*priority, back);
    }
}

// ============ TicketFile Parsing Tests ============

#[test]
fn test_ticket_file_parse_content() {
    let content = r#"# TICKET-PMAT-5001: Test Implementation
**Status**: GREEN
**Priority**: P1
**Complexity**: 3
**Estimated Time**: 2 hours
**Dependencies**: None
**Sprint**: Sprint 16

## Objective

Implement the test framework.

## Success Criteria

- [ ] All tests pass
- [ ] Coverage above 80%
"#;
    let ticket = TicketFile::parse_content(content).unwrap();
    assert_eq!(ticket.id, "TICKET-PMAT-5001");
    assert_eq!(ticket.title, "Test Implementation");
    assert_eq!(ticket.status, TicketStatus::Green);
    assert_eq!(ticket.priority, Priority::P1);
    assert_eq!(ticket.complexity, 3);
    assert!(!ticket.objective.is_empty());
    assert_eq!(ticket.success_criteria.len(), 2);
}

#[test]
fn test_ticket_file_parse_with_dependencies() {
    let content = r#"# TICKET-PMAT-5002: Dependent Task
**Status**: RED
**Priority**: P0
**Complexity**: 5
**Estimated Time**: 4 hours
**Dependencies**: TICKET-PMAT-5001, TICKET-PMAT-5000
**Sprint**: Sprint 17

## Objective

Build on previous work.

## Success Criteria

- [ ] Integration works
"#;
    let ticket = TicketFile::parse_content(content).unwrap();
    assert_eq!(ticket.dependencies.len(), 2);
    assert_eq!(ticket.dependencies[0], "TICKET-PMAT-5001");
}

#[test]
fn test_ticket_file_parse_empty_content_error() {
    let result = TicketFile::parse_content("");
    assert!(result.is_err());
}

#[test]
fn test_ticket_file_parse_invalid_header_error() {
    let result = TicketFile::parse_content("Not a valid header\n");
    assert!(result.is_err());
}

#[test]
fn test_ticket_file_validate_valid() {
    let ticket = TicketFile {
        id: "TICKET-PMAT-5001".to_string(),
        title: "Test".to_string(),
        status: TicketStatus::Green,
        priority: Priority::P1,
        complexity: 5,
        estimated_time: "2h".to_string(),
        dependencies: vec![],
        sprint: "Sprint 16".to_string(),
        objective: "Do something".to_string(),
        success_criteria: vec!["It works".to_string()],
        file_path: std::path::PathBuf::new(),
    };
    assert!(ticket.validate().is_ok());
}

#[test]
fn test_ticket_file_validate_bad_id() {
    let ticket = TicketFile {
        id: "BAD-ID".to_string(),
        title: "Test".to_string(),
        status: TicketStatus::Green,
        priority: Priority::P1,
        complexity: 5,
        estimated_time: "2h".to_string(),
        dependencies: vec![],
        sprint: "Sprint 16".to_string(),
        objective: "Do something".to_string(),
        success_criteria: vec!["It works".to_string()],
        file_path: std::path::PathBuf::new(),
    };
    assert!(ticket.validate().is_err());
}

#[test]
fn test_ticket_file_validate_bad_complexity() {
    let ticket = TicketFile {
        id: "TICKET-PMAT-5001".to_string(),
        title: "Test".to_string(),
        status: TicketStatus::Green,
        priority: Priority::P1,
        complexity: 0, // invalid: must be 1-10
        estimated_time: "2h".to_string(),
        dependencies: vec![],
        sprint: "Sprint 16".to_string(),
        objective: "Do something".to_string(),
        success_criteria: vec!["It works".to_string()],
        file_path: std::path::PathBuf::new(),
    };
    assert!(ticket.validate().is_err());
}

#[test]
fn test_ticket_file_validate_empty_objective() {
    let ticket = TicketFile {
        id: "TICKET-PMAT-5001".to_string(),
        title: "Test".to_string(),
        status: TicketStatus::Green,
        priority: Priority::P1,
        complexity: 5,
        estimated_time: "2h".to_string(),
        dependencies: vec![],
        sprint: "Sprint 16".to_string(),
        objective: "".to_string(),
        success_criteria: vec!["It works".to_string()],
        file_path: std::path::PathBuf::new(),
    };
    assert!(ticket.validate().is_err());
}

#[test]
fn test_ticket_file_validate_no_criteria() {
    let ticket = TicketFile {
        id: "TICKET-PMAT-5001".to_string(),
        title: "Test".to_string(),
        status: TicketStatus::Green,
        priority: Priority::P1,
        complexity: 5,
        estimated_time: "2h".to_string(),
        dependencies: vec![],
        sprint: "Sprint 16".to_string(),
        objective: "Do something".to_string(),
        success_criteria: vec![],
        file_path: std::path::PathBuf::new(),
    };
    assert!(ticket.validate().is_err());
}

#[test]
fn test_ticket_file_serde() {
    let ticket = TicketFile {
        id: "TICKET-PMAT-5001".to_string(),
        title: "Test".to_string(),
        status: TicketStatus::Green,
        priority: Priority::P1,
        complexity: 5,
        estimated_time: "2h".to_string(),
        dependencies: vec!["TICKET-PMAT-5000".to_string()],
        sprint: "Sprint 16".to_string(),
        objective: "Do something".to_string(),
        success_criteria: vec!["It works".to_string()],
        file_path: std::path::PathBuf::from("ticket.md"),
    };
    let json = serde_json::to_string(&ticket).unwrap();
    let back: TicketFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "TICKET-PMAT-5001");
}

// ============ TicketError Tests ============

#[test]
fn test_ticket_error_parse_display() {
    let err = TicketError::ParseError("bad".to_string());
    assert!(format!("{}", err).contains("Invalid ticket format"));
}

#[test]
fn test_ticket_error_missing_field() {
    let err = TicketError::MissingField("Status".to_string());
    assert!(format!("{}", err).contains("Missing required field"));
}

#[test]
fn test_ticket_error_not_found() {
    let err = TicketError::NotFound("TICKET-999".to_string());
    assert!(format!("{}", err).contains("not found"));
}

#[test]
fn test_ticket_error_invalid_status() {
    let err = TicketError::InvalidStatus("UNKNOWN".to_string());
    assert!(format!("{}", err).contains("Invalid ticket status"));
}

#[test]
fn test_ticket_error_invalid_priority() {
    let err = TicketError::InvalidPriority("P99".to_string());
    assert!(format!("{}", err).contains("Invalid priority"));
}

#[test]
fn test_ticket_error_invalid_complexity() {
    let err = TicketError::InvalidComplexity(99);
    assert!(format!("{}", err).contains("Invalid complexity"));
}

// ============ ValidationReport Tests ============

#[test]
fn test_validation_report_new() {
    let report = ValidationReport::new("PMAT".to_string());
    assert_eq!(report.project_name, "PMAT");
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 0);
    assert!(report.missing_tickets.is_empty());
    assert!(report.broken_dependencies.is_empty());
    assert!(report.orphaned_tickets.is_empty());
    assert!(report.status_mismatches.is_empty());
}

#[test]
fn test_validation_report_is_valid_true() {
    let report = ValidationReport::new("Test".to_string());
    assert!(report.is_valid());
}

#[test]
fn test_validation_report_is_valid_false() {
    let mut report = ValidationReport::new("Test".to_string());
    report.error_count = 1;
    assert!(!report.is_valid());
}

#[test]
fn test_validation_report_serde() {
    let report = ValidationReport::new("Test".to_string());
    let json = serde_json::to_string(&report).unwrap();
    let back: ValidationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.project_name, "Test");
}

// ============ MissingTicket Tests ============

#[test]
fn test_missing_ticket_serde() {
    let mt = MissingTicket {
        ticket_id: "TICKET-PMAT-5001".to_string(),
        sprint_number: 16,
    };
    let json = serde_json::to_string(&mt).unwrap();
    let back: MissingTicket = serde_json::from_str(&json).unwrap();
    assert_eq!(back.ticket_id, "TICKET-PMAT-5001");
    assert_eq!(back.sprint_number, 16);
}

// ============ BrokenDependency Tests ============

#[test]
fn test_broken_dependency_serde() {
    let bd = BrokenDependency {
        ticket_id: "TICKET-PMAT-5002".to_string(),
        dependency_id: "TICKET-PMAT-5001".to_string(),
    };
    let json = serde_json::to_string(&bd).unwrap();
    let back: BrokenDependency = serde_json::from_str(&json).unwrap();
    assert_eq!(back.ticket_id, "TICKET-PMAT-5002");
}

// ============ StatusMismatch Tests ============

#[test]
fn test_status_mismatch_serde() {
    let sm = StatusMismatch {
        ticket_id: "TICKET-PMAT-5001".to_string(),
        ticket_status: "RED".to_string(),
        roadmap_completed: true,
    };
    let json = serde_json::to_string(&sm).unwrap();
    let back: StatusMismatch = serde_json::from_str(&json).unwrap();
    assert_eq!(back.ticket_id, "TICKET-PMAT-5001");
    assert!(back.roadmap_completed);
}
