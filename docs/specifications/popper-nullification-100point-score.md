# Popper Falsifiability Score Specification v1.1

**Version**: 1.1.0
**Date**: 2025-12-09
**Status**: Revised Draft (Post Peer Review)
**Author**: PAIML Engineering Team

## Executive Summary

This specification defines a **100-point Popper Falsifiability Score** for evaluating software repositories against Karl Popper's philosophy of science—specifically his criterion of **falsifiability** as the demarcation between science and non-science. The score measures whether a project's claims are *capable of being tested and potentially refuted*.

> "A theory is scientific if and only if it is falsifiable." — Karl Popper, *The Logic of Scientific Discovery* (1934)

The Popper Falsifiability Score answers: **Can this repository's claims be tested and potentially refuted?**

**Terminology Note (v1.1):** We use "Falsifiability" rather than "Nullification" to align with Popper's original terminology. A *falsifiable* theory is one that *can* be refuted through empirical testing. A *nullified* (or falsified) theory is one that *has been* refuted. We measure the former—the capacity for scientific testing.

### Core Principles

1. **Falsifiability**: Claims must be testable and potentially refutable
2. **Reproducibility**: Results must be independently verifiable
3. **Transparency**: Methods, data, and code must be accessible
4. **Rigor**: Statistical practices must be sound
5. **Openness**: Artifacts must be available for scrutiny

### Toyota Way Integration

This specification applies Toyota Production System principles:

- **Genchi Genbutsu** (Go and See): Analyze actual artifacts, not claims
- **Jidoka** (Built-in Quality): Automated validation at every stage
- **Kaizen** (Continuous Improvement): Track score velocity over time
- **Hansei** (Reflection): Identify root causes of quality gaps

---

## 1. Scoring Categories (100 Total Points)

| Category | Points | Weight | Popper Principle | Gate Status |
|----------|--------|--------|------------------|-------------|
| A. Falsifiability & Testability | 25 | 25% | Core Criterion | **GATEWAY** |
| B. Reproducibility Infrastructure | 25 | 25% | Independent Verification | Standard |
| C. Transparency & Openness | 20 | 20% | Scrutiny Enablement | Standard |
| D. Statistical Rigor | 15 | 15% | Sound Methodology | Standard |
| E. Historical Integrity | 10 | 10% | Evolution Tracking | Standard |
| F. ML/AI Reproducibility | 5 | 5% | Modern Science Standards | Conditional |
| **Total** | **100** | **100%** | | |

### 1.1 Falsifiability Gateway (v1.1 Critical Addition)

**Category A is a prerequisite gate.** If a project scores below 15/25 (60%) on Falsifiability & Testability, the total score is capped regardless of other categories:

```
IF Category_A < 15 THEN:
    Total_Score = 0
    Status = "INSUFFICIENT FALSIFIABILITY - NOT EVALUABLE AS SCIENCE"
```

**Rationale (Popperian):** Under Popper's demarcation criterion, falsifiability is not merely *one factor among many*—it is the *defining characteristic* of scientific claims. A project with perfect documentation, reproducibility, and statistics but no testable claims is, by definition, not science. The gateway prevents a "B" grade for unfalsifiable projects.

**Toyota Way Alignment:** This implements *Jidoka* (stop the line) at the most critical quality checkpoint. If the core criterion fails, downstream metrics are meaningless.

---

## 2. Category A: Falsifiability & Testability (25 points)

The cornerstone of Popperian science: claims must be testable and potentially refutable.

### A1. Hypothesis Documentation (8 points)

**Full Score Criteria (8/8):**
- ✅ Clear hypothesis statements in README or DESIGN.md
- ✅ Explicit claims about what the software does/achieves
- ✅ Defined success criteria with measurable thresholds
- ✅ Documented failure conditions (what would falsify claims)

**Scoring:**
```yaml
8 points: All 4 criteria met, explicit falsifiable claims
6 points: 3 criteria met, implicit falsifiable claims
4 points: 2 criteria met, partial testability
2 points: 1 criterion met, vague claims
0 points: No testable claims documented
```

**Validation:**
```bash
# Check for hypothesis-related documentation
pmat popper-score --check hypothesis \
    --targets README.md DESIGN.md ARCHITECTURE.md \
    --keywords "hypothesis,claim,expected,threshold,benchmark"

# OIP integration: Analyze commit messages for claim evolution
oip analyze --org OWNER --repo REPO --filter "claim|hypothesis|expect"
```

**Example (Good):**
```markdown
## Claims & Falsification Criteria

This library claims:
1. **Performance**: Parser processes >10,000 LOC/second (falsifiable via benchmark)
2. **Correctness**: 100% compliance with language spec (falsifiable via conformance tests)
3. **Memory Safety**: Zero memory leaks under 24h stress test (falsifiable via valgrind)

### Falsification Conditions
- If benchmark shows <10,000 LOC/s on reference hardware, claim #1 is falsified
- If conformance test suite shows <100% pass rate, claim #2 is falsified
- If valgrind reports >0 definitely lost bytes, claim #3 is falsified
```

**Academic Foundation:**
- Popper, K. (1934/1959): *The Logic of Scientific Discovery* [1]
- Lakatos, I. (1970): "Falsification and the Methodology of Scientific Research Programmes" [2]

### A2. Test Coverage as Falsification (10 points)

Tests are the mechanism of falsification in software—they attempt to refute claims.

**Full Score Criteria (10/10):**
- ✅ Unit test coverage ≥85% (line coverage)
- ✅ Branch coverage ≥75%
- ✅ Mutation testing score ≥80% **on core logic/domain modules** (see scoping below)
- ✅ Property-based tests for critical invariants
- ✅ Negative tests (expected failures documented)

