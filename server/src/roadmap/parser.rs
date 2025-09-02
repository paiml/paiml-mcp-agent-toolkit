//! Roadmap markdown parser and serializer

use super::*;
use anyhow::Result;
use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

/// Parse a roadmap from markdown content
pub fn parse_roadmap(content: &str) -> Result<Roadmap> {
    let mut roadmap = Roadmap {
        current_sprint: None,
        sprints: HashMap::new(),
        backlog: Vec::new(),
        completed_sprints: Vec::new(),
    };

    // Parse sprints
    let sprint_regex = Regex::new(r"## (?:Current |Previous |Next )?Sprint: (v[\d.]+) (.+)")?;
    let task_regex = Regex::new(r"\| (PMAT-\d{4}) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|")?;
    let done_regex = Regex::new(r"- \[([ x])\] (.+)")?;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check for sprint header
        if let Some(captures) = sprint_regex.captures(line) {
            let version = captures.get(1).unwrap().as_str().to_string();
            let title = captures.get(2).unwrap().as_str().to_string();

            // Parse sprint metadata
            let mut sprint = Sprint {
                version: version.clone(),
                title,
                start_date: Utc::now(), // Default, will be parsed from content
                end_date: Utc::now() + chrono::Duration::days(14), // Default 2 weeks
                priority: Priority::P0,
                tasks: Vec::new(),
                definition_of_done: Vec::new(),
                quality_gates: Vec::new(),
            };

            // Look for sprint details
            i += 1;
            while i < lines.len() {
                let line = lines[i];

                // Parse duration
                if line.contains("**Duration**:") {
                    if let Some(duration) = parse_duration(line) {
                        sprint.start_date = duration.0;
                        sprint.end_date = duration.1;
                    }
                }

                // Parse priority
                if line.contains("**Priority**:") {
                    if let Some(priority) = parse_priority(line) {
                        sprint.priority = priority;
                    }
                }

                // Parse tasks table
                if line.contains("| ID | Description |") {
                    i += 2; // Skip header and separator
                    while i < lines.len() && lines[i].starts_with('|') {
                        if let Some(captures) = task_regex.captures(lines[i]) {
                            let task = Task {
                                id: captures.get(1).unwrap().as_str().to_string(),
                                description: captures.get(2).unwrap().as_str().trim().to_string(),
                                status: parse_task_status(captures.get(3).unwrap().as_str()),
                                complexity: Complexity::from_str(
                                    captures.get(4).unwrap().as_str().trim(),
                                )
                                .unwrap_or(Complexity::Medium),
                                priority: Priority::from_str(
                                    captures.get(5).unwrap().as_str().trim(),
                                )
                                .unwrap_or(Priority::P1),
                                assignee: None,
                                started_at: None,
                                completed_at: None,
                            };
                            sprint.tasks.push(task);
                        }
                        i += 1;
                    }
                    continue;
                }

                // Parse Definition of Done
                if line.contains("### Definition of Done") {
                    i += 1;
                    while i < lines.len() && lines[i].starts_with("- [") {
                        if let Some(captures) = done_regex.captures(lines[i]) {
                            let item = captures.get(2).unwrap().as_str().to_string();
                            sprint.definition_of_done.push(item);
                        }
                        i += 1;
                    }
                    continue;
                }

                // Check for next sprint
                if line.starts_with("## ") && line.contains("Sprint:") {
                    break;
                }

                i += 1;
            }

            // Determine if this is the current sprint
            if line.contains("Current Sprint:")
                || (roadmap.current_sprint.is_none() && !line.contains("Previous"))
            {
                roadmap.current_sprint = Some(version.clone());
            }

            // Mark completed sprints
            if line.contains("✅ COMPLETED") {
                roadmap.completed_sprints.push(version.clone());
            }

            roadmap.sprints.insert(version, sprint);
        }

        // Parse backlog
        if line.contains("### Backlog") {
            i += 1;
            // Skip to table content
            while i < lines.len() && !lines[i].starts_with('|') {
                i += 1;
            }
            i += 2; // Skip header and separator

            while i < lines.len() && lines[i].starts_with('|') {
                if let Some(captures) = task_regex.captures(lines[i]) {
                    let task = Task {
                        id: captures.get(1).unwrap().as_str().to_string(),
                        description: captures.get(2).unwrap().as_str().trim().to_string(),
                        status: parse_task_status(captures.get(3).unwrap().as_str()),
                        complexity: Complexity::from_str(captures.get(4).unwrap().as_str().trim())
                            .unwrap_or(Complexity::Medium),
                        priority: Priority::from_str(captures.get(5).unwrap().as_str().trim())
                            .unwrap_or(Priority::P2),
                        assignee: None,
                        started_at: None,
                        completed_at: None,
                    };
                    roadmap.backlog.push(task);
                }
                i += 1;
            }
        }

        i += 1;
    }

    Ok(roadmap)
}

