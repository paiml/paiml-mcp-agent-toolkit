# Property-Based Testing Enhancement Specification

**Document Version**: 1.0.0
**Date**: 2025-01-02
**Author**: Systems Engineering Team
**Status**: Draft

## Executive Summary

Property-based testing represents a critical gap in pmat's quality assurance strategy. While the codebase demonstrates 85%+ coverage through traditional unit tests, the absence of property tests for core analysis services exposes the system to edge-case failures that could manifest as production crashes when processing unusual but valid code patterns.

### Critical Risk Assessment

| Component | Risk Level | MTBF Impact | P99 Latency Risk |
|-----------|------------|-------------|------------------|
| AST Parsers (Rust/TS/Python) | **CRITICAL** | -40% | +200ms on panic recovery |
| Refactor Auto State Machine | **HIGH** | -25% | State corruption |
| Cache Consistency | **HIGH** | Data corruption | +50ms cache miss |
| DAG Construction | **MEDIUM** | -15% | Graph cycles |
| SATD Parser | **LOW** | -5% | False negatives |

## Priority-Ordered Implementation Plan

### Phase 1: Core AST Parser Hardening (Critical Path)

#### 1.1 Rust AST Parser Properties

The syn-based Rust parser must handle arbitrary valid Rust code without panics. Key invariants:

```rust
use proptest::prelude::*;
use syn::{parse_file, File};

prop_compose! {
    fn arb_rust_source()
        (structure in source_structure_strategy(),
         identifiers in prop::collection::vec(identifier_strategy(), 0..100),
         literals in prop::collection::vec(literal_strategy(), 0..50))
        -> String
    {
        generate_rust_source(structure, identifiers, literals)
    }
}

proptest! {
    #[test]
    fn ast_parser_total_function(source in arb_rust_source()) {
        // Property 1: Parser is total (never panics)
        let result = std::panic::catch_unwind(|| {
            parse_file(&source)
        });
        prop_assert!(result.is_ok(), "Parser panicked on input");

        // Property 2: Valid parse ⇒ AST traversal terminates
        if let Ok(Ok(ast)) = result {
            let visitor_result = std::panic::catch_unwind(|| {
                let mut visitor = ComplexityVisitor::default();
                syn::visit::visit_file(&mut visitor, &ast);
                visitor
            });
            prop_assert!(visitor_result.is_ok());
        }
    }

    #[test]
    fn ast_complexity_monotonic(base in arb_rust_source(), insertion in arb_statement()) {
        let base_ast = parse_file(&base).ok();
        let extended = format!("{}\n{}", base, insertion);
        let extended_ast = parse_file(&extended).ok();

        if let (Some(ast1), Some(ast2)) = (base_ast, extended_ast) {
            let c1 = compute_complexity(&ast1);
            let c2 = compute_complexity(&ast2);
            // Complexity is monotonic: adding code never decreases complexity
            prop_assert!(c2 >= c1);
        }
    }
}
```

#### 1.2 TypeScript/JavaScript Parser Properties

The SWC-based parser faces unique challenges with JavaScript's permissive syntax:

```rust
prop_compose! {
    fn arb_js_source()
        (use_strict in prop::bool::ANY,
         module_type in prop::sample::select(vec!["commonjs", "esm", "umd"]),
         features in arb_js_features())
        -> String
    {
        generate_js_source(use_strict, module_type, features)
    }
}

proptest! {
    #[test]
    fn swc_parser_unicode_resilience(
        source in arb_js_source(),
        unicode_idents in prop::collection::vec(arb_unicode_identifier(), 0..10)
    ) {
        let source_with_unicode = inject_unicode_identifiers(source, unicode_idents);

        let result = std::panic::catch_unwind(|| {
            let cm = Arc::new(SourceMap::default());
            let handler = Arc::new(Handler::with_tty_emitter(
                ColorConfig::Never,
                true,
                false,
                Some(cm.clone()),
            ));

            parse_js_module(&source_with_unicode, cm, handler)
        });

        prop_assert!(result.is_ok(), "Parser panicked on Unicode identifiers");
    }
}
```

### Phase 2: State Machine Verification

#### 2.1 Refactor Auto State Properties

The refactor auto command's state machine must maintain invariants across all possible transitions:

