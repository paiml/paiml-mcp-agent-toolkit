//! C and C++ language AST parsing strategies

use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use tree_sitter::{Parser as TsParser, Tree};

use crate::ast::core::{AstDag, Language, UnifiedAstNode, NodeFlags, AstKind, FunctionKind, ClassKind, ImportKind, StmtKind};
use super::LanguageStrategy;

/// C language parsing strategy  
pub struct CStrategy;

impl CStrategy {
    pub fn new() -> Self {
        Self
    }
    
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = TsParser::new();
        let language = tree_sitter_c::language();
        parser.set_language(&language)
            .map_err(|e| anyhow::anyhow!("Failed to set C language: {}", e))?;
        
        parser.parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse C code"))
    }
    
    fn convert_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);
        visitor.visit_node(&root, None);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for CStrategy {
    fn language(&self) -> Language {
        Language::C
    }
    
    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "c" | "h"))
            .unwrap_or(false)
    }
    
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_to_dag(&tree, content))
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
                if matches!(node.kind, AstKind::Class(_)) {
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

/// C++ language parsing strategy
pub struct CppStrategy;

impl CppStrategy {
    pub fn new() -> Self {
        Self
    }
    
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = TsParser::new();
        let language = tree_sitter_cpp::language();
        parser.set_language(&language)
            .map_err(|e| anyhow::anyhow!("Failed to set C++ language: {}", e))?;
        
        parser.parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse C++ code"))
    }
    
    fn convert_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);
        visitor.visit_node(&root, None);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for CppStrategy {
    fn language(&self) -> Language {
        Language::Cpp
    }
    
    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx"))
            .unwrap_or(false)
    }
    
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_to_dag(&tree, content))
    }
    
    // Delegate to C strategy since the AST structure is similar
    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        CStrategy::new().extract_imports(ast)
    }
    
    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        CStrategy::new().extract_functions(ast)
    }
    
    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        CStrategy::new().extract_types(ast)
    }
    
    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        CStrategy::new().calculate_complexity(ast)
    }
}

/// Tree-sitter visitor for C/C++ AST conversion
struct CTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    content: &'a str,
    language: Language,
    current_parent: Option<u32>,
}

impl<'a> CTreeSitterVisitor<'a> {
    fn new(dag: &'a mut AstDag, content: &'a str, language: Language) -> Self {
        Self {
            dag,
            content,
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
    
    fn visit_node(&mut self, node: &tree_sitter::Node, parent: Option<u32>) {
        let old_parent = self.current_parent;
        self.current_parent = parent;
        
        match node.kind() {
            "function_definition" | "function_declarator" => {
                let key = self.add_node(AstKind::Function(FunctionKind::Regular));
                
                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "struct_specifier" => {
                let key = self.add_node(AstKind::Class(ClassKind::Struct));
                
                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "enum_specifier" => {
                let key = self.add_node(AstKind::Class(ClassKind::Enum));
                
                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "class_specifier" => {
                // C++ specific
                let key = self.add_node(AstKind::Class(ClassKind::Regular));
                
                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "preproc_include" => {
                let mut n = UnifiedAstNode::new(
                    AstKind::Import(ImportKind::Module),
                    self.language
                );
                n.flags.set(NodeFlags::IMPORT);
                self.dag.add_node(n);
            }
            "if_statement" | "while_statement" | "for_statement" | "switch_statement" => {
                let mut n = UnifiedAstNode::new(
                    AstKind::Statement(StmtKind::If),
                    self.language
                );
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);
                
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            _ => {
                // Visit children for other node types
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
        }
        
        self.current_parent = old_parent;
    }
}