**Scoring:**
```yaml
10 points: All criteria met, comprehensive falsification suite
8 points: ≥85% coverage, ≥70% mutation on core, property tests present
6 points: ≥75% coverage, ≥60% mutation on core
4 points: ≥60% coverage, mutation testing attempted
2 points: ≥40% coverage, no mutation testing
0 points: <40% coverage or no tests
```

**Mutation Testing Scope (v1.1 - Heijunka Leveling):**

Not all code requires equal mutation testing rigor. To avoid computational waste (*Muda*), scope mutation testing to **core logic modules**:

```toml
# mutants.toml - Define core modules for mutation testing
[scope]
# Core domain logic - MUST have ≥80% mutation score
include = [
    "src/domain/**/*.rs",
    "src/core/**/*.rs",
    "src/engine/**/*.rs",
    "src/parser/**/*.rs",
]

# UI glue, generated code, tests - EXCLUDED from mutation requirements
exclude = [
    "src/cli/**/*.rs",
    "src/generated/**/*.rs",
    "tests/**/*.rs",
    "benches/**/*.rs",
]
```

**Rationale (Toyota Way):** This implements *Heijunka* (leveling)—not all code paths require the same rigor. Core algorithmic code that embodies scientific claims must be rigorously tested; UI scaffolding does not.

**Validation:**
```bash
# Line and branch coverage (full codebase)
cargo llvm-cov --all-features --workspace --summary-only

# Mutation testing (SCOPED to core modules)
cargo mutants --in-place --jobs 8 -- src/domain src/core src/engine

# Property-based test detection
grep -r "proptest\|quickcheck\|hypothesis" tests/
```

**Rationale:**
Mutation testing is the gold standard for test quality because it tests whether tests can detect deliberate faults—true Popperian falsification [3, 4]. However, applying it uniformly creates O(n²) computational cost that violates lean principles.

### A3. Benchmark Reproducibility (7 points)

Performance claims must be reproducible and falsifiable.

**Full Score Criteria (7/7):**
- ✅ Criterion or equivalent benchmark framework
- ✅ Hardware specifications documented
- ✅ Baseline comparisons with error margins
- ✅ Statistical significance reported (confidence intervals, not just means)
- ✅ Benchmark can be run by external parties

**Scoring:**
```yaml
7 points: All criteria met, benchmarks independently reproducible
5 points: Benchmarks present, hardware documented, no statistical analysis
3 points: Basic benchmarks, no hardware specs
1 point: Benchmarks exist but not reproducible
0 points: No benchmarks or performance claims untestable
```

**Falsification via Confidence Intervals (v1.1):**

Performance is inherently stochastic due to OS scheduling, thermal throttling, and hardware variability. Using strict thresholds (e.g., "if <10,000 LOC/s, claim is falsified") causes flaky falsification.

**Correct Approach:**
```markdown
### Falsification Criteria (Statistically Sound)

Claim: Parser processes >10,000 LOC/second

**Falsification Condition:**
- If the **95% confidence interval upper bound** is < 10,000 LOC/s, claim #1 is falsified
- Single-run measurements are insufficient for falsification
- Minimum 30 iterations required for statistical validity

**Example Output (Criterion.rs):**
```
parse_10k_lines    time: [9,823 µs 9,912 µs 10,015 µs]
                        ^^^^^^^^^^^^^^^^^^^^^^^^
                        95% CI: [9,985 - 10,180] LOC/s

Status: NOT FALSIFIED (CI upper bound 10,180 > threshold 10,000)
```
```

**Example (Criterion.rs with proper falsification):**
```rust
criterion_group!(benches, parser_benchmark, lexer_benchmark);
criterion_main!(benches);

fn parser_benchmark(c: &mut Criterion) {
    // Hardware: Intel i7-12700K, 32GB DDR5, Ubuntu 22.04, isolated cores
    // Claim: >10,000 LOC/s
    // Falsification: 95% CI upper bound < 10,000 LOC/s
    let mut group = c.benchmark_group("parser");
    group.significance_level(0.05);  // 95% CI
    group.sample_size(100);          // Statistical rigor
    group.bench_function("parse_10k_lines", |b| {
        b.iter(|| parser.parse(black_box(&large_input)))
    });
    group.finish();
}
```

**Academic Foundation:**
- Mytkowicz et al. (2009): "Producing Wrong Data Without Doing Anything Obviously Wrong!" - ASPLOS [5]
- Curtsinger & Berger (2013): "STABILIZER: Statistically Sound Performance Evaluation" - ASPLOS [6]

---

## 3. Category B: Reproducibility Infrastructure (25 points)

Independent verification is the hallmark of scientific validity.

### B1. Artifact Availability (10 points)

Following ACM/IEEE artifact evaluation standards [7, 8].

**Full Score Criteria (10/10):**
- ✅ Source code publicly available (GitHub, GitLab, etc.)
- ✅ All dependencies pinned with versions (Cargo.lock, package-lock.json)
- ✅ Build instructions complete and tested
- ✅ Data/models available or generation scripts provided
- ✅ Persistent identifier (DOI via Zenodo, Figshare, or similar)

**Scoring:**
```yaml
10 points: ACM "Artifacts Available" + "Artifacts Evaluated - Reusable" equivalent
8 points: All artifacts available, no DOI
6 points: Code available, some dependencies unpinned
4 points: Code available, incomplete build instructions
2 points: Partial code available
0 points: Code not publicly available
```

**Validation:**
```bash
# Check for locked dependencies
test -f Cargo.lock || test -f package-lock.json || test -f poetry.lock

# Check for DOI badge in README
grep -E "doi\.org|zenodo\.org|figshare\.com" README.md

# Validate build reproducibility
docker build . --no-cache && docker run --rm test-image make test
```

