# TICKET-PMAT-7010: TypeScript/JavaScript AST-Based Mutation Testing (Priority 0)

**Sprint:** Sprint 25 - Multi-Language Mutation Testing
**Priority:** P0 - Critical (TypeScript/JavaScript FIRST)
**Estimated Effort:** 25-30 hours (3 phases over 5-7 days)
**Status**: 🚀 TODO
**Created:** 2025-10-08
**Dependencies:** PMAT-7004 (ML Mutation Predictor - Complete)
**Release:** v2.145.0

---

## Problem Statement

PMAT has mutation testing for Rust only. TypeScript/JavaScript mutation testing is **broken** - it uses Rust's `syn` AST operators which don't work for TS/JS code. Multi-language mutation testing requires AST-native operators per language.

### Current Issues

1. **Non-Functional TypeScript Mutations:**
   ```rust
   // server/src/services/mutation/typescript_adapter.rs
   fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>> {
       vec![
           Box::new(ArithmeticOperatorReplacement),  // ❌ Uses Rust syn::Expr!
           // ...
       ]
   }
   ```
   **Problem:** `syn::Expr` only parses Rust, not TypeScript!

2. **Stub Test Execution:**
   ```rust
   async fn run_tests(&self, _source_file: &Path) -> Result<TestRunResult> {
       Ok(TestRunResult { passed: true, ... })  // ❌ Always passes!
   }
   ```
   **Problem:** No actual `npm test` or `jest` execution!

3. **No Tree-Sitter Integration:**
   - `tree-sitter-typescript` dependency exists but unused
   - `typescript-ast` feature flag exists but not leveraged
   - AST parsing validates syntax but doesn't mutate

4. **Missing Test Coverage:**
   - `typescript_adapter_tests.rs` has minimal tests
   - No mutation operator unit tests for TS/JS
   - No integration tests with real TypeScript projects

### Real-World Impact

**From Mutation Testing Status (v2.130.0):**
> "PMAT v2.130.0 now has functional empirical mutation testing for Rust, but TypeScript/JavaScript/Python/Go/C++ adapters are stubs. Cannot benchmark against other mutation tools for polyglot projects."

**User Expectation:**
- Run `pmat analyze mutate --path calculator.ts`
- Expect mutations like `+ → -`, `=== → !==`
- **Reality:** Nothing happens or crashes

---

## Solution: EXTREME TDD Implementation

### Phase 1: RED (Days 1-2) 🔴
Write ALL failing tests before implementation.

**Deliverables:**
- `server/src/services/mutation/typescript_tree_sitter_operators_tests.rs` - All RED
- `server/tests/typescript_mutation_integration.rs` - All RED
- Test fixtures: `fixtures/typescript/calculator.ts` with comprehensive tests
- Property tests for operator correctness

### Phase 2: GREEN (Days 3-5) 🟢
Implement tree-sitter AST mutation to pass tests.

**Deliverables:**
- `server/src/services/mutation/tree_sitter_operators.rs` - Generic trait
- `server/src/services/mutation/typescript_tree_sitter_mutations.rs` - TS-specific
- Real `npm test` / `jest` execution in `TypeScriptAdapter::run_tests()`
- All tests passing

### Phase 3: REFACTOR (Days 6-7) 🔵
Optimize, integrate mutation testing, achieve >80% mutation score.

**Deliverables:**
- Mutation score >80% on TypeScript mutation module itself
- Performance: <5s mutation generation for 1000-line TS files
- Integration with ML mutation predictor
- Documentation and examples

---

## Requirements

### FR-1: Tree-Sitter AST Mutation Operators for TypeScript/JavaScript

**Requirement:** Language-agnostic mutation operators using tree-sitter AST.

**Success Criteria:**
- [ ] `TreeSitterMutationOperator` trait implemented
- [ ] 6+ operators: AOR, ROR, COR, UOR, CRR, SDL
- [ ] Works for both TypeScript and JavaScript (.ts, .tsx, .js, .jsx)
- [ ] Mutations preserve syntax validity
- [ ] Location metadata (line, column) extracted from AST