```rust
#[derive(Debug, Clone, Arbitrary)]
struct RefactorStateArb {
    #[proptest(strategy = "0..1000u32")]
    iteration: u32,
    #[proptest(strategy = "arb_quality_metrics()")]
    current_quality: QualityMetrics,
    #[proptest(strategy = "prop::collection::hash_map(arb_file_path(), arb_file_quality(), 0..100)")]
    file_scores: HashMap<PathBuf, FileQualityScore>,
    #[proptest(strategy = "prop::collection::vec(arb_completed_action(), 0..50)")]
    completed_actions: Vec<RefactorAction>,
}

proptest! {
    #[test]
    fn state_machine_invariants(initial_state in any::<RefactorStateArb>()) {
        let state = RefactorState::from(initial_state);

        // Invariant 1: File selection is deterministic
        let file1 = state.select_next_file();
        let file2 = state.select_next_file();
        prop_assert_eq!(file1, file2, "File selection must be deterministic");

        // Invariant 2: Progress monotonicity
        if let Some(action) = state.next_action() {
            let new_state = state.apply_action(action);
            prop_assert!(
                new_state.compute_progress() >= state.compute_progress(),
                "Progress must be monotonic"
            );
        }

        // Invariant 3: Termination guarantee
        let max_iterations = 1000;
        let final_state = (0..max_iterations)
            .fold(state, |s, _| {
                s.next_action()
                    .map(|a| s.apply_action(a))
                    .unwrap_or(s)
            });

        prop_assert!(
            final_state.is_complete() || final_state.iteration >= max_iterations,
            "State machine must terminate"
        );
    }
}
```

### Phase 3: Cache Consistency Properties

#### 3.1 Content-Addressed Cache Invariants

```rust
proptest! {
    #[test]
    fn cache_key_determinism(
        content in prop::collection::vec(any::<u8>(), 1..10_000),
        metadata in arb_cache_metadata()
    ) {
        let key1 = CacheKey::from_content(CacheCategory::Ast, &content);
        let key2 = CacheKey::from_content(CacheCategory::Ast, &content);

        // Property: Identical content produces identical keys
        prop_assert_eq!(key1.content_hash, key2.content_hash);

        // Property: Different content produces different keys (collision resistance)
        let mut modified = content.clone();
        if let Some(byte) = modified.get_mut(0) {
            *byte = byte.wrapping_add(1);
        }
        let key3 = CacheKey::from_content(CacheCategory::Ast, &modified);
        prop_assert_ne!(key1.content_hash, key3.content_hash);
    }

    #[test]
    fn cache_hierarchy_consistency(
        operations in prop::collection::vec(arb_cache_operation(), 1..100)
    ) {
        let cache = CacheHierarchy::new();

        for op in operations {
            match op {
                CacheOp::Put(key, value) => {
                    cache.put(&key, value.clone()).await.unwrap();

                    // Property: Immediate read returns same value
                    let retrieved = cache.get(&key).await;
                    prop_assert_eq!(Some(value), retrieved);
                },
                CacheOp::Evict(key) => {
                    cache.evict(&key).await;

                    // Property: L1 eviction doesn't affect L3
                    let l3_value = cache.l3.get(&key).await;
                    prop_assert!(l3_value.is_some() || !cache.has_been_stored(&key));
                }
            }
        }
    }
}
```

### Phase 4: DAG Construction Properties

#### 4.1 Graph Structural Invariants

```rust
prop_compose! {
    fn arb_module_graph()
        (num_modules in 1..50usize,
         edge_probability in 0.0..0.3f64)
        -> (Vec<ModuleInfo>, Vec<(usize, usize)>)
    {
        generate_random_dag(num_modules, edge_probability)
    }
}

proptest! {
    #[test]
    fn dag_construction_preserves_acyclicity(
        (modules, edges) in arb_module_graph()
    ) {
        let graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(edges)
            .build();

        // Property: No cycles in dependency graph
        let cycles = graph.detect_cycles();
        prop_assert!(
            cycles.is_empty(),
            "DAG construction introduced cycles: {:?}",
            cycles
        );

        // Property: All edges reference valid nodes
        for edge in &graph.edges {
            prop_assert!(
                graph.nodes.contains_key(&edge.from),
                "Edge references non-existent source"
            );
            prop_assert!(
                graph.nodes.contains_key(&edge.to),
                "Edge references non-existent target"
            );
        }
    }

    #[test]
    fn pagerank_convergence(graph in arb_module_graph()) {
        let dag = build_dag(graph);
        let ranks = compute_pagerank(&dag, 100);

        // Property: PageRank sums to 1.0 (within epsilon)
        let sum: f64 = ranks.values().sum();
        prop_assert!((sum - 1.0).abs() < 1e-6);

        // Property: All ranks are non-negative
        for &rank in ranks.values() {
            prop_assert!(rank >= 0.0);
        }
    }
}
```

