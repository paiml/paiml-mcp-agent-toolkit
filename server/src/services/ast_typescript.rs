use crate::models::error::TemplateError;
use crate::services::complexity::{
    ClassComplexity, ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use crate::services::file_classifier::{FileClassifier, ParseDecision};
use std::path::Path;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

pub async fn analyze_typescript_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    analyze_typescript_file_with_complexity_cached(path, None).await
}

pub async fn analyze_typescript_file_with_complexity_cached(
    path: &Path,
    _cache_manager: Option<
        std::sync::Arc<crate::services::cache::persistent_manager::PersistentCacheManager>,
    >,
) -> Result<FileComplexityMetrics, TemplateError> {
    // For now, we'll skip caching until the cache interfaces are updated for complexity metrics
    // This is a placeholder for future cache integration
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom(path.display().to_string())),
        content,
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: path.extension().and_then(|s| s.to_str()) == Some("tsx"),
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("TypeScript parse error: {:?}", e)))?;

    let mut visitor = TypeScriptVisitor::new();
    visitor.enable_complexity = true;
    module.visit_with(&mut visitor);

    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: visitor.file_complexity,
        functions: visitor.functions,
        classes: visitor.classes,
    })
}

pub async fn analyze_javascript_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom(path.display().to_string())),
        content,
    );

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("JavaScript parse error: {:?}", e)))?;

    let mut visitor = TypeScriptVisitor::new();
    visitor.enable_complexity = true;
    module.visit_with(&mut visitor);

    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: visitor.file_complexity,
        functions: visitor.functions,
        classes: visitor.classes,
    })
}

pub async fn analyze_typescript_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_typescript_file_with_classifier(path, None).await
}

pub async fn analyze_typescript_file_with_classifier(
    path: &Path,
    classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Check if we should skip this file based on content
    if let Some(classifier) = classifier {
        match classifier.should_parse(path, content.as_bytes()) {
            ParseDecision::Skip(reason) => {
                return Err(TemplateError::InvalidUtf8(format!(
                    "Skipping file due to {:?}",
                    reason
                )));
            }
            ParseDecision::Parse => {}
        }
    }

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom(path.display().to_string())),
        content,
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: path.extension().and_then(|s| s.to_str()) == Some("tsx"),
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("TypeScript parse error: {:?}", e)))?;

    let mut visitor = TypeScriptVisitor::new();
    module.visit_with(&mut visitor);

    let complexity_metrics = if visitor.enable_complexity {
        Some(FileComplexityMetrics {
            path: path.display().to_string(),
            total_complexity: visitor.file_complexity,
            functions: visitor.functions,
            classes: visitor.classes,
        })
    } else {
        None
    };

    Ok(FileContext {
        path: path.display().to_string(),
        language: if path.extension().and_then(|s| s.to_str()) == Some("tsx") {
            "tsx".to_string()
        } else {
            "typescript".to_string()
        },
        items: visitor.items,
        complexity_metrics,
    })
}

pub async fn analyze_javascript_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_javascript_file_with_classifier(path, None).await
}

pub async fn analyze_javascript_file_with_classifier(
    path: &Path,
    classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Check if we should skip this file based on content
    if let Some(classifier) = classifier {
        match classifier.should_parse(path, content.as_bytes()) {
            ParseDecision::Skip(reason) => {
                return Err(TemplateError::InvalidUtf8(format!(
                    "Skipping file due to {:?}",
                    reason
                )));
            }
            ParseDecision::Parse => {}
        }
    }

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom(path.display().to_string())),
        content,
    );

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| TemplateError::InvalidUtf8(format!("JavaScript parse error: {:?}", e)))?;

    let mut visitor = TypeScriptVisitor::new();
    module.visit_with(&mut visitor);

    let complexity_metrics = if visitor.enable_complexity {
        Some(FileComplexityMetrics {
            path: path.display().to_string(),
            total_complexity: visitor.file_complexity,
            functions: visitor.functions,
            classes: visitor.classes,
        })
    } else {
        None
    };

    Ok(FileContext {
        path: path.display().to_string(),
        language: if path.extension().and_then(|s| s.to_str()) == Some("jsx") {
            "jsx".to_string()
        } else {
            "javascript".to_string()
        },
        items: visitor.items,
        complexity_metrics,
    })
}

struct TypeScriptVisitor {
    items: Vec<AstItem>,
    enable_complexity: bool,
    file_complexity: ComplexityMetrics,
    functions: Vec<FunctionComplexity>,
    classes: Vec<ClassComplexity>,
    current_function_complexity: Option<ComplexityMetrics>,
    current_function_name: Option<String>,
    current_function_start: u32,
    current_class: Option<ClassComplexity>,
    nesting_level: u8,
}

impl TypeScriptVisitor {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            enable_complexity: true,
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

    fn count_class_fields(&self, class_body: &[ClassMember]) -> usize {
        let mut fields_count = 0;
        for member in class_body {
            match member {
                ClassMember::ClassProp(_) | ClassMember::PrivateProp(_) => fields_count += 1,
                _ => {}
            }
        }
        fields_count
    }

