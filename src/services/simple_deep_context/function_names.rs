#![cfg_attr(coverage_nightly, coverage(off))]
//! Per-language function name extraction.
use std::path::Path;

use super::types::SimpleDeepContext;

impl SimpleDeepContext {
    /// Extract function names from Rust AST
    pub(super) async fn function_names_for_rust(&self, file_path: &Path) -> Vec<String> {
        use crate::services::ast_rust::analyze_rust_file_with_complexity;
        match analyze_rust_file_with_complexity(file_path).await {
            Ok(metrics) => metrics.functions.iter().map(|f| f.name.clone()).collect(),
            Err(_) => vec![],
        }
    }

    /// Extract function names from WASM/WAT AST
    #[allow(unused_variables)]
    pub(super) async fn function_names_for_wasm(
        &self,
        file_path: &Path,
        extension: &str,
    ) -> Vec<String> {
        #[cfg(feature = "wasm-ast")]
        {
            use crate::services::context::AstItem;
            use crate::services::languages::wasm::WasmModuleAnalyzer;
            use tokio::fs;

            let analyzer = WasmModuleAnalyzer::new(file_path);
            let items = if extension == "wasm" {
                match std::fs::read(file_path) {
                    Ok(wasm_bytes) => analyzer.analyze_wasm_binary(&wasm_bytes),
                    Err(_) => return vec![],
                }
            } else {
                match fs::read_to_string(file_path).await {
                    Ok(content) => analyzer.analyze_wat_text(&content),
                    Err(_) => return vec![],
                }
            };
            match items {
                Ok(ast_items) => ast_items
                    .iter()
                    .filter_map(|item| match item {
                        AstItem::Function { name, .. } => Some(name.clone()),
                        _ => None,
                    })
                    .collect(),
                Err(_) => vec![],
            }
        }
        #[cfg(not(feature = "wasm-ast"))]
        {
            let _ = extension;
            vec![]
        }
    }

    /// Extract function names using language-specific AST analyzer
    #[allow(dead_code)]
    pub(super) async fn function_names_via_ast<F, E>(
        &self,
        file_path: &Path,
        analyze_fn: F,
    ) -> Vec<String>
    where
        F: FnOnce(&str) -> std::result::Result<Vec<crate::services::context::AstItem>, E>,
    {
        use crate::services::context::AstItem;
        use tokio::fs;

        match fs::read_to_string(file_path).await {
            Ok(content) => match analyze_fn(&content) {
                Ok(items) => items
                    .iter()
                    .filter_map(|item| match item {
                        AstItem::Function { name, .. } => Some(name.clone()),
                        _ => None,
                    })
                    .collect(),
                Err(_) => vec![],
            },
            Err(_) => vec![],
        }
    }

    /// Extract function names from Go AST
    pub(super) async fn function_names_for_go(&self, file_path: &Path) -> Vec<String> {
        #[cfg(feature = "go-ast")]
        {
            use crate::services::languages::go::GoAstVisitor;
            let fp = file_path.to_path_buf();
            self.function_names_via_ast(file_path, move |content| {
                let visitor = GoAstVisitor::new(&fp);
                visitor.analyze_go_source(content)
            })
            .await
        }
        #[cfg(not(feature = "go-ast"))]
        {
            self.extract_function_names_heuristic(file_path, "go")
                .await
                .unwrap_or_default()
        }
    }

    /// Extract function names from C# AST
    pub(super) async fn function_names_for_csharp(&self, file_path: &Path) -> Vec<String> {
        #[cfg(feature = "csharp-ast")]
        {
            use crate::services::languages::csharp::CSharpAstVisitor;
            let fp = file_path.to_path_buf();
            self.function_names_via_ast(file_path, move |content| {
                let visitor = CSharpAstVisitor::new(&fp);
                visitor.analyze_csharp_source(content)
            })
            .await
        }
        #[cfg(not(feature = "csharp-ast"))]
        {
            self.extract_function_names_heuristic(file_path, "cs")
                .await
                .unwrap_or_default()
        }
    }

    /// Extract function names from Kotlin AST
    pub(super) async fn function_names_for_kotlin(&self, file_path: &Path) -> Vec<String> {
        #[cfg(feature = "kotlin-ast")]
        {
            use crate::services::languages::kotlin::KotlinAstVisitor;
            let fp = file_path.to_path_buf();
            self.function_names_via_ast(file_path, move |content| {
                let visitor = KotlinAstVisitor::new(&fp);
                visitor.analyze_kotlin_source(content)
            })
            .await
        }
        #[cfg(not(feature = "kotlin-ast"))]
        {
            self.extract_function_names_heuristic(file_path, "kt")
                .await
                .unwrap_or_default()
        }
    }

    /// Extract function names from Bash AST
    #[allow(unused_variables)]
    pub(super) async fn function_names_for_bash(&self, file_path: &Path) -> Vec<String> {
        #[cfg(feature = "shell-ast")]
        {
            use crate::services::languages::bash::BashScriptAnalyzer;
            let fp = file_path.to_path_buf();
            self.function_names_via_ast(file_path, move |content| {
                let analyzer = BashScriptAnalyzer::new(&fp);
                analyzer
                    .analyze_bash_script(content)
                    .map_err(|e| e.to_string())
            })
            .await
        }
        #[cfg(not(feature = "shell-ast"))]
        vec![]
    }

    /// Extract function names from PHP AST
    #[allow(unused_variables)]
    pub(super) async fn function_names_for_php(&self, file_path: &Path) -> Vec<String> {
        #[cfg(feature = "php-ast")]
        {
            use crate::services::languages::php::PhpScriptAnalyzer;
            let fp = file_path.to_path_buf();
            self.function_names_via_ast(file_path, move |content| {
                let analyzer = PhpScriptAnalyzer::new(&fp);
                analyzer
                    .analyze_php_script(content)
                    .map_err(|e| e.to_string())
            })
            .await
        }
        #[cfg(not(feature = "php-ast"))]
        vec![]
    }

    /// Extract function names from Swift AST
    #[allow(unused_variables)]
    pub(super) async fn function_names_for_swift(&self, file_path: &Path) -> Vec<String> {
        #[cfg(feature = "swift-ast")]
        {
            use crate::services::languages::swift::SwiftSourceAnalyzer;
            let fp = file_path.to_path_buf();
            self.function_names_via_ast(file_path, move |content| {
                let analyzer = SwiftSourceAnalyzer::new(&fp);
                analyzer
                    .analyze_swift_source(content)
                    .map_err(|e| e.to_string())
            })
            .await
        }
        #[cfg(not(feature = "swift-ast"))]
        vec![]
    }

    /// Extract function names from Lua AST
    #[allow(unused_variables)]
    pub(super) async fn function_names_for_lua(&self, file_path: &Path) -> Vec<String> {
        self.extract_function_names_heuristic(file_path, "lua")
            .await
            .unwrap_or_default()
    }
}
