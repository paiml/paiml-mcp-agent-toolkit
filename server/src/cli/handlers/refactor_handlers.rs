use crate::cli::{ExplainLevel, RefactorCommands, RefactorMode, RefactorOutputFormat};
use crate::models::refactor::RefactorConfig;
use crate::services::cache::unified_manager::UnifiedCacheManager;
use crate::services::refactor_engine::{EngineMode, UnifiedEngine};
use crate::services::unified_ast_engine::UnifiedAstEngine;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Parameters for the refactor serve command
pub struct RefactorServeParams {
    pub mode: RefactorMode,
    pub config: Option<PathBuf>,
    pub project: PathBuf,
    pub parallel: usize,
    pub memory_limit: usize,
    pub batch_size: usize,
    pub priority: Option<String>,
    pub checkpoint_dir: Option<PathBuf>,
    pub resume: bool,
    pub auto_commit: Option<String>,
    pub max_runtime: Option<u64>,
}

pub async fn route_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
    match refactor_cmd {
        RefactorCommands::Serve {
            refactor_mode,
            config,
            project,
            parallel,
            memory_limit,
            batch_size,
            priority,
            checkpoint_dir,
            resume,
            auto_commit,
            max_runtime,
        } => {
            let params = RefactorServeParams {
                mode: refactor_mode,
                config,
                project,
                parallel,
                memory_limit,
                batch_size,
                priority,
                checkpoint_dir,
                resume,
                auto_commit,
                max_runtime,
            };
            handle_refactor_serve(params).await
        }
        RefactorCommands::Interactive {
            project_path,
            explain,
            checkpoint,
            target_complexity,
            steps,
            config,
        } => {
            handle_refactor_interactive(
                project_path,
                explain,
                checkpoint,
                target_complexity,
                steps,
                config,
            )
            .await
        }
        RefactorCommands::Status { checkpoint, format } => {
            handle_refactor_status(checkpoint, format).await
        }
        RefactorCommands::Resume {
            checkpoint,
            steps,
            explain,
        } => handle_refactor_resume(checkpoint, steps, explain).await,
        RefactorCommands::Auto {
            project_path,
            single_file_mode,
            file,
            max_iterations,
            quality_profile: _,
            format,
            dry_run,
            skip_compilation: _,
            skip_tests: _,
            checkpoint,
            verbose: _,
            exclude,
            include,
            ignore_file,
            test,
            test_name,
            github_issue,
        } => {
            super::refactor_auto_handlers::handle_refactor_auto(
                project_path,
                single_file_mode,
                file,
                format,
                max_iterations,
                checkpoint,
                dry_run,
                false, // ci_mode - use false for interactive mode
                exclude,
                include,
                ignore_file,
                test,
                test_name,
                github_issue,
            )
            .await
        }
        RefactorCommands::Docs {
            project_path,
            include_docs,
            include_root,
            additional_dirs,
            format,
            dry_run,
            temp_patterns,
            status_patterns,
            artifact_patterns,
            custom_patterns,
            min_age_days,
            max_size_mb,
            recursive,
            preserve_patterns,
            output,
            auto_remove,
            backup,
            backup_dir,
            perf,
        } => {
            super::refactor_docs_handlers::handle_refactor_docs(
                project_path,
                include_docs,
                include_root,
                additional_dirs,
                format,
                dry_run,
                temp_patterns,
                status_patterns,
                artifact_patterns,
                custom_patterns,
                min_age_days,
                max_size_mb,
                recursive,
                preserve_patterns,
                output,
                auto_remove,
                backup,
                backup_dir,
                perf,
            )
            .await
        }
    }
}

