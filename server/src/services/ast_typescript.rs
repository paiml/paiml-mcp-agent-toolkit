// TypeScript/JavaScript AST analysis using SWC parser
// This module provides comprehensive AST parsing for TypeScript and JavaScript files

#[cfg(feature = "typescript-ast")]
use swc_common::{sync::Lrc, FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::*;
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
#[cfg(feature = "typescript-ast")]
use swc_ecma_visit::{Visit, VisitWith};

use crate::models::error::TemplateError;
use crate::services::complexity::{
    ClassComplexity, ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use std::path::Path;

/// TypeScript/JavaScript symbol extracted from AST
#[derive(Debug, Clone)]
pub struct TypeScriptSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub is_exported: bool,
    pub is_async: bool,
    pub variants_count: usize,
    pub fields_count: usize,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    Variable,
    Import,
    Export,
    Method,
    Property,
}

/// AST visitor for extracting symbols and calculating complexity
#[cfg(feature = "typescript-ast")]
struct TypeScriptVisitor {
    source_map: Lrc<SourceMap>,
    symbols: Vec<TypeScriptSymbol>,
    functions: Vec<FunctionComplexity>,
    classes: Vec<ClassComplexity>,
    current_function_complexity: ComplexityMetrics,
    current_class: Option<ClassComplexity>,
    total_complexity: ComplexityMetrics,
    nesting_level: u8,
    current_function_start: u32,
}

#[cfg(feature = "typescript-ast")]
impl TypeScriptVisitor {
    fn new(source_map: Lrc<SourceMap>) -> Self {
        Self {
            source_map,
            symbols: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            current_function_complexity: ComplexityMetrics::default(),
            current_class: None,
            total_complexity: ComplexityMetrics::default(),
            nesting_level: 0,
            current_function_start: 0,
        }
    }

    fn get_line_number(&self, span: swc_common::Span) -> usize {
        let loc = self.source_map.lookup_char_pos(span.lo);
        loc.line
    }

    fn add_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: swc_common::Span,
        is_exported: bool,
    ) {
        self.add_symbol_with_async(name, kind, span, is_exported, false);
    }

    fn add_symbol_with_async(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: swc_common::Span,
        is_exported: bool,
        is_async: bool,
    ) {
        self.add_symbol_full(name, kind, span, is_exported, is_async, 0);
    }

    fn add_symbol_full(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: swc_common::Span,
        is_exported: bool,
        is_async: bool,
        variants_count: usize,
    ) {
        self.add_symbol_complete(name, kind, span, is_exported, is_async, variants_count, 0);
    }

    #[allow(clippy::too_many_arguments)]
    fn add_symbol_complete(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: swc_common::Span,
        is_exported: bool,
        is_async: bool,
        variants_count: usize,
        fields_count: usize,
    ) {
        let symbol = TypeScriptSymbol {
            name,
            kind,
            line: self.get_line_number(span),
            is_exported,
            is_async,
            variants_count,
            fields_count,
        };
        self.symbols.push(symbol);
    }

    fn add_complexity(&mut self, cyclomatic: u16, cognitive: u16) {
        self.current_function_complexity.cyclomatic += cyclomatic;
        self.current_function_complexity.cognitive += cognitive;
        self.total_complexity.cyclomatic += cyclomatic;
        self.total_complexity.cognitive += cognitive;
    }

    fn enter_nesting(&mut self) {
        self.nesting_level += 1;
        self.current_function_complexity.nesting_max = self
            .current_function_complexity
            .nesting_max
            .max(self.nesting_level);
    }

    fn exit_nesting(&mut self) {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
        }
    }

    fn finalize_function(&mut self, name: String, span: swc_common::Span) {
        let line_start = self.get_line_number(span) as u32;

        // Estimate function size based on complexity
        let estimated_lines = (self.current_function_complexity.cyclomatic as u32 * 3).max(5);

        let function_complexity = FunctionComplexity {
            name: name.clone(),
            line_start,
            line_end: line_start + estimated_lines,
            metrics: self.current_function_complexity,
        };

        if let Some(ref mut class) = self.current_class {
            // Method in a class
            class.methods.push(function_complexity);
            class.metrics.cyclomatic += self.current_function_complexity.cyclomatic;
            class.metrics.cognitive += self.current_function_complexity.cognitive;
        } else {
            // Top-level function
            self.functions.push(function_complexity);
        }

        // Reset for next function
        self.current_function_complexity = ComplexityMetrics::default();
        self.nesting_level = 0;
    }
}

