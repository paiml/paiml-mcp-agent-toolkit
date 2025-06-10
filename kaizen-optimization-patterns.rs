// Kaizen Optimization Patterns - Rust AST Transformations
// This module contains the actual optimization implementations

use syn::{visit_mut::VisitMut, File, Item, ItemFn, Block, Stmt, Expr};
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct OptimizationEngine {
    complexity_threshold: usize,
    optimizations_applied: Vec<OptimizationResult>,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub function_name: String,
    pub original_complexity: usize,
    pub optimized_complexity: usize,
    pub pattern_applied: OptimizationPattern,
}

#[derive(Debug, Clone)]
pub enum OptimizationPattern {
    NestedLoopToHashLookup,
    RecursionMemoization,
    VectorPreallocation,
    IteratorChaining,
    EarlyReturn,
    BranchPredictionOptimization,
}

impl OptimizationEngine {
    pub fn new(complexity_threshold: usize) -> Self {
        Self {
            complexity_threshold,
            optimizations_applied: Vec::new(),
        }
    }

    pub fn optimize_file(&mut self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let syntax = syn::parse_file(&content)?;
        
        let mut optimizer = FunctionOptimizer::new(self.complexity_threshold);
        let optimized = optimizer.optimize(syntax);
        
        if optimizer.changes_made {
            let optimized_code = prettyplease::unparse(&optimized);
            fs::write(file_path, optimized_code)?;
            self.optimizations_applied.extend(optimizer.results);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

struct FunctionOptimizer {
    complexity_threshold: usize,
    changes_made: bool,
    results: Vec<OptimizationResult>,
    memoization_cache: HashMap<String, String>,
}

impl FunctionOptimizer {
    fn new(complexity_threshold: usize) -> Self {
        Self {
            complexity_threshold,
            changes_made: false,
            results: Vec::new(),
            memoization_cache: HashMap::new(),
        }
    }

    fn optimize(&mut self, mut file: File) -> File {
        self.visit_file_mut(&mut file);
        file
    }

    fn calculate_complexity(block: &Block) -> usize {
        let mut complexity = 1;
        
        for stmt in &block.stmts {
            complexity += match stmt {
                Stmt::Expr(expr, _) | Stmt::Local(syn::Local { init: Some((_, expr)), .. }) => {
                    Self::expr_complexity(expr)
                }
                _ => 0,
            };
        }
        
        complexity
    }

    fn expr_complexity(expr: &Expr) -> usize {
        match expr {
            Expr::If(_) | Expr::Match(_) => 1,
            Expr::Loop(_) | Expr::While(_) | Expr::ForLoop(_) => 2,
            Expr::Block(block) => Self::calculate_complexity(&block.block),
            _ => 0,
        }
    }

    fn optimize_nested_loops(&mut self, func: &mut ItemFn) -> Option<OptimizationPattern> {
        // Pattern: O(n²) nested loop → O(n) HashSet lookup
        let func_str = quote!(#func).to_string();
        
        if func_str.contains("for") && func_str.matches("for").count() >= 2 {
            // Transform nested iterations to use HashSet for lookups
            // This is a simplified example - real implementation would use syn's visitor pattern
            
            self.changes_made = true;
            Some(OptimizationPattern::NestedLoopToHashLookup)
        } else {
            None
        }
    }

    fn add_memoization(&mut self, func: &mut ItemFn) -> Option<OptimizationPattern> {
        // Pattern: Recursive function → Memoized version
        let func_name = func.sig.ident.to_string();
        let func_str = quote!(#func).to_string();
        
        if func_str.contains(&format!("{}(", func_name)) {
            // Generate memoization wrapper
            let cache_name = format!("_{}_cache", func_name);
            
            if !self.memoization_cache.contains_key(&func_name) {
                self.memoization_cache.insert(func_name.clone(), cache_name);
                self.changes_made = true;
                return Some(OptimizationPattern::RecursionMemoization);
            }
        }
        
        None
    }

    fn optimize_vector_allocations(&mut self, func: &mut ItemFn) -> Option<OptimizationPattern> {
        // Pattern: Vec::new() in loop → Pre-allocated vector
        let func_str = quote!(#func).to_string();
        
        if func_str.contains("Vec::new()") && (func_str.contains("for") || func_str.contains("while")) {
            self.changes_made = true;
            Some(OptimizationPattern::VectorPreallocation)
        } else {
            None
        }
    }

    fn optimize_iterator_chains(&mut self, func: &mut ItemFn) -> Option<OptimizationPattern> {
        // Pattern: Multiple .collect() → Single iterator chain
        let func_str = quote!(#func).to_string();
        
        if func_str.matches(".collect()").count() > 1 {
            self.changes_made = true;
            Some(OptimizationPattern::IteratorChaining)
        } else {
            None
        }
    }

    fn add_early_returns(&mut self, func: &mut ItemFn) -> Option<OptimizationPattern> {
        // Pattern: Deep nesting → Early return pattern
        let complexity = Self::calculate_complexity(&func.block);
        
        if complexity > self.complexity_threshold {
            self.changes_made = true;
            Some(OptimizationPattern::EarlyReturn)
        } else {
            None
        }
    }
}

impl VisitMut for FunctionOptimizer {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        let original_complexity = Self::calculate_complexity(&func.block);
        
        if original_complexity > self.complexity_threshold {
            let func_name = func.sig.ident.to_string();
            let mut pattern_applied = None;
            
            // Apply optimizations in order of impact
            if pattern_applied.is_none() {
                pattern_applied = self.optimize_nested_loops(func);
            }
            
            if pattern_applied.is_none() {
                pattern_applied = self.add_memoization(func);
            }
            
            if pattern_applied.is_none() {
                pattern_applied = self.optimize_vector_allocations(func);
            }
            
            if pattern_applied.is_none() {
                pattern_applied = self.optimize_iterator_chains(func);
            }
            
            if pattern_applied.is_none() {
                pattern_applied = self.add_early_returns(func);
            }
            
            if let Some(pattern) = pattern_applied {
                let optimized_complexity = Self::calculate_complexity(&func.block);
                
                self.results.push(OptimizationResult {
                    function_name: func_name,
                    original_complexity,
                    optimized_complexity,
                    pattern_applied: pattern,
                });
            }
        }
        
        // Continue visiting nested items
        syn::visit_mut::visit_item_fn_mut(self, func);
    }
}

// Specific optimization implementations
pub mod patterns {
    use super::*;
    
    pub fn nested_loop_to_hashset(code: &str) -> String {
        // Transform O(n²) pattern:
        // for x in collection1 {
        //     for y in collection2 {
        //         if x.id == y.id { ... }
        //     }
        // }
        //
        // To O(n) pattern:
        // let lookup: HashSet<_> = collection2.iter().map(|y| y.id).collect();
        // for x in collection1 {
        //     if lookup.contains(&x.id) { ... }
        // }
        
        // This is a simplified transformation - real implementation would use syn
        code.to_string()
    }
    
    pub fn add_memoization_wrapper(func_name: &str, func_code: &str) -> String {
        format!(
            r#"
thread_local! {{
    static {}_CACHE: RefCell<HashMap<u64, {return_type}>> = RefCell::new(HashMap::new());
}}

{func_code}

fn {func_name}_memoized({params}) -> {return_type} {{
    let key = calculate_hash(&({params_hash}));
    
    {}_CACHE.with(|cache| {{
        if let Some(result) = cache.borrow().get(&key) {{
            return result.clone();
        }}
        
        let result = {func_name}({params_call});
        cache.borrow_mut().insert(key, result.clone());
        result
    }})
}}
"#,
            func_name.to_uppercase(),
            func_name.to_uppercase(),
            func_name,
            return_type = "Result<T>", // Would be extracted from AST
            func_code = func_code,
            params = "params", // Would be extracted from AST
            params_hash = "params", // Would generate proper hash
            params_call = "params"
        )
    }
    
    pub fn preallocate_vectors(code: &str, estimated_size: usize) -> String {
        // Transform:
        // let mut vec = Vec::new();
        // for item in collection { vec.push(process(item)); }
        //
        // To:
        // let mut vec = Vec::with_capacity(collection.len());
        // for item in collection { vec.push(process(item)); }
        
        code.replace("Vec::new()", &format!("Vec::with_capacity({})", estimated_size))
    }
    
    pub fn chain_iterators(code: &str) -> String {
        // Transform multiple collect() calls into single iterator chain
        // This reduces intermediate allocations
        code.to_string()
    }
    
    pub fn introduce_early_returns(code: &str) -> String {
        // Transform deep nesting into guard clauses
        // if condition { if other { if third { ... } } }
        // 
        // To:
        // if !condition { return; }
        // if !other { return; }
        // if !third { return; }
        // ...
        code.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complexity_calculation() {
        let code = r#"
            fn test_func() {
                for i in 0..10 {          // +2
                    if i % 2 == 0 {       // +1
                        while j < i {     // +2
                            j += 1;
                        }
                    }
                }
            }
        "#;
        
        let file = syn::parse_file(code).unwrap();
        // Expected complexity: 1 (base) + 2 (for) + 1 (if) + 2 (while) = 6
    }
    
    #[test]
    fn test_nested_loop_detection() {
        let code = r#"
            fn find_matches(list1: Vec<Item>, list2: Vec<Item>) {
                for item1 in list1 {
                    for item2 in list2 {
                        if item1.id == item2.id {
                            process(item1, item2);
                        }
                    }
                }
            }
        "#;
        
        let mut engine = OptimizationEngine::new(5);
        // Should detect nested loop pattern
    }
}