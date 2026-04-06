// Scripting language analysis (Python, TypeScript/JS, Bash, Ruby, Lua,
// Elixir, Erlang, Haskell, OCaml)
// Extracted for file health (CB-040)

use std::path::Path;

#[cfg(feature = "shell-ast")]
use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;
#[cfg(feature = "python-ast")]
use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;
use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

#[cfg(feature = "shell-ast")]
use super::metrics::BASH_UNIFIED_CACHE;
#[cfg(feature = "python-ast")]
use super::metrics::PYTHON_UNIFIED_CACHE;
use super::metrics::TYPESCRIPT_UNIFIED_CACHE;

/// Toyota Way Single Responsibility: Handle TypeScript/JavaScript file analysis
/// OPTIMIZATION: Uses UnifiedTypeScriptAnalyzer to parse file once and extract both AST and complexity
pub async fn analyze_typescript_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    // Use unified analyzer for single-pass parsing
    let analyzer = UnifiedTypeScriptAnalyzer::new(file_path.to_path_buf());
    let analysis = analyzer
        .analyze()
        .await
        .map_err(|e| anyhow::anyhow!("Unified TypeScript analysis failed: {}", e))?;

    // Store complexity metrics in process-global cache for cross-task sharing
    TYPESCRIPT_UNIFIED_CACHE.insert(file_path.to_path_buf(), analysis.file_metrics.clone());

    Ok(analysis.ast_items)
}

/// Toyota Way Single Responsibility: Handle Python file analysis
/// OPTIMIZATION: Uses UnifiedPythonAnalyzer to parse file once and extract both AST and complexity
#[cfg(feature = "python-ast")]
pub async fn analyze_python_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    // Use unified analyzer for single-pass parsing
    let analyzer = UnifiedPythonAnalyzer::new(file_path.to_path_buf());
    let analysis = analyzer
        .analyze()
        .await
        .map_err(|e| anyhow::anyhow!("Unified Python analysis failed: {}", e))?;

    // Store complexity metrics in process-global cache for cross-task sharing
    PYTHON_UNIFIED_CACHE.insert(file_path.to_path_buf(), analysis.file_metrics.clone());

    Ok(analysis.ast_items)
}

#[cfg(not(feature = "python-ast"))]
pub async fn analyze_python_language(
    _file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // python-ast feature is disabled
    Ok(Vec::new())
}

/// Toyota Way Single Responsibility: Handle Bash script file analysis
/// TICKET-3006: Now uses unified parser to eliminate double parsing
#[cfg(feature = "shell-ast")]
pub async fn analyze_bash_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    tracing::debug!("Analyzing Bash script file: {}", file_path.display());

    // Use unified analyzer for single parse pass
    let analyzer = UnifiedBashAnalyzer::new(file_path.to_path_buf());
    match analyzer.analyze().await {
        Ok(analysis) => {
            // Store complexity metrics in process-global cache for cross-task sharing
            BASH_UNIFIED_CACHE.insert(file_path.to_path_buf(), analysis.file_metrics.clone());
            tracing::debug!("Bash analysis returned {} items", analysis.ast_items.len());
            Ok(analysis.ast_items)
        }
        Err(_) => {
            // Fall back to old analyzer on error
            let items = analyze_bash_file(file_path).await?;
            tracing::debug!("Bash analysis returned {} items", items.len());
            Ok(items)
        }
    }
}

#[cfg(not(feature = "shell-ast"))]
pub async fn analyze_bash_language(
    _file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // shell-ast feature is disabled
    Ok(Vec::new())
}

/// Toyota Way Single Responsibility: Handle Ruby file analysis
pub async fn analyze_ruby_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_ruby_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Lua file analysis (regex + tree-sitter)
#[cfg(feature = "lua-ast")]
pub async fn analyze_lua_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    use crate::services::context::AstItem;

    let content = tokio::fs::read_to_string(file_path).await?;
    let mut items = Vec::new();

    // Extract function names using proven regex patterns (same as simple_deep_context)
    let patterns = [
        r"(?m)^\s*function\s+(\w+(?:[.:]\w+)*)\s*\(",
        r"(?m)^\s*local\s+function\s+(\w+)\s*\(",
    ];
    for pat in &patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            for cap in re.captures_iter(&content) {
                if let Some(name_match) = cap.get(1) {
                    let line = content
                        .get(..name_match.start())
                        .unwrap_or_default()
                        .lines()
                        .count();
                    items.push(AstItem::Function {
                        name: name_match.as_str().to_string(),
                        visibility: "public".to_string(),
                        is_async: false,
                        line,
                    });
                }
            }
        }
    }

    // Extract imports (require calls)
    if let Ok(re) = regex::Regex::new(r#"(?m)require\s*\(\s*["']([^"']+)["']\s*\)"#) {
        for cap in re.captures_iter(&content) {
            if let Some(module_match) = cap.get(1) {
                let line = content
                    .get(..module_match.start())
                    .unwrap_or_default()
                    .lines()
                    .count();
                items.push(AstItem::Import {
                    module: module_match.as_str().to_string(),
                    items: vec![],
                    alias: None,
                    line,
                });
            }
        }
    }

    tracing::debug!("Lua analysis returned {} items", items.len());
    Ok(items)
}

