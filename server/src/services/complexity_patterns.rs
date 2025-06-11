//! Complexity Pattern Recognition - Phase 5 Day 13
//!
//! Pattern-based algorithmic complexity detection for common code patterns
//! including loops, recursion, and data structure operations.

use crate::models::complexity_bound::{
    ComplexityBound, ComplexityFlags, RecurrenceRelation, RecursiveCall,
};
use crate::models::unified_ast::{AstKind, ExprKind, StmtKind, UnifiedAstNode};
use std::collections::HashMap;
use tracing::debug;

/// Pattern matcher for algorithmic complexity analysis
pub struct ComplexityPatternMatcher {
    patterns: HashMap<String, ComplexityPattern>,
}

/// A complexity pattern that can be matched against AST nodes
#[derive(Debug, Clone)]
pub struct ComplexityPattern {
    pub name: String,
    pub description: String,
    pub complexity: ComplexityBound,
    pub pattern_type: PatternType,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    /// Single loop over collection
    LinearIteration,
    /// Nested loops
    NestedLoops { depth: u32 },
    /// Binary search pattern
    BinarySearch,
    /// Divide and conquer
    DivideAndConquer { divisions: u32 },
    /// Dynamic programming
    DynamicProgramming,
    /// Recursive pattern
    Recursive(RecurrenceRelation),
    /// Hash table operations
    HashTableOp,
    /// Tree traversal
    TreeTraversal,
    /// Graph algorithm
    GraphAlgorithm { vertices: bool, edges: bool },
}

impl ComplexityPatternMatcher {
    /// Create a new pattern matcher with built-in patterns
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Linear patterns
        patterns.insert(
            "linear_iteration".to_string(),
            ComplexityPattern {
                name: "Linear Iteration".to_string(),
                description: "Single loop over collection".to_string(),
                complexity: ComplexityBound::linear()
                    .with_confidence(90)
                    .with_flags(ComplexityFlags::WORST_CASE),
                pattern_type: PatternType::LinearIteration,
            },
        );

        // Quadratic patterns
        patterns.insert(
            "nested_loops_2".to_string(),
            ComplexityPattern {
                name: "Nested Loops (Depth 2)".to_string(),
                description: "Two nested loops over same collection".to_string(),
                complexity: ComplexityBound::quadratic()
                    .with_confidence(85)
                    .with_flags(ComplexityFlags::WORST_CASE),
                pattern_type: PatternType::NestedLoops { depth: 2 },
            },
        );

        // Logarithmic patterns
        patterns.insert(
            "binary_search".to_string(),
            ComplexityPattern {
                name: "Binary Search".to_string(),
                description: "Divide search space by 2 each iteration".to_string(),
                complexity: ComplexityBound::logarithmic()
                    .with_confidence(95)
                    .with_flags(ComplexityFlags::WORST_CASE | ComplexityFlags::PROVEN),
                pattern_type: PatternType::BinarySearch,
            },
        );

        // Linearithmic patterns
        patterns.insert(
            "merge_sort".to_string(),
            ComplexityPattern {
                name: "Merge Sort".to_string(),
                description: "Divide and conquer with linear merge".to_string(),
                complexity: ComplexityBound::linearithmic()
                    .with_confidence(95)
                    .with_flags(ComplexityFlags::WORST_CASE | ComplexityFlags::PROVEN),
                pattern_type: PatternType::DivideAndConquer { divisions: 2 },
            },
        );

        // Hash table operations
        patterns.insert(
            "hash_lookup".to_string(),
            ComplexityPattern {
                name: "Hash Table Lookup".to_string(),
                description: "Average case constant time lookup".to_string(),
                complexity: ComplexityBound::constant()
                    .with_confidence(80)
                    .with_flags(ComplexityFlags::AVERAGE_CASE | ComplexityFlags::AMORTIZED),
                pattern_type: PatternType::HashTableOp,
            },
        );

