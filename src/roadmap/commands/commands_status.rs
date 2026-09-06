// Status display commands
// Included from mod.rs - shares parent module scope

async fn show_status(
    roadmap_path: &Path,
    sprint_id: Option<&str>,
    task_id: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let roadmap = Roadmap::from_file(roadmap_path)?;

    if let Some(task_id) = task_id {
        show_task_status(&roadmap, task_id, format)?;
    } else {
        show_sprint_status(&roadmap, sprint_id, format).await?;
    }

    Ok(())
}

/// `--format junit` has no meaning here: a roadmap status is not a test run, and
/// synthesising testcases from tasks would report a result nobody measured.
/// Reject it by name rather than silently rendering the human table.
fn reject_unsupported_status_format(format: OutputFormat) -> Result<()> {
    if matches!(format, OutputFormat::Junit) {
        anyhow::bail!(
            "`roadmap status --format junit` is not supported: a roadmap status has no test \
             cases to report. Use table, json, yaml, markdown, csv, summary, text or plain."
        );
    }
    Ok(())
}

fn show_task_status(roadmap: &Roadmap, task_id: &str, format: OutputFormat) -> Result<()> {
    let task = roadmap
        .get_task(task_id)
        .context(format!("Task {task_id} not found"))?;

    println!("{}", format_task_status(task, format)?);
    Ok(())
}

/// Render one task in the requested format.
///
/// Eight of the nine advertised `--format` values used to fall through a bare
/// `_ =>` arm onto the human table, so `yaml`, `markdown`, `csv` and `summary`
/// were byte-identical to `table` — a flag that parses but changes nothing.
fn format_task_status(task: &Task, format: OutputFormat) -> Result<String> {
    use std::fmt::Write as _;

    reject_unsupported_status_format(format)?;

    let mut out = String::new();
    match format {
        OutputFormat::Json => out.push_str(&serde_json::to_string_pretty(task)?),
        OutputFormat::Yaml => out.push_str(&serde_yaml_ng::to_string(task)?),
        OutputFormat::Csv => {
            out.push_str(
                "id,status,complexity,priority,assignee,started_at,completed_at,description\n",
            );
            let _ = writeln!(
                out,
                "{},{:?},{:?},{:?},{},{},{},{}",
                task.id,
                task.status,
                task.complexity,
                task.priority,
                task.assignee.as_deref().unwrap_or(""),
                task.started_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                task.completed_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                csv_field(&task.description),
            );
        }
        OutputFormat::Markdown => {
            let _ = writeln!(out, "# Task {}\n", task.id);
            let _ = writeln!(out, "- **Status**: {:?}", task.status);
            let _ = writeln!(out, "- **Description**: {}", task.description);
            let _ = writeln!(out, "- **Complexity**: {:?}", task.complexity);
            let _ = writeln!(out, "- **Priority**: {:?}", task.priority);
            if let Some(assignee) = &task.assignee {
                let _ = writeln!(out, "- **Assignee**: {assignee}");
            }
            if let Some(started) = task.started_at {
                let _ = writeln!(out, "- **Started**: {}", started.format("%Y-%m-%d %H:%M"));
            }
            if let Some(completed) = task.completed_at {
                let _ = writeln!(
                    out,
                    "- **Completed**: {}",
                    completed.format("%Y-%m-%d %H:%M")
                );
            }
        }
        OutputFormat::Summary => {
            let _ = write!(
                out,
                "{} [{:?}] {}",
                task.id, task.status, task.description
            );
        }
        _ => out.push_str(&render_task_details(task)),
    }
    Ok(out)
}

/// Escape a value for the CSV renderers: quote when it carries a delimiter.
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn render_task_details(task: &Task) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}",
        crate::cli::colors::header(&format!("Task {}: {}", task.id, task.status.to_emoji()))
    );
    let _ = writeln!(out, "  Description: {}", task.description);
    let _ = writeln!(out, "  Complexity: {:?}", task.complexity);
    let _ = write!(out, "  Priority: {:?}", task.priority);

    if let Some(started) = task.started_at {
        let _ = write!(out, "\n  Started: {}", started.format("%Y-%m-%d %H:%M"));
    }

    if let Some(completed) = task.completed_at {
        let _ = write!(out, "\n  Completed: {}", completed.format("%Y-%m-%d %H:%M"));
    }

    out
}

