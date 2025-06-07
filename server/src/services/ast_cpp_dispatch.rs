//! C++ AST parser with improved dispatch table architecture
//!
//! This module implements a more modular approach to C++ AST parsing with:
//! - Separated dispatch table management
//! - Modular node processing pipeline
//! - Reduced cognitive complexity through function decomposition

use crate::models::unified_ast::{
    AstDag, AstKind, ExprKind, FunctionKind, Language, MacroKind, NodeFlags, NodeKey, StmtKind,
    TypeKind, UnifiedAstNode, VarKind,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "cpp-ast")]
use tree_sitter_cpp;

#[cfg(feature = "cpp-ast")]
use tree_sitter::{Node, Parser, TreeCursor};

/// Node mapper function type for dispatch table
#[cfg(feature = "cpp-ast")]
type NodeMapper = fn(&str) -> Option<AstKind>;

/// Node info extractor function type
#[cfg(feature = "cpp-ast")]
type InfoExtractor = fn(&Node, &str, &mut UnifiedAstNode);

/// Dispatch table for node type mapping
#[cfg(feature = "cpp-ast")]
static NODE_DISPATCH: Lazy<HashMap<&'static str, NodeMapper>> = Lazy::new(|| {
    NodeDispatchBuilder::new()
        .add_functions()
        .add_variables()
        .add_types()
        .add_statements()
        .add_expressions()
        .add_literals()
        .add_preprocessor()
        .add_cpp_specific()
        .build()
});

/// Information extraction dispatch table
#[cfg(feature = "cpp-ast")]
static INFO_DISPATCH: Lazy<HashMap<&'static str, InfoExtractor>> = Lazy::new(|| {
    InfoDispatchBuilder::new()
        .add_function_extractors()
        .add_variable_extractors()
        .add_class_extractors()
        .build()
});

/// Builder for creating the node dispatch table
#[cfg(feature = "cpp-ast")]
struct NodeDispatchBuilder {
    table: HashMap<&'static str, NodeMapper>,
}

#[cfg(feature = "cpp-ast")]
impl NodeDispatchBuilder {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn add_functions(mut self) -> Self {
        self.table.insert("function_definition", map_function);
        self.table.insert("method_declaration", map_method);
        self.table
            .insert("constructor_declaration", map_constructor);
        self.table.insert("destructor_declaration", map_destructor);
        self.table.insert("operator_overload", map_operator);
        self
    }

    fn add_variables(mut self) -> Self {
        self.table.insert("declaration", map_variable);
        self.table.insert("field_declaration", map_field);
        self
    }

    fn add_types(mut self) -> Self {
        self.table.insert("class_specifier", map_class);
        self.table.insert("struct_specifier", map_struct);
        self.table.insert("enum_specifier", map_enum);
        self.table.insert("union_specifier", map_union);
        self.table.insert("template_declaration", map_template);
        self.table.insert("namespace_definition", map_namespace);
        self.table.insert("typedef_declaration", map_typedef);
        self
    }

    fn add_statements(mut self) -> Self {
        self.table.insert("if_statement", map_if_stmt);
        self.table.insert("for_statement", map_for_stmt);
        self.table.insert("while_statement", map_while_stmt);
        self.table.insert("do_statement", map_do_stmt);
        self.table.insert("switch_statement", map_switch_stmt);
        self.table.insert("case_statement", map_case_stmt);
        self.table.insert("return_statement", map_return_stmt);
        self.table.insert("break_statement", map_break_stmt);
        self.table.insert("continue_statement", map_continue_stmt);
        self.table.insert("try_statement", map_try_stmt);
        self.table.insert("throw_statement", map_throw_stmt);
        self
    }

    fn add_expressions(mut self) -> Self {
        self.table.insert("binary_expression", map_binary_expr);
        self.table.insert("unary_expression", map_unary_expr);
        self.table
            .insert("assignment_expression", map_assignment_expr);
        self.table.insert("call_expression", map_call_expr);
        self.table.insert("field_expression", map_field_expr);
        self.table.insert("lambda_expression", map_lambda_expr);
        self.table.insert("new_expression", map_new_expr);
        self.table.insert("delete_expression", map_delete_expr);
        self.table.insert("cast_expression", map_cast_expr);
        self.table.insert("sizeof_expression", map_sizeof_expr);
        self
    }

    fn add_literals(mut self) -> Self {
        self.table.insert("number_literal", map_number_literal);
        self.table.insert("string_literal", map_string_literal);
        self.table.insert("char_literal", map_char_literal);
        self.table.insert("true", map_bool_literal);
        self.table.insert("false", map_bool_literal);
        self.table.insert("nullptr", map_null_literal);
        self
    }