**Test (RED Phase):**
```rust
#[test]
fn red_test_typescript_arithmetic_operator_replacement() {
    let source = "function add(a: number, b: number) { return a + b; }";
    let adapter = TypeScriptTreeSitterAdapter::new();

    let mutants = adapter.generate_mutants(source).unwrap();

    // Should generate: a - b, a * b, a / b
    assert!(mutants.len() >= 3);
    assert!(mutants.iter().any(|m| m.mutated_source.contains("a - b")));
    assert!(mutants.iter().any(|m| m.mutated_source.contains("a * b")));
    assert!(mutants.iter().any(|m| m.operator == "AOR"));
}
```

**Expected Behavior:**
```
❌ FAILED (RED phase expected)
   Expected 3+ mutants, got 0
   TypeScript AST mutation not implemented yet
```

---

### FR-2: Real Test Execution for TypeScript/JavaScript

**Requirement:** Execute npm/jest/vitest tests and parse failures.

**Success Criteria:**
- [ ] Detect package.json and test framework (jest, vitest, mocha, ava)
- [ ] Run tests: `npm test` or `npx jest` or framework-specific
- [ ] Parse test output to identify failures
- [ ] Timeout handling (default: 30s per mutant)
- [ ] Classify: Killed, Survived, CompileError, Timeout

**Test (RED Phase):**
```rust
#[tokio::test]
async fn red_test_typescript_test_execution() {
    let fixture = "fixtures/typescript/calculator.ts";
    let adapter = TypeScriptAdapter::new();

    let result = adapter.run_tests(fixture).await.unwrap();

    assert!(result.passed || !result.failures.is_empty());
    assert!(result.execution_time_ms > 0);
    assert!(!result.stdout.is_empty() || !result.stderr.is_empty());
}
```

**Expected Behavior:**
```
❌ FAILED (RED phase expected)
   Result always shows passed=true with empty stdout/stderr
   Real test execution not implemented
```

---

### FR-3: TypeScript-Specific Mutation Operators

**Requirement:** Operators for TypeScript-specific syntax.

**Success Criteria:**
- [ ] Strict equality mutations: `===` ↔ `==`, `!==` ↔ `!=`
- [ ] Optional chaining mutations: `obj?.prop` → `obj.prop`
- [ ] Nullish coalescing: `a ?? b` → `a || b`, `a ?? b` → `b`
- [ ] Async/await mutations: Remove `await`, change `async` → `sync`
- [ ] Arrow function mutations: `() => x` → `() => {}`, `() => {}` → `() => null`

**Test (RED Phase):**
```rust
#[test]
fn red_test_typescript_strict_equality_mutation() {
    let source = "if (x === 5) { return true; }";
    let mutants = generate_typescript_mutants(source).unwrap();

    assert!(mutants.iter().any(|m| m.mutated_source.contains("x == 5")));
    assert!(mutants.iter().any(|m| m.mutated_source.contains("x !== 5")));
}

#[test]
fn red_test_typescript_optional_chaining_mutation() {
    let source = "const val = obj?.prop?.nested;";
    let mutants = generate_typescript_mutants(source).unwrap();

    assert!(mutants.iter().any(|m| m.mutated_source.contains("obj.prop?.nested")));
    assert!(mutants.iter().any(|m| m.mutated_source.contains("obj?.prop.nested")));
}

#[test]
fn red_test_typescript_async_await_mutation() {
    let source = "async function fetch() { return await api(); }";
    let mutants = generate_typescript_mutants(source).unwrap();

    assert!(mutants.iter().any(|m| m.mutated_source.contains("return api()"))); // Remove await
    assert!(mutants.iter().any(|m| !m.mutated_source.contains("async"))); // Remove async
}
```

**Expected Behavior:**
```
❌ All tests FAILED (RED phase expected)
   TypeScript-specific operators not implemented
```

---

### FR-4: Mutation Score >80% on TypeScript Mutation Module

**Requirement:** Dogfood mutation testing on itself.

**Success Criteria:**
- [ ] Run `pmat analyze mutate --path server/src/services/mutation/typescript_tree_sitter_mutations.rs`
- [ ] Mutation score ≥80%
- [ ] All surviving mutants documented with justification
- [ ] Tests added to kill survivable mutants

**Test (RED Phase):**
```rust
#[test]
fn red_test_typescript_mutation_module_achieves_80_percent_score() {
    let mutation_module = Path::new("server/src/services/mutation/typescript_tree_sitter_mutations.rs");
    let engine = MutationEngine::new(/* ... */);

    let score = engine.calculate_mutation_score(mutation_module).unwrap();

    assert!(
        score >= 0.80,
        "Mutation score {:.1}% below 80% threshold. Add tests!",
        score * 100.0
    );
}
```