#[cfg(not(feature = "lua-ast"))]
pub async fn analyze_lua_language(
    _file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    Ok(Vec::new())
}

/// Toyota Way Single Responsibility: Handle Elixir file analysis
pub async fn analyze_elixir_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_elixir_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Erlang file analysis
pub async fn analyze_erlang_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_erlang_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Haskell file analysis
pub async fn analyze_haskell_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_haskell_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle OCaml file analysis
pub async fn analyze_ocaml_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    analyze_ocaml_file(file_path).await
}

// --- Fallback / simple analysis functions ---

/// Simple TypeScript/JavaScript file analysis
#[allow(dead_code)]
async fn analyze_typescript_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "typescript-ast")]
    {
        use crate::services::ast_typescript::analyze_typescript_file as analyze_ts;

        match analyze_ts(_file_path).await {
            Ok(file_context) => Ok(file_context.items),
            Err(_) => Ok(Vec::new()), // Return empty vec on parse error
        }
    }
    #[cfg(not(feature = "typescript-ast"))]
    Ok(Vec::new())
}

/// Simple Python file analysis
#[allow(dead_code)]
async fn analyze_python_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "python-ast")]
    {
        use crate::services::ast_python::analyze_python_file_with_classifier;

        match analyze_python_file_with_classifier(_file_path, None).await {
            Ok(file_context) => Ok(file_context.items),
            Err(_) => Ok(Vec::new()), // Return empty vec on parse error
        }
    }
    #[cfg(not(feature = "python-ast"))]
    Ok(Vec::new())
}

/// Simple Bash script file analysis
#[cfg(feature = "shell-ast")]
async fn analyze_bash_file(
    file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    use crate::services::languages::bash::BashScriptAnalyzer;
    use std::fs;

    match fs::read_to_string(file_path) {
        Ok(source) => {
            let analyzer = BashScriptAnalyzer::new(file_path);
            match analyzer.analyze_bash_script(&source) {
                Ok(items) => Ok(items),
                Err(_) => Ok(Vec::new()), // Return empty vec on parse error
            }
        }
        Err(_) => Ok(Vec::new()), // Return empty vec on file read error
    }
}

#[cfg(not(feature = "shell-ast"))]
#[allow(dead_code)]
async fn analyze_bash_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // shell-ast feature is disabled
    Ok(Vec::new())
}

/// Simple Ruby file analysis (tree-sitter)
async fn analyze_ruby_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    #[cfg(feature = "ruby-ast")]
    {
        // Tree-sitter Ruby analyzer not yet available; falls back to ruchy
        Ok(Vec::new())
    }
    #[cfg(not(feature = "ruby-ast"))]
    {
        // Ruchy parser fallback (API requires parsed Expr, not source string)
        #[cfg(feature = "ruchy-ast")]
        {
            // RuchyAstAnalyzer API incompatible with this code path
            // Would require parsing source to Expr first
            Ok(Vec::new())
        }
        #[cfg(not(feature = "ruchy-ast"))]
        Ok(Vec::new())
    }
}

/// Simple Elixir file analysis
/// Note: Elixir parser removed in Sprint 46 Phase 5 (unimplemented stub)
async fn analyze_elixir_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // Feature removed - was unimplemented stub
    Ok(Vec::new())
}

/// Simple Erlang file analysis
/// Note: Erlang parser removed in Sprint 46 Phase 5 (unimplemented stub)
async fn analyze_erlang_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // Feature removed - was unimplemented stub
    Ok(Vec::new())
}

/// Simple Haskell file analysis
/// Note: Haskell parser removed in Sprint 46 Phase 5 (unimplemented stub)
async fn analyze_haskell_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // Feature removed - was unimplemented stub
    Ok(Vec::new())
}

/// Simple OCaml file analysis
/// Note: OCaml parser removed in Sprint 46 Phase 5 (unimplemented stub)
async fn analyze_ocaml_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    // Feature removed - was unimplemented stub
    Ok(Vec::new())
}
