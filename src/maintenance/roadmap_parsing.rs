/// Extract version from roadmap title
///
/// # Complexity
/// - Time: O(n) where n is content length
/// - Cyclomatic: 3
fn extract_version(content: &str) -> String {
    content
        .lines()
        .find(|line| line.contains("v2."))
        .and_then(|line| {
            line.split("v2.")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
        })
        .map(|s| format!("v2.{}", s))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Parse sprint header line
///
/// # Example
/// "### Sprint 16: Scaffolding Foundation (2-3 days) - COMPLETE ✅"
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 6 (reduced from 12 via Extract Method refactoring)
fn parse_sprint_header(line: &str) -> Option<Sprint> {
    if !line.starts_with("### Sprint ") {
        return None;
    }

    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    // Extract sprint number
    let number = parts[0].split_whitespace().nth(2)?.parse::<u32>().ok()?;

    // Extract name and status
    let rest = parts[1].trim();
    let name = rest.split(" (").next()?.trim().to_string();
    let status = parse_sprint_status(rest);
    let duration = extract_duration(rest);

    Some(Sprint {
        number,
        name,
        focus: String::new(),
        status,
        duration,
        tickets: Vec::new(),
        quality_gates: Vec::new(),
    })
}

/// Extract sprint status from header text
///
/// # Complexity
/// - Time: O(n) where n is string length
/// - Cyclomatic: 3
fn parse_sprint_status(text: &str) -> SprintStatus {
    if text.contains(" - COMPLETE") {
        SprintStatus::Complete
    } else if text.contains(" - IN PROGRESS") {
        SprintStatus::InProgress
    } else {
        SprintStatus::Planned
    }
}

/// Extract duration from parenthesized text
///
/// # Complexity
/// - Time: O(n) where n is string length
/// - Cyclomatic: 3
fn extract_duration(text: &str) -> String {
    if let Some(start) = text.find('(') {
        if let Some(end) = text.find(')') {
            return text.get(start + 1..end).unwrap_or_default().to_string();
        }
    }
    "unknown".to_string()
}

/// Parse ticket line
///
/// # Example
/// "- [x] TICKET-PMAT-5001: Core ScaffoldEngine implementation (commit: 1adfcd7)"
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 6
fn parse_ticket_line(line: &str) -> Option<Ticket> {
    let line = line.trim();

    if !line.starts_with("- [") {
        return None;
    }

    // Only parse lines that contain TICKET- (skip quality gates, etc.)
    if !line.contains("TICKET-") {
        return None;
    }

    let completed = line.contains("[x]");

    // Extract ticket ID and description
    let content = if completed {
        line.strip_prefix("- [x]")?.trim()
    } else {
        line.strip_prefix("- [ ]")?.trim()
    };

    let parts: Vec<&str> = content.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let id = parts[0].trim().to_string();
    let description = parts[1].split('(').next()?.trim().to_string();

    // Extract commit if present
    let commit = if let Some(commit_start) = content.find("(commit: ") {
        let commit_end = content.get(commit_start..)?.find(')')?;
        Some(
            content
                .get(commit_start + 9..commit_start + commit_end)
                .unwrap_or_default()
                .to_string(),
        )
    } else {
        None
    };

    Some(Ticket {
        id,
        description,
        completed,
        commit,
    })
}

/// Check if line is quality gate section header
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn is_quality_gate_section(line: &str) -> bool {
    line.trim() == "**Quality Gates:**"
}

/// Parse quality gate line
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn parse_quality_gate(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with("- ") {
        Some(line.strip_prefix("- ")?.to_string())
    } else {
        None
    }
}

/// Validate ticket ID format
///
/// # Format
/// TICKET-PMAT-XXXX where XXXX is 4 digits
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn validate_ticket_id(id: &str) -> Result<()> {
    if !id.starts_with("TICKET-PMAT-") {
        return Err(RoadmapError::InvalidTicketId(id.to_string()));
    }

    let number_part = id
        .strip_prefix("TICKET-PMAT-")
        .ok_or_else(|| RoadmapError::InvalidTicketId(id.to_string()))?;

    if number_part.len() != 4 || !number_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(RoadmapError::InvalidTicketId(id.to_string()));
    }

    Ok(())
}
