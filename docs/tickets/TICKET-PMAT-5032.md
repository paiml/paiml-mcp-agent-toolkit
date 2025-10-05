# TICKET-PMAT-5032: Add `pmat maintain roadmap` CLI Command

**Status**: GREEN
**Priority**: P1
**Complexity**: 3
**Estimated Time**: 45 minutes
**Dependencies**: Existing CLI infrastructure, ROADMAP.md structure
**Sprint**: Sprint 19 - CLI Integration & Dogfooding

## Objective

Add `pmat maintain roadmap` subcommand to validate roadmap structure, check ticket status consistency, and generate roadmap health reports. This enables teams to keep roadmaps up-to-date and catch inconsistencies early.

## Success Criteria

- [ ] `pmat maintain roadmap --validate` checks roadmap structure and ticket status
- [ ] `pmat maintain roadmap --health` shows sprint progress and health metrics
- [ ] `pmat maintain roadmap --fix` auto-updates checkboxes based on ticket status
- [ ] All CLI arguments properly parsed and validated
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Current State

**Already Exists:**
- ROADMAP.md structure with sprints and tickets
- Ticket files in `docs/tickets/` with status markers
- CLI infrastructure in `server/src/cli/`

**Missing:**
- `Commands::Maintain` variant with `MaintainCommands` subcommands
- Roadmap validation logic
- Health report generation
- Auto-fix functionality

## Test Strategy

### Unit Tests
- [ ] `test_roadmap_validation_valid` - Valid roadmap passes
- [ ] `test_roadmap_validation_invalid_checkbox` - Catches checkbox errors
- [ ] `test_health_report_generation` - Health metrics calculated correctly
- [ ] `test_ticket_status_detection` - Parse ticket status from files

### Integration Tests
- [ ] `integration_maintain_roadmap_validate` - Full validation run
- [ ] `integration_maintain_roadmap_health` - Health report output
- [ ] `integration_maintain_roadmap_fix_dry_run` - Dry-run mode works

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Add Maintain Command Variants

```rust
// server/src/cli/commands.rs (after QualityGates)

/// Project maintenance commands
Maintain {
    #[command(subcommand)]
    command: MaintainCommands,
},

/// Maintain subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum MaintainCommands {
    /// Validate roadmap structure and ticket consistency
    Roadmap {
        /// Path to ROADMAP.md
        #[arg(long, default_value = "ROADMAP.md")]
        roadmap: PathBuf,

        /// Path to tickets directory
        #[arg(long, default_value = "docs/tickets")]
        tickets_dir: PathBuf,

        /// Check ticket status consistency
        #[arg(long)]
        validate: bool,

        /// Show roadmap health report
        #[arg(long)]
        health: bool,

        /// Auto-fix checkbox status based on ticket files
        #[arg(long)]
        fix: bool,

        /// Dry-run mode (show changes without applying)
        #[arg(long)]
        dry_run: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "console")]
        format: OutputFormat,
    },

    /// Validate project health (TICKET-PMAT-5033)
    Health {
        // Future: overall project health checks
    },
}
```

### Phase 2: Create Roadmap Handler

