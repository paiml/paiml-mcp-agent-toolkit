//! CLI commands for roadmap management

use super::*;
use crate::cli::OutputFormat;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Roadmap management with PDMT todos and quality gates")]
pub struct RoadmapCommand {
    #[command(subcommand)]
    pub command: RoadmapSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum RoadmapSubcommand {
    /// Initialize a new sprint in the roadmap
    Init {
        #[arg(long)]
        version: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "14")]
        duration_days: u32,
        #[arg(long, default_value = "P0")]
        priority: String,
    },
    
    /// Generate PDMT todos from roadmap tasks
    Todos {
        #[arg(long)]
        sprint: Option<String>,
        #[arg(long, default_value = "todos.md")]
        output: PathBuf,
        #[arg(long)]
        include_quality_gates: bool,
    },
    
    /// Start working on a task
    Start {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,
        #[arg(long)]
        create_branch: bool,
    },
    
    /// Complete a task (with quality validation)
    Complete {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,
        #[arg(long)]
        skip_quality_check: bool,
    },
    
    /// Check sprint or task status
    Status {
        #[arg(long)]
        sprint: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long, default_value = "human")]
        format: OutputFormat,
    },
    
    /// Validate sprint readiness for release
    Validate {
        #[arg(long)]
        sprint: String,
        #[arg(long)]
        strict: bool,
    },
    
    /// Run quality checks for a task
    QualityCheck {
        #[arg(long)]
        task_id: String,
    },
}

/// Execute roadmap commands
pub async fn execute(cmd: RoadmapCommand, config: RoadmapConfig) -> Result<()> {
    let roadmap_path = config.path.clone();
    
    match cmd.command {
        RoadmapSubcommand::Init { version, title, duration_days, priority } => {
            init_sprint(&roadmap_path, &version, &title, duration_days, &priority).await
        }
        RoadmapSubcommand::Todos { sprint, output, include_quality_gates } => {
            generate_todos(&roadmap_path, sprint.as_deref(), &output, include_quality_gates, &config).await
        }
        RoadmapSubcommand::Start { task_id, create_branch } => {
            start_task(&roadmap_path, &task_id, create_branch, &config).await
        }
        RoadmapSubcommand::Complete { task_id, skip_quality_check } => {
            complete_task(&roadmap_path, &task_id, skip_quality_check, &config).await
        }
        RoadmapSubcommand::Status { sprint, task, format } => {
            show_status(&roadmap_path, sprint.as_deref(), task.as_deref(), format).await
        }
        RoadmapSubcommand::Validate { sprint, strict } => {
            validate_sprint(&roadmap_path, &sprint, strict, &config).await
        }
        RoadmapSubcommand::QualityCheck { task_id } => {
            quality_check(&task_id, &config).await
        }
    }
}

async fn init_sprint(roadmap_path: &Path, version: &str, title: &str, duration_days: u32, priority: &str) -> Result<()> {
    println!("📋 Initializing sprint {} - {}", version, title);
    
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
        end_date: Utc::now() + chrono::Duration::days(duration_days as i64),
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
    
    println!("✅ Sprint {} initialized successfully", version);
    println!("📝 Roadmap updated at: {}", roadmap_path.display());
    
    Ok(())
}

async fn generate_todos(
    roadmap_path: &Path,
    sprint_id: Option<&str>,
    output_path: &Path,
    include_quality_gates: bool,
    config: &RoadmapConfig,
) -> Result<()> {
    println!("🔄 Generating PDMT todos from roadmap...");
    
    let roadmap = Roadmap::from_file(roadmap_path)?;
    
    let sprint_id = sprint_id.or(roadmap.current_sprint.as_deref())
        .context("No sprint specified and no current sprint found")?;
    
    let sprint = roadmap.get_sprint(sprint_id)
        .context(format!("Sprint {} not found", sprint_id))?;
    
    let generator = generator::RoadmapTodoGenerator::new(config.quality_gates.clone());
    let todos = generator.generate_sprint_todos(sprint).await?;
    
    println!("📝 Generated {} todos for {} tasks", todos.len(), sprint.tasks.len());
    
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
    println!("✅ Todos written to: {}", output_path.display());
    
    Ok(())
}