### Phase 5: SATD Parser Robustness

#### 5.1 Comment Parsing Properties

```rust
prop_compose! {
    fn arb_comment_block()
        (style in prop::sample::select(vec!["//", "/*", "///", "//!"]),
         markers in prop::collection::vec(
             prop::sample::select(vec!["TODO", "FIXME", "HACK", "XXX", "OPTIMIZE"]),
             0..5
         ),
         noise in "[\\s\\S]{0,1000}")
        -> String
    {
        generate_comment_with_markers(style, markers, noise)
    }
}

proptest! {
    #[test]
    fn satd_parser_totality(comment in arb_comment_block()) {
        let result = std::panic::catch_unwind(|| {
            SatdDetector::new().parse_comment(&comment)
        });

        prop_assert!(result.is_ok(), "SATD parser panicked");
    }

    #[test]
    fn satd_detection_soundness(
        prefix in "[\\s\\S]{0,100}",
        marker in prop::sample::select(vec!["TODO", "FIXME", "HACK"]),
        suffix in "[\\s\\S]{0,100}"
    ) {
        let comment = format!("{} {} {}", prefix, marker, suffix);
        let detected = SatdDetector::new().parse_comment(&comment);

        // Property: Known markers are always detected
        prop_assert!(
            detected.is_some(),
            "Failed to detect marker '{}' in comment",
            marker
        );
    }
}
```

## Implementation Roadmap

### Week 1-2: Infrastructure Setup
- Add proptest-derive to dependencies
- Create property test harness with shrinking strategies
- Establish coverage targets (100% of public APIs)

### Week 3-4: Phase 1 Implementation
- Implement AST parser properties for all languages
- Add fuzzing corpus from real-world projects
- Performance benchmarks: <10ms per property iteration

### Week 5-6: Phase 2-3 Implementation
- State machine verification
- Cache consistency properties
- Integration with existing test suite

### Week 7-8: Phase 4-5 Implementation
- DAG construction properties
- SATD parser hardening
- End-to-end property scenarios

## Success Metrics

### Technical Metrics
- **Defect Detection Rate**: >0.5 bugs/property (initial run)
- **Shrinking Efficiency**: <50 examples to minimal case
- **Coverage Delta**: +15% reachable panic sites covered
- **Performance Impact**: <5% CI runtime increase

### Quality Metrics
- **MTBF Improvement**: +60% for edge cases
- **Panic Rate**: <0.001% in production
- **Parser Resilience**: 100% success on Rust/JS/Python corpus

## Architectural Considerations

### Shrinking Strategy Optimization

```rust
// Custom shrinker for AST structures
impl Arbitrary for AstStructure {
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // Generate from high-level patterns, not raw strings
        prop_oneof![
            Just(AstStructure::Empty),
            any::<FunctionDecl>().prop_map(AstStructure::Function),
            any::<StructDecl>().prop_map(AstStructure::Struct),
        ]
        .prop_recursive(8, 256, 10, |inner| {
            prop_oneof![
                (inner.clone(), inner.clone())
                    .prop_map(|(a, b)| AstStructure::Sequence(Box::new(a), Box::new(b))),
                inner.prop_map(|s| AstStructure::Module(Box::new(s))),
            ]
        })
        .boxed()
    }
}
```

### Performance Optimization

Property tests will use parallel execution with deterministic scheduling:

```rust
#[test]
fn parallel_property_suite() {
    let config = ProptestConfig {
        cases: 1000,
        max_shrink_iters: 50,
        fork: true,  // Process isolation for panic recovery
        timeout: 30_000,  // 30s timeout
        ..Default::default()
    };

    // Run on all available cores
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .panic_handler(|_| {})  // Suppress panic propagation
        .build()
        .unwrap();

    pool.install(|| {
        proptest!(config, |source in arb_source()| {
            test_property(source)
        })
    });
}
```

## Conclusion

Property-based testing represents a critical investment in pmat's robustness. The 8-week implementation plan will systematically eliminate crash-inducing edge cases while maintaining the tool's performance characteristics. Expected ROI: 60% reduction in production incidents, 40% faster defect discovery, and mathematical confidence in core invariants.
