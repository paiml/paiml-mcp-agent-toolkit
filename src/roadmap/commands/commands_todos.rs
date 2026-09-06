// Todo generation commands
// Included from mod.rs - shares parent module scope

async fn generate_todos(
    roadmap_path: &Path,
    sprint_id: Option<&str>,
    output_path: &Path,
    include_quality_gates: bool,
    config: &RoadmapConfig,
) -> Result<()> {
    println!("🔄 Generating PDMT todos from roadmap...");

    let roadmap = Roadmap::from_file(roadmap_path)?;

    let sprint_id = sprint_id
        .or(roadmap.current_sprint.as_deref())
        .context("No sprint specified and no current sprint found")?;

    let sprint = roadmap
        .get_sprint(sprint_id)
        .context(format!("Sprint {sprint_id} not found"))?;

    let generator = generator::RoadmapTodoGenerator::new(config.quality_gates.clone());
    let todos = generator.generate_sprint_todos(sprint).await?;

    println!(
        "📝 Generated {} todos for {} tasks",
        todos.len(),
        sprint.tasks.len()
    );

    let output = if include_quality_gates {
        generator.export_todos_markdown(&todos)
    } else {
        // Simple format without quality details
        let mut simple = String::new();
        for todo in &todos {
            simple.push_str(&format!("- [ ] {}: {}\n", todo.id, todo.description));
        }
        simple
    };

    std::fs::write(output_path, output)?;
    println!(
        "{}",
        todos_written_line(output_path, todos.len(), include_quality_gates)
    );

    Ok(())
}

/// The confirmation line `roadmap todos` prints after writing its file.
fn todos_written_line(output_path: &Path, count: usize, include_quality_gates: bool) -> String {
    let _ = (count, include_quality_gates);
    format!("✅ Todos written to: {}", output_path.display())
}
