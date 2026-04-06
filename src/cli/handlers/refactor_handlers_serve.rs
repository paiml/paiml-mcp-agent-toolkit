// Refactor serve command handler and its helper functions.
// Handles the `pmat refactor serve` workflow including parameter extraction,
// configuration setup, checkpoint management, engine execution, and auto-commit.

/// Extracted parameters struct for cleaner parameter passing
struct ExtractedRefactorParams {
    mode: RefactorMode,
    config: Option<PathBuf>,
    project: PathBuf,
    parallel: usize,
    memory_limit: usize,
    batch_size: usize,
    priority: Option<String>,
    checkpoint_dir: Option<PathBuf>,
    resume: bool,
    auto_commit: Option<String>,
    max_runtime: Option<u64>,
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_refactor_serve(params: RefactorServeParams) -> anyhow::Result<()> {
    let extracted_params = extract_refactor_params(params);
    log_refactor_server_startup(&extracted_params);

    let refactor_config = setup_refactor_configuration(&extracted_params).await?;
    let checkpoint_path = setup_checkpoint_directory(&extracted_params).await?;
    let (cache, ast_engine) = setup_cache_and_ast_engine(extracted_params.memory_limit)?;
    let engine_mode = create_engine_mode(&extracted_params, &checkpoint_path);
    let targets = discover_and_prioritize_targets(&extracted_params, &refactor_config).await?;

    let summary = execute_refactor_engine(
        ast_engine,
        cache,
        engine_mode,
        refactor_config.clone(),
        targets,
        extracted_params.max_runtime,
    )
    .await?;

    print_refactor_summary(&summary);
    handle_auto_commit(&refactor_config, &summary).await?;

    Ok(())
}

/// Extract parameters from `RefactorServeParams` into a cleaner struct
fn extract_refactor_params(params: RefactorServeParams) -> ExtractedRefactorParams {
    debug_assert!(true, "contract: extract_refactor_params");
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

    ExtractedRefactorParams {
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
    }
}

/// Log refactor server startup information
fn log_refactor_server_startup(params: &ExtractedRefactorParams) {
    debug_assert!(true, "contract: log_refactor_server_startup");
    println!("🔧 Starting refactor server mode...");
    println!("📁 Project: {}", params.project.display());
    println!("⚙️  Mode: {:?}", params.mode);
    println!("🔄 Parallel workers: {}", params.parallel);
    println!("💾 Memory limit: {}MB", params.memory_limit);
    println!("📦 Batch size: {} files", params.batch_size);
}

/// Setup refactor configuration from file and command-line overrides
async fn setup_refactor_configuration(
    params: &ExtractedRefactorParams,
) -> anyhow::Result<RefactorConfig> {
    debug_assert!(true, "contract: setup_refactor_configuration");
    let mut refactor_config = load_base_configuration(params).await?;
    apply_command_line_overrides(&mut refactor_config, params);
    Ok(refactor_config)
}

/// Load base configuration from JSON file or use defaults
async fn load_base_configuration(
    params: &ExtractedRefactorParams,
) -> anyhow::Result<RefactorConfig> {
    debug_assert!(true, "contract: load_base_configuration");
    if let Some(config_path) = &params.config {
        println!("📋 Loading config from: {}", config_path.display());
        load_refactor_config_json(config_path).await
    } else {
        Ok(RefactorConfig::default())
    }
}

/// Apply command-line parameter overrides to configuration
fn apply_command_line_overrides(config: &mut RefactorConfig, params: &ExtractedRefactorParams) {
    debug_assert!(true, "contract: apply_command_line_overrides");
    if let Some(prio) = &params.priority {
        println!("🎯 Priority expression: {prio}");
        config.priority_expression = Some(prio.clone());
    }

    if let Some(commit_template) = &params.auto_commit {
        println!("🔗 Auto-commit template: {commit_template}");
        config.auto_commit_template = Some(commit_template.clone());
    }

    config.parallel_workers = params.parallel;
    config.memory_limit_mb = params.memory_limit;
    config.batch_size = params.batch_size;
}

/// Setup checkpoint directory for resume functionality
async fn setup_checkpoint_directory(params: &ExtractedRefactorParams) -> anyhow::Result<PathBuf> {
    debug_assert!(true, "contract: setup_checkpoint_directory");
    let checkpoint_path = params
        .checkpoint_dir
        .clone()
        .unwrap_or_else(|| params.project.join(".refactor_checkpoints"));

    if params.resume {
        println!(
            "🔄 Resuming from checkpoint in: {}",
            checkpoint_path.display()
        );
    } else {
        tokio::fs::create_dir_all(&checkpoint_path).await?;
    }

    Ok(checkpoint_path)
}

/// Setup cache and AST engine with proper memory allocation
fn setup_cache_and_ast_engine(
    memory_limit: usize,
) -> anyhow::Result<(Arc<UnifiedCacheManager>, Arc<UnifiedAstEngine>)> {
    debug_assert!(true, "contract: setup_cache_and_ast_engine");
    let cache_config = crate::services::cache::unified::UnifiedCacheConfig {
        max_memory_mb: memory_limit / 2, // Use half the memory for cache
        ..Default::default()
    };
    let cache = Arc::new(UnifiedCacheManager::new(cache_config)?);
    let ast_engine = Arc::new(UnifiedAstEngine::new());
    Ok((cache, ast_engine))
}

/// Create engine mode based on refactor mode and checkpoint path
fn create_engine_mode(params: &ExtractedRefactorParams, checkpoint_path: &Path) -> EngineMode {
    debug_assert!(checkpoint_path.exists(), "checkpoint_path must exist: {}", checkpoint_path.display());
    match params.mode {
        RefactorMode::Batch => EngineMode::Batch {
            checkpoint_dir: checkpoint_path.to_path_buf(),
            resume: params.resume,
            parallel_workers: params.parallel,
        },
        RefactorMode::Interactive => EngineMode::Interactive {
            checkpoint_file: checkpoint_path.join("interactive_state.json"),
            explain_level: crate::services::refactor_engine::ExplainLevel::Detailed,
        },
    }
}

/// Discover targets and apply priority sorting if specified
async fn discover_and_prioritize_targets(
    params: &ExtractedRefactorParams,
    config: &RefactorConfig,
) -> anyhow::Result<Vec<PathBuf>> {
    debug_assert!(true, "contract: discover_and_prioritize_targets");
    let mut targets = discover_refactor_targets(&params.project).await?;

    if let Some(priority_expr) = &config.priority_expression {
        println!(
            "🔀 Sorting {} targets by priority expression",
            targets.len()
        );
        targets = sort_targets_by_priority(targets, priority_expr).await?;
    }

    println!("🎯 Found {} refactoring targets", targets.len());
    Ok(targets)
}

/// Execute refactor engine with runtime monitoring
async fn execute_refactor_engine(
    ast_engine: Arc<UnifiedAstEngine>,
    cache: Arc<UnifiedCacheManager>,
    engine_mode: EngineMode,
    refactor_config: RefactorConfig,
    targets: Vec<PathBuf>,
    max_runtime: Option<u64>,
) -> anyhow::Result<Summary> {
    debug_assert!(true, "contract: execute_refactor_engine");
    let mut engine = UnifiedEngine::new(ast_engine, cache, engine_mode, refactor_config, targets);

    if let Some(runtime_seconds) = max_runtime {
        execute_with_timeout(engine, runtime_seconds).await
    } else {
        engine.run().await.map_err(anyhow::Error::from)
    }
}

/// Execute engine with timeout monitoring
async fn execute_with_timeout(
    mut engine: UnifiedEngine,
    runtime_seconds: u64,
) -> anyhow::Result<Summary> {
    debug_assert!(true, "contract: execute_with_timeout");
    let limit = Duration::from_secs(runtime_seconds);
    println!("⏱️  Maximum runtime: {} seconds", limit.as_secs());

    let result = tokio::time::timeout(limit, engine.run()).await;

    if let Ok(summary) = result {
        summary.map_err(anyhow::Error::from)
    } else {
        println!("⏰ Runtime limit reached, saving checkpoint...");
        engine.save_checkpoint().await?;
        std::process::exit(0);
    }
}

/// Print refactor execution summary
fn print_refactor_summary(summary: &Summary) {
    debug_assert!(true, "contract: print_refactor_summary");
    let start_time = std::time::Instant::now();
    println!("\n✅ Refactor server completed:");
    println!("   Files processed: {}", summary.files_processed);
    println!("   Refactors applied: {}", summary.refactors_applied);
    println!(
        "   Complexity reduction: {:.2}%",
        summary.complexity_reduction
    );
    println!("   SATD removed: {}", summary.satd_removed);
    println!("   Runtime: {:.2}s", start_time.elapsed().as_secs_f64());
}

/// Handle auto-commit if configured and refactors were applied
async fn handle_auto_commit(config: &RefactorConfig, summary: &Summary) -> anyhow::Result<()> {
    debug_assert!(true, "contract: handle_auto_commit");
    if let Some(commit_template) = &config.auto_commit_template {
        if summary.refactors_applied > 0 {
            println!("\n📝 Creating auto-commit...");
            create_auto_commit(commit_template, summary).await?;
        }
    }
    Ok(())
}