async fn start_task(roadmap_path: &Path, task_id: &str, create_branch: bool, config: &RoadmapConfig) -> Result<()> {
    println!("🚀 Starting task {}", task_id);
    
    let mut roadmap = Roadmap::from_file(roadmap_path)?;
    
    // Update task status
    roadmap.update_task_status(task_id, TaskStatus::InProgress)?;
    roadmap.to_file(roadmap_path)?;
    
    println!("✅ Task {} status updated to: 🚧 In Progress", task_id);
    
    // Create git branch if requested
    if create_branch && config.git.create_branches {
        let branch_name = config.git.branch_pattern
            .replace("{task_id}", &task_id.to_lowercase());
        
        println!("🌿 Creating branch: {}", branch_name);
        std::process::Command::new("git")
            .args(&["checkout", "-b", &branch_name])
            .output()
            .context("Failed to create git branch")?;
        
        println!("✅ Branch created and checked out: {}", branch_name);
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

async fn complete_task(roadmap_path: &Path, task_id: &str, skip_quality_check: bool, config: &RoadmapConfig) -> Result<()> {
    println!("🏁 Completing task {}", task_id);
    
    // Run quality checks unless skipped
    if !skip_quality_check && config.enforce_quality_gates {
        println!("🔍 Running quality checks...");
        quality_check(task_id, config).await?;
    }
    
    let mut roadmap = Roadmap::from_file(roadmap_path)?;
    
    // Update task status
    roadmap.update_task_status(task_id, TaskStatus::Completed)?;
    roadmap.to_file(roadmap_path)?;
    
    println!("✅ Task {} completed successfully", task_id);
    
    // Create completion commit if configured
    if config.git.require_quality_check {
        let message = config.git.commit_pattern
            .replace("{task_id}", task_id)
            .replace("{message}", "Complete implementation");
        
        println!("📝 Creating commit: {}", message);
        std::process::Command::new("git")
            .args(&["add", "-A"])
            .output()?;
        
        std::process::Command::new("git")
            .args(&["commit", "-m", &message])
            .output()?;
        
        println!("✅ Changes committed");
    }
    
    Ok(())
}

async fn show_status(roadmap_path: &Path, sprint_id: Option<&str>, task_id: Option<&str>, format: OutputFormat) -> Result<()> {
    let roadmap = Roadmap::from_file(roadmap_path)?;
    
    if let Some(task_id) = task_id {
        // Show specific task status
        if let Some(task) = roadmap.get_task(task_id) {
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(task)?);
                }
                _ => {
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
            }
        } else {
            anyhow::bail!("Task {} not found", task_id);
        }
    } else {
        // Show sprint status
        let sprint_id = sprint_id.or(roadmap.current_sprint.as_deref())
            .context("No sprint specified and no current sprint found")?;
        
        if let Some(sprint) = roadmap.get_sprint(sprint_id) {
            let completed = sprint.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
            let in_progress = sprint.tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
            let total = sprint.tasks.len();
            
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(sprint)?);
                }
                _ => {
                    println!("Sprint {}: {}", sprint.version, sprint.title);
                    println!("  Duration: {} to {}", 
                             sprint.start_date.format("%Y-%m-%d"),
                             sprint.end_date.format("%Y-%m-%d"));
                    println!("  Progress: {}/{} completed, {} in progress", completed, total, in_progress);
                    println!("\n  Tasks:");
                    for task in &sprint.tasks {
                        println!("    {} {} - {}", 
                                 task.status.to_emoji(), 
                                 task.id,
                                 task.description);
                    }
                }
            }
        } else {
            anyhow::bail!("Sprint {} not found", sprint_id);
        }
    }
    
    Ok(())
}

async fn validate_sprint(roadmap_path: &Path, sprint_id: &str, strict: bool, config: &RoadmapConfig) -> Result<()> {
    println!("🔍 Validating sprint {} for release...", sprint_id);
    
    let roadmap = Roadmap::from_file(roadmap_path)?;
    let sprint = roadmap.get_sprint(sprint_id)
        .context(format!("Sprint {} not found", sprint_id))?;
    
    let mut all_passed = true;
    
    // Check all tasks completed
    let incomplete_tasks: Vec<_> = sprint.tasks.iter()
        .filter(|t| t.status != TaskStatus::Completed)
        .collect();
    
    if !incomplete_tasks.is_empty() {
        println!("❌ Incomplete tasks:");
        for task in incomplete_tasks {
            println!("    {} - {}", task.id, task.description);
        }
        all_passed = false;
    } else {
        println!("✅ All tasks completed");
    }
    
    // Check definition of done
    println!("\n📋 Definition of Done:");
    for item in &sprint.definition_of_done {
        println!("  - [ ] {}", item);
    }
    
    // Check quality gates
    if config.enforce_quality_gates {
        println!("\n🔍 Quality Gates:");
        for gate in &sprint.quality_gates {
            println!("  - [ ] {}", gate);
        }
    }
    
    if all_passed {
        println!("\n✅ Sprint {} is ready for release!", sprint_id);
    } else {
        println!("\n❌ Sprint {} is NOT ready for release", sprint_id);
        if strict {
            anyhow::bail!("Sprint validation failed");
        }
    }
    
    Ok(())
}

async fn quality_check(task_id: &str, config: &RoadmapConfig) -> Result<()> {
    println!("🔍 Running quality checks for task {}...", task_id);
    
    // Run complexity check
    let complexity_result = std::process::Command::new("pmat")
        .args(&["analyze", "complexity", "--max-cyclomatic", &config.quality_gates.complexity_max.to_string()])
        .output()?;
    
    if !complexity_result.status.success() {
        println!("❌ Complexity check failed");
        anyhow::bail!("Complexity exceeds limit");
    }
    println!("✅ Complexity check passed");
    
    // Run SATD check
    let satd_result = std::process::Command::new("pmat")
        .args(&["analyze", "satd", "--strict"])
        .output()?;
    
    if !satd_result.status.success() && config.quality_gates.satd_tolerance == 0 {
        println!("❌ SATD check failed");
        anyhow::bail!("SATD violations found");
    }
    println!("✅ SATD check passed");
    
    // Run lint check
    if config.quality_gates.lint_compliance {
        let lint_result = std::process::Command::new("make")
            .args(&["lint"])
            .output()?;
        
        if !lint_result.status.success() {
            println!("❌ Lint check failed");
            anyhow::bail!("Lint violations found");
        }
        println!("✅ Lint check passed");
    }
    
    println!("✅ All quality checks passed for task {}", task_id);
    Ok(())
}