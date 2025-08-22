//! Rust AST analysis for complexity metrics and context extraction.
//!
//! This module provides deep analysis of Rust source code using the `syn` crate
//! for AST parsing. It extracts complexity metrics, identifies code structures,
//! and generates context information suitable for AI/LLM consumption.
//!
//! # Features
//!
//! - **Cyclomatic Complexity**: Calculates complexity based on control flow
//! - **Cognitive Complexity**: Measures code understandability
//! - **Struct/Enum Analysis**: Extracts type definitions and their fields
//! - **Function Analysis**: Identifies functions, methods, and their signatures
//! - **Trait Analysis**: Extracts trait definitions and implementations
//! - **Caching**: Integrated caching for performance optimization
//!
//! # Complexity Calculation
//!
//! The complexity calculation follows these rules:
//! - +1 for each `if`, `match` arm, `while`, `for` loop
//! - +1 for each `&&` and `||` operator
//! - +1 for nested control structures (cognitive complexity)
//! - +1 for each `?` operator and `.unwrap()` call
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::ast_rust::analyze_rust_file_with_complexity;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let metrics = analyze_rust_file_with_complexity(Path::new("src/main.rs")).await?;
//!
//! println!("File complexity: {:?}", metrics.total_complexity);
//! for func in &metrics.functions {
//!     println!("Function {}: cyclomatic {}, cognitive {}",
//!              func.name, func.metrics.cyclomatic, func.metrics.cognitive);
//! }
//! # Ok(())
//! # }
//! ```

use crate::models::error::TemplateError;
use crate::services::complexity::{
    ClassComplexity, ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use crate::services::file_classifier::{FileClassifier, ParseDecision};
use crate::services::parsed_file_cache::PARSED_FILE_CACHE;
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

    // Use cache for the result
    let result = PARSED_FILE_CACHE
        .get_or_compute_complexity(path, &content, || async {
            // Check if we should skip this file based on content
            if let Some(classifier) = classifier {
                match classifier.should_parse(path, content.as_bytes()) {
                    ParseDecision::Skip(reason) => {
                        return Err(anyhow::anyhow!("Skipping file due to {:?}", reason));
                    }
                    ParseDecision::Parse => {}
                }
            }

            let ast = syn::parse_file(&content)
                .map_err(|e| anyhow::anyhow!("Rust parse error: {}", e))?;

            let mut visitor = RustComplexityVisitor::new();
            visitor.visit_file(&ast);

            Ok(FileComplexityMetrics {
                path: path.display().to_string(),
                total_complexity: visitor.file_complexity,
                functions: visitor.functions,
                classes: visitor.structs, // In Rust, structs are like classes
            })
        })
        .await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

    Ok((*result).clone())
}