pub async fn handle_refactor_serve(params: RefactorServeParams) -> anyhow::Result<()> {
    let RefactorServeParams {
        mode,
        config,
        project,
        parallel,
        memory_limit,
        batch_size,
        priority,
        checkpoint_dir,
        resume,
        auto_commit,
        max_runtime,
    } = params;
    println!("🔧 Starting refactor server mode...");
    println!("📁 Project: {}", project.display());
    println!("⚙️  Mode: {:?}", mode);
    println!("🔄 Parallel workers: {}", parallel);
    println!("💾 Memory limit: {}MB", memory_limit);
    println!("📦 Batch size: {} files", batch_size);

    // Load configuration from JSON if provided
    let mut refactor_config = if let Some(config_path) = &config {
        println!("📋 Loading config from: {}", config_path.display());
        load_refactor_config_json(config_path).await?
    } else {
        RefactorConfig::default()
    };

    // Apply command-line overrides
    if let Some(prio) = &priority {
        println!("🎯 Priority expression: {}", prio);
        refactor_config.priority_expression = Some(prio.clone());
    }

    if let Some(commit_template) = &auto_commit {
        println!("🔗 Auto-commit template: {}", commit_template);
        refactor_config.auto_commit_template = Some(commit_template.clone());
    }

    refactor_config.parallel_workers = parallel;
    refactor_config.memory_limit_mb = memory_limit;
    refactor_config.batch_size = batch_size;

    // Handle checkpoint directory
    let checkpoint_path = checkpoint_dir.unwrap_or_else(|| project.join(".refactor_checkpoints"));

    if resume {
        println!(
            "🔄 Resuming from checkpoint in: {}",
            checkpoint_path.display()
        );
    } else {
        // Create checkpoint directory if it doesn't exist
        tokio::fs::create_dir_all(&checkpoint_path).await?;
    }

    // Create cache and AST engine
    let cache_config = crate::services::cache::unified::UnifiedCacheConfig {
        max_memory_bytes: (memory_limit / 2) * 1024 * 1024, // Use half the memory for cache (convert MB to bytes)
        ..Default::default()
    };
    let cache = Arc::new(UnifiedCacheManager::new(cache_config)?);
    let ast_engine = Arc::new(UnifiedAstEngine::new());

    // Setup engine mode based on refactor mode
    let engine_mode = match mode {
        RefactorMode::Batch => {
            // For batch mode, we'll use a state machine approach
            EngineMode::Batch {
                checkpoint_dir: checkpoint_path,
                resume,
                parallel_workers: parallel,
            }
        }
        RefactorMode::Interactive => {
            // For interactive mode, fallback to the existing interactive mode
            EngineMode::Interactive {
                checkpoint_file: checkpoint_path.join("interactive_state.json"),
                explain_level: crate::services::refactor_engine::ExplainLevel::Detailed,
            }
        }
    };

    // Discover targets with priority sorting if specified
    let mut targets = discover_refactor_targets(&project).await?;

    if let Some(priority_expr) = &refactor_config.priority_expression {
        println!(
            "🔀 Sorting {} targets by priority expression",
            targets.len()
        );
        // In a real implementation, we would evaluate the priority expression
        // For now, we'll just note that this would be done
        targets = sort_targets_by_priority(targets, priority_expr).await?;
    }

    println!("🎯 Found {} refactoring targets", targets.len());

    // Apply runtime limit if specified
    let start_time = std::time::Instant::now();
    let runtime_limit = max_runtime.map(Duration::from_secs);

    // Create and run engine
    let mut engine = UnifiedEngine::new(
        ast_engine,
        cache,
        engine_mode,
        refactor_config.clone(),
        targets,
    );

    // Run with runtime monitoring
    let summary = if let Some(limit) = runtime_limit {
        println!("⏱️  Maximum runtime: {} seconds", limit.as_secs());

        // Run with timeout
        let result = tokio::time::timeout(limit, engine.run()).await;

        match result {
            Ok(summary) => summary?,
            Err(_) => {
                println!("⏰ Runtime limit reached, saving checkpoint...");
                engine.save_checkpoint().await?;
                return Ok(());
            }
        }
    } else {
        engine.run().await?
    };

    // Print summary
    println!("\n✅ Refactor server completed:");
    println!("   Files processed: {}", summary.files_processed);
    println!("   Refactors applied: {}", summary.refactors_applied);
    println!(
        "   Complexity reduction: {:.2}%",
        summary.complexity_reduction
    );
    println!("   SATD removed: {}", summary.satd_removed);
    println!("   Runtime: {:.2}s", start_time.elapsed().as_secs_f64());

    // Auto-commit if configured
    if let Some(commit_template) = &refactor_config.auto_commit_template {
        if summary.refactors_applied > 0 {
            println!("\n📝 Creating auto-commit...");
            create_auto_commit(commit_template, &summary).await?;
        }
    }

    Ok(())
}

