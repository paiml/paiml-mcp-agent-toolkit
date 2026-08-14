# PMAT Quality Standards & Configuration

This document describes the comprehensive quality infrastructure for the PMAT project, configured to use all PMAT features on itself (meta-application).

**Version**: PMAT v2.200.0
**Updated**: 2025-11-21

## Quality Baseline (Current State)

### Defect Analysis
- **Status**: ⚠️ FAILING (43 critical defects)
- **Critical Defects**: 43 `.unwrap()` calls
- **Affected Files**: 16 files (examples/, build.rs, fuzz/)
- **Exit Code**: 1 (blocking)

**Note**: Examples and fuzz targets are excluded from production defect checks via `.pmat/analyze.toml` configuration.

### Technical Debt Grading (TDG)
- **Score**: 91.3/100 (Grade A)
- **Status**: ⚠️ FAILING (auto-fail due to critical defects)
- **Files Analyzed**: 909
- **Grade Distribution**:
  - A+: 440 files (48.4%)
  - A: 300 files (33.0%)
  - F: 16 files (1.8%) - due to critical defects

### Rust Project Score
- **Score**: 83.5/134 (62.3%, Grade B+)
- **Category Breakdown**:
  - Code Quality: 7.0/26 (26.9%) ❌
  - Testing Excellence: 13.5/20 (67.5%) ⚠️
  - Documentation: 12.0/15 (80.0%) ✅
  - Performance & Benchmarking: 7.0/10 (70.0%) ✅
  - Dependency Health: 7.0/12 (58.3%) ⚠️
  - Rust Tooling & CI/CD: 24.0/130 (18.5%) ❌

## Quality Infrastructure

### 1. Configuration Files

#### `.pmat/analyze.toml`
Comprehensive quality configuration with zero-tolerance policies:
- Critical defects auto-fail
- TDG minimum score: 90.0 (Grade A)
- Rust project score minimum: 80.0 (Grade B+)
- Test code exclusions (tests/, benches/, examples/, fuzz/)
- Fast mode enabled by default (skips slow checks)

**Key Features**:
- Known Defects v2.1 with `.unwrap()` detection
- TDG auto-fail on critical defects
- Comprehensive quality gates (defects, TDG, clippy, rustfmt, tests)
- Parallel analysis for CI/CD

**Quick Commands**:
```bash
# Run defect analysis (fast, <10 seconds)
pmat analyze defects --path server --format text

# Run TDG analysis with auto-fail
pmat analyze tdg --path server --format table

# Run comprehensive Rust project score (fast mode)
pmat rust-project-score --path server --format text

# Run full Rust project score with all checks (~10-15 min)
pmat rust-project-score --path server --format text --full
```

### 2. Pre-Commit Hooks

**Location**: `.git/hooks/pre-commit`

The pre-commit hook automatically enforces:
1. **TDG Quality Gates** - Checks for regressions against baseline
2. **bashrs Linting** - Shell script quality (for .sh and Makefile files)
3. **Known Defects** - Zero-tolerance for critical defects

> There is no branch check. A "Zero Branching" step was listed here and in
> `PMAT-INTEGRATION.md`; no committed hook implements it
> (`grep -c -i branch .git/hooks/pre-commit` -> 0), and it would contradict the
> workflow this project requires: feature branch -> PR -> the required
> `ci / gate` check on protected `master`. Direct pushes to master are blocked,
> so a hook demanding commits be ON master forbids the only path work can take.

**Configuration**: `.pmat/tdg-rules.toml`
- Mode: `strict` (blocks commits on violations)
- Min Grade: `B+`
- Max Score Drop: `5` points
- Block on regression: `true`
- Block on new files with low quality: `true`

**How to Use**:
```bash
# Normal commit (runs all checks automatically)
git add .
git commit -m "Your message"

# Bypass hooks (NOT RECOMMENDED - only for emergencies)
git commit --no-verify -m "Emergency fix"

# Refresh hooks (if you update PMAT)
pmat hooks refresh
```

### 3. GitHub Actions CI/CD

**Location**: `.github/workflows/quality.yml`

