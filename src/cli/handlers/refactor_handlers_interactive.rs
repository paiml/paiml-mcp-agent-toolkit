// Interactive and resume refactor command handlers.
// Handles `pmat refactor interactive` and `pmat refactor resume` workflows.

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_refactor_interactive(
    project_path: PathBuf,
    explain: ExplainLevel,
    checkpoint: PathBuf,
    target_complexity: u16,
    steps: Option<u32>,
    config: Option<PathBuf>,
) -> anyhow::Result<()> {
    crate::status_println!("🤖 Starting interactive refactor mode...");
    crate::status_println!("📁 Project path: {}", project_path.display());
    crate::status_println!("💾 Checkpoint: {}", checkpoint.display());
    crate::status_println!("🎯 Target complexity: {target_complexity}");
    crate::status_println!("📝 Explanation level: {explain:?}");

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
    crate::status_println!("🎯 Found {} refactoring targets", targets.len());

    // Create and run engine
    let mut engine = UnifiedEngine::new(ast_engine, cache, mode, refactor_config, targets);

    if let Some(max_steps) = steps {
        crate::status_println!("⏱️  Maximum steps: {max_steps}");
    }

    let summary = engine.run().await?;

    println!("✅ Interactive refactor completed:");
    println!("   Files processed: {}", summary.files_processed);
    println!("   Refactors applied: {}", summary.refactors_applied);

    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_refactor_resume(
    checkpoint: PathBuf,
    steps: u32,
    explain: Option<ExplainLevel>,
) -> anyhow::Result<()> {
    crate::status_println!("🔄 Resuming refactor from: {}", checkpoint.display());
    crate::status_println!("⏱️  Maximum steps: {steps}");

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
    crate::status_println!("📝 State loaded successfully");

    if let Some(explain_level) = explain {
        crate::status_println!("📖 Explanation level override: {explain_level:?}");
    }

    // Placeholder implementation
    println!("⚠️  Resume functionality not yet fully implemented");
    println!("   This would continue from the saved state for {steps} steps");

    Ok(())
}
