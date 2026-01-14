//! Shrinking strategies for property-based testing
//! 
//! This module provides custom shrinking strategies to find minimal
//! failing test cases efficiently.

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

/// Custom shrinker for AST structures
#[derive(Debug, Clone)]
pub struct AstStructure {
    pub kind: AstKind,
    pub children: Vec<Box<AstStructure>>,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub enum AstKind {
    Empty,
    Function(String),
    Struct(String),
    Enum(String),
    Module(String),
    Sequence,
}

impl AstStructure {
    /// Create an empty AST
    pub fn empty() -> Self {
        Self {
            kind: AstKind::Empty,
            children: Vec::new(),
            depth: 0,
        }
    }
    
    /// Convert to source code
    pub fn to_source(&self) -> String {
        match &self.kind {
            AstKind::Empty => String::new(),
            AstKind::Function(name) => format!("fn {}() {{}}", name),
            AstKind::Struct(name) => format!("struct {} {{}}", name),
            AstKind::Enum(name) => format!("enum {} {{}}", name),
            AstKind::Module(name) => {
                let children_src = self.children
                    .iter()
                    .map(|c| c.to_source())
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("mod {} {{\n{}\n}}", name, children_src)
            }
            AstKind::Sequence => {
                self.children
                    .iter()
                    .map(|c| c.to_source())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
    
    /// Shrink by removing nodes
    pub fn shrink_remove(&self) -> Vec<Self> {
        let mut shrunken = Vec::new();
        
        // Try removing each child
        for i in 0..self.children.len() {
            let mut new_children = self.children.clone();
            new_children.remove(i);
            shrunken.push(Self {
                kind: self.kind.clone(),
                children: new_children,
                depth: self.depth,
            });
        }
        
        // Try replacing with empty
        if !matches!(self.kind, AstKind::Empty) {
            shrunken.push(Self::empty());
        }
        
        shrunken
    }
    
    /// Shrink by simplifying nodes
    pub fn shrink_simplify(&self) -> Vec<Self> {
        let mut shrunken = Vec::new();
        
        // Try simplifying the kind
        match &self.kind {
            AstKind::Module(name) if !name.is_empty() => {
                // Simplify module name
                shrunken.push(Self {
                    kind: AstKind::Module("m".to_string()),
                    children: self.children.clone(),
                    depth: self.depth,
                });
            }
            AstKind::Function(name) if name.len() > 1 => {
                // Simplify function name
                shrunken.push(Self {
                    kind: AstKind::Function("f".to_string()),
                    children: self.children.clone(),
                    depth: self.depth,
                });
            }
            _ => {}
        }
        
        // Try simplifying children recursively
        for (i, child) in self.children.iter().enumerate() {
            for simplified in child.shrink_simplify() {
                let mut new_children = self.children.clone();
                new_children[i] = Box::new(simplified);
                shrunken.push(Self {
                    kind: self.kind.clone(),
                    children: new_children,
                    depth: self.depth,
                });
            }
        }
        
        shrunken
    }
}

impl Arbitrary for AstStructure {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            Just(AstStructure::empty()),
            "[a-z]{1,5}".prop_map(|name| AstStructure {
                kind: AstKind::Function(name),
                children: Vec::new(),
                depth: 1,
            }),
            "[a-z]{1,5}".prop_map(|name| AstStructure {
                kind: AstKind::Struct(name),
                children: Vec::new(),
                depth: 1,
            }),
        ];
        
        leaf.prop_recursive(
            8,   // Maximum depth
            256, // Maximum size
            10,  // Items per level
            |inner| {
                prop_oneof![
                    (inner.clone(), inner.clone()).prop_map(|(a, b)| {
                        AstStructure {
                            kind: AstKind::Sequence,
                            children: vec![Box::new(a), Box::new(b)],
                            depth: 2,
                        }
                    }),
                    ("[a-z]{1,5}", prop::collection::vec(inner, 0..3)).prop_map(|(name, children)| {
                        let max_depth = children.iter().map(|c| c.depth).max().unwrap_or(0);
                        AstStructure {
                            kind: AstKind::Module(name),
                            children: children.into_iter().map(Box::new).collect(),
                            depth: max_depth + 1,
                        }
                    }),
                ]
            }
        ).boxed()
    }
}

/// Shrinker for file paths
pub fn shrink_path(path: &str) -> Vec<String> {
    let mut shrunken = Vec::new();
    
    // Try removing directories
    if path.contains('/') {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 1 {
            // Just the filename
            shrunken.push(parts.last().unwrap().to_string());
            
            // Remove intermediate directories
            for i in 1..parts.len() - 1 {
                let mut new_parts = parts.clone();
                new_parts.remove(i);
                shrunken.push(new_parts.join("/"));
            }
        }
    }
    
    // Try simplifying the filename
    if path.len() > 5 {
        shrunken.push("a.rs".to_string());
        shrunken.push("test.rs".to_string());
    }
    
    shrunken
}

/// Shrinker for collections that maintains invariants
pub fn shrink_collection_maintaining<T, F>(
    collection: &[T],
    invariant: F,
) -> Vec<Vec<T>>
where
    T: Clone,
    F: Fn(&[T]) -> bool,
{
    let mut shrunken = Vec::new();
    
    // Try removing each element
    for i in 0..collection.len() {
        let mut new_collection = collection.to_vec();
        new_collection.remove(i);
        
        // Only include if invariant is maintained
        if invariant(&new_collection) {
            shrunken.push(new_collection);
        }
    }
    
    // Try removing pairs
    if collection.len() > 2 {
        for i in 0..collection.len() - 1 {
            let mut new_collection = collection.to_vec();
            new_collection.remove(i + 1);
            new_collection.remove(i);
            
            if invariant(&new_collection) {
                shrunken.push(new_collection);
            }
        }
    }
    
    // Try keeping only first/last half
    if collection.len() > 1 {
        let mid = collection.len() / 2;
        
        let first_half = collection[..mid].to_vec();
        if invariant(&first_half) {
            shrunken.push(first_half);
        }
        
        let second_half = collection[mid..].to_vec();
        if invariant(&second_half) {
            shrunken.push(second_half);
        }
    }
    
    shrunken
}

/// Binary search shrinker for numeric values
pub fn binary_shrink_number(value: i64, min: i64, check: impl Fn(i64) -> bool) -> i64 {
    let mut low = min;
    let mut high = value;
    
    while low < high {
        let mid = low + (high - low) / 2;
        
        if check(mid) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    
    low
}

/// Shrinker for error messages
pub fn shrink_error_message(msg: &str) -> Vec<String> {
    let mut shrunken = Vec::new();
    
    // Try standard error messages
    shrunken.push("Error".to_string());
    shrunken.push("Failed".to_string());
    
    // Try removing words
    let words: Vec<&str> = msg.split_whitespace().collect();
    if words.len() > 1 {
        // First word only
        shrunken.push(words[0].to_string());
        
        // Last word only
        shrunken.push(words[words.len() - 1].to_string());
        
        // Remove each word
        for i in 0..words.len() {
            let mut new_words = words.clone();
            new_words.remove(i);
            if !new_words.is_empty() {
                shrunken.push(new_words.join(" "));
            }
        }
    }
    
    // Try truncating
    if msg.len() > 10 {
        shrunken.push(msg[..msg.len() / 2].to_string());
        shrunken.push(msg[..10].to_string());
    }
    
    shrunken
}

/// Strategy combinator for conditional shrinking
pub struct ConditionalShrinker<T> {
    value: T,
    condition: Box<dyn Fn(&T) -> bool>,
}

impl<T: Clone> ConditionalShrinker<T> {
    pub fn new(value: T, condition: impl Fn(&T) -> bool + 'static) -> Self {
        Self {
            value,
            condition: Box::new(condition),
        }
    }
    
    pub fn shrink_if<F>(&self, shrinker: F) -> Vec<T>
    where
        F: Fn(&T) -> Vec<T>,
    {
        shrinker(&self.value)
            .into_iter()
            .filter(|v| (self.condition)(v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ast_structure_shrinking() {
        let ast = AstStructure {
            kind: AstKind::Module("test_module".to_string()),
            children: vec![
                Box::new(AstStructure {
                    kind: AstKind::Function("func1".to_string()),
                    children: Vec::new(),
                    depth: 1,
                }),
                Box::new(AstStructure {
                    kind: AstKind::Function("func2".to_string()),
                    children: Vec::new(),
                    depth: 1,
                }),
            ],
            depth: 2,
        };
        
        let shrunken = ast.shrink_remove();
        assert!(!shrunken.is_empty());
        
        // Should include versions with each child removed
        assert!(shrunken.iter().any(|s| s.children.len() == 1));
        
        // Should include empty version
        assert!(shrunken.iter().any(|s| matches!(s.kind, AstKind::Empty)));
    }
    
    #[test]
    fn test_path_shrinking() {
        let path = "src/module/submodule/file.rs";
        let shrunken = shrink_path(path);
        
        assert!(shrunken.contains(&"file.rs".to_string()));
        assert!(shrunken.contains(&"a.rs".to_string()));
    }
    
    #[test]
    fn test_collection_shrinking_with_invariant() {
        let collection = vec![1, 2, 3, 4, 5];
        
        // Invariant: sum must be even
        let invariant = |v: &[i32]| -> bool {
            v.iter().sum::<i32>() % 2 == 0
        };
        
        let shrunken = shrink_collection_maintaining(&collection, invariant);
        
        // All shrunken versions should maintain the invariant
        for s in &shrunken {
            assert!(invariant(s));
        }
    }
    
    #[test]
    fn test_binary_shrink() {
        // Find minimum value where x^2 >= 100
        let check = |x: i64| x * x >= 100;
        let result = binary_shrink_number(20, 0, check);
        
        assert_eq!(result, 10); // 10^2 = 100
    }
    
    #[test]
    fn test_error_message_shrinking() {
        let msg = "Failed to process the input file";
        let shrunken = shrink_error_message(msg);
        
        assert!(shrunken.contains(&"Error".to_string()));
        assert!(shrunken.contains(&"Failed".to_string()));
        assert!(shrunken.contains(&"Failed to process".to_string()));
    }
    
    #[test]
    fn test_conditional_shrinker() {
        let value = vec![1, 2, 3, 4, 5];
        
        // Condition: must have at least 2 elements
        let shrinker = ConditionalShrinker::new(value.clone(), |v: &Vec<i32>| v.len() >= 2);
        
        let shrunken = shrinker.shrink_if(|v| {
            let mut results = Vec::new();
            for i in 0..v.len() {
                let mut new_v = v.clone();
                new_v.remove(i);
                results.push(new_v);
            }
            results
        });
        
        // All shrunken versions should have at least 2 elements
        for s in shrunken {
            assert!(s.len() >= 2);
        }
    }
}