use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, get_node_text};

pub struct CouplingAnalyzer;

impl CouplingAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    fn calculate_afferent_coupling(&self, root: Node, source: &str) -> usize {
        let mut incoming = HashSet::new();
        
        walk_tree(root, |node| {
            match node.kind() {
                "function_item" | "impl_item" | "struct_item" | "trait_item" => {
                    if let Some(name) = node.child_by_field_name("name") {
                        let fn_name = get_node_text(name, source);
                        if self.is_public(node, source) {
                            incoming.insert(fn_name.to_string());
                        }
                    }
                }
                _ => {}
            }
        });
        
        incoming.len()
    }
    
    fn calculate_efferent_coupling(&self, root: Node, source: &str) -> usize {
        let mut outgoing = HashSet::new();
        
        walk_tree(root, |node| {
            match node.kind() {
                "use_declaration" | "use" | "import" | "extern_crate_declaration" => {
                    if let Some(path) = self.extract_import_path(node, source) {
                        outgoing.insert(path);
                    }
                }
                "call_expression" => {
                    if let Some(function) = node.child_by_field_name("function") {
                        let fn_text = get_node_text(function, source);
                        if fn_text.contains("::") {
                            outgoing.insert(fn_text.to_string());
                        }
                    }
                }
                "type_identifier" | "generic_type" => {
                    let type_text = get_node_text(node, source);
                    if !self.is_builtin_type(type_text) {
                        outgoing.insert(type_text.to_string());
                    }
                }
                _ => {}
            }
        });
        
        outgoing.len()
    }
    
    fn is_public(&self, node: Node, source: &str) -> bool {
        if let Some(visibility) = node.child_by_field_name("visibility_modifier") {
            let vis_text = get_node_text(visibility, source);
            vis_text.contains("pub")
        } else {
            false
        }
    }
    
    fn extract_import_path(&self, node: Node, source: &str) -> Option<String> {
        if let Some(path) = node.child_by_field_name("path") {
            Some(get_node_text(path, source).to_string())
        } else if let Some(argument) = node.child_by_field_name("argument") {
            Some(get_node_text(argument, source).to_string())
        } else {
            let text = get_node_text(node, source);
            let parts: Vec<&str> = text.split_whitespace().collect();
            if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            }
        }
    }
    
    fn is_builtin_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
            "f32" | "f64" | "bool" | "char" | "str" | "String" |
            "Vec" | "HashMap" | "HashSet" | "Option" | "Result" |
            "Box" | "Rc" | "Arc" | "Cell" | "RefCell" | "Mutex" |
            "int" | "float" | "double" | "void" | "auto" | "const"
        )
    }
    
    fn calculate_abstractness(&self, root: Node, source: &str) -> f32 {
        let mut abstract_count = 0;
        let mut total_count = 0;
        
        walk_tree(root, |node| {
            match node.kind() {
                "trait_item" => {
                    abstract_count += 1;
                    total_count += 1;
                }
                "impl_item" => {
                    total_count += 1;
                    if node.child_by_field_name("trait").is_some() {
                        abstract_count += 1;
                    }
                }
                "struct_item" | "enum_item" => {
                    total_count += 1;
                }
                _ => {}
            }
        });
        
        if total_count > 0 {
            abstract_count as f32 / total_count as f32
        } else {
            0.0
        }
    }
    
    fn build_dependency_graph(&self, root: Node, source: &str) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        let mut current_module = None;
        
        walk_tree(root, |node| {
            match node.kind() {
                "mod_item" | "module" => {
                    if let Some(name) = node.child_by_field_name("name") {
                        current_module = Some(get_node_text(name, source).to_string());
                    }
                }
                "use_declaration" | "use" | "import" => {
                    if let Some(module) = &current_module {
                        if let Some(imported) = self.extract_import_path(node, source) {
                            graph.add_edge(module.clone(), imported);
                        }
                    }
                }
                _ => {}
            }
        });
        
        graph
    }
}

