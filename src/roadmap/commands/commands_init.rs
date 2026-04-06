// Sprint initialization commands
// Included from mod.rs - shares parent module scope

async fn init_sprint(
    roadmap_path: &Path,
    version: &str,
    title: &str,
    duration_days: u32,
    priority: &str,
) -> Result<()> {
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    println!("📋 Initializing sprint {version} - {title}");

    let mut roadmap = if roadmap_path.exists() {
        Roadmap::from_file(roadmap_path)?
    } else {
        Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        }
    };

    let sprint = Sprint {
        version: version.to_string(),
        title: title.to_string(),
        start_date: Utc::now(),
        end_date: Utc::now() + chrono::Duration::days(i64::from(duration_days)),
        priority: Priority::from_str(priority).unwrap_or(Priority::P0),
        tasks: Vec::new(),
        definition_of_done: vec![
            "All tasks completed".to_string(),
            "Quality gates passed".to_string(),
            "Documentation updated".to_string(),
            "Tests passing".to_string(),
            "Changelog updated".to_string(),
        ],
        quality_gates: vec![
            format!("Complexity ≤ 20"),
            format!("SATD = 0"),
            format!("Coverage ≥ 80%"),
        ],
    };

    roadmap.sprints.insert(version.to_string(), sprint);
    if roadmap.current_sprint.is_none() {
        roadmap.current_sprint = Some(version.to_string());
    }

    roadmap.to_file(roadmap_path)?;

    println!("✅ Sprint {version} initialized successfully");
    println!("📝 Roadmap updated at: {}", roadmap_path.display());

    Ok(())
}

/// Initialize a new sprint in the roadmap
#[allow(dead_code)]
fn handle_init(
    version: String,
    title: String,
    duration_days: u32,
    priority: String,
    roadmap_path: PathBuf,
) -> Result<()> {
    // Validate priority
    Priority::from_str(&priority)
        .map_err(|()| anyhow::anyhow!("Invalid priority format. Use P0, P1, or P2"))?;

    // Create basic roadmap structure
    let content = format!(
        r"# Roadmap

## Current Sprint: {version} {title}
- **Duration**: {duration_days} days
- **Priority**: {priority}
- **Status**: Active

### Tasks
- [ ] Initial task placeholder

### Quality Gates
- [ ] All tests pass
- [ ] Code coverage maintained
- [ ] Zero SATD violations

"
    );

    std::fs::write(&roadmap_path, content)
        .with_context(|| format!("Failed to write roadmap to {roadmap_path:?}"))?;

    println!("✅ Initialized roadmap at {roadmap_path:?}");
    Ok(())
}