#[cfg(feature = "typescript-ast")]
impl Visit for TypeScriptVisitor {
    // Removed visit_function to avoid circular calls with visit_fn_decl

    fn visit_fn_decl(&mut self, n: &FnDecl) {
        let name = n.ident.sym.to_string();
        let span = n.ident.span;
        let is_async = n.function.is_async;
        self.current_function_start = self.get_line_number(span) as u32;

        // Visit function body directly without visiting the function itself
        if let Some(body) = &n.function.body {
            for stmt in &body.stmts {
                stmt.visit_with(self);
            }
        }

        // Add symbol with async info
        self.add_symbol_with_async(name.clone(), SymbolKind::Function, span, false, is_async);

        // Finalize function complexity
        self.finalize_function(name, span);
    }

    fn visit_class_decl(&mut self, n: &ClassDecl) {
        let name = n.ident.sym.to_string();
        let span = n.ident.span;

        // Count class fields (properties)
        let mut fields_count = 0;
        for member in &n.class.body {
            if let ClassMember::ClassProp(_) = member {
                fields_count += 1;
            }
        }

        let class_complexity = ClassComplexity {
            name: name.clone(),
            line_start: self.get_line_number(span) as u32,
            line_end: self.get_line_number(span) as u32 + 50, // Estimate
            metrics: ComplexityMetrics::default(),
            methods: Vec::new(),
        };

        let prev_class = self.current_class.take();
        self.current_class = Some(class_complexity);

        // Add symbol with field count
        self.add_symbol_complete(name, SymbolKind::Class, span, false, false, 0, fields_count);

        // Visit class members
        for member in &n.class.body {
            member.visit_with(self);
        }

        if let Some(class_complexity) = self.current_class.take() {
            self.classes.push(class_complexity);
        }

        self.current_class = prev_class;
    }

    fn visit_class_method(&mut self, n: &ClassMethod) {
        if let PropName::Ident(ident) = &n.key {
            let name = ident.sym.to_string();
            let span = ident.span;
            let is_async = n.function.is_async;
            self.current_function_start = self.get_line_number(span) as u32;

            // Visit method body directly without calling visit_with on the function
            if let Some(body) = &n.function.body {
                for stmt in &body.stmts {
                    stmt.visit_with(self);
                }
            }

            // Add symbol with async info
            self.add_symbol_with_async(name.clone(), SymbolKind::Method, span, false, is_async);

            // Finalize method complexity
            self.finalize_function(name, span);
        }
    }

    fn visit_var_decl(&mut self, n: &VarDecl) {
        for decl in &n.decls {
            if let Pat::Ident(ident) = &decl.name {
                let name = ident.id.sym.to_string();
                let span = ident.id.span;
                self.add_symbol(name, SymbolKind::Variable, span, false);
            }
        }
    }

    fn visit_ts_interface_decl(&mut self, n: &TsInterfaceDecl) {
        let name = n.id.sym.to_string();
        let span = n.id.span;
        self.add_symbol(name, SymbolKind::Interface, span, false);
    }

    fn visit_ts_type_alias_decl(&mut self, n: &TsTypeAliasDecl) {
        let name = n.id.sym.to_string();
        let span = n.id.span;
        self.add_symbol(name, SymbolKind::TypeAlias, span, false);
    }

    fn visit_ts_enum_decl(&mut self, n: &TsEnumDecl) {
        let name = n.id.sym.to_string();
        let span = n.id.span;
        let variants_count = n.members.len();
        self.add_symbol_full(name, SymbolKind::Enum, span, false, false, variants_count);
    }

    fn visit_import_decl(&mut self, n: &ImportDecl) {
        let source = n.src.value.to_string();
        self.add_symbol(source, SymbolKind::Import, n.span, false);
    }

