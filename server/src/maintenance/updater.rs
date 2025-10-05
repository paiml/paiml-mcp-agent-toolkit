//! Roadmap update utilities for TICKET-PMAT-5013
//!
//! Automatically updates roadmap with commit information.

use super::roadmap::{Roadmap, RoadmapError};
use super::git::{extract_ticket_ids, get_current_commit, ticket_file_updated};
use super::ticket::{TicketFile, TicketStatus};
use std::path::Path;

/// Update roadmap with commit information
///
/// # Complexity
/// - Time: O(n*m) where n=sprints, m=tickets
/// - Cyclomatic: 5
pub fn update_roadmap_ticket(
    roadmap: &mut Roadmap,
    ticket_id: &str,
    commit_hash: &str,
) -> Result<bool, RoadmapError> {
    let mut updated = false;

    for sprint in &mut roadmap.sprints {
        for ticket in &mut sprint.tickets {
            if ticket.id == ticket_id && !ticket.completed {
                ticket.completed = true;
                ticket.commit = Some(commit_hash.to_string());
                updated = true;
                break;
            }
        }
        if updated {
            break;
        }
    }

    Ok(updated)
}

/// Write updated roadmap back to file
///
/// # Complexity
/// - Time: O(n) where n is roadmap size
/// - Cyclomatic: 2
pub fn write_roadmap(roadmap: &Roadmap, path: &Path) -> Result<(), std::io::Error> {
    let content = format_roadmap_markdown(roadmap);
    std::fs::write(path, content)?;
    Ok(())
}

/// Format roadmap as markdown
///
/// # Complexity
/// - Time: O(n*m) where n=sprints, m=tickets
/// - Cyclomatic: 5
fn format_roadmap_markdown(roadmap: &Roadmap) -> String {
    let mut output = String::new();

    output.push_str("# PMAT Agent System Roadmap\n\n");
    output.push_str(&format!("## 📋 Planned: {}\n\n", roadmap.version));

    for sprint in &roadmap.sprints {
        // Sprint header
        let status_marker = if sprint.is_complete() {
            "COMPLETE ✅"
        } else {
            "IN PROGRESS"
        };

        output.push_str(&format!(
            "### Sprint {}: {} ({}) - {}\n",
            sprint.number, sprint.name, sprint.duration, status_marker
        ));

        output.push_str(&format!("**Focus:** {}\n\n", sprint.focus));

        // Tickets
        for ticket in &sprint.tickets {
            let checkbox = if ticket.completed { "[x]" } else { "[ ]" };
            let commit_ref = if let Some(ref commit) = ticket.commit {
                format!(" (commit: {})", &commit[..7.min(commit.len())])
            } else {
                String::new()
            };

            output.push_str(&format!(
                "- {} {}: {}{}\n",
                checkbox, ticket.id, ticket.description, commit_ref
            ));
        }

        output.push('\n');

        // Quality gates
        if !sprint.quality_gates.is_empty() {
            output.push_str("**Quality Gates:**\n");
            for gate in &sprint.quality_gates {
                output.push_str(&format!("- {}\n", gate));
            }
            output.push('\n');
        }
    }

    output
}

