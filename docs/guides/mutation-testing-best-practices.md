# Mutation Testing Best Practices

Practical guidance for effective mutation testing with PMAT.

## Table of Contents

- [When to Use Mutation Testing](#when-to-use-mutation-testing)
- [Team Adoption Strategy](#team-adoption-strategy)
- [Setting Quality Thresholds](#setting-quality-thresholds)
- [Workflow Integration](#workflow-integration)
- [Performance Optimization](#performance-optimization)
- [Interpreting Results](#interpreting-results)
- [Common Pitfalls](#common-pitfalls)
- [Multi-Language Projects](#multi-language-projects)
- [CI/CD Best Practices](#cicd-best-practices)
- [Measuring Success](#measuring-success)

---

## When to Use Mutation Testing

### Ideal Use Cases

#### 1. Critical Business Logic
Mutation testing is most valuable for code with high business impact.

**Example: Payment Processing**
```rust
fn calculate_discount(price: f64, loyalty_points: i32) -> f64 {
    if loyalty_points > 1000 {  // Business rule
        price * 0.9
    } else {
        price
    }
}
```

**Why**: A subtle bug in discount logic could cost thousands of dollars. Mutation testing ensures tests catch edge cases like `loyalty_points == 1000`.

**Recommended Threshold**: 95-100%

---

#### 2. Security-Critical Code
Authentication, authorization, cryptography, input validation.

**Example: Authentication**
```python
def is_admin(user):
    if user.role == "admin" and user.is_active:
        return True
    return False
```

**Why**: Security vulnerabilities have severe consequences. Mutation testing ensures tests verify both conditions (`role` AND `is_active`).

**Recommended Threshold**: 95-100%

---

#### 3. Complex Algorithms
Code with non-obvious behavior, multiple edge cases, or intricate logic.

**Example: Binary Search**
```typescript
function binarySearch(arr: number[], target: number): number {
  let left = 0;
  let right = arr.length - 1;

  while (left <= right) {
    const mid = Math.floor((left + right) / 2);
    if (arr[mid] === target) {
      return mid;
    } else if (arr[mid] < target) {
      left = mid + 1;
    } else {
      right = mid - 1;
    }
  }
  return -1;
}
```

**Why**: Off-by-one errors are common. Mutation testing catches boundary issues like `left < right` vs `left <= right`.

**Recommended Threshold**: 85-95%

---

### When NOT to Use Mutation Testing

#### 1. Simple Getters/Setters
```rust
struct User {
    name: String,
}

impl User {
    fn name(&self) -> &str {
        &self.name
    }
}
```

**Why**: Low complexity, low risk. Mutation testing adds minimal value.

---

#### 2. UI Layout Code
```typescript
function renderButton(props: ButtonProps) {
  return <button style={{color: props.color}}>{props.label}</button>;
}
```

**Why**: Visual correctness requires human judgment, not automated mutation testing.

---

#### 3. Generated Code
Auto-generated files (protobufs, GraphQL schemas, migrations).

**Why**: Not manually maintained. Bugs are fixed in generators, not generated code.

---

#### 4. Glue Code
Simple integration code with no business logic.

```python
def save_user(user):
    db.users.insert(user)
```

**Why**: Tests integration, not logic. Integration tests are more appropriate.

---

### Decision Matrix

| Code Type | Mutation Testing | Code Coverage | Integration Tests |
|-----------|------------------|---------------|-------------------|
| Business Logic | ✅ Critical | ✅ Required | ⚠️ Optional |
| Security | ✅ Critical | ✅ Required | ✅ Required |
| Algorithms | ✅ Important | ✅ Required | ⚠️ Optional |
| Utilities | ⚠️ Optional | ✅ Required | ⚠️ Optional |
| UI Components | ❌ Skip | ⚠️ Optional | ✅ Required |
| Getters/Setters | ❌ Skip | ⚠️ Optional | ❌ Skip |

---

## Team Adoption Strategy

### Phase 1: Pilot (Weeks 1-2)

**Goal**: Prove value on 1-2 critical modules

**Actions**:
1. Select 2 high-impact modules (e.g., payment processing, authentication)
2. Run mutation testing locally
3. Document survived mutants and add missing test cases
4. Measure mutation score improvement

**Example**:
```bash
# Week 1: Baseline measurement
pmat mutate --target src/auth/ --output-format json > week1.json
# Mutation score: 72%

# Add missing tests based on survived mutants

# Week 2: Remeasure
pmat mutate --target src/auth/ --output-format json > week2.json
# Mutation score: 91% (+19%)
```

**Success Criteria**:
- Mutation score improved by 15%+
- Found at least 3 real test gaps
- Team understands mutation testing value

---

### Phase 2: Expansion (Weeks 3-4)

**Goal**: Integrate into CI/CD for critical modules

**Actions**:
1. Add mutation testing to CI pipeline for pilot modules
2. Set conservative thresholds (70-80%)
3. Train team on interpreting results

**GitHub Actions Example**:
```yaml
name: Mutation Testing (Critical Modules)
on: [pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install PMAT
        run: cargo install pmat
      - name: Mutation test auth module
        run: pmat mutate --target src/auth/ --threshold 80 --failures-only
      - name: Mutation test payment module
        run: pmat mutate --target src/payment/ --threshold 80 --failures-only
```

**Success Criteria**:
- CI pipeline runs mutation testing on every PR
- Developers understand how to fix survived mutants
- No major CI/CD slowdown (<5 minutes added)

---

### Phase 3: Full Adoption (Weeks 5-8)

**Goal**: Mutation testing across entire codebase

**Actions**:
1. Expand to all business logic modules
2. Increase thresholds to 85%+
3. Add mutation score to PR comments
4. Establish team ownership

**Success Criteria**:
- 80%+ of critical code has mutation testing
- Average mutation score >85%
- Team proactively writes tests to kill mutants

---

### Common Adoption Challenges

#### Challenge 1: "Mutation testing is too slow"

**Solution**: Use differential testing (test only changed files)

```bash
# Get changed files in PR
CHANGED=$(git diff --name-only origin/main...HEAD | grep '\.rs$')

# Test only changed files
for file in $CHANGED; do
    pmat mutate --target "$file" --failures-only
done
```

**Result**: 10× faster (2 minutes instead of 20 minutes)

---

#### Challenge 2: "Too many false positives"

**Solution**: Focus on survived mutants, not killed ones

```bash
pmat mutate --target src/ --failures-only
```

**Result**: Shows only actionable issues (survived mutants)

---

#### Challenge 3: "Developers don't know how to fix survived mutants"

**Solution**: Create team guidelines with examples

**Example Guideline**:
```markdown
## Fixing Survived Mutants

### Survived Mutant: `> to >=`
**Location**: src/auth.rs:42
**Original**: `if age > 18`
**Mutated**: `if age >= 18`

**Root Cause**: Missing boundary test

**Fix**: Add test case
```rust
#[test]
fn test_exactly_18_years_old() {
    assert!(is_adult(18));  // Tests boundary
}
```
```

---

## Setting Quality Thresholds

### Threshold Recommendations by Code Type

| Code Type | Initial Threshold | Target Threshold | Rationale |
|-----------|-------------------|------------------|-----------|
| **Security** | 90% | 95-100% | Zero tolerance for vulnerabilities |
| **Business Logic** | 80% | 85-95% | High impact, but some equivalent mutants expected |
| **Algorithms** | 75% | 85-90% | Complex logic requires thorough testing |
| **Utilities** | 70% | 80-85% | Lower risk, but still valuable |
| **UI Components** | N/A | N/A | Use visual regression testing instead |

---

### Gradual Threshold Increases

**Anti-Pattern**: Setting 90% threshold immediately
```yaml
# ❌ BAD: Too aggressive
- run: pmat mutate --target src/ --threshold 90
```

**Best Practice**: Start low, increase gradually
```yaml
# ✅ GOOD: Gradual improvement
# Week 1-2: 70% threshold
- run: pmat mutate --target src/ --threshold 70

# Week 3-4: 75% threshold
- run: pmat mutate --target src/ --threshold 75

# Week 5-8: 80% threshold
- run: pmat mutate --target src/ --threshold 80
```

**Rationale**: Allows team to learn without blocking all PRs.

---

### Module-Specific Thresholds

Different modules have different requirements.

**Example**: Tiered thresholds
```bash
# Critical: 95% threshold
pmat mutate --target src/auth/ --threshold 95

# Important: 85% threshold
pmat mutate --target src/api/ --threshold 85

# Utilities: 75% threshold
pmat mutate --target src/utils/ --threshold 75
```

**Makefile Example**:
```makefile
.PHONY: mutation-test-all
mutation-test-all: mutation-auth mutation-api mutation-utils

mutation-auth:
	pmat mutate --target src/auth/ --threshold 95 --failures-only

mutation-api:
	pmat mutate --target src/api/ --threshold 85 --failures-only

mutation-utils:
	pmat mutate --target src/utils/ --threshold 75 --failures-only
```

---

## Workflow Integration

### Local Development Workflow

#### Pre-Commit Hook
Catch issues before they enter version control.

`.git/hooks/pre-commit`:
```bash
#!/bin/bash

echo "Running mutation testing on staged files..."

STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -z "$STAGED_FILES" ]; then
    exit 0
fi

for file in $STAGED_FILES; do
    echo "Testing $file..."
    pmat mutate --target "$file" --threshold 80 --failures-only

    if [ $? -ne 0 ]; then
        echo "❌ Mutation testing failed for $file"
        echo "Fix survived mutants before committing"
        exit 1
    fi
done

echo "✅ All mutation tests passed"
exit 0
```

**Benefits**:
- Catches test gaps before code review
- Enforces quality at earliest stage
- Faster feedback loop

---

### Pull Request Workflow

#### 1. Run Mutation Testing in CI
```yaml
# .github/workflows/mutation-test.yml
name: Mutation Testing
on: [pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Fetch all history for diff

      - name: Install PMAT
        run: cargo install pmat

      - name: Get changed files
        id: changed-files
        run: |
          CHANGED=$(git diff --name-only origin/${{ github.base_ref }}...HEAD | grep '\.rs$' | tr '\n' ' ')
          echo "files=$CHANGED" >> $GITHUB_OUTPUT

      - name: Run mutation tests on changed files
        run: |
          for file in ${{ steps.changed-files.outputs.files }}; do
            pmat mutate --target "$file" --output-format json > mutation-$file.json
          done

      - name: Generate report
        run: |
          pmat mutate --target src/ --output-format markdown > mutation-report.md

      - name: Comment on PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

---

#### 2. Require Mutation Score in PR Checklist

**PR Template** (`.github/pull_request_template.md`):
```markdown
## Checklist

- [ ] All tests pass
- [ ] Code coverage >80%
- [ ] **Mutation score >85%** ⚠️ Critical
- [ ] No survived mutants in critical code
```

---

### Code Review Integration

Reviewers should check mutation testing results:

**Review Checklist**:
1. ✅ Mutation score meets threshold?
2. ✅ Survived mutants reviewed?
3. ✅ New tests added to kill mutants?
4. ✅ Critical code has 90%+ mutation score?

**Example PR Comment**:
```markdown
## Mutation Testing Review

✅ Overall mutation score: 87% (above 85% threshold)

⚠️ Found 2 survived mutants in `src/auth.rs`:
1. Line 42: `> to >=` - Please add boundary test
2. Line 58: `&& to ||` - Missing test for both conditions

Please add tests to kill these mutants before approval.
```

---

## Performance Optimization

### Problem: Mutation Testing Takes Too Long

**Symptom**: CI pipeline takes 30+ minutes

**Solutions**:

#### 1. Differential Testing (Test Only Changed Files)
```bash
# Full codebase: 30 minutes
pmat mutate --target src/  # ❌ Slow

# Changed files only: 2 minutes
CHANGED=$(git diff --name-only origin/main...HEAD | grep '\.rs$')
for file in $CHANGED; do
    pmat mutate --target "$file"
done  # ✅ Fast
```

**Speedup**: 15×

---

#### 2. Parallel Execution
```bash
# Sequential: 20 minutes
pmat mutate --target src/ --jobs 1  # ❌ Slow

# Parallel (4 cores): 6 minutes
pmat mutate --target src/ --jobs 4  # ✅ Fast
```

**Speedup**: 3-4×

---

#### 3. Shorter Timeouts
```bash
# 60-second timeout: 20 minutes
pmat mutate --target src/ --timeout 60  # ❌ Slow

# 10-second timeout: 8 minutes
pmat mutate --target src/ --timeout 10  # ✅ Fast
```

**Speedup**: 2-3× (if tests are fast)

---

#### 4. Failures-Only Mode
```bash
# Show all mutants: 100 MB logs
pmat mutate --target src/  # ❌ Large output

# Show only failures: 5 MB logs
pmat mutate --target src/ --failures-only  # ✅ Compact
```

**Benefit**: Faster CI log parsing, less noise

---

#### 5. Caching
```yaml
# GitHub Actions caching
- name: Cache PMAT binary
  uses: actions/cache@v3
  with:
    path: ~/.cargo/bin/pmat
    key: ${{ runner.os }}-pmat-${{ hashFiles('**/Cargo.lock') }}

- name: Install PMAT (if not cached)
  run: |
    if [ ! -f ~/.cargo/bin/pmat ]; then
      cargo install pmat
    fi
```

**Speedup**: Saves 2-5 minutes on PMAT installation

---

#### 6. Scheduled Full Runs
Run comprehensive mutation testing nightly, not on every PR.

```yaml
name: Nightly Mutation Testing
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily

jobs:
  full-mutation-test:
    runs-on: ubuntu-latest
    steps:
      - name: Full codebase mutation testing
        run: pmat mutate --target src/ --threshold 85 --output-format json

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: mutation-results
          path: mutation-results.json
```

**Benefit**: Comprehensive testing without blocking PRs

---

## Interpreting Results

### Understanding Mutation Statuses

| Status | Meaning | Action Required |
|--------|---------|-----------------|
| **Killed** | Test suite detected mutation | ✅ Good! No action needed |
| **Survived** | Test suite did NOT detect mutation | ⚠️ Add test to kill mutant |
| **CompileError** | Mutation created invalid syntax | ℹ️ Ignore (excluded from score) |
| **Timeout** | Test took too long | ⚠️ Check for infinite loops |

---

### Analyzing Survived Mutants

#### Example 1: Boundary Condition

**Survived Mutant**:
```
src/lib.rs:15:9 - Changed > to >=
Original: if age > 18
Mutated:  if age >= 18
```

**Root Cause**: Missing boundary test

**Fix**: Add test for `age == 18`
```rust
#[test]
fn test_exactly_18_years_old() {
    assert!(is_adult(18));  // Tests age == 18 case
}
```

---

#### Example 2: Logical Operator

**Survived Mutant**:
```
src/auth.rs:42:5 - Changed && to ||
Original: if user.is_admin && user.is_active
Mutated:  if user.is_admin || user.is_active
```

**Root Cause**: Test doesn't verify both conditions

**Fix**: Add test for partial conditions
```rust
#[test]
fn test_admin_but_inactive() {
    let user = User { is_admin: true, is_active: false };
    assert!(!has_access(user));  // Should fail (inactive)
}
```

---

#### Example 3: Return Value

**Survived Mutant**:
```
src/math.rs:10:5 - Changed return value
Original: return Some(result)
Mutated:  return None
```

**Root Cause**: Test doesn't assert return value

**Fix**: Add assertion
```rust
#[test]
fn test_division_returns_result() {
    let result = divide(10, 2);
    assert_eq!(result, Some(5));  // Verifies return value
}
```

---

### Equivalent Mutants

Some mutants are **behaviorally equivalent** to the original code.

**Example**:
```rust
// Original
fn abs(x: i32) -> i32 {
    if x < 0 {
        return -x;
    }
    return x;
}

// Mutation: Change second return to 0
fn abs(x: i32) -> i32 {
    if x < 0 {
        return -x;
    }
    return 0;  // Equivalent if x >= 0 always returns x
}
```

**How to Handle**:
1. Verify the mutant is truly equivalent
2. Document why it can't be killed
3. Accept slightly lower mutation score (90-95% instead of 100%)

---

## Common Pitfalls

### Pitfall 1: Chasing 100% Mutation Score

**Anti-Pattern**: Writing brittle tests just to kill mutants

```rust
// ❌ BAD: Brittle test that checks implementation details
#[test]
fn test_implementation_details() {
    let result = calculate(5);
    assert_eq!(result.intermediate_value, 10);  // Exposes internals
}
```

**Best Practice**: Accept 85-95% mutation score as excellent

---

### Pitfall 2: Ignoring Equivalent Mutants

**Anti-Pattern**: Spending hours trying to kill equivalent mutants

**Best Practice**: Document equivalent mutants and move on

```rust
// EQUIVALENT MUTANT DOCUMENTED:
// Line 42: `return x` to `return 0` - Equivalent when x is always 0
// Cannot kill without brittle test. Mutation score: 94% (acceptable)
fn identity_zero(x: i32) -> i32 {
    return x;
}
```

---

### Pitfall 3: Testing Implementation, Not Behavior

**Anti-Pattern**: Tests that verify internal state

```rust
// ❌ BAD: Tests internal state
#[test]
fn test_internal_counter() {
    let obj = MyObject::new();
    obj.do_work();
    assert_eq!(obj.internal_counter, 1);  // Implementation detail
}
```

**Best Practice**: Test public behavior

```rust
// ✅ GOOD: Tests behavior
#[test]
fn test_work_completed() {
    let obj = MyObject::new();
    obj.do_work();
    assert!(obj.is_work_done());  // Public API
}
```

---

### Pitfall 4: Running Mutation Testing on Every Commit

**Anti-Pattern**: Slowing down CI pipeline

```yaml
# ❌ BAD: Runs on every commit (slow)
on: [push, pull_request]
```

**Best Practice**: Run on PRs only, or nightly

```yaml
# ✅ GOOD: Runs on PRs (fast feedback)
on: [pull_request]

# ✅ GOOD: Comprehensive nightly run
on:
  schedule:
    - cron: '0 2 * * *'
```

---

## Multi-Language Projects

### Consistent Thresholds Across Languages

**Example Project**: Backend (Rust) + Frontend (TypeScript)

**Backend (Rust)**: 90% threshold
```bash
pmat mutate --target backend/src/ --threshold 90 --language rust
```

**Frontend (TypeScript)**: 80% threshold
```bash
pmat mutate --target frontend/src/ --threshold 80 --language typescript
```

**Rationale**: Backend has more critical logic than frontend

---

### Unified Reporting

Aggregate results from multiple languages:

```bash
# Run mutation testing for all languages
pmat mutate --target backend/src/ --output-format json > backend-results.json
pmat mutate --target frontend/src/ --output-format json > frontend-results.json

# Aggregate results
jq -s '{
    total_score: (.[0].mutation_score + .[1].mutation_score) / 2,
    backend: .[0].mutation_score,
    frontend: .[1].mutation_score
}' backend-results.json frontend-results.json
```

**Output**:
```json
{
  "total_score": 85.0,
  "backend": 90.0,
  "frontend": 80.0
}
```

---

## CI/CD Best Practices

### 1. Fail Fast
Exit immediately if mutation score is too low.

```yaml
- name: Mutation testing with threshold
  run: pmat mutate --target src/ --threshold 85
  # Exits with code 1 if score < 85%
```

---

### 2. Cache Dependencies
Speed up CI by caching PMAT binary.

```yaml
- name: Cache PMAT
  uses: actions/cache@v3
  with:
    path: ~/.cargo/bin/pmat
    key: pmat-${{ hashFiles('**/Cargo.lock') }}
```

---

### 3. Artifact Storage
Store mutation results for debugging.

```yaml
- name: Upload mutation results
  uses: actions/upload-artifact@v3
  with:
    name: mutation-results
    path: mutation-results.json
    retention-days: 30
```

---

### 4. Status Badges
Display mutation score in README.

**GitHub Actions Badge**:
```markdown
![Mutation Score](https://img.shields.io/badge/mutation-87%25-brightgreen)
```

---

## Measuring Success

### Key Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Mutation Score** | 85%+ | `pmat mutate --target src/` |
| **Test Gap Reduction** | 50%+ improvement | Compare survived mutants week-over-week |
| **CI/CD Performance** | <5 minutes added | Measure pipeline time before/after |
| **Bug Escape Rate** | 30%+ reduction | Track production bugs after adoption |

---

### Success Story Template

**Before Mutation Testing**:
- Mutation Score: 68%
- 15 survived mutants in critical code
- 3 production bugs per month from missing test cases

**After Mutation Testing** (8 weeks):
- Mutation Score: 89% (+21%)
- 2 survived mutants in critical code (-87%)
- 1 production bug per month (-67%)

**ROI**: Prevented 16 bugs in 8 months = $80,000 saved (assuming $5,000 per bug)

---

## Summary Checklist

### Getting Started
- [ ] Identify 1-2 critical modules for pilot
- [ ] Run mutation testing locally
- [ ] Document survived mutants
- [ ] Add missing tests

### Team Adoption
- [ ] Add mutation testing to CI/CD
- [ ] Set initial threshold (70-80%)
- [ ] Train team on interpreting results
- [ ] Create team guidelines

### Continuous Improvement
- [ ] Gradually increase thresholds (5% per month)
- [ ] Monitor mutation score trends
- [ ] Optimize performance (differential testing, parallel execution)
- [ ] Measure bug escape rate reduction

---

## Additional Resources

- **User Guide**: [Mutation Testing with PMAT](./mutation-testing.md)
- **API Reference**: [CLI API Reference](./mutation-testing-api-reference.md)
- **CI/CD Integration**:
  - [GitHub Actions Integration](../ci-cd/github-actions-integration.md)
  - [GitLab CI Integration](../ci-cd/gitlab-ci-integration.md)
  - [Jenkins Integration](../ci-cd/jenkins-integration.md)
- **Example Projects**:
  - [Rust Mutation Testing Example](../../examples/rust-mutation-testing/)
  - [Python Mutation Testing Example](../../examples/python-mutation-testing/)
  - [TypeScript Mutation Testing Example](../../examples/typescript-mutation-testing/)

---

**Version**: v2.177.0
**Last Updated**: October 28, 2025
**Sprint**: Sprint 64 Day 3
