// Utility parsing helpers: parse_task_status, parse_duration, parse_priority

fn parse_task_status(s: &str) -> TaskStatus {
    debug_assert!(!s.is_empty(), "s must not be empty");
    let s = s.trim();
    TaskStatus::from_emoji(s).unwrap_or_else(|| match s.to_lowercase().as_str() {
        "planned" => TaskStatus::Planned,
        "in_progress" | "in progress" | "inprogress" => TaskStatus::InProgress,
        "completed" | "done" => TaskStatus::Completed,
        "blocked" => TaskStatus::Blocked,
        "deferred" => TaskStatus::Deferred,
        _ => TaskStatus::Planned,
    })
}

fn parse_duration(line: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    debug_assert!(!line.is_empty(), "line must not be empty");
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
    debug_assert!(!line.is_empty(), "line must not be empty");
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
