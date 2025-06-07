# Big-O Notation Annotation Specification

## Executive Summary

This specification defines the implementation of algorithmic complexity annotations for the PAIML MCP Agent Toolkit's AST analysis engine. The system computes time and space complexity bounds during AST construction, storing them in cache-aligned structures for zero-overhead runtime queries. Expected analysis overhead: 10-15% during parsing, 16 bytes per function node.

## 1. Data Structures

### 1.1 Core Complexity Representation

```rust
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComplexityBound {
    pub class: BigOClass,           // 1 byte
    pub coefficient: u16,           // 2 bytes - worst-case constant factor
    pub input_var: InputVariable,   // 1 byte
    pub confidence: u8,             // 1 byte - 0-100%
    pub flags: ComplexityFlags,     // 1 byte
    _padding: [u8; 2],              // 2 bytes - maintain 8-byte alignment
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BigOClass {
    O1 = 0,          // O(1) - constant
    OLogN = 1,       // O(log n) - logarithmic
    ON = 2,          // O(n) - linear
    ONLogN = 3,      // O(n log n) - linearithmic
    ON2 = 4,         // O(n²) - quadratic
    ON3 = 5,         // O(n³) - cubic
    O2N = 6,         // O(2ⁿ) - exponential
    ONM = 7,         // O(n*m) - bivariate
    OSqrtN = 8,      // O(√n) - sublinear
    OCustom = 255,   // Complex expression stored separately
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputVariable {
    N = 0,           // Primary input size
    M = 1,           // Secondary input size
    K = 2,           // Tertiary input size
    LogN = 3,        // log(n)
    SqrtN = 4,       // √n
    NM = 5,          // n*m product
    Composite = 255, // Complex expression
}

bitflags! {
    #[repr(transparent)]
    pub struct ComplexityFlags: u8 {
        const AMORTIZED = 0b00000001;      // Amortized analysis
        const PROBABILISTIC = 0b00000010;  // Expected case
        const TIGHT_BOUND = 0b00000100;    // Θ notation
        const VECTORIZABLE = 0b00001000;   // SIMD-optimizable
        const CACHE_AWARE = 0b00010000;    // Cache-oblivious
        const RECURSIVE = 0b00100000;      // From recurrence
        const VERIFIED = 0b01000000;       // Formally verified
        const APPROXIMATE = 0b10000000;    // Heuristic bound
    }
}
```

### 1.2 Extended Metadata Integration

```rust
pub enum NodeMetadata {
    Function { 
        complexity: u16,
        param_count: u8,
        time_bound: ComplexityBound,      // 8 bytes
        space_bound: ComplexityBound,     // 8 bytes
        cache_complexity: CacheComplexity, // 8 bytes
    },
    Loop {
        iteration_count: LoopBound,       // 8 bytes
        body_complexity: ComplexityBound, // 8 bytes
        is_vectorizable: bool,
    },
    // ... other variants
}

#[repr(C)]
pub struct CacheComplexity {
    pub l1_misses: ComplexityBound,    // Per-iteration L1 miss rate
    pub l2_misses: ComplexityBound,    // Per-iteration L2 miss rate  
    pub tlb_misses: ComplexityBound,   // TLB miss complexity
}

#[repr(C)]
pub struct LoopBound {
    pub lower: u32,      // Minimum iterations (0 for unknown)
    pub upper: u32,      // Maximum iterations (u32::MAX for unbounded)
    pub typical: u32,    // Expected iterations
    pub stride: u16,     // Access pattern stride
    pub is_exact: bool,  // True if bounds are precise
    _padding: u8,
}
```

## 2. Analysis Algorithms

### 2.1 Pattern-Based Recognition