    fn visit_export_decl(&mut self, n: &ExportDecl) {
        match &n.decl {
            Decl::Class(class) => {
                let name = class.ident.sym.to_string();
                // Count class fields for exported classes
                let mut fields_count = 0;
                for member in &class.class.body {
                    if let ClassMember::ClassProp(_) = member {
                        fields_count += 1;
                    }
                }
                self.add_symbol_complete(
                    name,
                    SymbolKind::Class,
                    class.ident.span,
                    true,
                    false,
                    0,
                    fields_count,
                );
                // Also visit the class to collect its members
                self.visit_class_decl(class);
            }
            Decl::Fn(func) => {
                let name = func.ident.sym.to_string();
                let is_async = func.function.is_async;
                self.add_symbol_with_async(
                    name,
                    SymbolKind::Function,
                    func.ident.span,
                    true,
                    is_async,
                );
                // Also visit the function to collect complexity
                self.visit_fn_decl(func);
            }
            Decl::Var(var) => {
                for decl in &var.decls {
                    if let Pat::Ident(ident) = &decl.name {
                        let name = ident.id.sym.to_string();
                        // Check if this is a function variable (arrow function)
                        if let Some(init) = &decl.init {
                            if let Expr::Arrow(arrow) = init.as_ref() {
                                let is_async = arrow.is_async;
                                self.add_symbol_with_async(
                                    name,
                                    SymbolKind::Function,
                                    ident.id.span,
                                    true,
                                    is_async,
                                );
                            } else {
                                self.add_symbol(name, SymbolKind::Variable, ident.id.span, true);
                            }
                        } else {
                            self.add_symbol(name, SymbolKind::Variable, ident.id.span, true);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Complexity calculation visitors
    fn visit_if_stmt(&mut self, n: &IfStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        n.test.visit_with(self);
        n.cons.visit_with(self);
        if let Some(alt) = &n.alt {
            alt.visit_with(self);
        }

        self.exit_nesting();
    }

    fn visit_while_stmt(&mut self, n: &WhileStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        n.test.visit_with(self);
        n.body.visit_with(self);

        self.exit_nesting();
    }

    fn visit_for_stmt(&mut self, n: &ForStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        if let Some(init) = &n.init {
            init.visit_with(self);
        }
        if let Some(test) = &n.test {
            test.visit_with(self);
        }
        if let Some(update) = &n.update {
            update.visit_with(self);
        }
        n.body.visit_with(self);

        self.exit_nesting();
    }

    fn visit_for_in_stmt(&mut self, n: &ForInStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        n.left.visit_with(self);
        n.right.visit_with(self);
        n.body.visit_with(self);

        self.exit_nesting();
    }

    fn visit_for_of_stmt(&mut self, n: &ForOfStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        n.left.visit_with(self);
        n.right.visit_with(self);
        n.body.visit_with(self);

        self.exit_nesting();
    }

    fn visit_switch_stmt(&mut self, n: &SwitchStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        n.discriminant.visit_with(self);
        for case in &n.cases {
            if case.test.is_some() {
                self.add_complexity(1, 1); // +1 for each case
            }
            case.visit_with(self);
        }

        self.exit_nesting();
    }

    fn visit_try_stmt(&mut self, n: &TryStmt) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        n.block.visit_with(self);
        if let Some(handler) = &n.handler {
            self.add_complexity(1, 1); // +1 for catch
            handler.visit_with(self);
        }
        if let Some(finalizer) = &n.finalizer {
            finalizer.visit_with(self);
        }

        self.exit_nesting();
    }

    fn visit_cond_expr(&mut self, n: &CondExpr) {
        self.add_complexity(1, 1); // +1 for ternary
        n.test.visit_with(self);
        n.cons.visit_with(self);
        n.alt.visit_with(self);
    }

    fn visit_bin_expr(&mut self, n: &BinExpr) {
        if matches!(n.op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
            self.add_complexity(1, 1); // +1 for logical operators
        }
        n.left.visit_with(self);
        n.right.visit_with(self);
    }

    fn visit_arrow_expr(&mut self, n: &ArrowExpr) {
        // Arrow functions add complexity
        self.add_complexity(1, 1);

        // Visit arrow function body
        match &*n.body {
            BlockStmtOrExpr::BlockStmt(block) => {
                for stmt in &block.stmts {
                    stmt.visit_with(self);
                }
            }
            BlockStmtOrExpr::Expr(expr) => {
                expr.visit_with(self);
            }
        }
    }
}

#[cfg(feature = "typescript-ast")]
fn parse_typescript_file(path: &Path) -> Result<(Module, Lrc<SourceMap>), TemplateError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| TemplateError::InvalidUtf8(format!("Failed to read file: {e}")))?;

    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(FileName::Real(path.to_path_buf()), source);

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: path.extension().is_some_and(|ext| ext == "tsx"),
            decorators: true,
            dts: path.extension().is_some_and(|ext| ext == "d.ts"),
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("Parse error: {e:?}")))?;

    Ok((module, source_map))
}

#[cfg(feature = "typescript-ast")]
fn parse_javascript_file(path: &Path) -> Result<(Module, Lrc<SourceMap>), TemplateError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| TemplateError::InvalidUtf8(format!("Failed to read file: {e}")))?;

    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(FileName::Real(path.to_path_buf()), source);

    let lexer = Lexer::new(
        Syntax::Es(swc_ecma_parser::EsConfig {
            jsx: path.extension().is_some_and(|ext| ext == "jsx"),
            decorators: true,
            decorators_before_export: true,
            export_default_from: true,
            import_attributes: true,
            auto_accessors: true,
            explicit_resource_management: true,
            allow_return_outside_function: true,
            allow_super_outside_method: true,
            fn_bind: false,
        }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("Parse error: {e:?}")))?;

    Ok((module, source_map))
}

