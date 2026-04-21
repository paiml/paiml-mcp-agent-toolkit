#[cfg_attr(coverage_nightly, coverage(off))]
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
                Ticket {
                    id: "TICKET-PMAT-5001".into(),
                    description: "".into(),
                    completed: true,
                    commit: None,
                },
                Ticket {
                    id: "TICKET-PMAT-5002".into(),
                    description: "".into(),
                    completed: true,
                    commit: None,
                },
                Ticket {
                    id: "TICKET-PMAT-5003".into(),
                    description: "".into(),
                    completed: false,
                    commit: None,
                },
            ],
            quality_gates: vec![],
        };

        assert_eq!(sprint.completion_percentage(), 66.66666666666666);
    }

    #[test]
    fn test_sprint_completion_percentage_empty() {
        let sprint = Sprint {
            number: 99,
            name: "Empty".to_string(),
            focus: "".to_string(),
            status: SprintStatus::Planned,
            duration: "0 days".to_string(),
            tickets: vec![],
            quality_gates: vec![],
        };
        // Empty tickets branch returns 0.0
        assert_eq!(sprint.completion_percentage(), 0.0);
    }

    #[test]
    fn test_roadmap_completion_percentage_found() {
        let roadmap = Roadmap {
            version: "1.0".to_string(),
            sprints: vec![Sprint {
                number: 42,
                name: "Target".to_string(),
                focus: "".to_string(),
                status: SprintStatus::InProgress,
                duration: "1 day".to_string(),
                tickets: vec![
                    Ticket {
                        id: "TICKET-PMAT-4201".into(),
                        description: "".into(),
                        completed: true,
                        commit: None,
                    },
                    Ticket {
                        id: "TICKET-PMAT-4202".into(),
                        description: "".into(),
                        completed: false,
                        commit: None,
                    },
                ],
                quality_gates: vec![],
            }],
        };
        let pct = roadmap.completion_percentage(42);
        assert!(pct.is_some());
        assert!((pct.unwrap() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_roadmap_completion_percentage_missing_sprint() {
        let roadmap = Roadmap {
            version: "1.0".to_string(),
            sprints: vec![],
        };
        assert!(roadmap.completion_percentage(999).is_none());
    }

    #[test]
    fn test_roadmap_from_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ROADMAP.md");
        let content = "### Sprint 7: Demo Sprint (1 day) - IN PROGRESS\n\
             **Focus:** testing\n\
             - [x] TICKET-PMAT-0007: Done item (commit: abc1234)\n\
             - [ ] TICKET-PMAT-0008: Pending item\n";
        std::fs::write(&path, content).unwrap();

        let roadmap = Roadmap::from_file(&path).unwrap();
        assert_eq!(roadmap.sprints.len(), 1);
        assert_eq!(roadmap.sprints[0].number, 7);
        assert_eq!(roadmap.sprints[0].tickets.len(), 2);
    }

    #[test]
    fn test_roadmap_validate_accepts_valid_ids() {
        let roadmap = Roadmap {
            version: "1.0".to_string(),
            sprints: vec![Sprint {
                number: 1,
                name: "Ok".to_string(),
                focus: "".to_string(),
                status: SprintStatus::Complete,
                duration: "1 day".to_string(),
                tickets: vec![Ticket {
                    id: "TICKET-PMAT-0001".into(),
                    description: "".into(),
                    completed: true,
                    commit: None,
                }],
                quality_gates: vec![],
            }],
        };
        assert!(roadmap.validate().is_ok());
    }

    #[test]
    fn test_roadmap_validate_rejects_bad_ticket_id() {
        let roadmap = Roadmap {
            version: "1.0".to_string(),
            sprints: vec![Sprint {
                number: 1,
                name: "Bad".to_string(),
                focus: "".to_string(),
                status: SprintStatus::Complete,
                duration: "1 day".to_string(),
                tickets: vec![Ticket {
                    id: "BAD-FORMAT".into(),
                    description: "".into(),
                    completed: false,
                    commit: None,
                }],
                quality_gates: vec![],
            }],
        };
        assert!(roadmap.validate().is_err());
    }

    #[test]
    fn test_sprint_is_complete() {
        let complete_sprint = Sprint {
            number: 16,
            name: "Test".to_string(),
            focus: "".to_string(),
            status: SprintStatus::Complete,
            duration: "2 days".to_string(),
            tickets: vec![Ticket {
                id: "TICKET-PMAT-5001".into(),
                description: "".into(),
                completed: true,
                commit: None,
            }],
            quality_gates: vec![],
        };

        assert!(complete_sprint.is_complete());
    }

    #[test]
    fn integration_parse_real_roadmap() {
        // Parse the actual PMAT ROADMAP.md file
        let roadmap_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ROADMAP.md");

        if !roadmap_path.exists() {
            eprintln!("Skipping: ROADMAP.md not found at {:?}", roadmap_path);
            return;
        }

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();

        // Verify we parsed something
        assert!(
            !roadmap.sprints.is_empty(),
            "Should have parsed sprints from ROADMAP.md"
        );

        // Find Sprint 16 (should be complete)
        let sprint16 = roadmap.sprints.iter().find(|s| s.number == 16);
        assert!(sprint16.is_some(), "Sprint 16 should exist in roadmap");

        let sprint16 = sprint16.unwrap();
        assert_eq!(sprint16.name, "Scaffolding Foundation");
        assert_eq!(sprint16.status, SprintStatus::Complete);
        assert_eq!(sprint16.tickets.len(), 5);

        // All Sprint 16 tickets should be complete
        assert!(
            sprint16.is_complete(),
            "Sprint 16 should be marked complete"
        );
        assert_eq!(sprint16.completion_percentage(), 100.0);
    }

    #[test]
    fn integration_validate_pmat_roadmap() {
        // Validate the actual PMAT ROADMAP.md structure
        let roadmap_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ROADMAP.md");

        if !roadmap_path.exists() {
            eprintln!("Skipping: ROADMAP.md not found at {:?}", roadmap_path);
            return;
        }

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();

        // Validate structure
        let result = roadmap.validate();
        assert!(
            result.is_ok(),
            "PMAT roadmap should pass validation: {:?}",
            result
        );
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
        assert_eq!(
            sprint16.focus,
            "Core scaffolding engine and template system"
        );
        assert_eq!(sprint16.status, SprintStatus::Complete);
        assert_eq!(sprint16.tickets.len(), 3);
        assert_eq!(sprint16.quality_gates.len(), 2);

        assert!(sprint16.tickets[0].completed);
        assert!(!sprint16.tickets[2].completed);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
            prop_assert!((0.0..=100.0).contains(&pct));
        }
    }
}
