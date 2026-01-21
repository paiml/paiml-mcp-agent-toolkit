# File Health Score Specification v1.0

**Version**: 1.0.0
**Date**: 2026-01-21
**Status**: Draft
**Author**: PAIML Engineering Team

## Executive Summary

This specification defines a **File Health Score** system to detect, prevent, and remediate excessively large source files. Large files (>500 lines) are empirically correlated with lower testability, higher defect density, and increased cognitive load. The system integrates file size limits, test-to-lines ratio (TLR), and complexity scoring into a unified health metric enforced through `pmat comply`.

> "A class should have only one reason to change." — Robert C. Martin, Single Responsibility Principle

### Problem Statement

**Current State (PMAT Codebase - 2026-01-21):**

| File | Lines | Status |
|------|-------|--------|
| `analysis_utilities.rs` | 12,087 | CRITICAL |
| `deep_context.rs` | 7,211 | CRITICAL |
| `commands.rs` | 6,273 | CRITICAL |
| `tools.rs` | 6,111 | CRITICAL |
| `refactor_auto_handlers.rs` | 5,203 | CRITICAL |
| `command_dispatcher.rs` | 4,954 | CRITICAL |
| ... | ... | ... |

**30+ files exceed 2,000 lines.** These files are:
- Untestable: TLR ratios below 0.1
- Unmaintainable: High cognitive load
- Defect-prone: Change amplification

### Toyota Way Principles

| Principle | Application |
|-----------|-------------|
| **Jidoka** (Built-in Quality) | Pre-commit hook blocks large file creation |
| **Genchi Genbutsu** (Go and See) | Measure actual file sizes, not estimates |
| **Kaizen** (Continuous Improvement) | Ratchet mechanism forces gradual reduction |
| **Muda** (Waste Elimination) | Large files create cognitive waste |
| **Andon Cord** | Stop the line when thresholds exceeded |

---

## 1. Scientific Foundation

### 1.1 Empirical Evidence for File Size Limits

**Primary Research (Peer-Reviewed):**

1. **Hindle, A., et al. (2008)**. "Reading Beside the Lines: Indentation as a Proxy for Complexity." *MSR '08*. DOI: 10.1145/1370750.1370786
   - Finding: Files >500 lines exhibit exponential increase in defect density
   - Recommendation: "Keep modules small and focused"

2. **Nagappan, N., Ball, T., & Zeller, A. (2006)**. "Mining Metrics to Predict Component Failures." *ICSE '06*. DOI: 10.1145/1134285.1134349
   - Finding: File size is a leading indicator of post-release defects
   - Correlation: r=0.67 between LOC and defect count

3. **Bird, C., et al. (2011)**. "Don't Touch My Code! Examining the Effects of Ownership on Software Quality." *FSE '11*. DOI: 10.1145/2025113.2025119
   - Finding: Large files have diffuse ownership, correlating with bugs
   - Threshold: Files >400 LOC show ownership fragmentation

4. **Zimmermann, T., & Nagappan, N. (2008)**. "Predicting Defects Using Network Analysis on Dependency Graphs." *ICSE '08*. DOI: 10.1145/1368088.1368161
   - Finding: Highly coupled large files are defect hotspots
   - Recommendation: Module size under 500 LOC

5. **Rahman, F., & Devanbu, P. (2013)**. "How, and Why, Process Metrics Are Better." *ICSE '13*. DOI: 10.1109/ICSE.2013.6606589
   - Finding: Churn in large files predicts defects better than size alone
   - TLR insight: Test coverage matters more in large files

### 1.2 Test-to-Lines Ratio (TLR) Research

6. **Mockus, A., et al. (2009)**. "Experiences from Replicating a Case Study to Investigate the Effects of Testing on Quality." *ESEM '09*.
   - Finding: Projects with TLR ≥ 0.5 have 60% fewer post-release defects
   - Threshold: 1:1 test-to-code ratio for critical modules

7. **Athanasiou, D., et al. (2014)**. "Test Code Quality and Its Relation to Issue Handling Performance." *IEEE TSE*, 40(11).
   - Finding: High TLR correlates with faster defect resolution
   - Recommendation: TLR ≥ 0.7 for files >300 LOC

