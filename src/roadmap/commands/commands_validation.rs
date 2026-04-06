// Validation and quality check commands
// Included from mod.rs - shares parent module scope

async fn validate_sprint(
    roadmap_path: &Path,
    sprint_id: &str,
    strict: bool,
    config: &RoadmapConfig,
) -> Result<()> {
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    println!("🔍 Validating sprint {sprint_id} for release...");

    let roadmap = Roadmap::from_file(roadmap_path)?;
    let sprint = roadmap
        .get_sprint(sprint_id)
        .context(format!("Sprint {sprint_id} not found"))?;

    let mut all_passed = true;

    // Check all tasks completed
    let incomplete_tasks: Vec<_> = sprint
        .tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Completed)
        .collect();

    if incomplete_tasks.is_empty() {
        println!("✅ All tasks completed");
    } else {
        println!("❌ Incomplete tasks:");
        for task in incomplete_tasks {
            println!("    {} - {}", task.id, task.description);
        }
        all_passed = false;
    }

    // Check definition of done
    println!("\n📋 Definition of Done:");
    for item in &sprint.definition_of_done {
        println!("  - [ ] {item}");
    }

    // Check quality gates
    if config.enforce_quality_gates {
        println!("\n🔍 Quality Gates:");
        for gate in &sprint.quality_gates {
            println!("  - [ ] {gate}");
        }
    }

    if all_passed {
        println!("\n✅ Sprint {sprint_id} is ready for release!");
    } else {
        println!("\n❌ Sprint {sprint_id} is NOT ready for release");
        if strict {
            anyhow::bail!("Sprint validation failed");
        }
    }

    Ok(())
}

async fn quality_check(task_id: &str, config: &RoadmapConfig) -> Result<()> {
    println!("🔍 Running quality checks for task {task_id}...");

    // Run complexity check
    let complexity_result = std::process::Command::new("pmat")
        .args([
            "analyze",
            "complexity",
            "--max-cyclomatic",
            &config.quality_gates.complexity_max.to_string(),
        ])
        .output()?;

    if !complexity_result.status.success() {
        println!("❌ Complexity check failed");
        anyhow::bail!("Complexity exceeds limit");
    }
    println!("✅ Complexity check passed");

    // Run SATD check
    let satd_result = std::process::Command::new("pmat")
        .args(["analyze", "satd", "--strict"])
        .output()?;

    if !satd_result.status.success() && config.quality_gates.satd_tolerance == 0 {
        println!("❌ SATD check failed");
        anyhow::bail!("SATD violations found");
    }
    println!("✅ SATD check passed");

    // Run lint check
    if config.quality_gates.lint_compliance {
        let lint_result = std::process::Command::new("make").args(["lint"]).output()?;

        if !lint_result.status.success() {
            println!("❌ Lint check failed");
            anyhow::bail!("Lint violations found");
        }
        println!("✅ Lint check passed");
    }

    println!("✅ All quality checks passed for task {task_id}");
    Ok(())
}