**Expected Behavior:**
```
❌ FAILED (RED phase expected)
   Mutation score 0.0% below 80% threshold
   Module doesn't exist yet
```

---

## Implementation Plan

### Phase 1: RED Phase (Days 1-2)

**Day 1: Core Operator Tests**

1. Create test file structure:
```
server/src/services/mutation/
├── tree_sitter_operators.rs          (trait definition - stub)
├── typescript_tree_sitter_mutations.rs (TS implementation - stub)
└── tests/
    ├── typescript_tree_sitter_tests.rs (unit tests - RED)
    └── typescript_mutation_integration.rs (integration - RED)
```

2. Write RED tests for:
   - Arithmetic operator replacement (AOR)
   - Relational operator replacement (ROR)
   - Conditional operator replacement (COR)
   - Unary operator replacement (UOR)
   - Constant replacement (CRR)
   - Statement deletion (SDL)

3. Create test fixtures:
```typescript
// fixtures/typescript/calculator.ts
export function add(a: number, b: number): number {
    return a + b;
}

export function isPositive(x: number): boolean {
    return x > 0;
}

// fixtures/typescript/calculator.test.ts
import { add, isPositive } from './calculator';

test('add returns sum', () => {
    expect(add(2, 3)).toBe(5);
});

test('isPositive checks sign', () => {
    expect(isPositive(5)).toBe(true);
    expect(isPositive(-5)).toBe(false);
});
```

**Day 2: Test Execution & Advanced Operators Tests**

4. Write RED tests for test execution:
   - Detect package.json
   - Run npm/jest/vitest
   - Parse test failures
   - Handle timeouts

5. Write RED tests for TypeScript-specific operators:
   - Strict equality mutations
   - Optional chaining mutations
   - Nullish coalescing mutations
   - Async/await mutations
   - Arrow function mutations

6. Property tests:
```rust
#[proptest]
fn test_all_mutations_preserve_syntax(
    #[strategy(typescript_source_strategy())] source: String
) {
    let adapter = TypeScriptTreeSitterAdapter::new();
    let mutants = adapter.generate_mutants(&source).unwrap();

    for mutant in mutants {
        // All mutants must parse without syntax errors
        prop_assert!(adapter.parse(&mutant.mutated_source).is_ok());
    }
}
```

**Deliverable:** All tests written, all failing (RED phase complete)

---

### Phase 2: GREEN Phase (Days 3-5)

**Day 3: Tree-Sitter Operator Trait & Core Implementation**

1. Implement `TreeSitterMutationOperator` trait:
```rust
// server/src/services/mutation/tree_sitter_operators.rs
use tree_sitter::{Node, Tree, Parser};

pub trait TreeSitterMutationOperator: Send + Sync {
    /// Operator name (e.g., "AOR", "ROR")
    fn name(&self) -> &str;

    /// Can this operator mutate the given AST node?
    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool;

    /// Generate mutants for the given node
    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource>;

    /// Estimated kill probability
    fn kill_probability(&self) -> f64 { 0.5 }
}

pub struct MutatedSource {
    pub source: String,
    pub description: String,
    pub location: SourceLocation,
}
```

2. Implement TypeScript binary operator mutation:
```rust
// server/src/services/mutation/typescript_tree_sitter_mutations.rs
pub struct TypeScriptBinaryOpMutation;

impl TreeSitterMutationOperator for TypeScriptBinaryOpMutation {
    fn name(&self) -> &str { "AOR/ROR" }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        matches!(node.kind(), "binary_expression")
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let op_node = node.child_by_field_name("operator").unwrap();
        let op_text = &source[op_node.byte_range()];

        let replacements = match op_text {
            b"+" => vec!["-", "*", "/"],
            b"-" => vec!["+", "*", "/"],
            b"===" => vec!["!==", "==", "!="],
            b">" => vec!["<", ">=", "<=", "=="],
            // ... all operators
            _ => vec![],
        };

        replacements.into_iter().map(|new_op| {
            let mut mutated = source.to_vec();
            mutated.splice(
                op_node.byte_range(),
                new_op.bytes(),
            );

            MutatedSource {
                source: String::from_utf8(mutated).unwrap(),
                description: format!("{} → {}",
                    String::from_utf8_lossy(op_text),
                    new_op
                ),
                location: SourceLocation {
                    line: op_node.start_position().row + 1,
                    column: op_node.start_position().column + 1,
                },
            }
        }).collect()
    }
}
```

