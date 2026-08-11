/// Analyze files with TDG transactional storage
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn create_storage_backend(
    backend_type: Option<&str>,
) -> Result<Box<dyn crate::tdg::storage_backend::StorageBackend>> {
    match backend_type {
        Some("inmemory") => {
            use crate::tdg::storage_backend::InMemoryBackend;
            Ok(Box::new(InMemoryBackend::new()))
        }
        Some("libsql") | None => {
            // Default to libsql (modern SQLite-compatible database).
            // Use a per-call unique path: a single fixed path made parallel
            // callers race on the same SQLite file (concurrent MCP requests, and
            // the libsql/default tests under coverage) → intermittent open
            // failures. Storage here is per-invocation/ephemeral, so a unique
            // file per call is correct and removes the contention.
            use crate::tdg::storage_backend::LibsqlBackend;
            use std::sync::atomic::{AtomicU64, Ordering};
            static DB_SEQ: AtomicU64 = AtomicU64::new(0);
            let seq = DB_SEQ.fetch_add(1, Ordering::Relaxed);
            let temp_path = std::env::temp_dir()
                .join(format!("tdg-mcp-libsql-{}-{seq}.db", std::process::id()));
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
                // GH #704: a path that graded nothing contributes nothing --
                // it used to contribute the 0.0 default of an unmeasured score.
                if let Some(score) = project_score.average_score {
                    avg_score += score;
                }

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
        // GH #704: null, not 0.0/"F", when no file under this path was graded.
        "average_score": project_score.average_score,
        "average_grade": project_score.average_grade.map(|g| g.to_string()),
        "not_measured": project_score.not_measured,
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

/// Get TDG storage statistics and management
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

/// Configure TDG storage backend and create optimized setup
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