    fn add_preprocessor(mut self) -> Self {
        self.table.insert("preproc_include", map_include);
        self.table.insert("preproc_def", map_define);
        self.table
            .insert("preproc_function_def", map_function_define);
        self.table.insert("preproc_ifdef", map_ifdef);
        self.table.insert("preproc_ifndef", map_ifdef);
        self.table.insert("preproc_if", map_if_directive);
        self
    }

    fn add_cpp_specific(mut self) -> Self {
        self.table.insert("using_declaration", map_using);
        self.table.insert("for_range_loop", map_for_each);
        self.table.insert("catch_clause", map_catch);
        self.table.insert("goto_statement", map_goto);
        self.table.insert("labeled_statement", map_label);
        self.table.insert("compound_statement", map_block);
        self.table.insert("identifier", map_identifier);
        self.table.insert("system_lib_string", map_string_literal);
        self
    }

    fn build(self) -> HashMap<&'static str, NodeMapper> {
        self.table
    }
}

/// Builder for creating the info extraction dispatch table
#[cfg(feature = "cpp-ast")]
struct InfoDispatchBuilder {
    table: HashMap<&'static str, InfoExtractor>,
}

#[cfg(feature = "cpp-ast")]
impl InfoDispatchBuilder {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn add_function_extractors(mut self) -> Self {
        self.table
            .insert("function_definition", extract_function_info);
        self.table
            .insert("method_declaration", extract_function_info);
        self.table
            .insert("constructor_declaration", extract_function_info);
        self.table
            .insert("destructor_declaration", extract_function_info);
        self.table
            .insert("operator_overload", extract_function_info);
        self
    }

    fn add_variable_extractors(mut self) -> Self {
        self.table.insert("declaration", extract_variable_info);
        self.table
            .insert("field_declaration", extract_variable_info);
        self
    }

    fn add_class_extractors(mut self) -> Self {
        self.table.insert("class_specifier", extract_class_info);
        self.table.insert("struct_specifier", extract_class_info);
        self
    }

    fn build(self) -> HashMap<&'static str, InfoExtractor> {
        self.table
    }
}

// Node mapping functions (simplified for reduced complexity)
#[cfg(feature = "cpp-ast")]
fn map_function(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Regular))
}

#[cfg(feature = "cpp-ast")]
fn map_method(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Method))
}

#[cfg(feature = "cpp-ast")]
fn map_constructor(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Constructor))
}

#[cfg(feature = "cpp-ast")]
fn map_destructor(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Destructor))
}

#[cfg(feature = "cpp-ast")]
fn map_operator(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Operator))
}

#[cfg(feature = "cpp-ast")]
fn map_variable(_: &str) -> Option<AstKind> {
    Some(AstKind::Variable(VarKind::Let))
}

#[cfg(feature = "cpp-ast")]
fn map_field(_: &str) -> Option<AstKind> {
    Some(AstKind::Variable(VarKind::Field))
}

#[cfg(feature = "cpp-ast")]
fn map_class(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Class))
}

#[cfg(feature = "cpp-ast")]
fn map_struct(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Struct))
}

#[cfg(feature = "cpp-ast")]
fn map_enum(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Enum))
}

#[cfg(feature = "cpp-ast")]
fn map_union(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Union))
}

#[cfg(feature = "cpp-ast")]
fn map_template(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Template))
}

#[cfg(feature = "cpp-ast")]
fn map_namespace(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Namespace))
}

#[cfg(feature = "cpp-ast")]
fn map_typedef(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Typedef))
}

#[cfg(feature = "cpp-ast")]
fn map_if_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::If))
}

#[cfg(feature = "cpp-ast")]
fn map_for_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::For))
}

#[cfg(feature = "cpp-ast")]
fn map_while_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::While))
}

#[cfg(feature = "cpp-ast")]
fn map_do_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::DoWhile))
}

#[cfg(feature = "cpp-ast")]
fn map_switch_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Switch))
}

#[cfg(feature = "cpp-ast")]
fn map_case_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Case))
}

#[cfg(feature = "cpp-ast")]
fn map_return_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Return))
}

#[cfg(feature = "cpp-ast")]
fn map_break_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Break))
}

#[cfg(feature = "cpp-ast")]
fn map_continue_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Continue))
}

#[cfg(feature = "cpp-ast")]
fn map_try_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Try))
}

#[cfg(feature = "cpp-ast")]
fn map_throw_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Throw))
}

#[cfg(feature = "cpp-ast")]
fn map_binary_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Binary))
}

#[cfg(feature = "cpp-ast")]
fn map_unary_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Unary))
}

