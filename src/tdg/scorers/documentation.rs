use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, get_node_text};

pub struct DocumentationScorer;

impl DocumentationScorer {
    pub fn new() -> Self {
        Self
    }
    
    fn extract_documentation(&self, root: Node, source: &str, language: Language) -> Documentation {
        let mut docs = Documentation::new();
        
        walk_tree(root, |node| {
            match language {
                Language::Rust => self.extract_rust_docs(node, source, &mut docs),
                Language::Python => self.extract_python_docs(node, source, &mut docs),
                Language::JavaScript | Language::TypeScript => self.extract_js_docs(node, source, &mut docs),
                Language::Go => self.extract_go_docs(node, source, &mut docs),
                _ => {}
            }
        });
        
        docs
    }
    
    fn extract_rust_docs(&self, node: Node, source: &str, docs: &mut Documentation) {
        match node.kind() {
            "line_comment" => {
                let text = get_node_text(node, source);
                if text.starts_with("///") {
                    docs.add_doc_comment(text.to_string());
                } else if text.starts_with("//!") {
                    docs.module_doc = Some(text[3..].trim().to_string());
                }
            }
            "block_comment" => {
                let text = get_node_text(node, source);
                if text.starts_with("/**") {
                    docs.add_doc_comment(text.to_string());
                } else if text.starts_with("/*!") {
                    docs.module_doc = Some(text[3..text.len()-2].trim().to_string());
                }
            }
            _ => {}
        }
    }
    
