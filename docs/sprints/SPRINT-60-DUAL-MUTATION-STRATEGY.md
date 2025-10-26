# Sprint 60: Dual Mutation Testing Strategy
## PMAT Built-in + cargo-mutants Comparison

**Date**: October 26, 2025
**Version**: v2.173.0
**Sprint Type**: Quality Enhancement - Dual Mutation Testing
**Status**: 🚀 IN PROGRESS

## Executive Summary

Leverage **both** PMAT's built-in mutation testing AND cargo-mutants for comprehensive mutation analysis. This dual approach provides:

1. **PMAT Mutation Testing** - Tree-sitter based, multi-language, ML-powered
2. **cargo-mutants** - Rust-native, compilation-based, industry standard
3. **Comparison Analysis** - Validate both tools, identify strengths/weaknesses

## Dual Mutation Testing Architecture

### Tool Comparison Matrix

| Feature | PMAT Mutation Testing | cargo-mutants |
|---------|----------------------|---------------|
| **Approach** | Tree-sitter AST mutations | Rust source code mutations |
| **Languages** | Rust, Python, TypeScript, JavaScript, Go, C++, Java, Scala | Rust only |
| **Speed** | Fast (AST-level, no compilation) | Slower (requires compilation per mutant) |
| **Mutation Operators** | 50+ operators (custom, tree-sitter based) | Standard Rust mutators (proven) |
| **ML Integration** | ✅ ML predictor for mutant prioritization | ❌ No ML |
| **Coverage Integration** | ✅ Deep coverage analysis | ⚠️  Basic coverage |
| **Distributed** | ✅ Multi-worker support | ❌ Single-threaded |
| **Equivalent Detection** | ✅ ML-based equivalent mutant detection | ⚠️  Heuristic-based |
| **Industry Standard** | ❌ PAIML custom | ✅ Widely adopted |

### Strategy: Use Both Tools

**PMAT Mutation Testing**:
- Fast iteration during development
- Multi-language codebase testing
- ML-powered mutant prioritization
- Ideal for CI/CD (fast feedback)

**cargo-mutants**:
- Validation benchmark (industry standard)
- Rust-specific deep testing
- Final quality gate before release
- Slower but comprehensive

## PMAT Mutation Testing Usage

### 1. Via Sub-Agent (AI-Assisted)

**Command**:
```bash
pmat scaffold agent --name mutation-tester
cd mutation-tester
./pmat-agent @mutation-tester test server/src/utils/path_validator.rs
```

**Features**:
- AI-powered mutant generation
- Automatic test execution
- Intelligent result analysis
- Markdown report generation

### 2. Via MCP Tool (Programmatic)

**MCP Tool**: `mutation_test`

**Input Schema**:
```json
{
  "path": "server/src/",
  "target_file": "server/src/utils/path_validator.rs",
  "timeout": 60,
  "max_mutants": 100,
  "ml_prioritize": true
}
```

**Output**:
```json
{
  "total_mutants": 45,
  "caught": 40,
  "missed": 5,
  "timeout": 0,
  "score": 88.9,
  "ml_predictions": {
    "high_value_mutants": 15,
    "equivalent_mutants_detected": 3
  }
}
```

### 3. Via CLI (Direct)

**Commands**:

```bash
# Quick mutation test (path_validator)
pmat analyze mutation \
  --file server/src/utils/path_validator.rs \
  --timeout 60 \
  --format json

# Full server mutation test (distributed)
pmat analyze mutation \
  --path server/src/ \
  --workers 8 \
  --ml-prioritize \
  --output mutation_report.json

# Specific module mutation test
pmat analyze mutation \
  --file server/src/tdg/calculator.rs \
  --operators "arithmetic,logical,relational" \
  --timeout 30
```

**Available Mutation Operators** (PMAT):
- `arithmetic`: +, -, *, /, %
- `logical`: &&, ||, !
- `relational`: ==, !=, <, >, <=, >=
- `bitwise`: &, |, ^, <<, >>
- `assignment`: =, +=, -=, *=
- `constant`: 0, 1, -1, true, false, null
- `control-flow`: if/else, loop, return
- `method-call`: replace method calls
- `variable`: swap variables

### 4. PMAT Mutation Testing Architecture

**Multi-Language Support**:
```
server/src/services/mutation/
├── rust_tree_sitter_mutations.rs       # Rust AST mutations
├── python_tree_sitter_mutations.rs     # Python AST mutations
├── typescript_tree_sitter_mutations.rs # TypeScript AST mutations
├── go_tree_sitter_mutations.rs         # Go AST mutations
├── cpp_tree_sitter_mutations.rs        # C++ AST mutations
├── ml_predictor.rs                      # ML-based mutant prioritization
├── equivalent_detector.rs               # Equivalent mutant detection
├── distributed.rs                       # Multi-worker execution
├── coverage.rs                          # Coverage-guided mutation
└── fuzzing.rs                           # Mutation + fuzzing hybrid
```

