// Analysis dispatch/routing - extracted for file health (CB-040)

use crate::services::context::FileContext;

use super::metrics::detect_language;
use super::rust::analyze_rust_language;
use super::scripting::{
    analyze_bash_language, analyze_elixir_language, analyze_erlang_language,
    analyze_haskell_language, analyze_lua_language, analyze_ocaml_language,
    analyze_python_language, analyze_ruby_language, analyze_typescript_language,
};
use super::systems::{
    analyze_c_language, analyze_cpp_language, analyze_csharp_language, analyze_go_language,
    analyze_java_language, analyze_kotlin_language, analyze_lean_language, analyze_swift_language,
    analyze_wasm_language,
};

/// Analyze a single source file and extract AST items
/// Toyota Way refactored: Reduced complexity from 14 to <8 using Extract Method
pub async fn analyze_single_file(file_path: &std::path::Path) -> anyhow::Result<FileContext> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    let path_str = file_path.to_string_lossy().to_string();
    let language = detect_language(file_path);

    // EXTREME TDD FIX: Fail-fast for nonexistent files (RED test contract)
    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path_str));
    }

    let items = analyze_file_by_language(file_path, &language).await?;

    Ok(FileContext {
        path: path_str,
        language,
        items,
        complexity_metrics: None,
    })
}

/// Toyota Way Extract Method: Single responsibility for language-specific analysis
/// Reduced complexity by extracting the match logic into focused functions
pub async fn analyze_file_by_language(
    file_path: &std::path::Path,
    language: &str,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    match language {
        // Core languages with full AST analysis
        "rust" => analyze_rust_language(file_path).await,
        "typescript" | "javascript" => analyze_typescript_language(file_path).await,
        "python" => analyze_python_language(file_path).await,
        "go" => analyze_go_language(file_path).await,
        "c" => analyze_c_language(file_path).await,
        "cpp" => analyze_cpp_language(file_path).await,

        // JVM languages
        "java" => analyze_java_language(file_path).await,
        "kotlin" => analyze_kotlin_language(file_path).await,

        // .NET languages
        "csharp" => analyze_csharp_language(file_path).await,

        // Scripting languages
        "bash" => analyze_bash_language(file_path).await,
        "ruby" => analyze_ruby_language(file_path).await,
        "lua" => analyze_lua_language(file_path).await,

        // Functional languages
        "elixir" => analyze_elixir_language(file_path).await,
        "erlang" => analyze_erlang_language(file_path).await,
        "haskell" => analyze_haskell_language(file_path).await,
        "ocaml" => analyze_ocaml_language(file_path).await,

        // Apple ecosystem
        "swift" => analyze_swift_language(file_path).await,

        // WebAssembly
        "wasm" => analyze_wasm_language(file_path).await,

        // Proof assistants
        "lean" => analyze_lean_language(file_path).await,

        _ => Ok(Vec::new()),
    }
}
