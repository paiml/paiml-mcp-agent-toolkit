// Rust-specific analysis functions - extracted for file health (CB-040)

use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;
use super::metrics::RUST_UNIFIED_CACHE;

/// Toyota Way Single Responsibility: Handle Rust file analysis
/// OPTIMIZATION: Uses UnifiedRustAnalyzer to parse file once and extract both AST and complexity
pub async fn analyze_rust_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    // Use unified analyzer for single-pass parsing
    let analyzer = UnifiedRustAnalyzer::new(file_path.to_path_buf());
    let analysis = analyzer
        .analyze()
        .await
        .map_err(|e| anyhow::anyhow!("Unified Rust analysis failed: {}", e))?;

    // Store complexity metrics in thread-local cache for later retrieval
    RUST_UNIFIED_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .insert(file_path.to_path_buf(), analysis.file_metrics.clone());
    });

    Ok(analysis.ast_items)
}

/// Simple Rust file analysis
#[allow(dead_code)]
pub(super) async fn analyze_rust_file(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    use crate::services::ast_rust::analyze_rust_file as analyze_rust;

    match analyze_rust(file_path).await {
        Ok(file_context) => Ok(file_context.items),
        Err(_) => Ok(Vec::new()), // Return empty vec on parse error
    }
}
