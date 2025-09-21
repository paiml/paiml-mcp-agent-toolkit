# Sprint 1: Graph Statistics Core Foundation
## 🎯 Sprint Goal: Implement core graph data structures and PageRank with extreme TDD

### 📝 Tickets

---

## TICKET-001: Graph Type System and Data Structures
**Priority**: P0
**Complexity**: 5
**Estimated**: 4h

### Acceptance Criteria
- [ ] NodeData struct with all fields (path, module, symbols, loc, complexity, ast_hash)
- [ ] EdgeData enum with all variants (Import, FunctionCall, TypeDependency, DataFlow, Inheritance)
- [ ] DependencyGraph type alias using petgraph::DiGraph
- [ ] GraphMatrices struct with CSR matrices
- [ ] All types implement Serialize/Deserialize
- [ ] Cyclomatic complexity ≤ 10 for all methods

### Test Requirements
```rust
// Must write these tests FIRST:
- test_node_data_creation()
- test_edge_data_weight_conversion()
- test_graph_matrices_from_graph()
- property_test_graph_construction()
```

---

## TICKET-002: Symbol Table and Resolution
**Priority**: P0
**Complexity**: 7
**Estimated**: 6h

### Acceptance Criteria
- [ ] SymbolTable with O(1) lookup
- [ ] Symbol resolution across modules
- [ ] Handle visibility (pub, private, protected)
- [ ] Track symbol usage count
- [ ] Support for generics and traits

### Test Requirements
```rust
- test_symbol_table_insertion()
- test_symbol_resolution_cross_module()
- test_visibility_rules()
- test_generic_symbol_resolution()
```

---

## TICKET-003: Rust AST Parser for Dependencies
**Priority**: P0
**Complexity**: 8
**Estimated**: 8h

### Acceptance Criteria
- [ ] Parse Rust files using syn
- [ ] Extract imports (use statements)
- [ ] Extract function calls
- [ ] Extract type dependencies
- [ ] Build edges with proper weights
- [ ] Handle macro expansions

### Test Requirements
```rust
- test_parse_rust_imports()
- test_parse_rust_function_calls()
- test_parse_rust_type_deps()
- test_rust_macro_handling()
- property_test_rust_parser_never_panics()
```

---

## TICKET-004: Python AST Parser for Dependencies
**Priority**: P0
**Complexity**: 8
**Estimated**: 8h

### Acceptance Criteria
- [ ] Parse Python using tree-sitter
- [ ] Extract import/from statements
- [ ] Extract function calls
- [ ] Extract class inheritance
- [ ] Handle dynamic imports
- [ ] Support type hints

### Test Requirements
```rust
- test_parse_python_imports()
- test_parse_python_inheritance()
- test_parse_python_type_hints()
- test_python_dynamic_imports()
```

---

## TICKET-005: TypeScript/JavaScript Parser
**Priority**: P0
**Complexity**: 8
**Estimated**: 8h

### Acceptance Criteria
- [ ] Parse TS/JS using swc
- [ ] Extract ES6 imports/exports
- [ ] Extract require() calls
- [ ] Extract class extends
- [ ] Handle JSX/TSX
- [ ] Support dynamic imports

### Test Requirements
```rust
- test_parse_es6_imports()
- test_parse_commonjs_require()
- test_parse_jsx_dependencies()
- test_typescript_type_imports()
```

---

## TICKET-006: DependencyGraphBuilder Integration
**Priority**: P0
**Complexity**: 10
**Estimated**: 8h

### Acceptance Criteria
- [ ] from_workspace() method working
- [ ] Collect all source files
- [ ] Build global symbol table
- [ ] Analyze each file by language
- [ ] Resolve cross-file dependencies
- [ ] Compute edge weights
- [ ] Incremental updates via ast_hash

### Test Requirements
```rust
- test_build_from_small_workspace()
- test_incremental_graph_update()
- test_circular_dependency_handling()
- test_multi_language_workspace()
- benchmark_graph_construction_performance()
```

---

## TICKET-007: PageRank Implementation
**Priority**: P0
**Complexity**: 9
**Estimated**: 6h

### Acceptance Criteria
- [ ] Power iteration implementation
- [ ] Damping factor support (default 0.85)
- [ ] Convergence detection (tolerance 1e-6)
- [ ] Handle dangling nodes
- [ ] CSR matrix optimization
- [ ] Max iterations limit

### Test Requirements
```rust
- test_pagerank_sum_preservation()  // Must sum to 1.0
- test_pagerank_convergence()
- test_pagerank_star_graph()  // Center has highest
- test_pagerank_complete_graph()  // All equal
- property_test_pagerank_invariants()
- benchmark_pagerank_scaling()
```

---

## TICKET-008: Graph to Matrix Conversion
**Priority**: P0
**Complexity**: 6
**Estimated**: 4h

### Acceptance Criteria
- [ ] Convert DiGraph to CSR adjacency matrix
- [ ] Build column-stochastic transition matrix
- [ ] Compute graph Laplacian
- [ ] Handle sparse graphs efficiently
- [ ] Support weighted edges

### Test Requirements
```rust
- test_adjacency_matrix_construction()
- test_transition_matrix_stochastic()
- test_laplacian_properties()
- test_sparse_graph_efficiency()
```

---

## TICKET-009: Property-Based Testing Suite
**Priority**: P0
**Complexity**: 5
**Estimated**: 4h

### Acceptance Criteria
- [ ] Graph generator for proptest
- [ ] Arbitrary graph generation (5-1000 nodes)
- [ ] Edge weight generation
- [ ] Test invariants for all algorithms
- [ ] Never panic on any input

### Test Requirements
```rust
- arbitrary_graph_generator()
- test_all_algorithms_handle_empty_graph()
- test_all_algorithms_handle_single_node()
- test_no_panics_on_random_inputs()
```

---

## TICKET-010: Sprint 1 Quality Gate & Release
**Priority**: P0
**Complexity**: 3
**Estimated**: 4h

### Acceptance Criteria
- [ ] All tests passing (100%)
- [ ] Code coverage ≥ 95%
- [ ] Zero clippy warnings
- [ ] Zero SATD comments
- [ ] All functions complexity ≤ 10
- [ ] Benchmarks established
- [ ] Documentation complete

### Release Checklist
- [ ] Run: `cargo test --all-features`
- [ ] Run: `cargo clippy -- -D warnings`
- [ ] Run: `pmat enforce . --zero-tolerance`
- [ ] Run: `make coverage` (must show ≥ 95%)
- [ ] Commit message follows convention
- [ ] Push to master branch

---

## 📊 Sprint 1 Metrics
- **Total Complexity Points**: 69
- **Total Estimated Hours**: 56h
- **Test-First Tickets**: 10/10 (100%)
- **Quality Gates**: Every ticket must pass before moving to next

## 🚀 Definition of Done
1. All tests written BEFORE implementation
2. All tests passing
3. Code coverage ≥ 95% for new code
4. Cyclomatic complexity ≤ 10
5. Zero SATD
6. Performance benchmarks met
7. Documentation complete
8. Pushed to master with clean build