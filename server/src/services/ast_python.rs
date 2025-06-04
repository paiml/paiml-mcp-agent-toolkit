use crate::models::error::TemplateError;
use crate::services::complexity::{
    ClassComplexity, ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use crate::services::file_classifier::{FileClassifier, ParseDecision};
use rustpython_parser::ast;
use std::path::Path;

// Helper functions to reduce code duplication
async fn read_file_content(path: &Path) -> Result<String, TemplateError> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)
}

fn parse_python_content(content: &str, path: &Path) -> Result<ast::Mod, TemplateError> {
    rustpython_parser::parse(
        content,
        rustpython_parser::Mode::Module,
        path.to_str().unwrap_or("<unknown>"),
    )
    .map_err(|e| TemplateError::InvalidUtf8(format!("Python parse error: {:?}", e)))
}

fn calculate_complexity_metrics(ast: &ast::Mod, path: &Path) -> FileComplexityMetrics {
    let mut visitor = PythonComplexityVisitor::new();

    if let ast::Mod::Module(module) = ast {
        for stmt in &module.body {
            visitor.visit_stmt(stmt);
        }
    }

    FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: visitor.file_complexity,
        functions: visitor.functions,
        classes: visitor.classes,
    }
}

fn check_file_classification(
    classifier: &FileClassifier,
    path: &Path,
    content: &str,
) -> Result<(), TemplateError> {
    match classifier.should_parse(path, content.as_bytes()) {
        ParseDecision::Skip(reason) => Err(TemplateError::InvalidUtf8(format!(
            "Skipping file due to {:?}",
            reason
        ))),
        ParseDecision::Parse => Ok(()),
    }
}

fn extract_ast_items(ast: &ast::Mod) -> Vec<AstItem> {
    let mut items = Vec::new();
    if let ast::Mod::Module(module) = ast {
        for stmt in &module.body {
            extract_python_items(stmt, &mut items);
        }
    }
    items
}

pub async fn analyze_python_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    let content = read_file_content(path).await?;
    let ast = parse_python_content(&content, path)?;
    let complexity_metrics = calculate_complexity_metrics(&ast, path);
    Ok(complexity_metrics)
}

pub async fn analyze_python_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_python_file_with_classifier(path, None).await
}

pub async fn analyze_python_file_with_classifier(
    path: &Path,
    classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    let content = read_file_content(path).await?;

    // Check if we should skip this file based on content
    if let Some(classifier) = classifier {
        check_file_classification(classifier, path, &content)?;
    }

    let ast = parse_python_content(&content, path)?;
    let items = extract_ast_items(&ast);
    let complexity_metrics = Some(calculate_complexity_metrics(&ast, path));

    Ok(FileContext {
        path: path.display().to_string(),
        language: "python".to_string(),
        items,
        complexity_metrics,
    })
}

fn extract_python_items(stmt: &ast::Stmt, items: &mut Vec<AstItem>) {
    match stmt {
        ast::Stmt::FunctionDef(func) => {
            items.push(create_function_item(&func.name, false));
        }
        ast::Stmt::AsyncFunctionDef(func) => {
            items.push(create_function_item(&func.name, true));
        }
        ast::Stmt::ClassDef(class) => {
            let attributes_count = count_class_attributes(&class.body);

            // Python classes are similar to structs
            items.push(AstItem::Struct {
                name: class.name.to_string(),
                visibility: "public".to_string(),
                fields_count: attributes_count,
                derives: vec![], // Python doesn't have derives like Rust
                line: 1,         // TODO: Extract actual line numbers from AST
            });

            // Also extract methods from the class
            for stmt in &class.body {
                extract_python_items(stmt, items);
            }
        }
        ast::Stmt::Import(import) => {
            for alias in &import.names {
                items.push(AstItem::Use {
                    path: alias.name.to_string(),
                    line: 1, // TODO: Extract actual line numbers from AST
                });
            }
        }
        ast::Stmt::ImportFrom(import_from) => {
            extract_import_from_items(import_from, items);
        }
        _ => {
            // Handle other statement types if needed
        }
    }
}

struct PythonComplexityVisitor {
    file_complexity: ComplexityMetrics,
    functions: Vec<FunctionComplexity>,
    classes: Vec<ClassComplexity>,
    current_function_complexity: Option<ComplexityMetrics>,
    current_function_name: Option<String>,
    current_function_start: u32,
    current_class: Option<ClassComplexity>,
    nesting_level: u8,
}

impl PythonComplexityVisitor {
    fn new() -> Self {
        Self {
            file_complexity: ComplexityMetrics::default(),
            functions: Vec::new(),
            classes: Vec::new(),
            current_function_complexity: None,
            current_function_name: None,
            current_function_start: 0,
            current_class: None,
            nesting_level: 0,
        }
    }