8. **Parsai, A., et al. (2018)**. "How Do Code Coverage-Based Criteria Compare to Mutation Testing." *ICST '18*.
   - Finding: TLR is a better proxy for test quality than raw coverage
   - Mutation insight: High TLR files have higher mutation scores

### 1.3 Cognitive Load Research

9. **Sweller, J. (1988)**. "Cognitive Load During Problem Solving." *Cognitive Science*, 12(2), 257-285.
   - Theory: Working memory limited to 7±2 chunks
   - Application: Large files exceed cognitive chunking capacity

10. **Hermans, F. (2021)**. *The Programmer's Brain*. Manning Publications.
    - Finding: Files >500 lines cause "cognitive overload"
    - Long-term memory cannot retain context for large files

### 1.4 Industry Standards

11. **Martin, R. C. (2008)**. *Clean Code: A Handbook of Agile Software Craftsmanship*. Prentice Hall.
    - Guideline: Functions ≤20 lines, files ≤200-500 lines
    - Principle: "The first rule of functions is that they should be small"

12. **Google Engineering Practices (2024)**. "Code Health: Reducing Technical Debt."
    - Guideline: Files should generally be <500 lines
    - Exception: Generated code with `_generated` suffix

13. **Linux Kernel Style Guide (2024)**. Documentation/process/coding-style.rst
    - Guideline: Functions ≤50 lines, excessive file size discouraged
    - Rationale: "If a function exceeds a screen-full, you're probably doing something wrong"

---

## 2. Metrics and Thresholds

### 2.1 File Size Thresholds

| Lines | Classification | Action | Rationale |
|-------|---------------|--------|-----------|
| 0-200 | **Ideal** | None | Optimal cognitive chunk |
| 201-500 | **Acceptable** | None | Within SRP tolerance |
| 501-1000 | **Warning** | Soft block | Approaching limit |
| 1001-2000 | **Problem** | Hard block (new files) | Exceeds cognitive capacity |
| >2000 | **Critical** | Hard block + Kaizen plan | Untestable monolith |

### 2.2 Test-to-Lines Ratio (TLR)

**Formula:**
```
TLR = (lines_of_test_code) / (lines_of_source_code)
```

**Where test lines include:**
- Inline `#[cfg(test)]` modules
- Corresponding `*_test.rs` / `*_tests.rs` files
- Test files in `tests/` directory matching module name

**Scaling TLR Requirements:**