    fn add_class_to_items(&mut self, class_name: &str, fields_count: usize) {
        self.items.push(AstItem::Struct {
            name: class_name.to_string(),
            visibility: "public".to_string(),
            fields_count,
            derives: vec![], // TypeScript uses decorators, not derives
            line: 1,
        });
    }

    fn process_class_complexity(&mut self, class_name: &str, class_body: &[ClassMember]) {
        let class_complexity = ClassComplexity {
            name: class_name.to_string(),
            line_start: 1,
            line_end: 100, // Estimate
            metrics: ComplexityMetrics::default(),
            methods: Vec::new(),
        };

        self.current_class = Some(class_complexity.clone());
        self.process_class_members(class_body);

        if let Some(class_complexity) = self.current_class.take() {
            self.classes.push(class_complexity);
        }
    }

    fn process_class_members(&mut self, class_body: &[ClassMember]) {
        for member in class_body {
            match member {
                ClassMember::Method(method) => self.process_method(method),
                ClassMember::Constructor(constructor) => self.process_constructor(constructor),
                _ => {}
            }
        }
    }

    fn process_method(&mut self, method: &ClassMethod) {
        let method_name = self.extract_method_name(&method.key);
        self.analyze_function_complexity(&method_name, |visitor| {
            method.function.visit_children_with(visitor);
        });
    }

    fn process_constructor(&mut self, constructor: &Constructor) {
        self.analyze_function_complexity("constructor", |visitor| {
            constructor.visit_children_with(visitor);
        });
    }

    fn extract_method_name(&self, key: &PropName) -> String {
        match key {
            PropName::Ident(ident) => ident.sym.to_string(),
            PropName::Str(s) => s.value.to_string(),
            _ => "anonymous".to_string(),
        }
    }

    fn analyze_function_complexity<F>(&mut self, function_name: &str, visit_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        self.current_function_complexity = Some(ComplexityMetrics::default());
        self.current_function_name = Some(function_name.to_string());
        self.current_function_start = 1;

        visit_fn(self);

        if let Some(complexity) = self.current_function_complexity.take() {
            if let Some(ref mut current_class) = self.current_class {
                current_class.methods.push(FunctionComplexity {
                    name: function_name.to_string(),
                    line_start: self.current_function_start,
                    line_end: self.current_function_start + 10,
                    metrics: complexity,
                });
                // Add method complexity to class total
                current_class.metrics.cyclomatic += complexity.cyclomatic;
                current_class.metrics.cognitive += complexity.cognitive;
            }
        }
    }

    fn process_variable_declaration(&mut self, decl: &VarDeclarator) {
        if let Pat::Ident(ident) = &decl.name {
            if let Some(init) = &decl.init {
                match &**init {
                    Expr::Arrow(arrow) => self.process_arrow_function(ident, arrow),
                    Expr::Fn(fn_expr) => self.process_function_expression(fn_expr),
                    _ => {}
                }
            }
        }
    }

    fn process_arrow_function(&mut self, ident: &BindingIdent, arrow: &ArrowExpr) {
        let name = ident.id.sym.to_string();
        self.add_function_to_items(&name, arrow.is_async);

        if self.enable_complexity {
            self.analyze_standalone_function(&name, |visitor| {
                arrow.visit_children_with(visitor);
            });
        }
    }

    fn process_function_expression(&mut self, fn_expr: &FnExpr) {
        if let Some(ident) = &fn_expr.ident {
            let name = ident.sym.to_string();
            self.add_function_to_items(&name, fn_expr.function.is_async);

            if self.enable_complexity {
                self.analyze_standalone_function(&name, |visitor| {
                    fn_expr.function.visit_children_with(visitor);
                });
            }
        }
    }

    fn add_function_to_items(&mut self, name: &str, is_async: bool) {
        self.items.push(AstItem::Function {
            name: name.to_string(),
            visibility: "public".to_string(),
            is_async,
            line: 1,
        });
    }

    fn analyze_standalone_function<F>(&mut self, function_name: &str, visit_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        self.current_function_complexity = Some(ComplexityMetrics::default());
        self.current_function_name = Some(function_name.to_string());
        self.current_function_start = 1;

        visit_fn(self);

        if let Some(complexity) = self.current_function_complexity.take() {
            if let Some(name) = self.current_function_name.take() {
                self.functions.push(FunctionComplexity {
                    name,
                    line_start: self.current_function_start,
                    line_end: self.current_function_start + 10,
                    metrics: complexity,
                });
            }
        }
    }