/// Update roadmap from current commit
///
/// # TICKET-PMAT-5013
///
/// # Complexity
/// - Time: O(n*m) where n=sprints, m=tickets
/// - Cyclomatic: 7
pub fn update_roadmap_from_commit(
    roadmap_path: &Path,
    tickets_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get current commit info
    let commit = get_current_commit()?;

    // Extract ticket IDs from commit message
    let ticket_ids = extract_ticket_ids(&commit.message);

    if ticket_ids.is_empty() {
        return Ok(()); // No tickets in commit
    }

    // Load roadmap
    let mut roadmap = Roadmap::from_file(roadmap_path)?;
    let mut updated = false;

    // Check each ticket
    for ticket_id in ticket_ids {
        // Only update if ticket file was modified
        if ticket_file_updated(&commit, &ticket_id) {
            // Check if ticket is now GREEN or COMPLETE
            let ticket_path = tickets_dir.join(format!("{}.md", ticket_id));
            if let Ok(ticket_file) = TicketFile::from_file(&ticket_path) {
                if matches!(ticket_file.status, TicketStatus::Green | TicketStatus::Complete) {
                    // Update roadmap
                    if update_roadmap_ticket(&mut roadmap, &ticket_id, &commit.hash)? {
                        updated = true;
                    }
                }
            }
        }
    }

    // Write roadmap if updated
    if updated {
        write_roadmap(&roadmap, roadmap_path)?;
        println!("✓ Updated roadmap with commit {}", &commit.hash[..7.min(commit.hash.len())]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maintenance::roadmap::{Sprint, SprintStatus, Ticket};

    fn create_test_roadmap() -> Roadmap {
        Roadmap {
            version: "v2.139.0".into(),
            sprints: vec![Sprint {
                number: 17,
                name: "Test Sprint".into(),
                focus: "Testing".into(),
                status: SprintStatus::InProgress,
                duration: "2 days".into(),
                tickets: vec![
                    Ticket {
                        id: "TICKET-PMAT-5013".into(),
                        description: "Auto-update hooks".into(),
                        completed: false,
                        commit: None,
                    },
                    Ticket {
                        id: "TICKET-PMAT-5014".into(),
                        description: "Health score".into(),
                        completed: false,
                        commit: None,
                    },
                ],
                quality_gates: vec!["Coverage >80%".into()],
            }],
        }
    }

    #[test]
    fn test_update_roadmap_ticket_success() {
        let mut roadmap = create_test_roadmap();

        let updated = update_roadmap_ticket(&mut roadmap, "TICKET-PMAT-5013", "abc1234").unwrap();

        assert!(updated);
        assert!(roadmap.sprints[0].tickets[0].completed);
        assert_eq!(
            roadmap.sprints[0].tickets[0].commit,
            Some("abc1234".into())
        );
    }

    #[test]
    fn test_update_roadmap_ticket_not_found() {
        let mut roadmap = create_test_roadmap();

        let updated = update_roadmap_ticket(&mut roadmap, "TICKET-PMAT-9999", "abc1234").unwrap();

        assert!(!updated);
    }

    #[test]
    fn test_update_roadmap_ticket_already_completed() {
        let mut roadmap = create_test_roadmap();
        roadmap.sprints[0].tickets[0].completed = true;
        roadmap.sprints[0].tickets[0].commit = Some("old123".into());

        let updated = update_roadmap_ticket(&mut roadmap, "TICKET-PMAT-5013", "abc1234").unwrap();

        assert!(!updated);
        assert_eq!(
            roadmap.sprints[0].tickets[0].commit,
            Some("old123".into())
        );
    }

    #[test]
    fn test_format_roadmap_markdown_structure() {
        let roadmap = create_test_roadmap();
        let markdown = format_roadmap_markdown(&roadmap);

        assert!(markdown.contains("# PMAT Agent System Roadmap"));
        assert!(markdown.contains("## 📋 Planned: v2.139.0"));
        assert!(markdown.contains("Sprint 17"));
        assert!(markdown.contains("Test Sprint"));
    }

    #[test]
    fn test_format_roadmap_markdown_uncompleted_ticket() {
        let roadmap = create_test_roadmap();
        let markdown = format_roadmap_markdown(&roadmap);

        assert!(markdown.contains("[ ] TICKET-PMAT-5013"));
        assert!(!markdown.contains("(commit:"));
    }

    #[test]
    fn test_format_roadmap_markdown_completed_ticket() {
        let mut roadmap = create_test_roadmap();
        roadmap.sprints[0].tickets[0].completed = true;
        roadmap.sprints[0].tickets[0].commit = Some("abc1234567".into());

        let markdown = format_roadmap_markdown(&roadmap);

        assert!(markdown.contains("[x] TICKET-PMAT-5013"));
        assert!(markdown.contains("(commit: abc1234)"));
    }

    #[test]
    fn test_format_roadmap_markdown_quality_gates() {
        let roadmap = create_test_roadmap();
        let markdown = format_roadmap_markdown(&roadmap);

        assert!(markdown.contains("**Quality Gates:**"));
        assert!(markdown.contains("- Coverage >80%"));
    }

    #[test]
    fn test_format_roadmap_markdown_short_commit_hash() {
        let mut roadmap = create_test_roadmap();
        roadmap.sprints[0].tickets[0].completed = true;
        roadmap.sprints[0].tickets[0].commit = Some("abc".into());

        let markdown = format_roadmap_markdown(&roadmap);

        assert!(markdown.contains("(commit: abc)"));
    }

    #[test]
    fn test_write_roadmap_creates_file() {
        use tempfile::NamedTempFile;

        let roadmap = create_test_roadmap();
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Close the file so we can write to it
        drop(temp_file);

        write_roadmap(&roadmap, &temp_path).unwrap();

        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("# PMAT Agent System Roadmap"));
        assert!(content.contains("TICKET-PMAT-5013"));
    }

    #[test]
    fn test_update_multiple_sprints() {
        let mut roadmap = Roadmap {
            version: "v2.139.0".into(),
            sprints: vec![
                Sprint {
                    number: 16,
                    name: "Sprint 16".into(),
                    focus: "Scaffolding".into(),
                    status: SprintStatus::Complete,
                    duration: "2 days".into(),
                    tickets: vec![Ticket {
                        id: "TICKET-PMAT-5001".into(),
                        description: "Core scaffold".into(),
                        completed: true,
                        commit: Some("old123".into()),
                    }],
                    quality_gates: vec![],
                },
                Sprint {
                    number: 17,
                    name: "Sprint 17".into(),
                    focus: "Maintenance".into(),
                    status: SprintStatus::InProgress,
                    duration: "3 days".into(),
                    tickets: vec![Ticket {
                        id: "TICKET-PMAT-5013".into(),
                        description: "Auto-update hooks".into(),
                        completed: false,
                        commit: None,
                    }],
                    quality_gates: vec![],
                },
            ],
        };

        let updated = update_roadmap_ticket(&mut roadmap, "TICKET-PMAT-5013", "new456").unwrap();

        assert!(updated);
        assert!(roadmap.sprints[1].tickets[0].completed);
        assert_eq!(
            roadmap.sprints[1].tickets[0].commit,
            Some("new456".into())
        );
        // First sprint should be unchanged
        assert_eq!(
            roadmap.sprints[0].tickets[0].commit,
            Some("old123".into())
        );
    }
}
