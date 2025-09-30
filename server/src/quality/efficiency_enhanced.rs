use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use syn::{self, visit::Visit, Expr, Stmt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Complexity {
    O1,         // O(1) - Constant
    OLogN,      // O(log n) - Logarithmic
    ON,         // O(n) - Linear
    ONLogN,     // O(n log n) - Linearithmic
    ON2,        // O(n²) - Quadratic
    ON3,        // O(n³) - Cubic
    OExp,       // O(2^n) - Exponential
    OFactorial, // O(n!) - Factorial
}

impl Display for Complexity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::O1 => write!(f, "O(1)"),
            Complexity::OLogN => write!(f, "O(log n)"),
            Complexity::ON => write!(f, "O(n)"),
            Complexity::ONLogN => write!(f, "O(n log n)"),
            Complexity::ON2 => write!(f, "O(n^2)"),
            Complexity::ON3 => write!(f, "O(n^3)"),
            Complexity::OExp => write!(f, "O(2^n)"),
            Complexity::OFactorial => write!(f, "O(n!)"),
        }
    }
}

impl Complexity {
    pub fn combine(&self, other: &Complexity) -> Complexity {
        // When combining complexities (e.g., nested loops), multiply
        use Complexity::*;
        match (self, other) {
            (O1, x) | (x, O1) => x.clone(),
            (OLogN, OLogN) => ON, // log n * log n ≈ O(n) for practical purposes
            (OLogN, ON) | (ON, OLogN) => ONLogN,
            (ON, ON) => ON2,
            (ON, ON2) | (ON2, ON) => ON3,
            (ON2, ON2) => ON3, // Simplified - could be O(n^4)
            _ => OExp,         // Conservative estimate for complex combinations
        }
    }

    pub fn max(&self, other: &Complexity) -> Complexity {
        if self > other {
            self.clone()
        } else {
            other.clone()
        }
    }
}

pub struct SymbolicExecutor {
    loop_depths: Vec<Complexity>,
    recursive_depth: usize,
    function_complexities: HashMap<String, Complexity>,
    current_path_complexity: Complexity,
}

impl Default for SymbolicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolicExecutor {
    pub fn new() -> Self {
        Self {
            loop_depths: Vec::new(),
            recursive_depth: 0,
            function_complexities: HashMap::new(),
            current_path_complexity: Complexity::O1,
        }
    }

    pub fn analyze_function(&mut self, func: &syn::ItemFn) -> Complexity {
        let name = func.sig.ident.to_string();

        // Reset state for new function
        self.loop_depths.clear();
        self.current_path_complexity = Complexity::O1;

        // Check for recursion
        let mut recursion_detector = RecursionDetector {
            function_name: name.clone(),
            is_recursive: false,
        };
        recursion_detector.visit_block(&func.block);

        if recursion_detector.is_recursive {
            self.recursive_depth = 1;
        }

        // Analyze function body
        self.visit_block(&func.block);

        // Handle recursive complexity
        let mut complexity = self.current_path_complexity.clone();
        if self.recursive_depth > 0 {
            complexity = match complexity {
                Complexity::O1 => Complexity::ON,  // Simple recursion
                Complexity::ON => Complexity::ON2, // Recursive with linear work
                _ => Complexity::OExp,             // Complex recursion
            };
        }

        self.function_complexities.insert(name, complexity.clone());
        complexity
    }

    pub fn analyze_loop_pattern(&self, loop_expr: &syn::ExprForLoop) -> Complexity {
        // Analyze loop bounds to determine complexity
        if let syn::Expr::Range(range) = &*loop_expr.expr {
            return self.analyze_range_complexity(range);
        }

        // Check for common patterns
        if self.is_iterator_pattern(&loop_expr.expr) {
            return Complexity::ON;
        }

        // Conservative estimate
        Complexity::ON
    }

    fn analyze_range_complexity(&self, range: &syn::ExprRange) -> Complexity {
        // Check for logarithmic patterns (e.g., i *= 2)
        if self.is_logarithmic_range(range) {
            return Complexity::OLogN;
        }

        Complexity::ON
    }

    fn is_logarithmic_range(&self, _range: &syn::ExprRange) -> bool {
        // Simplified - would need more sophisticated analysis
        false
    }

    fn is_iterator_pattern(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(call) => {
                let method = call.method.to_string();
                matches!(method.as_str(), "iter" | "into_iter" | "iter_mut")
            }
            _ => false,
        }
    }

    pub fn analyze_algorithm_patterns(&self, ast: &syn::File) -> Vec<AlgorithmPattern> {
        let mut patterns = Vec::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                if self.is_sorting_algorithm(func) {
                    patterns.push(AlgorithmPattern::Sorting);
                }
                if self.is_search_algorithm(func) {
                    patterns.push(AlgorithmPattern::Search);
                }
                if self.is_graph_algorithm(func) {
                    patterns.push(AlgorithmPattern::Graph);
                }
                if self.is_dynamic_programming(func) {
                    patterns.push(AlgorithmPattern::DynamicProgramming);
                }
            }
        }

        patterns
    }

    fn is_sorting_algorithm(&self, func: &syn::ItemFn) -> bool {
        let name = func.sig.ident.to_string();
        name.contains("sort") || name.contains("heap") || name.contains("quick")
    }

    fn is_search_algorithm(&self, func: &syn::ItemFn) -> bool {
        let name = func.sig.ident.to_string();
        name.contains("search") || name.contains("find") || name.contains("binary")
    }

    fn is_graph_algorithm(&self, func: &syn::ItemFn) -> bool {
        let name = func.sig.ident.to_string();
        name.contains("dfs") || name.contains("bfs") || name.contains("dijkstra")
    }

    fn is_dynamic_programming(&self, func: &syn::ItemFn) -> bool {
        // Check for memoization patterns
        let has_cache = false;
        let mut _has_recursion = false;

        for stmt in &func.block.stmts {
            if let Stmt::Local(local) = stmt {
                if let Some(_init) = &local.init {
                    // TODO: Fix quote macro usage with LocalInit
                    // let code = quote::quote!(#init).to_string();
                    // if code.contains("HashMap") || code.contains("cache") || code.contains("memo") {
                    //     has_cache = true;
                    // }
                }
            }
        }

        // Simplified check - would need deeper analysis
        has_cache
    }
}