**Day 4: TypeScript Test Runner Implementation**

3. Implement real test execution:
```rust
// server/src/services/mutation/typescript_adapter.rs
impl LanguageAdapter for TypeScriptAdapter {
    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult> {
        // Find package.json root
        let project_root = find_package_json_root(source_file)
            .ok_or_else(|| anyhow!("No package.json found"))?;

        // Detect test framework from package.json
        let package_json = fs::read_to_string(project_root.join("package.json"))?;
        let test_cmd = detect_test_command(&package_json)?;

        // Run tests with timeout
        let start = Instant::now();
        let output = tokio::process::Command::new("npm")
            .arg("run")
            .arg(test_cmd)
            .current_dir(project_root)
            .output()
            .timeout(Duration::from_secs(30))
            .await??;

        let execution_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Parse failures
        let failures = parse_test_failures(&stdout, &stderr)?;
        let passed = output.status.success();

        Ok(TestRunResult {
            passed,
            failures,
            execution_time_ms,
            stdout,
            stderr,
        })
    }
}

fn detect_test_command(package_json: &str) -> Result<String> {
    let pkg: serde_json::Value = serde_json::from_str(package_json)?;

    // Check scripts for test command
    if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
        if scripts.contains_key("test") {
            return Ok("test".to_string());
        }
    }

    // Check devDependencies for framework
    if let Some(deps) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key("jest") {
            return Ok("jest".to_string());
        }
        if deps.contains_key("vitest") {
            return Ok("vitest".to_string());
        }
    }

    Err(anyhow!("No test command found in package.json"))
}

fn parse_test_failures(stdout: &str, stderr: &str) -> Result<Vec<String>> {
    let mut failures = Vec::new();

    // Jest format: "FAIL path/to/test.ts"
    for line in stdout.lines().chain(stderr.lines()) {
        if line.trim_start().starts_with("FAIL ") {
            failures.push(line.trim().to_string());
        }
        // Vitest format: "❯ test name"
        if line.contains("❯") && line.contains("FAIL") {
            failures.push(line.trim().to_string());
        }
    }

    Ok(failures)
}
```

**Day 5: TypeScript-Specific Operators & Integration**

4. Implement TypeScript-specific mutations:
   - Strict equality operator
   - Optional chaining operator
   - Nullish coalescing operator
   - Async/await mutations
   - Arrow function mutations

5. Update `TypeScriptAdapter::mutation_operators()`:
```rust
fn mutation_operators(&self) -> Vec<Box<dyn TreeSitterMutationOperator>> {
    vec![
        Box::new(TypeScriptBinaryOpMutation),
        Box::new(TypeScriptUnaryOpMutation),
        Box::new(TypeScriptStrictEqualityMutation),
        Box::new(TypeScriptOptionalChainingMutation),
        Box::new(TypeScriptNullishCoalescingMutation),
        Box::new(TypeScriptAsyncAwaitMutation),
        Box::new(TypeScriptArrowFunctionMutation),
        Box::new(TypeScriptConstantReplacementMutation),
        Box::new(TypeScriptStatementDeletionMutation),
    ]
}
```

6. Integration with mutation engine:
```rust
// server/src/services/mutation/engine.rs (modify)
pub async fn generate_mutants_from_source(
    &self,
    file_path: &Path,
    source: &str,
) -> Result<Vec<Mutant>> {
    // Detect language
    let lang = detect_language(file_path)?;

    match lang {
        Language::Rust => self.generate_rust_mutants(source).await,
        Language::TypeScript | Language::JavaScript => {
            self.generate_typescript_mutants(source).await
        }
        // ... other languages
    }
}
```

**Deliverable:** All tests passing (GREEN phase complete)

---

### Phase 3: REFACTOR Phase (Days 6-7)

**Day 6: Mutation Testing the Mutation Module**

1. Run mutation testing on TypeScript mutation module:
```bash
pmat analyze mutate \
  --path server/src/services/mutation/typescript_tree_sitter_mutations.rs \
  --operators all \
  --output typescript_mutation_score.json
```

2. Analyze survivors:
   - Identify weak tests
   - Add tests to kill survivable mutants
   - Document equivalent mutants