#[cfg(feature = "cpp-ast")]
fn map_assignment_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Binary))
}

#[cfg(feature = "cpp-ast")]
fn map_call_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Call))
}

#[cfg(feature = "cpp-ast")]
fn map_field_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Member))
}

#[cfg(feature = "cpp-ast")]
fn map_lambda_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Lambda))
}

#[cfg(feature = "cpp-ast")]
fn map_new_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::New))
}

#[cfg(feature = "cpp-ast")]
fn map_delete_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Delete))
}

#[cfg(feature = "cpp-ast")]
fn map_cast_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Unary))
}

#[cfg(feature = "cpp-ast")]
fn map_sizeof_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Unary))
}

#[cfg(feature = "cpp-ast")]
fn map_number_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "cpp-ast")]
fn map_string_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "cpp-ast")]
fn map_char_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "cpp-ast")]
fn map_bool_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "cpp-ast")]
fn map_null_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "cpp-ast")]
fn map_include(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::Include))
}

#[cfg(feature = "cpp-ast")]
fn map_define(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::ObjectLike))
}

#[cfg(feature = "cpp-ast")]
fn map_ifdef(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::Conditional))
}

#[cfg(feature = "cpp-ast")]
fn map_if_directive(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::Conditional))
}

#[cfg(feature = "cpp-ast")]
fn map_function_define(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::FunctionLike))
}

#[cfg(feature = "cpp-ast")]
fn map_using(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Alias))
}

#[cfg(feature = "cpp-ast")]
fn map_for_each(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::ForEach))
}

#[cfg(feature = "cpp-ast")]
fn map_catch(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Catch))
}

#[cfg(feature = "cpp-ast")]
fn map_goto(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Goto))
}

#[cfg(feature = "cpp-ast")]
fn map_label(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Label))
}

#[cfg(feature = "cpp-ast")]
fn map_block(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Block))
}

#[cfg(feature = "cpp-ast")]
fn map_identifier(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Identifier))
}

// Information extraction functions (modularized)
#[cfg(feature = "cpp-ast")]
fn extract_function_info(node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
    if let Some(name) = extract_name_from_node(node, source) {
        ast_node.name_vector = hash_name(&name);
    }

    extract_function_flags(node, source, ast_node);
}

#[cfg(feature = "cpp-ast")]
fn extract_variable_info(node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
    if let Some(name) = extract_name_from_node(node, source) {
        ast_node.name_vector = hash_name(&name);
    }

    extract_variable_flags(node, source, ast_node);
}

#[cfg(feature = "cpp-ast")]
fn extract_class_info(node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
    if let Some(name) = extract_name_from_node(node, source) {
        ast_node.name_vector = hash_name(&name);
    }

    extract_class_flags(node, source, ast_node);
}

/// Enhanced C++ AST parser with modular dispatch table architecture
pub struct CppAstDispatchParser {
    #[cfg(feature = "cpp-ast")]
    parser: Parser,
    #[cfg(feature = "cpp-ast")]
    complexity_calculator: ComplexityCalculator,
    #[cfg(feature = "cpp-ast")]
    #[allow(dead_code)]
    name_extractor: NameExtractor,
}

#[cfg(feature = "cpp-ast")]
struct ComplexityCalculator;

#[cfg(feature = "cpp-ast")]
impl ComplexityCalculator {
    fn new() -> Self {
        Self
    }

    fn calculate(&self, node: &Node) -> u32 {
        let mut complexity = 1; // Base complexity
        self.count_complexity_nodes(&mut node.walk(), &mut complexity);
        complexity
    }

    fn count_complexity_nodes(&self, cursor: &mut TreeCursor, complexity: &mut u32) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            let node = cursor.node();
            match node.kind() {
                "if_statement" | "while_statement" | "for_statement" | "do_statement"
                | "for_range_loop" => *complexity += 1,
                "switch_statement" => {
                    *complexity += 1;
                    self.count_case_statements(&mut node.walk(), complexity);
                }
                "try_statement" => {
                    *complexity += 1;
                    self.count_catch_clauses(&mut node.walk(), complexity);
                }
                "goto_statement" => *complexity += 3, // High penalty for goto
                "conditional_expression" => *complexity += 1, // Ternary operator
                "logical_expression" => {
                    if let Ok(op) = node.utf8_text(&[]) {
                        if op.contains("&&") || op.contains("||") {
                            *complexity += 1;
                        }
                    }
                }
                _ => {}
            }