```rust
pub struct ComplexityAnalyzer {
    pattern_cache: DashMap<u64, ComplexityBound>,
    loop_analyzer: LoopComplexityAnalyzer,
    recursion_solver: RecurrenceSolver,
    dataflow_engine: DataflowAnalyzer,
}

impl ComplexityAnalyzer {
    pub fn analyze_node(&self, node: &UnifiedAstNode, ctx: &AnalysisContext) 
        -> (ComplexityBound, ComplexityBound) // (time, space)
    {
        // Fast path: check pattern cache
        let pattern_hash = self.compute_pattern_hash(node);
        if let Some(cached) = self.pattern_cache.get(&pattern_hash) {
            return (*cached, self.infer_space_from_time(*cached));
        }

        match node.kind {
            AstKind::Function(_) => self.analyze_function(node, ctx),
            AstKind::Loop(_) => self.analyze_loop(node, ctx),
            AstKind::Conditional(_) => self.analyze_branch(node, ctx),
            AstKind::Call(_) => self.analyze_call(node, ctx),
            _ => (ComplexityBound::constant(), ComplexityBound::constant()),
        }
    }

    fn analyze_loop(&self, node: &UnifiedAstNode, ctx: &AnalysisContext) 
        -> (ComplexityBound, ComplexityBound) 
    {
        let loop_info = self.loop_analyzer.extract_loop_info(node);
        let body_complexity = self.analyze_node(&node.body, ctx);
        
        match loop_info.pattern {
            LoopPattern::FixedCount(n) => {
                ComplexityBound::multiply_by_constant(body_complexity.0, n)
            },
            LoopPattern::LinearTraversal => {
                ComplexityBound::compose(BigOClass::ON, body_complexity.0)
            },
            LoopPattern::NestedLinear(depth) => {
                ComplexityBound::power(BigOClass::ON, depth)
            },
            LoopPattern::Logarithmic => {
                ComplexityBound::compose(BigOClass::OLogN, body_complexity.0)
            },
            LoopPattern::Unknown => {
                ComplexityBound::unknown()
            },
        }
    }
}
```

### 2.2 Recurrence Relation Solver

```rust
pub struct RecurrenceSolver {
    master_theorem: MasterTheoremSolver,
    substitution_method: SubstitutionSolver,
}

impl RecurrenceSolver {
    pub fn solve(&self, relation: &RecurrenceRelation) -> ComplexityBound {
        // Try Master Theorem first (fastest)
        if let Some(bound) = self.master_theorem.try_solve(relation) {
            return bound;
        }
        
        // Fall back to substitution method
        if let Some(bound) = self.substitution_method.try_solve(relation) {
            return bound;
        }
        
        // Heuristic for common patterns
        self.apply_heuristics(relation)
    }
}

#[derive(Debug, Clone)]
pub struct RecurrenceRelation {
    pub base_cases: Vec<(u32, ComplexityBound)>,
    pub recursive_calls: Vec<RecursiveCall>,
    pub non_recursive_work: ComplexityBound,
}

#[derive(Debug, Clone)]
pub struct RecursiveCall {
    pub size_reduction: SizeReduction,
    pub multiplicity: u32,
}

#[derive(Debug, Clone)]
pub enum SizeReduction {
    Divide(u32),      // T(n/k)
    Subtract(u32),    // T(n-k)
    SquareRoot,       // T(√n)
    Logarithm,        // T(log n)
}
```

### 2.3 Language-Specific Analyzers

