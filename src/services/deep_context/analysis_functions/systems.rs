// Systems language analysis (C, C++, Go, Java, Kotlin, C#, Swift, WASM, Lean)
// Extracted for file health (CB-040)

use std::path::Path;

#[cfg(feature = "go-ast")]
use crate::services::unified_go_analyzer::UnifiedGoAnalyzer;
#[cfg(feature = "wasm-ast")]
use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

#[cfg(feature = "go-ast")]
use super::metrics::GO_UNIFIED_CACHE;
#[cfg(feature = "wasm-ast")]
use super::metrics::WASM_UNIFIED_CACHE;

/// Toyota Way Single Responsibility: Handle Go file analysis
/// TICKET-3004: Now uses unified parser to eliminate double parsing
#[allow(unused_variables)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_go_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    #[cfg(feature = "go-ast")]
    {
        // Use unified analyzer for single parse pass
        let analyzer = UnifiedGoAnalyzer::new(file_path.to_path_buf());
        match analyzer.analyze().await {
            Ok(analysis) => {
                // Store complexity metrics in process-global cache for cross-task sharing
                GO_UNIFIED_CACHE.insert(file_path.to_path_buf(), analysis.file_metrics.clone());
                Ok(analysis.ast_items)
            }
            Err(_) => {
                // Fall back to old analyzer on error
                analyze_go_file(file_path).await
            }
        }
    }
    #[cfg(not(feature = "go-ast"))]
    Ok(Vec::new())
}

/// Toyota Way Single Responsibility: Handle C language analysis with improved features
/// Uses the enhanced CAstVisitor from services/ast/languages/c.rs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_c_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    #[cfg(feature = "c-ast")]
    {
        // Use the new comprehensive C language analyzer
        use crate::services::ast::languages::c;
        let file_context = c::analyze_c_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("C analysis error: {}", e))?;

        // Return the AST items from the file context
        Ok(file_context.items)
    }

    #[cfg(not(feature = "c-ast"))]
    analyze_c_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle C++ language analysis with modern features
/// Uses the enhanced CppAstVisitor from services/ast/languages/cpp.rs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_cpp_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    #[cfg(feature = "cpp-ast")]
    {
        // Use the comprehensive C++ language analyzer
        use crate::services::ast::languages::cpp;
        let file_context = cpp::analyze_cpp_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("C++ analysis error: {}", e))?;

        // Return the AST items from the file context
        Ok(file_context.items)
    }

    #[cfg(not(feature = "cpp-ast"))]
    analyze_c_file(file_path).await // Fallback to C analysis if C++ feature is not enabled
}

/// Toyota Way Single Responsibility: Handle Kotlin file analysis with debug logging
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_kotlin_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    tracing::debug!("Analyzing Kotlin file: {}", file_path.display());
    let items = analyze_kotlin_file(file_path).await?;
    tracing::debug!("Kotlin analysis returned {} items", items.len());
    Ok(items)
}

/// Toyota Way Single Responsibility: Handle Java file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_java_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_java_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle C# file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_csharp_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_csharp_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Swift file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_swift_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_swift_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle WebAssembly file analysis
/// TICKET-3005: Now uses unified parser to eliminate double parsing
#[allow(unused_variables)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_wasm_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    #[cfg(feature = "wasm-ast")]
    {
        // Use unified analyzer for single parse pass
        let analyzer = UnifiedWasmAnalyzer::new(file_path.to_path_buf());
        match analyzer.analyze().await {
            Ok(analysis) => {
                // Store complexity metrics in process-global cache for cross-task sharing
                WASM_UNIFIED_CACHE.insert(file_path.to_path_buf(), analysis.file_metrics.clone());
                Ok(analysis.ast_items)
            }
            Err(_) => {
                // Fall back to old analyzer on error
                analyze_wasm_file(file_path).await
            }
        }
    }
    #[cfg(not(feature = "wasm-ast"))]
    Ok(Vec::new())
}

/// Toyota Way Single Responsibility: Handle Lean 4 file analysis
#[allow(unused_variables)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_lean_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    #[cfg(feature = "lean-ast")]
    {
        use crate::services::languages::lean;
        match lean::analyze_lean_file(file_path).await {
            Ok(file_context) => Ok(file_context.items),
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "lean-ast"))]
    Ok(Vec::new())
}

// --- Fallback / simple analysis functions ---

/// Simple Go file analysis
#[allow(dead_code)]
async fn analyze_go_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "go-ast")]
    {
        use crate::services::languages::go;

        match go::analyze_go_file(_file_path).await {
            Ok(file_context) => Ok(file_context.items),
            Err(_) => Ok(Vec::new()), // Return empty vec on parse error
        }
    }
    #[cfg(not(feature = "go-ast"))]
    Ok(Vec::new())
}

/// Legacy C file analysis - redirects to services/ast/languages/c.rs
/// This function is kept for backward compatibility but delegates to the new implementation
#[allow(dead_code)]
async fn analyze_c_file(
    #[allow(unused_variables)] file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    #[cfg(feature = "c-ast")]
    {
        // Direct delegation to the new implementation
        // This avoids code duplication and ensures consistency
        use crate::services::ast::languages::c;
        let file_context = c::analyze_c_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("C analysis error: {}", e))?;

        Ok(file_context.items)
    }
    #[cfg(not(feature = "c-ast"))]
    Ok(Vec::new())
}

async fn analyze_kotlin_file(
    #[allow(unused_variables)] file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    // kotlin-ast feature is disabled
    Ok(Vec::new())
}

/// Simple Java file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_java_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "java-ast")]
    {
        use crate::services::languages::java::JavaAstVisitor;
        use tokio::fs;

        match fs::read_to_string(_file_path).await {
            Ok(source) => {
                let visitor = JavaAstVisitor::new(_file_path);
                match visitor.analyze_java_source(&source) {
                    Ok(items) => Ok(items),
                    Err(_) => Ok(Vec::new()),
                }
            }
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "java-ast"))]
    Ok(Vec::new())
}

/// Simple C# file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_csharp_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "csharp-ast")]
    {
        use crate::services::languages::csharp::CSharpAstVisitor;
        use tokio::fs;

        match fs::read_to_string(_file_path).await {
            Ok(source) => {
                let visitor = CSharpAstVisitor::new(_file_path);
                match visitor.analyze_csharp_source(&source) {
                    Ok(items) => Ok(items),
                    Err(_) => Ok(Vec::new()),
                }
            }
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "csharp-ast"))]
    Ok(Vec::new())
}

/// Simple Swift file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_swift_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "swift-ast")]
    {
        // Swift tree-sitter analyzer not yet available
        Ok(Vec::new())
    }
    #[cfg(not(feature = "swift-ast"))]
    Ok(Vec::new())
}

/// Simple WebAssembly file analysis
#[allow(dead_code)]
async fn analyze_wasm_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "wasm-ast")]
    {
        use crate::services::languages::wasm::WasmModuleAnalyzer;

        match tokio::fs::read_to_string(_file_path).await {
            Ok(source) => {
                let analyzer = WasmModuleAnalyzer::new(_file_path);
                match analyzer.analyze_wat_text(&source) {
                    Ok(items) => Ok(items),
                    Err(_) => Ok(Vec::new()),
                }
            }
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "wasm-ast"))]
    Ok(Vec::new())
}