async fn show_sprint_status(
    roadmap: &Roadmap,
    sprint_id: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let sprint_id = sprint_id
        .or(roadmap.current_sprint.as_deref())
        .context("No sprint specified and no current sprint found")?;

    let sprint = roadmap
        .get_sprint(sprint_id)
        .context(format!("Sprint {sprint_id} not found"))?;

    println!("{}", format_sprint_status(sprint, format)?);
    Ok(())
}

/// Render one sprint in the requested format. See [`format_task_status`] for the
/// defect this replaced: every non-json value collapsed onto the human table.
fn format_sprint_status(sprint: &Sprint, format: OutputFormat) -> Result<String> {
    use std::fmt::Write as _;

    reject_unsupported_status_format(format)?;

    let (completed, in_progress, total) = calculate_sprint_progress(sprint);
    let mut out = String::new();
    match format {
        OutputFormat::Json => out.push_str(&serde_json::to_string_pretty(sprint)?),
        OutputFormat::Yaml => out.push_str(&serde_yaml_ng::to_string(sprint)?),
        OutputFormat::Csv => {
            out.push_str("sprint,task_id,status,complexity,priority,description\n");
            for task in &sprint.tasks {
                let _ = writeln!(
                    out,
                    "{},{},{:?},{:?},{:?},{}",
                    sprint.version,
                    task.id,
                    task.status,
                    task.complexity,
                    task.priority,
                    csv_field(&task.description),
                );
            }
        }
        OutputFormat::Markdown => {
            let _ = writeln!(out, "# Sprint {}: {}\n", sprint.version, sprint.title);
            let _ = writeln!(
                out,
                "- **Duration**: {} to {}",
                sprint.start_date.format("%Y-%m-%d"),
                sprint.end_date.format("%Y-%m-%d")
            );
            let _ = writeln!(
                out,
                "- **Progress**: {completed}/{total} completed, {in_progress} in progress\n"
            );
            out.push_str("| Task | Status | Description |\n| --- | --- | --- |\n");
            for task in &sprint.tasks {
                let _ = writeln!(
                    out,
                    "| {} | {:?} | {} |",
                    task.id, task.status, task.description
                );
            }
        }
        OutputFormat::Summary => {
            let _ = write!(
                out,
                "Sprint {}: {} — {completed}/{total} completed, {in_progress} in progress",
                sprint.version, sprint.title
            );
        }
        _ => out.push_str(&render_sprint_details(sprint)),
    }
    Ok(out)
}

fn render_sprint_details(sprint: &Sprint) -> String {
    use std::fmt::Write as _;

    let (completed, in_progress, total) = calculate_sprint_progress(sprint);

    let mut out = String::new();
    // PMAT-688: the human table is the one format that may carry colour;
    // the header goes through the env-aware helper so `--color always`
    // is observable and `--color never` leaves the bytes untouched.
    let _ = writeln!(
        out,
        "{}",
        crate::cli::colors::header(&format!("Sprint {}: {}", sprint.version, sprint.title))
    );
    let _ = writeln!(
        out,
        "  Duration: {} to {}",
        sprint.start_date.format("%Y-%m-%d"),
        sprint.end_date.format("%Y-%m-%d")
    );
    let _ = writeln!(
        out,
        "  Progress: {completed}/{total} completed, {in_progress} in progress"
    );

    out.push_str("\n  Tasks:");
    for task in &sprint.tasks {
        let _ = write!(
            out,
            "\n    {} {} - {}",
            task.status.to_emoji(),
            task.id,
            task.description
        );
    }

    out
}

fn calculate_sprint_progress(sprint: &Sprint) -> (usize, usize, usize) {
    let completed = sprint
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .count();

    let in_progress = sprint
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();

    let total = sprint.tasks.len();

    (completed, in_progress, total)
}
