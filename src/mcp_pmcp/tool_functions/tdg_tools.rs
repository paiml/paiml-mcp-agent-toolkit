/// Analyze Technical Debt Grading (TDG) scores using the new TDG implementation
pub async fn analyze_tdg(
    paths: &[PathBuf],
    threshold: Option<f64>,
    top_files: Option<usize>,
    include_components: Option<bool>,
    with_git_context: Option<bool>, // Sprint 65: Git-commit correlation
) -> Result<Value> {
    use crate::tdg::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let mut analyzer = TdgAnalyzer::new()?;
    let _threshold = threshold.unwrap_or(1.5);
    let _top_files = top_files.unwrap_or(10);
    let _include_components = include_components.unwrap_or(false);

    // Sprint 65: Extract git context if requested
    if with_git_context.unwrap_or(false) && !paths.is_empty() {
        let git_context = crate::models::git_context::GitContext::try_from_current_dir(&paths[0]);
        analyzer.set_git_context(git_context);
    }

    if paths.len() == 1 {
        analyze_single_tdg_path(&analyzer, &paths[0]).await
    } else {
        analyze_multiple_tdg_paths(&analyzer, paths).await
    }
}

async fn analyze_single_tdg_path(analyzer: &crate::tdg::TdgAnalyzer, path: &Path) -> Result<Value> {
    // Sprint 65: Get git context from analyzer for output
    let git_context = analyzer.get_git_context();

    if PathValidator::ensure_directory(path).is_ok() {
        let project_score = analyzer.analyze_project(path).await?;
        Ok(json!({
            "status": "completed",
            "message": "TDG project analysis completed",
            "result_type": "project",
            "results": {
                "average_score": project_score.average_score,
                "average_grade": project_score.average_grade,
                "total_files": project_score.total_files,
                "language_distribution": project_score.language_distribution,
                "files": project_score.files
            },
            "git_context": git_context.map(|git| json!({
                "commit_sha": git.commit_sha,
                "commit_sha_short": git.commit_sha_short,
                "branch": git.branch,
                "author_name": git.author_name,
                "author_email": git.author_email,
                "commit_timestamp": git.commit_timestamp.to_rfc3339(),
                "commit_message": git.commit_message,
                "tags": git.tags,
                "is_clean": git.is_clean,
                "uncommitted_files": git.uncommitted_files,
            }))
        }))
    } else {
        let score = analyzer.analyze_file(path).await?;
        Ok(json!({
            "status": "completed",
            "message": "TDG file analysis completed",
            "result_type": "file",
            "results": score,
            "git_context": git_context.map(|git| json!({
                "commit_sha": git.commit_sha,
                "commit_sha_short": git.commit_sha_short,
                "branch": git.branch,
                "author_name": git.author_name,
                "author_email": git.author_email,
                "commit_timestamp": git.commit_timestamp.to_rfc3339(),
                "commit_message": git.commit_message,
                "tags": git.tags,
                "is_clean": git.is_clean,
                "uncommitted_files": git.uncommitted_files,
            }))
        }))
    }
}

async fn analyze_multiple_tdg_paths(
    analyzer: &crate::tdg::TdgAnalyzer,
    paths: &[PathBuf],
) -> Result<Value> {
    use crate::tdg::ProjectScore;
    let mut all_scores = Vec::new();

    // Sprint 65: Get git context from analyzer for output
    let git_context = analyzer.get_git_context();

    for path in paths {
        if PathValidator::ensure_directory(path).is_ok() {
            let project_score = analyzer.analyze_project(path).await?;
            all_scores.extend(project_score.files);
        } else {
            let score = analyzer.analyze_file(path).await?;
            all_scores.push(score);
        }
    }

    let aggregated = ProjectScore::aggregate(all_scores);
    Ok(json!({
        "status": "completed",
        "message": "TDG multi-path analysis completed",
        "result_type": "multi_path",
        "results": {
            "average_score": aggregated.average_score,
            "average_grade": aggregated.average_grade,
            "total_files": aggregated.total_files,
            "language_distribution": aggregated.language_distribution,
            "files": aggregated.files
        },
        "git_context": git_context.map(|git| json!({
            "commit_sha": git.commit_sha,
            "commit_sha_short": git.commit_sha_short,
            "branch": git.branch,
            "author_name": git.author_name,
            "author_email": git.author_email,
            "commit_timestamp": git.commit_timestamp.to_rfc3339(),
            "commit_message": git.commit_message,
            "tags": git.tags,
            "is_clean": git.is_clean,
            "uncommitted_files": git.uncommitted_files,
        }))
    }))
}