pub async fn handle_refactor_interactive(
    project_path: PathBuf,
    explain: ExplainLevel,
    checkpoint: PathBuf,
    target_complexity: u16,
    steps: Option<u32>,
    config: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!("🤖 Starting interactive refactor mode...");
    println!("📁 Project path: {}", project_path.display());
    println!("💾 Checkpoint: {}", checkpoint.display());
    println!("🎯 Target complexity: {}", target_complexity);
    println!("📝 Explanation level: {:?}", explain);

    // Load configuration
    let refactor_config = if let Some(config_path) = config {
        load_refactor_config(&config_path).await?
    } else {
        RefactorConfig {
            target_complexity,
            ..Default::default()
        }
    };

    // Create cache and AST engine
    let cache_config = crate::services::cache::unified::UnifiedCacheConfig::default();
    let cache = Arc::new(UnifiedCacheManager::new(cache_config)?);
    let ast_engine = Arc::new(UnifiedAstEngine::new());

    // Setup interactive mode
    let mode = EngineMode::Interactive {
        checkpoint_file: checkpoint,
        explain_level: explain.into(),
    };

    // Discover targets
    let targets = discover_refactor_targets(&project_path).await?;
    println!("🎯 Found {} refactoring targets", targets.len());

    // Create and run engine
    let mut engine = UnifiedEngine::new(ast_engine, cache, mode, refactor_config, targets);

    if let Some(max_steps) = steps {
        println!("⏱️  Maximum steps: {}", max_steps);
    }

    let summary = engine.run().await?;

    println!("✅ Interactive refactor completed:");
    println!("   Files processed: {}", summary.files_processed);
    println!("   Refactors applied: {}", summary.refactors_applied);

    Ok(())
}

pub async fn handle_refactor_status(
    checkpoint: PathBuf,
    format: RefactorOutputFormat,
) -> anyhow::Result<()> {
    println!("📊 Reading refactor status from: {}", checkpoint.display());

    // Try to read checkpoint file
    if !checkpoint.exists() {
        return Err(anyhow::anyhow!(
            "Checkpoint file not found: {}",
            checkpoint.display()
        ));
    }

    let checkpoint_data = tokio::fs::read_to_string(&checkpoint).await?;

    match format {
        RefactorOutputFormat::Json => {
            // Validate and pretty-print JSON
            let parsed: serde_json::Value = serde_json::from_str(&checkpoint_data)?;
            println!("{}", serde_json::to_string_pretty(&parsed)?);
        }
        RefactorOutputFormat::Table => {
            // Parse and display as table
            let state: serde_json::Value = serde_json::from_str(&checkpoint_data)?;
            println!("┌─────────────────┬──────────────────────────────────────┐");
            println!("│ Property        │ Value                                │");
            println!("├─────────────────┼──────────────────────────────────────┤");

            if let Some(current) = state.get("current") {
                println!(
                    "│ Current State   │ {:36} │",
                    format!("{:?}", current)
                        .chars()
                        .take(36)
                        .collect::<String>()
                );
            }

            if let Some(targets) = state.get("targets") {
                if let Some(targets_array) = targets.as_array() {
                    println!("│ Target Count    │ {:36} │", targets_array.len());
                }
            }

            if let Some(index) = state.get("current_target_index") {
                println!("│ Current Index   │ {:36} │", index.as_u64().unwrap_or(0));
            }

            println!("└─────────────────┴──────────────────────────────────────┘");
        }
        RefactorOutputFormat::Summary => {
            let state: serde_json::Value = serde_json::from_str(&checkpoint_data)?;
            println!("🔧 Refactor Status Summary");
            println!("   Checkpoint: {}", checkpoint.display());

            if let Some(current) = state.get("current") {
                println!("   Current state: {:?}", current);
            }

            if let Some(targets) = state.get("targets") {
                if let Some(targets_array) = targets.as_array() {
                    println!("   Total targets: {}", targets_array.len());
                }
            }
        }
    }

    Ok(())
}

pub async fn handle_refactor_resume(
    checkpoint: PathBuf,
    steps: u32,
    explain: Option<ExplainLevel>,
) -> anyhow::Result<()> {
    println!("🔄 Resuming refactor from: {}", checkpoint.display());
    println!("⏱️  Maximum steps: {}", steps);

    if !checkpoint.exists() {
        return Err(anyhow::anyhow!(
            "Checkpoint file not found: {}",
            checkpoint.display()
        ));
    }

    // Load the state machine from checkpoint
    let checkpoint_data = tokio::fs::read_to_string(&checkpoint).await?;
    let _state: serde_json::Value = serde_json::from_str(&checkpoint_data)?;

    // This would resume with the loaded state
    println!("📝 State loaded successfully");

    if let Some(explain_level) = explain {
        println!("📖 Explanation level override: {:?}", explain_level);
    }

    // Placeholder implementation
    println!("⚠️  Resume functionality not yet fully implemented");
    println!(
        "   This would continue from the saved state for {} steps",
        steps
    );

    Ok(())
}