impl<'ast> Visit<'ast> for SymbolicExecutor {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        let loop_complexity = self.analyze_loop_pattern(node);

        // Push current loop complexity
        self.loop_depths.push(loop_complexity.clone());

        // Update path complexity
        if self.loop_depths.len() == 1 {
            self.current_path_complexity =
                self.current_path_complexity.clone().max(loop_complexity);
        } else {
            // Nested loops multiply complexity
            let nested = self
                .loop_depths
                .iter()
                .fold(Complexity::O1, |acc, c| acc.combine(c));
            self.current_path_complexity = self.current_path_complexity.clone().max(nested);
        }

        // Visit loop body
        syn::visit::visit_expr_for_loop(self, node);

        // Pop loop depth
        self.loop_depths.pop();
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.loop_depths.push(Complexity::ON);

        let nested = self
            .loop_depths
            .iter()
            .fold(Complexity::O1, |acc, c| acc.combine(c));
        self.current_path_complexity = self.current_path_complexity.clone().max(nested);

        syn::visit::visit_expr_while(self, node);
        self.loop_depths.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Check for known complex operations
        if let Expr::Path(path) = &*node.func {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();

                // Known standard library complexities
                let complexity = match name.as_str() {
                    "sort" | "sort_by" => Complexity::ONLogN,
                    "binary_search" => Complexity::OLogN,
                    "contains" | "find" => Complexity::ON,
                    "reverse" => Complexity::ON,
                    _ => Complexity::O1,
                };

                self.current_path_complexity = self.current_path_complexity.clone().max(complexity);
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

struct RecursionDetector {
    function_name: String,
    is_recursive: bool,
}

impl<'ast> Visit<'ast> for RecursionDetector {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(path) = &*node.func {
            if let Some(ident) = path.path.get_ident() {
                if *ident == self.function_name {
                    self.is_recursive = true;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

#[derive(Debug, Clone)]
pub enum AlgorithmPattern {
    Sorting,
    Search,
    Graph,
    DynamicProgramming,
    Greedy,
    DivideAndConquer,
    Backtracking,
}

pub struct SpaceComplexityAnalyzer {
    allocations: Vec<Allocation>,
    max_depth: usize,
}

#[derive(Debug, Clone)]
struct Allocation {
    size: AllocationSize,
    _location: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum AllocationSize {
    Constant(usize),
    Linear,
    Quadratic,
    Dynamic,
}

impl Default for SpaceComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
            max_depth: 0,
        }
    }

    pub fn analyze(&mut self, ast: &syn::File) -> Complexity {
        self.allocations.clear();
        self.visit_file(ast);

        // Determine overall space complexity
        let has_recursive = self.max_depth > 1;
        let has_dynamic_allocation = self
            .allocations
            .iter()
            .any(|a| matches!(a.size, AllocationSize::Linear | AllocationSize::Quadratic));

        if has_recursive && has_dynamic_allocation {
            Complexity::ON2
        } else if has_recursive || has_dynamic_allocation {
            Complexity::ON
        } else {
            Complexity::O1
        }
    }
}

impl<'ast> Visit<'ast> for SpaceComplexityAnalyzer {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(_init) = &node.init {
            // TODO: Fix quote macro usage with LocalInit
            // let expr_str = quote::quote!(#init).to_string();

            // Check for vector/array allocations (simplified check)
            // TODO: Implement proper allocation detection without quote macro
            self.allocations.push(Allocation {
                size: AllocationSize::Dynamic,
                _location: "local".to_string(),
            });
        }

        syn::visit::visit_local(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_ordering() {
        assert!(Complexity::O1 < Complexity::OLogN);
        assert!(Complexity::OLogN < Complexity::ON);
        assert!(Complexity::ON < Complexity::ONLogN);
        assert!(Complexity::ONLogN < Complexity::ON2);
        assert!(Complexity::ON2 < Complexity::ON3);
        assert!(Complexity::ON3 < Complexity::OExp);
    }

    #[test]
    fn test_complexity_combination() {
        assert_eq!(Complexity::ON.combine(&Complexity::ON), Complexity::ON2);
        assert_eq!(Complexity::O1.combine(&Complexity::ON), Complexity::ON);
        assert_eq!(
            Complexity::ON.combine(&Complexity::OLogN),
            Complexity::ONLogN
        );
    }

    #[test]
    fn test_symbolic_execution() {
        let code = r#"
            fn bubble_sort(arr: &mut [i32]) {
                for i in 0..arr.len() {
                    for j in 0..arr.len() - 1 {
                        if arr[j] > arr[j + 1] {
                            arr.swap(j, j + 1);
                        }
                    }
                }
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                let complexity = executor.analyze_function(func);
                assert_eq!(complexity, Complexity::ON2);
            }
        }
    }
}