/// Compare TDG scores between two files or directories
pub async fn compare_tdg(
    path1: &Path,
    path2: &Path,
    with_git_context: Option<bool>, // Sprint 65: Git-commit correlation
) -> Result<Value> {
    use crate::tdg::TdgAnalyzer;

    let mut analyzer = TdgAnalyzer::new()?;

    // Sprint 65: Extract git context if requested (uses first path as reference)
    if with_git_context.unwrap_or(false) {
        let git_context = crate::models::git_context::GitContext::try_from_current_dir(path1);
        analyzer.set_git_context(git_context.clone());
    }

    let comparison = analyzer.compare(path1, path2).await?;
    let git_context = analyzer.get_git_context();

    Ok(json!({
        "status": "completed",
        "message": "TDG comparison completed",
        "result_type": "comparison",
        "results": comparison,
        "git_context": git_context.map(|git| json!({
            "commit_sha": git.commit_sha,
            "commit_sha_short": git.commit_sha_short,
            "branch": git.branch,
            "author_name": git.author_name,
            "author_email": git.author_email,
            "commit_timestamp": git.commit_timestamp.to_rfc3339(),
            "commit_message": git.commit_message,
            "tags": git.tags,
            "is_clean": git.is_clean,
            "uncommitted_files": git.uncommitted_files,
        }))
    }))
}

// ==================== SPRINT 30 TDG SYSTEM MCP TOOLS ====================

/// Get comprehensive TDG system diagnostics
pub async fn tdg_system_diagnostics(
    detailed: bool,
    components: Vec<String>, // ["storage", "scheduler", "adaptive", "resources"]
) -> Result<Value> {
    let base_path = PathBuf::from(".");

    // Create diagnostic command
    let show_all = components.contains(&"all".to_string()) || components.is_empty();
    let command = TdgCommand::Diagnostics {
        detailed,
        storage: show_all || components.contains(&"storage".to_string()),
        scheduler: show_all || components.contains(&"scheduler".to_string()),
        adaptive: show_all || components.contains(&"adaptive".to_string()),
        resources: show_all || components.contains(&"resources".to_string()),
        all: show_all,
        format: DiagnosticOutputFormat::Json,
    };

    // Execute diagnostics
    match tdg_diagnostic_handler::handle_tdg_diagnostics(&command, &base_path).await {
        Ok(()) => Ok(json!({
            "status": "completed",
            "message": "TDG system diagnostics completed",
            "result_type": "diagnostics",
            "components_checked": if show_all {
                vec!["storage", "scheduler", "adaptive", "resources"]
            } else {
                components.iter().map(std::string::String::as_str).collect::<Vec<&str>>()
            },
            "detailed": detailed
        })),
        Err(e) => Ok(json!({
            "status": "error",
            "message": format!("Diagnostics failed: {}", e),
            "error": e.to_string()
        })),
    }
}

