# Self-Enforcement & Dogfooding

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 26

## Problem Statement

PMAT enforces quality on other projects but does not systematically enforce
quality on itself. Target: **Grade A (≥90%) with 95% category penetration**
(≥10/11 categories at ≥80%, ≥9/11 at ≥90%).

Dogfooding credibility requires that pmat scores itself at A-level before
claiming to judge other projects.

## Progress Tracking

### Baseline (v3.12.0, 2026-04-08): Grade B (76.3%, 195/289)

| Category | Earned | Max | % | Status |
|----------|--------|-----|---|--------|
| GPU/SIMD Quality | 10.0 | 10 | 100% | ✓ |
| Known Defects | 20.0 | 20 | 100% | ✓ |
| Performance & Benchmarking | 10.0 | 10 | 100% | ✓ |
| Reproducibility | 12.9 | 15 | 86% | ⚠ |
| Testing Excellence | 16.5 | 20 | 82.5% | ⚠ |
| Documentation | 12.0 | 15 | 80% | ⚠ |
| Build Performance | 10.0 | 15 | 66.7% | ✗ |
| Dependency Health | 8.0 | 12 | 66.7% | ✗ |
| Formal Verification | 8.6 | 16 | 53.9% | ✗ |
| Rust Tooling & CI/CD | 75.0 | 130 | 57.7% | ✗ |
| Code Quality | 12.0 | 26 | 46.2% | ✗ |

Penetration@80: 6/11 (55%). Penetration@90: 3/11 (27%).

### After Phase 1 (2026-04-08): Grade B+ (80.8%, 224.5/289)

| Category | Before | After | Delta | Status |
|----------|--------|-------|-------|--------|
| GPU/SIMD Quality | 100% | 100% | — | ✓ |
| Known Defects | 100% | 100% | — | ✓ |
| Performance & Benchmarking | 100% | 100% | — | ✓ |
| Reproducibility | 86% | 86% | — | ⚠ |
| Build Performance | 66.7% | **83.3%** | +16.6% | ✓ |
| Testing Excellence | 82.5% | 82.5% | — | ⚠ |
| Documentation | 80% | 80% | — | ⚠ |
| Rust Tooling & CI/CD | 57.7% | **75.4%** | +17.7% | ⚠ |
| Dependency Health | 66.7% | 66.7% | — | ✗ |
| Code Quality | 46.2% | **61.5%** | +15.3% | ✗ |
| Formal Verification | 53.9% | 53.9% | — | ✗ |

Penetration@80: **7/11 (64%)**. Penetration@90: 3/11 (27%).

**Phase 1 actions completed:**
- Added `.cargo/config.toml` (+Build Perf)
- Added SAFETY comments to 21 unsafe blocks (unsafe ratio 58%→100%, +4 pts Code Quality)
- Added `[workspace]`, `resolver="2"`, `[workspace.package]` (+11 pts Rust Tooling)
- Added `[package.metadata.release]` with CHANGELOG automation (+8 pts Rust Tooling)
- Added `--generate-link-to-definition` to docs.rs rustdoc-args (+2 pts Rust Tooling)
- Updated fastrand 2.4.0→2.4.1 (yanked fix)
- Added MSRV badge to README (+2 pts MSRV tracking)

### After Phase 2 (2026-04-08): Grade B+ (82.6%, 234.2/289)

| Category | Phase 1 | Phase 2 | Delta | Status |
|----------|---------|---------|-------|--------|
| GPU/SIMD Quality | 100% | 100% | — | ✓ |
| Known Defects | 100% | 100% | — | ✓ |
| Performance & Benchmarking | 100% | 100% | — | ✓ |
| Reproducibility | 86% | 86% | — | ⚠ |
| Build Performance | 83.3% | 83.3% | — | ⚠ |
| Testing Excellence | 82.5% | 82.5% | — | ⚠ |
| Rust Tooling & CI/CD | 75.4% | **80.8%** | +5.4% | ⚠ |
| Documentation | 80% | 80% | — | ⚠ |
| Dependency Health | 66.7% | 66.7% | — | ✗ |
| Code Quality | 61.5% | **65.4%** | +3.9% | ✗ |
| Formal Verification | 53.9% | **64.4%** | +10.5% | ✗ |