**ACM Badging Alignment:**
- **Artifacts Available**: Artifacts archived in public repository [7]
- **Artifacts Evaluated - Functional**: Documented, consistent, complete [7]
- **Artifacts Evaluated - Reusable**: Well-structured for reuse [7]

### B2. Environment Reproducibility (8 points)

Computational environment must be reproducible.

**Full Score Criteria (8/8):**
- ✅ Dockerfile or Nix flake for environment isolation
- ✅ CI/CD reproduces results on clean environment
- ✅ Random seeds documented and controllable
- ✅ Non-determinism sources identified and mitigated

**Scoring:**
```yaml
8 points: Nix/Guix flake (hermetic), deterministic, CI-verified
7 points: Dockerfile with pinned base image, deterministic, CI-verified
6 points: Dockerfile (unpinned), mostly deterministic
4 points: Build scripts present, some non-determinism
2 points: Manual setup instructions only
0 points: No reproducibility infrastructure
```

**Nix/Guix Gold Standard (v1.1):**

Nix and Guix provide **hermetic builds** where the entire dependency graph is content-addressed. Unlike Docker, which can contain non-deterministic `apt-get update` calls that rot over time, Nix guarantees bit-for-bit reproducibility.

| Technology | Determinism Level | Time Decay Risk | Score Bonus |
|------------|-------------------|-----------------|-------------|
| **Nix/Guix flake** | Hermetic (100%) | None | +1 point |
| **Docker (pinned)** | High (~95%) | Low (base image drift) | 0 |
| **Docker (unpinned)** | Medium (~70%) | High (`apt-get update` rot) | -1 point |
| **Shell scripts** | Low (~50%) | Very High | -2 points |

**Example (flake.nix - Gold Standard):**
```nix
{
  description = "Reproducible research environment";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

  outputs = { self, nixpkgs }: {
    devShells.x86_64-linux.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
      buildInputs = with nixpkgs.legacyPackages.x86_64-linux; [
        rustc cargo clippy  # Pinned to nixos-23.11 versions
      ];
    };
  };
}
```

**Validation:**
```bash
# Check for container definitions (prioritize Nix)
if test -f flake.nix; then
    echo "Gold Standard: Nix flake detected (+1 bonus)"
elif test -f Dockerfile; then
    # Check if base image is pinned
    grep -q "FROM.*@sha256:" Dockerfile && echo "Pinned Docker" || echo "Unpinned Docker (-1)"
fi

# OIP integration: Detect non-determinism in CI history
oip analyze --org OWNER --repo REPO --pattern "flaky|random|non-deterministic"

# bashrs: Check for determinism issues in scripts
bashrs lint Makefile --check DET003  # Non-deterministic wildcards
```

**Academic Foundation:**
- Boettiger (2015): "An Introduction to Docker for Reproducible Research" - ACM SIGOPS [9]
- Hinsen (2015): "ActivePapers: A Platform for Publishing and Archiving Computer-Aided Research" [10]
- Dolstra (2006): "The Purely Functional Software Deployment Model" - PhD Thesis, Utrecht University [31]

### B3. Result Reproduction (7 points)

Can an independent party reproduce the claimed results?

**Full Score Criteria (7/7):**
- ✅ Single-command reproduction (`make reproduce` or equivalent)
- ✅ Expected outputs documented (checksums, ranges, statistics)
- ✅ Reproduction time documented and reasonable
- ✅ Hardware requirements clearly stated
- ✅ Reproduction verified by CI on multiple platforms

**Scoring:**
```yaml
7 points: One-command reproduction, multi-platform CI verification
5 points: Reproduction possible, single platform
3 points: Reproduction requires manual steps
1 point: Reproduction theoretically possible but untested
0 points: No reproduction path documented
```

**Example (Makefile):**
```makefile
.PHONY: reproduce
reproduce:  ## Reproduce all results from scratch (estimated: 45 min)
	@echo "Hardware: Requires 16GB RAM, 4 CPU cores"
	@echo "Expected checksum: sha256:abc123..."
	cargo build --release
	cargo test --release
	./scripts/run_benchmarks.sh
	./scripts/verify_checksums.sh  # Fails if results don't match
```

**Academic Foundation:**
- Peng (2011): "Reproducible Research in Computational Science" - Science [11]
- Stodden et al. (2016): "Enhancing reproducibility for computational methods" - Science [12]

---

## 4. Category C: Transparency & Openness (20 points)

Scientific claims must be open to scrutiny.

### C1. Documentation Accuracy (8 points)

Documentation must accurately reflect the codebase (no drift or staleness).

**Full Score Criteria (8/8):**
- ✅ All README links return HTTP 200
- ✅ Code examples execute successfully
- ✅ API documentation matches implementation
- ✅ Version numbers consistent across files
- ✅ No contradictions between docs and code

**Scoring:**
```yaml
8 points: Zero documentation drift
6 points: 1-2 minor inaccuracies (broken links)
4 points: 3-5 inaccuracies or 1 major contradiction
2 points: Significant doc/code divergence
0 points: Documentation misleading or absent
```

**Terminology Note (v1.1):** We use "Documentation Drift" rather than "hallucination" to avoid anthropomorphizing software. "Drift" and "staleness" are standard software engineering terms for documentation that diverges from the codebase over time. The term "hallucination" is reserved for LLM-generated content where the validator (`pmat validate-readme`) may use LLM-based semantic analysis.