```rust
// Rust-specific patterns
pub struct RustComplexityAnalyzer {
    iterator_chains: IteratorChainAnalyzer,
    trait_dispatch: TraitDispatchAnalyzer,
}

impl RustComplexityAnalyzer {
    fn analyze_iterator_chain(&self, chain: &[IteratorOp]) -> ComplexityBound {
        let mut total = ComplexityBound::constant();
        
        for op in chain {
            match op {
                IteratorOp::Map => {}, // O(1) per element
                IteratorOp::Filter => {}, // O(1) per element
                IteratorOp::FlatMap => {
                    total = total.multiply(ComplexityBound::linear());
                },
                IteratorOp::Sort => {
                    total = total.max(ComplexityBound::linearithmic());
                },
                IteratorOp::Collect(collection) => {
                    match collection {
                        CollectionType::Vec => {}, // O(n) allocation
                        CollectionType::HashMap => {
                            total = total.max(ComplexityBound::linear());
                        },
                        CollectionType::BTreeMap => {
                            total = total.max(ComplexityBound::linearithmic());
                        },
                    }
                },
            }
        }
        
        total
    }
}

// Python-specific patterns
pub struct PythonComplexityAnalyzer {
    comprehension_analyzer: ComprehensionAnalyzer,
    builtin_complexity: BuiltinComplexityDB,
}

// TypeScript-specific patterns
pub struct TypeScriptComplexityAnalyzer {
    promise_chain_analyzer: PromiseChainAnalyzer,
    array_method_db: ArrayMethodComplexityDB,
}

// C/C++ specific patterns
pub struct CppComplexityAnalyzer {
    stl_complexity: StlComplexityDB,
    pointer_analysis: PointerArithmeticAnalyzer,
}
```

## 3. Dataflow Analysis for Complex Cases

```rust
pub struct DataflowComplexityAnalyzer {
    lattice: ComplexityLattice,
    transfer_functions: TransferFunctionDB,
}

impl DataflowComplexityAnalyzer {
    pub fn analyze_cfg(&self, cfg: &ControlFlowGraph) -> ComplexityBound {
        let mut worklist = VecDeque::from_iter(cfg.nodes());
        let mut node_complexity = HashMap::new();
        
        // Initialize all nodes to bottom (O(1))
        for node in cfg.nodes() {
            node_complexity.insert(node.id, ComplexityBound::constant());
        }
        
        while let Some(node) = worklist.pop_front() {
            let old_complexity = node_complexity[&node.id];
            let new_complexity = self.compute_node_complexity(node, &node_complexity);
            
            if new_complexity != old_complexity {
                node_complexity.insert(node.id, new_complexity);
                // Add successors to worklist
                for succ in cfg.successors(node.id) {
                    worklist.push_back(succ);
                }
            }
        }
        
        // Extract worst-case path
        self.extract_worst_case_path(&cfg, &node_complexity)
    }
}
```

## 4. SIMD and Cache Analysis

```rust
pub struct SimdComplexityAnalyzer {
    vector_width: u32, // 8 for AVX2, 16 for AVX-512
    
    pub fn analyze_vectorization(&self, loop: &LoopNode) -> ComplexityBound {
        let base = loop.base_complexity;
        
        if loop.is_vectorizable() {
            // Adjust complexity by vector width
            ComplexityBound {
                class: base.class,
                coefficient: base.coefficient / self.vector_width as u16,
                input_var: base.input_var,
                confidence: base.confidence * 90 / 100, // Slight confidence reduction
                flags: base.flags | ComplexityFlags::VECTORIZABLE,
                ..base
            }
        } else {
            base
        }
    }
}

pub struct CacheComplexityAnalyzer {
    cache_line_size: u32, // Typically 64 bytes
    l1_size: u32,
    l2_size: u32,
    
    pub fn analyze_cache_behavior(&self, access_pattern: &AccessPattern) 
        -> CacheComplexity 
    {
        match access_pattern {
            AccessPattern::Sequential { stride } => {
                CacheComplexity {
                    l1_misses: ComplexityBound::linear_with_coefficient(
                        stride / self.cache_line_size
                    ),
                    l2_misses: ComplexityBound::constant(),
                    tlb_misses: ComplexityBound::constant(),
                }
            },
            AccessPattern::Random => {
                CacheComplexity {
                    l1_misses: ComplexityBound::linear(),
                    l2_misses: ComplexityBound::linear_with_coefficient(
                        self.l1_size / self.l2_size
                    ),
                    tlb_misses: ComplexityBound::logarithmic(),
                }
            },
            AccessPattern::Strided { major_stride, minor_stride } => {
                self.analyze_2d_access_pattern(major_stride, minor_stride)
            },
        }
    }
}
```

