# Sprint 25 Plan: Dogfooding Mutation Testing on PMAT

**Sprint Duration**: 1-2 weeks
**Target Release**: v2.155.0
**Start Date**: 2025-10-09
**Focus**: Internal Quality Validation Through Self-Testing

---

## Sprint Goals

### Primary Objective
**Use PMAT's new multi-language mutation testing capabilities to test PMAT itself!**

This sprint focuses on **dogfooding** - using our own tools to validate and improve PMAT's code quality.

### Success Criteria
- ✅ Mutation testing run on 5+ core PMAT modules
- ✅ Achieve 80%+ mutation score on tested modules
- ✅ Identify and fix 10+ test gaps revealed by surviving mutants
- ✅ Document case study for mutation testing best practices
- ✅ Add mutation testing to CI pipeline (optional)

---

## Motivation: Why Dogfooding?

### Toyota Way Principle: "Build Quality In"

By testing PMAT with PMAT, we:
1. **Validate** our mutation operators work on real-world code
2. **Discover** bugs and test gaps in our own codebase
3. **Improve** test quality through concrete examples
4. **Demonstrate** mutation testing value to users
5. **Iterate** on operator design based on real feedback

### Marketing Value

**Case Study:** "How PMAT Achieved 85% Mutation Score Using Its Own Tools"
- Real-world validation
- Concrete before/after metrics
- Actionable insights for users
- Credibility through self-application

---

## Scope

### Phase 1: Module Selection (Day 1)

**Criteria for Testing:**
- ✅ Critical to PMAT functionality
- ✅ Good existing test coverage (>70%)
- ✅ Pure Rust (leverage our fastest mutation generator)
- ✅ Not too large (<1000 LOC per module)
- ✅ Representative of different code patterns

**Recommended Modules (Pick 5-8):**

1. **`services/mutation/types.rs`** (Core mutation types)
   - LOC: ~300
   - Existing coverage: ~90%
   - Why: Foundation of mutation testing system

2. **`services/mutation/operators.rs`** (Mutation operators)
   - LOC: ~400
   - Existing coverage: ~85%
   - Why: Core logic for mutation generation

3. **`services/mutation/scoring.rs`** (Mutation score calculation)
   - LOC: ~200
   - Existing coverage: ~95%
   - Why: Critical metrics calculation

4. **`services/mutation/equivalent_detector.rs`** (Equivalent mutant detection)
   - LOC: ~350
   - Existing coverage: ~80%
   - Why: Complex pattern matching logic

5. **`services/complexity/metrics.rs`** (Complexity calculation)
   - LOC: ~500
   - Existing coverage: ~85%
   - Why: Mathematical algorithms

6. **`services/dead_code/analyzer.rs`** (Dead code detection)
   - LOC: ~400
   - Existing coverage: ~75%
   - Why: AST traversal patterns

7. **`cli/handlers/analyze_handler.rs`** (CLI analyze command)
   - LOC: ~600
   - Existing coverage: ~70%
   - Why: Real-world integration logic

8. **`scaffold/agent/generator.rs`** (Agent scaffolding)
   - LOC: ~450
   - Existing coverage: ~80%
   - Why: Template rendering and validation

### Phase 2: Baseline Testing (Days 2-3)

**For Each Module:**
1. Run existing test suite
2. Measure line coverage (using `cargo llvm-cov`)
3. Document current test count and assertions
4. Establish baseline mutation score (expected: 50-70% initially)

**Deliverable:** Baseline metrics document

### Phase 3: Mutation Testing (Days 4-7)

**For Each Module:**
1. Generate mutants using `RustMutationGenerator`
2. Run test suite against each mutant
3. Calculate mutation score
4. Identify surviving mutants
5. Analyze test gaps
6. Document findings

**Workflow:**
```rust
use pmat::services::mutation::RustMutationGenerator;

// Read module source
let source = std::fs::read_to_string("src/services/mutation/types.rs")?;

// Generate mutants
let generator = RustMutationGenerator::with_default_operators();
let mutants = generator.generate_mutants(&source, "types.rs")?;

println!("Generated {} mutants", mutants.len());

// Test each mutant (automated)
for mutant in &mut mutants {
    // Write mutant, run `cargo test`, capture result
    mutant.status = test_mutant(&mutant)?;
}

// Calculate score
let score = MutationScore::from_results(&mutants);
println!("Mutation Score: {:.1}%", score.score * 100.0);
```