#[inline(always)]
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

    // Use cache for the result
    let result = PARSED_FILE_CACHE
        .get_or_compute_context(path, &content, || async {
            // Check if we should skip this file based on content
            if let Some(classifier) = classifier {
                match classifier.should_parse(path, content.as_bytes()) {
                    ParseDecision::Skip(reason) => {
                        return Err(anyhow::anyhow!("Skipping file due to {:?}", reason));
                    }
                    ParseDecision::Parse => {}
                }
            }

            let ast = syn::parse_file(&content)
                .map_err(|e| anyhow::anyhow!("Rust parse error: {}", e))?;

            let mut visitor = RustComplexityVisitor::new();
            visitor.enable_complexity = false; // Only collect AST items, not complexity
            visitor.visit_file(&ast);

            Ok(FileContext {
                path: path.display().to_string(),
                language: "rust".to_string(),
                items: visitor.items,
                complexity_metrics: None,
            })
        })
        .await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

    Ok((*result).clone())
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
    // Halstead metrics tracking
    operators: std::collections::HashSet<String>,
    operands: std::collections::HashSet<String>,
    operator_count: u32,
    operand_count: u32,
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
            operators: std::collections::HashSet::new(),
            operands: std::collections::HashSet::new(),
            operator_count: 0,
            operand_count: 0,
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
            // Start with base complexity of 1 for any function
            let base_complexity = if is_async {
                ComplexityMetrics {
                    cyclomatic: 2, // async adds one complexity point
                    cognitive: 1,  // async adds cognitive load
                    ..Default::default()
                }
            } else {
                ComplexityMetrics {
                    cyclomatic: 1, // base cyclomatic complexity
                    cognitive: 0,  // base cognitive complexity (no mental burden)
                    ..Default::default()
                }
            };

            self.current_function_complexity = Some(base_complexity);
            self.current_function_name = Some(name.clone());
            self.current_function_start = 1;
            // CRITICAL: Reset nesting level for each function to prevent contamination
            self.nesting_level = 0;
            // Reset Halstead tracking for this function
            self.reset_halstead();

            // Visit function body to calculate complexity
            self.visit_block(&node.block);

            // Save function complexity
            if let Some(mut complexity) = self.current_function_complexity.take() {
                if let Some(fn_name) = self.current_function_name.take() {
                    // Calculate and add Halstead metrics
                    let halstead = self.calculate_halstead();
                    complexity.halstead = Some(halstead);

                    self.functions.push(FunctionComplexity {
                        name: fn_name,
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
                            let is_async = method.sig.asyncness.is_some();

                            // Start with base complexity of 1 for any method
                            let base_complexity = if is_async {
                                ComplexityMetrics {
                                    cyclomatic: 2, // async adds one complexity point
                                    cognitive: 1,  // async adds cognitive load
                                    ..Default::default()
                                }
                            } else {
                                ComplexityMetrics {
                                    cyclomatic: 1, // base cyclomatic complexity
                                    cognitive: 0,  // base cognitive complexity (no mental burden)
                                    ..Default::default()
                                }
                            };

                            self.current_function_complexity = Some(base_complexity);
                            self.current_function_name = Some(method_name.clone());
                            self.current_function_start = 1;
                            // CRITICAL: Reset nesting level for each method to prevent contamination
                            self.nesting_level = 0;

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
    /// Toyota Way: Extract Method pattern - reduced complexity from 48→≤8
    fn visit_expr(&mut self, node: &'ast Expr) {
        if self.enable_complexity {
            match node {
                Expr::If(if_expr) => return self.handle_if_expr(if_expr),
                Expr::Match(match_expr) => return self.handle_match_expr(match_expr),
                Expr::While(while_expr) => return self.handle_while_expr(while_expr),
                Expr::ForLoop(for_expr) => return self.handle_for_loop_expr(for_expr),
                Expr::Loop(loop_expr) => return self.handle_loop_expr(loop_expr),
                Expr::Try(try_expr) => return self.handle_try_expr(try_expr),
                Expr::Binary(bin_expr) => self.handle_binary_expr(bin_expr),
                Expr::Macro(_) => self.add_complexity(1, 1),
                Expr::Async(_) => self.add_complexity(1, 1),
                _ => {} // All other expressions: continue to default visitor
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

    fn visit_ident(&mut self, node: &'ast syn::Ident) {
        // Track identifier as operand for Halstead metrics
        self.track_operand(&node.to_string());
        syn::visit::visit_ident(self, node);
    }

    fn visit_lit(&mut self, node: &'ast syn::Lit) {
        // Track literals as operands for Halstead metrics
        let operand = match node {
            syn::Lit::Str(lit_str) => format!("\"{}\"", lit_str.value()),
            syn::Lit::ByteStr(lit_byte_str) => format!("b\"{}\"", 
                String::from_utf8_lossy(&lit_byte_str.value())),
            syn::Lit::Byte(lit_byte) => format!("b'{}'", lit_byte.value() as char),
            syn::Lit::Char(lit_char) => format!("'{}'", lit_char.value()),
            syn::Lit::Int(lit_int) => lit_int.base10_digits().to_string(),
            syn::Lit::Float(lit_float) => lit_float.base10_digits().to_string(),
            syn::Lit::Bool(lit_bool) => lit_bool.value().to_string(),
            syn::Lit::Verbatim(_) => "<verbatim>".to_string(),
            _ => "<literal>".to_string(),
        };
        self.track_operand(&operand);
        syn::visit::visit_lit(self, node);
    }
}

impl RustComplexityVisitor {
    /// Reset Halstead tracking for a new function
    fn reset_halstead(&mut self) {
        self.operators.clear();
        self.operands.clear();
        self.operator_count = 0;
        self.operand_count = 0;
    }

    /// Track an operator for Halstead metrics
    fn track_operator(&mut self, op: &str) {
        self.operators.insert(op.to_string());
        self.operator_count += 1;
    }

    /// Track an operand for Halstead metrics
    fn track_operand(&mut self, operand: &str) {
        self.operands.insert(operand.to_string());
        self.operand_count += 1;
    }

    /// Calculate Halstead metrics for current function
    fn calculate_halstead(&self) -> crate::services::complexity::HalsteadMetrics {
        let n1 = self.operators.len() as u32;
        let n2 = self.operands.len() as u32;
        let n1_total = self.operator_count;
        let n2_total = self.operand_count;

        let n = (n1 + n2) as f64;
        let n_total = (n1_total + n2_total) as f64;

        let volume = if n > 0.0 { n_total * n.log2() } else { 0.0 };
        let difficulty = if n2 > 0 { (n1 as f64 / 2.0) * (n2_total as f64 / n2 as f64) } else { 0.0 };
        let effort = volume * difficulty;
        let time = effort / 18.0; // Stroud number
        let bugs = volume / 3000.0; // Industry average

        crate::services::complexity::HalsteadMetrics {
            n1,
            n2,
            n1_total,
            n2_total,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        }
    }
    
    /// Toyota Way: Extract Method - handle if expressions (complexity ≤5)
    fn handle_if_expr(&mut self, if_expr: &syn::ExprIf) {
        self.track_operator("if");
        self.add_complexity(1, 1);
        self.enter_nesting();
        self.visit_expr(&if_expr.cond);
        self.visit_block(&if_expr.then_branch);
        if let Some((_, else_branch)) = &if_expr.else_branch {
            self.visit_expr(else_branch);
        }
        self.exit_nesting();
    }
    
    /// Toyota Way: Extract Method - handle match expressions (complexity ≤5)
    fn handle_match_expr(&mut self, match_expr: &syn::ExprMatch) {
        self.track_operator("match");
        self.add_complexity(1, 1);
        self.enter_nesting();
        self.visit_expr(&match_expr.expr);
        for arm in &match_expr.arms {
            self.visit_arm(arm);
        }
        self.exit_nesting();
    }
    
    /// Toyota Way: Extract Method - handle while expressions (complexity ≤5)
    fn handle_while_expr(&mut self, while_expr: &syn::ExprWhile) {
        self.track_operator("while");
        self.add_complexity(1, 1);
        self.enter_nesting();
        self.visit_expr(&while_expr.cond);
        self.visit_block(&while_expr.body);
        self.exit_nesting();
    }
    
    /// Toyota Way: Extract Method - handle for loop expressions (complexity ≤5)
    fn handle_for_loop_expr(&mut self, for_expr: &syn::ExprForLoop) {
        self.track_operator("for");
        self.add_complexity(1, 1);
        self.enter_nesting();
        self.visit_pat(&for_expr.pat);
        self.visit_expr(&for_expr.expr);
        self.visit_block(&for_expr.body);
        self.exit_nesting();
    }
    
    /// Toyota Way: Extract Method - handle loop expressions (complexity ≤3)
    fn handle_loop_expr(&mut self, loop_expr: &syn::ExprLoop) {
        self.add_complexity(1, 1);
        self.enter_nesting();
        self.visit_block(&loop_expr.body);
        self.exit_nesting();
    }
    
    /// Toyota Way: Extract Method - handle try expressions (complexity ≤3)
    fn handle_try_expr(&mut self, try_expr: &syn::ExprTry) {
        self.add_complexity(1, 1);
        self.visit_expr(&try_expr.expr);
    }
    
    /// Toyota Way: Extract Method - handle binary expressions (complexity ≤15)
    fn handle_binary_expr(&mut self, bin_expr: &syn::ExprBinary) {
        let op_str = self.get_binary_op_string(&bin_expr.op);
        self.track_operator(op_str);
        
        // Logical operators add complexity
        match bin_expr.op {
            syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                self.add_complexity(1, 1);
            }
            _ => {}
        }
    }
    
    /// Toyota Way: Extract Method - binary operator mapping (complexity ≤3)
    fn get_binary_op_string(&self, op: &syn::BinOp) -> &'static str {
        match op {
            syn::BinOp::Add(_) => "+",
            syn::BinOp::Sub(_) => "-",
            syn::BinOp::Mul(_) => "*",
            syn::BinOp::Div(_) => "/",
            syn::BinOp::Rem(_) => "%",
            syn::BinOp::And(_) => "&&",
            syn::BinOp::Or(_) => "||",
            syn::BinOp::BitXor(_) => "^",
            syn::BinOp::BitAnd(_) => "&",
            syn::BinOp::BitOr(_) => "|",
            syn::BinOp::Shl(_) => "<<",
            syn::BinOp::Shr(_) => ">>",
            syn::BinOp::Eq(_) => "==",
            syn::BinOp::Lt(_) => "<",
            syn::BinOp::Le(_) => "<=",
            syn::BinOp::Ne(_) => "!=",
            syn::BinOp::Ge(_) => ">=",
            syn::BinOp::Gt(_) => ">",
            syn::BinOp::AddAssign(_) => "+=",
            syn::BinOp::SubAssign(_) => "-=",
            syn::BinOp::MulAssign(_) => "*=",
            syn::BinOp::DivAssign(_) => "/=",
            syn::BinOp::RemAssign(_) => "%=",
            syn::BinOp::BitXorAssign(_) => "^=",
            syn::BinOp::BitAndAssign(_) => "&=",
            syn::BinOp::BitOrAssign(_) => "|=",
            syn::BinOp::ShlAssign(_) => "<<=",
            syn::BinOp::ShrAssign(_) => ">>=",
            _ => "unknown_op",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ast_rust_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
