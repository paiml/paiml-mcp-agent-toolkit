//! TypeScript/JavaScript AST parser with improved dispatch table architecture
//!
//! This module implements a more modular approach to TypeScript/JavaScript AST parsing with:
//! - Separated dispatch table management
//! - Modular node processing pipeline  
//! - Support for TypeScript-specific constructs (interfaces, types, generics)
//! - Reduced cognitive complexity through function decomposition

#![allow(dead_code)]

use crate::models::unified_ast::{
    AstDag, AstKind, ExprKind, FunctionKind, Language, MacroKind, NodeFlags, NodeKey, StmtKind,
    TypeKind, UnifiedAstNode, VarKind,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "typescript-ast")]
use swc_common::{sync::Lrc, FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::*;
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

/// Node mapper function type for dispatch table
#[cfg(feature = "typescript-ast")]
type NodeMapper = fn(&str) -> Option<AstKind>;

/// Node info extractor function type
#[cfg(feature = "typescript-ast")]
type InfoExtractor = fn(&dyn std::any::Any, &str, &mut UnifiedAstNode);

/// Dispatch table for node type mapping
#[cfg(feature = "typescript-ast")]
static NODE_DISPATCH: Lazy<HashMap<&'static str, NodeMapper>> = Lazy::new(|| {
    TsNodeDispatchBuilder::new()
        .add_functions()
        .add_variables()
        .add_types()
        .add_statements()
        .add_expressions()
        .add_literals()
        .add_typescript_specific()
        .build()
});

/// Information extraction dispatch table
#[cfg(feature = "typescript-ast")]
static INFO_DISPATCH: Lazy<HashMap<&'static str, InfoExtractor>> = Lazy::new(|| {
    TsInfoDispatchBuilder::new()
        .add_function_extractors()
        .add_variable_extractors()
        .add_type_extractors()
        .build()
});

/// Builder for creating the TypeScript node dispatch table
#[cfg(feature = "typescript-ast")]
struct TsNodeDispatchBuilder {
    table: HashMap<&'static str, NodeMapper>,
}

#[cfg(feature = "typescript-ast")]
impl TsNodeDispatchBuilder {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn add_functions(mut self) -> Self {
        self.table.insert("function_declaration", map_function);
        self.table.insert("method_definition", map_method);
        self.table.insert("constructor", map_constructor);
        self.table.insert("arrow_function", map_arrow_function);
        self.table.insert("function_expression", map_function_expr);
        self.table.insert("getter", map_getter);
        self.table.insert("setter", map_setter);
        self
    }

    fn add_variables(mut self) -> Self {
        self.table.insert("variable_declaration", map_variable);
        self.table.insert("parameter", map_parameter);
        self.table.insert("property", map_property);
        self.table.insert("field", map_field);
        self
    }

    fn add_types(mut self) -> Self {
        self.table.insert("class_declaration", map_class);
        self.table.insert("interface_declaration", map_interface);
        self.table.insert("type_alias", map_type_alias);
        self.table.insert("enum_declaration", map_enum);
        self.table.insert("namespace_declaration", map_namespace);
        self.table.insert("module_declaration", map_module);
        self
    }

    fn add_statements(mut self) -> Self {
        self.table.insert("if_statement", map_if_stmt);
        self.table.insert("while_statement", map_while_stmt);
        self.table.insert("do_while_statement", map_do_stmt);
        self.table.insert("for_statement", map_for_stmt);
        self.table.insert("for_in_statement", map_for_in_stmt);
        self.table.insert("for_of_statement", map_for_of_stmt);
        self.table.insert("switch_statement", map_switch_stmt);
        self.table.insert("case_clause", map_case_stmt);
        self.table.insert("return_statement", map_return_stmt);
        self.table.insert("break_statement", map_break_stmt);
        self.table.insert("continue_statement", map_continue_stmt);
        self.table.insert("try_statement", map_try_stmt);
        self.table.insert("throw_statement", map_throw_stmt);
        self.table.insert("block_statement", map_block_stmt);
        self
    }

