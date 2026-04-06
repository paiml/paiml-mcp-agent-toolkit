#![cfg_attr(coverage_nightly, coverage(off))]
//! AST visitor for converting TypeScript/JavaScript AST to unified AST

#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::{Decl, Module, ModuleDecl, ModuleItem, Stmt};

use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, ImportKind, Language, NodeFlags, StmtKind, TypeKind,
    UnifiedAstNode,
};

/// Visitor for converting TypeScript/JavaScript AST to unified AST
pub(crate) struct TypeScriptAstVisitor<'a> {
    dag: &'a mut AstDag,
    language: Language,
    current_parent: Option<u32>,
}

impl<'a> TypeScriptAstVisitor<'a> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn new(dag: &'a mut AstDag, language: Language) -> Self {
        Self {
            dag,
            language,
            current_parent: None,
        }
    }

    #[allow(dead_code)]
    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, self.language);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::Import(_import) => {
                    let mut node =
                        UnifiedAstNode::new(AstKind::Import(ImportKind::Module), self.language);
                    node.flags.set(NodeFlags::IMPORT);
                    self.dag.add_node(node);
                }
                ModuleDecl::ExportDecl(export) => {
                    self.visit_decl(&export.decl);
                }
                _ => {}
            },
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl(decl) => self.visit_decl(decl),
            Stmt::Expr(expr_stmt) => self.visit_expr(&expr_stmt.expr),
            Stmt::If(_) | Stmt::While(_) | Stmt::For(_) | Stmt::Switch(_) => {
                let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), self.language);
                node.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(node);
            }
            _ => {}
        }
    }

    fn visit_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Fn(f) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), self.language);

                if f.function.is_async {
                    node.flags.set(NodeFlags::ASYNC);
                }

                let key = self.dag.add_node(node);

                let old_parent = self.current_parent;
                self.current_parent = Some(key);
                // Visit function body if needed
                self.current_parent = old_parent;
            }
            Decl::Class(c) => {
                let node = UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), self.language);
                let key = self.dag.add_node(node);

                let old_parent = self.current_parent;
                self.current_parent = Some(key);

                // Visit class members and extract methods as functions
                for member in &c.class.body {
                    match member {
                        swc_ecma_ast::ClassMember::Method(_method) => {
                            let mut method_node = UnifiedAstNode::new(
                                AstKind::Function(FunctionKind::Regular),
                                self.language,
                            );
                            method_node.parent = key;
                            self.dag.add_node(method_node);
                        }
                        swc_ecma_ast::ClassMember::Constructor(_) => {
                            let mut ctor_node = UnifiedAstNode::new(
                                AstKind::Function(FunctionKind::Regular),
                                self.language,
                            );
                            ctor_node.parent = key;
                            self.dag.add_node(ctor_node);
                        }
                        _ => {}
                    }
                }

                self.current_parent = old_parent;
            }
            Decl::TsInterface(_) => {
                let node = UnifiedAstNode::new(AstKind::Class(ClassKind::Interface), self.language);
                self.dag.add_node(node);
            }
            Decl::TsTypeAlias(_) => {
                let node = UnifiedAstNode::new(AstKind::Type(TypeKind::Alias), self.language);
                self.dag.add_node(node);
            }
            Decl::Var(var_decl) => {
                // Handle variable declarations that might contain function expressions
                for declarator in &var_decl.decls {
                    if let Some(init) = &declarator.init {
                        self.visit_expr(init);
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &swc_ecma_ast::Expr) {
        match expr {
            swc_ecma_ast::Expr::Fn(fn_expr) => {
                self.add_function_node(fn_expr.function.is_async);
            }
            swc_ecma_ast::Expr::Arrow(arrow_fn) => {
                self.add_function_node(arrow_fn.is_async);
            }
            swc_ecma_ast::Expr::Object(obj_lit) => {
                self.visit_object_props(obj_lit);
            }
            swc_ecma_ast::Expr::Call(call_expr) => {
                self.visit_call_expr(call_expr);
            }
            _ => {}
        }
    }

    fn add_function_node(&mut self, is_async: bool) {
        let mut node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), self.language);
        if is_async {
            node.flags.set(NodeFlags::ASYNC);
        }
        self.dag.add_node(node);
    }

    fn visit_object_props(&mut self, obj_lit: &swc_ecma_ast::ObjectLit) {
        for prop_or_spread in &obj_lit.props {
            let swc_ecma_ast::PropOrSpread::Prop(prop) = prop_or_spread else {
                continue;
            };
            match prop.as_ref() {
                swc_ecma_ast::Prop::Method(_) => {
                    let node = UnifiedAstNode::new(
                        AstKind::Function(FunctionKind::Regular),
                        self.language,
                    );
                    self.dag.add_node(node);
                }
                swc_ecma_ast::Prop::KeyValue(kv_prop) => {
                    self.visit_expr(&kv_prop.value);
                }
                _ => {}
            }
        }
    }

    fn visit_call_expr(&mut self, call_expr: &swc_ecma_ast::CallExpr) {
        if let swc_ecma_ast::Callee::Expr(expr) = &call_expr.callee {
            self.visit_expr(expr);
        }
        for arg in &call_expr.args {
            self.visit_expr(&arg.expr);
        }
    }
}
