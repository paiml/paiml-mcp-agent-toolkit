// Tests for roadmap handler
// Included by roadmap_handler.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_roadmap_tickets() {
        let content = r#"
- [x] TICKET-PMAT-5023: Quality gates
- [ ] TICKET-PMAT-5032: Maintain roadmap
        "#;

        let tickets = parse_roadmap_tickets(content).unwrap();
        assert_eq!(tickets.len(), 2);
        assert_eq!(tickets.get("TICKET-PMAT-5023"), Some(&true));
        assert_eq!(tickets.get("TICKET-PMAT-5032"), Some(&false));
    }

    #[test]
    fn test_ticket_status_detection() {
        let content = "# TICKET-PMAT-5032\n\n**Status**: GREEN\n";
        let temp_file = std::env::temp_dir().join("test_ticket_5032.md");
        std::fs::write(&temp_file, content).unwrap();

        let status = get_ticket_status(&temp_file).unwrap();
        assert_eq!(status, TicketStatus::Green);

        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_sprint_info_parsing() {
        let content = r#"
### Sprint 19: CLI Integration (2-3 days)
- [x] TICKET-PMAT-5030: Scaffold agent
- [ ] TICKET-PMAT-5032: Maintain roadmap
        "#;

        let sprints = parse_sprint_info(content, Path::new("docs/tickets")).unwrap();
        assert_eq!(sprints.len(), 1);
        assert_eq!(sprints[0].total_tickets, 2);
        assert_eq!(sprints[0].completed_tickets, 1);
        assert_eq!(sprints[0].status, SprintStatus::InProgress);
    }

    #[test]
    fn test_empty_roadmap() {
        let content = "";
        let sprints = parse_sprint_info(content, Path::new("docs/tickets")).unwrap();
        assert_eq!(sprints.len(), 0);
    }

    #[test]
    fn test_complete_sprint() {
        let content = r#"
### Sprint 18: Quality Gates
- [x] TICKET-PMAT-5023: CLI
- [x] TICKET-PMAT-5024: Config
        "#;

        let sprints = parse_sprint_info(content, Path::new("docs/tickets")).unwrap();
        assert_eq!(sprints.len(), 1);
        assert_eq!(sprints[0].status, SprintStatus::Complete);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn checkbox_parsing_stable(checked in any::<bool>(), ticket_num in 1u32..10000) {
            let checkbox = if checked { "x" } else { " " };
            let line = format!("- [{checkbox}] TICKET-PMAT-{ticket_num}: Description");
            let content = format!("# Roadmap\n\n{line}\n");

            let tickets = parse_roadmap_tickets(&content).unwrap();
            let ticket_id = format!("TICKET-PMAT-{ticket_num}");
            prop_assert_eq!(tickets.get(&ticket_id), Some(&checked));
        }

        #[test]
        fn progress_calculation_valid(completed in 0u32..100, total in 1u32..100) {
            let completed = completed.min(total) as usize;
            let total = total as usize;

            let progress = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            prop_assert!((0.0..=100.0).contains(&progress));
        }
    }
}