#[cfg(feature = "typescript-ast")]
fn analyze_with_swc(path: &Path, is_typescript: bool) -> Result<FileContext, TemplateError> {
    let (module, source_map) = if is_typescript {
        parse_typescript_file(path)?
    } else {
        parse_javascript_file(path)?
    };

    let mut visitor = TypeScriptVisitor::new(source_map);
    module.visit_with(&mut visitor);

    // Convert symbols to AST items
    let mut items = Vec::new();
    for symbol in visitor.symbols {
        let item = match symbol.kind {
            SymbolKind::Function => AstItem::Function {
                name: symbol.name,
                visibility: if symbol.is_exported {
                    "public"
                } else {
                    "private"
                }
                .to_string(),
                is_async: symbol.is_async,
                line: symbol.line,
            },
            SymbolKind::Class => AstItem::Struct {
                name: symbol.name,
                visibility: if symbol.is_exported {
                    "public"
                } else {
                    "private"
                }
                .to_string(),
                fields_count: symbol.fields_count,
                derives: Vec::new(),
                line: symbol.line,
            },
            SymbolKind::Interface => AstItem::Trait {
                name: symbol.name,
                visibility: if symbol.is_exported {
                    "public"
                } else {
                    "private"
                }
                .to_string(),
                line: symbol.line,
            },
            SymbolKind::TypeAlias => AstItem::Trait {
                name: symbol.name,
                visibility: if symbol.is_exported {
                    "public"
                } else {
                    "private"
                }
                .to_string(),
                line: symbol.line,
            },
            SymbolKind::Enum => AstItem::Enum {
                name: symbol.name,
                visibility: if symbol.is_exported {
                    "public"
                } else {
                    "private"
                }
                .to_string(),
                variants_count: symbol.variants_count,
                line: symbol.line,
            },
            SymbolKind::Variable => {
                // Skip regular variables in item collection as they're not structural elements
                continue;
            }
            SymbolKind::Method => AstItem::Function {
                name: symbol.name,
                visibility: if symbol.is_exported {
                    "public"
                } else {
                    "private"
                }
                .to_string(),
                is_async: symbol.is_async,
                line: symbol.line,
            },
            SymbolKind::Import => AstItem::Use {
                path: symbol.name,
                line: symbol.line,
            },
            _ => continue,
        };
        items.push(item);
    }

    // Create complexity metrics
    let complexity_metrics = FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity: visitor.total_complexity,
        functions: visitor.functions,
        classes: visitor.classes,
    };

    // Detect language based on file extension
    let language = match path.extension().and_then(|s| s.to_str()) {
        Some("tsx") => "tsx",
        Some("jsx") => "jsx",
        Some("ts") => "typescript",
        Some("js") => "javascript",
        _ => {
            if is_typescript {
                "typescript"
            } else {
                "javascript"
            }
        }
    };

    // Create file context
    let context = FileContext {
        path: path.to_string_lossy().to_string(),
        language: language.to_string(),
        items,
        complexity_metrics: Some(complexity_metrics),
    };

    Ok(context)
}

#[cfg(feature = "typescript-ast")]
fn calculate_complexity_with_swc(
    path: &Path,
    is_typescript: bool,
) -> Result<FileComplexityMetrics, TemplateError> {
    let (module, source_map) = if is_typescript {
        parse_typescript_file(path)?
    } else {
        parse_javascript_file(path)?
    };

    let mut visitor = TypeScriptVisitor::new(source_map);
    module.visit_with(&mut visitor);

    let metrics = FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity: visitor.total_complexity,
        functions: visitor.functions,
        classes: visitor.classes,
    };

    Ok(metrics)
}

// Public API functions
pub async fn analyze_typescript_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    #[cfg(feature = "typescript-ast")]
    {
        calculate_complexity_with_swc(path, true)
    }
    #[cfg(not(feature = "typescript-ast"))]
    {
        Err(TemplateError::InvalidUtf8(
            "TypeScript AST feature not enabled. Compile with --features typescript-ast"
                .to_string(),
        ))
    }
}