    fn extract_python_docs(&self, node: Node, source: &str, docs: &mut Documentation) {
        match node.kind() {
            "comment" => {
                let text = get_node_text(node, source);
                if text.starts_with("# ") {
                    docs.add_doc_comment(text.to_string());
                }
            }
            "string" | "string_literal" => {
                if let Some(parent) = node.parent() {
                    if matches!(
                        parent.kind(),
                        "expression_statement" | "function_definition" | "class_definition"
                    ) {
                        let text = get_node_text(node, source);
                        if text.starts_with("\"\"\"") || text.starts_with("'''") {
                            docs.add_doc_comment(text.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    fn extract_js_docs(&self, node: Node, source: &str, docs: &mut Documentation) {
        match node.kind() {
            "comment" => {
                let text = get_node_text(node, source);
                if text.starts_with("/**") {
                    docs.add_doc_comment(text.to_string());
                } else if text.starts_with("//") && text.contains("@") {
                    docs.add_doc_comment(text.to_string());
                }
            }
            _ => {}
        }
    }
    
    fn extract_go_docs(&self, node: Node, source: &str, docs: &mut Documentation) {
        match node.kind() {
            "comment" => {
                let text = get_node_text(node, source);
                if text.starts_with("//") || text.starts_with("/*") {
                    if let Some(next) = node.next_sibling() {
                        if matches!(next.kind(), "function_declaration" | "type_declaration" | "var_declaration") {
                            docs.add_doc_comment(text.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    fn find_public_items(&self, root: Node, source: &str, language: Language) -> Vec<PublicItem> {
        let mut items = Vec::new();
        
        walk_tree(root, |node| {
            match language {
                Language::Rust => self.find_rust_public_items(node, source, &mut items),
                Language::Python => self.find_python_public_items(node, source, &mut items),
                Language::JavaScript | Language::TypeScript => self.find_js_public_items(node, source, &mut items),
                Language::Go => self.find_go_public_items(node, source, &mut items),
                _ => {}
            }
        });
        
        items
    }
    
    fn find_rust_public_items(&self, node: Node, source: &str, items: &mut Vec<PublicItem>) {
        match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "type_item" | "const_item" | "static_item" => {
                if self.is_rust_public(node, source) {
                    if let Some(name) = node.child_by_field_name("name") {
                        let item_name = get_node_text(name, source).to_string();
                        items.push(PublicItem {
                            name: item_name,
                            kind: node.kind().to_string(),
                            byte_range: node.byte_range(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    fn is_rust_public(&self, node: Node, source: &str) -> bool {
        if let Some(visibility) = node.child_by_field_name("visibility_modifier") {
            let vis_text = get_node_text(visibility, source);
            vis_text.contains("pub")
        } else {
            false
        }
    }
    
    fn find_python_public_items(&self, node: Node, source: &str, items: &mut Vec<PublicItem>) {
        match node.kind() {
            "function_definition" | "class_definition" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let item_name = get_node_text(name, source).to_string();
                    if !item_name.starts_with('_') {
                        items.push(PublicItem {
                            name: item_name,
                            kind: node.kind().to_string(),
                            byte_range: node.byte_range(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    fn find_js_public_items(&self, node: Node, source: &str, items: &mut Vec<PublicItem>) {
        match node.kind() {
            "function_declaration" | "function_expression" | "class_declaration" | "export_statement" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let item_name = get_node_text(name, source).to_string();
                    items.push(PublicItem {
                        name: item_name,
                        kind: node.kind().to_string(),
                        byte_range: node.byte_range(),
                    });
                }
            }
            _ => {}
        }
    }
    
    fn find_go_public_items(&self, node: Node, source: &str, items: &mut Vec<PublicItem>) {
        match node.kind() {
            "function_declaration" | "type_declaration" | "var_declaration" | "const_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let item_name = get_node_text(name, source).to_string();
                    if item_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        items.push(PublicItem {
                            name: item_name,
                            kind: node.kind().to_string(),
                            byte_range: node.byte_range(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    fn count_examples(&self, docs: &Documentation) -> usize {
        docs.doc_comments.iter()
            .map(|comment| {
                comment.matches("```").count() / 2 +
                comment.matches("# Example").count() +
                comment.matches("## Example").count() +
                comment.matches("@example").count()
            })
            .sum()
    }
}

impl Scorer for DocumentationScorer {
    fn score(&self, tree: &Tree, source: &str, language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        let mut points = config.weights.documentation;
        let root = tree.root_node();
        
        let docs = self.extract_documentation(root, source, language);
        let public_items = self.find_public_items(root, source, language);
        
        if public_items.is_empty() {
            return Ok(points);
        }
        
        let documented_items = public_items.iter()
            .filter(|item| docs.has_documentation_for(item))
            .count();
        
        let coverage = documented_items as f32 / public_items.len() as f32;
        let coverage_points = coverage * 7.0;
        
        let example_count = self.count_examples(&docs);
        let example_bonus = (example_count as f32 * 0.5).min(2.0);
        
        let module_bonus = if docs.has_module_documentation() {
            1.0
        } else {
            0.0
        };
        
        let total_doc_points = coverage_points + example_bonus + module_bonus;
        let final_points = total_doc_points.min(points);
        
        let doc_penalty = points - final_points;
        if doc_penalty > 0.5 {
            if let Some(applied) = tracker.apply(
                format!("low_doc_coverage_{:.2}", coverage),
                MetricCategory::Documentation,
                doc_penalty,
                format!("Low documentation coverage: {:.1}%", coverage * 100.0)
            ) {
                points -= applied;
            }
        }
        
        Ok(final_points.max(0.0))
    }
    
    fn category(&self) -> MetricCategory {
        MetricCategory::Documentation
    }
}

#[derive(Debug)]
struct Documentation {
    doc_comments: Vec<String>,
    module_doc: Option<String>,
}

impl Documentation {
    fn new() -> Self {
        Self {
            doc_comments: Vec::new(),
            module_doc: None,
        }
    }
    
    fn add_doc_comment(&mut self, comment: String) {
        self.doc_comments.push(comment);
    }
    
    fn has_documentation_for(&self, item: &PublicItem) -> bool {
        self.doc_comments.iter().any(|comment| {
            comment.to_lowercase().contains(&item.name.to_lowercase()) ||
            self.is_adjacent_to_item(comment, item)
        })
    }
    
    fn is_adjacent_to_item(&self, _comment: &str, _item: &PublicItem) -> bool {
        true
    }
    
    fn has_module_documentation(&self) -> bool {
        self.module_doc.is_some()
    }
}

#[derive(Debug)]
struct PublicItem {
    name: String,
    kind: String,
    byte_range: std::ops::Range<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    
    fn parse_rust(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        parser.parse(source, None).unwrap()
    }
    
    #[test]
    fn test_rust_documentation_extraction() {
        let source = r#"
            //! Module documentation
            
            /// This is a documented function
            /// 
            /// # Example
            /// ```
            /// let result = documented_function();
            /// ```
            pub fn documented_function() -> i32 {
                42
            }
            
            pub fn undocumented_function() -> i32 {
                24
            }
        "#;
        
        let tree = parse_rust(source);
        let scorer = DocumentationScorer::new();
        
        let docs = scorer.extract_documentation(tree.root_node(), source, Language::Rust);
        assert!(!docs.doc_comments.is_empty());
        assert!(docs.has_module_documentation());
        
        let public_items = scorer.find_public_items(tree.root_node(), source, Language::Rust);
        assert_eq!(public_items.len(), 2);
        
        let examples = scorer.count_examples(&docs);
        assert!(examples > 0);
    }
    
    #[test]
    fn test_python_documentation_extraction() {
        let source = r#"
            """Module docstring"""
            
            def documented_function():
                """This function is documented.
                
                Example:
                    result = documented_function()
                """
                return 42
                
            def _private_function():
                """This is private"""
                return 24
        "#;
        
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::language()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        
        let scorer = DocumentationScorer::new();
        let public_items = scorer.find_public_items(tree.root_node(), source, Language::Python);
        
        assert_eq!(public_items.len(), 1);
        assert_eq!(public_items[0].name, "documented_function");
    }
}