    fn add_expressions(mut self) -> Self {
        self.table.insert("binary_expression", map_binary_expr);
        self.table.insert("unary_expression", map_unary_expr);
        self.table
            .insert("assignment_expression", map_assignment_expr);
        self.table.insert("call_expression", map_call_expr);
        self.table.insert("member_expression", map_member_expr);
        self.table
            .insert("conditional_expression", map_conditional_expr);
        self.table.insert("new_expression", map_new_expr);
        self.table.insert("this_expression", map_this_expr);
        self.table.insert("identifier", map_identifier);
        self.table.insert("array_expression", map_array_expr);
        self.table.insert("object_expression", map_object_expr);
        self
    }

    fn add_literals(mut self) -> Self {
        self.table.insert("number_literal", map_number_literal);
        self.table.insert("string_literal", map_string_literal);
        self.table.insert("boolean_literal", map_bool_literal);
        self.table.insert("null_literal", map_null_literal);
        self.table
            .insert("undefined_literal", map_undefined_literal);
        self.table.insert("template_literal", map_template_literal);
        self.table.insert("regex_literal", map_regex_literal);
        self
    }

    fn add_typescript_specific(mut self) -> Self {
        self.table.insert("type_annotation", map_type_annotation);
        self.table.insert("generic_type", map_generic_type);
        self.table.insert("union_type", map_union_type);
        self.table
            .insert("intersection_type", map_intersection_type);
        self.table.insert("mapped_type", map_mapped_type);
        self.table.insert("conditional_type", map_conditional_type);
        self.table.insert("import_statement", map_import);
        self.table.insert("export_statement", map_export);
        self.table.insert("decorator", map_decorator);
        self
    }

    fn build(self) -> HashMap<&'static str, NodeMapper> {
        self.table
    }
}

/// Builder for creating the TypeScript info extraction dispatch table
#[cfg(feature = "typescript-ast")]
struct TsInfoDispatchBuilder {
    table: HashMap<&'static str, InfoExtractor>,
}

#[cfg(feature = "typescript-ast")]
impl TsInfoDispatchBuilder {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn add_function_extractors(mut self) -> Self {
        self.table
            .insert("function_declaration", extract_function_info);
        self.table
            .insert("method_definition", extract_function_info);
        self.table.insert("arrow_function", extract_function_info);
        self
    }

    fn add_variable_extractors(mut self) -> Self {
        self.table
            .insert("variable_declaration", extract_variable_info);
        self.table.insert("parameter", extract_variable_info);
        self.table.insert("property", extract_variable_info);
        self
    }

    fn add_type_extractors(mut self) -> Self {
        self.table.insert("class_declaration", extract_type_info);
        self.table
            .insert("interface_declaration", extract_type_info);
        self.table.insert("type_alias", extract_type_info);
        self.table.insert("enum_declaration", extract_type_info);
        self
    }

    fn build(self) -> HashMap<&'static str, InfoExtractor> {
        self.table
    }
}

// Node mapping functions (simplified for reduced complexity)
#[cfg(feature = "typescript-ast")]
fn map_function(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Regular))
}

#[cfg(feature = "typescript-ast")]
fn map_method(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Method))
}

#[cfg(feature = "typescript-ast")]
fn map_constructor(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Constructor))
}

#[cfg(feature = "typescript-ast")]
fn map_arrow_function(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Regular))
}

#[cfg(feature = "typescript-ast")]
fn map_function_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Regular))
}

#[cfg(feature = "typescript-ast")]
fn map_getter(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Getter))
}

#[cfg(feature = "typescript-ast")]
fn map_setter(_: &str) -> Option<AstKind> {
    Some(AstKind::Function(FunctionKind::Setter))
}

#[cfg(feature = "typescript-ast")]
fn map_variable(_: &str) -> Option<AstKind> {
    Some(AstKind::Variable(VarKind::Let))
}

#[cfg(feature = "typescript-ast")]
fn map_parameter(_: &str) -> Option<AstKind> {
    Some(AstKind::Variable(VarKind::Parameter))
}