Automated quality gates run on every push and PR:

**7 Quality Gates** (in order):
1. **Critical Defects** (BLOCKING) - Known Defects v2.1
2. **TDG Analysis** (BLOCKING) - Technical debt scoring
3. **Clippy Lints** (BLOCKING) - Rust linter
4. **Code Formatting** (BLOCKING) - rustfmt compliance
5. **Tests** (BLOCKING) - Unit & integration tests
6. **Rust Project Score** (WARNING) - Comprehensive quality
7. **Security Audit** (WARNING) - cargo-audit scan

**Status Indicators**:
- ✅ Green check = all gates passed
- ❌ Red X = blocking gate failed
- ⚠️  Yellow warning = advisory gate (non-blocking)

**CI Configuration**:
- Fast mode enabled (2-3 minute runs)
- Caching enabled (cargo registry, git, target/)
- Parallel execution where possible
- Artifacts uploaded (quality reports, 30-day retention)

## Quality Commands Reference

### Defect Analysis (Known Defects v2.1)

```bash
# Quick scan (text output)
pmat analyze defects --path server --format text

# JSON output for CI/CD
pmat analyze defects --path server --format json > defects.json

# JUnit XML for test frameworks
pmat analyze defects --path server --format junit > defects.xml

# Check exit code
pmat analyze defects --path server && echo "✅ Clean" || echo "❌ Defects found"
```

**Exit Codes**:
- `0` - No critical defects found
- `1` - Critical defects found (blocks CI/CD)

### TDG Analysis

```bash
# Quick TDG scan
pmat analyze tdg --path server --format table

# With baseline comparison
pmat tdg check-regression --baseline .pmat/baseline.json --path server

# Create/update baseline
pmat tdg baseline create --output .pmat/baseline.json --path server
pmat tdg baseline update --output .pmat/baseline.json
```

### Rust Project Score

```bash
# Fast mode (default, ~2-3 minutes)
pmat rust-project-score --path server

# Full mode with all checks (~10-15 minutes)
pmat rust-project-score --path server --full

# JSON output for parsing
pmat rust-project-score --path server --format json > score.json

# Markdown report for documentation
pmat rust-project-score --path server --format markdown --output SCORE.md

# Verbose with detailed breakdown
pmat rust-project-score --path server --verbose

# Show only failures and warnings
pmat rust-project-score --path server --failures-only
```

## Quality Improvement Roadmap

### High Priority (Blocking Production)

1. **Fix Critical Defects** (43 instances)
   - [ ] Replace `.unwrap()` with `.expect()` or proper error handling
   - [ ] Focus on production code (src/ directory)
   - [ ] Examples and fuzz targets are lower priority
   - Target: 0 critical defects in src/

2. **Improve Code Quality Score** (Current: 26.9%)
   - [ ] Add unsafe documentation (23 unsafe blocks need safety comments)
   - [ ] Reduce cyclomatic complexity where needed
   - [ ] Run `cargo clippy --fix` to auto-fix issues
   - Target: 20.0/26 points (76.9%)

3. **Enhance Rust Tooling & CI/CD** (Current: 18.5%)
   - [x] Configure quality gates in GitHub Actions
   - [x] Set up pre-commit hooks
   - [ ] Add workspace lints to Cargo.toml
   - Target: 100.0/130 points (76.9%)

### Medium Priority (Quality Enhancements)

4. **Improve Testing** (Current: 67.5%)
   - [ ] Increase code coverage to ≥85%
   - [ ] Add mutation testing (cargo-mutants)
   - [ ] Improve integration test coverage
   - Target: 17.0/20 points (85.0%)

5. **Optimize Dependencies** (Current: 58.3%)
   - [ ] Reduce dependency count (current: high)
   - [ ] Use feature flags for optional dependencies
   - [ ] Prune dependency tree
   - Target: 9.0/12 points (75.0%)

### Low Priority (Nice to Have)

6. **Formal Verification** (Current: 37.5%)
   - [ ] Run Miri on unsafe code
   - [ ] Add Kani formal verification
   - Target: 6.0/8 points (75.0%)