Penetration@80: **8/11 (73%)**. Penetration@90: 3/11 (27%).

**Phase 2 actions completed:**
- Added `[workspace.dependencies]` (+2 pts Rust Tooling)
- Added `.github/workflows/post-release.yml` with MSRV testing (+5 pts Rust Tooling)
- Added 7 lean_theorem refs to contracts/pmat-core.yaml (3→10, +1.68 pts Formal Verification)
- Reduced deep nesting from 19→3 lines (+1 pt Code Quality complexity)
- Dedented string literals in 6 test/template files

## Remaining Gap Analysis (Post-Phase 2)

### To reach 95% penetration (10/11 categories ≥80%)

Need 2 more categories above 80% from: Code Quality (65.4%),
Dependency Health (66.7%), Formal Verification (64.4%).

### Structural Limits (Fast-Mode Scoring)

| Category | Max Fast-Mode | Why |
|----------|--------------|-----|
| Code Quality | 76.9% (20/26) | Mutation defaults to 4/8, build time to 2/4 |
| Formal Verification | 64.4% (10.3/16) | Miri=0.9 (not run), Kani=2.0 (not run) |
| Dependency Health | 66.7% (8/12) | 113 deps → 1/5 pts, can't reduce without removing features |

**Key insight**: Code Quality and Formal Verification **cannot reach 80% in
fast mode**. Full mode is required, which means actually running `cargo-mutants`,
Miri, and Kani. These tools must be installed and the codebase must pass them.

### Path to 95% Penetration (Full Mode)

| Fix | Category | Impact | Effort |
|-----|----------|--------|--------|
| Install + pass Miri | Formal Verification | 64.4%→83% | Medium |
| Install + pass Kani | Formal Verification | →95%+ | Medium (already have 13 proofs) |
| Install cargo-mutants + pass | Code Quality | 65.4%→80%+ | High |
| Remove 336 `#[allow(dead_code)]` | Code Quality +2 | +7.7% | Very High |

### Unfixable Constraints

- **Dependency count**: 113 direct deps → 1/5 pts. Caps Dep Health at 66.7%.
  Would need ≤50 deps for 2/5 pts. Not practical without major feature removal.
- **Fast-mode ceiling**: Several categories score conservatively in fast mode.
  Full-mode scoring (`--full`) is the authoritative measure for A-level.

## Penetration Model

**Definition**: Penetration = percentage of RPS categories at or above threshold.

```
penetration(threshold) = count(categories where % >= threshold) / total_categories
```

**Targets**:
- 80% penetration at ≥80%: 9/11 categories (current: 6/11 = 55%)
- 95% penetration at ≥80%: 10/11 categories (target)
- 95% penetration at ≥90%: 10/11 categories (stretch)

**Grade A threshold**: normalized avg ≥ 90% AND penetration(80%) ≥ 95%.

## Dogfooding Workflow

### Continuous Self-Assessment

```bash
# Step 1: Install latest pmat from source
cargo install --path .

# Step 2: Run self-assessment
pmat rust-project-score --full --format json -o .pmat-metrics/self-score.json

# Step 3: Run compliance
pmat comply check

# Step 4: Query coverage gaps on self
pmat query --coverage-gaps --limit 20 --exclude-tests

# Step 5: Score diagnosis
pmat query --score-diagnosis --limit 10
```

### Pre-Release Gate

Before `cargo publish`:
1. `cargo install --path .` — rebuild binary
2. `pmat rust-project-score` — must be Grade A
3. `pmat comply check` — must be COMPLIANT, 0 errors
4. Penetration ≥ 95% at 80% threshold
5. No category below 70%

### CI Self-Score Job