**Deliverable:** Per-module mutation reports

### Phase 4: Test Improvement (Days 8-11)

**For Each Surviving Mutant:**
1. Analyze why it survived
2. Determine if it's:
   - **Real test gap** → Add test
   - **Equivalent mutant** → Mark as equivalent
   - **Uncovered edge case** → Add edge case test
   - **Invalid mutant** → Improve operator

**Target:** Improve mutation score from ~60% to 80%+

**Example Test Gap:**
```rust
// Surviving Mutant: `+` mutated to `-` in score calculation
// Original: killed / (total - equivalent)
// Mutant: killed - (total - equivalent)

// Missing Test (ADD THIS):
#[test]
fn test_mutation_score_calculation_operator() {
    let results = vec![
        killed_mutant(),
        killed_mutant(),
        survived_mutant(),
    ];

    let score = MutationScore::from_results(&results);

    // This test would catch the + → - mutation
    assert_eq!(score.killed, 2);
    assert_eq!(score.total, 3);
    assert!((score.score - 0.666).abs() < 0.01);

    // Edge case: ensure division, not subtraction
    assert!(score.score > 0.0 && score.score <= 1.0);
}
```

**Deliverable:** 20-40 new tests added across modules

### Phase 5: Documentation (Days 12-13)

**Create Case Study Document:**
- `docs/case-studies/PMAT-SELF-TESTING.md`
- Before/after metrics
- Test gaps discovered
- Improvements made
- Lessons learned
- Best practices

**Update Guides:**
- Add dogfooding example to RUST-MUTATION-TESTING.md
- Update testing documentation with mutation testing section
- Create mutation testing CI/CD guide

### Phase 6: CI Integration (Day 14 - Optional)

**Add to GitHub Actions:**
```yaml
name: Mutation Testing

on:
  pull_request:
    paths:
      - 'server/src/services/mutation/**'
      - 'server/src/services/complexity/**'

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1

      - name: Run Mutation Testing
        run: |
          cargo run --example rust_mutation_workflow --features rust-ast -- \
            --module services/mutation/types.rs

      - name: Check Mutation Score
        run: |
          SCORE=$(grep "Mutation Score:" mutation_output.txt | awk '{print $3}' | tr -d '%')
          if [ "$SCORE" -lt 80 ]; then
            echo "❌ Mutation score $SCORE% below 80% threshold"
            exit 1
          fi
```

---

## Deliverables

### Code Artifacts
1. **New Tests**: 20-40 tests across 5-8 modules
2. **Bug Fixes**: Any bugs discovered through mutation testing
3. **Operator Improvements**: Refinements to Rust mutation operators
4. **CI Workflow**: GitHub Actions for mutation testing (optional)

### Documentation
1. **Case Study**: `docs/case-studies/PMAT-SELF-TESTING.md` (~5,000 words)
2. **Metrics Report**: Per-module mutation testing results
3. **Best Practices**: Lessons learned from dogfooding
4. **Guide Updates**: Enhanced mutation testing documentation

### Metrics
1. **Before/After Mutation Scores**: Per module comparison
2. **Test Count**: Before/after test counts
3. **Coverage**: Line coverage improvements
4. **Bug Count**: Bugs discovered and fixed

---

## Expected Outcomes

### Quantitative
- **Mutation Score**: 60% → 80%+ (average across modules)
- **Test Count**: +20-40 tests
- **Line Coverage**: +5-10% on tested modules
- **Bugs Found**: 3-10 real bugs
- **Equivalent Mutants**: 5-15% of total

### Qualitative
- **Confidence**: Higher confidence in PMAT's core logic
- **Validation**: Real-world proof of mutation testing value
- **Insights**: Practical lessons for improving operators
- **Marketing**: Compelling case study for users

---

## Risk Management