/// Get TDG storage statistics and management
pub async fn tdg_storage_management(
    action: String, // "stats", "cleanup", "flush", "migrate"
    options: Value,
) -> Result<Value> {
    let base_path = PathBuf::from(".");

    let storage_command = match action.as_str() {
        "stats" => StorageCommand::Stats {
            detailed: options
                .get("detailed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        },
        "cleanup" => StorageCommand::Cleanup {
            max_age: options
                .get("max_age")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(3600),
        },
        "flush" => StorageCommand::Flush,
        "migrate" => StorageCommand::Migrate {
            backend: options
                .get("backend")
                .and_then(|v| v.as_str())
                .unwrap_or("sled")
                .to_string(),
            path: options
                .get("path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from),
        },
        _ => {
            return Ok(json!({
                "status": "error",
                "message": format!("Unknown storage action: {}", action),
                "valid_actions": ["stats", "cleanup", "flush", "migrate"]
            }))
        }
    };

    let command = TdgCommand::Storage {
        command: storage_command,
    };

    match tdg_diagnostic_handler::handle_tdg_diagnostics(&command, &base_path).await {
        Ok(()) => Ok(json!({
            "status": "completed",
            "message": format!("Storage {} completed successfully", action),
            "result_type": "storage_management",
            "action": action,
            "options": options
        })),
        Err(e) => Ok(json!({
            "status": "error",
            "message": format!("Storage {} failed: {}", action, e),
            "error": e.to_string()
        })),
    }
}

/// Analyze files with TDG transactional storage
pub async fn tdg_analyze_with_storage(
    paths: Vec<PathBuf>,
    storage_backend: Option<String>, // "sled", "rocksdb", "inmemory"
    _priority: Option<String>,       // "critical", "high", "medium", "low"
) -> Result<Value> {
    let storage = create_storage_backend(storage_backend.as_deref())?;
    let analyzer = TdgAnalyzer::new()?;

    let analysis_results = analyze_paths_with_storage(paths, &analyzer, storage.as_ref()).await?;

    let storage_stats = storage.as_ref().get_stats();

    build_analysis_response(analysis_results, storage_backend, storage_stats)
}

/// Create storage backend based on the provided backend type
fn create_storage_backend(
    backend_type: Option<&str>,
) -> Result<Box<dyn crate::tdg::storage_backend::StorageBackend>> {
    match backend_type {
        Some("inmemory") => {
            use crate::tdg::storage_backend::InMemoryBackend;
            Ok(Box::new(InMemoryBackend::new()))
        }
        Some("libsql") | None => {
            // Default to libsql (modern SQLite-compatible database)
            use crate::tdg::storage_backend::LibsqlBackend;
            let temp_path = std::env::temp_dir().join("tdg-mcp-libsql.db");
            Ok(Box::new(LibsqlBackend::new(&temp_path)?))
        }
        Some(backend) => Err(anyhow::anyhow!(
            "Unsupported storage backend: {backend}. Supported: libsql (default), inmemory"
        )),
    }
}

/// Analysis results container
struct AnalysisResults {
    results: Vec<Value>,
    total_files: u32,
    avg_score: f32,
}

/// Analyze all paths with storage
async fn analyze_paths_with_storage(
    paths: Vec<PathBuf>,
    analyzer: &TdgAnalyzer,
    storage: &dyn crate::tdg::storage_backend::StorageBackend,
) -> Result<AnalysisResults> {
    let mut results = Vec::new();
    let mut total_files = 0;
    let mut avg_score = 0.0;

    for path in paths {
        let analysis_result = analyze_single_path(&path, analyzer).await;

        match analysis_result {
            Ok(project_score) => {
                total_files += project_score.total_files;
                avg_score += project_score.average_score;

                store_project_results(&project_score, storage).await;

                let result_json = create_success_result(&path, &project_score);
                results.push(result_json);
            }
            Err(e) => {
                let error_result = create_error_result(&path, &e);
                results.push(error_result);
            }
        }
    }

    if total_files > 0 {
        avg_score /= results.len() as f32;
    }

    Ok(AnalysisResults {
        results,
        total_files: total_files.try_into().unwrap_or(0),
        avg_score,
    })
}

/// Analyze a single path (file or directory)
async fn analyze_single_path(
    path: &Path,
    analyzer: &TdgAnalyzer,
) -> Result<crate::tdg::ProjectScore> {
    if PathValidator::ensure_directory(path).is_ok() {
        analyzer.analyze_project(path).await
    } else {
        analyzer.analyze_file(path).await.map(|score| {
            use crate::tdg::ProjectScore;
            ProjectScore::aggregate(vec![score])
        })
    }
}

/// Store project analysis results in TDG storage
async fn store_project_results(
    project_score: &crate::tdg::ProjectScore,
    storage: &dyn crate::tdg::storage_backend::StorageBackend,
) {
    for file_score in &project_score.files {
        if let Some(file_path) = &file_score.file_path {
            if let Ok(record) = create_tdg_record(file_path, file_score) {
                // Convert record to key/value for storage
                let key = file_path.to_string_lossy().as_bytes().to_vec();
                if let Ok(value) = serde_json::to_vec(&record) {
                    if let Err(e) = storage.put(&key, &value) {
                        eprintln!("Warning: Failed to store TDG record: {e}");
                    }
                }
            }
        }
    }
}

/// Create a TDG record for storage
fn create_tdg_record(
    file_path: &Path,
    file_score: &crate::tdg::TdgScore,
) -> Result<crate::tdg::FullTdgRecord> {
    let content = std::fs::read(file_path).unwrap_or_default();
    let hash = blake3::hash(&content);

    Ok(crate::tdg::FullTdgRecord {
        identity: create_file_identity(file_path, &hash, &content),
        score: file_score.clone(),
        components: create_component_scores(),
        semantic_sig: create_semantic_signature(&hash),
        metadata: create_analysis_metadata(file_score),
        git_context: None, // MCP tool doesn't collect git context
    })
}

/// Create file identity for TDG record
fn create_file_identity(
    file_path: &Path,
    hash: &blake3::Hash,
    content: &[u8],
) -> crate::tdg::FileIdentity {
    crate::tdg::FileIdentity {
        path: file_path.to_path_buf(),
        content_hash: *hash,
        size_bytes: content.len() as u64,
        modified_time: std::time::SystemTime::now(),
    }
}

/// Create component scores for TDG record
fn create_component_scores() -> crate::tdg::ComponentScores {
    crate::tdg::ComponentScores {
        complexity_breakdown: std::collections::HashMap::new(),
        duplication_sources: Vec::new(),
        coupling_dependencies: Vec::new(),
        doc_missing_items: Vec::new(),
        consistency_violations: Vec::new(),
    }
}

/// Create semantic signature for TDG record
fn create_semantic_signature(hash: &blake3::Hash) -> crate::tdg::SemanticSignature {
    crate::tdg::SemanticSignature {
        ast_structure_hash: hash.as_bytes()[0..8]
            .iter()
            .fold(0u64, |acc, &b| acc.wrapping_mul(256) + u64::from(b)),
        identifier_pattern: "mcp_analysis".to_string(),
        control_flow_pattern: "function_call".to_string(),
        import_dependencies: Vec::new(),
    }
}

/// Create analysis metadata for TDG record
fn create_analysis_metadata(file_score: &crate::tdg::TdgScore) -> crate::tdg::AnalysisMetadata {
    crate::tdg::AnalysisMetadata {
        analyzer_version: "2.38.0-mcp".to_string(),
        analysis_duration_ms: 10,
        language_confidence: file_score.confidence,
        analysis_timestamp: std::time::SystemTime::now(),
        cache_hit: false,
    }
}

/// Create success result JSON
fn create_success_result(path: &Path, project_score: &crate::tdg::ProjectScore) -> Value {
    json!({
        "path": path.display().to_string(),
        "total_files": project_score.total_files,
        "average_score": project_score.average_score,
        "average_grade": format!("{}", project_score.average_grade),
        "language_distribution": project_score.language_distribution,
    })
}

/// Create error result JSON
fn create_error_result(path: &Path, error: &anyhow::Error) -> Value {
    json!({
        "path": path.display().to_string(),
        "error": error.to_string(),
        "status": "failed"
    })
}

/// Build final analysis response
fn build_analysis_response(
    analysis_results: AnalysisResults,
    storage_backend: Option<String>,
    storage_stats: HashMap<String, String>,
) -> Result<Value> {
    Ok(json!({
        "status": "completed",
        "message": "TDG analysis with transactional storage completed",
        "result_type": "tdg_analysis_storage",
        "summary": {
            "total_files_analyzed": analysis_results.total_files,
            "average_score": analysis_results.avg_score,
            "storage_backend": storage_backend.unwrap_or("sled".to_string()),
            "storage_stats": storage_stats
        },
        "results": analysis_results.results
    }))
}

/// Get TDG performance metrics and adaptive threshold status
pub async fn tdg_performance_metrics() -> Result<Value> {
    // Create adaptive threshold manager to get performance stats
    let adaptive = AdaptiveThresholdFactory::create_default();
    let thresholds = adaptive.get_current_thresholds().await;
    let performance = adaptive.get_performance_stats().await;

    // Create scheduler for scheduling stats
    let scheduler = SchedulerFactory::create_balanced();
    let scheduler_stats = scheduler.get_statistics().await;

    Ok(json!({
        "status": "completed",
        "message": "TDG performance metrics retrieved",
        "result_type": "performance_metrics",
        "adaptive_thresholds": {
            "hot_cache_size": thresholds.hot_cache_size,
            "compression_level": thresholds.compression_level,
            "high_priority_permits": thresholds.high_priority_permits,
            "low_priority_permits": thresholds.low_priority_permits,
        },
        "performance_stats": {
            "avg_analysis_duration_ms": performance.avg_analysis_duration_ms,
            "avg_cache_hit_ratio": performance.avg_cache_hit_ratio,
            "avg_memory_usage_mb": performance.avg_memory_usage_mb,
            "avg_cpu_utilization": performance.avg_cpu_utilization,
            "total_samples": performance.total_samples,
        },
        "scheduler_stats": {
            "high_permits_available": scheduler_stats.high_permits_available,
            "low_permits_available": scheduler_stats.low_permits_available,
            "active_commits": scheduler_stats.active_commits,
            "active_background": scheduler_stats.active_background,
            "avg_wait_time_ms": scheduler_stats.avg_wait_time_ms,
            "total_active_operations": scheduler_stats.total_active_operations,
        }
    }))
}

/// Configure TDG storage backend and create optimized setup
pub async fn tdg_configure_storage(
    backend_type: String,
    path: Option<String>,
    cache_size_mb: Option<u32>,
    compression: Option<bool>,
) -> Result<Value> {
    let backend_enum = match backend_type.as_str() {
        "libsql" => StorageBackendType::Libsql,
        "inmemory" => StorageBackendType::InMemory,
        _ => {
            return Ok(json!({
                "status": "error",
                "message": format!("Unsupported backend type: {}", backend_type),
                "supported_types": ["libsql", "inmemory"]
            }))
        }
    };

    let config = StorageConfig {
        backend_type: backend_enum,
        path: path.clone().map(PathBuf::from),
        cache_size_mb,
        compression: compression.unwrap_or(true),
    };

    // Test the configuration by creating a storage instance
    match crate::tdg::StorageBackendFactory::create_from_config(&config) {
        Ok(backend) => {
            let stats = backend.get_stats();
            Ok(json!({
                "status": "completed",
                "message": "Storage backend configuration validated",
                "result_type": "storage_config",
                "configuration": {
                    "backend_type": backend_type,
                    "backend_name": backend.backend_name(),
                    "path": path,
                    "cache_size_mb": cache_size_mb,
                    "compression": compression.unwrap_or(true),
                },
                "backend_stats": stats,
                "validation": "success"
            }))
        }
        Err(e) => Ok(json!({
            "status": "error",
            "message": format!("Storage configuration validation failed: {}", e),
            "configuration": config,
            "error": e.to_string()
        })),
    }
}

/// Get TDG system health status with recommendations
pub async fn tdg_health_check() -> Result<Value> {
    let mut health_issues = Vec::new();
    let mut recommendations = Vec::new();
    let mut overall_status = "healthy".to_string();

    // Check storage health
    match TieredStorageFactory::create_default() {
        Ok(storage) => {
            let stats = storage.get_statistics();
            if stats.hot_memory_kb > 100_000 {
                // > 100MB
                health_issues.push("High hot cache memory usage detected".to_string());
                recommendations.push(
                    "Consider cleaning up hot cache or increasing archival frequency".to_string(),
                );
            }
            if stats.compression_ratio > 0.9 {
                health_issues.push("Low compression ratio detected".to_string());
                recommendations
                    .push("Consider different compression settings or backend".to_string());
            }
        }
        Err(e) => {
            health_issues.push(format!("Storage system unavailable: {e}"));
            overall_status = "critical".to_string();
        }
    }

    // Check scheduler health
    let scheduler = SchedulerFactory::create_balanced();
    let scheduler_stats = scheduler.get_statistics().await;
    if scheduler_stats.avg_wait_time_ms > 1000 {
        health_issues.push("High scheduler wait times detected".to_string());
        recommendations
            .push("Consider increasing scheduler permits or optimizing workload".to_string());
    }

    // Check adaptive thresholds health
    let adaptive = AdaptiveThresholdFactory::create_default();
    let performance = adaptive.get_performance_stats().await;
    if performance.avg_cache_hit_ratio < 0.7 {
        health_issues.push("Low cache hit ratio detected".to_string());
        recommendations
            .push("Consider increasing cache size or reviewing access patterns".to_string());
    }

    if !health_issues.is_empty() && overall_status == "healthy" {
        overall_status = "warning".to_string();
    }

    Ok(json!({
        "status": "completed",
        "message": "TDG system health check completed",
        "result_type": "health_check",
        "overall_status": overall_status,
        "health_score": if overall_status == "healthy" { 100 } else if overall_status == "warning" { 75 } else { 25 },
        "issues": health_issues,
        "recommendations": recommendations,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "storage": if health_issues.iter().any(|i| i.contains("Storage")) { "warning" } else { "healthy" },
            "scheduler": if health_issues.iter().any(|i| i.contains("scheduler")) { "warning" } else { "healthy" },
            "adaptive": if health_issues.iter().any(|i| i.contains("Adaptive")) { "warning" } else { "healthy" }
        }
    }))
}