**Validation:**
```bash
# PMAT documentation validation (deterministic checks)
pmat context --output deep_context.md --format llm-optimized
pmat validate-readme \
    --targets README.md CLAUDE.md \
    --deep-context deep_context.md \
    --fail-on-contradiction

# Note: If using LLM-based semantic validation, ensure determinism
# by fixing temperature=0 and providing seed values
```

**Academic Foundation:**
- Farquhar et al. (2024): "Detecting hallucinations in large language models using semantic entropy" - Nature [13]
- MIND Framework (2025): "Unified Detection" - Complex & Intelligent Systems [14]

### C2. Commit History Integrity (7 points)

Git history reveals the evolution of scientific claims.

**Full Score Criteria (7/7):**
- ✅ Conventional commits or consistent message format
- ✅ Atomic commits (single logical change per commit)
- ✅ No force-pushed rewrites on main branch
- ✅ Signed commits (GPG/SSH)
- ✅ Clear traceability from commit to issue/PR

**Scoring:**
```yaml
7 points: All criteria met, exemplary history
5 points: 4 criteria met
3 points: 3 criteria met
1 point: Inconsistent but traceable history
0 points: Squashed/rebased history with no context
```

**Validation:**
```bash
# OIP integration: Analyze commit message quality
oip analyze --org OWNER --repo REPO --check commit-quality

# Check for signed commits
git log --show-signature -10 | grep "Good signature"

# Conventional commit adherence
git log --oneline -100 | grep -E "^[a-f0-9]+ (feat|fix|docs|style|refactor|test|chore):"
```

**Academic Foundation:**
- OpenStack GitCommitMessages Standard [15]
- Conventional Commits Specification v1.0.0 [16]

### C3. Open Science Compliance (5 points)

Adherence to FAIR principles and open science standards.

**Full Score Criteria (5/5):**
- ✅ Open source license (OSI-approved)
- ✅ CITATION.cff or equivalent citation file
- ✅ FAIR principles compliance (Findable, Accessible, Interoperable, Reusable)
- ✅ Preprint or publication linked (if applicable)
- ✅ Data availability statement

**Scoring:**
```yaml
5 points: Full FAIR compliance, citation file, open license
4 points: Open license, citation file, mostly FAIR
3 points: Open license, partial FAIR
1 point: Open license only
0 points: No open license or proprietary
```

**Validation:**
```bash
# Check for CITATION.cff
test -f CITATION.cff

# Check for OSI-approved license
grep -l "MIT\|Apache-2.0\|GPL\|BSD" LICENSE

# FAIR compliance check
pmat fair-check --path .
```

**Academic Foundation:**
- Wilkinson et al. (2016): "The FAIR Guiding Principles for scientific data management" - Scientific Data [17]
- Chue Hong et al. (2022): "FAIR Principles for Research Software (FAIR4RS)" - RDA [18]

---

## 5. Category D: Statistical Rigor (15 points)

Sound methodology prevents false confidence in results.

### D1. Statistical Reporting (8 points)

**Full Score Criteria (8/8):**
- ✅ Effect sizes reported (not just p-values)
- ✅ Confidence intervals provided
- ✅ Multiple comparison corrections applied (if applicable)
- ✅ Sample sizes justified (power analysis)
- ✅ Assumptions stated and tested

**Scoring:**
```yaml
8 points: Comprehensive statistical reporting
6 points: P-values with effect sizes
4 points: P-values only, no effect sizes
2 points: Basic statistics without rigor
0 points: No statistical reporting or p-hacking evident
```

**Anti-Patterns (Deductions):**
- -3 points: Dichotomous p-value interpretation ("p < 0.05 therefore significant")
- -2 points: HARKing (Hypothesizing After Results are Known)
- -2 points: Selective reporting of favorable results

**Academic Foundation:**
- Greenland et al. (2016): "Statistical tests, P values, confidence intervals, and power" - PMC [19]
- Wasserstein & Lazar (2016): "The ASA Statement on p-Values" - American Statistician [20]

### D2. Equivalence & Null Results (7 points)

Proper handling of null results and equivalence testing.

**Full Score Criteria (7/7):**
- ✅ Null results reported (not hidden)
- ✅ Equivalence testing used where appropriate
- ✅ Pre-registration of hypotheses (if applicable)
- ✅ Negative results documented in changelog/papers

**Scoring:**
```yaml
7 points: Null results documented, equivalence testing, pre-registration
5 points: Null results documented, equivalence testing
3 points: Null results acknowledged
1 point: Only positive results reported
0 points: Evidence of selective reporting
```

**Validation:**
```bash
# Check for null result documentation
grep -r "null result\|no significant\|failed to replicate" docs/ CHANGELOG.md

# OIP: Analyze for hidden negative results in commit history
oip analyze --org OWNER --repo REPO --pattern "revert|rollback|didn't work"
```

**Academic Foundation:**
- Ferguson & Heene (2012): "A Vast Graveyard of Undead Theories" - Perspectives on Psychological Science [21]
- Lakens (2017): "Equivalence Tests: A Practical Primer" - Social Psychological and Personality Science [22]

---

## 6. Category E: Historical Integrity (10 points)

Track record and evolution of the project.

### E1. Version History (5 points)

**Full Score Criteria (5/5):**
- ✅ Semantic versioning (SemVer) adherence
- ✅ CHANGELOG.md with all releases documented
- ✅ Breaking changes clearly marked
- ✅ Deprecation warnings before removal
- ✅ Release notes explain "why" not just "what"

**Scoring:**
```yaml
5 points: Full SemVer compliance, comprehensive changelog
4 points: SemVer, changelog present
3 points: Version tags, sparse changelog
1 point: Version tags only
0 points: No versioning strategy
```

