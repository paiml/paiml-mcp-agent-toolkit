#![cfg_attr(coverage_nightly, coverage(off))]
//! Per-language complexity analysis dispatchers.
use std::path::Path;

use super::types::SimpleDeepContext;

impl SimpleDeepContext {
    /// Rust complexity via AST
    pub(super) async fn complexity_for_rust(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use crate::services::ast_rust::analyze_rust_file_with_complexity;
        match analyze_rust_file_with_complexity(file_path).await {
            Ok(file_complexity_metrics) => {
                let functions = &file_complexity_metrics.functions;
                let function_count = functions.len();
                if function_count == 0 {
                    (0, 0, 0.0)
                } else {
                    let high_complexity_functions = functions
                        .iter()
                        .filter(|f| f.metrics.cyclomatic > 10)
                        .count();
                    let total_cyclomatic: u32 = functions
                        .iter()
                        .map(|f| u32::from(f.metrics.cyclomatic))
                        .sum();
                    let avg_complexity = f64::from(total_cyclomatic) / function_count as f64;
                    (function_count, high_complexity_functions, avg_complexity)
                }
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// TypeScript/JavaScript complexity via SWC AST
    pub(super) async fn complexity_for_typescript(
        &self,
        file_path: &Path,
        extension: &str,
    ) -> (usize, usize, f64) {
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            Ok(content) => {
                #[cfg(feature = "typescript-ast")]
                {
                    self.complexity_typescript_ast(file_path, extension, &content)
                }
                #[cfg(not(feature = "typescript-ast"))]
                {
                    let _ = (extension, &content);
                    self.complexity_heuristic_fallback(file_path, extension)
                        .await
                }
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// Inner TypeScript AST analysis (cfg-gated)
    #[cfg(feature = "typescript-ast")]
    pub(super) fn complexity_typescript_ast(
        &self,
        file_path: &Path,
        extension: &str,
        content: &str,
    ) -> (usize, usize, f64) {
        use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;
        use std::sync::Arc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

        let source_map = Arc::new(SourceMap::default());
        let _source_file = source_map.new_source_file(
            FileName::Custom(file_path.display().to_string()).into(),
            content.to_owned(),
        );

        let syntax = match extension {
            "tsx" => Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                dts: false,
                no_early_errors: true,
                disallow_ambiguous_jsx_like: true,
            }),
            "jsx" => Syntax::Es(swc_ecma_parser::EsSyntax {
                jsx: true,
                ..Default::default()
            }),
            "ts" => Syntax::Typescript(TsSyntax {
                tsx: false,
                decorators: true,
                dts: false,
                no_early_errors: true,
                disallow_ambiguous_jsx_like: true,
            }),
            _ => Syntax::Es(Default::default()),
        };

        let lexer = Lexer::new(
            syntax,
            Default::default(),
            StringInput::new(content, Default::default(), Default::default()),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        match parser.parse_module() {
            Ok(module) => {
                let visitor = EnhancedTypeScriptVisitor::new(file_path);
                let items = visitor.extract_items(&module);
                let function_count = items
                    .iter()
                    .filter(|item| {
                        matches!(item, crate::services::context::AstItem::Function { .. })
                    })
                    .count();
                if function_count == 0 {
                    (0, 0, 0.0)
                } else {
                    let high_complexity_functions = function_count / 4;
                    let avg_complexity = 2.5;
                    (function_count, high_complexity_functions, avg_complexity)
                }
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// WASM/WAT complexity analysis
    pub(super) async fn complexity_for_wasm(
        &self,
        file_path: &Path,
        extension: &str,
    ) -> (usize, usize, f64) {
        use tokio::fs;
        let content = if extension == "wasm" {
            match fs::read(file_path).await {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => return (0, 0, 0.0),
            }
        } else {
            match fs::read_to_string(file_path).await {
                Ok(text) => text,
                Err(_) => return (0, 0, 0.0),
            }
        };

        #[cfg(feature = "wasm-ast")]
        {
            use crate::services::languages::wasm::WasmModuleAnalyzer;
            let analyzer = WasmModuleAnalyzer::new(file_path);
            let items = if extension == "wasm" {
                match std::fs::read(file_path) {
                    Ok(wasm_bytes) => analyzer.analyze_wasm_binary(&wasm_bytes),
                    Err(_) => Err("Failed to read WASM binary".to_string()),
                }
            } else {
                analyzer.analyze_wat_text(&content)
            };
            match items {
                Ok(ast_items) => {
                    let function_count = ast_items
                        .iter()
                        .filter(|item| {
                            matches!(item, crate::services::context::AstItem::Function { .. })
                        })
                        .count();
                    if function_count == 0 {
                        (0, 0, 0.0)
                    } else {
                        let high_complexity_functions = function_count / 5;
                        let avg_complexity = 3.0;
                        (function_count, high_complexity_functions, avg_complexity)
                    }
                }
                Err(_) => (0, 0, 0.0),
            }
        }
        #[cfg(not(feature = "wasm-ast"))]
        {
            let _ = content;
            self.complexity_heuristic_fallback(file_path, extension)
                .await
        }
    }

    /// Go complexity via AST
    pub(super) async fn complexity_for_go(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            #[allow(unused_variables)]
            Ok(content) => {
                #[cfg(feature = "go-ast")]
                {
                    use crate::services::context::AstItem;
                    use crate::services::languages::go::{GoAstVisitor, GoComplexityAnalyzer};
                    let visitor = GoAstVisitor::new(file_path);
                    match visitor.analyze_go_source(&content) {
                        Ok(items) => {
                            let function_count = items
                                .iter()
                                .filter(|item| matches!(item, AstItem::Function { .. }))
                                .count();
                            if function_count == 0 {
                                return (0, 0, 0.0);
                            }
                            let mut analyzer = GoComplexityAnalyzer::new();
                            let (cyclomatic, _cognitive) =
                                analyzer.analyze_complexity(&content).unwrap_or((1, 1));
                            let high = if cyclomatic > 10 { 1 } else { 0 };
                            let avg = cyclomatic as f64 / function_count.max(1) as f64;
                            (function_count, high, avg)
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "go").await,
                    }
                }
                #[cfg(not(feature = "go-ast"))]
                self.complexity_heuristic_fallback(file_path, "go").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// C# complexity via AST
    pub(super) async fn complexity_for_csharp(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        #[allow(unused_variables)]
        match fs::read_to_string(file_path).await {
            Ok(content) => {
                #[cfg(feature = "csharp-ast")]
                {
                    use crate::services::context::AstItem;
                    use crate::services::languages::csharp::{
                        CSharpAstVisitor, CSharpComplexityAnalyzer,
                    };
                    let visitor = CSharpAstVisitor::new(file_path);
                    match visitor.analyze_csharp_source(&content) {
                        Ok(items) => {
                            let function_count = items
                                .iter()
                                .filter(|item| matches!(item, AstItem::Function { .. }))
                                .count();
                            if function_count == 0 {
                                return (0, 0, 0.0);
                            }
                            let mut analyzer = CSharpComplexityAnalyzer::new();
                            let (cyclomatic, _cognitive) =
                                analyzer.analyze_complexity(&content).unwrap_or((1, 1));
                            let high = if cyclomatic > 10 { 1 } else { 0 };
                            let avg = cyclomatic as f64 / function_count.max(1) as f64;
                            (function_count, high, avg)
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "cs").await,
                    }
                }
                #[cfg(not(feature = "csharp-ast"))]
                self.complexity_heuristic_fallback(file_path, "cs").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// Kotlin complexity via AST
    pub(super) async fn complexity_for_kotlin(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            #[cfg_attr(not(feature = "kotlin-ast"), allow(unused_variables))]
            Ok(content) => {
                #[cfg(feature = "kotlin-ast")]
                {
                    use crate::services::context::AstItem;
                    use crate::services::languages::kotlin::{
                        KotlinAstVisitor, KotlinComplexityAnalyzer,
                    };
                    let visitor = KotlinAstVisitor::new(file_path);
                    match visitor.analyze_kotlin_source(&content) {
                        Ok(items) => {
                            let function_count = items
                                .iter()
                                .filter(|item| matches!(item, AstItem::Function { .. }))
                                .count();
                            if function_count == 0 {
                                return (0, 0, 0.0);
                            }
                            let mut analyzer = KotlinComplexityAnalyzer::new();
                            let (cyclomatic, _cognitive) =
                                analyzer.analyze_complexity(&content).unwrap_or((1, 1));
                            let high = if cyclomatic > 10 { 1 } else { 0 };
                            let avg = cyclomatic as f64 / function_count.max(1) as f64;
                            (function_count, high, avg)
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "kt").await,
                    }
                }
                #[cfg(not(feature = "kotlin-ast"))]
                self.complexity_heuristic_fallback(file_path, "kt").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// Bash complexity via AST (function count estimation)
    pub(super) async fn complexity_for_bash(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            #[allow(unused_variables)]
            Ok(content) => {
                #[cfg(feature = "shell-ast")]
                {
                    use crate::services::context::AstItem;
                    use crate::services::languages::bash::BashScriptAnalyzer;
                    let analyzer = BashScriptAnalyzer::new(file_path);
                    match analyzer.analyze_bash_script(&content) {
                        Ok(items) => {
                            let function_count = items
                                .iter()
                                .filter(|item| matches!(item, AstItem::Function { .. }))
                                .count();
                            if function_count == 0 {
                                (0, 0, 0.0)
                            } else {
                                (function_count, 0, 2.0)
                            }
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "sh").await,
                    }
                }
                #[cfg(not(feature = "shell-ast"))]
                self.complexity_heuristic_fallback(file_path, "sh").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// PHP complexity via AST (function count estimation)
    pub(super) async fn complexity_for_php(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            #[allow(unused_variables)]
            Ok(content) => {
                #[cfg(feature = "php-ast")]
                {
                    use crate::services::context::AstItem;
                    use crate::services::languages::php::PhpScriptAnalyzer;
                    let analyzer = PhpScriptAnalyzer::new(file_path);
                    match analyzer.analyze_php_script(&content) {
                        Ok(items) => {
                            let function_count = items
                                .iter()
                                .filter(|item| matches!(item, AstItem::Function { .. }))
                                .count();
                            if function_count == 0 {
                                (0, 0, 0.0)
                            } else {
                                (function_count, 0, 2.5)
                            }
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "php").await,
                    }
                }
                #[cfg(not(feature = "php-ast"))]
                self.complexity_heuristic_fallback(file_path, "php").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// Swift complexity via AST (function count estimation)
    pub(super) async fn complexity_for_swift(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            #[allow(unused_variables)]
            Ok(content) => {
                #[cfg(feature = "swift-ast")]
                {
                    use crate::services::context::AstItem;
                    use crate::services::languages::swift::SwiftSourceAnalyzer;
                    let analyzer = SwiftSourceAnalyzer::new(file_path);
                    match analyzer.analyze_swift_source(&content) {
                        Ok(items) => {
                            let function_count = items
                                .iter()
                                .filter(|item| matches!(item, AstItem::Function { .. }))
                                .count();
                            if function_count == 0 {
                                (0, 0, 0.0)
                            } else {
                                (function_count, 0, 2.5)
                            }
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "swift").await,
                    }
                }
                #[cfg(not(feature = "swift-ast"))]
                self.complexity_heuristic_fallback(file_path, "swift").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// Lua complexity via tree-sitter AST
    pub(super) async fn complexity_for_lua(&self, file_path: &Path) -> (usize, usize, f64) {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        use tokio::fs;
        match fs::read_to_string(file_path).await {
            #[allow(unused_variables)]
            Ok(content) => {
                #[cfg(feature = "lua-ast")]
                {
                    use crate::ast::languages::lua::LuaStrategy;
                    use crate::ast::languages::LanguageStrategy;
                    let strategy = LuaStrategy::new();
                    match strategy.parse_file(file_path, &content).await {
                        Ok(ast) => {
                            let functions = strategy.extract_functions(&ast);
                            let function_count = functions.len();
                            if function_count == 0 {
                                (0, 0, 0.0)
                            } else {
                                let (cyclomatic, _cognitive) = strategy.calculate_complexity(&ast);
                                let high = if cyclomatic > 10 { 1 } else { 0 };
                                let avg = cyclomatic as f64 / function_count.max(1) as f64;
                                (function_count, high, avg)
                            }
                        }
                        Err(_) => self.complexity_heuristic_fallback(file_path, "lua").await,
                    }
                }
                #[cfg(not(feature = "lua-ast"))]
                self.complexity_heuristic_fallback(file_path, "lua").await
            }
            Err(_) => (0, 0, 0.0),
        }
    }
}