#[cfg(feature = "typescript-ast")]
fn map_property(_: &str) -> Option<AstKind> {
    Some(AstKind::Variable(VarKind::Field))
}

#[cfg(feature = "typescript-ast")]
fn map_field(_: &str) -> Option<AstKind> {
    Some(AstKind::Variable(VarKind::Field))
}

#[cfg(feature = "typescript-ast")]
fn map_class(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Class))
}

#[cfg(feature = "typescript-ast")]
fn map_interface(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Interface))
}

#[cfg(feature = "typescript-ast")]
fn map_type_alias(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Alias))
}

#[cfg(feature = "typescript-ast")]
fn map_enum(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Enum))
}

#[cfg(feature = "typescript-ast")]
fn map_namespace(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Namespace))
}

#[cfg(feature = "typescript-ast")]
fn map_module(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Module))
}

#[cfg(feature = "typescript-ast")]
fn map_if_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::If))
}

#[cfg(feature = "typescript-ast")]
fn map_while_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::While))
}

#[cfg(feature = "typescript-ast")]
fn map_do_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::DoWhile))
}

#[cfg(feature = "typescript-ast")]
fn map_for_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::For))
}

#[cfg(feature = "typescript-ast")]
fn map_for_in_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::ForEach))
}

#[cfg(feature = "typescript-ast")]
fn map_for_of_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::ForEach))
}

#[cfg(feature = "typescript-ast")]
fn map_switch_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Switch))
}

#[cfg(feature = "typescript-ast")]
fn map_case_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Case))
}

#[cfg(feature = "typescript-ast")]
fn map_return_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Return))
}

#[cfg(feature = "typescript-ast")]
fn map_break_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Break))
}

#[cfg(feature = "typescript-ast")]
fn map_continue_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Continue))
}

#[cfg(feature = "typescript-ast")]
fn map_try_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Try))
}

#[cfg(feature = "typescript-ast")]
fn map_throw_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Throw))
}

#[cfg(feature = "typescript-ast")]
fn map_block_stmt(_: &str) -> Option<AstKind> {
    Some(AstKind::Statement(StmtKind::Block))
}

#[cfg(feature = "typescript-ast")]
fn map_binary_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Binary))
}

#[cfg(feature = "typescript-ast")]
fn map_unary_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Unary))
}

#[cfg(feature = "typescript-ast")]
fn map_assignment_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Binary))
}

#[cfg(feature = "typescript-ast")]
fn map_call_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Call))
}

#[cfg(feature = "typescript-ast")]
fn map_member_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Member))
}

#[cfg(feature = "typescript-ast")]
fn map_conditional_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Conditional))
}

#[cfg(feature = "typescript-ast")]
fn map_new_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::New))
}

#[cfg(feature = "typescript-ast")]
fn map_this_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::This))
}

#[cfg(feature = "typescript-ast")]
fn map_identifier(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Identifier))
}

#[cfg(feature = "typescript-ast")]
fn map_array_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Array))
}

#[cfg(feature = "typescript-ast")]
fn map_object_expr(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Object))
}

#[cfg(feature = "typescript-ast")]
fn map_number_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_string_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_bool_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_null_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_undefined_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_template_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_regex_literal(_: &str) -> Option<AstKind> {
    Some(AstKind::Expression(ExprKind::Literal))
}

#[cfg(feature = "typescript-ast")]
fn map_type_annotation(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Annotation))
}

#[cfg(feature = "typescript-ast")]
fn map_generic_type(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Generic))
}

#[cfg(feature = "typescript-ast")]
fn map_union_type(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Union))
}

#[cfg(feature = "typescript-ast")]
fn map_intersection_type(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Intersection))
}

#[cfg(feature = "typescript-ast")]
fn map_mapped_type(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Mapped))
}

#[cfg(feature = "typescript-ast")]
fn map_conditional_type(_: &str) -> Option<AstKind> {
    Some(AstKind::Type(TypeKind::Conditional))
}