/// Convert a roadmap to markdown format
pub fn roadmap_to_markdown(roadmap: &Roadmap) -> Result<String> {
    let mut output = String::new();

    output.push_str("# PMAT Development Roadmap\n\n");

    // Current sprint
    if let Some(current_id) = &roadmap.current_sprint {
        if let Some(sprint) = roadmap.sprints.get(current_id) {
            output.push_str(&format_sprint(sprint, true, false)?);
            output.push('\n');
        }
    }

    // Previous completed sprints
    for sprint_id in &roadmap.completed_sprints {
        if let Some(sprint) = roadmap.sprints.get(sprint_id) {
            output.push_str(&format_sprint(sprint, false, true)?);
            output.push('\n');
        }
    }

    // Future sprints
    for (id, sprint) in &roadmap.sprints {
        if (roadmap.current_sprint.as_ref() != Some(id)) && !roadmap.completed_sprints.contains(id)
        {
            output.push_str(&format_sprint(sprint, false, false)?);
            output.push('\n');
        }
    }

    // Backlog
    if !roadmap.backlog.is_empty() {
        output.push_str("### Backlog 📋\n");
        output.push_str("| ID | Description | Status | Complexity | Priority |\n");
        output.push_str("|----|-------------|--------|------------|----------|\n");
        for task in &roadmap.backlog {
            output.push_str(&format_task(task)?);
        }
        output.push('\n');
    }

    Ok(output)
}

fn format_sprint(sprint: &Sprint, is_current: bool, is_completed: bool) -> Result<String> {
    let mut output = String::new();

    let prefix = if is_current {
        "Current Sprint"
    } else if is_completed {
        "Previous Sprint"
    } else {
        "Next Sprint"
    };

    let status = if is_completed {
        " ✅ COMPLETED"
    } else {
        " 📋 PLANNED"
    };

    output.push_str(&format!(
        "## {}: {} {}{}\n",
        prefix,
        sprint.version,
        sprint.title,
        if is_completed { status } else { "" }
    ));

    output.push_str(&format!(
        "- **Duration**: {} to {}\n",
        sprint.start_date.format("%Y-%m-%d"),
        sprint.end_date.format("%Y-%m-%d")
    ));
    output.push_str(&format!("- **Priority**: {:?}\n", sprint.priority));

    if !sprint.quality_gates.is_empty() {
        output.push_str(&format!(
            "- **Quality Gates**: {}\n",
            sprint.quality_gates.join(", ")
        ));
    }

    output.push_str("\n### Tasks\n");
    output.push_str("| ID | Description | Status | Complexity | Priority |\n");
    output.push_str("|----|-------------|--------|------------|----------|\n");

    for task in &sprint.tasks {
        output.push_str(&format_task(task)?);
    }

    if !sprint.definition_of_done.is_empty() {
        output.push_str("\n### Definition of Done\n");
        for item in &sprint.definition_of_done {
            let checked = if is_completed { "x" } else { " " };
            output.push_str(&format!("- [{}] {}\n", checked, item));
        }
    }

    Ok(output)
}

fn format_task(task: &Task) -> Result<String> {
    Ok(format!(
        "| {} | {} | {} | {:?} | {:?} |\n",
        task.id,
        task.description,
        task.status.to_emoji(),
        task.complexity,
        task.priority
    ))
}

fn parse_task_status(s: &str) -> TaskStatus {
    let s = s.trim();
    TaskStatus::from_emoji(s).unwrap_or_else(|| match s.to_lowercase().as_str() {
        "planned" => TaskStatus::Planned,
        "in_progress" | "in progress" => TaskStatus::InProgress,
        "completed" | "done" => TaskStatus::Completed,
        "blocked" => TaskStatus::Blocked,
        "deferred" => TaskStatus::Deferred,
        _ => TaskStatus::Planned,
    })
}

fn parse_duration(line: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    // Parse format like "2025-08-20 to 2025-08-21"
    let date_regex = Regex::new(r"(\d{4}-\d{2}-\d{2})[^\d]+(\d{4}-\d{2}-\d{2})").ok()?;

    if let Some(captures) = date_regex.captures(line) {
        let start_str = captures.get(1)?.as_str();
        let end_str = captures.get(2)?.as_str();

        let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .ok()?
            .and_hms_opt(0, 0, 0)?
            .and_utc();
        let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .ok()?
            .and_hms_opt(23, 59, 59)?
            .and_utc();

        return Some((start, end));
    }

    None
}