3. Iterate until mutation score ≥80%

**Day 7: Performance Optimization & Documentation**

4. Performance benchmarks:
```rust
#[bench]
fn bench_typescript_mutation_generation(b: &mut Bencher) {
    let source = include_str!("../../../fixtures/typescript/large_file.ts"); // 1000 lines
    let adapter = TypeScriptTreeSitterAdapter::new();

    b.iter(|| {
        let mutants = adapter.generate_mutants(source).unwrap();
        black_box(mutants)
    });
}
```

**Target:** <5s for 1000-line files

5. Integration with ML mutation predictor:
```rust
// Use pattern learning to predict TypeScript mutant survivability
let prediction = ml_predictor.predict_with_patterns(
    &mutant,
    &context,
    &pattern_service
).await?;

// Prioritize high-probability survivors
mutants.sort_by(|a, b| {
    b.survival_probability.partial_cmp(&a.survival_probability).unwrap()
});
```

6. Documentation:
   - Update `docs/mutation-testing.md` with TypeScript examples
   - Create `docs/typescript-mutation-operators.md`
   - Add CLI usage examples

**Deliverable:** Optimized, documented, production-ready (REFACTOR complete)

---

## Files to Create

### New Files (GREEN Phase)
```
server/src/services/mutation/tree_sitter_operators.rs          (200 lines)
server/src/services/mutation/typescript_tree_sitter_mutations.rs (800 lines)
server/src/services/mutation/tests/typescript_tree_sitter_tests.rs (600 lines)
server/tests/typescript_mutation_integration.rs                (400 lines)
fixtures/typescript/calculator.ts                              (100 lines)
fixtures/typescript/calculator.test.ts                         (150 lines)
fixtures/typescript/package.json                               (50 lines)
docs/typescript-mutation-operators.md                          (500 lines)
```

### Files to Modify
```
server/src/services/mutation/typescript_adapter.rs             (+200 lines for run_tests)
server/src/services/mutation/mod.rs                            (export tree_sitter_operators)
server/src/services/mutation/engine.rs                         (+100 lines language detection)
server/Cargo.toml                                              (ensure tree-sitter-typescript)
docs/mutation-testing.md                                       (+300 lines TS examples)
```

**Estimated Total:** ~2,600 new lines + 600 modified lines

---

## Success Criteria

### Functional Requirements ✅
- [ ] Tree-sitter AST mutation working for TypeScript/JavaScript
- [ ] 9+ mutation operators implemented (6 core + 3 TS-specific)
- [ ] Real test execution (npm test / jest / vitest)
- [ ] Mutation score calculation accurate
- [ ] CLI `pmat analyze mutate --path file.ts` functional

### Quality Requirements ✅
- [ ] Test coverage ≥85% for new modules
- [ ] Property tests for all operators
- [ ] Integration tests with real TypeScript projects
- [ ] Mutation score ≥80% on TypeScript mutation module itself
- [ ] All RED tests passing in GREEN phase

### Performance Requirements ✅
- [ ] Mutation generation <5s for 1000-line TypeScript files
- [ ] Test execution with proper timeout handling
- [ ] No memory leaks (validate with valgrind or similar)

### Documentation Requirements ✅
- [ ] TypeScript mutation operator guide
- [ ] CLI usage examples for TypeScript
- [ ] Integration examples with CI/CD
- [ ] Comparison with other TS mutation tools (Stryker.js)

---

## Risks & Mitigation

### Risk 1: Test Framework Detection
**Impact:** High - Can't run tests if framework unknown
**Mitigation:**
- Support top 5 frameworks: jest, vitest, mocha, ava, tap
- Fallback to `npm test` if framework unclear
- Provide `--test-cmd` override flag

### Risk 2: Tree-Sitter TypeScript Grammar Limitations
**Impact:** Medium - Some TS syntax might not parse
**Mitigation:**
- Test with real-world TypeScript codebases
- Document known limitations
- Provide regex-based fallback for unsupported syntax

### Risk 3: Performance on Large Files
**Impact:** Medium - Tree-sitter parsing can be slow
**Mitigation:**
- Incremental parsing for repeated mutations
- Caching AST between mutants
- Parallel mutation generation

### Risk 4: Achieving 80% Mutation Score
**Impact:** Medium - Self-dogfooding may reveal gaps
**Mitigation:**
- Start with comprehensive tests in RED phase
- Iterate on test improvements in REFACTOR phase
- Document equivalent mutants as acceptable survivors