        Self { patterns }
    }

    /// Match AST node against known patterns
    pub fn match_pattern(&self, ast: &UnifiedAstNode) -> Option<&ComplexityPattern> {
        // Try each pattern
        for pattern in self.patterns.values() {
            if self.matches_pattern(ast, pattern) {
                debug!("Matched pattern: {} for AST node", pattern.name);
                return Some(pattern);
            }
        }
        None
    }

    /// Check if AST matches a specific pattern
    fn matches_pattern(&self, ast: &UnifiedAstNode, pattern: &ComplexityPattern) -> bool {
        match &pattern.pattern_type {
            PatternType::LinearIteration => self.is_linear_iteration(ast),
            PatternType::NestedLoops { depth } => self.is_nested_loops(ast, *depth),
            PatternType::BinarySearch => self.is_binary_search(ast),
            PatternType::DivideAndConquer { divisions } => {
                self.is_divide_and_conquer(ast, *divisions)
            }
            PatternType::HashTableOp => self.is_hash_operation(ast),
            _ => false, // Other patterns need more sophisticated analysis
        }
    }

    /// Detect single loop pattern
    fn is_linear_iteration(&self, ast: &UnifiedAstNode) -> bool {
        match &ast.kind {
            AstKind::Statement(stmt) => {
                // Check for loop statements
                matches!(
                    stmt,
                    StmtKind::For | StmtKind::While | StmtKind::DoWhile | StmtKind::ForEach
                )
            }
            AstKind::Expression(expr) => {
                // Check for iterator methods (would need more context to determine)
                matches!(
                    expr,
                    ExprKind::Call // Could be an iterator method call
                )
            }
            _ => false,
        }
    }

    /// Detect nested loops
    fn is_nested_loops(&self, ast: &UnifiedAstNode, target_depth: u32) -> bool {
        self.count_loop_depth(ast) >= target_depth
    }

    /// Count maximum loop nesting depth
    fn count_loop_depth(&self, ast: &UnifiedAstNode) -> u32 {
        let is_loop = self.is_linear_iteration(ast);
        let mut max_child_depth = 0;

        // Check children for nested loops
        // TRACKED: Need actual child traversal once AST structure is complete
        // For now, return conservative estimate based on AST type
        if ast.first_child != 0 {
            max_child_depth = 1;
        }

        if is_loop {
            1 + max_child_depth
        } else {
            max_child_depth
        }
    }

    /// Detect binary search pattern
    fn is_binary_search(&self, ast: &UnifiedAstNode) -> bool {
        // Look for characteristic patterns:
        // 1. Loop with low/high/mid variables
        // 2. Division by 2 or shift operation
        // 3. Comparison and bounds adjustment

        // Simple heuristic for now
        if let AstKind::Function(_) = &ast.kind {
            // Check function name from node
            if let Some(name) = self.get_function_name(ast) {
                let lower_name = name.to_lowercase();
                return lower_name.contains("binary") && lower_name.contains("search");
            }
        }
        false
    }

    /// Detect divide and conquer pattern
    fn is_divide_and_conquer(&self, ast: &UnifiedAstNode, _expected_divisions: u32) -> bool {
        // Look for recursive calls with input division
        if let AstKind::Function(_) = &ast.kind {
            // TRACKED: Analyze function body for recursive calls
            // For now, use name-based heuristic
            if let Some(name) = self.get_function_name(ast) {
                let lower_name = name.to_lowercase();
                return (lower_name.contains("merge") && lower_name.contains("sort"))
                    || (lower_name.contains("quick") && lower_name.contains("sort"));
            }
        }
        false
    }

    /// Detect hash table operations
    fn is_hash_operation(&self, ast: &UnifiedAstNode) -> bool {
        match &ast.kind {
            AstKind::Expression(expr) => {
                // For hash operations, we'd need more context
                // For now, check if it's a member access or call
                matches!(expr, ExprKind::Member | ExprKind::Call)
            }
            _ => false,
        }
    }

    /// Extract function name from AST node
    fn get_function_name(&self, ast: &UnifiedAstNode) -> Option<String> {
        match &ast.kind {
            AstKind::Function(_) => {
                // Extract name from AST node metadata or name field
                // For now, return None - would need actual name from node
                None
            }
            _ => None,
        }
    }

    /// Analyze recursive patterns
    #[inline]
    pub fn analyze_recursion(&self, ast: &UnifiedAstNode) -> Option<RecurrenceRelation> {
        if !self.is_recursive_function(ast) {
            return None;
        }

        // Analyze recursive calls
        let recursive_calls = self.find_recursive_calls(ast);
        let work_complexity = self.estimate_non_recursive_work(ast);

        if recursive_calls.is_empty() {
            return None;
        }

        Some(RecurrenceRelation {
            recursive_calls,
            work_per_call: work_complexity,
            base_case_size: 1, // Default assumption
        })
    }

    /// Check if function is recursive
    fn is_recursive_function(&self, ast: &UnifiedAstNode) -> bool {
        // Simple check: function that calls itself
        if let AstKind::Function(_) = &ast.kind {
            // TRACKED: Check function body for self-calls
            // Would need to get function name and check for recursive calls
            return false; // Placeholder
        }
        false
    }

    /// Find recursive call patterns
    fn find_recursive_calls(&self, _ast: &UnifiedAstNode) -> Vec<RecursiveCall> {
        Vec::new() // TRACKED: Implement AST traversal
    }

    /// Estimate non-recursive work in function
    fn estimate_non_recursive_work(&self, _ast: &UnifiedAstNode) -> ComplexityBound {
        // Count operations excluding recursive calls
        ComplexityBound::linear() // Placeholder
    }

    #[inline]
    /// Analyze loop complexity
    pub fn analyze_loop_complexity(&self, ast: &UnifiedAstNode) -> ComplexityBound {
        let loop_depth = self.count_loop_depth(ast);

        match loop_depth {
            0 => ComplexityBound::constant(),
            1 => ComplexityBound::linear(),
            2 => ComplexityBound::quadratic(),
            3 => ComplexityBound::polynomial(3, 1),
            _ => ComplexityBound::polynomial(loop_depth, 1),
        }
        .with_confidence(80 - (loop_depth * 10).min(50) as u8)
    }

    /// Add custom pattern
    pub fn add_pattern(&mut self, id: String, pattern: ComplexityPattern) {
        self.patterns.insert(id, pattern);
    }

    /// Get all registered patterns
    pub fn patterns(&self) -> &HashMap<String, ComplexityPattern> {
        &self.patterns
    }
}