```yaml
# .github/workflows/self-score.yml
name: Self-Score Dogfood
on: [push]
jobs:
  self-score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install --path .
      - run: pmat rust-project-score --format json > self-score.json
      - run: |
          GRADE=$(jq -r '.grade' self-score.json)
          if [ "$GRADE" != "A" ]; then
            echo "FAIL: Self-score grade is $GRADE, expected A"
            exit 1
          fi
```

## Implementation Roadmap

### Phase 1: Mechanical Fixes (Tier 1) — Immediate

1. Create `.cargo/config.toml` with build settings
2. Run `cargo update` to fix yanked `fastrand`
3. Add workspace-level lint configuration to `Cargo.toml`
4. Create `.clippy.toml` with `disallowed-methods = ["unwrap"]`
5. Add `SAFETY:` comments to undocumented unsafe blocks
6. Re-score: target Build Perf ≥80%, Dep Health ≥80%, Rust Tooling ≥75%

### Phase 2: Code Quality Push — Short-term

7. Audit and delete dead code marked `#[allow(dead_code)]`
8. Refactor deeply-nested functions (>40 char indent → helper extraction)
9. Add rustdoc examples to public API (≥5 doc-tests)
10. Re-score: target Code Quality ≥70%, Documentation ≥90%

### Phase 3: Verification & Testing — Medium-term

11. Set up Miri in CI (nightly toolchain)
12. Add more Kani proof harnesses (target: ≥10 verified)
13. Run `cargo-mutants` and fix surviving mutants
14. Re-score: target Formal Verification ≥80%, Testing ≥90%

### Phase 4: A-Level Lock-In

15. Add `pmat rust-project-score --gate A` to pre-push hook
16. Add self-score to CI as required check
17. Track score trend in `.pmat-metrics/` history
18. Document penetration in README badge

## Compliance Integration

### New Check: CB-160 (Self-Score Gate)

```yaml
cb-160:
  name: Self-Score Grade Gate
  severity: error
  description: pmat must score itself at Grade A
  check: |
    score = run("pmat rust-project-score --format json")
    FAIL if score.grade != "A"
    FAIL if penetration(80%) < 0.95
```

### New Check: CB-161 (Penetration Gate)

```yaml
cb-161:
  name: Category Penetration Gate
  severity: warning
  description: All RPS categories must be above minimum threshold
  check: |
    for category in score.categories:
      WARN if category.percentage < 80%
      FAIL if category.percentage < 60%
```

## Key Metrics

| Metric | Baseline | Phase 1 | Phase 2 | Target | Method |
|--------|----------|---------|---------|--------|--------|
| RPS Grade | B | B+ | **B+** | A | `pmat rust-project-score` |
| RPS % | 76.3% | 80.8% | **82.6%** | ≥90% | Normalized avg |
| RPS Points | 195/289 | 224.5/289 | **234.2/289** | ≥260/289 | Raw score |
| Penetration@80 | 55% (6/11) | 64% (7/11) | **73% (8/11)** | 95% (10/11) | Categories ≥80% |
| Penetration@90 | 27% (3/11) | 27% (3/11) | 27% (3/11) | 82% (9/11) | Categories ≥90% |
| Comply Status | COMPLIANT | COMPLIANT | COMPLIANT | COMPLIANT | `pmat comply check` |
| Muda Score | 13.6 | 13.6 | 13.6 | ≤15 | CB-300 |
| Dead Code | 0.6% | 0.6% | 0.6% | ≤1% | CB-304 |

## Key Files

| File | Purpose |
|------|---------|
| `src/services/rust_project_score/orchestrator.rs` | RPS orchestrator |
| `.pmat-metrics/self-score.json` | Self-score history |
| `.cargo/config.toml` | Build configuration (Phase 1) |
| `.clippy.toml` | Lint policy (Phase 1) |
| `.github/workflows/self-score.yml` | CI self-score gate (Phase 4) |

## References

- [Scoring Convergence](scoring-convergence.md) — composite score design
- [Repo Health](repo-health.md) — RPS v3.0 categories and weights
- [Quality Gates](quality-gates.md) — O(1) pre-commit enforcement
- [Code Quality](code-quality.md) — DBC and quality metrics