---

## Dependencies

### Internal Dependencies
- ✅ Mutation engine (`server/src/services/mutation/engine.rs`)
- ✅ ML mutation predictor (`server/src/services/mutation/ml_predictor.rs`)
- ✅ Pattern learning system (PMAT-7009 - optional, can defer)

### External Dependencies
- ✅ tree-sitter (already in Cargo.toml with `typescript-ast` feature)
- ✅ tree-sitter-typescript (already in deps)
- ⏳ npm/node.js (runtime requirement for test execution)

### Blockers
- None - all dependencies available

---

## Testing Strategy

### RED Phase Testing (Days 1-2)
1. **Unit Tests:** Each operator has 5+ failing tests
2. **Property Tests:** Syntax preservation, operator coverage
3. **Integration Tests:** End-to-end with fixtures
4. **Expected:** 100% test failure rate

### GREEN Phase Testing (Days 3-5)
1. **Make tests pass:** Minimal implementation
2. **Validate:** All RED tests now green
3. **Coverage:** ≥85% on new modules

### REFACTOR Phase Testing (Days 6-7)
1. **Mutation Testing:** Dogfood on TypeScript mutation module
2. **Performance Testing:** Benchmarks for 1000-line files
3. **Regression Testing:** Ensure Rust mutation still works
4. **Final Validation:** Mutation score ≥80%

---

## Deliverables Checklist

### Code ✅
- [ ] `TreeSitterMutationOperator` trait
- [ ] TypeScript-specific mutation operators (9+)
- [ ] Real test execution (`run_tests()`)
- [ ] Integration with mutation engine
- [ ] All tests passing (RED → GREEN)
- [ ] Mutation score ≥80% (REFACTOR)

### Tests ✅
- [ ] Unit tests (600+ lines)
- [ ] Property tests (200+ lines)
- [ ] Integration tests (400+ lines)
- [ ] Mutation tests (self-dogfooding)

### Documentation ✅
- [ ] TypeScript mutation operator guide
- [ ] CLI usage examples
- [ ] Comparison with Stryker.js
- [ ] Migration guide for existing users

### Validation ✅
- [ ] Benchmarked against external TypeScript projects
- [ ] Performance meets <5s requirement
- [ ] Mutation score ≥80% achieved
- [ ] All quality gates passed

---

## Post-MVP Enhancements (Deferred to Sprint 26)

### Phase 2: JavaScript Support (1-2 days)
- Ensure .js/.jsx files work (currently covered by TypeScript parser)
- Add JavaScript-specific operators (e.g., var → let/const mutations)

### Phase 3: Advanced TypeScript Features (2-3 days)
- Generics mutations (<T> → <unknown>)
- Interface mutations (property removal)
- Type guard mutations (is checks)
- Decorator mutations

### Phase 4: React/Vue-Specific Mutations (3-4 days)
- JSX element mutations
- Hook dependency mutations (useEffect)
- Props mutations

---

## Related Tickets

- **PMAT-7004:** ML Mutation Predictor (✅ Complete) - Will enhance with TS patterns
- **PMAT-7009:** Pattern Learning System (⏳ In Progress) - Will learn from TS mutations
- **PMAT-7007:** Sub-Agent Scaffolding (⏳ In Progress) - MutationTester sub-agent will use this
- **PMAT-6012:** Roadmap Generation (✅ Complete) - Referenced mutation testing gaps

---

## References

### Specifications
- [Mutant-Fuzz-AST Testing Spec](../specifications/mutant-fuzz-ast-testing.md)
- [Mutation Testing Status](../../MUTATION_TESTING_STATUS.md)
- [Multi-Language Mutation Testing](../specifications/multi-language-mutation.md)

### Existing Code
- [Rust Mutation Adapter](../../server/src/services/mutation/rust_adapter.rs)
- [ML Mutation Predictor](../../server/src/services/mutation/ml_predictor.rs)
- [Mutation Engine](../../server/src/services/mutation/engine.rs)

### External Tools (For Comparison)
- [Stryker.js](https://stryker-mutator.io/) - TypeScript mutation testing
- [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) - Rust mutation testing

---

**Created:** 2025-10-08
**Last Updated:** 2025-10-08
**Status:** Ready for RED phase implementation
**Next Steps:** Begin RED phase - write all failing tests