pub async fn analyze_typescript_file_with_complexity_cached(
    path: &Path,
    _cache_manager: Option<
        std::sync::Arc<crate::services::cache::persistent_manager::PersistentCacheManager>,
    >,
) -> Result<FileComplexityMetrics, TemplateError> {
    // TODO: Implement caching
    analyze_typescript_file_with_complexity(path).await
}

pub async fn analyze_typescript_file(path: &Path) -> Result<FileContext, TemplateError> {
    #[cfg(feature = "typescript-ast")]
    {
        analyze_with_swc(path, true)
    }
    #[cfg(not(feature = "typescript-ast"))]
    {
        Err(TemplateError::InvalidUtf8(
            "TypeScript AST feature not enabled. Compile with --features typescript-ast"
                .to_string(),
        ))
    }
}

pub async fn analyze_javascript_file(path: &Path) -> Result<FileContext, TemplateError> {
    #[cfg(feature = "typescript-ast")]
    {
        analyze_with_swc(path, false)
    }
    #[cfg(not(feature = "typescript-ast"))]
    {
        Err(TemplateError::InvalidUtf8(
            "TypeScript AST feature not enabled. Compile with --features typescript-ast"
                .to_string(),
        ))
    }
}

pub async fn analyze_typescript_file_with_classifier(
    path: &Path,
    _classifier: Option<&crate::services::file_classifier::FileClassifier>,
) -> Result<FileContext, TemplateError> {
    // TODO: Use classifier for enhanced analysis
    analyze_typescript_file(path).await
}

pub async fn analyze_javascript_file_with_classifier(
    path: &Path,
    _classifier: Option<&crate::services::file_classifier::FileClassifier>,
) -> Result<FileContext, TemplateError> {
    // TODO: Use classifier for enhanced analysis
    analyze_javascript_file(path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_javascript_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let js_file = temp_dir.path().join("test.js");

        fs::write(
            &js_file,
            r#"
            function fibonacci(n) {
                if (n <= 1) {
                    return n;
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
            
            class Calculator {
                add(a, b) {
                    return a + b;
                }
                
                subtract(a, b) {
                    return a - b;
                }
            }
            
            export { fibonacci, Calculator };
        "#,
        )
        .unwrap();

        #[cfg(feature = "typescript-ast")]
        {
            let result = analyze_javascript_file(&js_file).await;
            assert!(result.is_ok());
            let context = result.unwrap();
            assert_eq!(context.language, "javascript");
            assert!(!context.items.is_empty());
        }
    }

    #[tokio::test]
    async fn test_typescript_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let ts_file = temp_dir.path().join("test.ts");

        fs::write(
            &ts_file,
            r#"
            interface User {
                name: string;
                age: number;
            }
            
            function greet(user: User): string {
                if (user.age >= 18) {
                    return `Hello, ${user.name}!`;
                } else {
                    return `Hi, ${user.name}!`;
                }
            }
            
            export { User, greet };
        "#,
        )
        .unwrap();

        #[cfg(feature = "typescript-ast")]
        {
            let result = analyze_typescript_file(&ts_file).await;
            assert!(result.is_ok());
            let context = result.unwrap();
            assert_eq!(context.language, "typescript");
            assert!(!context.items.is_empty());
        }
    }

    #[tokio::test]
    async fn test_complexity_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let js_file = temp_dir.path().join("complex.js");

        fs::write(
            &js_file,
            r#"
            function complexFunction(x) {
                if (x > 0) {
                    for (let i = 0; i < x; i++) {
                        if (i % 2 === 0) {
                            console.log(i);
                        } else {
                            console.log("odd");
                        }
                    }
                } else if (x < 0) {
                    while (x < 0) {
                        x++;
                        if (x === -1) {
                            break;
                        }
                    }
                } else {
                    try {
                        throw new Error("Zero not allowed");
                    } catch (e) {
                        console.error(e);
                    }
                }
                return x > 0 ? "positive" : "negative";
            }
        "#,
        )
        .unwrap();

        #[cfg(feature = "typescript-ast")]
        {
            let result = analyze_typescript_file_with_complexity(&js_file).await;
            assert!(result.is_ok());
            let metrics = result.unwrap();
            assert!(metrics.total_complexity.cyclomatic > 1);
            assert!(!metrics.functions.is_empty());
        }
    }
}