    fn enter_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_add(1);
        if self.nesting_level > self.file_complexity.nesting_max {
            self.file_complexity.nesting_max = self.nesting_level;
        }
        if let Some(ref mut func) = self.current_function_complexity {
            if self.nesting_level > func.nesting_max {
                func.nesting_max = self.nesting_level;
            }
        }
    }

    fn exit_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_sub(1);
    }

    fn add_complexity(&mut self, cyclomatic: u16, cognitive_base: u16) {
        // Add to file complexity
        self.file_complexity.cyclomatic =
            self.file_complexity.cyclomatic.saturating_add(cyclomatic);

        // Calculate cognitive complexity based on nesting
        let cognitive = if self.nesting_level > 0 {
            cognitive_base + self.nesting_level.saturating_sub(1) as u16
        } else {
            cognitive_base
        };
        self.file_complexity.cognitive = self.file_complexity.cognitive.saturating_add(cognitive);

        // Add to current function if we're in one
        if let Some(ref mut func) = self.current_function_complexity {
            func.cyclomatic = func.cyclomatic.saturating_add(cyclomatic);
            func.cognitive = func.cognitive.saturating_add(cognitive);
        }
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::FunctionDef(func) => {
                self.visit_function_def(&func.name, &func.body, false);
            }
            ast::Stmt::AsyncFunctionDef(func) => {
                self.visit_function_def(&func.name, &func.body, true);
            }
            ast::Stmt::ClassDef(class) => {
                self.visit_class_def(class);
            }
            ast::Stmt::If(if_stmt) => {
                self.visit_if_stmt(if_stmt);
            }
            ast::Stmt::For(for_stmt) => {
                self.visit_for_stmt(for_stmt);
            }
            ast::Stmt::AsyncFor(for_stmt) => {
                self.visit_async_for_stmt(for_stmt);
            }
            ast::Stmt::While(while_stmt) => {
                self.visit_while_stmt(while_stmt);
            }
            ast::Stmt::Match(match_stmt) => {
                self.visit_match_stmt(match_stmt);
            }
            ast::Stmt::Try(try_stmt) => {
                self.visit_try_stmt(try_stmt);
            }
            ast::Stmt::With(with_stmt) => {
                self.visit_with_stmt(with_stmt);
            }
            ast::Stmt::AsyncWith(with_stmt) => {
                self.visit_async_with_stmt(with_stmt);
            }
            _ => {
                // For other statements, just visit children if any
                if let ast::Stmt::Expr(expr) = stmt {
                    self.visit_expr(&expr.value)
                }
            }
        }
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::BoolOp(bool_op) => {
                // and/or operators add complexity
                self.add_complexity(1, 1);
                for value in &bool_op.values {
                    self.visit_expr(value);
                }
            }
            ast::Expr::IfExp(if_exp) => {
                // Ternary operator adds complexity
                self.add_complexity(1, 1);
                self.visit_expr(&if_exp.test);
                self.visit_expr(&if_exp.body);
                self.visit_expr(&if_exp.orelse);
            }
            ast::Expr::Lambda(lambda) => {
                // Lambda functions add complexity
                self.add_complexity(1, 1);
                self.visit_expr(&lambda.body);
            }
            _ => {
                // Visit other expressions recursively
            }
        }
    }

    fn visit_function_def(&mut self, name: &str, body: &[ast::Stmt], _is_async: bool) {
        let prev_func = self.current_function_complexity.take();
        let prev_name = self.current_function_name.take();

        self.current_function_complexity = Some(ComplexityMetrics::default());
        self.current_function_name = Some(name.to_string());
        self.current_function_start = 1; // Would need source location for real line numbers

        // Visit function body
        for stmt in body {
            self.visit_stmt(stmt);
        }

        // Save function complexity
        if let Some(complexity) = self.current_function_complexity.take() {
            if let Some(name) = self.current_function_name.take() {
                if let Some(ref mut class) = self.current_class {
                    // This is a method in a class
                    class.methods.push(FunctionComplexity {
                        name,
                        line_start: self.current_function_start,
                        line_end: self.current_function_start + 10, // Estimate
                        metrics: complexity,
                    });
                    // Add to class total
                    class.metrics.cyclomatic += complexity.cyclomatic;
                    class.metrics.cognitive += complexity.cognitive;
                } else {
                    // Top-level function
                    self.functions.push(FunctionComplexity {
                        name,
                        line_start: self.current_function_start,
                        line_end: self.current_function_start + 10, // Estimate
                        metrics: complexity,
                    });
                }
            }
        }

        // Restore previous function context
        self.current_function_complexity = prev_func;
        self.current_function_name = prev_name;
    }

    fn visit_class_def(&mut self, class: &ast::StmtClassDef) {
        let class_complexity = ClassComplexity {
            name: class.name.to_string(),
            line_start: 1,
            line_end: 100, // Estimate
            metrics: ComplexityMetrics::default(),
            methods: Vec::new(),
        };

        let prev_class = self.current_class.take();
        self.current_class = Some(class_complexity.clone());

        // Visit class body
        for stmt in &class.body {
            self.visit_stmt(stmt);
        }

        if let Some(class_complexity) = self.current_class.take() {
            self.classes.push(class_complexity);
        }

        self.current_class = prev_class;
    }

    fn visit_if_stmt(&mut self, if_stmt: &ast::StmtIf) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        self.visit_expr(&if_stmt.test);
        for stmt in &if_stmt.body {
            self.visit_stmt(stmt);
        }

        for stmt in &if_stmt.orelse {
            // elif/else adds complexity
            if matches!(stmt, ast::Stmt::If(_)) {
                self.add_complexity(1, 1);
            }
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }

    fn visit_for_stmt(&mut self, for_stmt: &ast::StmtFor) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        self.visit_expr(&for_stmt.iter);
        for stmt in &for_stmt.body {
            self.visit_stmt(stmt);
        }

        for stmt in &for_stmt.orelse {
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }

    fn visit_async_for_stmt(&mut self, for_stmt: &ast::StmtAsyncFor) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        self.visit_expr(&for_stmt.iter);
        for stmt in &for_stmt.body {
            self.visit_stmt(stmt);
        }

        for stmt in &for_stmt.orelse {
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }

    fn visit_while_stmt(&mut self, while_stmt: &ast::StmtWhile) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        self.visit_expr(&while_stmt.test);
        for stmt in &while_stmt.body {
            self.visit_stmt(stmt);
        }

        for stmt in &while_stmt.orelse {
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }

    fn visit_match_stmt(&mut self, match_stmt: &ast::StmtMatch) {
        self.add_complexity(1, 1);
        self.enter_nesting();

        self.visit_expr(&match_stmt.subject);

        for case in &match_stmt.cases {
            // Each case adds complexity
            self.add_complexity(1, 1);

            // Visit case pattern and guard
            if let Some(guard) = &case.guard {
                self.visit_expr(guard);
            }

            // Visit case body
            for stmt in &case.body {
                self.visit_stmt(stmt);
            }
        }

        self.exit_nesting();
    }

    fn visit_try_stmt(&mut self, try_stmt: &ast::StmtTry) {
        self.enter_nesting();

        for stmt in &try_stmt.body {
            self.visit_stmt(stmt);
        }

        for handler in &try_stmt.handlers {
            // except clause adds complexity
            self.add_complexity(1, 1);

            match handler {
                ast::ExceptHandler::ExceptHandler(eh) => {
                    for stmt in &eh.body {
                        self.visit_stmt(stmt);
                    }
                }
            }
        }

        for stmt in &try_stmt.orelse {
            self.visit_stmt(stmt);
        }

        for stmt in &try_stmt.finalbody {
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }

    fn visit_with_stmt(&mut self, with_stmt: &ast::StmtWith) {
        self.enter_nesting();

        for stmt in &with_stmt.body {
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }

    fn visit_async_with_stmt(&mut self, with_stmt: &ast::StmtAsyncWith) {
        self.enter_nesting();

        for stmt in &with_stmt.body {
            self.visit_stmt(stmt);
        }

        self.exit_nesting();
    }
}

// Additional helper functions to reduce code duplication
fn create_function_item(name: &str, is_async: bool) -> AstItem {
    AstItem::Function {
        name: name.to_string(),
        visibility: if name.starts_with('_') {
            "private".to_string()
        } else {
            "public".to_string()
        },
        is_async,
        line: 1, // TODO: Extract actual line numbers from AST
    }
}

fn count_class_attributes(body: &[ast::Stmt]) -> usize {
    body.iter()
        .filter(|stmt| matches!(stmt, ast::Stmt::AnnAssign(_) | ast::Stmt::Assign(_)))
        .count()
}

fn extract_import_from_items(import_from: &ast::StmtImportFrom, items: &mut Vec<AstItem>) {
    if let Some(module) = &import_from.module {
        let base_path = module.to_string();
        for alias in &import_from.names {
            items.push(AstItem::Use {
                path: format!("{}.{}", base_path, alias.name),
                line: 1, // TODO: Extract actual line numbers from AST
            });
        }
    }
}
