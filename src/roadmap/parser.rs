#![cfg_attr(coverage_nightly, coverage(off))]
//! Roadmap markdown parser and serializer

use super::{Complexity, DateTime, Priority, Roadmap, Sprint, Task, TaskStatus};
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

/// Container for regex parsers
struct Parsers {
    sprint_regex: Regex,
    task_regex: Regex,
    done_regex: Regex,
}

/// Create regex parsers
fn create_parsers() -> Result<Parsers> {
    Ok(Parsers {
        sprint_regex: Regex::new(r"## (?:Current |Previous |Next )?Sprint: (v[\d.]+) (.+)")?,
        task_regex: Regex::new(r"\| (PMAT-\d{4}) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|")?,
        done_regex: Regex::new(r"- \[([ x])\] (.+)")?,
    })
}

/// Parse a roadmap from markdown content
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn parse_roadmap(content: &str) -> Result<Roadmap> {
    let mut roadmap = Roadmap {
        current_sprint: None,
        sprints: HashMap::new(),
        backlog: Vec::new(),
        completed_sprints: Vec::new(),
    };

    let parsers = create_parsers()?;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check for sprint header
        if let Some(captures) = parsers.sprint_regex.captures(line) {
            let (sprint, version, advance) = parse_sprint_section(&lines, i, &captures, &parsers)?;
            i += advance;

            // Update roadmap state
            update_roadmap_state(&mut roadmap, line, &version);
            roadmap.sprints.insert(version, sprint);
        }
        // Parse backlog
        else if line.contains("### Backlog") {
            let (tasks, advance) = parse_backlog_section(&lines, i, &parsers.task_regex)?;
            i += advance;
            roadmap.backlog = tasks;
        } else {
            i += 1;
        }
    }

    Ok(roadmap)
}

/// Update roadmap state based on sprint header
fn update_roadmap_state(roadmap: &mut Roadmap, line: &str, version: &str) {
    // Determine if this is the current sprint
    if line.contains("Current Sprint:")
        || (roadmap.current_sprint.is_none() && !line.contains("Previous"))
    {
        roadmap.current_sprint = Some(version.to_string());
    }

    // Mark completed sprints
    if line.contains("\u{2705} COMPLETED") {
        roadmap.completed_sprints.push(version.to_string());
    }
}

// Utility parsing helpers: parse_task_status, parse_duration, parse_priority
include!("parser_helpers.rs");

// Sprint parsing: parse_sprint_section, sprint content, tasks, backlog, definition of done
include!("parser_sprint.rs");

// Roadmap serialization: roadmap_to_markdown, format_sprint, format_task
include!("parser_serialize.rs");

// Unit tests and property tests
include!("parser_tests.rs");
