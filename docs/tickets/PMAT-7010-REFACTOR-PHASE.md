# PMAT-7010 REFACTOR Phase - TypeScript Mutation Testing

**Ticket:** TypeScript/JavaScript AST-Based Mutation Testing (Priority 0)
**Phase:** REFACTOR - Integration, Optimization, Production Readiness
**Started:** 2025-10-08
**Status:** 🔵 **IN PROGRESS**

---

## Overview

With GREEN phase complete (5 mutation operators, 11+ mutants generated, zero compilation errors), the REFACTOR phase focuses on:

1. **Real test execution** - Run jest/vitest on mutants
2. **Mutation score calculation** - Quantify test quality
3. **Performance optimization** - <5s generation time
4. **ML integration** - Survivability prediction
5. **Production readiness** - Documentation, examples, polish

---

## Success Criteria

### Must Have (Phase Complete)
- [ ] Execute real tests on TypeScript mutants
- [ ] Calculate mutation score (killed/survived ratio)
- [ ] Achieve >80% mutation score on calculator.ts fixture
- [ ] Generation time <5s for 50+ mutants
- [ ] Integration with TypeScriptAdapter complete
- [ ] End-to-end CLI workflow working

### Should Have (Production Quality)
- [ ] Parallel mutant generation
- [ ] Mutation caching for incremental runs
- [ ] ML predictor integration
- [ ] User documentation
- [ ] Example project

### Nice to Have (Future Enhancement)
- [ ] Mutation visualization
- [ ] HTML reports
- [ ] CI/CD integration examples
- [ ] Multi-file mutation support

---

## Phase 1: Test Execution Integration (Day 1)

### Goal
Execute real npm tests on generated mutants and calculate mutation score.

### Tasks

#### 1.1 Connect Generator to TypeScriptAdapter
```rust
// In typescript_adapter.rs
impl LanguageAdapter for TypeScriptAdapter {
    fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>> {
        // Currently returns syn-based operators (wrong!)
        // Change to return tree-sitter operators
        use super::typescript_tree_sitter_mutations::*;
        vec![
            Box::new(TypeScriptBinaryOpMutation),
            Box::new(TypeScriptStrictEqualityMutation),
            Box::new(TypeScriptOptionalChainingMutation),
            Box::new(TypeScriptNullishCoalescingMutation),
            Box::new(TypeScriptAsyncAwaitMutation),
        ]
    }
}
```

**Challenge:** MutationOperator trait vs TreeSitterMutationOperator trait mismatch

**Solution:** Create adapter or extend MutationOperator to support tree-sitter

#### 1.2 Test Execution Pipeline
```rust
// Workflow:
1. Generate mutants from source file
2. For each mutant:
   a. Write mutated source to temp file
   b. Run npm test
   c. Parse test results (pass/fail)
   d. Record mutant status (killed/survived)
3. Calculate mutation score
```

#### 1.3 Validate with Fixtures
```bash
cd fixtures/typescript
npm install  # Install vitest
cargo run -- analyze mutate --path calculator.ts
# Expected: Mutation score >80%
```

### Acceptance Criteria
- [ ] Mutants run against real vitest tests
- [ ] Mutation score calculated correctly
- [ ] At least 80% of mutants killed by tests
- [ ] Test output parsed correctly
- [ ] Failed tests show which mutant survived

---

## Phase 2: Performance Optimization (Day 2)

### Goal
Achieve <5s mutation generation for 50+ mutants.

### Current Baseline
- 11 mutants from 490 bytes: ~instantaneous
- Need to test with larger files (500+ LOC)

### Optimization Strategies

#### 2.1 Parallel Mutant Generation
```rust
use rayon::prelude::*;

mutants.par_iter().map(|mutant| {
    // Generate and test in parallel
}).collect()
```

#### 2.2 AST Parsing Cache
```rust
// Parse once, reuse for all operators
let tree = parse_typescript(source)?; // Parse once
for operator in operators {
    operator.mutate(&tree, source); // Reuse AST
}
```

#### 2.3 Incremental Mutation
- Cache previously generated mutants
- Only regenerate changed functions
- Use file modification time

### Acceptance Criteria
- [ ] 100+ mutants generated in <5s
- [ ] Parallel generation working
- [ ] AST parsing cached efficiently
- [ ] Benchmark results documented

---

## Phase 3: ML Integration (Day 3)

### Goal
Connect SurvivabilityPredictor to prioritize high-value mutants.

### Integration Points

#### 3.1 Feature Extraction
```rust
// Extract features from mutant
let features = extract_mutant_features(&mutant);
// - Operator type
// - Code complexity at location
// - Test coverage of line
// - Historical survival rate
```

#### 3.2 Prediction
```rust
let predictor = SurvivabilityPredictor::load()?;
let score = predictor.predict(&features)?;

// Prioritize low-confidence mutants (harder to kill)
mutants.sort_by(|a, b| {
    predictor.predict(a).cmp(predictor.predict(b))
});
```

