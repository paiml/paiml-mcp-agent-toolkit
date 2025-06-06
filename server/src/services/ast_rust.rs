use crate::models::error::TemplateError;
use crate::services::complexity::{
    ClassComplexity, ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use crate::services::file_classifier::{FileClassifier, ParseDecision};
use std::path::Path;
use syn::{
    visit::Visit, Arm, Expr, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, ItemFn, ItemImpl,
    ItemStruct, ItemTrait, Stmt,
};

pub async fn analyze_rust_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    analyze_rust_file_with_complexity_and_classifier(path, None).await
}

pub async fn analyze_rust_file_with_complexity_and_classifier(
    path: &Path,
    classifier: Option<&FileClassifier>,
) -> Result<FileComplexityMetrics, TemplateError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Check if we should skip this file based on content
    if let Some(classifier) = classifier {
        match classifier.should_parse(path, content.as_bytes()) {
            ParseDecision::Skip(reason) => {
                return Err(TemplateError::InvalidUtf8(format!(
                    "Skipping file due to {reason:?}"
                )));
            }
            ParseDecision::Parse => {}
        }
    }

    let ast = syn::parse_file(&content)
        .map_err(|e| TemplateError::InvalidUtf8(format!("Rust parse error: {e}")))?;

    let mut visitor = RustComplexityVisitor::new();
    visitor.visit_file(&ast);

    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: visitor.file_complexity,
        functions: visitor.functions,
        classes: visitor.structs, // In Rust, structs are like classes
    })
}

pub async fn analyze_rust_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_rust_file_with_classifier(path, None).await
}

pub async fn analyze_rust_file_with_classifier(
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
                    "Skipping file due to {reason:?}"
                )));
            }
            ParseDecision::Parse => {}
        }
    }

    let ast = syn::parse_file(&content)
        .map_err(|e| TemplateError::InvalidUtf8(format!("Rust parse error: {e}")))?;

    let mut visitor = RustComplexityVisitor::new();
    visitor.enable_complexity = false; // Only collect AST items, not complexity
    visitor.visit_file(&ast);

    Ok(FileContext {
        path: path.display().to_string(),
        language: "rust".to_string(),
        items: visitor.items,
        complexity_metrics: None,
    })
}

struct RustComplexityVisitor {
    items: Vec<AstItem>,
    enable_complexity: bool,
    file_complexity: ComplexityMetrics,
    functions: Vec<FunctionComplexity>,
    structs: Vec<ClassComplexity>, // Using ClassComplexity for structs
    current_function_complexity: Option<ComplexityMetrics>,
    current_function_name: Option<String>,
    current_function_start: u32,
    #[allow(dead_code)]
    current_struct: Option<ClassComplexity>,
    nesting_level: u8,
}

impl RustComplexityVisitor {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            enable_complexity: true,
            file_complexity: ComplexityMetrics::default(),
            functions: Vec::new(),
            structs: Vec::new(),
            current_function_complexity: None,
            current_function_name: None,
            current_function_start: 0,
            current_struct: None,
            nesting_level: 0,
        }
    }

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

    fn get_visibility_string(vis: &syn::Visibility) -> String {
        match vis {
            syn::Visibility::Public(_) => "public".to_string(),
            syn::Visibility::Restricted(_) => "restricted".to_string(),
            syn::Visibility::Inherited => "private".to_string(),
        }
    }

    fn count_fields(fields: &Fields) -> usize {
        match fields {
            Fields::Named(FieldsNamed { named, .. }) => named.len(),
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed.len(),
            Fields::Unit => 0,
        }
    }
}

