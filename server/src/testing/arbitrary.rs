use quickcheck::{Arbitrary, Gen};
use crate::models::unified_ast::*;
use crate::models::dag::*;
use std::path::PathBuf;
use std::ops::Range;

impl Arbitrary for UnifiedAstNode {
    fn arbitrary(g: &mut Gen) -> Self {
        UnifiedAstNode {
            kind: AstKind::arbitrary(g),
            lang: Language::arbitrary(g),
            flags: NodeFlags::new(),
            parent: g.gen_range(0..1000),
            first_child: g.gen_range(0..1000),
            next_sibling: g.gen_range(0..1000),
            source_range: {
                let start = g.gen_range(0..10000);
                let end = start + g.gen_range(1..100);
                Range { start, end }
            },
            semantic_hash: u64::arbitrary(g),
            structural_hash: u64::arbitrary(g),
            name_vector: u64::arbitrary(g),
            metadata: NodeMetadata { raw: 0 },
            proof_annotations: if g.gen_bool(0.1) {
                Some(Vec::new()) // Simple case for now
            } else {
                None
            },
        }
    }
    
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(std::iter::once(UnifiedAstNode {
            kind: AstKind::Expression(ExprKind::Literal),
            lang: Language::Rust,
            flags: NodeFlags::new(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: Range { start: 0, end: 1 },
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: NodeMetadata { raw: 0 },
            proof_annotations: None,
        }))
    }
}

impl Arbitrary for AstKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..8) {
            0 => AstKind::Function(FunctionKind::arbitrary(g)),
            1 => AstKind::Class(ClassKind::arbitrary(g)),
            2 => AstKind::Variable(VarKind::arbitrary(g)),
            3 => AstKind::Import(ImportKind::arbitrary(g)),
            4 => AstKind::Expression(ExprKind::arbitrary(g)),
            5 => AstKind::Statement(StmtKind::arbitrary(g)),
            6 => AstKind::Type(TypeKind::arbitrary(g)),
            _ => AstKind::Module(ModuleKind::arbitrary(g)),
        }
    }
}

impl Arbitrary for FunctionKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..7) {
            0 => FunctionKind::Regular,
            1 => FunctionKind::Method,
            2 => FunctionKind::Constructor,
            3 => FunctionKind::Getter,
            4 => FunctionKind::Setter,
            5 => FunctionKind::Lambda,
            _ => FunctionKind::Closure,
        }
    }
}

impl Arbitrary for ClassKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..6) {
            0 => ClassKind::Regular,
            1 => ClassKind::Abstract,
            2 => ClassKind::Interface,
            3 => ClassKind::Trait,
            4 => ClassKind::Enum,
            _ => ClassKind::Struct,
        }
    }
}

impl Arbitrary for VarKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..4) {
            0 => VarKind::Local,
            1 => VarKind::Parameter,
            2 => VarKind::Field,
            _ => VarKind::Global,
        }
    }
}

impl Arbitrary for ImportKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..4) {
            0 => ImportKind::Named,
            1 => ImportKind::Default,
            2 => ImportKind::Namespace,
            _ => ImportKind::Wildcard,
        }
    }
}

impl Arbitrary for ExprKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..5) {
            0 => ExprKind::Literal,
            1 => ExprKind::Variable,
            2 => ExprKind::FunctionCall,
            3 => ExprKind::BinaryOp,
            _ => ExprKind::UnaryOp,
        }
    }
}

impl Arbitrary for StmtKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..6) {
            0 => StmtKind::Expression,
            1 => StmtKind::Assignment,
            2 => StmtKind::If,
            3 => StmtKind::Loop,
            4 => StmtKind::Return,
            _ => StmtKind::Block,
        }
    }
}

impl Arbitrary for TypeKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..4) {
            0 => TypeKind::Primitive,
            1 => TypeKind::Array,
            2 => TypeKind::Object,
            _ => TypeKind::Generic,
        }
    }
}

impl Arbitrary for ModuleKind {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..3) {
            0 => ModuleKind::File,
            1 => ModuleKind::Directory,
            _ => ModuleKind::Namespace,
        }
    }
}

impl Arbitrary for Language {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..4) {
            0 => Language::Rust,
            1 => Language::TypeScript,
            2 => Language::Python,
            _ => Language::Other("unknown".to_string()),
        }
    }
}

impl Arbitrary for Symbol {
    fn arbitrary(g: &mut Gen) -> Self {
        Symbol {
            name: arbitrary_identifier(g),
            kind: match g.gen_range(0..5) {
                0 => SymbolKind::Function,
                1 => SymbolKind::Class,
                2 => SymbolKind::Variable,
                3 => SymbolKind::Module,
                _ => SymbolKind::Other,
            },
            visibility: match g.gen_range(0..3) {
                0 => Visibility::Public,
                1 => Visibility::Private,
                _ => Visibility::Internal,
            },
            location: SourceLocation::arbitrary(g),
            language: Language::arbitrary(g),
        }
    }
}

