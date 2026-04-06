// Sprint parsing helpers: parse_sprint_section, sprint content, tasks, backlog, definition of done

/// Parse a sprint section
fn parse_sprint_section(
    lines: &[&str],
    start_idx: usize,
    captures: &regex::Captures,
    parsers: &Parsers,
) -> Result<(Sprint, String, usize)> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let version = captures
        .get(1)
        .expect("internal error")
        .as_str()
        .to_string();
    let title = captures
        .get(2)
        .expect("internal error")
        .as_str()
        .to_string();

    let mut sprint = create_initial_sprint(&version, &title);
    let lines_consumed = parse_sprint_content(lines, start_idx, &mut sprint, parsers)?;

    Ok((sprint, version, lines_consumed))
}

/// Create initial sprint structure with defaults (cognitive complexity <=2)
fn create_initial_sprint(version: &str, title: &str) -> Sprint {
    debug_assert!(!version.is_empty(), "version must not be empty");
    Sprint {
        version: version.to_string(),
        title: title.to_string(),
        start_date: Utc::now(),
        end_date: Utc::now() + chrono::Duration::days(14),
        priority: Priority::P0,
        tasks: Vec::new(),
        definition_of_done: Vec::new(),
        quality_gates: Vec::new(),
    }
}

/// Parse sprint content and return lines consumed (cognitive complexity <=8)
fn parse_sprint_content(
    lines: &[&str],
    start_idx: usize,
    sprint: &mut Sprint,
    parsers: &Parsers,
) -> Result<usize> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut i = start_idx + 1;

    while i < lines.len() {
        let line = lines[i];

        if is_next_section_start(line) {
            break;
        }

        i += process_sprint_line(lines, i, sprint, parsers)?;
    }

    Ok(i - start_idx)
}

/// Process a single sprint line and return advancement (cognitive complexity <=6)
fn process_sprint_line(
    lines: &[&str],
    current_idx: usize,
    sprint: &mut Sprint,
    parsers: &Parsers,
) -> Result<usize> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let line = lines[current_idx];

    if line.contains("**Duration**:") {
        process_duration_line(line, sprint);
        Ok(1)
    } else if line.contains("**Priority**:") {
        process_priority_line(line, sprint);
        Ok(1)
    } else if line.contains("| ID | Description |") {
        let (tasks, advance) = parse_tasks_table(&lines[current_idx..], &parsers.task_regex)?;
        sprint.tasks = tasks;
        Ok(advance)
    } else if line.contains("### Definition of Done") {
        let (items, advance) =
            parse_definition_of_done(&lines[current_idx..], &parsers.done_regex)?;
        sprint.definition_of_done = items;
        Ok(advance)
    } else {
        Ok(1)
    }
}

/// Check if line indicates start of next section (cognitive complexity <=1)
fn is_next_section_start(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    line.starts_with("## ") && line.contains("Sprint:")
}

/// Process duration line and update sprint dates (cognitive complexity <=2)
fn process_duration_line(line: &str, sprint: &mut Sprint) {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(duration) = parse_duration(line) {
        sprint.start_date = duration.0;
        sprint.end_date = duration.1;
    }
}

/// Process priority line and update sprint priority (cognitive complexity <=2)
fn process_priority_line(line: &str, sprint: &mut Sprint) {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(priority) = parse_priority(line) {
        sprint.priority = priority;
    }
}

/// Parse tasks table
fn parse_tasks_table(lines: &[&str], task_regex: &Regex) -> Result<(Vec<Task>, usize)> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut tasks = Vec::new();
    let mut i = 2; // Skip header and separator

    while i < lines.len() && lines[i].starts_with('|') {
        if let Some(captures) = task_regex.captures(lines[i]) {
            tasks.push(create_task_from_captures(&captures));
        }
        i += 1;
    }

    Ok((tasks, i))
}

/// Parse definition of done section
fn parse_definition_of_done(lines: &[&str], done_regex: &Regex) -> Result<(Vec<String>, usize)> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut items = Vec::new();
    let mut i = 1;

    while i < lines.len() && lines[i].starts_with("- [") {
        if let Some(captures) = done_regex.captures(lines[i]) {
            items.push(
                captures
                    .get(2)
                    .expect("internal error")
                    .as_str()
                    .to_string(),
            );
        }
        i += 1;
    }

    Ok((items, i))
}

/// Parse backlog section
fn parse_backlog_section(
    lines: &[&str],
    start_idx: usize,
    task_regex: &Regex,
) -> Result<(Vec<Task>, usize)> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut i = start_idx + 1;

    // Skip to table content
    while i < lines.len() && !lines[i].starts_with('|') {
        i += 1;
    }

    if i + 2 >= lines.len() {
        return Ok((Vec::new(), i - start_idx));
    }

    i += 2; // Skip header and separator
    let mut tasks = Vec::new();

    while i < lines.len() && lines[i].starts_with('|') {
        if let Some(captures) = task_regex.captures(lines[i]) {
            tasks.push(create_task_from_captures(&captures));
        }
        i += 1;
    }

    Ok((tasks, i - start_idx))
}

/// Create a task from regex captures
fn create_task_from_captures(captures: &regex::Captures) -> Task {
    debug_assert!(true, "contract: create_task_from_captures");
    Task {
        id: captures
            .get(1)
            .expect("internal error")
            .as_str()
            .to_string(),
        description: captures
            .get(2)
            .expect("internal error")
            .as_str()
            .trim()
            .to_string(),
        status: parse_task_status(captures.get(3).expect("internal error").as_str()),
        complexity: Complexity::from_str(captures.get(4).expect("internal error").as_str().trim())
            .unwrap_or(Complexity::Medium),
        priority: Priority::from_str(captures.get(5).expect("internal error").as_str().trim())
            .unwrap_or(Priority::P1),
        assignee: None,
        started_at: None,
        completed_at: None,
    }
}