**ML Predictor** (`ml_predictor.rs`):
- Predicts which mutants are most likely to be caught
- Prioritizes high-value mutants for faster feedback
- Detects equivalent mutants (saves test time)
- Uses historical mutation data for training

**Equivalent Mutant Detection** (`equivalent_detector.rs`):
- Semantic equivalence analysis
- AST diff + data flow analysis
- ML-based prediction (reduces false positives)
- Saves 10-30% test execution time

## cargo-mutants Usage

### 1. Quick Mutation Test (Single File)

```bash
# Test path_validator (5-10 minutes)
cargo mutants \
  --manifest-path server/Cargo.toml \
  --file server/src/utils/path_validator.rs \
  --timeout 60 \
  --output mutants.out

# View results
cat mutants.out/mutants.txt
```

### 2. Comprehensive Mutation Test (Full Workspace)

```bash
# Full server mutation test (30-60 minutes)
cargo mutants \
  --manifest-path server/Cargo.toml \
  --workspace \
  --timeout 120 \
  --jobs 8 \
  --output mutants_full.out

# Generate HTML report
cargo mutants \
  --manifest-path server/Cargo.toml \
  --workspace \
  --timeout 120 \
  --output mutants.out \
  --html
```

### 3. Targeted Mutation Test (Module)

```bash
# Test TDG calculator module
cargo mutants \
  --manifest-path server/Cargo.toml \
  --file server/src/tdg/calculator.rs \
  --timeout 60

# Test MCP integration
cargo mutants \
  --manifest-path server/Cargo.toml \
  --file server/src/mcp_integration/java_tools.rs \
  --timeout 90
```

## Dual Strategy Workflow

### Phase 1: PMAT Fast Iteration (Daily)

**During Development** (fast feedback):
```bash
# Run PMAT mutation test on changed files (2-5 minutes)
pmat analyze mutation \
  --file server/src/utils/path_validator.rs \
  --ml-prioritize \
  --workers 4 \
  --timeout 30

# Output:
# - 45 mutants generated
# - 40 caught (88.9% mutation score)
# - 3 equivalent mutants detected (ML)
# - 2 high-value mutants missed → write tests
```

**CI/CD Pipeline** (on every PR):
```bash
# PMAT mutation test (5-minute budget)
pmat analyze mutation \
  --path server/src/utils/ \
  --ml-prioritize \
  --workers 8 \
  --timeout 300 \
  --min-score 75 \
  --fail-below-threshold

# Exit code 0 if score >= 75%, else 1
```

### Phase 2: cargo-mutants Validation (Weekly)

**Before Release** (comprehensive validation):
```bash
# cargo-mutants full test (60 minutes)
cargo mutants \
  --manifest-path server/Cargo.toml \
  --file server/src/utils/path_validator.rs \
  --file server/src/tdg/calculator.rs \
  --file server/src/mcp_integration/java_tools.rs \
  --timeout 120 \
  --output mutants_release.out

# Compare with PMAT results
./scripts/compare_mutation_results.sh \
  pmat_mutations.json \
  mutants_release.out/mutants.txt
```

### Phase 3: Comparison Analysis (Sprint Review)

**Metrics to Compare**:
1. **Mutation Score**: PMAT vs cargo-mutants (should be within 5%)
2. **Mutants Caught**: Which tool finds more gaps?
3. **False Positives**: Equivalent mutants detected correctly?
4. **Execution Time**: PMAT (fast) vs cargo-mutants (thorough)
5. **Unique Mutants**: What does each tool find that the other misses?

**Example Comparison Report**:
```markdown
## Mutation Testing Comparison: path_validator.rs

| Metric | PMAT | cargo-mutants | Delta |
|--------|------|---------------|-------|
| **Mutants Generated** | 45 | 38 | +7 (PMAT more operators) |
| **Mutants Caught** | 40 (88.9%) | 32 (84.2%) | +4.7% (PMAT ML prioritization) |
| **Mutants Missed** | 5 (11.1%) | 6 (15.8%) | -4.7% (PMAT better) |
| **Equivalent Detected** | 3 (ML-based) | 1 (heuristic) | +2 (PMAT ML advantage) |
| **Execution Time** | 2m 15s | 8m 42s | 3.9x faster (PMAT) |
| **Unique Mutants** | 12 (tree-sitter ops) | 5 (Rust-specific) | PMAT more creative |

### Analysis:
- PMAT finds more mutants (tree-sitter flexibility)
- PMAT faster (AST-level, no compilation)
- cargo-mutants finds Rust-specific edge cases
- Both tools complement each other
```