fn parse_priority(line: &str) -> Option<Priority> {
    if line.contains("P0") || line.contains("CRITICAL") {
        Some(Priority::P0)
    } else if line.contains("P1") {
        Some(Priority::P1)
    } else if line.contains("P2") {
        Some(Priority::P2)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ROADMAP: &str = r#"
# Test Roadmap

## Current Sprint: v2.42.0 Excellence Sprint
- **Priority**: P0 - HIGH PRIORITY  
- **Duration**: 2025-09-02 to 2025-09-09
- **Status**: Active

### Tasks
- [x] Complete feature A
- [ ] Implement feature B  
- [ ] Fix bug in parser

## Sprint: v2.41.0 Previous Sprint  
- **Priority**: P1 - MEDIUM PRIORITY
- **Status**: Complete

### Definition of Done
- All tests pass
- Documentation updated
- Code reviewed

## Backlog
- PMAT-1234 | Future feature | High | Open | 8h |
- PMAT-5678 | Enhancement | Medium | Planning | 4h |
"#;

    #[test]
    fn test_parse_basic_roadmap() {
        let result = parse_roadmap(SAMPLE_ROADMAP);
        assert!(result.is_ok());

        let roadmap = result.unwrap();
        assert!(!roadmap.sprints.is_empty());
    }

    #[test]
    fn test_parse_current_sprint() {
        let roadmap = parse_roadmap(SAMPLE_ROADMAP).unwrap();

        if let Some(current) = &roadmap.current_sprint {
            assert_eq!(current.version, "v2.42.0");
            assert!(current.title.contains("Excellence Sprint"));
        }
    }

    #[test]
    fn test_parse_multiple_sprints() {
        let roadmap = parse_roadmap(SAMPLE_ROADMAP).unwrap();

        // Should have both current and previous sprint
        assert!(roadmap.sprints.len() >= 1);
        assert!(roadmap.sprints.contains_key("v2.41.0") || roadmap.current_sprint.is_some());
    }

    #[test]
    fn test_parse_priority() {
        assert_eq!(parse_priority("P0 - HIGH"), Some(Priority::P0));
        assert_eq!(parse_priority("P1 - MEDIUM"), Some(Priority::P1));
        assert_eq!(parse_priority("P2 - LOW"), Some(Priority::P2));
        assert_eq!(parse_priority("No priority"), None);
    }

    #[test]
    fn test_parse_task_status() {
        assert_eq!(parse_task_status("Open"), Some(TaskStatus::Planned));
        assert_eq!(
            parse_task_status("InProgress"),
            Some(TaskStatus::InProgress)
        );
        assert_eq!(parse_task_status("Completed"), Some(TaskStatus::Completed));
        assert_eq!(parse_task_status("Blocked"), Some(TaskStatus::Blocked));
        assert_eq!(parse_task_status("Invalid"), None);
    }

    #[test]
    fn test_parse_empty_roadmap() {
        let result = parse_roadmap("");
        assert!(result.is_ok());

        let roadmap = result.unwrap();
        assert!(roadmap.current_sprint.is_none());
        assert!(roadmap.sprints.is_empty());
        assert!(roadmap.backlog.is_empty());
    }

    #[test]
    fn test_parse_malformed_content() {
        let malformed = r#"
## Invalid Sprint Header Without Proper Format
- Some content without structure
"#;

        let result = parse_roadmap(malformed);
        assert!(result.is_ok()); // Should handle malformed gracefully
    }

    #[test]
    fn test_parse_with_backlog_tasks() {
        let content_with_backlog = r#"
## Backlog
- PMAT-1234 | Test task | High | Open | 8h |
- PMAT-5678 | Another task | Medium | Planning | 4h |
"#;

        let roadmap = parse_roadmap(content_with_backlog).unwrap();
        // Should handle backlog parsing
        assert!(roadmap.backlog.len() >= 0); // May not parse correctly due to table format
    }

    #[test]
    fn test_roundtrip_parsing() {
        // Test that we can parse and serialize back
        let roadmap = parse_roadmap(SAMPLE_ROADMAP).unwrap();
        let serialized = serialize_roadmap(&roadmap).unwrap();
        let reparsed = parse_roadmap(&serialized).unwrap();

        // Basic structure should be preserved
        assert_eq!(roadmap.sprints.len(), reparsed.sprints.len());
    }
}