impl<'ast> Visit<'ast> for RustComplexityVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();
        let visibility = Self::get_visibility_string(&node.vis);
        let is_async = node.sig.asyncness.is_some();

        self.items.push(AstItem::Function {
            name: name.clone(),
            visibility,
            is_async,
            line: 1, // syn doesn't provide line numbers easily
        });

        if self.enable_complexity {
            self.current_function_complexity = Some(ComplexityMetrics::default());
            self.current_function_name = Some(name);
            self.current_function_start = 1;

            // Visit function body to calculate complexity
            self.visit_block(&node.block);

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
        } else {
            // Just visit children normally
            syn::visit::visit_item_fn(self, node);
        }
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = node.ident.to_string();
        let visibility = Self::get_visibility_string(&node.vis);
        let fields_count = Self::count_fields(&node.fields);

        // Extract derives from attributes (simplified)
        let mut derives = Vec::new();
        for attr in &node.attrs {
            if attr.path().is_ident("derive") {
                // For now, just indicate that there are derives without parsing them
                derives.push("derive".to_string());
            }
        }

        self.items.push(AstItem::Struct {
            name: name.clone(),
            visibility,
            fields_count,
            derives,
            line: 1,
        });

        if self.enable_complexity {
            let struct_complexity = ClassComplexity {
                name,
                line_start: 1,
                line_end: 100, // Estimate
                metrics: ComplexityMetrics::default(),
                methods: Vec::new(),
            };

            self.structs.push(struct_complexity);
        }

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let name = node.ident.to_string();
        let visibility = Self::get_visibility_string(&node.vis);
        let variants_count = node.variants.len();

        self.items.push(AstItem::Enum {
            name,
            visibility,
            variants_count,
            line: 1,
        });

        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        let name = node.ident.to_string();
        let visibility = Self::get_visibility_string(&node.vis);

        self.items.push(AstItem::Trait {
            name,
            visibility,
            line: 1,
        });

        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        // Handle impl blocks - these can contain methods for structs
        if let syn::Type::Path(type_path) = &*node.self_ty {
            if let Some(segment) = type_path.path.segments.last() {
                let struct_name = segment.ident.to_string();

                if self.enable_complexity {
                    // Find the corresponding struct and add methods
                    for item in &node.items {
                        if let syn::ImplItem::Fn(method) = item {
                            let method_name = method.sig.ident.to_string();

                            self.current_function_complexity = Some(ComplexityMetrics::default());
                            self.current_function_name = Some(method_name.clone());
                            self.current_function_start = 1;

                            self.visit_block(&method.block);

                            if let Some(complexity) = self.current_function_complexity.take() {
                                // Add to functions list for now (could be enhanced to link to structs)
                                self.functions.push(FunctionComplexity {
                                    name: format!("{struct_name}::{method_name}"),
                                    line_start: self.current_function_start,
                                    line_end: self.current_function_start + 10,
                                    metrics: complexity,
                                });
                            }
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        // Extract the use path (simplified for now)
        let path = "use statement".to_string(); // Simplified placeholder

        self.items.push(AstItem::Use { path, line: 1 });

        syn::visit::visit_item_use(self, node);
    }

    // Control flow statements for complexity calculation
    fn visit_expr(&mut self, node: &'ast Expr) {
        if self.enable_complexity {
            match node {
                Expr::If(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                    return;
                }
                Expr::Match(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                    return;
                }
                Expr::While(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                    return;
                }
                Expr::ForLoop(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                    return;
                }
                Expr::Loop(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                    return;
                }
                Expr::Try(_) => {
                    self.add_complexity(1, 1);
                    syn::visit::visit_expr(self, node);
                    return;
                }
                Expr::Binary(bin_expr) => {
                    // Logical operators add complexity
                    match bin_expr.op {
                        syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                            self.add_complexity(1, 1);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        syn::visit::visit_expr(self, node);
    }

    fn visit_arm(&mut self, node: &'ast Arm) {
        // Each match arm adds complexity
        if self.enable_complexity {
            self.add_complexity(1, 1);
        }

        syn::visit::visit_arm(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        if self.enable_complexity {
            self.count_lines();
        }

        syn::visit::visit_stmt(self, node);
    }
}
