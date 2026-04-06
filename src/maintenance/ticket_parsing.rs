/// Parse ticket header line
///
/// # Example
/// "# TICKET-PMAT-5011: Ticket Management System"
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn parse_header(line: &str) -> Result<(String, String)> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if !line.starts_with("# TICKET-") {
        return Err(TicketError::ParseError("Invalid header format".into()));
    }

    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return Err(TicketError::ParseError("Header missing title".into()));
    }

    let id = parts[0].trim_start_matches("# ").trim().to_string();
    let title = parts[1].trim().to_string();

    Ok((id, title))
}

/// Extract metadata value
///
/// # Complexity
/// - Time: O(n) where n is number of lines
/// - Cyclomatic: 3
fn extract_metadata(lines: &[&str], key: &str) -> Result<String> {
    debug_assert!(!key.is_empty(), "key must not be empty");
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    for line in lines {
        if line.starts_with(key) {
            let value = line
                .strip_prefix(key)
                .and_then(|s| s.strip_prefix(":"))
                .map(|s| s.trim())
                .ok_or_else(|| {
                    TicketError::ParseError(format!("Invalid metadata format for {}", key))
                })?;
            return Ok(value.to_string());
        }
    }
    Err(TicketError::MissingField(key.to_string()))
}

/// Extract section content
///
/// # Complexity
/// - Time: O(n) where n is number of lines
/// - Cyclomatic: 4
fn extract_section(lines: &[&str], header: &str) -> Result<String> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut in_section = false;
    let mut content = String::new();

    for line in lines {
        if line.starts_with(header) {
            in_section = true;
            continue;
        }

        if in_section {
            if line.starts_with("## ") {
                // Next section started
                break;
            }
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(line);
        }
    }

    if content.is_empty() {
        Err(TicketError::MissingField(header.to_string()))
    } else {
        Ok(content.trim().to_string())
    }
}

/// Extract checklist items
///
/// # Complexity
/// - Time: O(n) where n is number of lines
/// - Cyclomatic: 4
fn extract_checklist(lines: &[&str], header: &str) -> Result<Vec<String>> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut in_section = false;
    let mut items = Vec::new();

    for line in lines {
        if line.starts_with(header) {
            in_section = true;
            continue;
        }

        if in_section {
            if line.starts_with("## ") {
                break;
            }
            if line.trim().starts_with("- [ ]") {
                let item = line
                    .trim()
                    .strip_prefix("- [ ]")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                items.push(item);
            }
        }
    }

    Ok(items)
}

/// Parse status string
///
/// # Complexity
/// - Time: O(n) where n is string length (for emoji stripping)
/// - Cyclomatic: 5
///
/// # Note
/// Strips non-ASCII characters (emojis) from status values.
/// Example: "GREEN ✅" → "GREEN"
fn parse_status(s: &str) -> Result<TicketStatus> {
    debug_assert!(!s.is_empty(), "s must not be empty");
    // Strip non-ASCII characters (emojis) and trim whitespace
    let clean_status: String = s
        .chars()
        .filter(|c| c.is_ascii())
        .collect::<String>()
        .trim()
        .to_uppercase();

    match clean_status.as_str() {
        "RED" => Ok(TicketStatus::Red),
        "GREEN" => Ok(TicketStatus::Green),
        "REFACTOR" => Ok(TicketStatus::Refactor),
        "COMPLETE" => Ok(TicketStatus::Complete),
        _ => Err(TicketError::InvalidStatus(s.to_string())),
    }
}

/// Parse priority string
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 4
fn parse_priority(s: &str) -> Result<Priority> {
    debug_assert!(!s.is_empty(), "s must not be empty");
    match s.to_uppercase().as_str() {
        "P0" => Ok(Priority::P0),
        "P1" => Ok(Priority::P1),
        "P2" => Ok(Priority::P2),
        _ => Err(TicketError::InvalidPriority(s.to_string())),
    }
}

/// Parse complexity number
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn parse_complexity(s: &str) -> Result<u8> {
    debug_assert!(!s.is_empty(), "s must not be empty");
    s.parse::<u8>()
        .map_err(|_| TicketError::ParseError(format!("Invalid complexity: {}", s)))
}

/// Parse dependencies list
///
/// # Complexity
/// - Time: O(n) where n is number of dependencies
/// - Cyclomatic: 2
fn parse_dependencies(s: &str) -> Vec<String> {
    debug_assert!(!s.is_empty(), "s must not be empty");
    if s.to_lowercase() == "none" {
        return Vec::new();
    }

    s.split(',')
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .collect()
}

/// List all tickets in a directory
///
/// # Complexity
/// - Time: O(n) where n is number of files
/// - Cyclomatic: 4
pub fn list_tickets(dir: &Path) -> Result<Vec<TicketFile>> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut tickets = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if name.starts_with("TICKET-PMAT-") {
                    match TicketFile::from_file(&path) {
                        Ok(ticket) => tickets.push(ticket),
                        Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
                    }
                }
            }
        }
    }

    Ok(tickets)
}

/// Check if ticket exists
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
pub fn ticket_exists(tickets_dir: &Path, ticket_id: &str) -> bool {
    debug_assert!(tickets_dir.exists(), "tickets_dir must exist: {}", tickets_dir.display());
    let path = tickets_dir.join(format!("{}.md", ticket_id));
    path.exists()
}