#[cfg(feature = "typescript-ast")]
fn map_import(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::Include))
}

#[cfg(feature = "typescript-ast")]
fn map_export(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::Export))
}

#[cfg(feature = "typescript-ast")]
fn map_decorator(_: &str) -> Option<AstKind> {
    Some(AstKind::Macro(MacroKind::Decorator))
}

// Information extraction functions (modularized)
#[cfg(feature = "typescript-ast")]
fn extract_function_info(_node: &dyn std::any::Any, _source: &str, ast_node: &mut UnifiedAstNode) {
    // Extract function-specific information like async, export status, etc.
    ast_node.flags.set(NodeFlags::ASYNC); // Example - would check actual node
}

#[cfg(feature = "typescript-ast")]
fn extract_variable_info(_node: &dyn std::any::Any, _source: &str, ast_node: &mut UnifiedAstNode) {
    // Extract variable-specific information like const, readonly, etc.
    ast_node.flags.set(NodeFlags::CONST); // Example - would check actual node
}

#[cfg(feature = "typescript-ast")]
fn extract_type_info(_node: &dyn std::any::Any, _source: &str, _ast_node: &mut UnifiedAstNode) {
    // Extract type-specific information
    // This would analyze the actual node for type-specific flags
}

/// Enhanced TypeScript/JavaScript AST parser with modular dispatch table architecture
pub struct TsAstDispatchParser {
    #[cfg(feature = "typescript-ast")]
    complexity_calculator: TsComplexityCalculator,
    #[cfg(feature = "typescript-ast")]
    name_extractor: TsNameExtractor,
    #[cfg(feature = "typescript-ast")]
    source_map: Option<Lrc<SourceMap>>,
}

#[cfg(feature = "typescript-ast")]
struct TsComplexityCalculator;

#[cfg(feature = "typescript-ast")]
impl TsComplexityCalculator {
    fn new() -> Self {
        Self
    }

    fn calculate_from_swc_node(&self, _node: &dyn std::any::Any) -> u32 {
        // This would calculate complexity based on the SWC AST node type
        // For now, return a base complexity
        1
    }
}

#[cfg(feature = "typescript-ast")]
struct TsNameExtractor;

#[cfg(feature = "typescript-ast")]
impl TsNameExtractor {
    fn new() -> Self {
        Self
    }

    fn extract_from_swc_node(&self, _node: &dyn std::any::Any) -> Option<String> {
        // This would extract names from SWC AST nodes
        // For now, return a placeholder
        Some("placeholder_name".to_string())
    }
}

impl Default for TsAstDispatchParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TsAstDispatchParser {
    pub fn new() -> Self {
        #[cfg(feature = "typescript-ast")]
        {
            Self {
                complexity_calculator: TsComplexityCalculator::new(),
                name_extractor: TsNameExtractor::new(),
                source_map: None,
            }
        }

        #[cfg(not(feature = "typescript-ast"))]
        {
            Self {}
        }
    }

#[inline]
    pub fn parse_file(&mut self, path: &Path, content: &str) -> Result<AstDag> {
        #[cfg(feature = "typescript-ast")]
        {
            let is_typescript = path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| matches!(ext, "ts" | "tsx"));

            let (module, source_map) = if is_typescript {
                self.parse_typescript_content(path, content)?
            } else {
                self.parse_javascript_content(path, content)?
            };

            self.source_map = Some(source_map);

            let mut dag = AstDag::new();

            // Create root module node
            let language = if is_typescript {
                Language::TypeScript
            } else {
                Language::JavaScript
            };

            let mut root_node = UnifiedAstNode::new(
                AstKind::Module(crate::models::unified_ast::ModuleKind::File),
                language,
            );
            root_node.source_range = 0..content.len() as u32;
            let root_key = dag.add_node(root_node);

            self.convert_swc_module(&module, &mut dag, root_key)?;

            Ok(dag)
        }

