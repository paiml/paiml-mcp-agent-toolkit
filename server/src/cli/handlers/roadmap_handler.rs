//! Roadmap maintenance and validation handlers
//!
//! This module provides functionality for maintaining project roadmaps,
//! including validation, health reporting, and auto-fixing checkbox status.

use anyhow::Result;
use crate::cli::OutputFormat;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Roadmap validation result
#[derive(Debug)]
pub struct RoadmapValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Sprint information
#[derive(Debug, Serialize)]
pub struct SprintInfo {
    pub name: String,
    pub total_tickets: usize,
    pub completed_tickets: usize,
    pub status: SprintStatus,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum SprintStatus {
    NotStarted,
    InProgress,
    Complete,
}

/// Ticket status from ticket file
#[derive(Debug, PartialEq)]
pub enum TicketStatus {
    Red,
    Green,
    Yellow,
    Unknown,
}

/// Ticket generation result (TICKET-PMAT-6021)
#[derive(Debug, Serialize)]
pub struct TicketGenerationResult {
    pub generated: Vec<String>,
    pub skipped: Vec<String>,
}

/// Handle roadmap maintenance command (TICKET-PMAT-6012)
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 7
pub async fn handle_maintain_roadmap(
    roadmap_path: PathBuf,
    tickets_dir: PathBuf,
    validate: bool,
    health: bool,
    fix: bool,
    generate_tickets: bool,
    dry_run: bool,
    format: OutputFormat,
) -> Result<()> {
    if validate {
        validate_roadmap(&roadmap_path, &tickets_dir).await?;
    }

    if health {
        show_health_report(&roadmap_path, &tickets_dir, &format).await?;
    }

    if fix {
        fix_roadmap_status(&roadmap_path, &tickets_dir, dry_run).await?;
    }

    if generate_tickets {
        generate_missing_ticket_files(&roadmap_path, &tickets_dir, dry_run).await?;
    }

    if !validate && !health && !fix && !generate_tickets {
        // Default: show health
        show_health_report(&roadmap_path, &tickets_dir, &format).await?;
    }

    Ok(())
}

/// Validate roadmap structure and ticket consistency (internal, reusable)
/// (TICKET-PMAT-6019)
pub async fn validate_roadmap_internal(
    roadmap_path: &Path,
    tickets_dir: &Path,
) -> Result<RoadmapValidation> {
    let roadmap_content = fs::read_to_string(roadmap_path).map_err(|_| {
        let error = crate::cli::error_context::roadmap_not_found(roadmap_path);
        anyhow::anyhow!(error.format_detailed())
    })?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Parse roadmap tickets
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    // Check each ticket file
    for (ticket_id, checkbox_status) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if !ticket_path.exists() {
            errors.push(format!("Missing ticket file: {ticket_id}.md"));
            continue;
        }

        let ticket_status = get_ticket_status(&ticket_path)?;

        // Check consistency
        if *checkbox_status && ticket_status != TicketStatus::Green {
            warnings.push(format!(
                "{ticket_id}: Checked in roadmap but status is {:?}",
                ticket_status
            ));
        }

        if !checkbox_status && ticket_status == TicketStatus::Green {
            warnings.push(format!("{ticket_id}: Unchecked in roadmap but status is GREEN"));
        }
    }

    Ok(RoadmapValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

/// Validate roadmap structure and ticket consistency (CLI wrapper)
async fn validate_roadmap(roadmap_path: &Path, tickets_dir: &Path) -> Result<()> {
    let validation = validate_roadmap_internal(roadmap_path, tickets_dir).await?;

    // Report results
    if validation.valid && validation.warnings.is_empty() {
        eprintln!("✅ Roadmap validation passed!");
    } else {
        if !validation.errors.is_empty() {
            eprintln!("❌ Validation errors:");
            for error in &validation.errors {
                eprintln!("  - {error}");
            }
        }
        if !validation.warnings.is_empty() {
            eprintln!("⚠️  Warnings:");
            for warning in &validation.warnings {
                eprintln!("  - {warning}");
            }
        }
        if !validation.valid {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Show roadmap health report
async fn show_health_report(
    roadmap_path: &Path,
    tickets_dir: &Path,
    format: &OutputFormat,
) -> Result<()> {
    let roadmap_content = fs::read_to_string(roadmap_path).map_err(|_| {
        let error = crate::cli::error_context::roadmap_not_found(roadmap_path);
        anyhow::anyhow!(error.format_detailed())
    })?;
    let sprints = parse_sprint_info(&roadmap_content, tickets_dir)?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&sprints)?);
        }
        OutputFormat::Yaml => {
            print_health_yaml(&sprints);
        }
        OutputFormat::Table => {
            print_health_console(&sprints);
        }
    }

    Ok(())
}

/// Fix roadmap checkbox status based on ticket files
async fn fix_roadmap_status(
    roadmap_path: &Path,
    tickets_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    let roadmap_content = fs::read_to_string(roadmap_path)?;
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    let mut updated_content = roadmap_content.clone();
    let mut changes = Vec::new();

    for (ticket_id, checkbox_status) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if !ticket_path.exists() {
            continue;
        }

        let ticket_status = get_ticket_status(&ticket_path)?;
        let should_be_checked = ticket_status == TicketStatus::Green;

        if *checkbox_status != should_be_checked {
            changes.push((ticket_id.clone(), should_be_checked));

            // Replace checkbox
            let old_pattern = if *checkbox_status {
                format!("- [x] {ticket_id}")
            } else {
                format!("- [ ] {ticket_id}")
            };

            let new_pattern = if should_be_checked {
                format!("- [x] {ticket_id}")
            } else {
                format!("- [ ] {ticket_id}")
            };

            updated_content = updated_content.replace(&old_pattern, &new_pattern);
        }
    }