### E2. Replication Enablement & Evidence (5 points)

Measures both the *capacity* for replication (intrinsic) and *evidence* of replication (extrinsic).

**v1.1 Split Scoring (Addressing "Rich Get Richer" Bias):**

A new project with perfect replication infrastructure but no external users should not be penalized. We split scoring into **intrinsic** (can others replicate?) and **extrinsic** (have others replicated?).

**Scoring Breakdown:**
```yaml
Replication Enablement (Intrinsic) - 3 points:
  3 points: Single-command replication, CI-verified, docs for external users
  2 points: Documented replication steps, tested internally
  1 point: Replication theoretically possible
  0 points: No replication path

Replication Evidence (Extrinsic) - 2 points:
  2 points: External replications documented, discrepancies explained
  1 point: Community validation (issues/PRs from external users)
  0 points: No external validation (acceptable for new projects)
```

**Rationale (Popperian):** A theory's scientific validity is *intrinsic* to its logical structure—whether it can be tested—not *extrinsic* to its popularity. A perfectly falsifiable theory that no one has tested yet is still scientific. We weight intrinsic enablement (3 points) higher than extrinsic evidence (2 points).

**New Project Guidance:**
- New projects (< 6 months) can achieve 3/5 by focusing on enablement
- External evidence builds naturally over time
- No penalty for being "new science"

**Validation:**
```bash
# OIP: Analyze external contributions
oip analyze --org OWNER --repo REPO --check external-contributors

# Check for replication mentions
grep -r "replicated\|reproduced\|confirmed" issues/ docs/

# Check replication enablement (intrinsic)
test -f REPRODUCING.md || grep -l "reproduce\|replication" README.md
```

**Academic Foundation:**
- Shull et al. (2008): "The role and value of replication in empirical software engineering" - ESE [23]
- Gómez et al. (2014): "Replication of Empirical Studies in Software Engineering" - ESE [24]

---

## 7. Category F: ML/AI Reproducibility (5 points)

Modern science standards for machine learning claims.

### F1. ML Reproducibility Checklist (5 points)

Following NeurIPS/ICML/IJCAI standards [25].

**Full Score Criteria (5/5):**
- ✅ Model architecture fully specified
- ✅ Training procedure documented (hyperparameters, optimizer, learning rate)
- ✅ Random seeds fixed and documented
- ✅ Hardware requirements stated (GPU type, memory)
- ✅ Pre-trained models or training scripts available

**Scoring:**
```yaml
5 points: Full ML reproducibility checklist compliance
4 points: Model/training documented, seeds may vary
3 points: Model available, training partially documented
1 point: Model available, no training documentation
0 points: Claims unverifiable
N/A: No ML component (excluded from denominator - see Section 8.1)
```

**Note (v1.1):** Projects without ML components mark this category as **N/A** (Not Applicable). The score is then normalized—see Section 8.1 for the normalization formula. This prevents artificial score inflation for non-ML projects.

**Validation:**
```bash
# Check for ML reproducibility documentation
test -f MODEL_CARD.md || grep -l "hyperparameters\|random_seed\|learning_rate" README.md

# NeurIPS checklist items
grep -E "seed|deterministic|GPU|CUDA" config/ README.md
```

**Academic Foundation:**
- Pineau et al. (2020): "The Machine Learning Reproducibility Checklist" [25]
- REFORMS Checklist (2024): "Consensus-based Recommendations for ML Science" - PMC [26]

---

## 8. Scoring Methodology

### 8.1 Score Calculation (v1.1 - Normalized)

**Two-Phase Calculation:**

**Phase 1: Falsifiability Gateway Check**
```
IF Category_A < 15 THEN:
    Total_Score = 0
    Status = "INSUFFICIENT FALSIFIABILITY"
    STOP (do not proceed to Phase 2)
```

**Phase 2: Normalized Score Calculation**

For categories with N/A values (e.g., non-ML projects), normalize the score:

```
Points_Earned = A + B + C + D + E + F_applicable
Points_Available = 25 + 25 + 20 + 15 + 10 + F_max

Where:
  F_applicable = 0 if N/A, else actual F score
  F_max = 0 if N/A, else 5

Normalized_Score = (Points_Earned / Points_Available) × 100
```

**Example (Non-ML Project):**
```
A = 22, B = 23, C = 17, D = 12, E = 8, F = N/A

Points_Earned = 22 + 23 + 17 + 12 + 8 + 0 = 82
Points_Available = 25 + 25 + 20 + 15 + 10 + 0 = 95

Normalized_Score = (82 / 95) × 100 = 86.3 → Grade: A-
```

**Example (ML Project):**
```
A = 22, B = 23, C = 17, D = 12, E = 8, F = 4

Points_Earned = 22 + 23 + 17 + 12 + 8 + 4 = 86
Points_Available = 25 + 25 + 20 + 15 + 10 + 5 = 100

Normalized_Score = (86 / 100) × 100 = 86.0 → Grade: A-
```

**Rationale (v1.1):** This normalization prevents "free points" distortion where non-ML projects artificially inflate their scores. A "Hello World" app cannot get 5 free points for not having ML.

### 8.2 Grade Assignment

| Grade | Normalized Score | Interpretation |
|-------|------------------|----------------|
| **A+** | 95-100% | Exemplary Popperian Science |
| **A** | 90-94% | Strong Scientific Standards |
| **A-** | 85-89% | Meets Reproducibility Requirements |
| **B+** | 80-84% | Good Practices, Minor Gaps |
| **B** | 70-79% | Acceptable, Improvement Needed |
| **C** | 60-69% | Significant Reproducibility Gaps |
| **D** | 50-59% | Major Falsifiability Issues |
| **F** | 0-49% | Insufficient Rigor for Independent Verification |

