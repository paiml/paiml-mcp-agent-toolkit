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

fn show_task_status(roadmap: &Roadmap, task_id: &str, format: OutputFormat) -> Result<()> {
    let task = roadmap
        .get_task(task_id)
        .context(format!("Task {task_id} not found"))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(task)?);
        }
        _ => {
            display_task_details(task);
        }
    }

    Ok(())
}

fn display_task_details(task: &Task) {
    println!("Task {}: {}", task.id, task.status.to_emoji());
    println!("  Description: {}", task.description);
    println!("  Complexity: {:?}", task.complexity);
    println!("  Priority: {:?}", task.priority);

    if let Some(started) = task.started_at {
        println!("  Started: {}", started.format("%Y-%m-%d %H:%M"));
    }

    if let Some(completed) = task.completed_at {
        println!("  Completed: {}", completed.format("%Y-%m-%d %H:%M"));
    }
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

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(sprint)?);
        }
        _ => {
            display_sprint_details(sprint);
        }
    }

    Ok(())
}

fn display_sprint_details(sprint: &Sprint) {
    let (completed, in_progress, total) = calculate_sprint_progress(sprint);

    println!("Sprint {}: {}", sprint.version, sprint.title);
    println!(
        "  Duration: {} to {}",
        sprint.start_date.format("%Y-%m-%d"),
        sprint.end_date.format("%Y-%m-%d")
    );
    println!("  Progress: {completed}/{total} completed, {in_progress} in progress");

    display_sprint_tasks(sprint);
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

fn display_sprint_tasks(sprint: &Sprint) {
    println!("\n  Tasks:");
    for task in &sprint.tasks {
        println!(
            "    {} {} - {}",
            task.status.to_emoji(),
            task.id,
            task.description
        );
    }
}
