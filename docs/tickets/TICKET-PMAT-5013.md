# TICKET-PMAT-5013: Auto-update Hooks (Post-commit)

**Status**: GREEN
**Priority**: P0
**Complexity**: 6
**Estimated Time**: 3 hours
**Dependencies**: TICKET-PMAT-5010, TICKET-PMAT-5011, TICKET-PMAT-5012
**Sprint**: Sprint 17 - Maintenance Engine

## Objective

Implement post-commit hooks that automatically update the roadmap when ticket status changes. This keeps the roadmap synchronized with ticket files automatically, reducing manual maintenance and ensuring the roadmap always reflects current project state.

## Success Criteria

- [ ] Detect ticket status changes in commits
- [ ] Update roadmap with commit references automatically
- [ ] Mark tickets as complete in roadmap when status is GREEN/COMPLETE
- [ ] Generate post-commit hook script
- [ ] Install hook alongside pre-commit hooks
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_detect_ticket_in_commit_message` - Parse ticket IDs from commit messages
- [ ] `test_update_roadmap_ticket_status` - Mark ticket complete in roadmap
- [ ] `test_add_commit_reference` - Add commit hash to ticket entry
- [ ] `test_generate_post_commit_hook` - Generate hook script
- [ ] `test_parse_git_log` - Extract commit info

### Property Tests
- [ ] Property: Updated roadmap is still valid
- [ ] Property: Commit references are valid git hashes
- [ ] Property: Ticket updates are idempotent

### Integration Tests
- [ ] `integration_update_roadmap_from_commit` - Full workflow test
- [ ] `integration_hook_execution` - Test hook runs correctly

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Git Integration

```rust
// server/src/maintenance/git.rs

use std::path::Path;
use std::process::Command;

/// Git commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Changed files
    pub files: Vec<String>,
}

/// Extract ticket IDs from commit message
///
/// # Example
/// "feat: TICKET-PMAT-5013 - Auto-update hooks (GREEN)"
/// Returns: ["TICKET-PMAT-5013"]
///
/// # Complexity
/// - Time: O(n) where n is message length
/// - Cyclomatic: 3
pub fn extract_ticket_ids(commit_message: &str) -> Vec<String> {
    use regex::Regex;

    let re = Regex::new(r"TICKET-PMAT-\d{4}").unwrap();
    re.find_iter(commit_message)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Get current commit info
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
pub fn get_current_commit() -> Result<CommitInfo, std::io::Error> {
    let hash_output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()?;

    let message_output = Command::new("git")
        .args(&["log", "-1", "--pretty=%B"])
        .output()?;

    let files_output = Command::new("git")
        .args(&["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .output()?;

    Ok(CommitInfo {
        hash: String::from_utf8_lossy(&hash_output.stdout).trim().to_string(),
        message: String::from_utf8_lossy(&message_output.stdout).trim().to_string(),
        files: String::from_utf8_lossy(&files_output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// Check if ticket file was updated in commit
///
/// # Complexity
/// - Time: O(n) where n is number of files
/// - Cyclomatic: 2
pub fn ticket_file_updated(commit: &CommitInfo, ticket_id: &str) -> bool {
    let ticket_file = format!("docs/tickets/{}.md", ticket_id);
    commit.files.iter().any(|f| f.contains(&ticket_file))
}
```

### Phase 2: Roadmap Updater

```rust
// server/src/maintenance/updater.rs

use super::roadmap::{Roadmap, Ticket};
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
) -> Result<bool, super::roadmap::RoadmapError> {
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
    }

    Ok(updated)
}

/// Write updated roadmap back to file
///
/// # Complexity
/// - Time: O(n) where n is roadmap size
/// - Cyclomatic: 4
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

    output.push_str(&format!("# PMAT Agent System Roadmap\n\n"));
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

        output.push_str(&format!("**Focus:** {}\n", sprint.focus));

        // Tickets
        for ticket in &sprint.tickets {
            let checkbox = if ticket.completed { "[x]" } else { "[ ]" };
            let commit_ref = if let Some(ref commit) = ticket.commit {
                format!(" (commit: {})", &commit[..7])
            } else {
                String::new()
            };

            output.push_str(&format!(
                "- {} {}: {}{}\n",
                checkbox, ticket.id, ticket.description, commit_ref
            ));
        }

        output.push_str("\n");

        // Quality gates
        if !sprint.quality_gates.is_empty() {
            output.push_str("**Quality Gates:**\n");
            for gate in &sprint.quality_gates {
                output.push_str(&format!("- {}\n", gate));
            }
            output.push_str("\n");
        }
    }

    output
}
```

### Phase 3: Post-commit Hook

```rust
// server/src/maintenance/hooks.rs (extend existing)

/// Generate post-commit hook script
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
pub fn generate_post_commit_hook() -> String {
    r#"#!/bin/bash
# Post-commit hook for roadmap updates
# Generated by PMAT Maintenance Engine

# Only run if this is a ticket commit
if ! git log -1 --pretty=%B | grep -q "TICKET-PMAT-"; then
    exit 0
fi

# Run roadmap updater
if command -v pmat &> /dev/null; then
    pmat maintain update-roadmap --from-commit HEAD 2>/dev/null || true
fi

exit 0
"#.to_string()
}

/// Install post-commit hook
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
pub fn install_post_commit_hook(project_dir: &Path) -> Result<(), std::io::Error> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let hook_path = project_dir.join(".git/hooks/post-commit");
    let script = generate_post_commit_hook();

    fs::write(&hook_path, script)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}
```

### Phase 4: CLI Integration

```rust
// Main update function called by CLI or hook

/// Update roadmap from current commit
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
                use crate::maintenance::ticket::TicketStatus;
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
        println!("✓ Updated roadmap with commit {}", &commit.hash[..7]);
    }

    Ok(())
}
```

### Phase 5: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ticket_ids() {
        let message = "feat: TICKET-PMAT-5013 - Auto-update hooks (GREEN)";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "TICKET-PMAT-5013");
    }

    #[test]
    fn test_extract_multiple_ticket_ids() {
        let message = "fix: TICKET-PMAT-5001 and TICKET-PMAT-5002 issues";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_extract_no_ticket_ids() {
        let message = "chore: Update documentation";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_ticket_file_updated() {
        let commit = CommitInfo {
            hash: "abc123".into(),
            message: "test".into(),
            files: vec![
                "docs/tickets/TICKET-PMAT-5013.md".into(),
                "server/src/main.rs".into(),
            ],
        };

        assert!(ticket_file_updated(&commit, "TICKET-PMAT-5013"));
        assert!(!ticket_file_updated(&commit, "TICKET-PMAT-9999"));
    }

    #[test]
    fn test_update_roadmap_ticket() {
        use super::super::roadmap::{Roadmap, Sprint, Ticket, SprintStatus};

        let mut roadmap = Roadmap {
            version: "v2.139.0".into(),
            sprints: vec![
                Sprint {
                    number: 17,
                    name: "Test".into(),
                    focus: "".into(),
                    status: SprintStatus::InProgress,
                    duration: "2 days".into(),
                    tickets: vec![
                        Ticket {
                            id: "TICKET-PMAT-5013".into(),
                            description: "Test".into(),
                            completed: false,
                            commit: None,
                        },
                    ],
                    quality_gates: vec![],
                },
            ],
        };

        let updated = update_roadmap_ticket(&mut roadmap, "TICKET-PMAT-5013", "abc123").unwrap();

        assert!(updated);
        assert!(roadmap.sprints[0].tickets[0].completed);
        assert_eq!(roadmap.sprints[0].tickets[0].commit, Some("abc123".into()));
    }

    #[test]
    fn test_format_roadmap_markdown() {
        use super::super::roadmap::{Roadmap, Sprint, Ticket, SprintStatus};

        let roadmap = Roadmap {
            version: "v2.139.0".into(),
            sprints: vec![
                Sprint {
                    number: 17,
                    name: "Test Sprint".into(),
                    focus: "Testing".into(),
                    status: SprintStatus::InProgress,
                    duration: "2 days".into(),
                    tickets: vec![
                        Ticket {
                            id: "TICKET-PMAT-5013".into(),
                            description: "Auto-update hooks".into(),
                            completed: true,
                            commit: Some("abc1234".into()),
                        },
                    ],
                    quality_gates: vec!["Coverage >80%".into()],
                },
            ],
        };

        let markdown = format_roadmap_markdown(&roadmap);

        assert!(markdown.contains("# PMAT Agent System Roadmap"));
        assert!(markdown.contains("Sprint 17"));
        assert!(markdown.contains("[x] TICKET-PMAT-5013"));
        assert!(markdown.contains("(commit: abc1234)"));
        assert!(markdown.contains("Coverage >80%"));
    }

    #[test]
    fn test_generate_post_commit_hook() {
        let hook = generate_post_commit_hook();

        assert!(hook.starts_with("#!/bin/bash"));
        assert!(hook.contains("TICKET-PMAT-"));
        assert!(hook.contains("pmat maintain update-roadmap"));
    }
}
```

