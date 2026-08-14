#![allow(unused)]
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
    analyze_java_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle C# file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_csharp_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    analyze_csharp_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Swift file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_swift_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    analyze_swift_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle WebAssembly file analysis
/// TICKET-3005: Now uses unified parser to eliminate double parsing
#[allow(unused_variables)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_wasm_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
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
async fn analyze_go_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
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
async fn analyze_c_file(
    #[allow(unused_variables)] file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
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
    // kotlin-ast feature is disabled
    Ok(Vec::new())
}

/// Error text returned when the binary was built without a Java AST parser.
///
/// `java-ast` is NOT in `default`, so a stock `cargo install pmat` has no Java
/// parser linked in. Returning `Ok(vec![])` in that case made "this build cannot
/// read Java at all" indistinguishable from "this Java file declares nothing",
/// which is how `pmat context` came to report `items: []` for a perfectly valid
/// `.java` file. Refuse honestly instead — same shape as the `--features
/// deep-wasm` / `--features demo` refusals elsewhere in the CLI.
pub(crate) const JAVA_AST_UNAVAILABLE: &str =
    "Java analysis unavailable: this build has no Java AST parser (feature `java-ast` is off, and it is not in the default feature set). Rebuild with --features java-ast. Reporting zero items would be indistinguishable from a Java file that genuinely declares none.";

/// Simple Java file analysis.
///
/// Every failure mode is representable in the return value:
/// * the file cannot be read -> `Err`, naming the path and the io error;
/// * the source does not parse -> `Err`, naming the path and the parser message;
/// * this build has no Java parser -> `Err`, [`JAVA_AST_UNAVAILABLE`].
///
/// `Ok(vec![])` therefore means one thing only: the file parsed and declared
/// nothing.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_java_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    #[cfg(feature = "java-ast")]
    {
        use crate::services::languages::java::JavaAstVisitor;
        use tokio::fs;

        let source = fs::read_to_string(_file_path).await.map_err(|e| {
            anyhow::anyhow!("Java analysis: cannot read {}: {}", _file_path.display(), e)
        })?;
        let visitor = JavaAstVisitor::new(_file_path);
        visitor.analyze_java_source(&source).map_err(|e| {
            anyhow::anyhow!(
                "Java analysis: cannot parse {}: {}",
                _file_path.display(),
                e
            )
        })
    }
    #[cfg(not(feature = "java-ast"))]
    Err(anyhow::anyhow!(
        "{} (path: {})",
        JAVA_AST_UNAVAILABLE,
        _file_path.display()
    ))
}

/// Simple C# file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_csharp_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
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
    #[cfg(feature = "swift-ast")]
    {
        // Swift tree-sitter analyzer not yet available
        Ok(Vec::new())
    }
    #[cfg(not(feature = "swift-ast"))]
    Ok(Vec::new())
}

/// Simple WebAssembly file analysis
async fn analyze_wasm_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
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

/// Regression tests for #966's adjacent defect: Java analysis rendered every
/// failure as an empty success.
///
/// All of these fail on the pre-fix code, which returned `Ok(Vec::new())` from
/// the read-error arm, the parse-error arm and the `cfg(not(java-ast))` arm.
#[cfg(test)]
mod java_failure_is_representable_tests {
    use super::analyze_java_file;

    /// A build without `java-ast` — which is what `cargo install pmat` produces,
    /// since `java-ast` is not in `default` — must say it cannot analyse Java
    /// rather than report a Java file as having no items.
    #[cfg(not(feature = "java-ast"))]
    #[tokio::test]
    async fn without_java_ast_feature_refuses_instead_of_reporting_zero_items() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Good.java");
        std::fs::write(
            &file,
            "package com.example;\npublic class Good {\n  public int add(int a, int b) { return a + b; }\n}\n",
        )
        .unwrap();

        let err = analyze_java_file(&file)
            .await
            .expect_err("a build with no Java parser must refuse, not return an empty item list");
        let msg = err.to_string();
        assert!(
            msg.contains("--features java-ast"),
            "refusal must name the feature to rebuild with, got: {msg}"
        );
        assert!(
            msg.contains("Java analysis unavailable"),
            "refusal must say Java analysis is unavailable, got: {msg}"
        );
    }

    /// With the parser linked in, a file that does not parse must be an error,
    /// not an empty item list that looks like an empty-but-valid class.
    #[cfg(feature = "java-ast")]
    #[tokio::test]
    async fn unparseable_java_is_an_error_not_an_empty_item_list() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Broken.java");
        // Unbalanced braces: `is_valid_java_syntax` rejects this.
        std::fs::write(
            &file,
            "package com.example;\npublic class Broken {\n  public int add(int a) { return a;\n",
        )
        .unwrap();

        let err = analyze_java_file(&file)
            .await
            .expect_err("an unparseable Java file must not analyse as zero items");
        assert!(
            err.to_string().contains("cannot parse"),
            "error must say the file could not be parsed, got: {err}"
        );
    }

    /// The `Ok(vec![])` that remains must mean exactly one thing, so a file that
    /// really does parse still has to produce items.
    #[cfg(feature = "java-ast")]
    #[tokio::test]
    async fn parseable_java_still_yields_items() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Good.java");
        std::fs::write(
            &file,
            "package com.example;\npublic class Good {\n  public int add(int a, int b) { return a + b; }\n}\n",
        )
        .unwrap();

        let items = analyze_java_file(&file)
            .await
            .expect("valid Java must parse");
        assert!(
            !items.is_empty(),
            "a class with a method must yield AST items"
        );
    }

    /// A path that does not exist is a read failure, and read failures were the
    /// other arm that silently produced an empty success.
    #[cfg(feature = "java-ast")]
    #[tokio::test]
    async fn unreadable_java_is_an_error_not_an_empty_item_list() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("Absent.java");

        let err = analyze_java_file(&missing)
            .await
            .expect_err("an unreadable Java file must not analyse as zero items");
        assert!(
            err.to_string().contains("cannot read"),
            "error must say the file could not be read, got: {err}"
        );
    }
}