async fn load_refactor_config(config_path: &Path) -> anyhow::Result<RefactorConfig> {
    // Placeholder implementation - would load from TOML file
    println!("📝 Loading config from: {}", config_path.display());
    Ok(RefactorConfig::default())
}

async fn load_refactor_config_json(config_path: &Path) -> anyhow::Result<RefactorConfig> {
    println!("📝 Loading JSON config from: {}", config_path.display());

    let config_data = tokio::fs::read_to_string(config_path).await?;
    let config: serde_json::Value = serde_json::from_str(&config_data)?;

    // Parse the JSON configuration into RefactorConfig
    let mut refactor_config = RefactorConfig::default();

    if let Some(rules) = config.get("rules") {
        if let Some(target_complexity) = rules.get("target_complexity").and_then(|v| v.as_u64()) {
            refactor_config.target_complexity = target_complexity as u16;
        }
        if let Some(max_function_lines) = rules.get("max_function_lines").and_then(|v| v.as_u64()) {
            refactor_config.max_function_lines = max_function_lines as u32;
        }
        if let Some(remove_satd) = rules.get("remove_satd").and_then(|v| v.as_bool()) {
            refactor_config.remove_satd = remove_satd;
        }
    }

    if let Some(parallel) = config.get("parallel_workers").and_then(|v| v.as_u64()) {
        refactor_config.parallel_workers = parallel as usize;
    }

    if let Some(memory) = config.get("memory_limit_mb").and_then(|v| v.as_u64()) {
        refactor_config.memory_limit_mb = memory as usize;
    }

    if let Some(batch) = config.get("batch_size").and_then(|v| v.as_u64()) {
        refactor_config.batch_size = batch as usize;
    }

    if let Some(priority) = config.get("priority_expression").and_then(|v| v.as_str()) {
        refactor_config.priority_expression = Some(priority.to_string());
    }

    if let Some(auto_commit) = config.get("auto_commit_template").and_then(|v| v.as_str()) {
        refactor_config.auto_commit_template = Some(auto_commit.to_string());
    }

    Ok(refactor_config)
}

async fn sort_targets_by_priority(
    mut targets: Vec<PathBuf>,
    _priority_expr: &str,
) -> anyhow::Result<Vec<PathBuf>> {
    // In a real implementation, this would:
    // 1. Analyze each file to get metrics (complexity, defect_probability, etc.)
    // 2. Evaluate the priority expression for each file
    // 3. Sort by the resulting priority score

    // For now, just reverse the order as a placeholder
    targets.reverse();
    Ok(targets)
}

async fn create_auto_commit(
    template: &str,
    summary: &crate::models::refactor::Summary,
) -> anyhow::Result<()> {
    use std::process::Command;

    // Stage all changes
    let status = Command::new("git").args(["add", "-A"]).status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to stage changes"));
    }

    // Format the commit message using the template
    let message = template
        .replace("{files}", &summary.files_processed.to_string())
        .replace("{refactors}", &summary.refactors_applied.to_string())
        .replace(
            "{complexity_reduction}",
            &format!("{:.1}%", summary.complexity_reduction),
        )
        .replace("{satd_removed}", &summary.satd_removed.to_string());

    // Create the commit
    let status = Command::new("git")
        .args(["commit", "-m", &message])
        .status()?;

    if status.success() {
        println!("✅ Auto-commit created: {}", message);
    } else {
        println!("⚠️  Auto-commit failed");
    }

    Ok(())
}

async fn discover_refactor_targets(project_path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    // Placeholder implementation - would discover files that need refactoring
    let mut targets = Vec::new();

    // Add some common patterns for now
    let extensions = ["rs", "ts", "tsx", "js", "jsx", "py"];

    for entry in walkdir::WalkDir::new(project_path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if extensions.contains(&ext.to_string_lossy().as_ref()) {
                    targets.push(entry.path().to_path_buf());
                }
            }
        }
    }

    Ok(targets)
}

impl From<ExplainLevel> for crate::services::refactor_engine::ExplainLevel {
    fn from(level: ExplainLevel) -> Self {
        match level {
            ExplainLevel::Brief => crate::services::refactor_engine::ExplainLevel::Brief,
            ExplainLevel::Detailed => crate::services::refactor_engine::ExplainLevel::Detailed,
            ExplainLevel::Verbose => crate::services::refactor_engine::ExplainLevel::Verbose,
        }
    }
}