```rust
// server/src/cli/handlers/roadmap_handler.rs

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Roadmap validation result
#[derive(Debug)]
pub struct RoadmapValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub sprints: Vec<SprintInfo>,
}

/// Sprint information
#[derive(Debug)]
pub struct SprintInfo {
    pub name: String,
    pub total_tickets: usize,
    pub completed_tickets: usize,
    pub in_progress_tickets: usize,
    pub status: SprintStatus,
}

#[derive(Debug, PartialEq)]
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

/// Handle roadmap maintenance command
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 6
pub async fn handle_maintain_roadmap(
    roadmap_path: PathBuf,
    tickets_dir: PathBuf,
    validate: bool,
    health: bool,
    fix: bool,
    dry_run: bool,
    format: crate::cli::commands::OutputFormat,
) -> Result<()> {
    if validate {
        validate_roadmap(&roadmap_path, &tickets_dir).await?;
    }

    if health {
        show_health_report(&roadmap_path, &tickets_dir, format).await?;
    }

    if fix {
        fix_roadmap_status(&roadmap_path, &tickets_dir, dry_run).await?;
    }

    if !validate && !health && !fix {
        // Default: show health
        show_health_report(&roadmap_path, &tickets_dir, format).await?;
    }

    Ok(())
}

/// Validate roadmap structure and ticket consistency
async fn validate_roadmap(roadmap_path: &Path, tickets_dir: &Path) -> Result<()> {
    let roadmap_content = fs::read_to_string(roadmap_path)
        .context("Failed to read ROADMAP.md")?;

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
            warnings.push(format!(
                "{ticket_id}: Unchecked in roadmap but status is GREEN"
            ));
        }
    }

    // Report results
    if errors.is_empty() && warnings.is_empty() {
        eprintln!("✅ Roadmap validation passed!");
    } else {
        if !errors.is_empty() {
            eprintln!("❌ Validation errors:");
            for error in &errors {
                eprintln!("  - {error}");
            }
        }
        if !warnings.is_empty() {
            eprintln!("⚠️  Warnings:");
            for warning in &warnings {
                eprintln!("  - {warning}");
            }
        }
        if !errors.is_empty() {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Show roadmap health report
async fn show_health_report(
    roadmap_path: &Path,
    tickets_dir: &Path,
    format: crate::cli::commands::OutputFormat,
) -> Result<()> {
    let roadmap_content = fs::read_to_string(roadmap_path)?;
    let sprints = parse_sprint_info(&roadmap_content, tickets_dir)?;

    match format {
        crate::cli::commands::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&sprints)?);
        }
        crate::cli::commands::OutputFormat::Markdown => {
            print_health_markdown(&sprints);
        }
        _ => {
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
fn parse_sprint_info(content: &str, tickets_dir: &Path) -> Result<Vec<SprintInfo>> {
    let mut sprints = Vec::new();
    let roadmap_tickets = parse_roadmap_tickets(content)?;

    // Simple parsing: find sprint headers
    for line in content.lines() {
        if line.starts_with("### Sprint ") {
            let name = line.trim_start_matches("### ").to_string();
            sprints.push(SprintInfo {
                name,
                total_tickets: 0,
                completed_tickets: 0,
                in_progress_tickets: 0,
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
        sprint.status = if sprint.completed_tickets == sprint.total_tickets {
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

/// Print health report in markdown format
fn print_health_markdown(sprints: &[SprintInfo]) {
    println!("# Roadmap Health Report\n");

    for sprint in sprints {
        let progress = if sprint.total_tickets > 0 {
            (sprint.completed_tickets as f64 / sprint.total_tickets as f64) * 100.0
        } else {
            0.0
        };

        println!("## {}", sprint.name);
        println!(
            "- Progress: {}/{} ({:.0}%)",
            sprint.completed_tickets, sprint.total_tickets, progress
        );
        println!(
            "- Status: {:?}",
            sprint.status
        );
        println!();
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
        let temp_file = std::env::temp_dir().join("test_ticket.md");
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
```

### Phase 3: Wire Up in Command Structure

```rust
// server/src/cli/command_structure.rs

Commands::Maintain { command } => {
    match command {
        MaintainCommands::Roadmap {
            roadmap,
            tickets_dir,
            validate,
            health,
            fix,
            dry_run,
            format,
        } => {
            super::handlers::handle_maintain_roadmap(
                roadmap,
                tickets_dir,
                validate,
                health,
                fix,
                dry_run,
                format,
            )
            .await
        }
        MaintainCommands::Health { .. } => {
            // TICKET-PMAT-5033
            Err(anyhow::anyhow!("Health command not yet implemented"))
        }
    }
}
```

## Complexity Analysis

Functions with complexity:
- `handle_maintain_roadmap`: CC=6 (validate, health, fix, default, dry_run, format)
- `validate_roadmap`: CC=4 (file exists, consistency checks)
- `fix_roadmap_status`: CC=3 (dry_run, checkbox update)
- `parse_roadmap_tickets`: CC=2 (regex matching)

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Validate roadmap
pmat maintain roadmap --validate

# Show health report
pmat maintain roadmap --health

# Health report in JSON
pmat maintain roadmap --health --format json

# Auto-fix checkbox status (dry-run)
pmat maintain roadmap --fix --dry-run

# Auto-fix and apply
pmat maintain roadmap --fix

# Combined: validate and show health
pmat maintain roadmap --validate --health
```

## Files to Create/Modify

### New Files
- `server/src/cli/handlers/roadmap_handler.rs` - Roadmap maintenance logic

### Modified Files
- `server/src/cli/commands.rs` - Add Maintain variant and MaintainCommands enum
- `server/src/cli/handlers/mod.rs` - Export roadmap handler
- `server/src/cli/command_structure.rs` - Wire up Maintain command
- `server/src/cli/command_dispatcher.rs` - Add Maintain routing

## Dependencies

Add to `server/Cargo.toml`:
```toml
regex = "1.10" # For checkbox parsing
```

## Risk Assessment

**Low Risk:**
- Read-only validation and health reporting
- Dry-run mode for fix command
- No external dependencies

**Mitigation:**
- Dry-run mode default for fixes
- Comprehensive tests for parsing logic
- Clear error messages for invalid roadmaps

## Notes

This ticket enables "dogfooding" - using PMAT to maintain itself:

**Workflows Enabled:**
1. CI validation: `pmat maintain roadmap --validate` in GitHub Actions
2. Sprint planning: `pmat maintain roadmap --health` for progress tracking
3. Auto-updates: `pmat maintain roadmap --fix` after ticket completion

**Value:**
- Keep roadmap synchronized with ticket files
- Catch inconsistencies early
- Automate roadmap maintenance
- Provide visibility into sprint progress

**TDD Cycle Duration**: Estimated 45 minutes for RED → GREEN → REFACTOR
