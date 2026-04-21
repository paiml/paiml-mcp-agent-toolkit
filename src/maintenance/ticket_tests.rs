#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn test_parse_header_invalid_format() {
        // Wrong prefix hits the "Invalid header format" arm.
        assert!(parse_header("## TICKET-PMAT-0001: Not hash one").is_err());
    }

    #[test]
    fn test_parse_header_missing_title() {
        // No colon means parts.len() < 2 — "Header missing title" arm.
        assert!(parse_header("# TICKET-PMAT-0001 no colon separator").is_err());
    }

    #[test]
    fn test_extract_metadata_missing_field() {
        let lines: Vec<&str> = vec!["**Status**: GREEN", "**Priority**: P0"];
        // Asking for a non-existent key hits the MissingField arm.
        let result = extract_metadata(&lines, "**Sprint**");
        assert!(result.is_err());
    }

    /// ticket_parsing.rs:37-39 — `ok_or_else(|| ParseError("Invalid metadata
    /// format for {key}"))` fires when a line starts with `key` but the
    /// remainder does not start with ":". `.strip_prefix(key)` succeeds but
    /// `.and_then(|s| s.strip_prefix(":"))` returns None.
    #[test]
    fn test_extract_metadata_parse_error_when_key_missing_colon() {
        let lines: Vec<&str> = vec!["**Status**Active"];
        let result = extract_metadata(&lines, "**Status**");
        match result {
            Err(TicketError::ParseError(msg)) => {
                assert!(
                    msg.contains("Invalid metadata format"),
                    "ParseError message must mention the invalid format, got {msg:?}"
                );
                assert!(
                    msg.contains("**Status**"),
                    "ParseError message must echo the key, got {msg:?}"
                );
            }
            other => panic!(
                "expected ParseError for key-without-colon line, got {other:?}"
            ),
        }
    }

    #[test]
    fn test_extract_section_missing_header() {
        // No matching header → empty content → MissingField arm.
        let lines: Vec<&str> = vec!["## Something Else", "body", "## Another"];
        let result = extract_section(&lines, "## Objective");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_priority_p2_variant() {
        // P2 arm isn't covered by the happy-path P0/P1 tests.
        assert_eq!(parse_priority("P2").unwrap(), Priority::P2);
    }

    #[test]
    fn test_parse_status_refactor_variant() {
        // REFACTOR arm isn't covered by the existing RED/Green/COMPLETE test.
        assert_eq!(parse_status("REFACTOR").unwrap(), TicketStatus::Refactor);
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
    fn test_parse_content_success() {
        let content = "# TICKET-PMAT-9001: Coverage Sample\n\n\
             **Status**: RED\n\
             **Priority**: P1\n\
             **Complexity**: 5\n\
             **Estimated Time**: 2 hours\n\
             **Dependencies**: None\n\
             **Sprint**: Sprint 42\n\n\
             ## Objective\n\n\
             Cover parse_content success path.\n\n\
             ## Success Criteria\n\n\
             - [ ] First criterion\n\
             - [ ] Second criterion\n";
        let ticket = TicketFile::parse_content(content).unwrap();
        assert_eq!(ticket.id, "TICKET-PMAT-9001");
        assert_eq!(ticket.title, "Coverage Sample");
        assert_eq!(ticket.status, TicketStatus::Red);
        assert_eq!(ticket.priority, Priority::P1);
        assert_eq!(ticket.complexity, 5);
        assert_eq!(ticket.estimated_time, "2 hours");
        assert!(ticket.dependencies.is_empty());
        assert_eq!(ticket.sprint, "Sprint 42");
        assert!(!ticket.objective.is_empty());
        assert_eq!(ticket.success_criteria.len(), 2);
    }

    #[test]
    fn test_parse_content_empty_errors() {
        let result = TicketFile::parse_content("");
        assert!(result.is_err());
    }

    #[test]
    fn test_ticket_file_from_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("TICKET-PMAT-9002.md");
        let content = "# TICKET-PMAT-9002: File Roundtrip\n\n\
             **Status**: GREEN\n\
             **Priority**: P0\n\
             **Complexity**: 3\n\
             **Estimated Time**: 1 hour\n\
             **Dependencies**: None\n\
             **Sprint**: Sprint 42\n\n\
             ## Objective\n\n\
             Cover from_file path.\n\n\
             ## Success Criteria\n\n\
             - [ ] Only criterion\n";
        std::fs::write(&path, content).unwrap();

        let ticket = TicketFile::from_file(&path).unwrap();
        assert_eq!(ticket.id, "TICKET-PMAT-9002");
        assert_eq!(ticket.file_path, path);
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
    fn test_validate_ticket_bad_id_prefix() {
        let ticket = TicketFile {
            id: "BAD-PREFIX-0001".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1 hour".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Objective".into(),
            success_criteria: vec!["Criterion".into()],
            file_path: PathBuf::new(),
        };
        // Bad prefix hits the ParseError arm.
        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_validate_ticket_empty_objective() {
        let ticket = TicketFile {
            id: "TICKET-PMAT-0001".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1 hour".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "   ".into(), // whitespace-only → trim().is_empty()
            success_criteria: vec!["Criterion".into()],
            file_path: PathBuf::new(),
        };
        // Empty objective hits the MissingField arm.
        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_validate_ticket_empty_success_criteria() {
        let ticket = TicketFile {
            id: "TICKET-PMAT-0001".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1 hour".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Objective".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        };
        // Empty success_criteria hits the final MissingField arm.
        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_list_tickets_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let tickets = list_tickets(dir.path()).unwrap();
        assert!(tickets.is_empty());
    }

    #[test]
    fn test_list_tickets_skips_non_ticket_files() {
        let dir = tempfile::tempdir().unwrap();
        // Non-.md file
        std::fs::write(dir.path().join("readme.txt"), "not markdown").unwrap();
        // .md file without TICKET-PMAT- prefix
        std::fs::write(dir.path().join("notes.md"), "# Just notes").unwrap();
        // Valid ticket
        let content = "# TICKET-PMAT-9100: Sample\n\n\
             **Status**: RED\n\
             **Priority**: P0\n\
             **Complexity**: 3\n\
             **Estimated Time**: 1 hour\n\
             **Dependencies**: None\n\
             **Sprint**: Sprint 1\n\n\
             ## Objective\n\n\
             Sample.\n\n\
             ## Success Criteria\n\n\
             - [ ] Crit\n";
        std::fs::write(dir.path().join("TICKET-PMAT-9100.md"), content).unwrap();

        let tickets = list_tickets(dir.path()).unwrap();
        assert_eq!(tickets.len(), 1);
        assert_eq!(tickets[0].id, "TICKET-PMAT-9100");
    }

    #[test]
    fn test_ticket_exists_with_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("TICKET-PMAT-7777.md"), "# content").unwrap();
        assert!(ticket_exists(dir.path(), "TICKET-PMAT-7777"));
        assert!(!ticket_exists(dir.path(), "TICKET-PMAT-8888"));
    }

    #[test]
    fn integration_parse_ticket_5010() {
        let ticket_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/tickets/TICKET-PMAT-5010.md");

        if !ticket_path.exists() {
            eprintln!("Skipping: ticket file not found at {:?}", ticket_path);
            return;
        }

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
        let tickets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/tickets");

        if !tickets_dir.exists() {
            eprintln!("Skipping: tickets dir not found at {:?}", tickets_dir);
            return;
        }

        let tickets = list_tickets(&tickets_dir).unwrap();

        // Should have at least some tickets (Sprint 16 + Sprint 17 started)
        assert!(!tickets.is_empty());
        assert!(
            tickets.len() >= 5,
            "Expected at least 5 tickets, found {}",
            tickets.len()
        );

        // Verify we can parse real tickets without errors
        for ticket in &tickets {
            assert!(
                ticket.validate().is_ok(),
                "Ticket {} failed validation",
                ticket.id
            );
        }
    }

    #[test]
    fn test_ticket_exists() {
        let tickets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/tickets");

        if !tickets_dir.exists() {
            eprintln!("Skipping: tickets dir not found at {:?}", tickets_dir);
            return;
        }

        assert!(ticket_exists(&tickets_dir, "TICKET-PMAT-5010"));
        assert!(!ticket_exists(&tickets_dir, "TICKET-PMAT-9999"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
