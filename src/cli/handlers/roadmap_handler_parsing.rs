// Parsing and utility functions for roadmap/ticket data
// Included by roadmap_handler.rs - shares parent module scope

/// Parse ticket IDs and checkbox status from roadmap
fn parse_roadmap_tickets(content: &str) -> Result<HashMap<String, bool>> {
    let mut tickets = HashMap::new();
    let checkbox_re = regex::Regex::new(r"- \[([ x])\] (TICKET-PMAT-\d+)")?;

    for line in content.lines() {
        if let Some(captures) = checkbox_re.captures(line) {
            let checked = &captures[1] == "x";
            let ticket_id = captures[2].to_string();
            tickets.insert(ticket_id, checked);
        }
    }

    Ok(tickets)
}

/// Get ticket status from ticket file
fn get_ticket_status(ticket_path: &Path) -> Result<TicketStatus> {
    let content = fs::read_to_string(ticket_path)?;

    for line in content.lines().take(10) {
        if line.starts_with("**Status**:") {
            return Ok(match line {
                l if l.contains("GREEN") => TicketStatus::Green,
                l if l.contains("RED") => TicketStatus::Red,
                l if l.contains("YELLOW") => TicketStatus::Yellow,
                _ => TicketStatus::Unknown,
            });
        }
    }

    Ok(TicketStatus::Unknown)
}

/// Parse sprint information from roadmap
fn parse_sprint_info(content: &str, _tickets_dir: &Path) -> Result<Vec<SprintInfo>> {
    let mut sprints = Vec::new();

    // Simple parsing: find sprint headers
    for line in content.lines() {
        if line.starts_with("### Sprint ") {
            let name = line.trim_start_matches("### ").to_string();
            sprints.push(SprintInfo {
                name,
                total_tickets: 0,
                completed_tickets: 0,
                status: SprintStatus::NotStarted,
            });
        } else if line.starts_with("- [") {
            if let Some(sprint) = sprints.last_mut() {
                sprint.total_tickets += 1;
                if line.contains("[x]") {
                    sprint.completed_tickets += 1;
                }
            }
        }
    }

    // Calculate status
    for sprint in &mut sprints {
        sprint.status =
            if sprint.completed_tickets == sprint.total_tickets && sprint.total_tickets > 0 {
                SprintStatus::Complete
            } else if sprint.completed_tickets > 0 {
                SprintStatus::InProgress
            } else {
                SprintStatus::NotStarted
            };
    }

    Ok(sprints)
}

/// Extract sprint name for a ticket from roadmap context
///
/// # Complexity
/// - Time: O(n) where n is lines in roadmap
/// - Cyclomatic: 4
fn extract_sprint_for_ticket(roadmap_content: &str, ticket_id: &str) -> String {
    let lines: Vec<&str> = roadmap_content.lines().collect();
    let mut current_sprint = "Unknown Sprint".to_string();

    for line in lines.iter() {
        // Look for sprint headers like "### Sprint 21:"
        if line.starts_with("### Sprint") || line.starts_with("## Sprint") {
            if let Some(sprint_name) = line.split(':').next() {
                current_sprint = sprint_name.trim_start_matches('#').trim().to_string();
            }
        }

        // Check if this line contains our ticket
        if line.contains(ticket_id) {
            return current_sprint;
        }
    }

    current_sprint
}

/// Generate ticket file template (TICKET-PMAT-6012)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn generate_ticket_template(ticket_id: &str, sprint: &str, status: &str) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d");

    format!(
        r#"# {ticket_id}: [Title - TODO: Update from roadmap]

**Sprint:** {sprint}
**Priority:** [TBD - To be determined]
**Estimated Effort:** [TBD - To be estimated]
**Status**: {status}
**Created:** {today}

## Problem Statement

[TODO: Describe the problem this ticket solves]

## Solution

[TODO: Describe the proposed solution]

## Acceptance Criteria

- [ ] [TODO: Add acceptance criteria]

## Quality Metrics

- **CC:** [TBD]
- **Tests:** [TBD]
- **Coverage:** [TBD]

## Files Modified

- [TODO: List files to be modified]

## Related Tickets

- [TODO: Link to related tickets]

---

**Status:** {status}
**Delivered:** [TBD]
**Target Release:** [TBD]
"#
    )
}

/// Build checkbox markdown pattern for a ticket
fn build_checkbox_pattern(ticket_id: &str, checked: bool) -> String {
    if checked {
        format!("- [x] {ticket_id}")
    } else {
        format!("- [ ] {ticket_id}")
    }
}

/// Apply and report roadmap changes
fn apply_roadmap_changes(
    roadmap_path: &Path,
    updated_content: String,
    changes: &[(String, bool)],
    dry_run: bool,
) -> Result<()> {
    eprintln!("📝 Changes to apply:");
    for (ticket_id, checked) in changes {
        let action = if *checked { "✓" } else { "☐" };
        eprintln!("  {action} {ticket_id}");
    }

    if dry_run {
        eprintln!("\n🔍 Dry-run mode - no changes applied");
    } else {
        fs::write(roadmap_path, updated_content)?;
        eprintln!("\n✅ Updated {}", roadmap_path.display());
    }

    Ok(())
}