### 8.3 Falsifiability Interpretation

The score directly answers: **Can this project's claims be tested and potentially refuted?**

- **Score ≥85% (A-/A/A+)**: Claims are falsifiable, reproducible, and meet Popperian standards
- **Score 70-84% (B/B+)**: Claims are partially falsifiable, reproducibility gaps exist
- **Score <70%**: **Insufficient Rigor for Independent Verification**

**Terminology Note (v1.1):** We avoid the label "NOT SCIENCE" for scores <70%. Science exists on a spectrum of rigor. A score of 69% indicates "Insufficient Rigor for Independent Verification"—the project may be *pre-scientific* or *emerging science* that requires additional falsification infrastructure before claims can be independently tested. This framing respects the *Kaizen* (continuous improvement) path rather than creating a discouraging binary.

**Passing Tests ≠ Verified Correct (Popperian Caveat):**
A passing test suite means the software has *corroborated* its hypotheses *so far*—it has survived attempts at falsification. It does NOT mean the software is "verified correct." In Popperian terms, science never verifies; it only fails to falsify. The output message uses "MEETS STANDARDS" rather than "VERIFIED CORRECT."

---

## 9. Implementation

### 9.1 CLI Command

```bash
# Basic usage
pmat popper-score --path .

# With OIP integration for commit history analysis
pmat popper-score --path . --with-oip

# Detailed breakdown
pmat popper-score --path . --verbose --format markdown

# CI/CD integration (fail if below threshold)
pmat popper-score --path . --min-score 85 || exit 1

# JSON output for automation
pmat popper-score --path . --format json > popper-score.json
```

### 9.2 Example Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Popper Falsifiability Score - paiml/example-project
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Falsifiability Gateway: PASSED (22/25 ≥ 15) ✅

Raw Score: 82/95 (ML: N/A)
Normalized Score: 86.3% (A-)
Status: MEETS POPPERIAN SCIENCE STANDARDS ✅

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Category Breakdown
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

A. Falsifiability & Testability:    22/25 (88%) [GATEWAY: PASSED]
   ✅ Hypothesis Documentation:       7/8  (explicit claims)
   ✅ Test Coverage:                   9/10 (87% coverage, 82% mutation on core)
   ⚠️  Benchmark Reproducibility:      6/7  (needs confidence intervals)

B. Reproducibility Infrastructure:  23/25 (92%)
   ✅ Artifact Availability:          10/10 (DOI, locked deps)
   ✅ Environment Reproducibility:     8/8  (Nix flake, CI verified)
   ⚠️  Result Reproduction:            5/7  (single command, single platform)

C. Transparency & Openness:         17/20 (85%)
   ✅ Documentation Accuracy:          7/8  (1 broken link - drift detected)
   ✅ Commit History Integrity:        6/7  (conventional commits)
   ⚠️  Open Science Compliance:        4/5  (no CITATION.cff)

D. Statistical Rigor:               12/15 (80%)
   ✅ Statistical Reporting:           7/8  (effect sizes present)
   ⚠️  Equivalence & Null Results:     5/7  (null results not documented)

E. Historical Integrity:             8/10 (80%)
   ✅ Version History:                 5/5  (SemVer, CHANGELOG)
   ⚠️  Replication Enablement:         3/5  (intrinsic: 3/3, extrinsic: 0/2)

F. ML/AI Reproducibility:            N/A (excluded from denominator)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Kaizen Recommendations (Path to A: 90%)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎯 Next Actions (+4% to reach A):
  1. [+1%] Add 95% CI to benchmarks (falsification via confidence intervals)
  2. [+1%] Fix broken documentation link (drift correction)
  3. [+1%] Add CITATION.cff file
  4. [+1%] Document null results in CHANGELOG

📈 External validation will build naturally over time (extrinsic replication)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Popper Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Falsifiability: Claims are testable and potentially refutable
✅ Reproducibility: Independent verification is possible
⚠️  Scrutiny: Minor documentation drift limits full scrutiny
✅ Methodology: Statistical practices are sound
ℹ️  Validation: Replication infrastructure ready, awaiting external use

Verdict: This project has CORROBORATED its hypotheses through testing.
         Claims can be falsified through documented methods.
         (Note: Corroboration ≠ Verification—science never verifies)
```

### 9.3 OIP Integration

```bash
# Full analysis with organizational intelligence
pmat popper-score --path . --with-oip --org paiml --repo example

# OIP provides:
# - Commit message quality analysis
# - Defect pattern detection
# - Historical trend tracking
# - External contributor analysis
```

---

## 10. Toyota Way Analysis

### 10.1 Genchi Genbutsu (Go and See)

The Popper Score analyzes **actual artifacts**, not claimed capabilities:

| What We Analyze | What We Ignore |
|-----------------|----------------|
| Actual test coverage | Marketing claims |
| Real commit history | Self-reported metrics |
| Executed benchmarks | Theoretical performance |
| Documented failures | Hidden negative results |

### 10.2 Jidoka (Built-in Quality)

Quality gates prevent bad science from entering the repository:

```yaml
# .pmat-popper-gates.toml
[falsifiability]
min_test_coverage = 0.85
min_mutation_score = 0.80
require_hypothesis_doc = true

[reproducibility]
require_locked_deps = true
require_dockerfile = true
require_single_command_reproduce = true

[transparency]
max_broken_links = 0
require_citation_file = true
require_open_license = true

[statistics]
require_effect_sizes = true
forbid_dichotomous_pvalue = true
```

### 10.3 Kaizen (Continuous Improvement)

Track score velocity over time:

```bash
# Generate baseline
pmat popper-score --path . --save-baseline

