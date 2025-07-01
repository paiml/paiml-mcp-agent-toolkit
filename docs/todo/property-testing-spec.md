# Property Testing Integration for PMAT

## Overview

Property-based testing represents a paradigm shift from example-based testing, enabling exploration of vast input spaces through automated generation of test cases. For PMAT's codebase, this approach offers systematic coverage of edge cases that traditional unit tests miss, particularly crucial for AST manipulation and refactoring operations where input space complexity exceeds manual enumeration capacity.

## Motivation

### Coverage Amplification

Traditional unit tests achieve linear coverage growth: one test, one scenario. Property tests achieve exponential coverage through input space exploration:

```rust
// Traditional test: covers 1 case
#[test]
fn test_complexity_calculation() {
    let ast = parse_rust_code("fn foo() { if x { y } }");
    assert_eq!(calculate_complexity(&ast), 2);
}

// Property test: covers 10^6+ cases
#[quickcheck]
fn prop_complexity_monotonic(ast: ArbitraryAst) -> bool {
    let base_complexity = calculate_complexity(&ast.0);
    let extended_ast = ast.0.add_branch();
    calculate_complexity(&extended_ast) >= base_complexity
}
```

### Integration with `pmat refactor auto`

The refactoring engine's correctness depends on preserving semantic equivalence while improving code quality metrics. Property testing provides formal guarantees:

```rust
#[quickcheck]
fn prop_refactoring_preserves_semantics(code: ValidRustCode) -> TestResult {
    let original_ast = parse(&code.0)?;
    let refactored = pmat_refactor_auto(&original_ast)?;
    
    // Semantic preservation
    assert_eq!(
        extract_semantics(&original_ast),
        extract_semantics(&refactored)
    );
    
    // Quality improvement
    assert!(calculate_tdg(&refactored) <= calculate_tdg(&original_ast));
    
    TestResult::passed()
}
```

## Implementation Strategy

### Phase 1: Infrastructure

1. **Dependency Integration**
   ```toml
   [dev-dependencies]
   quickcheck = "1.0"
   quickcheck_macros = "1.0"
   proptest = "1.4"  # For stateful testing
   ```

2. **Arbitrary Type Generation**
   ```rust
   // src/testing/arbitrary.rs
   use quickcheck::{Arbitrary, Gen};
   
   impl Arbitrary for ValidRustCode {
       fn arbitrary(g: &mut Gen) -> Self {
           // Generate syntactically valid Rust code
           let depth = g.size().min(5);
           ValidRustCode(generate_ast_recursive(g, depth).to_string())
       }
       
       fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
           // Minimal reproducible failing case
           Box::new(self.0.lines()
               .enumerate()
               .filter_map(|(i, _)| {
                   let mut lines: Vec<_> = self.0.lines().collect();
                   lines.remove(i);
                   let candidate = lines.join("\n");
                   syn::parse_str::<syn::File>(&candidate)
                       .ok()
                       .map(|_| ValidRustCode(candidate))
               }))
       }
   }
   ```

### Phase 2: Core Properties

1. **AST Analysis Properties**
   ```rust
   mod ast_properties {
       use super::*;
       
       #[quickcheck]
       fn prop_complexity_bounds(ast: ArbitraryAst) -> bool {
           let complexity = calculate_complexity(&ast.0);
           complexity >= 1 && complexity <= ast.0.node_count()
       }
       
       #[quickcheck]
       fn prop_dead_code_detection_sound(ast: ArbitraryAst) -> bool {
           let dead_nodes = detect_dead_code(&ast.0);
           dead_nodes.iter().all(|node| !is_reachable(node, &ast.0))
       }
   }
   ```

2. **Refactoring Properties**
   ```rust
   #[derive(Clone, Debug)]
   struct RefactoringChain {
       initial: ValidRustCode,
       operations: Vec<RefactoringOp>,
   }
   
   impl Arbitrary for RefactoringChain {
       fn arbitrary(g: &mut Gen) -> Self {
           let initial = ValidRustCode::arbitrary(g);
           let op_count = g.choose(&[1, 2, 3, 5, 8]).copied().unwrap();
           let operations = (0..op_count)
               .map(|_| RefactoringOp::arbitrary(g))
               .collect();
           RefactoringChain { initial, operations }
       }
   }
   
   #[quickcheck]
   fn prop_refactoring_chain_convergence(chain: RefactoringChain) -> TestResult {
       let mut current = parse(&chain.initial.0)?;
       let mut tdg_history = vec![calculate_tdg(&current)];
       
       for op in &chain.operations {
           current = apply_refactoring(current, op)?;
           tdg_history.push(calculate_tdg(&current));
       }
       
       // TDG should monotonically decrease or stabilize
       tdg_history.windows(2).all(|w| w[1] <= w[0])
   }
   ```