| Source Lines | Required TLR | Rationale |
|--------------|--------------|-----------|
| 0-100 | ≥ 0.3 | Basic edge case coverage |
| 101-300 | ≥ 0.5 | Moderate complexity |
| 301-500 | ≥ 0.7 | High complexity, many paths |
| 501-1000 | ≥ 1.0 | 1:1 minimum (shouldn't exist) |
| >1000 | ≥ 1.5 | Must over-test (splitting required) |

**Rationale for Scaling:**
Large files have exponentially more:
- Edge cases (O(n) input combinations)
- State combinations (O(2^n) for booleans)
- Integration points
- Hidden coupling

### 2.3 File Health Score Formula

**Composite Score (0-100):**

```rust
pub fn calculate_health_score(
    lines: usize,
    test_lines: usize,
    avg_cyclomatic: f32,
    churn_30d: usize,  // commits touching this file in last 30 days
) -> u8 {
    // Size Score (30 points max)
    let size_score = match lines {
        0..=200 => 30,
        201..=500 => 25,
        501..=1000 => 15,
        1001..=2000 => 5,
        _ => 0,  // >2000 lines = 0 points
    };

    // TLR Score (40 points max)
    let required_tlr = required_tlr_for_size(lines);
    let actual_tlr = test_lines as f32 / lines.max(1) as f32;
    let tlr_ratio = (actual_tlr / required_tlr).min(1.0);
    let tlr_score = (tlr_ratio * 40.0) as u8;

    // Complexity Score (20 points max)
    let complexity_score = match avg_cyclomatic {
        c if c <= 5.0 => 20,
        c if c <= 10.0 => 15,
        c if c <= 15.0 => 10,
        c if c <= 20.0 => 5,
        _ => 0,  // >20 average cyclomatic
    };

    // Stability Score (10 points max)
    let stability_score = match churn_30d {
        0..=2 => 10,   // Stable
        3..=5 => 7,    // Moderate
        6..=10 => 4,   // Volatile
        _ => 0,        // Hot file
    };

    size_score + tlr_score + complexity_score + stability_score
}

fn required_tlr_for_size(lines: usize) -> f32 {
    match lines {
        0..=100 => 0.3,
        101..=300 => 0.5,
        301..=500 => 0.7,
        501..=1000 => 1.0,
        _ => 1.5,
    }
}
```

### 2.4 Grade Assignment

| Score | Grade | Status | Action |
|-------|-------|--------|--------|
| 90-100 | A | Excellent | None |
| 80-89 | B | Good | None |
| 70-79 | C | Acceptable | Warning |
| 60-69 | D | Poor | Soft block |
| 50-59 | E | Critical | Hard block |
| 0-49 | F | Failing | Hard block + Kaizen plan |

---

## 3. Enforcement Mechanisms

### 3.1 Pre-Commit Hook

```bash
#!/bin/bash
# .pmat/hooks/pre-commit-file-health

set -e

# Check new/modified files
for file in $(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$'); do
    lines=$(wc -l < "$file")

    # Hard block: New files over 500 lines
    if [ "$lines" -gt 500 ]; then
        if ! git ls-files --error-unmatch "$file" &>/dev/null; then
            echo "❌ BLOCKED: New file $file has $lines lines (max: 500)"
            exit 1
        fi
    fi

    # Ratchet check: Existing files cannot grow
    if git ls-files --error-unmatch "$file" &>/dev/null; then
        baseline=$(git show HEAD:"$file" 2>/dev/null | wc -l || echo 0)
        if [ "$lines" -gt "$baseline" ]; then
            echo "❌ BLOCKED: $file grew from $baseline to $lines lines"
            echo "   Files can only shrink or stay the same (ratchet mechanism)"
            exit 1
        fi
    fi
done

echo "✅ File health check passed"
```

### 3.2 Ratchet Mechanism

**Principle:** Files can never get bigger, only smaller or unchanged.

**Baseline File (`.pmat/file-health-baseline.json`):**

```json
{
  "version": "1.0",
  "generated": "2026-01-21T00:00:00Z",
  "files": {
    "src/cli/analysis_utilities.rs": {
      "lines": 12087,
      "test_lines": 245,
      "tlr": 0.02,
      "health": 12,
      "status": "critical"
    },
    "src/services/deep_context.rs": {
      "lines": 7211,
      "test_lines": 189,
      "tlr": 0.03,
      "health": 15,
      "status": "critical"
    }
  }
}
```

**Ratchet Rules:**

1. **New files:** Must have health score ≥70 and lines ≤500
2. **Existing files:** Lines can only decrease or stay same
3. **Touching large files:** Must improve health score by ≥1 point
4. **Critical files:** Cannot be modified without a Kaizen plan

### 3.3 `pmat comply` Integration

```bash
# Check file health
pmat comply --file-health

# Detailed report
pmat comply --file-health --verbose

# JSON output for CI
pmat comply --file-health --format json

# Fail if any file is critical
pmat comply --file-health --min-health 50
```

**Example Output:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
File Health Report - paiml-mcp-agent-toolkit
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Summary:
  Files analyzed: 287
  Total lines: 142,891
  Average health: 67/100 (D)

Critical Files (health < 50):

  Score │ File                            │ Lines  │ Tests │  TLR  │ CC
  ──────┼─────────────────────────────────┼────────┼───────┼───────┼─────
    12  │ cli/analysis_utilities.rs       │ 12,087 │   245 │ 0.02  │ 18.3
    15  │ services/deep_context.rs        │  7,211 │   189 │ 0.03  │ 22.1
    18  │ cli/commands.rs                 │  6,273 │   156 │ 0.02  │ 15.4
    19  │ handlers/tools.rs               │  6,111 │   201 │ 0.03  │ 19.2
    22  │ cli/handlers/refactor_auto.rs   │  5,203 │   312 │ 0.06  │ 14.8

Problem Files (50 ≤ health < 70):

    52  │ cli/handlers/tdg_handlers.rs    │  4,694 │   412 │ 0.09  │ 12.4
    55  │ cli/handlers/comply_handlers.rs │  4,469 │   389 │ 0.09  │ 11.2
    ...

Healthy Files (health ≥ 70): 198 files

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Kaizen Recommendations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Top 5 refactoring targets (highest impact):

1. analysis_utilities.rs (12,087 lines)
   Suggested splits:
   - analysis_ast.rs (~2,500 lines) - AST utilities
   - analysis_complexity.rs (~1,800 lines) - Complexity calculations
   - analysis_churn.rs (~1,200 lines) - Git churn analysis
   - analysis_defects.rs (~2,100 lines) - Defect prediction
   - analysis_common.rs (~500 lines) - Shared utilities
   - [remaining ~4,000 lines need further analysis]

   Impact: +15 average project health

2. deep_context.rs (7,211 lines)
   Suggested splits:
   - deep_context_rust.rs (~1,800 lines) - Rust analysis
   - deep_context_typescript.rs (~1,400 lines) - TS/JS analysis
   - deep_context_python.rs (~1,100 lines) - Python analysis
   - deep_context_graph.rs (~900 lines) - Graph building
   - deep_context_cache.rs (~600 lines) - Caching
   - deep_context.rs (~1,400 lines) - Core orchestration

   Impact: +12 average project health

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Status: FAILED (5 critical files, 24 problem files)
```

---

## 4. 100-Point Popperian Falsification Criteria

Following Karl Popper's philosophy: claims must be **falsifiable** through empirical testing.

### Category A: File Size Falsification (25 points)

| ID | Criterion | Points | Falsification Method |
|----|-----------|--------|---------------------|
| A1 | No file >500 lines (new) | 10 | `wc -l` on new files |
| A2 | No file growth (ratchet) | 8 | Baseline comparison |
| A3 | 90% files <300 lines | 4 | Distribution analysis |
| A4 | No file >10,000 lines | 3 | Hard threshold |

**Falsification Test:**
```bash
# A1: New file check
find . -name "*.rs" -newer .git/index | xargs wc -l | awk '$1 > 500'
# If ANY output, A1 is FALSIFIED

# A4: Critical threshold
find . -name "*.rs" -exec wc -l {} \; | awk '$1 > 10000'
# If ANY output, A4 is FALSIFIED
```

### Category B: Test-to-Lines Ratio Falsification (30 points)

| ID | Criterion | Points | Falsification Method |
|----|-----------|--------|---------------------|
| B1 | Average TLR ≥ 0.5 | 10 | Aggregate calculation |
| B2 | No file with TLR < 0.1 | 8 | Per-file check |
| B3 | Critical files TLR ≥ 1.0 | 7 | Threshold check |
| B4 | TLR improves over time | 5 | Historical comparison |

**Falsification Test:**
```bash
# B2: Minimum TLR check
pmat comply --file-health --format json | \
  jq '.files[] | select(.tlr < 0.1) | .path'
# If ANY output, B2 is FALSIFIED
```

### Category C: Health Score Falsification (25 points)

| ID | Criterion | Points | Falsification Method |
|----|-----------|--------|---------------------|
| C1 | Average health ≥ 70 | 10 | Aggregate score |
| C2 | No file health < 30 | 8 | Per-file threshold |
| C3 | Health improves monthly | 4 | Trend analysis |
| C4 | Zero F-grade files | 3 | Grade distribution |

**Falsification Test:**
```bash
# C2: Minimum health
pmat comply --file-health --format json | \
  jq '.files[] | select(.health < 30) | .path'
# If ANY output, C2 is FALSIFIED
```

### Category D: Enforcement Falsification (10 points)

| ID | Criterion | Points | Falsification Method |
|----|-----------|--------|---------------------|
| D1 | Pre-commit hook active | 4 | Hook execution test |
| D2 | CI blocks violations | 3 | CI pipeline check |
| D3 | Baseline file current | 3 | Staleness check |

**Falsification Test:**
```bash
# D1: Hook execution
echo "test" > /tmp/test.rs
git add /tmp/test.rs
git commit -m "test" 2>&1 | grep -q "File health check"
# If grep fails, D1 is FALSIFIED
```

### Category E: Split Tool Falsification (10 points)

| ID | Criterion | Points | Falsification Method |
|----|-----------|--------|---------------------|
| E1 | `pmat split` suggests valid splits | 5 | AST analysis accuracy |
| E2 | Suggested splits compile | 3 | Build verification |
| E3 | Splits maintain tests | 2 | Test execution |

**Falsification Test:**
```bash
# E2: Split compilation
pmat split src/cli/analysis_utilities.rs --dry-run --output /tmp/splits
cd /tmp/splits && cargo check
# If cargo check fails, E2 is FALSIFIED
```

### Scoring Summary

| Category | Max Points | Passing Threshold |
|----------|-----------|-------------------|
| A. File Size | 25 | 20 (80%) |
| B. TLR | 30 | 24 (80%) |
| C. Health Score | 25 | 20 (80%) |
| D. Enforcement | 10 | 8 (80%) |
| E. Split Tool | 10 | 8 (80%) |
| **Total** | **100** | **80 (80%)** |

**Grade Interpretation:**

| Score | Grade | Meaning |
|-------|-------|---------|
| 90-100 | A | Exemplary file hygiene |
| 80-89 | B | Meets standards |
| 70-79 | C | Acceptable, needs work |
| 60-69 | D | Below standards |
| <60 | F | FAILING - Immediate action required |

---

## 5. Implementation Plan

### Phase 1: Detection (Week 1)

- [ ] Implement `pmat comply --file-health` command
- [ ] Add TLR calculation to existing analysis
- [ ] Generate baseline file for PMAT codebase
- [ ] Output file health report

### Phase 2: Enforcement (Week 2)

- [ ] Create pre-commit hook for file health
- [ ] Implement ratchet mechanism
- [ ] Add CI pipeline integration
- [ ] Block new large files

### Phase 3: Remediation (Week 3-4)

- [ ] Implement `pmat split` command
- [ ] Create Kaizen plans for top 10 critical files
- [ ] Begin systematic file splitting
- [ ] Track health score improvements

### Phase 4: Dogfooding (Ongoing)

- [ ] Install via `cargo install`
- [ ] Run `pmat comply --file-health` daily
- [ ] Report weekly health metrics
- [ ] Achieve project health ≥80

---

## 6. `pmat split` Command Specification

### Usage

```bash
# Analyze file and suggest splits
pmat split src/cli/analysis_utilities.rs

# Generate split files (dry-run)
pmat split src/cli/analysis_utilities.rs --dry-run --output /tmp/splits

# Execute split with git operations
pmat split src/cli/analysis_utilities.rs --execute

# Interactive mode
pmat split src/cli/analysis_utilities.rs --interactive
```

### Algorithm

1. **Parse AST** - Extract functions, structs, impls, modules
2. **Build dependency graph** - Track which items use which
3. **Cluster analysis** - Group items by cohesion (shared deps)
4. **Suggest splits** - Name clusters, estimate sizes
5. **Generate code** - Create new files with proper imports
6. **Update references** - Fix `use` statements across codebase

### Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Split Analysis: src/cli/analysis_utilities.rs (12,087 lines)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Detected Clusters (by cohesion):

1. AST Analysis Cluster (cohesion: 0.87)
   Items: parse_rust_ast, visit_item, extract_functions, ...
   Lines: ~2,450
   Dependencies: syn, quote
   Suggested name: analysis_ast.rs

2. Complexity Cluster (cohesion: 0.82)
   Items: calculate_cyclomatic, cognitive_complexity, ...
   Lines: ~1,820
   Dependencies: analysis_ast
   Suggested name: analysis_complexity.rs

3. Churn Analysis Cluster (cohesion: 0.91)
   Items: git_log_parse, calculate_churn, hotspot_detection, ...
   Lines: ~1,180
   Dependencies: git2
   Suggested name: analysis_churn.rs

4. Defect Prediction Cluster (cohesion: 0.79)
   Items: defect_model, feature_extraction, prediction, ...
   Lines: ~2,100
   Dependencies: analysis_complexity, analysis_churn
   Suggested name: analysis_defects.rs

5. Common Utilities (cohesion: 0.65)
   Items: file_utils, path_helpers, string_utils, ...
   Lines: ~520
   Dependencies: std only
   Suggested name: analysis_common.rs

Remaining (unclustered): ~3,917 lines
  Recommendation: Further manual analysis required

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Impact Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Files that import analysis_utilities.rs: 47
  - cli/commands.rs (23 imports)
  - handlers/tools.rs (18 imports)
  - ...

Required changes:
  - 47 files need import updates
  - 0 circular dependency risks detected
  - Estimated refactor time: [not provided per guidelines]

Commands:
  pmat split src/cli/analysis_utilities.rs --execute
  pmat split src/cli/analysis_utilities.rs --dry-run --output /tmp/preview
```

---

## 7. Coverage Protection

### Requirement

**95% minimum test coverage must be maintained during all file health improvements.**

### Enforcement

```toml
# .pmat-metrics.toml
[thresholds]
coverage_min = 95.0
coverage_mode = "GATE"  # Block if below

[file_health]
max_lines_new = 500
max_lines_growth = 0  # Ratchet
min_health_new = 70
min_tlr = 0.3
```

### CI Integration

```yaml
# .github/workflows/quality.yml
- name: File Health Check
  run: |
    pmat comply --file-health --min-health 50 || exit 1

- name: Coverage Check
  run: |
    cargo llvm-cov --summary-only | grep -E "^TOTAL" | awk '{
      if ($NF < 95) {
        print "Coverage " $NF "% below 95% threshold"
        exit 1
      }
    }'
```

---

## 8. References

### Peer-Reviewed Research

1. Hindle, A., et al. (2008). "Reading Beside the Lines: Indentation as a Proxy for Complexity." MSR '08.
2. Nagappan, N., et al. (2006). "Mining Metrics to Predict Component Failures." ICSE '06.
3. Bird, C., et al. (2011). "Don't Touch My Code!" FSE '11.
4. Zimmermann, T., & Nagappan, N. (2008). "Predicting Defects Using Network Analysis." ICSE '08.
5. Rahman, F., & Devanbu, P. (2013). "How, and Why, Process Metrics Are Better." ICSE '13.
6. Mockus, A., et al. (2009). "Experiences from Replicating a Case Study." ESEM '09.
7. Athanasiou, D., et al. (2014). "Test Code Quality and Issue Handling Performance." IEEE TSE.
8. Parsai, A., et al. (2018). "Code Coverage vs Mutation Testing." ICST '18.
9. Sweller, J. (1988). "Cognitive Load During Problem Solving." Cognitive Science.
10. Hermans, F. (2021). *The Programmer's Brain*. Manning.

### Industry Standards

11. Martin, R. C. (2008). *Clean Code*. Prentice Hall.
12. Google Engineering Practices (2024). "Code Health: Reducing Technical Debt."
13. Linux Kernel Style Guide (2024). Documentation/process/coding-style.rst.

### Toyota Production System

14. Ohno, T. (1988). *Toyota Production System*. Productivity Press.
15. Liker, J. (2004). *The Toyota Way*. McGraw-Hill.

---

## 9. Appendix: Quick Reference

```
FILE HEALTH SCORE v1.0 - QUICK REFERENCE
========================================

THRESHOLDS:
  New files: ≤500 lines, health ≥70
  Existing: Cannot grow (ratchet)
  Critical: >2000 lines, health <30

TLR REQUIREMENTS (test lines / source lines):
  0-100 lines:   TLR ≥ 0.3
  101-300 lines: TLR ≥ 0.5
  301-500 lines: TLR ≥ 0.7
  501+ lines:    TLR ≥ 1.0 (shouldn't exist)

HEALTH SCORE (0-100):
  Size:       30 pts (0-200=30, 201-500=25, 501-1000=15, 1001-2000=5, >2000=0)
  TLR:        40 pts (actual/required × 40)
  Complexity: 20 pts (CC≤5=20, ≤10=15, ≤15=10, ≤20=5, >20=0)
  Stability:  10 pts (churn: 0-2=10, 3-5=7, 6-10=4, >10=0)

GRADES:
  A: 90-100 (Excellent)
  B: 80-89  (Good)
  C: 70-79  (Acceptable)
  D: 60-69  (Poor)
  E: 50-59  (Critical)
  F: 0-49   (Failing)

COMMANDS:
  pmat comply --file-health           # Check all files
  pmat comply --file-health --verbose # Detailed report
  pmat split <file>                   # Suggest splits
  pmat split <file> --execute         # Execute split

ENFORCEMENT:
  Pre-commit: Blocks new large files, enforces ratchet
  CI: Fails if critical files exist
  Coverage: Maintains 95% minimum
```

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-21
**Maintainer**: PAIML Engineering Team
**License**: MIT OR Apache-2.0