impl Arbitrary for SourceLocation {
    fn arbitrary(g: &mut Gen) -> Self {
        SourceLocation {
            file_path: PathBuf::from(format!("src/file_{}.rs", g.gen_range(1..100))),
            line: g.gen_range(1..1000),
            column: g.gen_range(1..100),
            byte_offset: g.gen_range(0..10000),
        }
    }
}

impl Arbitrary for DependencyGraph {
    fn arbitrary(g: &mut Gen) -> Self {
        let node_count = g.gen_range(1..20);
        let mut graph = DependencyGraph::new();
        
        // Add nodes
        for i in 0..node_count {
            let node = DependencyNode {
                id: format!("node_{}", i),
                name: arbitrary_identifier(g),
                kind: match g.gen_range(0..4) {
                    0 => "function".to_string(),
                    1 => "class".to_string(), 
                    2 => "module".to_string(),
                    _ => "variable".to_string(),
                },
                file_path: PathBuf::from(format!("src/module_{}.rs", g.gen_range(1..10))),
                language: Language::arbitrary(g),
            };
            graph.add_node(node);
        }
        
        // Add some edges (dependencies)
        let edge_count = g.gen_range(0..node_count * 2);
        for _ in 0..edge_count {
            let from = g.gen_range(0..node_count);
            let to = g.gen_range(0..node_count);
            if from != to {
                graph.add_edge(
                    &format!("node_{}", from),
                    &format!("node_{}", to),
                    DependencyType::Uses,
                );
            }
        }
        
        graph
    }
}

impl Arbitrary for DependencyType {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..4) {
            0 => DependencyType::Uses,
            1 => DependencyType::Calls,
            2 => DependencyType::Imports,
            _ => DependencyType::Inherits,
        }
    }
}

// Domain-specific generators
fn arbitrary_identifier(g: &mut Gen) -> String {
    let prefixes = ["", "_", "test_", "impl_", "__"];
    let bases = ["foo", "bar", "baz", "data", "value", "item", "node", "element"];
    
    let prefix = prefixes.choose(g).unwrap();
    let base = bases.choose(g).unwrap();
    
    if g.gen_bool(0.1) {  // 10% chance of edge case
        format!("{}{}{}", prefix, base, g.gen_range(0..1000))
    } else {
        format!("{}{}", prefix, base)
    }
}

fn arbitrary_type_name(g: &mut Gen) -> String {
    let types = ["String", "i32", "f64", "bool", "Vec<T>", "Option<T>", "Result<T, E>"];
    types.choose(g).unwrap().to_string()
}

fn arbitrary_module_path(g: &mut Gen) -> String {
    let modules = ["std", "core", "alloc", "super", "crate", "self"];
    let submodules = ["collections", "fs", "io", "sync", "thread"];
    
    if g.gen_bool(0.3) {
        format!("{}::{}", modules.choose(g).unwrap(), submodules.choose(g).unwrap())
    } else {
        modules.choose(g).unwrap().to_string()
    }
}

fn arbitrary_import_list(g: &mut Gen) -> Vec<String> {
    let items = ["HashMap", "Vec", "String", "Result", "Option", "Arc", "Mutex"];
    let count = g.gen_range(1..5);
    (0..count).map(|_| items.choose(g).unwrap().to_string()).collect()
}

fn arbitrary_bounded_vec<T: Arbitrary>(g: &mut Gen, min: usize, max: usize) -> Vec<T> {
    let size = g.gen_range(min..=max);
    (0..size).map(|_| T::arbitrary(g)).collect()
}

// File system related arbitrary implementations
impl Arbitrary for PathBuf {
    fn arbitrary(g: &mut Gen) -> Self {
        let components = ["src", "tests", "lib", "bin", "examples"];
        let files = ["main.rs", "lib.rs", "mod.rs", "test.rs"];
        let extensions = ["rs", "ts", "py", "js"];
        
        if g.gen_bool(0.7) {
            // Normal file path
            PathBuf::from(format!(
                "{}/{}.{}",
                components.choose(g).unwrap(),
                arbitrary_identifier(g),
                extensions.choose(g).unwrap()
            ))
        } else {
            // Edge case paths
            match g.gen_range(0..4) {
                0 => PathBuf::from(""),  // Empty path
                1 => PathBuf::from("/"),  // Root path
                2 => PathBuf::from("./relative/path"),  // Relative path
                _ => PathBuf::from(files.choose(g).unwrap()),  // Just filename
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;

    #[test]
    fn test_ast_node_arbitrary_terminates() {
        let mut g = Gen::new(10);
        let _node = UnifiedAstNode::arbitrary(&mut g);
        // If we reach here, arbitrary generation terminated successfully
    }

    #[test]
    fn test_dependency_graph_has_nodes() {
        let mut g = Gen::new(10);
        let graph = DependencyGraph::arbitrary(&mut g);
        assert!(graph.nodes.len() > 0);
    }

    #[test]
    fn test_symbol_arbitrary_valid_name() {
        let mut g = Gen::new(10);
        let symbol = Symbol::arbitrary(&mut g);
        assert!(!symbol.name.is_empty());
    }
}