### Phase 3: Coverage-Driven Property Discovery

1. **Coverage-Guided Fuzzing Integration**
   ```rust
   use cargo_fuzz::FuzzTarget;
   
   #[fuzz_target]
   fn fuzz_refactor_auto(data: &[u8]) {
       if let Ok(code) = std::str::from_utf8(data) {
           if let Ok(ast) = syn::parse_str::<syn::File>(code) {
               let _ = pmat_refactor_auto(&ast);
           }
       }
   }
   ```

2. **Mutation Testing Feedback Loop**
   ```rust
   #[test]
   fn property_test_coverage_analysis() {
       let coverage_before = measure_coverage(|| run_unit_tests());
       let coverage_after = measure_coverage(|| run_property_tests());
       
       // Identify uncovered branches
       let uncovered = coverage_after.uncovered_branches();
       
       // Generate targeted properties
       for branch in uncovered {
           generate_property_for_branch(branch);
       }
   }
   ```

## Integration with Quality Gates

### Automated Property Generation

```rust
// src/quality/property_generator.rs
pub fn generate_properties_from_satd(satd_entries: &[SatdEntry]) -> Vec<PropertyTest> {
    satd_entries.iter()
        .filter_map(|entry| match entry.debt_type {
            DebtType::MissingValidation => Some(generate_validation_property(entry)),
            DebtType::EdgeCaseHandling => Some(generate_edge_case_property(entry)),
            _ => None,
        })
        .collect()
}
```

### CI/CD Integration

```yaml
# .github/workflows/property-tests.yml
property-coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - name: Run property tests with coverage
      run: |
        cargo tarpaulin --features property-tests \
          --timeout 1200 \
          --run-types Tests \
          --packages paiml-mcp-agent-toolkit
    - name: Enforce coverage threshold
      run: |
        coverage=$(cargo tarpaulin --print-summary | grep 'Coverage' | awk '{print $2}' | sed 's/%//')
        if (( $(echo "$coverage < 95" | bc -l) )); then
          echo "Coverage $coverage% below 95% threshold"
          exit 1
        fi
```

## Performance Considerations

### Lazy Generation Strategy

```rust
struct LazyAstGenerator<'a> {
    rng: &'a mut Gen,
    cache: HashMap<AstPattern, syn::Expr>,
}

impl<'a> LazyAstGenerator<'a> {
    fn generate_expr(&mut self, depth: usize) -> syn::Expr {
        let pattern = self.classify_pattern(depth);
        
        self.cache.entry(pattern)
            .or_insert_with(|| self.generate_fresh_expr(depth))
            .clone()
    }
}
```

### Parallel Property Execution

```rust
use rayon::prelude::*;

#[test]
fn parallel_property_suite() {
    let properties: Vec<Box<dyn Fn() -> TestResult + Send + Sync>> = vec![
        Box::new(|| prop_complexity_bounds(ArbitraryAst::arbitrary(&mut Gen::new(100)))),
        Box::new(|| prop_refactoring_preserves_semantics(ValidRustCode::arbitrary(&mut Gen::new(100)))),
        // ... more properties
    ];
    
    let results: Vec<_> = properties
        .par_iter()
        .map(|prop| std::panic::catch_unwind(|| prop()))
        .collect();
        
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## Metrics and Monitoring

### Coverage Progression Tracking

```rust
#[derive(Serialize)]
struct CoverageReport {
    timestamp: DateTime<Utc>,
    line_coverage: f64,
    branch_coverage: f64,
    property_test_count: usize,
    unique_inputs_generated: usize,
    bugs_found: Vec<BugReport>,
}

impl CoverageReport {
    fn compare_with_baseline(&self, baseline: &CoverageReport) -> CoverageDelta {
        CoverageDelta {
            line_delta: self.line_coverage - baseline.line_coverage,
            branch_delta: self.branch_coverage - baseline.branch_coverage,
            new_bugs: self.bugs_found.len() - baseline.bugs_found.len(),
        }
    }
}
```

## Success Criteria

1. **Coverage Targets**
   - Line coverage: ≥95%
   - Branch coverage: ≥90%
   - Property test execution: <5min on CI

2. **Quality Metrics**
   - Zero false positives in property failures
   - 100% reproducibility of property test failures
   - <100ms shrinking time for failing cases

3. **Integration Goals**
   - Seamless `pmat refactor auto` validation
   - Automated property generation from code patterns
   - CI/CD enforcement of property test passage

## References

1. Claessen, K., & Hughes, J. (2000). QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs
2. Padhye, R., et al. (2019). Semantic Fuzzing with Zest
3. The Rust Fuzz Book: https://rust-fuzz.github.io/book/