#### 3.3 Learning Loop
```rust
// After test execution
for mutant in tested_mutants {
    predictor.learn(
        &mutant.features,
        mutant.status == MutantStatus::Killed
    );
}
predictor.save()?;
```

### Acceptance Criteria
- [ ] Predictor integrated with TypeScript mutants
- [ ] Features extracted correctly
- [ ] Prediction accuracy >70%
- [ ] Learning loop updating model

---

## Phase 4: Production Polish (Day 4)

### Goal
Production-ready TypeScript mutation testing.

### Tasks

#### 4.1 CLI Integration
```bash
# Simple usage
pmat mutate calculator.ts

# With options
pmat mutate calculator.ts \
    --operators AOR,ROR \
    --mutation-score 80 \
    --parallel

# Output
Mutation Testing Results:
  Total Mutants: 45
  Killed: 38 (84.4%)
  Survived: 7 (15.6%)
  Mutation Score: 84.4% ✅

Surviving Mutants:
  1. calculator.ts:19 (=== → ==) - Weak equality test
  2. calculator.ts:34 (?? → ||) - Missing nullish test
  ...
```

#### 4.2 Documentation
- User guide: "TypeScript Mutation Testing with PMAT"
- API docs: TreeSitterMutationOperator
- Example project: React component mutation testing
- Best practices: Writing mutation-resistant tests

#### 4.3 Example Project
```
examples/typescript-react/
  src/
    Button.tsx
    Button.test.tsx
  package.json
  README.md
  .pmat.yml
```

### Acceptance Criteria
- [ ] CLI workflow documented
- [ ] User guide complete
- [ ] Example project works
- [ ] API documentation published

---

## Technical Design Decisions

### Decision 1: Trait Unification

**Problem:** MutationOperator vs TreeSitterMutationOperator

**Options:**
1. Keep separate traits, create adapter
2. Extend MutationOperator with optional tree-sitter support
3. Replace MutationOperator with unified trait

**Decision:** Option 1 - Keep separate, create adapter
- Least disruptive
- Preserves existing Rust mutation operators
- Clean separation of concerns

### Decision 2: Test Execution Strategy

**Problem:** How to run tests on mutants?

**Options:**
1. Write temp file, run npm test, parse stdout
2. Programmatic test execution via Node.js
3. Use test framework APIs directly

**Decision:** Option 1 - Temp file + npm test
- Simple, reliable
- Works with any test framework
- Easy to debug

### Decision 3: Mutation Score Calculation

**Formula:**
```
Mutation Score = (Killed Mutants / Total Mutants) × 100%

Where:
- Killed = Tests failed (mutant detected)
- Survived = Tests passed (mutant not detected)
- Timeout/Error = Configurable (count as killed or exclude)
```

---

## Implementation Plan

### Day 1: Test Execution
1. Create TreeSitterMutationOperator → MutationOperator adapter
2. Update TypeScriptAdapter.mutation_operators()
3. Implement mutant test execution
4. Calculate mutation score
5. Test with fixtures/typescript/calculator.ts

### Day 2: Performance
1. Benchmark current performance
2. Implement parallel generation
3. Add AST caching
4. Re-benchmark and document

### Day 3: ML Integration
1. Extract features from TypeScript mutants
2. Connect SurvivabilityPredictor
3. Test prediction accuracy
4. Implement learning loop

### Day 4: Polish
1. CLI integration
2. Write documentation
3. Create example project
4. Final testing and validation

---

## Risk Mitigation

### Risk 1: Test Framework Compatibility
**Mitigation:** Support jest, vitest, mocha via auto-detection

### Risk 2: Performance Bottleneck
**Mitigation:** Profile early, optimize hot paths, parallel execution

### Risk 3: ML Accuracy Too Low
**Mitigation:** Start with simple features, iterate based on data

### Risk 4: Integration Complexity
**Mitigation:** Keep adapter layer simple, maintain separation

---

## Quality Gates

Before marking REFACTOR complete:
- [ ] All tests pass
- [ ] Mutation score >80% on fixtures
- [ ] Performance <5s for 50+ mutants
- [ ] Documentation complete
- [ ] Example working
- [ ] Zero compilation warnings
- [ ] Code reviewed and cleaned

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Mutation Score | >80% | fixtures/typescript/calculator.ts |
| Generation Time | <5s | 50+ mutants |
| Test Execution | <2s/mutant | Average across all mutants |
| ML Accuracy | >70% | Predicted vs actual survival |
| Code Coverage | >90% | Unit tests for new code |

---

## Related Documentation

- `PMAT-7010-GREEN-PHASE-DAY5-COMPLETE.md` - GREEN phase summary
- `PMAT-7010-RED-PHASE-RESULTS.md` - RED phase tests
- `TICKET-PMAT-7010.md` - Original specification

---

**Phase:** REFACTOR
**Priority:** 0 (Critical path)
**Estimated Duration:** 4 days
**Started:** 2025-10-08

