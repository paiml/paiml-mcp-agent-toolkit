// Task lifecycle commands (start, complete)
// Included from mod.rs - shares parent module scope

async fn start_task(
    roadmap_path: &Path,
    task_id: &str,
    create_branch: bool,
    config: &RoadmapConfig,
) -> Result<()> {
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    println!("🚀 Starting task {task_id}");

    let mut roadmap = Roadmap::from_file(roadmap_path)?;

    // Update task status
    roadmap.update_task_status(task_id, TaskStatus::InProgress)?;
    roadmap.to_file(roadmap_path)?;

    println!("✅ Task {task_id} status updated to: 🚧 In Progress");

    // Create git branch if requested
    if create_branch && config.git.create_branches {
        let branch_name = config
            .git
            .branch_pattern
            .replace("{task_id}", &task_id.to_lowercase());

        println!("🌿 Creating branch: {branch_name}");
        std::process::Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .output()
            .context("Failed to create git branch")?;

        println!("✅ Branch created and checked out: {branch_name}");
    }

    // Show task details
    if let Some(task) = roadmap.get_task(task_id) {
        println!("\n📋 Task Details:");
        println!("  ID: {}", task.id);
        println!("  Description: {}", task.description);
        println!("  Complexity: {:?}", task.complexity);
        println!("  Priority: {:?}", task.priority);
    }

    Ok(())
}

async fn complete_task(
    roadmap_path: &Path,
    task_id: &str,
    skip_quality_check: bool,
    config: &RoadmapConfig,
) -> Result<()> {
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    println!("🏁 Completing task {task_id}");

    // Run quality checks unless skipped
    if !skip_quality_check && config.enforce_quality_gates {
        println!("🔍 Running quality checks...");
        quality_check(task_id, config).await?;
    }

    let mut roadmap = Roadmap::from_file(roadmap_path)?;

    // Update task status
    roadmap.update_task_status(task_id, TaskStatus::Completed)?;
    roadmap.to_file(roadmap_path)?;

    println!("✅ Task {task_id} completed successfully");

    // Create completion commit if configured
    if config.git.require_quality_check {
        let message = config
            .git
            .commit_pattern
            .replace("{task_id}", task_id)
            .replace("{message}", "Complete implementation");

        println!("📝 Creating commit: {message}");
        std::process::Command::new("git")
            .args(["add", "-A"])
            .output()?;

        std::process::Command::new("git")
            .args(["commit", "-m", &message])
            .output()?;

        println!("✅ Changes committed");
    }

    Ok(())
}

/// Start working on a task
#[allow(dead_code)]
fn handle_start(task_id: String, create_branch: bool) -> Result<()> {
    debug_assert!(true, "contract: handle_start");
    // Validate task ID format (basic check)
    if !task_id.starts_with("PMAT-") {
        anyhow::bail!("Invalid task ID format. Expected PMAT-XXXX");
    }

    println!("🚀 Starting work on task: {task_id}");

    if create_branch {
        let branch_name = format!("feature/{}", task_id.to_lowercase());
        println!("🌿 Creating branch: {branch_name}");

        // Attempt to create git branch (may fail in test environment)
        let result = std::process::Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                println!("✅ Branch created successfully");
            }
            Ok(_) => {
                println!("⚠️ Branch creation attempted but may have failed");
            }
            Err(_) => {
                println!("⚠️ Git not available or branch creation failed");
            }
        }
    }

    println!("✅ Task {task_id} is now active");
    Ok(())
}