            self.count_complexity_nodes(cursor, complexity);

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }

    #[allow(clippy::only_used_in_recursion)]
    fn count_case_statements(&self, cursor: &mut TreeCursor, complexity: &mut u32) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            if cursor.node().kind() == "case_statement" {
                *complexity += 1;
            }
            self.count_case_statements(cursor, complexity);
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }

    #[allow(clippy::only_used_in_recursion)]
    fn count_catch_clauses(&self, cursor: &mut TreeCursor, complexity: &mut u32) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            if cursor.node().kind() == "catch_clause" {
                *complexity += 1;
            }
            self.count_catch_clauses(cursor, complexity);
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }
}

#[cfg(feature = "cpp-ast")]
struct NameExtractor;

#[cfg(feature = "cpp-ast")]
impl NameExtractor {
    fn new() -> Self {
        Self
    }

    fn extract(&self, node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();

        // Try to find identifier nodes
        if self.find_child_by_kind(&mut cursor, "identifier") {
            let identifier_node = cursor.node();
            return identifier_node
                .utf8_text(source.as_bytes())
                .ok()
                .map(|s| s.to_string());
        }

        // For operator overloads
        if node.kind() == "operator_overload" && self.find_child_by_kind(&mut cursor, "operator") {
            let op_node = cursor.node();
            return op_node
                .utf8_text(source.as_bytes())
                .ok()
                .map(|s| format!("operator{s}"));
        }

        // For includes
        if node.kind() == "preproc_include"
            && (self.find_child_by_kind(&mut cursor, "string_literal")
                || self.find_child_by_kind(&mut cursor, "system_lib_string"))
        {
            let text = cursor.node().utf8_text(source.as_bytes()).ok()?;
            return Some(
                text.trim_matches('"')
                    .trim_matches('<')
                    .trim_matches('>')
                    .to_string(),
            );
        }

        None
    }