## 5. Verification and Confidence Scoring

```rust
pub struct ComplexityVerifier {
    smt_solver: Z3Context,
    empirical_validator: EmpiricalValidator,
    
    pub fn verify_bound(&self, 
        ast: &UnifiedAstNode, 
        claimed_bound: ComplexityBound
    ) -> VerificationResult {
        // First, try symbolic verification
        if let Some(proof) = self.verify_symbolically(ast, claimed_bound) {
            return VerificationResult::Verified(proof);
        }
        
        // Fall back to empirical validation
        let confidence = self.empirical_validator.test_bound(ast, claimed_bound);
        
        if confidence > 0.95 {
            VerificationResult::HighConfidence(confidence)
        } else if confidence > 0.80 {
            VerificationResult::MediumConfidence(confidence)
        } else {
            VerificationResult::LowConfidence(confidence)
        }
    }
}

pub struct EmpiricalValidator {
    pub fn test_bound(&self, ast: &UnifiedAstNode, bound: ComplexityBound) 
        -> f32 
    {
        let test_sizes = vec![10, 100, 1000, 10000];
        let measurements = Vec::new();
        
        for size in test_sizes {
            let input = self.generate_worst_case_input(ast, size);
            let operations = self.count_operations(ast, &input);
            measurements.push((size, operations));
        }
        
        // Fit curve and compare to claimed bound
        self.compute_fit_confidence(&measurements, bound)
    }
}
```

## 6. Report Generation

### 6.1 Markdown Format

```rust
impl ComplexityReportGenerator {
    pub fn generate_markdown(&self, analysis: &ComplexityAnalysis) -> String {
        let mut report = String::new();
        
        // Add summary section
        writeln!(report, "## Complexity Analysis Summary\n");
        writeln!(report, "**Total Functions Analyzed:** {}", analysis.function_count);
        writeln!(report, "**Functions with Superlinear Complexity:** {}", 
            analysis.superlinear_count);
        writeln!(report, "**Verification Coverage:** {:.1}%\n", 
            analysis.verification_coverage * 100.0);
        
        // Add hotspots table
        writeln!(report, "## Complexity Hotspots\n");
        writeln!(report, "| Function | Time | Space | Cache | Verified |");
        writeln!(report, "|----------|------|-------|-------|----------|");
        
        for hotspot in &analysis.hotspots {
            writeln!(report, "| {} | {} | {} | {} | {} |",
                hotspot.name,
                format_complexity(&hotspot.time_bound),
                format_complexity(&hotspot.space_bound),
                format_cache_complexity(&hotspot.cache_complexity),
                if hotspot.verified { "✓" } else { "✗" }
            );
        }
        
        report
    }
}
```

### 6.2 JSON Format for LLM Consumption

```rust
#[derive(Serialize)]
pub struct ComplexityReport {
    pub metadata: ReportMetadata,
    pub functions: Vec<FunctionComplexity>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub recommendations: Vec<OptimizationRecommendation>,
}

#[derive(Serialize)]
pub struct FunctionComplexity {
    pub name: String,
    pub location: Location,
    pub time_complexity: ComplexityAnnotation,
    pub space_complexity: ComplexityAnnotation,
    pub cache_behavior: Option<CacheBehavior>,
    pub vectorization_potential: Option<VectorizationAnalysis>,
    pub verification_status: VerificationStatus,
}

#[derive(Serialize)]
pub struct ComplexityAnnotation {
    pub bound: String,              // "O(n²)"
    pub class: String,              // "quadratic"
    pub input_variable: String,     // "n = array length"
    pub coefficient: Option<u16>,   // Worst-case constant
    pub confidence: f32,            // 0.0 - 1.0
    pub is_tight: bool,             // Θ notation
    pub is_amortized: bool,         
    pub proof_method: Option<String>, // "master theorem", "empirical", etc.
}
```

