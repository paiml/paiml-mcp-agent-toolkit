// Display and formatting functions for roadmap health reports
// Included by roadmap_handler.rs - shares parent module scope

/// Show roadmap health report
async fn show_health_report(
    roadmap_path: &Path,
    tickets_dir: &Path,
    format: &OutputFormat,
) -> Result<()> {
    let roadmap_content = fs::read_to_string(roadmap_path).map_err(|_| {
        let error = crate::cli::error_context::roadmap_not_found(roadmap_path);
        anyhow::anyhow!(error.format_detailed())
    })?;
    let sprints = parse_sprint_info(&roadmap_content, tickets_dir)?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&sprints)?);
        }
        OutputFormat::Yaml => {
            print_health_yaml(&sprints);
        }
        OutputFormat::Table => {
            print_health_console(&sprints);
        }
    }

    Ok(())
}

/// Print health report in console format
fn print_health_console(sprints: &[SprintInfo]) {
    use crate::cli::colors as c;
    eprintln!("{}", c::header("Roadmap Health Report"));
    eprintln!();

    for sprint in sprints {
        let progress = if sprint.total_tickets > 0 {
            (sprint.completed_tickets as f64 / sprint.total_tickets as f64) * 100.0
        } else {
            0.0
        };

        let (status_icon, color) = match sprint.status {
            SprintStatus::Complete => ("✓", c::GREEN),
            SprintStatus::InProgress => ("◉", c::YELLOW),
            SprintStatus::NotStarted => ("○", c::DIM),
        };

        eprintln!("{color}{status_icon}{} {}{}", c::RESET, c::BOLD, sprint.name);
        eprintln!(
            "{}   Progress: {}{}/{}{} ({})",
            c::RESET,
            c::BOLD_WHITE,
            sprint.completed_tickets,
            c::RESET,
            sprint.total_tickets,
            c::pct(progress, 80.0, 50.0)
        );
        eprintln!();
    }
}

/// Print health report in YAML format
fn print_health_yaml(sprints: &[SprintInfo]) {
    println!("roadmap_health:");
    for sprint in sprints {
        let progress = if sprint.total_tickets > 0 {
            (sprint.completed_tickets as f64 / sprint.total_tickets as f64) * 100.0
        } else {
            0.0
        };

        println!("  - name: {}", sprint.name);
        println!("    total_tickets: {}", sprint.total_tickets);
        println!("    completed_tickets: {}", sprint.completed_tickets);
        println!("    progress: {:.0}%", progress);
        println!("    status: {:?}", sprint.status);
    }
}