# Weekly comparison
pmat popper-score --path . --compare-baseline --velocity-report
```

### 10.4 Hansei (Reflection)

Root cause analysis for score gaps:

```bash
# Five Whys integration (EXPERIMENTAL - requires LLM backend)
pmat five-whys "Low Popper score in reproducibility" \
    --evidence-from popper-score \
    --depth 5
```

**Note (v1.1 - Experimental Feature):** The `pmat five-whys` integration is marked as **EXPERIMENTAL**. It assumes an advanced AI agent performing automated root cause analysis. Requirements:
- LLM API key (OpenAI, Anthropic, or local)
- `--experimental` flag required to enable
- Non-deterministic outputs (LLM temperature > 0)

For deterministic root cause analysis, use the manual Five Whys template in `docs/templates/five-whys-template.md` instead.

---

## 11. Scientific Foundation

### 11.1 Primary References (Popper & Philosophy of Science)

1. **Popper, K. (1934/1959)**. *The Logic of Scientific Discovery*. Routledge. [Foundational text on falsifiability]

2. **Lakatos, I. (1970)**. "Falsification and the Methodology of Scientific Research Programmes." In *Criticism and the Growth of Knowledge*, Cambridge University Press.

3. **Kuhn, T. S. (1962)**. *The Structure of Scientific Revolutions*. University of Chicago Press. [Critique of naive falsificationism]

### 11.2 Mutation Testing & Falsification (4-6)

4. **Jia, Y., & Harman, M. (2011)**. "An Analysis and Survey of the Development of Mutation Testing." *IEEE Transactions on Software Engineering*, 37(5), 649-678.

5. **Mytkowicz, T., et al. (2009)**. "Producing Wrong Data Without Doing Anything Obviously Wrong!" *ASPLOS '09*.

6. **Curtsinger, C., & Berger, E. D. (2013)**. "STABILIZER: Statistically Sound Performance Evaluation." *ASPLOS '13*.

### 11.3 Artifact Evaluation & Reproducibility (7-12)

7. **ACM (2020)**. "Artifact Review and Badging Version 2.0." *ACM Digital Library*.

8. **IEEE/ACM SC24 (2024)**. "Implementing a Reproducibility Initiative in HPC." *Proceedings of ACM REP '24*.

9. **Boettiger, C. (2015)**. "An Introduction to Docker for Reproducible Research." *ACM SIGOPS Operating Systems Review*, 49(1), 71-79.

10. **Hinsen, K. (2015)**. "ActivePapers: A Platform for Publishing and Archiving Computer-Aided Research." *F1000Research*.

11. **Peng, R. D. (2011)**. "Reproducible Research in Computational Science." *Science*, 334(6060), 1226-1227.

12. **Stodden, V., et al. (2016)**. "Enhancing reproducibility for computational methods." *Science*, 354(6317), 1240-1241.

### 11.4 Documentation & Transparency (13-18)

13. **Farquhar, S., et al. (2024)**. "Detecting hallucinations in large language models using semantic entropy." *Nature*, 630, 625-630.

14. **MIND Framework (2025)**. "Unified Detection of Misinformation." *Complex & Intelligent Systems*.

15. **OpenStack Foundation (2024)**. "GitCommitMessages Standard." wiki.openstack.org.

16. **Conventional Commits (2024)**. "Conventional Commits Specification v1.0.0." conventionalcommits.org.

17. **Wilkinson, M. D., et al. (2016)**. "The FAIR Guiding Principles for scientific data management and stewardship." *Scientific Data*, 3, 160018.

18. **Chue Hong, N. P., et al. (2022)**. "FAIR Principles for Research Software (FAIR4RS)." Research Data Alliance.

### 11.5 Statistical Rigor (19-22)

19. **Greenland, S., et al. (2016)**. "Statistical tests, P values, confidence intervals, and power: a guide to misinterpretations." *European Journal of Epidemiology*, 31(4), 337-350.

20. **Wasserstein, R. L., & Lazar, N. A. (2016)**. "The ASA Statement on Statistical Significance and P-Values." *The American Statistician*, 70(2), 129-133.

21. **Ferguson, C. J., & Heene, M. (2012)**. "A Vast Graveyard of Undead Theories." *Perspectives on Psychological Science*, 7(6), 555-561.

22. **Lakens, D. (2017)**. "Equivalence Tests: A Practical Primer for t Tests, Correlations, and Meta-Analyses." *Social Psychological and Personality Science*, 8(4), 355-362.

### 11.6 Replication & Historical Integrity (23-24)

23. **Shull, F. J., et al. (2008)**. "The role and value of replication in empirical software engineering results." *Empirical Software Engineering*, 13(2), 211-218.

24. **Gómez, O. S., et al. (2014)**. "Replication of Empirical Studies in Software Engineering Research: Preliminary Findings from a Systematic Mapping Study." *MSR '14*.

### 11.7 ML Reproducibility (25-26)

25. **Pineau, J., et al. (2020)**. "The Machine Learning Reproducibility Checklist v2.0." McGill University / MILA.

26. **REFORMS Checklist (2024)**. "Consensus-based Recommendations for Machine-learning-based Science." *PMC*, PMID: 38730305.

### 11.8 Replication Crisis Literature (27-30)

27. **Baker, M. (2016)**. "1,500 scientists lift the lid on reproducibility." *Nature*, 533(7604), 452-454.

28. **CACM (2023)**. "Threats of a Replication Crisis in Empirical Computer Science." *Communications of the ACM*, 66(9).

29. **FSE 2024 Doctoral Symposium**. "The Replication Crisis in Software Engineering: Guidelines for the Scientific Community."

30. **Semmelrock, N., & Beyer, S. (2025)**. "Reproducibility in machine-learning-based research: Overview, barriers, and drivers." *AI Magazine*.

### 11.9 Environment Reproducibility (31)

31. **Dolstra, E. (2006)**. "The Purely Functional Software Deployment Model." PhD Thesis, Utrecht University. [Foundational work on Nix and hermetic builds]

---

## 12. Appendices

### Appendix A: Quick Reference Card

```
POPPER FALSIFIABILITY SCORE v1.1 - QUICK REFERENCE
===================================================