## Complexity Analysis

Functions with complexity:
- `extract_ticket_ids`: CC=3
- `get_current_commit`: CC=3
- `ticket_file_updated`: CC=2
- `update_roadmap_ticket`: CC=5
- `write_roadmap`: CC=4
- `format_roadmap_markdown`: CC=5
- `update_roadmap_from_commit`: CC=7
- `generate_post_commit_hook`: CC=1

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run tests
cargo test --lib maintenance::git
cargo test --lib maintenance::updater

# Manual test: Create a commit and check roadmap update
git commit -m "feat: TICKET-PMAT-5013 - Auto-update hooks (GREEN)"
cat ROADMAP.md | grep TICKET-PMAT-5013
```

## Files to Create/Modify

### New Files
- `server/src/maintenance/git.rs` - Git integration utilities
- `server/src/maintenance/updater.rs` - Roadmap update logic

### Modified Files
- `server/src/maintenance/mod.rs` - Add git and updater modules
- `server/src/maintenance/hooks.rs` - Add post-commit hook generation

### Dependencies
- `regex` crate for ticket ID extraction (likely already available)

## Risk Assessment

**Low Risk:**
- Read-only git operations
- Roadmap updates are atomic file writes
- Hook runs after commit (non-blocking)

**Mitigation:**
- Backup roadmap before writing
- Validate roadmap after update
- Silent failure in hook (doesn't block commits)

## Notes

This ticket completes the automatic maintenance loop:
1. Developer updates ticket status to GREEN/COMPLETE
2. Commits changes with ticket ID in message
3. Post-commit hook detects ticket update
4. Roadmap automatically updated with commit reference

**No more manual roadmap updates needed!**

The system becomes fully self-maintaining - Rule B enforcement is now automatic.

**TDD Cycle Duration**: Estimated 2-3 hours for RED → GREEN → REFACTOR