        #[cfg(not(feature = "typescript-ast"))]
        {
            let _ = (path, content);
            Err(anyhow::anyhow!(
                "TypeScript AST parsing requires the 'typescript-ast' feature"
            ))
        }
    }

    #[cfg(feature = "typescript-ast")]
    fn parse_typescript_content(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<(Module, Lrc<SourceMap>)> {
        let source_map: Lrc<SourceMap> = Default::default();
        let source_file =
            source_map.new_source_file(FileName::Real(path.to_path_buf()), content.to_string());

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
            .map_err(|e| anyhow::anyhow!("TypeScript parse error: {e:?}"))?;

        Ok((module, source_map))
    }

    #[cfg(feature = "typescript-ast")]
    fn parse_javascript_content(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<(Module, Lrc<SourceMap>)> {
        let source_map: Lrc<SourceMap> = Default::default();
        let source_file =
            source_map.new_source_file(FileName::Real(path.to_path_buf()), content.to_string());

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
            .map_err(|e| anyhow::anyhow!("JavaScript parse error: {e:?}"))?;

        Ok((module, source_map))
    }

    #[cfg(feature = "typescript-ast")]
    fn convert_swc_module(
        &self,
        module: &Module,
        dag: &mut AstDag,
        parent_key: NodeKey,
    ) -> Result<()> {
        for item in &module.body {
            if let Some(ast_node) = self.convert_module_item(item)? {
                let node_key = dag.add_node(ast_node);
                self.link_to_parent(dag, parent_key, node_key);
            }
        }
        Ok(())
    }

    #[cfg(feature = "typescript-ast")]
    fn convert_module_item(&self, item: &ModuleItem) -> Result<Option<UnifiedAstNode>> {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::Import(_) => {
                    let mut node = UnifiedAstNode::new(
                        AstKind::Macro(MacroKind::Include),
                        Language::TypeScript,
                    );
                    node.name_vector = self.hash_name("import");
                    Ok(Some(node))
                }
                ModuleDecl::ExportDecl(_) => {
                    let mut node = UnifiedAstNode::new(
                        AstKind::Macro(MacroKind::Export),
                        Language::TypeScript,
                    );
                    node.name_vector = self.hash_name("export");
                    Ok(Some(node))
                }
                _ => Ok(None),
            },
            ModuleItem::Stmt(stmt) => self.convert_statement(stmt),
        }
    }

    #[cfg(feature = "typescript-ast")]
    fn convert_statement(&self, stmt: &Stmt) -> Result<Option<UnifiedAstNode>> {
        let ast_kind = match stmt {
            Stmt::If(_) => Some(AstKind::Statement(StmtKind::If)),
            Stmt::While(_) => Some(AstKind::Statement(StmtKind::While)),
            Stmt::DoWhile(_) => Some(AstKind::Statement(StmtKind::DoWhile)),
            Stmt::For(_) => Some(AstKind::Statement(StmtKind::For)),
            Stmt::ForIn(_) => Some(AstKind::Statement(StmtKind::ForEach)),
            Stmt::ForOf(_) => Some(AstKind::Statement(StmtKind::ForEach)),
            Stmt::Switch(_) => Some(AstKind::Statement(StmtKind::Switch)),
            Stmt::Try(_) => Some(AstKind::Statement(StmtKind::Try)),
            Stmt::Return(_) => Some(AstKind::Statement(StmtKind::Return)),
            Stmt::Break(_) => Some(AstKind::Statement(StmtKind::Break)),
            Stmt::Continue(_) => Some(AstKind::Statement(StmtKind::Continue)),
            Stmt::Throw(_) => Some(AstKind::Statement(StmtKind::Throw)),
            Stmt::Block(_) => Some(AstKind::Statement(StmtKind::Block)),
            Stmt::Decl(decl) => return self.convert_declaration(decl),
            _ => None,
        };

        if let Some(kind) = ast_kind {
            let mut node = UnifiedAstNode::new(kind, Language::TypeScript);

            // Calculate complexity for control flow statements
            if self.is_complexity_node(stmt) {
                let complexity = self.calculate_statement_complexity(stmt);
                node.set_complexity(complexity);
            }

            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "typescript-ast")]
    fn convert_declaration(&self, decl: &Decl) -> Result<Option<UnifiedAstNode>> {
        match decl {
            Decl::Fn(_) => {
                let mut node = UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::TypeScript,
                );
                node.name_vector = self.hash_name("function");
                Ok(Some(node))
            }
            Decl::Class(_) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Type(TypeKind::Class), Language::TypeScript);
                node.name_vector = self.hash_name("class");
                Ok(Some(node))
            }
            Decl::Var(_) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Variable(VarKind::Let), Language::TypeScript);
                node.name_vector = self.hash_name("variable");
                Ok(Some(node))
            }
            Decl::TsInterface(_) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Type(TypeKind::Interface), Language::TypeScript);
                node.name_vector = self.hash_name("interface");
                Ok(Some(node))
            }
            Decl::TsTypeAlias(_) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Type(TypeKind::Alias), Language::TypeScript);
                node.name_vector = self.hash_name("type_alias");
                Ok(Some(node))
            }
            Decl::TsEnum(_) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Type(TypeKind::Enum), Language::TypeScript);
                node.name_vector = self.hash_name("enum");
                Ok(Some(node))
            }
            Decl::TsModule(_) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Type(TypeKind::Namespace), Language::TypeScript);
                node.name_vector = self.hash_name("namespace");
                Ok(Some(node))
            }
            _ => Ok(None),
        }
    }

    #[cfg(feature = "typescript-ast")]
    fn is_complexity_node(&self, stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::If(_)
                | Stmt::While(_)
                | Stmt::DoWhile(_)
                | Stmt::For(_)
                | Stmt::ForIn(_)
                | Stmt::ForOf(_)
                | Stmt::Switch(_)
                | Stmt::Try(_)
        )
    }

    #[cfg(feature = "typescript-ast")]
    fn calculate_statement_complexity(&self, stmt: &Stmt) -> u32 {
        match stmt {
            Stmt::If(_)
            | Stmt::While(_)
            | Stmt::DoWhile(_)
            | Stmt::For(_)
            | Stmt::ForIn(_)
            | Stmt::ForOf(_) => 1,
            Stmt::Switch(_) => 1, // Base complexity, cases add more
            Stmt::Try(_) => 1,    // Base complexity, catch adds more
            _ => 0,
        }
    }

    #[cfg(feature = "typescript-ast")]
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

    #[cfg(feature = "typescript-ast")]
    fn hash_name(&self, name: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "typescript-ast")]
    fn test_dispatch_parser_simple_typescript() {
        let mut parser = TsAstDispatchParser::new();
        let content = r#"
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
"#;
        let result = parser.parse_file(Path::new("test.ts"), content);
        assert!(result.is_ok());

        let dag = result.unwrap();
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    #[cfg(feature = "typescript-ast")]
    fn test_dispatch_builder() {
        let builder = TsNodeDispatchBuilder::new();
        let dispatch = builder.add_functions().add_types().add_statements().build();

        assert!(dispatch.contains_key("function_declaration"));
        assert!(dispatch.contains_key("class_declaration"));
        assert!(dispatch.contains_key("interface_declaration"));
        assert!(dispatch.contains_key("if_statement"));
    }

    #[test]
    #[cfg(feature = "typescript-ast")]
    fn test_javascript_parsing() {
        let mut parser = TsAstDispatchParser::new();
        let content = r#"
class Calculator {
    add(a, b) {
        return a + b;
    }
    
    subtract(a, b) {
        return a - b;
    }
}

export default Calculator;
"#;
        let result = parser.parse_file(Path::new("test.js"), content);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(feature = "typescript-ast"))]
    fn test_typescript_dispatch_disabled() {
        let mut parser = TsAstDispatchParser::new();
        let content = "interface A {}";
        let result = parser.parse_file(Path::new("test.ts"), content);
        assert!(result.is_err());
    }
}