    if changes.is_empty() {
        eprintln!("✅ Roadmap is already up-to-date!");
        return Ok(());
    }

    eprintln!("📝 Changes to apply:");
    for (ticket_id, checked) in &changes {
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

/// Generate missing ticket files from roadmap (internal, reusable)
/// (TICKET-PMAT-6021)
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 6
pub async fn generate_tickets_internal(
    roadmap_path: &Path,
    tickets_dir: &Path,
    dry_run: bool,
) -> Result<TicketGenerationResult> {
    let roadmap_content = fs::read_to_string(roadmap_path)?;
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    let mut generated = Vec::new();
    let mut skipped = Vec::new();

    for (ticket_id, checked) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if ticket_path.exists() {
            skipped.push(ticket_id.clone());
            continue;
        }

        generated.push(ticket_id.clone());

        // Extract sprint context from roadmap
        let sprint = extract_sprint_for_ticket(&roadmap_content, ticket_id);
        let status = if *checked { "GREEN ✅" } else { "PLANNED 📋" };

        let template = generate_ticket_template(ticket_id, &sprint, status);

        if !dry_run {
            fs::create_dir_all(tickets_dir)?;
            fs::write(&ticket_path, template)?;
        }
    }

    Ok(TicketGenerationResult { generated, skipped })
}

/// Generate missing ticket files from roadmap (CLI wrapper)
/// (TICKET-PMAT-6012)
async fn generate_missing_ticket_files(
    roadmap_path: &Path,
    tickets_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    eprintln!("📝 Checking for missing ticket files...\n");

    let result = generate_tickets_internal(roadmap_path, tickets_dir, dry_run).await?;

    // Print each generated ticket
    for ticket_id in &result.generated {
        if dry_run {
            let roadmap_content = fs::read_to_string(roadmap_path)?;
            let sprint = extract_sprint_for_ticket(&roadmap_content, ticket_id);
            eprintln!("Would create: {ticket_id}.md (Sprint: {sprint})");
        } else {
            eprintln!("Created: {ticket_id}.md");
        }
    }

    eprintln!();
    if result.generated.is_empty() {
        eprintln!("✅ No missing ticket files");
    } else {
        eprintln!("✅ Generated {} ticket file(s)", result.generated.len());
        if dry_run {
            eprintln!("🔍 Dry-run mode - no files created");
        }
    }

    if !result.skipped.is_empty() {
        eprintln!("⏭️  Skipped {} existing ticket(s)", result.skipped.len());
    }

    Ok(())
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

    format!(r#"# {ticket_id}: [Title - TODO: Update from roadmap]

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
"#)
}

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
        sprint.status = if sprint.completed_tickets == sprint.total_tickets
            && sprint.total_tickets > 0
        {
            SprintStatus::Complete
        } else if sprint.completed_tickets > 0 {
            SprintStatus::InProgress
        } else {
            SprintStatus::NotStarted
        };
    }

    Ok(sprints)
}

/// Print health report in console format
fn print_health_console(sprints: &[SprintInfo]) {
    eprintln!("📊 Roadmap Health Report\n");

    for sprint in sprints {
        let progress = if sprint.total_tickets > 0 {
            (sprint.completed_tickets as f64 / sprint.total_tickets as f64) * 100.0
        } else {
            0.0
        };

        let status_emoji = match sprint.status {
            SprintStatus::Complete => "✅",
            SprintStatus::InProgress => "🔄",
            SprintStatus::NotStarted => "⏳",
        };

        eprintln!("{status_emoji} {}", sprint.name);
        eprintln!(
            "   Progress: {}/{} ({:.0}%)",
            sprint.completed_tickets, sprint.total_tickets, progress
        );
        eprintln!();
    }
}

/// Print health report in YAML format
fn print_health_yaml(sprints: &[SprintInfo]) {
    println!("roadmap_health:");
    for sprint in sprints {
        let progress = if sprint.total_tickets > 0 {
            (sprint.completed_tickets as f64 / sprint.total_tickets as f64) * 100.0
        } else {
            0.0
        };

        println!("  - name: {}", sprint.name);
        println!("    total_tickets: {}", sprint.total_tickets);
        println!("    completed_tickets: {}", sprint.completed_tickets);
        println!("    progress: {:.0}%", progress);
        println!("    status: {:?}", sprint.status);
    }
}

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

            prop_assert!(progress >= 0.0 && progress <= 100.0);
        }
    }
}