    #[allow(dead_code)]
    fn count_lines(&mut self) {
        if let Some(ref mut func) = self.current_function_complexity {
            func.lines = func.lines.saturating_add(1);
        }
        self.file_complexity.lines = self.file_complexity.lines.saturating_add(1);
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
}

impl Visit for TypeScriptVisitor {
    fn visit_fn_decl(&mut self, node: &FnDecl) {
        let name = node.ident.sym.to_string();
        self.items.push(AstItem::Function {
            name: name.clone(),
            visibility: "public".to_string(), // JS/TS doesn't have visibility modifiers like Rust
            is_async: node.function.is_async,
            line: 1, // SWC doesn't provide line numbers easily
        });

        if self.enable_complexity {
            self.current_function_complexity = Some(ComplexityMetrics::default());
            self.current_function_name = Some(name);
            self.current_function_start = 1; // Would need source map for real line numbers

            // Visit function body to calculate complexity
            node.function.visit_children_with(self);

            // Save function complexity
            if let Some(complexity) = self.current_function_complexity.take() {
                if let Some(name) = self.current_function_name.take() {
                    self.functions.push(FunctionComplexity {
                        name,
                        line_start: self.current_function_start,
                        line_end: self.current_function_start + 10, // Estimate
                        metrics: complexity,
                    });
                }
            }
        }
    }

    fn visit_class_decl(&mut self, node: &ClassDecl) {
        let class_name = node.ident.sym.to_string();
        let fields_count = self.count_class_fields(&node.class.body);

        self.add_class_to_items(&class_name, fields_count);

        if self.enable_complexity {
            self.process_class_complexity(&class_name, &node.class.body);
        }
    }

    fn visit_var_decl(&mut self, node: &VarDecl) {
        // Track exported const functions (common pattern in JS/TS)
        for decl in &node.decls {
            self.process_variable_declaration(decl);
        }
    }

    fn visit_ts_interface_decl(&mut self, node: &TsInterfaceDecl) {
        // Treat interfaces as traits
        self.items.push(AstItem::Trait {
            name: node.id.sym.to_string(),
            visibility: "public".to_string(),
            line: 1,
        });
    }

    fn visit_ts_type_alias_decl(&mut self, node: &TsTypeAliasDecl) {
        // Treat type aliases as a special kind of struct
        self.items.push(AstItem::Struct {
            name: node.id.sym.to_string(),
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        });
    }

    fn visit_ts_enum_decl(&mut self, node: &TsEnumDecl) {
        self.items.push(AstItem::Enum {
            name: node.id.sym.to_string(),
            visibility: "public".to_string(),
            variants_count: node.members.len(),
            line: 1,
        });
    }

    fn visit_import_decl(&mut self, node: &ImportDecl) {
        self.items.push(AstItem::Use {
            path: node.src.value.to_string(),
            line: 1,
        });
    }

    fn visit_export_decl(&mut self, node: &ExportDecl) {
        // Visit the declaration inside the export
        match &node.decl {
            Decl::Fn(fn_decl) => self.visit_fn_decl(fn_decl),
            Decl::Class(class_decl) => self.visit_class_decl(class_decl),
            Decl::Var(var_decl) => self.visit_var_decl(var_decl),
            Decl::TsInterface(interface) => self.visit_ts_interface_decl(interface),
            Decl::TsTypeAlias(type_alias) => self.visit_ts_type_alias_decl(type_alias),
            Decl::TsEnum(enum_decl) => self.visit_ts_enum_decl(enum_decl),
            _ => {}
        }
    }

    // Control flow statements for complexity calculation
    fn visit_if_stmt(&mut self, node: &IfStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.test.visit_with(self);
        node.cons.visit_with(self);

        if let Some(alt) = &node.alt {
            if self.enable_complexity {
                // else adds complexity but not nesting
                self.add_complexity(1, 1);
            }
            alt.visit_with(self);
        }

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_switch_stmt(&mut self, node: &SwitchStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.discriminant.visit_with(self);

        for case in &node.cases {
            if self.enable_complexity && case.test.is_some() {
                self.add_complexity(1, 1);
            }
            case.visit_with(self);
        }

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_for_stmt(&mut self, node: &ForStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.visit_children_with(self);

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_for_in_stmt(&mut self, node: &ForInStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.visit_children_with(self);

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_for_of_stmt(&mut self, node: &ForOfStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.visit_children_with(self);

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_while_stmt(&mut self, node: &WhileStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.visit_children_with(self);

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_do_while_stmt(&mut self, node: &DoWhileStmt) {
        if self.enable_complexity {
            self.add_complexity(1, 1);
            self.enter_nesting();
        }

        node.visit_children_with(self);

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_try_stmt(&mut self, node: &TryStmt) {
        if self.enable_complexity {
            self.enter_nesting();
        }

        node.block.visit_with(self);

        if let Some(handler) = &node.handler {
            if self.enable_complexity {
                self.add_complexity(1, 1);
            }
            handler.visit_with(self);
        }

        if let Some(finalizer) = &node.finalizer {
            finalizer.visit_with(self);
        }

        if self.enable_complexity {
            self.exit_nesting();
        }
    }

    fn visit_bin_expr(&mut self, node: &BinExpr) {
        // Logical operators add complexity
        if self.enable_complexity {
            match node.op {
                BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                    self.add_complexity(1, 1);
                }
                _ => {}
            }
        }

        node.visit_children_with(self);
    }

    fn visit_cond_expr(&mut self, node: &CondExpr) {
        // Ternary operator adds complexity
        if self.enable_complexity {
            self.add_complexity(1, 1);
        }

        node.visit_children_with(self);
    }
}