## High-Value Mutation Targets (Both Tools)

### 1. Path Validator (`server/src/utils/path_validator.rs`)
**Why Critical**: Security (path traversal attacks)
**PMAT Command**:
```bash
pmat analyze mutation --file server/src/utils/path_validator.rs --timeout 60
```
**cargo-mutants Command**:
```bash
cargo mutants --file server/src/utils/path_validator.rs --timeout 60
```
**Expected Mutants**: 40-50
**Target Score**: 95%+ (security-critical)

### 2. TDG Calculator (`server/src/tdg/calculator.rs`)
**Why Critical**: Core business logic (complexity scoring)
**PMAT Command**:
```bash
pmat analyze mutation --file server/src/tdg/calculator.rs --ml-prioritize
```
**cargo-mutants Command**:
```bash
cargo mutants --file server/src/tdg/calculator.rs --timeout 90
```
**Expected Mutants**: 60-80
**Target Score**: 90%+

### 3. Java/Scala MCP Tools (`server/src/mcp_integration/*_tools.rs`)
**Why Critical**: User-facing API (high visibility)
**PMAT Command**:
```bash
pmat analyze mutation \
  --file server/src/mcp_integration/java_tools.rs \
  --file server/src/mcp_integration/scala_tools.rs \
  --workers 4
```
**cargo-mutants Command**:
```bash
cargo mutants \
  --file server/src/mcp_integration/java_tools.rs \
  --file server/src/mcp_integration/scala_tools.rs \
  --timeout 120
```
**Expected Mutants**: 50-70 each
**Target Score**: 85%+

### 4. Polyglot AST (`server/src/ast/polyglot/language_mapper.rs`)
**Why Critical**: Complex logic (cross-language analysis)
**PMAT Command** (multi-language advantage):
```bash
pmat analyze mutation \
  --file server/src/ast/polyglot/language_mapper.rs \
  --operators "all" \
  --ml-prioritize
```
**cargo-mutants Command**:
```bash
cargo mutants \
  --file server/src/ast/polyglot/language_mapper.rs \
  --timeout 90
```
**Expected Mutants**: 80-100
**Target Score**: 80%+

### 5. AST Parsers (`server/src/services/ast/languages/*.rs`)
**Why Critical**: Core functionality (multi-language support)
**PMAT Command** (best suited - tree-sitter native):
```bash
pmat analyze mutation \
  --path server/src/services/ast/languages/ \
  --workers 8 \
  --ml-prioritize \
  --timeout 600
```
**cargo-mutants Command** (Rust files only):
```bash
cargo mutants \
  --file server/src/services/ast/languages/rust.rs \
  --timeout 120
```
**Expected Mutants**: 200-300 (all languages)
**Target Score**: 80%+

## Makefile Integration (Dual Approach)

```makefile
# PMAT mutation testing (fast, daily use)
test-mutation-pmat-quick:
	@echo "🧬 Running PMAT mutation tests (fast)..."
	@pmat analyze mutation \
		--file server/src/utils/path_validator.rs \
		--ml-prioritize \
		--workers 4 \
		--timeout 300 \
		--format json > pmat_mutations.json
	@echo "✅ PMAT mutation tests completed! Score: $$(jq '.score' pmat_mutations.json)%"

test-mutation-pmat-full:
	@echo "🧬 Running comprehensive PMAT mutation tests..."
	@pmat analyze mutation \
		--path server/src/ \
		--ml-prioritize \
		--workers 8 \
		--timeout 1800 \
		--format json > pmat_mutations_full.json
	@echo "✅ Full PMAT mutation tests completed!"

# cargo-mutants (thorough, weekly validation)
test-mutation-cargo-quick:
	@echo "🧬 Running cargo-mutants (5 critical files)..."
	@cargo mutants \
		--manifest-path server/Cargo.toml \
		--file server/src/utils/path_validator.rs \
		--file server/src/tdg/calculator.rs \
		--file server/src/mcp_integration/java_tools.rs \
		--file server/src/mcp_integration/scala_tools.rs \
		--file server/src/ast/polyglot/language_mapper.rs \
		--timeout 120 \
		--output mutants_quick.out
	@echo "✅ cargo-mutants quick test completed! Results: mutants_quick.out/"

test-mutation-cargo-full:
	@echo "🧬 Running comprehensive cargo-mutants (30-60 minutes)..."
	@cargo mutants \
		--manifest-path server/Cargo.toml \
		--workspace \
		--timeout 120 \
		--jobs 8 \
		--output mutants_full.out
	@echo "✅ Full cargo-mutants test completed! Results: mutants_full.out/"

# Dual strategy: Run both and compare
test-mutation-dual:
	@echo "🧬🧬 Running dual mutation testing (PMAT + cargo-mutants)..."
	@$(MAKE) test-mutation-pmat-quick
	@$(MAKE) test-mutation-cargo-quick
	@./scripts/compare_mutation_results.sh pmat_mutations.json mutants_quick.out/mutants.txt
	@echo "✅ Dual mutation testing completed! Comparison report generated."

# CI target (5-minute budget)
test-mutation-ci:
	@echo "🧬 Running mutation tests for CI (PMAT only, 5-minute budget)..."
	@timeout 300 pmat analyze mutation \
		--path server/src/utils/ \
		--ml-prioritize \
		--workers 8 \
		--min-score 75 \
		--fail-below-threshold \
		--format json || { echo "❌ Mutation score below 75%"; exit 1; }
	@echo "✅ CI mutation tests passed!"
```

