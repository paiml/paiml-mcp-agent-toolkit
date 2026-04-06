// Configuration loading, target discovery, auto-commit, and type conversions.
// Handles JSON config parsing, file discovery with walkdir, priority sorting,
// git auto-commit creation, and ExplainLevel conversion.

async fn load_refactor_config(config_path: &Path) -> anyhow::Result<RefactorConfig> {
    debug_assert!(config_path.exists(), "config_path must exist: {}", config_path.display());
    // Placeholder implementation - would load from TOML file
    println!("📝 Loading config from: {}", config_path.display());
    Ok(RefactorConfig::default())
}

async fn load_refactor_config_json(config_path: &Path) -> anyhow::Result<RefactorConfig> {
    debug_assert!(config_path.exists(), "config_path must exist: {}", config_path.display());
    println!("📝 Loading JSON config from: {}", config_path.display());

    let config_data = tokio::fs::read_to_string(config_path).await?;
    let config: serde_json::Value = serde_json::from_str(&config_data)?;

    // Parse the JSON configuration into RefactorConfig
    let mut refactor_config = RefactorConfig::default();

    if let Some(rules) = config.get("rules") {
        if let Some(target_complexity) = rules
            .get("target_complexity")
            .and_then(serde_json::Value::as_u64)
        {
            refactor_config.target_complexity = target_complexity as u16;
        }
        if let Some(max_function_lines) = rules
            .get("max_function_lines")
            .and_then(serde_json::Value::as_u64)
        {
            refactor_config.max_function_lines = max_function_lines as u32;
        }
        if let Some(remove_satd) = rules
            .get("remove_satd")
            .and_then(serde_json::Value::as_bool)
        {
            refactor_config.remove_satd = remove_satd;
        }
    }

    if let Some(parallel) = config
        .get("parallel_workers")
        .and_then(serde_json::Value::as_u64)
    {
        refactor_config.parallel_workers = parallel as usize;
    }

    if let Some(memory) = config
        .get("memory_limit_mb")
        .and_then(serde_json::Value::as_u64)
    {
        refactor_config.memory_limit_mb = memory as usize;
    }

    if let Some(batch) = config.get("batch_size").and_then(serde_json::Value::as_u64) {
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
    debug_assert!(!_priority_expr.is_empty(), "_priority_expr must not be empty");
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
    debug_assert!(true, "contract: create_auto_commit");
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
        println!("✅ Auto-commit created: {message}");
    } else {
        println!("⚠️  Auto-commit failed");
    }

    Ok(())
}

async fn discover_refactor_targets(project_path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
