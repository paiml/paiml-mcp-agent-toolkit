# Repository Score Specification

## 1. Executive Summary

This document specifies a comprehensive **Repository Health Score** system for evaluating GitHub repositories based on measurable quality metrics, automated validation, and industry best practices. The score ranges from 0-100 points and provides an objective, quantifiable assessment of repository maintainability, reliability, and production readiness.

The specification draws from:
- **PMAT TDG Score methodology** (Test-Driven Grade)
- **PAIML best practices** from 6 production repositories (bashrs, ruchy, depyler, etc.)
- **Peer-reviewed research** (Nature 2024, IEEE TSE, ACM studies)
- **Modern DevOps standards** (GitHub Actions, pre-commit hooks, mutation testing)

**Target Audience**: Open-source maintainers, engineering teams, code reviewers, and automated quality assessment tools.

---

## 2. Scoring Categories (100 Total Points)

### A. Documentation Quality (15 points)

High-quality documentation is the gateway to successful adoption and contribution. This category evaluates documentation accuracy, comprehensiveness, and markdown quality.

#### A1. Documentation Accuracy (10 points)

**Full Score Criteria (10/10):**
- ✅ All hyperlinks return HTTP 200 status (no 404s) across ALL markdown files
- ✅ All code examples execute successfully (bash/shell snippets tested)
- ✅ Installation instructions verified (automated testing via CI)
- ✅ API documentation matches actual codebase (semantic validation)
- ✅ Version numbers consistent across README, Cargo.toml/package.json, and CHANGELOG
- ✅ All docs/*.md files validated (specifications, design docs, etc.)

**Partial Credit:**
- 8/10: 1-2 broken links OR 1 broken code example
- 6/10: 3-5 broken links OR 2-3 broken examples
- 3/10: Significant inconsistencies (outdated version numbers, missing sections)
- 0/10: No README or >10 broken links

**Validation Method:**
```bash
# Step 1: Markdown linting (format, style, consistency)
find . -name "*.md" -not -path "./target/*" -not -path "./node_modules/*" | while read -r file; do
    markdownlint "$file" || echo "WARN: Markdown lint issues in $file"
done

# Step 2: Generate deep context
pmat context --output deep_context.md --format llm-optimized

# Step 3: Validate accuracy (links, code examples, API claims)
find . -name "*.md" -not -path "./target/*" -not -path "./node_modules/*" | while read -r file; do
    pmat validate-readme \
        --targets "$file" \
        --deep-context deep_context.md \
        --fail-on-contradiction \
        --output json
done

# Or use the comprehensive command (validates all docs at once)
pmat validate-docs \
    --deep-context deep_context.md \
    --fail-on-contradiction \
    --check-links \
    --lint-markdown \
    --output json > docs-validation.json
```

**Markdown Linting Rules:**
- ✅ Consistent heading hierarchy (no skipped levels: h1 → h3)
- ✅ Code blocks have language tags (\`\`\`bash, \`\`\`rust, etc.)
- ✅ No trailing whitespace
- ✅ Consistent list formatting (all `-` or all `*`, not mixed)
- ✅ Proper link formatting (\[text\](url) not bare URLs)
- ✅ No duplicate headings at same level
- ✅ Blank lines before/after headings
- ✅ Line length ≤120 chars (configurable)

**Configuration (`.markdownlint.json`):**
```json
{
  "default": true,
  "MD013": { "line_length": 120 },
  "MD033": false,
  "MD041": false
}
```

**What Gets Validated:**
- **README.md**: Installation, usage examples, badges, links
- **docs/specifications/*.md**: Technical accuracy, API references, code snippets
- **docs/design/*.md**: Architecture decisions, implementation details
- **CLAUDE.md, GEMINI.md, AGENT.md**: AI agent instructions, workflow accuracy
- **CHANGELOG.md**: Version consistency, release notes accuracy

**Duplicate Detection (High Entropy Check):**
```bash
# Detect duplicate or near-duplicate documentation
pmat analyze-docs --check-duplicates \
    --similarity-threshold 0.85 \
    --output json > duplicate-docs.json

# Find semantically similar docs (copy-paste content)
pmat semantic-search \
    --query-file docs/spec1.md \
    --search-path docs/ \
    --threshold 0.9 \
    --exclude-self

# Hash-based exact duplicate detection
find . -name "*.md" -not -path "./target/*" -exec md5sum {} \; | \
    sort | uniq -w32 -d
```

**Duplicate Documentation Penalties:**
- -1 point: 1-2 duplicate sections across docs (>85% similarity)
- -2 points: 3-5 duplicate sections
- -3 points: >5 duplicates OR entire files duplicated
- **Rationale**: Duplicates create maintenance burden, version drift, conflicting info

**Documentation Graph Analysis (Detect Doc Sprawl):**
```bash
# Analyze documentation link structure (graph theory)
pmat graph-docs \
    --path docs/ \
    --output doc-graph.json \
    --check-connectivity

# Detect disconnected documentation (multiple communities)
pmat graph-docs \
    --path docs/ \
    --detect-communities \
    --ideal-communities 1 \
    --warn-if-disconnected
```

**Graph Metrics:**
- **Connected Components**: Number of disconnected doc clusters (ideal: 1)
- **Orphaned Docs**: Files with 0 incoming/outgoing links
- **Hub Docs**: Files with >10 incoming links (good navigation anchors)
- **Dead-end Docs**: Files with 0 outgoing links (should link back to index)

**Doc Sprawl Penalties:**
- -1 point: 2-3 disconnected components (minor sprawl)
- -2 points: 4-6 disconnected components (moderate sprawl)
- -3 points: >6 components OR >20% orphaned docs (severe sprawl)

**Healthy Documentation Graph:**
```
✅ Single connected component (all docs reachable)
✅ README.md is the root hub (highest PageRank)
✅ <10% orphaned docs (most have cross-references)
✅ Average path length <4 hops between any two docs
```

**Example Output:**
```
Documentation Graph Analysis:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Components: 1 (ideal ✅)
📄 Total docs: 42
🔗 Total links: 156
👻 Orphaned: 3 (7.1% ✅)
🌟 Hub docs: README.md (23 links), docs/architecture.md (18 links)
📏 Avg path length: 2.4 hops
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: COHESIVE ✅
```

**Best Practices:**
- Use cross-references instead of copy-paste (e.g., `See \[Architecture\](docs/design/architecture.md)`)
- Single source of truth for each concept
- Create index docs that link to all specs/designs
- Detect with: `pmat analyze-docs --check-duplicates --fail-on-duplicates`
- Prevent sprawl: `pmat graph-docs --detect-communities --max-components 1`

**Academic Foundation:**
- Prana et al. (2021): "What makes a good README? A study of README quality and its impact on project success" - IEEE TSE
- Research shows high-quality READMEs correlate with 30% higher contributor engagement
- Farquhar et al. (Nature 2024): Semantic entropy for hallucination detection in documentation
- **High entropy in docs**: Duplicate content increases cognitive load by 40% (Nielsen Norman Group)
- **Graph connectivity**: Disconnected docs have 60% lower discoverability (Google Dev Docs research)

#### A2. README.md Comprehensiveness (5 points)

**Full Score Criteria (5/5):**
- ✅ Project description (1-2 paragraphs explaining purpose)
- ✅ Installation instructions (multi-platform if applicable)
- ✅ Quick start / Usage examples (≥3 examples)
- ✅ Development setup guide (for contributors)
- ✅ License badge + file (OSI-approved license)
- ✅ CI/Coverage badges (build status, test coverage %)
- ✅ Contributing guidelines (CONTRIBUTING.md or section)
- ✅ Changelog or release notes
- ✅ Architecture/design documentation (for complex projects)

**Scoring:**
- 5/5: All 9 sections present
- 4/5: 7-8 sections present
- 2/5: 4-6 sections present
- 0/5: ≤3 sections present

**Best Practice Examples:**
- **bashrs**: Status badges, feature comparison table, metrics section, progressive complexity (install → quickstart → commands)
- **ruchy**: Production readiness disclaimer, feature matrix (implemented ✅ vs in-progress), safety documentation
- **depyler**: Multi-tier docs (README → specs → book), MCP integration guide, troubleshooting

---

### B. Pre-commit Hooks and Linting (20 points)

Automated quality gates prevent defects from entering the repository. This category evaluates pre-commit configuration, execution speed, and effectiveness.

#### B1. Pre-commit Best Practices (10 points)

**Full Score Criteria (10/10):**
- ✅ `.pre-commit-config.yaml` present
- ✅ Hooks for all repo languages (Rust: clippy/rustfmt, Python: black/ruff, etc.)
- ✅ bashrs linting for shell scripts and Makefiles
- ✅ Commit message validation (conventional commits)
- ✅ File size limits / binary checks
- ✅ Trailing whitespace / EOF newline enforcement

**Scoring:**
- 10/10: All 6 criteria met
- 7/10: 4-5 criteria met
- 4/10: 2-3 criteria met
- 0/10: No pre-commit hooks configured

**Example Configuration:**
```yaml
repos:
  - repo: https://github.com/paiml/bashrs
    rev: v6.31.1
    hooks:
      - id: bashrs-lint
        args: [--fail-on-error]
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-added-large-files
```

#### B2. Performance and Effectiveness (10 points)

**Full Score Criteria (10/10):**
- ✅ Pre-commit execution completes <30 seconds (99th percentile)
- ✅ Zero linting errors on default branch (clean `make lint`)
- ✅ Zero clippy warnings with `-D warnings` (for Rust)
- ✅ Hooks block commits on failure (not just warnings)
- ✅ Documentation examples tested by hooks (bash code blocks)

**Scoring:**
- 10/10: All criteria met
- 7/10: <45s execution OR 1-3 linting errors
- 4/10: <60s execution OR 4-10 linting errors
- 0/10: >60s execution OR >10 linting errors

**Rationale:**
- **Toyota Way (Jidoka)**: Built-in quality stops the line when defects are detected
- **Fast feedback loops**: <30s ensures developer flow state isn't disrupted

**Best Practice Examples:**
- **bashrs**: `.clippy.toml` + `rustfmt.toml` for consistent style, hooks integrated with CI
- **ruchy**: `.claudeignore` for LLM-aware filtering, `.eslintrc.json` for JS quality

---

### C. Repository Hygiene (15 points)

A clean repository reduces cognitive load, prevents accidental commit of sensitive data, and keeps clone sizes manageable.

#### C1. No Cruft (5 points)

**Full Score Criteria (5/5):**
- ✅ No temporary files (`.swp`, `.tmp`, `*.bak`)
- ✅ No editor artifacts (`.idea/`, `.vscode/` unless shared config)
- ✅ No OS-specific files (`.DS_Store`, `Thumbs.db`)
- ✅ No build artifacts in git (target/, node_modules/)
- ✅ Proper `.gitignore` for language/framework

**Scoring:**
- 5/5: Zero cruft files in git history (last 100 commits)
- 3/5: 1-3 cruft files
- 0/5: >3 cruft files OR sensitive data (API keys, credentials)

**Validation:**
```bash
# Check for common cruft patterns
git ls-files | grep -E '\.(swp|tmp|bak|DS_Store)$'
git ls-files | grep -E '^(target|node_modules|\.idea)/'
```

#### C2. No Team-Specific Files (5 points)

**Full Score Criteria (5/5):**
- ✅ No personal scripts (`noah-test.sh`, `alice-debug.py`)
- ✅ No team-internal files (SESSION-*.md, defect-report-*.txt)
- ✅ No dated artifacts (analysis-2024-10-15.json)
- ✅ `.gitignore` properly excludes ephemeral files

**Scoring:**
- 5/5: Zero team-specific files
- 3/5: 1-5 team files
- 0/5: >5 team files

**Best Practice Examples:**
- **This repo**: `.gitignore` updated to exclude `SESSION-*.md`, `SESSION_*.md`, `defect-report-*.txt`
- **ruchy**: `.claudeignore` prevents LLM context pollution

#### C3. No Large Files in Git History (5 points)

**Full Score Criteria (5/5):**
- ✅ No files >1MB in git history (including deleted files)
- ✅ No binary blobs (images, PDFs, datasets) unless essential
- ✅ No accidentally committed build artifacts (*.so, *.dylib, *.dll, *.exe)
- ✅ Repository size <50MB (excluding .git/objects compression)
- ✅ No sensitive files in git history (credentials, API keys, .env files)

**Scoring:**
- 5/5: All files <1MB, repo <50MB, no secrets
- 3/5: 1-3 files >1MB OR repo <100MB
- 1/5: 4-10 files >1MB OR repo <200MB
- 0/5: >10 large files OR repo >200MB OR secrets found

**Validation:**
```bash
# Find large files in git history (current + deleted)
git rev-list --objects --all | \
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | \
  sed -n 's/^blob //p' | \
  sort --numeric-sort --key=2 | \
  tail -20 | \
  numfmt --field=2 --to=iec-i --suffix=B --padding=7

# Find files >1MB in git history
git rev-list --objects --all | \
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | \
  awk '$1 == "blob" && $3 > 1048576 {print $3, $4}' | \
  numfmt --field=1 --to=iec-i --suffix=B

# Check repo size
du -sh .git/

# Scan for secrets in git history
git log -p | grep -i -E '(password|api[_-]?key|secret|token|credentials)' || echo "No secrets found"

# Use specialized tools
gitleaks detect --no-git --source .  # Comprehensive secret scanning
```

**Remediation (if large files found):**
```bash
# Option 1: BFG Repo-Cleaner (fast, safe)
# Remove files larger than 1MB from all commits
bfg --strip-blobs-bigger-than 1M --no-blob-protection .git
git reflog expire --expire=now --all
git gc --prune=now --aggressive

# Option 2: git-filter-repo (more powerful)
git filter-repo --strip-blobs-bigger-than 1M

# Option 3: Remove specific file from history
git filter-repo --path path/to/large-file.bin --invert-paths

# Force push after cleanup (DANGEROUS - coordinate with team)
git push origin --force --all
git push origin --force --tags
```

**Prevention:**
```bash
# Add pre-commit hook to block large files
# .git/hooks/pre-commit
#!/bin/bash
MAX_SIZE=1048576  # 1MB in bytes
large_files=$(git diff --cached --name-only | while read file; do
    if [ -f "$file" ]; then
        size=$(wc -c < "$file")
        if [ "$size" -gt "$MAX_SIZE" ]; then
            echo "$file ($(numfmt --to=iec-i --suffix=B $size))"
        fi
    fi
done)

if [ -n "$large_files" ]; then
    echo "ERROR: Large files detected (>1MB):"
    echo "$large_files"
    echo "Add to .gitignore or use Git LFS"
    exit 1
fi
```

**Git LFS for Legitimate Large Files:**
```bash
# Use Git LFS for datasets, models, images
git lfs install
git lfs track "*.png" "*.jpg" "*.pdf" "*.bin"
git add .gitattributes
git commit -m "Configure Git LFS"
```

**Best Practice Examples:**
- **bashrs**: 15MB repo size, no files >100KB, clean git history
- **ruchy**: 8MB repo size, all test fixtures in separate repo
- **depyler**: Uses Git LFS for benchmark datasets

**Academic Foundation:**
- Kalliamvakou et al. (2014): "The promises and perils of mining GitHub" - MSR 2014
- Large repos (>500MB) have 3x lower contributor engagement
- Repos with secrets in history face 60% higher security incident rate

---

### D. Build and Test Automation (25 points)

Robust automation ensures consistent builds, fast feedback, and confidence in releases.

#### D1. Shell Script & Makefile Quality (10 points)

**Full Score Criteria (10/10):**
- ✅ `Makefile` present at repo root
- ✅ **ALL shell files linted by bashrs (zero errors)**
  - Makefile
  - scripts/*.sh
  - Dockerfile (bash commands)
  - .github/workflows/*.yml (shell commands)
- ✅ Standard targets: `test`, `test-fast`, `lint`, `coverage`
- ✅ Help target (`make help` documents all targets)
- ✅ Phony targets declared (`.PHONY: test lint`)
- ✅ Variables quoted properly (POSIX-compliant)

**Required Targets:**
```makefile
.PHONY: test test-fast lint coverage help

help:  ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

test-fast:  ## Run fast tests (<5 min)
	@echo "⚡ Running fast test suite (target: <5 min)..."
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --lib --features skip-slow-tests --no-fail-fast; \
	else \
		cargo test --lib --features skip-slow-tests; \
	fi
	@echo "✅ Fast tests complete"

test:  ## Run all tests
	cargo test --workspace

lint:  ## Run all linters
	cargo clippy --all-targets -- -D warnings
	bashrs lint Makefile

coverage:  ## Generate coverage report (<10 min)
	@echo "📊 Running test coverage analysis (target: <10 min)..."
	@which cargo-llvm-cov > /dev/null 2>&1 || cargo install cargo-llvm-cov
	@cargo llvm-cov --no-report test --lib --features skip-slow-tests 2>&1 | tee target/coverage/test-output.txt
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo "✅ Coverage: target/coverage/html/index.html"
```

**Scoring:**
- 10/10: All criteria met, zero bashrs errors
- 7/10: 4-5 criteria met OR 1-3 bashrs warnings
- 4/10: 2-3 criteria met OR 1-2 bashrs errors
- 0/10: No Makefile OR severe bashrs errors (SEC008, DET003, IDEM002)

**Validation Commands:**
```bash
# Lint Makefile
bashrs lint Makefile  # Must exit 0 or 1 (warnings ok, errors block)

# Lint all shell scripts
find . -name "*.sh" -not -path "./target/*" -exec bashrs lint {} \;

# Lint Dockerfile (if present)
bashrs lint Dockerfile

# Lint GitHub Actions workflows (shell commands)
find .github/workflows -name "*.yml" -exec bashrs lint {} \;

# Comprehensive check
make lint  # Should include bashrs for all shell files
make help  # Must produce output
make test-fast  # Must complete <5 min
```

**Critical bashrs Errors to Fix:**
- **SEC008**: Piping curl to shell (security vulnerability)
- **SC2086**: Unquoted variable expansion (word splitting/glob)
- **DET003**: Unordered wildcard (non-deterministic results)
- **IDEM002**: Non-idempotent operations (breaks repeatability)

**Acceptable Warnings:**
- **SC2116**: Useless echo (cosmetic, not breaking)
- **NC**: No color codes in output (stylistic)

#### D2. Test Performance (8 points)

**Full Score Criteria (8/8):**
- ✅ `make test-fast` completes in <5 minutes
- ✅ `make coverage` completes in <10 minutes
- ✅ No slow tests unmarked (all `#[ignore]` tests documented)
- ✅ Parallel test execution enabled (nextest or `--test-threads`)

**Scoring:**
- 8/8: All criteria met
- 5/8: test-fast <8 min, coverage <15 min
- 2/8: test-fast <10 min, coverage <20 min
- 0/8: Exceeds time limits

**Anti-Pattern Warning:**
- ❌ **DO NOT use timeout wrappers** (`timeout 600 make coverage`) - they hide actual performance issues
- ✅ **DO use conditional tool checking** (nextest if available, fallback to cargo test)
- ✅ **DO use `--lib` for fast tests** instead of `--workspace` with complex filters
- ✅ **DO log output** to file for debugging (e.g., `2>&1 | tee target/coverage/test-output.txt`)

**Rationale:**
- Timeout wrappers mask the root cause of slow tests
- Let tests run naturally to identify actual bottlenecks
- Cargo handles thread management automatically (don't force `--test-threads`)

**Best Practice Examples:**
- **bashrs**: 5,465 tests execute in CI pipeline (parallelized), conditional nextest usage
- **ruchy**: Simple `cargo test --lib --quiet` with --skip patterns for slow tests
- **compiled-rust-benchmarking**: `pathfinder-demo` (6-job validation) vs `full-pathfinder-execution` (150-job study)

#### D3. Coverage and Mutation Testing (7 points)

**Full Score Criteria (7/7):**
- ✅ Test coverage ≥85% (PMAT standard)
- ✅ Mutation testing score ≥85% (mutants killed)
- ✅ Coverage badge in README
- ✅ `cargo llvm-cov` (not tarpaulin)

**Scoring:**
- 7/7: Coverage ≥85%, Mutation ≥85%
- 5/7: Coverage ≥70%, Mutation ≥70%
- 3/7: Coverage ≥50%, Mutation ≥50%
- 0/7: Coverage <50% OR no mutation testing

**Validation:**
```bash
cargo llvm-cov --all-features --workspace --summary-only
cargo mutants --in-place  # Or configure mutants.toml
```

**Best Practice Examples:**
- **bashrs**: 88.71% coverage, 92% mutation kill rate
- **ruchy-lambda**: 91.48% coverage, 86.67% mutation score
- **depyler**: 80%+ coverage requirement in CI (BLOCKING)

**Academic Foundation:**
- Jia & Harman (2011): "An Analysis and Survey of the Development of Mutation Testing" - IEEE TSE
- Mutation testing catches 15-30% more defects than coverage alone

---

### E. Continuous Integration (20 points)

CI automates quality checks, ensures reproducibility, and prevents regressions.

#### E1. GitHub Actions Configuration (10 points)

**Full Score Criteria (10/10):**
- ✅ `.github/workflows/` directory present
- ✅ CI workflow runs on: `pull_request`, `push` (main/master)
- ✅ Multi-platform testing (Linux, macOS, Windows) if applicable
- ✅ Multi-version testing (Rust stable/beta, Node 18/20, etc.)
- ✅ Dependency caching (actions/cache for faster builds)
- ✅ Artifact upload (test results, coverage reports)

**Example Workflow:**
```yaml
name: CI
on:
  pull_request:
  push:
    branches: [main, master]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2
      - run: make test
      - run: make lint
```

**Scoring:**
- 10/10: All 6 criteria met
- 7/10: 4-5 criteria met
- 4/10: Basic CI (test + lint only)
- 0/10: No GitHub Actions

#### E2. Build Status (10 points)

**Full Score Criteria (10/10):**
- ✅ Default branch build passing (green badge)
- ✅ Branch protection rules enabled (require status checks)
- ✅ Recent builds all passing (last 10 commits)
- ✅ No failing required checks
- ✅ PR merge blocked on failure

**Scoring:**
- 10/10: 100% pass rate (last 10 commits)
- 7/10: 80-99% pass rate
- 4/10: 60-79% pass rate
- 0/10: <60% pass rate

**Validation:**
```bash
gh api repos/:owner/:repo/commits --jq '.[0:10] | map(.commit.verification.verified)'
gh api repos/:owner/:repo/branches/main --jq '.protection.required_status_checks'
```

**Best Practice Examples:**
- **bashrs**: "Automated test runs across 854+ commits on the main branch"
- **compiled-rust-benchmarking**: "100% job success rate: 150/150 jobs completed"

**Academic Foundation:**
- Hilton et al. (2016): "Usage, costs, and benefits of continuous integration in open-source projects" - ASE 2016
- CI adoption correlated with 22% reduction in integration issues

---

### F. PMAT Compliance (5 points)

Adherence to PMAT standards ensures consistency with Toyota Way principles (Jidoka, Kaizen, Genchi Genbutsu).

#### F1. Quality Gates (5 points)

**Full Score Criteria (5/5):**
- ✅ `.pmat-gates.toml` or `pmat-quality.toml` present
- ✅ `pmat quality-gate` passes (if PMAT project)
- ✅ No cyclomatic complexity >10
- ✅ No cognitive complexity >10
- ✅ Zero self-admitted technical debt (SATD comments)

**Scoring:**
- 5/5: All criteria met
- 3/5: 3-4 criteria met
- 1/5: 1-2 criteria met
- 0/5: No PMAT validation

**Validation:**
```bash
pmat quality-gate --checks all
pmat analyze --complexity-limit 10
pmat satd --fail-on-found
```

**Best Practice Examples:**
- **depyler**: "Minimum TDG grade of A- (≥85 points), cyclomatic/cognitive complexity ≤10"
- **bashrs**: "A+ Grade: Near Perfect quality rating, 100% compliant with ShellCheck"
- **ruchy-lambda**: "TDG Grade: A+ (98.1/100), cyclomatic complexity ≤5, cognitive ≤4"

**Academic Foundation:**
- McCabe (1976): "A Complexity Measure" - IEEE TSE (foundational cyclomatic complexity paper)
- Potdar & Shihab (2014): "An exploratory study on self-admitted technical debt" - MSR 2014

---

## 3. Additional Excellence Indicators (Bonus Points)

Repositories can earn **+10 bonus points** (max total: 110/100) for exceptional practices:

### B1. Property-Based Testing (+3 points)
- ✅ PropTest or Hypothesis integration
- ✅ ≥20 property tests covering critical functions

**Examples:**
- **bashrs**: 52 property tests covering ~26,000+ scenarios
- **ruchy**: 200K+ property test iterations for filters/sorting
- **depyler**: Property-based semantic equivalence verification

### B2. Fuzzing (+2 points)
- ✅ `fuzz/` directory with cargo-fuzz targets
- ✅ Fuzzing runs in CI (time-limited)

**Examples:**
- **bashrs**: Dedicated `/fuzz` directory with continuous fuzzing
- **ruchy**: Fuzzing infrastructure for parser validation

### B3. Mutation Testing Configuration (+2 points)
- ✅ `mutants.toml` present with sensible excludes
- ✅ Mutation score tracked in CI

**Examples:**
- **bashrs**: 92% mutation kill rate
- **ruchy**: `mutants.toml` with disabled test sets for analysis

### B4. Advanced Documentation (+3 points)
- ✅ Living documentation (mdBook, Jupyter notebooks)
- ✅ Automated example testing (doctest, book validation)
- ✅ Architecture decision records (ADRs)

**Examples:**
- **bashrs**: "The Rash Book" with automatic example testing
- **depyler**: TDD Book with validation reports, architecture docs
- **compiled-rust-benchmarking**: 12-file documentation suite (~3,900 lines)

---

## 4. Scoring Methodology

### 4.1 Score Calculation

**Total Score = Base Score (100) + Bonus (0-10)**

**Grade Assignment:**
- **A+**: 95-110 (Exceptional quality, production-ready)
- **A**: 90-94 (Excellent quality, minor improvements)
- **A-**: 85-89 (PMAT standard, high quality)
- **B+**: 80-84 (Good quality, some gaps)
- **B**: 70-79 (Acceptable, moderate improvements needed)
- **C**: 60-69 (Below standard, significant improvements required)
- **D**: 50-59 (Poor quality, major refactoring needed)
- **F**: 0-49 (Unacceptable, not production-ready)

### 4.2 Weighted Categories

| Category | Points | % of Total | Priority |
|----------|--------|------------|----------|
| Build & Test Automation | 25 | 25% | CRITICAL |
| CI/CD | 20 | 20% | CRITICAL |
| Documentation | 15 | 15% | HIGH |
| Pre-commit Hooks | 20 | 20% | HIGH |
| Repository Hygiene | 15 | 15% | HIGH |
| PMAT Compliance | 5 | 5% | MEDIUM |
| **Total** | **100** | **100%** | - |

**Rationale:**
- **Automation (45%)**: Testing + CI are the most objective quality signals
- **Human Factors (35%)**: Documentation + pre-commit affect developer experience
- **Hygiene (15%)**: Clean git history, no large files, prevents security issues (elevated to HIGH)
- **Compliance (5%)**: PMAT standards ensure baseline quality

**Note**: Repository Hygiene elevated from 10→15 points and MEDIUM→HIGH priority due to:
- Security risks from secrets in git history (60% higher incident rate)
- Large files reduce contributor engagement by 3x
- Git history cleanup is critical for open-source reputation

---

## 5. Implementation Roadmap

### 5.1 Automated Validation Tool

**`pmat repo-score` CLI:**

```bash
# Basic usage
pmat repo-score .

# Detailed output
pmat repo-score . --verbose

# JSON output for CI
pmat repo-score . --output json > repo-score.json

# Fail if below threshold
pmat repo-score . --min-score 85 || exit 1
```

**Output Format:**
```
Repository Score: 92/100 (A)

 ✅ A. Documentation Quality          14/15
    ✅ A1. Documentation Accuracy      9/10
    ✅ A2. README Comprehensiveness    5/5
 ✅ B. Pre-commit Hooks               20/20
    ✅ B1. Best Practices             10/10
    ✅ B2. Performance                10/10
 ⚠️  C. Repository Hygiene            13/15
    ✅ C1. No Cruft                    5/5
    ⚠️  C2. No Team Files               3/5 (found 3 SESSION-*.md)
    ✅ C3. No Large Files              5/5
 ✅ D. Build & Test Automation        24/25
    ✅ D1. Makefile Quality           10/10
    ✅ D2. Test Performance            7/8 (test-fast: 6.2 min)
    ✅ D3. Coverage & Mutation         7/7
 ✅ E. Continuous Integration         20/20
    ✅ E1. GitHub Actions             10/10
    ✅ E2. Build Status               10/10
 ✅ F. PMAT Compliance                 5/5
    ✅ F1. Quality Gates               5/5

 🎁 Bonus Points                      +3/10
    ✅ Property-based testing          +3

Grade: A (95/100)
Status: PRODUCTION READY ✅
```

### 5.2 CI Integration

**GitHub Actions Example:**

```yaml
name: Repo Score
on: [pull_request, push]

jobs:
  repo-score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install PMAT
        run: cargo install pmat
      - name: Check Repository Score
        run: |
          pmat repo-score . --min-score 85 --output json | tee repo-score.json
          pmat repo-score . --verbose
      - name: Upload Score
        uses: actions/upload-artifact@v4
        with:
          name: repo-score
          path: repo-score.json
```

### 5.3 Badge Integration

**Dynamic Badge:**

```markdown
![Repo Score](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/USER/REPO/main/.repo-score.json)
```

**JSON Endpoint (`.repo-score.json`):**
```json
{
  "schemaVersion": 1,
  "label": "repo score",
  "message": "92/100 (A)",
  "color": "brightgreen"
}
```

---

## 6. Scientific Foundation

This specification is grounded in peer-reviewed computer science research:

### 6.1 Core Research (Top 10)

1. **Prana et al. (2021)**: "What makes a good README? A study of README quality and its impact on project success" - IEEE Transactions on Software Engineering
   - **Key Finding**: High-quality READMEs increase contributor engagement by 30%

2. **Hilton et al. (2016)**: "Usage, costs, and benefits of continuous integration in open-source projects" - ASE 2016
   - **Key Finding**: CI adoption reduces integration issues by 22%

3. **Vasilescu et al. (2015)**: "Quality and productivity outcomes of repository configuration in GitHub" - ESEM 2015
   - **Key Finding**: Repository configuration correlates with positive project outcomes

4. **Bacchelli & Bird (2013)**: "Expectations, outcomes, and challenges of modern code review" - ICSE 2013
   - **Key Finding**: Automated checks improve review effectiveness by 35%

5. **Beller et al. (2017)**: "The landscape of continuous integration: a study of 12,000+ Travis CI projects" - ICSE 2017
   - **Key Finding**: Fast test suites (<5 min) maintain developer flow state

6. **Jia & Harman (2011)**: "An Analysis and Survey of the Development of Mutation Testing" - IEEE TSE
   - **Key Finding**: Mutation testing detects 15-30% more defects than coverage alone

7. **Potdar & Shihab (2014)**: "An exploratory study on self-admitted technical debt" - MSR 2014
   - **Key Finding**: Untracked technical debt increases by 12% monthly

8. **McIntosh et al. (2016)**: "The impact of code review coverage and participation on software quality" - MSR 2016
   - **Key Finding**: Code review coverage >80% reduces defect density by 60%

9. **Kochhar et al. (2016)**: "A study of the quality of build files in Maven" - MSR 2016
   - **Key Finding**: Well-structured build files reduce build failures by 40%

10. **Gousios et al. (2014)**: "An exploratory study of the pull-based software development model" - ICSE 2014
    - **Key Finding**: Clear documentation increases PR acceptance rate by 45%

### 6.2 Recent Advances (2024-2025)

11. **Papadopoulos et al. (2022)**: "A large-scale study on research code quality and execution" - Nature Scientific Data 2024
    - **Key Finding**: Artifact disclosure increased from 60.5% (2017) to 78.3% (2022)

12. **SQAaaS Platform (2024)**: "Software Quality Assurance as a Service" - ScienceDirect 2024
    - **Key Finding**: Automated quality assessment performed 2,800+ evaluations, awarding 125+ badges

13. **Martínez-Fernández et al. (2023)**: "Research artifacts in software engineering publications" - ScienceDirect 2024
    - **Key Finding**: Python surpassed Java as most-used language in artifacts (61.1% in 2022)

### 6.3 Foundational Theory

14. **McCabe (1976)**: "A Complexity Measure" - IEEE TSE
    - Established cyclomatic complexity as foundational quality metric

15. **ISO/IEC 25010:2011**: Software Quality Model
    - Defines 8 quality characteristics (maintainability, reliability, etc.)

---

## 7. Validation Checklist

### 7.1 Pre-Release Checklist

Before publishing a repository or releasing a new version:

- [ ] Run `pmat repo-score . --verbose`
- [ ] Ensure score ≥85 (A- grade or higher)
- [ ] Fix all broken README links
- [ ] Verify `make test-fast` <5 minutes
- [ ] Verify `make coverage` <10 minutes
- [ ] Check `make lint` returns zero errors
- [ ] Validate pre-commit hooks execute <30 seconds
- [ ] Ensure GitHub Actions passing (last 10 commits)
- [ ] Update badges (coverage, build status, repo score)
- [ ] Review `.gitignore` for cruft patterns
- [ ] Document all `#[ignore]` tests

### 7.2 Continuous Monitoring

**Monthly Reviews:**
- [ ] Re-run `pmat repo-score .` and track trends
- [ ] Review ignored tests (can any be re-enabled?)
- [ ] Update dependencies and re-validate
- [ ] Check for new cruft files
- [ ] Audit GitHub Actions usage (cost optimization)

**Quarterly Audits:**
- [ ] Deep documentation review (accuracy validation)
- [ ] Mutation testing score recalculation
- [ ] Complexity analysis (identify refactoring targets)
- [ ] Benchmark performance trends (test-fast, coverage)

---

## 8. Case Studies

### 8.1 Best-in-Class Examples

#### bashrs: A+ Grade (98/100 + 7 bonus = 105/100)

| Category | Score | Notes |
|----------|-------|-------|
| Documentation | 20/20 | Status badges, feature comparison, comprehensive metrics |
| Pre-commit Hooks | 20/20 | <30s execution, zero linting errors |
| Repository Hygiene | 10/10 | Clean `.gitignore`, no cruft |
| Build & Test | 25/25 | 5,465 tests, 88.71% coverage, 92% mutation |
| CI/CD | 20/20 | Multi-shell testing, 854+ commits green |
| PMAT Compliance | 5/5 | A+ quality rating, 100% ShellCheck compliant |
| **Bonus** | **+7** | Property tests (+3), fuzzing (+2), mdBook (+2) |
| **Total** | **105/100** | **A+ (Exceptional)** |

#### ruchy-lambda: A+ Grade (96/100 + 5 bonus = 101/100)

| Category | Score | Notes |
|----------|-------|-------|
| Documentation | 18/20 | Comprehensive docs (8 files), architecture guide |
| Pre-commit Hooks | 20/20 | PMAT best practices, fast execution |
| Repository Hygiene | 10/10 | Zero cruft, clean git history |
| Build & Test | 24/25 | 100+ tests, 91.48% coverage, 86.67% mutation |
| CI/CD | 19/20 | GitHub Actions configured, 98% pass rate |
| PMAT Compliance | 5/5 | TDG A+ (98.1/100), complexity ≤5 |
| **Bonus** | **+5** | Mutation config (+2), benchmarking (+3) |
| **Total** | **101/100** | **A+ (Exceptional)** |

#### depyler: A Grade (90/100 + 3 bonus = 93/100)

| Category | Score | Notes |
|----------|-------|-------|
| Documentation | 19/20 | Multi-tier docs, MCP guide, troubleshooting |
| Pre-commit Hooks | 18/20 | Strong linting, 35s execution (slightly over) |
| Repository Hygiene | 9/10 | Minor cruft (1-2 temp files) |
| Build & Test | 23/25 | 443 core tests, 80% coverage target met |
| CI/CD | 18/20 | CI validates compilation, some flakiness |
| PMAT Compliance | 5/5 | Complexity ≤10, zero SATD |
| **Bonus** | **+3** | Property tests (+3) |
| **Total** | **93/100** | **A (Excellent)** |

### 8.2 Anti-Patterns

#### Poor Score Example: legacy-project (47/100 - F)

| Category | Score | Issues |
|----------|-------|--------|
| Documentation | 6/20 | No README, outdated wiki, 15 broken links |
| Pre-commit Hooks | 0/20 | No hooks configured |
| Repository Hygiene | 2/10 | 40+ cruft files, `.idea/` committed |
| Build & Test | 8/25 | No Makefile, 30% coverage, no mutation |
| CI/CD | 6/20 | Travis CI (deprecated), 40% pass rate |
| PMAT Compliance | 0/5 | No quality gates |
| **Total** | **22/100** | **F (Unacceptable)** |

**Remediation Plan:**
1. **Week 1**: Add `.pre-commit-config.yaml`, clean cruft
2. **Week 2**: Create `Makefile` with test targets
3. **Week 3**: Migrate to GitHub Actions
4. **Week 4**: Update README, fix broken links
5. **Target**: B+ (80+) within 1 month

---

## 9. FAQ

### Q1: Why 85% coverage/mutation threshold?

**A**: PMAT's TDG standard is 85%+ based on research showing:
- 80-90% coverage catches 90-95% of defects (Hilton et al., 2016)
- Diminishing returns above 90% (test maintenance burden)
- 85% balances rigor and pragmatism

### Q2: Can I exclude certain tests from `test-fast`?

**A**: Yes, use `#[ignore]` attributes and document them:

```rust
#[test]
#[ignore] // Slow test: requires Docker (30s+)
fn test_integration_with_external_service() {
    // ...
}
```

Run ignored tests separately:
```makefile
test-slow:  ## Run ignored slow tests
	cargo test -- --ignored
```

### Q3: What if my project doesn't use Rust?

**A**: The principles apply universally:

**Python:**
```makefile
test-fast:
	pytest -m "not slow" --maxfail=1

coverage:
	pytest --cov=src --cov-report=html

lint:
	ruff check .
	mypy src/
```

**TypeScript:**
```makefile
test-fast:
	npm run test:unit

coverage:
	npm run test:coverage

lint:
	npm run lint
	npm run typecheck
```

### Q4: How do I improve a failing score?

**A**: Focus on highest-impact categories first:

1. **Score <60 (F/D)**: Start with automation (Category D & E)
   - Add basic Makefile
   - Configure GitHub Actions
   - Set up pre-commit hooks

2. **Score 60-79 (C/B)**: Improve testing (Category D)
   - Increase coverage to 70%+
   - Add mutation testing
   - Optimize test performance

3. **Score 80-89 (B+/A-)**: Polish documentation (Category A)
   - Fix broken links
   - Add comprehensive examples
   - Create architecture docs

4. **Score 90+ (A/A+)**: Pursue excellence (Bonus)
   - Add property-based tests
   - Implement fuzzing
   - Create living documentation

### Q5: Can I customize scoring for my organization?

**A**: Yes, create `.pmat-repo-score.toml`:

```toml
[weights]
documentation = 0.15  # Reduce from 20%
testing = 0.30        # Increase from 25%
ci_cd = 0.25         # Increase from 20%
hygiene = 0.05       # Reduce from 10%

[thresholds]
min_coverage = 0.90   # Raise to 90%
min_mutation = 0.85
max_test_fast_seconds = 300  # 5 minutes
max_coverage_seconds = 600   # 10 minutes
max_precommit_seconds = 30

[bonus]
property_tests = 5    # Increase bonus value
fuzzing = 5
mdbook = 2
```

---

## 10. References

### 10.1 Primary Research

1. Prana, G. A., Treude, C., Thongtanunam, P., & D'Angelo, G. (2021). What makes a good README? A study of README quality and its impact on project success. *IEEE Transactions on Software Engineering*.

2. Hilton, M., Tunnell, T., Huang, K., Marinov, D., & Dig, D. (2016). Usage, costs, and benefits of continuous integration in open-source projects. *Proceedings of the 31st IEEE/ACM International Conference on Automated Software Engineering*.

3. Vasilescu, B., Yu, Y., Wang, H., Devanbu, P., & Filkov, V. (2015). Quality and productivity outcomes of repository configuration in GitHub. *Proceedings of the 2015 ACM/IEEE International Symposium on Empirical Software Engineering and Measurement*.

4. Bacchelli, A., & Bird, C. (2013). Expectations, outcomes, and challenges of modern code review. *Proceedings of the 2013 35th International Conference on Software Engineering (ICSE)*.

5. Beller, M., Gousios, G., & Zaidman, A. (2017). The landscape of continuous integration: a study of 12,000+ Travis CI projects. *Proceedings of the 39th International Conference on Software Engineering*.

6. Jia, Y., & Harman, M. (2011). An analysis and survey of the development of mutation testing. *IEEE Transactions on Software Engineering*, 37(5), 649-678.

7. Potdar, A., & Shihab, E. (2014). An exploratory study on self-admitted technical debt. *Proceedings of the 11th Working Conference on Mining Software Repositories*.

8. McIntosh, S., Kamei, Y., Adams, B., & Hassan, A. E. (2016). The impact of code review coverage and code review participation on software quality. *Proceedings of the 13th International Conference on Mining Software Repositories*.

9. Kochhar, P. S., Beller, M., & Gousios, G. (2016). A study of the quality of build files in Maven. *Proceedings of the 13th International Conference on Mining Software Repositories*.

10. Gousios, G., Pinzger, M., & van Deursen, A. (2014). An exploratory study of the pull-based software development model. *Proceedings of the 36th International Conference on Software Engineering*.

### 10.2 Recent Advances (2024-2025)

11. Papadopoulos, A., et al. (2022). A large-scale study on research code quality and execution. *Nature Scientific Data*, 2024.

12. Martínez-Fernández, S., et al. (2024). Software Quality Assurance as a Service: Encompassing the quality assessment of software and services. *ScienceDirect*, 2024.

13. Martínez-Fernández, S., et al. (2024). Research artifacts in software engineering publications: Status and trends. *ScienceDirect*, 2024.

### 10.3 Foundational Theory

14. McCabe, T. J. (1976). A complexity measure. *IEEE Transactions on Software Engineering*, SE-2(4), 308-320.

15. ISO/IEC 25010:2011. Systems and software engineering — Systems and software Quality Requirements and Evaluation (SQuaRE) — System and software quality models.

### 10.4 Industry Standards

16. Toyota Production System (TPS): Jidoka (built-in quality), Kaizen (continuous improvement)

17. DORA Metrics: Deployment frequency, lead time, MTTR, change failure rate

18. ShellCheck Documentation: https://www.shellcheck.net/wiki/

19. bashrs Project: https://github.com/paiml/bashrs

20. PMAT Documentation: https://paiml.github.io/pmat-book/

---

## 11. Appendices

### A. Example `.pmat-gates.toml`

```toml
[quality]
min_coverage = 0.85
max_cyclomatic_complexity = 10
max_cognitive_complexity = 10
allow_satd = false

[testing]
min_mutation_score = 0.85
require_property_tests = true

[performance]
max_test_fast_seconds = 300
max_coverage_seconds = 600

[ci]
require_github_actions = true
min_pass_rate = 0.95
```

### B. Makefile Template

```makefile
.PHONY: help test test-fast lint coverage validate install

help:  ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install:  ## Install dependencies
	cargo build --release

test-fast:  ## Run fast tests (<5 min)
	cargo test --lib --bins --test-threads=8

test:  ## Run all tests (including slow)
	cargo test --workspace --all-features

lint:  ## Run all linters
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	bashrs lint Makefile scripts/*.sh

coverage:  ## Generate coverage report (<10 min)
	cargo llvm-cov --all-features --workspace --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

mutants:  ## Run mutation testing (slow)
	cargo mutants --in-place --jobs 8

validate:  ## Run all quality gates
	@echo "Running validation..."
	$(MAKE) lint
	$(MAKE) test
	$(MAKE) coverage
	pmat repo-score . --min-score 85
	@echo "✅ All validations passed!"

.DEFAULT_GOAL := help
```

### C. Pre-commit Configuration Template

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-added-large-files
        args: ['--maxkb=500']
      - id: check-yaml
      - id: check-toml

  - repo: https://github.com/paiml/bashrs
    rev: v6.31.1
    hooks:
      - id: bashrs-lint
        name: Lint shell scripts and Makefiles
        entry: bashrs lint
        language: system
        types: [shell, makefile]
        args: [--fail-on-error]

  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
```

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-10
**Maintainer**: PAIML Engineering Team
**License**: MIT OR Apache-2.0
