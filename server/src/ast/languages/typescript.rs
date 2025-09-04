//! TypeScript and JavaScript language AST parsing strategies

use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{Module, ModuleItem, Decl, Stmt, ModuleDecl};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig, EsConfig};

use crate::ast::core::{AstDag, Language, UnifiedAstNode, NodeFlags, AstKind, FunctionKind, ClassKind, TypeKind, ImportKind, StmtKind};
use super::LanguageStrategy;

/// TypeScript language parsing strategy
pub struct TypeScriptStrategy;

impl Default for TypeScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptStrategy {
    pub fn new() -> Self {
        Self
    }
    
    fn parse_module(&self, content: &str, filename: &str) -> Result<Module> {
        let source_map = SourceMap::default();
        let source_file = source_map.new_source_file(
            FileName::Custom(filename.to_string()),
            content.to_string(),
        );
        
        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: filename.ends_with(".tsx"),
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );
        
        let mut parser = Parser::new_from(lexer);
        parser.parse_module()
            .map_err(|e| anyhow::anyhow!("TypeScript parse error: {:?}", e))
    }
    
    fn convert_to_dag(&self, module: &Module, language: Language) -> AstDag {
        let mut dag = AstDag::new();
        let mut visitor = TypeScriptAstVisitor::new(&mut dag, language);
        visitor.visit_module(module);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for TypeScriptStrategy {
    fn language(&self) -> Language {
        Language::TypeScript
    }
    
    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "ts" | "tsx"))
            .unwrap_or(false)
    }
    
    async fn parse_file(&self, path: &Path, content: &str) -> Result<AstDag> {
        let filename = path.display().to_string();
        let module = self.parse_module(content, &filename)?;
        Ok(self.convert_to_dag(&module, Language::TypeScript))
    }
    
    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        let mut imports = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Import(_)) {
                    imports.push(format!("import_{}", i));
                }
            }
        }
        imports
    }
    
    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut functions = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Function(_)) {
                    functions.push(node.clone());
                }
            }
        }
        functions
    }
    
    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut types = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Class(_) | AstKind::Type(_)) {
                    types.push(node.clone());
                }
            }
        }
        types
    }
    
    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1;
        let mut cognitive = 0;
        
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::CONTROL_FLOW) {
                    cyclomatic += 1;
                    cognitive += 1;
                }
            }
        }
        
        (cyclomatic, cognitive)
    }
}

/// JavaScript language parsing strategy
pub struct JavaScriptStrategy;

impl Default for JavaScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptStrategy {
    pub fn new() -> Self {
        Self
    }
    
    fn parse_module(&self, content: &str, filename: &str) -> Result<Module> {
        let source_map = SourceMap::default();
        let source_file = source_map.new_source_file(
            FileName::Custom(filename.to_string()),
            content.to_string(),
        );
        
        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                jsx: filename.ends_with(".jsx"),
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );
        
        let mut parser = Parser::new_from(lexer);
        parser.parse_module()
            .map_err(|e| anyhow::anyhow!("JavaScript parse error: {:?}", e))
    }
}

#[async_trait]
impl LanguageStrategy for JavaScriptStrategy {
    fn language(&self) -> Language {
        Language::JavaScript
    }
    
    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "js" | "jsx" | "mjs"))
            .unwrap_or(false)
    }
    
    async fn parse_file(&self, path: &Path, content: &str) -> Result<AstDag> {
        let filename = path.display().to_string();
        let module = self.parse_module(content, &filename)?;
        let ts_strategy = TypeScriptStrategy::new();
        Ok(ts_strategy.convert_to_dag(&module, Language::JavaScript))
    }
    
    // Delegate other methods to TypeScript strategy since the AST is the same
    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        TypeScriptStrategy::new().extract_imports(ast)
    }
    
    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        TypeScriptStrategy::new().extract_functions(ast)
    }
    
    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        TypeScriptStrategy::new().extract_types(ast)
    }
    
    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        TypeScriptStrategy::new().calculate_complexity(ast)
    }
}

/// Visitor for converting TypeScript/JavaScript AST to unified AST
struct TypeScriptAstVisitor<'a> {
    dag: &'a mut AstDag,
    language: Language,
    current_parent: Option<u32>,
}

impl<'a> TypeScriptAstVisitor<'a> {
    fn new(dag: &'a mut AstDag, language: Language) -> Self {
        Self {
            dag,
            language,
            current_parent: None,
        }
    }
    
    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, self.language);
        
        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }
        
        self.dag.add_node(node)
    }
    
    fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
            self.visit_module_item(item);
        }
    }
    
    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::Import(_import) => {
                    let mut node = UnifiedAstNode::new(
                        AstKind::Import(ImportKind::Module),
                        self.language
                    );
                    node.flags.set(NodeFlags::IMPORT);
                    self.dag.add_node(node);
                }
                ModuleDecl::ExportDecl(export) => {
                    self.visit_decl(&export.decl);
                }
                _ => {}
            }
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
        }
    }
    
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl(decl) => self.visit_decl(decl),
            Stmt::If(_) | Stmt::While(_) | Stmt::For(_) | Stmt::Switch(_) => {
                let mut node = UnifiedAstNode::new(
                    AstKind::Statement(StmtKind::If),
                    self.language
                );
                node.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(node);
            }
            _ => {}
        }
    }
    
    fn visit_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Fn(f) => {
                let mut node = UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    self.language
                );
                
                if f.function.is_async {
                    node.flags.set(NodeFlags::ASYNC);
                }
                
                let key = self.dag.add_node(node);
                
                let old_parent = self.current_parent;
                self.current_parent = Some(key);
                // Visit function body if needed
                self.current_parent = old_parent;
            }
            Decl::Class(_c) => {
                let node = UnifiedAstNode::new(
                    AstKind::Class(ClassKind::Regular),
                    self.language
                );
                
                let key = self.dag.add_node(node);
                
                let old_parent = self.current_parent;
                self.current_parent = Some(key);
                // Visit class members if needed
                self.current_parent = old_parent;
            }
            Decl::TsInterface(_) => {
                let node = UnifiedAstNode::new(
                    AstKind::Class(ClassKind::Interface),
                    self.language
                );
                self.dag.add_node(node);
            }
            Decl::TsTypeAlias(_) => {
                let node = UnifiedAstNode::new(
                    AstKind::Type(TypeKind::Alias),
                    self.language
                );
                self.dag.add_node(node);
            }
            _ => {}
        }
    }
}