impl Default for ComplexityPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Complexity analysis result
#[derive(Debug)]
pub struct ComplexityAnalysisResult {
    pub time_complexity: ComplexityBound,
    pub space_complexity: ComplexityBound,
    pub matched_patterns: Vec<String>,
    pub confidence: u8,
    pub notes: Vec<String>,
}

impl ComplexityAnalysisResult {
    pub fn new(time: ComplexityBound, space: ComplexityBound) -> Self {
        Self {
            time_complexity: time,
            space_complexity: space,
            matched_patterns: Vec::new(),
            confidence: (time.confidence + space.confidence) / 2,
            notes: Vec::new(),
        }
    }

    pub fn with_pattern(mut self, pattern_name: String) -> Self {
        self.matched_patterns.push(pattern_name);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::complexity_bound::BigOClass;

    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = ComplexityPatternMatcher::new();
        assert!(matcher.patterns.contains_key("linear_iteration"));
        assert!(matcher.patterns.contains_key("binary_search"));
        assert!(matcher.patterns.contains_key("merge_sort"));
    }

    #[test]
    fn test_complexity_bound_properties() {
        let pattern = &ComplexityPatternMatcher::new().patterns["binary_search"];
        assert_eq!(pattern.complexity.class, BigOClass::Logarithmic);
        assert!(pattern.complexity.confidence >= 90);
        assert!(pattern.complexity.flags.is_worst_case());
    }

    #[test]
    fn test_loop_complexity_analysis() {
        let matcher = ComplexityPatternMatcher::new();

        // Mock AST node
        let ast = UnifiedAstNode {
            kind: AstKind::Statement(StmtKind::For),
            lang: crate::models::unified_ast::Language::JavaScript,
            first_child: 0,
            next_sibling: 0,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            parent: 0,
            source_range: 0..10,
            flags: Default::default(),
            metadata: Default::default(),
            proof_annotations: None,
        };

        let result = matcher.analyze_loop_complexity(&ast);
        assert_eq!(result.class, BigOClass::Linear);
    }
}