### Potential Issues

1. **Low Initial Scores** (<50%)
   - **Mitigation**: Expected for first run, focus on improvement
   - **Response**: Use as motivation to improve tests

2. **Too Many Equivalent Mutants**
   - **Mitigation**: Refine operators based on patterns
   - **Response**: Add equivalent mutant detection patterns

3. **Performance Issues** (mutation testing takes too long)
   - **Mitigation**: Start with smaller modules
   - **Response**: Optimize test execution, parallel testing

4. **Difficult Surviving Mutants**
   - **Mitigation**: Not all mutants need to be killed
   - **Response**: Document as acceptable edge cases

---

## Timeline

### Week 1: Testing & Analysis

**Days 1-3** (Module Selection + Baseline)
- [ ] Select 5-8 modules for testing
- [ ] Run baseline test suites
- [ ] Measure current line coverage
- [ ] Establish baseline metrics

**Days 4-7** (Mutation Testing)
- [ ] Generate mutants for each module
- [ ] Run mutation testing workflows
- [ ] Calculate mutation scores
- [ ] Analyze surviving mutants
- [ ] Categorize test gaps

### Week 2: Improvement & Documentation

**Days 8-11** (Test Improvement)
- [ ] Write tests for real test gaps
- [ ] Mark equivalent mutants
- [ ] Add edge case tests
- [ ] Improve mutation operators if needed
- [ ] Re-run mutation testing
- [ ] Verify score improvements

**Days 12-13** (Documentation)
- [ ] Write case study document
- [ ] Create metrics visualization
- [ ] Update mutation testing guides
- [ ] Document best practices

**Day 14** (Optional CI Integration)
- [ ] Create GitHub Actions workflow
- [ ] Test CI integration
- [ ] Document CI setup

---

## Success Metrics

### Must-Have (MVP)
- ✅ 5+ modules tested with mutation testing
- ✅ 20+ new tests added
- ✅ Case study document complete
- ✅ Average mutation score 70%+

### Should-Have
- ✅ 8 modules tested
- ✅ 40+ new tests added
- ✅ Average mutation score 80%+
- ✅ 5+ real bugs found and fixed

### Nice-to-Have
- ✅ CI integration complete
- ✅ Average mutation score 85%+
- ✅ All surviving mutants analyzed
- ✅ Blog post ready for publication

---

## Post-Sprint Actions

### v2.155.0 Release
1. Version bump
2. Release notes highlighting:
   - Improved test quality
   - Mutation score achievements
   - Bugs fixed through dogfooding
3. Case study publication

### Community Engagement
1. Reddit/HN post: "We tested our mutation testing tool with itself"
2. Twitter thread with before/after metrics
3. Blog post: Technical deep-dive

### Future Work
1. Expand to more modules (Sprint 26)
2. Add mutation testing to regular CI (all PRs)
3. Continuous mutation score monitoring
4. Mutation testing dashboard

---

## Notes

### Why Rust First?
- **Performance**: Fastest mutation generation (~3ms)
- **Self-Contained**: PMAT is written in Rust
- **Validation**: Prove Rust operators work on real code
- **Dogfooding**: Use what we built

### Why Not All Languages?
- Focus on quality over quantity
- Rust is PMAT's primary language
- Other languages tested via examples/fixtures
- Can expand in future sprints

### Learning Objectives
1. How effective are our mutation operators on real code?
2. What patterns do real surviving mutants reveal?
3. How can we improve test quality systematically?
4. What is a realistic mutation score target?
5. How long does mutation testing take on production code?

---

## References

- [RUST-MUTATION-TESTING.md](../features/RUST-MUTATION-TESTING.md)
- [Toyota Way Quality Principles](../toyota-way-tdd-pattern-library.md)
- [PMAT Testing Strategy](../testing/STRATEGY.md)
- Sprint 24 completion (Multi-language mutation testing)

---

**Created**: 2025-10-09
**Status**: Ready for Sprint 25
**Next Review**: End of Week 1 (2025-10-16)

---

🦀 **Let's test PMAT with PMAT and prove mutation testing works!** 🦀