7. **Performance Benchmarking** (Current: 70.0%)
   - [ ] Add Criterion benchmarks
   - [ ] Set up benchmark CI
   - Target: 9.0/10 points (90.0%)

## Quick Start for Contributors

### First-Time Setup

```bash
# 1. Install PMAT (if not already installed)
cargo install pmat

# 2. Install quality tools
cargo install cargo-llvm-cov cargo-audit bashrs

# 3. Install pre-commit hooks
pmat hooks install --tdg-enforcement

# 4. Create TDG baseline (first time only)
pmat tdg baseline create --output .pmat/baseline.json --path server
```

### Before Every Commit

```bash
# 1. Run quick quality check
pmat analyze defects --path server

# 2. Run TDG check
pmat analyze tdg --path server

# 3. Run clippy
cargo clippy --fix --allow-dirty --allow-staged

# 4. Format code
cargo fmt --all

# 5. Run tests
cargo test --lib

# 6. Commit (pre-commit hooks run automatically)
git add .
git commit -m "Your commit message"
```

### Before Release

```bash
# 1. Run full quality assessment
pmat rust-project-score --path server --full --verbose

# 2. Ensure all critical gates pass
pmat analyze defects --path server && \
pmat analyze tdg --path server && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo fmt --all -- --check && \
cargo test --lib --release

# 3. Run security audit
cargo audit

# 4. Update baseline if quality improved
pmat tdg baseline update --output .pmat/baseline.json
```

## Troubleshooting

### "Critical defects found" error

**Cause**: `.unwrap()` calls detected in production code

**Fix**:
```bash
# See defect report
pmat analyze defects --path server --format text

# Replace .unwrap() with .expect() or proper error handling
# Example:
value.unwrap()  // ❌ Causes panic
value.expect("Descriptive error message")  // ✅ Better panic message
value?  // ✅ Best - propagate error
```

### "TDG auto-fail" error

**Cause**: Critical defects triggering TDG auto-fail

**Fix**: Same as above - fix critical defects first

### Pre-commit hook blocking commit

**Cause**: Quality regression or new files below minimum grade

**Fix**:
```bash
# See what failed
git commit -m "test"  # Will show error details

# Option 1: Fix quality issues
# Improve code quality to meet standards

# Option 2: Update baseline (if intentional changes)
pmat tdg baseline update --output .pmat/baseline.json

# Option 3: Bypass (NOT RECOMMENDED)
git commit --no-verify -m "Emergency fix"
```

### CI/CD workflow failing

**Cause**: Quality gate failure in GitHub Actions

**Fix**:
1. Check workflow logs in GitHub Actions tab
2. Identify which gate failed (1-7)
3. Run that check locally: `pmat <command> --path server`
4. Fix issues and push again

## Scientific Foundation

All quality thresholds are based on peer-reviewed research:

- **Known Defects v2.1**: Zero-tolerance policy informed by Cloudflare outage 2025-11-18 (3+ hour network outage caused by `.unwrap()` panic)
- **TDG Scoring**: Complexity weights reduced based on arXiv 2024 ("No correlation between complexity and presence of bugs")
- **Mutation Testing**: Weights increased based on ICST 2024 findings (developers find it highly valuable)
- **Rust Project Score v1.1**: 106-point scoring across 6 categories, evidence-based design from 15 peer-reviewed papers (2022-2025)

## References

- PMAT Documentation: https://paiml.github.io/pmat-book/
- GitHub Repository: https://github.com/paiml/paiml-mcp-agent-toolkit
- crates.io: https://crates.io/crates/pmat
- Known Defects Spec: `docs/specifications/known-defects-v2.1.md`
- Rust Project Score Spec: `docs/specifications/components/repo-health.md`

## Support

For issues, bugs, or questions:
- GitHub Issues: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- PMAT Book: https://paiml.github.io/pmat-book/ch05-00-analyze-suite.html (Known Defects chapter)

---

**Last Updated**: 2025-11-21
**PMAT Version**: v2.200.0
**Maintainer**: PAIML Team
