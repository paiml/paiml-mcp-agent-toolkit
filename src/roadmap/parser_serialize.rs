// Roadmap serialization: roadmap_to_markdown, format_sprint, format_task, section helpers

/// Convert a roadmap to markdown format
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn roadmap_to_markdown(roadmap: &Roadmap) -> Result<String> {
    let mut output = String::new();

    output.push_str("# PMAT Development Roadmap\n\n");

    // Extract each section to reduce cognitive complexity
    add_current_sprint_section(&mut output, roadmap)?;
    add_completed_sprints_section(&mut output, roadmap)?;
    add_future_sprints_section(&mut output, roadmap)?;
    add_backlog_section(&mut output, roadmap)?;

    Ok(output)
}

/// Add current sprint section to output (cognitive complexity <=3)
fn add_current_sprint_section(output: &mut String, roadmap: &Roadmap) -> Result<()> {
    if let Some(current_id) = &roadmap.current_sprint {
        if let Some(sprint) = roadmap.sprints.get(current_id) {
            output.push_str(&format_sprint(sprint, true, false)?);
            output.push('\n');
        }
    }
    Ok(())
}

/// Add completed sprints section to output (cognitive complexity <=3)
fn add_completed_sprints_section(output: &mut String, roadmap: &Roadmap) -> Result<()> {
    for sprint_id in &roadmap.completed_sprints {
        if let Some(sprint) = roadmap.sprints.get(sprint_id) {
            output.push_str(&format_sprint(sprint, false, true)?);
            output.push('\n');
        }
    }
    Ok(())
}

/// Add future sprints section to output (cognitive complexity <=5)
fn add_future_sprints_section(output: &mut String, roadmap: &Roadmap) -> Result<()> {
    for (id, sprint) in &roadmap.sprints {
        if is_future_sprint(id, roadmap) {
            output.push_str(&format_sprint(sprint, false, false)?);
            output.push('\n');
        }
    }
    Ok(())
}

/// Check if sprint is a future sprint (not current and not completed)
fn is_future_sprint(id: &String, roadmap: &Roadmap) -> bool {
    roadmap.current_sprint.as_ref() != Some(id) && !roadmap.completed_sprints.contains(id)
}

/// Add backlog section to output (cognitive complexity <=4)
fn add_backlog_section(output: &mut String, roadmap: &Roadmap) -> Result<()> {
    if !roadmap.backlog.is_empty() {
        output.push_str("### Backlog \u{1f4cb}\n");
        output.push_str("| ID | Description | Status | Complexity | Priority |\n");
        output.push_str("|----|-------------|--------|------------|----------|\n");
        for task in &roadmap.backlog {
            output.push_str(&format_task(task)?);
        }
        output.push('\n');
    }
    Ok(())
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
        " \u{2705} COMPLETED"
    } else {
        " \u{1f4cb} PLANNED"
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
            output.push_str(&format!("- [{checked}] {item}\n"));
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