⚠️  GATEWAY CHECK (Must Pass First):
   □ Category A ≥ 15/25 (60%) — If FAIL, Total = 0

A. FALSIFIABILITY (25 pts) [GATEWAY]
   □ Hypothesis documented with falsification criteria
   □ Test coverage ≥85%, mutation score ≥80% ON CORE MODULES
   □ Benchmarks reproducible with 95% CONFIDENCE INTERVALS

B. REPRODUCIBILITY (25 pts)
   □ All artifacts publicly available (DOI preferred)
   □ Environment containerized (Nix preferred > Docker pinned > Docker unpinned)
   □ Single-command reproduction possible

C. TRANSPARENCY (20 pts)
   □ Documentation accurate (no DRIFT or broken links)
   □ Commit history atomic and traceable
   □ FAIR compliance, open license, citation file

D. STATISTICAL RIGOR (15 pts)
   □ Effect sizes and confidence intervals reported
   □ Null results documented
   □ No p-hacking or selective reporting

E. HISTORICAL INTEGRITY (10 pts)
   □ SemVer + comprehensive CHANGELOG
   □ Replication ENABLEMENT (3 pts) + EVIDENCE (2 pts)

F. ML REPRODUCIBILITY (5 pts or N/A)
   □ NeurIPS checklist compliance
   □ If N/A: Excluded from denominator (normalized)

SCORING:
  Normalized_Score = (Points_Earned / Points_Available) × 100
  If ML N/A: Points_Available = 95

MINIMUM FOR RIGOROUS SCIENCE: 85% (A-)
GATEWAY FAILURE (<15/25 on A): Score = 0
```

### Appendix B: CI/CD Integration

```yaml
# .github/workflows/popper-check.yml
name: Popper Science Check

on: [pull_request, push]

jobs:
  popper-score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install PMAT
        run: cargo install pmat

      - name: Install OIP (optional)
        run: cargo install organizational-intelligence-plugin

      - name: Popper Nullification Score
        run: |
          pmat popper-score --path . --min-score 85 --format json | tee popper-score.json
          SCORE=$(jq '.total_score' popper-score.json)
          echo "Popper Score: $SCORE/100"
          if (( $(echo "$SCORE < 85" | bc -l) )); then
            echo "::error::Popper Score $SCORE below threshold (85)"
            exit 1
          fi

      - name: Upload Score
        uses: actions/upload-artifact@v4
        with:
          name: popper-score
          path: popper-score.json
```

### Appendix C: Badge Integration

```markdown
<!-- README.md -->
![Popper Score](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/USER/REPO/main/.popper-score.json)
```

**JSON Endpoint (`.popper-score.json`):**
```json
{
  "schemaVersion": 1,
  "label": "Popper Score",
  "message": "87/100 (A-)",
  "color": "brightgreen"
}
```

---

## 13. Conclusion

The Popper Falsifiability Score v1.1 provides an objective, automated assessment of whether a software project meets the standards of reproducible science. By grounding the evaluation in Karl Popper's falsifiability criterion and modern reproducibility standards (ACM/IEEE artifact evaluation, NeurIPS ML checklist, FAIR principles), this specification ensures that:

1. **Claims are testable** — through documented hypotheses and comprehensive test suites
2. **Results are reproducible** — through containerization (Nix preferred), locked dependencies, and single-command reproduction
3. **Methods are transparent** — through accurate documentation (no drift) and traceable commit history
4. **Statistics are sound** — through proper effect size reporting, confidence intervals, and null result documentation
5. **History is verifiable** — through versioning and replication enablement

**Key v1.1 Changes (Post Peer Review):**
- Renamed from "Nullification" to "Falsifiability" (terminology precision)
- Added **Falsifiability Gateway**: Category A < 15/25 → Total Score = 0 (Popperian primacy)
- **Mutation testing scoped** to core logic modules (Muda/waste reduction)
- **Confidence intervals** required for benchmark falsification (statistical soundness)
- **Nix/Guix bonus** for hermetic reproducibility (Jidoka/built-in quality)
- **Documentation Drift** terminology (avoiding anthropomorphization)
- **Replication split** into enablement (intrinsic) vs evidence (extrinsic) (new project fairness)
- **Score normalization** for N/A categories (prevents free points distortion)
- **Softer language** for <70%: "Insufficient Rigor" not "NOT SCIENCE" (Kaizen respect)
- **Five Whys marked experimental** (LLM dependency transparency)

A normalized score of **≥85% (A-)** indicates the project meets Popperian scientific standards and its claims can be falsified through documented methods. Scores below 70% indicate "Insufficient Rigor for Independent Verification"—the project may need additional falsification infrastructure before claims can be independently tested.

> "In so far as a scientific statement speaks about reality, it must be falsifiable; and in so far as it is not falsifiable, it does not speak about reality." — Karl Popper

---

**Document Version**: 1.1.0
**Last Updated**: 2025-12-09
**Peer Review Status**: Addressed (10/10 annotations incorporated)
**Maintainer**: PAIML Engineering Team
**License**: MIT OR Apache-2.0
