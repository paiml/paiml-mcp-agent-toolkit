use anyhow::Result;
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};

pub mod complexity;
pub mod coupling;
pub mod documentation;
pub mod duplication;
pub mod consistency;

pub use complexity::{StructuralComplexityScorer, SemanticComplexityScorer};
pub use coupling::CouplingAnalyzer;
pub use documentation::DocumentationScorer;
pub use duplication::DuplicationDetector;
pub use consistency::ConsistencyAnalyzer;

pub trait Scorer: Send + Sync {
    fn score(&self, tree: &Tree, source: &str, language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32>;
    fn category(&self) -> MetricCategory;
}

pub struct ScorerSet {
    scorers: Vec<Box<dyn Scorer>>,
}

impl ScorerSet {
    pub fn new() -> Self {
        Self {
            scorers: vec![
                Box::new(StructuralComplexityScorer::new()),
                Box::new(SemanticComplexityScorer::new()),
                Box::new(DuplicationDetector::new()),
                Box::new(CouplingAnalyzer::new()),
                Box::new(DocumentationScorer::new()),
                Box::new(ConsistencyAnalyzer::new()),
            ],
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Scorer>> {
        self.scorers.iter()
    }
}

pub fn walk_tree<F>(node: Node, mut callback: F)
where
    F: FnMut(Node),
{
    callback(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_tree(child, &mut callback);
    }
}

pub fn count_nodes_of_kind(node: Node, kind: &str) -> usize {
    let mut count = 0;
    walk_tree(node, |n| {
        if n.kind() == kind {
            count += 1;
        }
    });
    count
}

pub fn max_depth(node: Node) -> usize {
    if node.child_count() == 0 {
        return 0;
    }
    
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .map(|child| max_depth(child) + 1)
        .max()
        .unwrap_or(0)
}

pub fn get_node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    
    #[test]
    fn test_walk_tree() {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        
        let source = "fn main() { let x = 1; }";
        let tree = parser.parse(source, None).unwrap();
        
        let mut count = 0;
        walk_tree(tree.root_node(), |_| count += 1);
        assert!(count > 0);
    }
    
    #[test]
    fn test_max_depth() {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        
        let source = "fn main() { if true { if false { let x = 1; } } }";
        let tree = parser.parse(source, None).unwrap();
        
        let depth = max_depth(tree.root_node());
        assert!(depth > 3);
    }
}