## Comparison Script

**File**: `scripts/compare_mutation_results.sh`

```bash
#!/bin/bash
# Compare PMAT and cargo-mutants results

PMAT_JSON=$1
CARGO_MUTANTS_TXT=$2

echo "## Mutation Testing Comparison"
echo ""
echo "| Metric | PMAT | cargo-mutants |"
echo "|--------|------|---------------|"

# Extract scores
PMAT_SCORE=$(jq '.score' "$PMAT_JSON")
PMAT_TOTAL=$(jq '.total_mutants' "$PMAT_JSON")
PMAT_CAUGHT=$(jq '.caught' "$PMAT_JSON")
PMAT_MISSED=$(jq '.missed' "$PMAT_JSON")

CARGO_CAUGHT=$(grep -c "CAUGHT" "$CARGO_MUTANTS_TXT")
CARGO_MISSED=$(grep -c "NOT CAUGHT" "$CARGO_MUTANTS_TXT")
CARGO_TOTAL=$((CARGO_CAUGHT + CARGO_MISSED))
CARGO_SCORE=$(echo "scale=1; 100 * $CARGO_CAUGHT / $CARGO_TOTAL" | bc)

echo "| Total Mutants | $PMAT_TOTAL | $CARGO_TOTAL |"
echo "| Caught | $PMAT_CAUGHT ($PMAT_SCORE%) | $CARGO_CAUGHT ($CARGO_SCORE%) |"
echo "| Missed | $PMAT_MISSED | $CARGO_MISSED |"
echo ""

# Calculate delta
DELTA=$(echo "$PMAT_SCORE - $CARGO_SCORE" | bc)
echo "**Delta**: PMAT is ${DELTA}% different from cargo-mutants"

# Recommend action
if (( $(echo "$PMAT_SCORE < 75" | bc -l) )); then
    echo ""
    echo "⚠️  **Action Required**: Mutation score below 75% threshold"
    echo "   → Write tests for missed mutants"
fi
```

## Sprint 60 Execution Plan

### Week 1: PMAT Mutation Testing

**Day 1-2**: Setup & Baseline
- Run PMAT mutation tests on 5 critical modules
- Document baseline mutation scores
- Identify gaps in test coverage

**Day 3-4**: Test Improvements
- Write tests for missed mutants
- Re-run PMAT mutation tests
- Measure improvement

**Day 5**: CI Integration
- Add `test-mutation-ci` to GitHub Actions
- Set 75% threshold
- Validate on PR

### Week 2: cargo-mutants Validation

**Day 6-7**: cargo-mutants Baseline
- Run cargo-mutants on same 5 modules
- Compare with PMAT results
- Document differences

**Day 8-9**: Cross-Tool Analysis
- Identify unique mutants from each tool
- Write tests for mutants missed by both
- Re-run both tools

**Day 10**: Reporting & Documentation
- Generate comparison report
- Update CLAUDE.md with dual strategy
- Sprint retrospective

## Success Metrics

**Coverage Improvement**:
- Mutation Score (PMAT): 65% → 80% (target)
- Mutation Score (cargo-mutants): 60% → 75% (target)
- Test Suite Size: 5,052 → 5,150 tests (+100 tests)

**Tool Comparison**:
- PMAT vs cargo-mutants delta: <10% (validation)
- Unique mutants caught by each: Documented
- Execution time ratio: PMAT 3-5x faster

**CI/CD Integration**:
- PMAT mutation tests in CI: ✅ (5-minute budget)
- cargo-mutants in release pipeline: ✅ (60-minute budget)
- Dual strategy documented: ✅

---

**Generated**: 2025-10-26
**Author**: Claude Code (Sonnet 4.5)
**Version**: pmat 2.173.0
**Status**: 🚀 IN PROGRESS