impl Scorer for CouplingAnalyzer {
    fn score(&self, tree: &Tree, source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        let mut points = config.weights.coupling;
        let root = tree.root_node();
        
        let afferent = self.calculate_afferent_coupling(root, source);
        let efferent = self.calculate_efferent_coupling(root, source);
        
        let instability = if afferent + efferent > 0 {
            efferent as f32 / (afferent + efferent) as f32
        } else {
            0.0
        };
        
        let abstractness = self.calculate_abstractness(root, source);
        let distance = (instability + abstractness - 1.0).abs();
        
        if afferent + efferent > config.thresholds.max_coupling {
            let excess = (afferent + efferent - config.thresholds.max_coupling) as f32;
            let penalty = config.penalties.coupling_penalty_curve.apply(excess * 0.3, 1.0).min(7.0);
            
            if let Some(applied) = tracker.apply(
                format!("high_coupling_{}", afferent + efferent),
                MetricCategory::Coupling,
                penalty,
                format!("High coupling: Ca={}, Ce={}", afferent, efferent)
            ) {
                points -= applied;
            }
        }
        
        let distance_penalty = (distance * 8.0).min(8.0);
        if distance_penalty > 0.5 {
            if let Some(applied) = tracker.apply(
                format!("main_sequence_distance_{:.2}", distance),
                MetricCategory::Coupling,
                distance_penalty,
                format!("Distance from main sequence: {:.2}", distance)
            ) {
                points -= applied;
            }
        }
        
        Ok(points.max(0.0))
    }
    
    fn category(&self) -> MetricCategory {
        MetricCategory::Coupling
    }
}

struct DependencyGraph {
    edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }
    
    fn add_edge(&mut self, from: String, to: String) {
        self.edges.entry(from).or_insert_with(HashSet::new).insert(to);
    }
    
    fn topological_sort(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        for node in self.edges.keys() {
            if !visited.contains(node) {
                self.dfs(node, &mut visited, &mut stack);
            }
        }
        
        stack.reverse();
        stack
    }
    
    fn dfs(&self, node: &str, visited: &mut HashSet<String>, stack: &mut Vec<String>) {
        visited.insert(node.to_string());
        
        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs(neighbor, visited, stack);
                }
            }
        }
        
        stack.push(node.to_string());
    }
    
    fn calculate_depth(&self) -> usize {
        let topo_order = self.topological_sort();
        let mut depths = HashMap::new();
        let mut max_depth = 0;
        
        for node in topo_order {
            let incoming_depth = self.edges.values()
                .filter_map(|deps| {
                    if deps.contains(&node) {
                        deps.iter()
                            .filter_map(|dep| depths.get(dep))
                            .max()
                            .copied()
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0);
            
            let node_depth = incoming_depth + 1;
            depths.insert(node.clone(), node_depth);
            max_depth = max_depth.max(node_depth);
        }
        
        max_depth
    }
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
    fn test_coupling_calculation() {
        let source = r#"
            use std::collections::HashMap;
            use crate::module::SomeType;
            
            pub struct MyStruct {
                field: HashMap<String, SomeType>,
            }
            
            impl MyStruct {
                pub fn new() -> Self {
                    Self {
                        field: HashMap::new(),
                    }
                }
            }
        "#;
        
        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        
        let afferent = analyzer.calculate_afferent_coupling(tree.root_node(), source);
        let efferent = analyzer.calculate_efferent_coupling(tree.root_node(), source);
        
        assert!(afferent >= 0);
        assert!(efferent > 0);
    }
    
    #[test]
    fn test_abstractness() {
        let source = r#"
            trait MyTrait {
                fn method(&self);
            }
            
            struct Concrete;
            
            impl MyTrait for Concrete {
                fn method(&self) {}
            }
        "#;
        
        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let abstractness = analyzer.calculate_abstractness(tree.root_node(), source);
        
        assert!(abstractness > 0.0);
        assert!(abstractness <= 1.0);
    }
}