## 7. Integration Points

### 7.1 AST Parser Integration

```rust
// In ast_rust.rs, ast_typescript.rs, etc.
impl UnifiedAstParser for RustAstParser {
    fn parse_with_complexity(&self, source: &str) -> Result<AnnotatedAst> {
        let ast = self.parse(source)?;
        let complexity_analyzer = ComplexityAnalyzer::new();
        
        // Annotate in single pass
        let annotated = ast.map_nodes(|node| {
            let (time, space) = complexity_analyzer.analyze_node(node);
            node.with_complexity(time, space)
        });
        
        Ok(annotated)
    }
}
```

### 7.2 Cache Integration

```rust
impl ComplexityCacheStrategy {
    fn cache_key(&self, node: &UnifiedAstNode) -> u64 {
        let mut hasher = XxHash64::new();
        hasher.update(&node.kind.to_bytes());
        hasher.update(&node.structural_hash.to_le_bytes());
        hasher.finish()
    }
    
    fn should_cache(&self, complexity: &ComplexityBound) -> bool {
        // Cache verified and high-confidence results
        complexity.flags.contains(ComplexityFlags::VERIFIED) ||
        complexity.confidence > 90
    }
}
```

## 8. Performance Targets

- **Analysis Overhead**: ≤15% increase in parse time
- **Memory Overhead**: 16-24 bytes per function node
- **Cache Hit Rate**: >80% for common patterns
- **Verification Time**: <100ms for functions under 100 LOC

## 9. Testing Strategy

```rust
#[cfg(test)]
mod complexity_tests {
    use super::*;
    
    #[test]
    fn test_known_algorithms() {
        let test_cases = vec![
            ("binary_search", BigOClass::OLogN),
            ("bubble_sort", BigOClass::ON2),
            ("merge_sort", BigOClass::ONLogN),
            ("fibonacci_recursive", BigOClass::O2N),
            ("matrix_multiply", BigOClass::ON3),
        ];
        
        for (algorithm, expected) in test_cases {
            let ast = parse_algorithm(algorithm);
            let (time, _) = analyze_complexity(&ast);
            assert_eq!(time.class, expected);
        }
    }
    
    #[quickcheck]
    fn prop_complexity_monotonic(ast1: AstNode, ast2: AstNode) -> bool {
        // If ast2 contains ast1, complexity(ast2) >= complexity(ast1)
        if ast2.contains(&ast1) {
            let c1 = analyze_complexity(&ast1).0;
            let c2 = analyze_complexity(&ast2).0;
            c2.dominates(&c1)
        } else {
            true
        }
    }
}
```

## 10. Implementation Phases

**Phase 1** (Week 1): Core data structures and pattern matching for loops
- Implement `ComplexityBound` and integration with `UnifiedAstNode`
- Basic loop pattern recognition (fixed, linear, nested)
- Initial test suite with known algorithms

**Phase 2** (Week 2): Language-specific analyzers
- Rust iterator chain analysis
- Python comprehension analysis
- TypeScript promise/array method analysis
- C++ STL complexity database

**Phase 3** (Week 3): Advanced analysis
- Recurrence relation solver
- Dataflow-based analysis for complex control flow
- Cache and SIMD complexity annotations

**Phase 4** (Week 4): Verification and reporting
- SMT-based verification for simple bounds
- Empirical validation framework
- Report generation (Markdown, JSON)

**Phase 5** (Week 5): Optimization and integration
- Pattern caching for performance
- Integration with existing deep context reports
- LLM-optimized output formats

## Success Metrics

- **Coverage**: >80% of functions receive non-trivial complexity bounds
- **Accuracy**: >90% agreement with manual analysis on benchmark suite
- **Performance**: <15% overhead on baseline parsing time
- **Verification**: >30% of bounds formally verified or high-confidence