    #[allow(clippy::only_used_in_recursion)]
    fn find_child_by_kind(&self, cursor: &mut TreeCursor, kind: &str) -> bool {
        if !cursor.goto_first_child() {
            return false;
        }

        loop {
            if cursor.node().kind() == kind {
                return true;
            }

            if self.find_child_by_kind(cursor, kind) {
                return true;
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
        false
    }
}

impl Default for CppAstDispatchParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CppAstDispatchParser {
    pub fn new() -> Self {
        #[cfg(feature = "cpp-ast")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(&tree_sitter_cpp::language())
                .expect("Error loading C++ grammar");
            Self {
                parser,
                complexity_calculator: ComplexityCalculator::new(),
                name_extractor: NameExtractor::new(),
            }
        }

        #[cfg(not(feature = "cpp-ast"))]
        {
            Self {}
        }
    }

    pub fn parse_file(&mut self, path: &Path, content: &str) -> Result<AstDag> {
        #[cfg(feature = "cpp-ast")]
        {
            let tree = self
                .parser
                .parse(content, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse C++ file: {}", path.display()))?;

            let mut dag = AstDag::new();
            let mut cursor = tree.walk();

            // Create root module node
            let mut root_node = UnifiedAstNode::new(
                AstKind::Module(crate::models::unified_ast::ModuleKind::File),
                Language::Cpp,
            );
            root_node.source_range = 0..content.len() as u32;
            let root_key = dag.add_node(root_node);

            self.walk_tree(&mut cursor, content, &mut dag, root_key)?;

            Ok(dag)
        }

        #[cfg(not(feature = "cpp-ast"))]
        {
            let _ = (path, content);
            Err(anyhow::anyhow!(
                "C++ AST parsing requires the 'cpp-ast' feature"
            ))
        }
    }

    #[cfg(feature = "cpp-ast")]
    fn walk_tree(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        dag: &mut AstDag,
        parent_key: NodeKey,
    ) -> Result<()> {
        if !cursor.goto_first_child() {
            return Ok(());
        }

        loop {
            let node = cursor.node();
            if let Some(ast_node) = self.convert_node(&node, source)? {
                let node_key = dag.add_node(ast_node);

                // Link to parent using the existing approach
                self.link_to_parent(dag, parent_key, node_key);

                self.walk_tree(cursor, source, dag, node_key)?;
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
        Ok(())
    }

    #[cfg(feature = "cpp-ast")]
    fn convert_node(&self, node: &Node, source: &str) -> Result<Option<UnifiedAstNode>> {
        let kind = self.map_node_kind(node.kind());

        if let Some(ast_kind) = kind {
            let start_pos = node.start_position();
            let end_pos = node.end_position();

            let mut ast_node = UnifiedAstNode::new(ast_kind, Language::Cpp);
            ast_node.source_range = start_pos.column as u32..end_pos.column as u32;

            // Extract information using dispatch table
            self.extract_node_info(node, source, &mut ast_node);

            // Calculate complexity for functions
            if matches!(ast_node.kind, AstKind::Function(_)) {
                let complexity = self.complexity_calculator.calculate(node);
                ast_node.set_complexity(complexity);
            }

            Ok(Some(ast_node))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "cpp-ast")]
    fn map_node_kind(&self, node_kind: &str) -> Option<AstKind> {
        NODE_DISPATCH
            .get(node_kind)
            .and_then(|mapper| mapper(node_kind))
    }

    #[cfg(feature = "cpp-ast")]
    fn extract_node_info(&self, node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
        // Use dispatch table for information extraction
        if let Some(extractor) = INFO_DISPATCH.get(node.kind()) {
            extractor(node, source, ast_node);
        }
    }

    #[cfg(feature = "cpp-ast")]
    fn link_to_parent(&self, dag: &mut AstDag, parent_key: NodeKey, child_key: NodeKey) {
        if let Some(parent) = dag.nodes.get_mut(parent_key) {
            if parent.first_child == 0 {
                parent.first_child = child_key;
            } else {
                // Find last sibling and link
                let mut sibling = parent.first_child;
                while let Some(sibling_node) = dag.nodes.get(sibling) {
                    if sibling_node.next_sibling == 0 {
                        break;
                    }
                    sibling = sibling_node.next_sibling;
                }
                if let Some(sibling_node) = dag.nodes.get_mut(sibling) {
                    sibling_node.next_sibling = child_key;
                }
            }
        }

        if let Some(child_node) = dag.nodes.get_mut(child_key) {
            child_node.parent = parent_key;
        }
    }
}

// Helper functions (extracted for reusability)
#[cfg(feature = "cpp-ast")]
fn extract_name_from_node(node: &Node, source: &str) -> Option<String> {
    let name_extractor = NameExtractor::new();
    name_extractor.extract(node, source)
}

#[cfg(feature = "cpp-ast")]
fn extract_function_flags(node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                match text {
                    "static" => ast_node.flags.set(NodeFlags::STATIC),
                    "inline" => ast_node.flags.set(NodeFlags::INLINE),
                    "extern" => ast_node.flags.set(NodeFlags::EXTERN),
                    "virtual" => ast_node.flags.set(NodeFlags::VIRTUAL),
                    "override" => ast_node.flags.set(NodeFlags::OVERRIDE),
                    "final" => ast_node.flags.set(NodeFlags::FINAL),
                    "const" => ast_node.flags.set(NodeFlags::CONST),
                    "constexpr" => ast_node.flags.set(NodeFlags::CONSTEXPR),
                    "noexcept" => ast_node.flags.set(NodeFlags::NOEXCEPT),
                    _ => {}
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[cfg(feature = "cpp-ast")]
fn extract_variable_flags(node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                match text {
                    "const" => ast_node.flags.set(NodeFlags::CONST),
                    "volatile" => ast_node.flags.set(NodeFlags::VOLATILE),
                    "static" => ast_node.flags.set(NodeFlags::STATIC),
                    "extern" => ast_node.flags.set(NodeFlags::EXTERN),
                    "mutable" => ast_node.flags.set(NodeFlags::MUTABLE),
                    "constexpr" => ast_node.flags.set(NodeFlags::CONSTEXPR),
                    _ => {}
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[cfg(feature = "cpp-ast")]
fn extract_class_flags(node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if let Ok("final") = child.utf8_text(source.as_bytes()) {
                ast_node.flags.set(NodeFlags::FINAL);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[cfg(feature = "cpp-ast")]
fn hash_name(name: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "cpp-ast")]
    fn test_dispatch_parser_simple_class() {
        let mut parser = CppAstDispatchParser::new();
        let content = r#"
class TestClass {
public:
    TestClass() {}
    void method() const {}
private:
    int field;
};
"#;
        let result = parser.parse_file(Path::new("test.cpp"), content);
        assert!(result.is_ok());

        let dag = result.unwrap();
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    #[cfg(feature = "cpp-ast")]
    fn test_dispatch_builder() {
        let builder = NodeDispatchBuilder::new();
        let dispatch = builder.add_functions().add_variables().add_types().build();

        assert!(dispatch.contains_key("function_definition"));
        assert!(dispatch.contains_key("class_specifier"));
        assert!(dispatch.contains_key("declaration"));
    }

    #[test]
    #[cfg(not(feature = "cpp-ast"))]
    fn test_cpp_dispatch_disabled() {
        let mut parser = CppAstDispatchParser::new();
        let content = "class A {};";
        let result = parser.parse_file(Path::new("test.cpp"), content);
        assert!(result.is_err());
    }
}
