# Claude Agent Guide: paiml-mcp-agent-toolkit (pmat)

This guide provides the essential operational instructions for working on the `pmat` codebase, grounded in the principles of the Toyota Way.

## 🏆 Sprint 46 Quality Perfection COMPLETE - v2.44.0

**MAJOR ACHIEVEMENT**: Sprint 46 has successfully **EXCEEDED** the 80% test coverage target, reaching **80.2%** through systematic application of Toyota Way TDD methodology.

### Key Achievements
- **Coverage**: 72.8% → **80.2%** ✅ (Target exceeded by 0.2%)
- **Tests Added**: **150 comprehensive tests** across 7 phases
- **Lines Covered**: **15,870 lines** with strict TDD methodology
- **Quality**: Zero-defect compilation maintained throughout
- **Release**: v2.44.0 published to GitHub

**Proven Methodology**: The Toyota Way TDD approach has delivered industry-leading test coverage while maintaining zero-defect standards.

## The Toyota Way: Our Guiding Philosophy

-   **Kaizen (改善): Continuous, Incremental Improvement.** We improve the codebase one file at a time. This ensures that every change is small, verifiable, and moves us toward our quality goals. Avoid large, sweeping changes.
-   **Genchi Genbutsu (現地現物): Go and See.** We don't guess where problems are. We use `pmat`'s analysis tools to find the *actual* root cause of quality issues, such as complexity hotspots or technical debt.
-   **Jidoka (自働化): Automation with a Human Touch.** We use `pmat refactor auto` to automate the creation of a refactoring plan, but an intelligent agent (you) must verify and apply the changes, ensuring correctness.

## Absolute Rules

1.  **NEVER `cd server`:** All commands **MUST** be run from the project root (`/home/noah/src/paiml-mcp-agent-toolkit`). The `Makefile` is configured to correctly handle the workspace structure.
2.  **ALWAYS Use Workspace Commands:** If you must run `cargo` commands directly, use workspace flags to target the `server` package (e.g., `cargo check --package pmat`). The `make` commands handle this for you.
3.  **Binary Location:** The `pmat` binary is ALWAYS at `./target/debug/pmat` (workspace root), NOT in `server/target/`. This is a workspace project!
4.  **NEVER Leave Stub Implementations:** This is a P0 (highest priority) rule. Never leave stub implementations with messages like "not yet implemented" or "TODO". Every feature must be fully functional. If you add a new command option or feature, you MUST implement it completely.
5.  **NEVER Add SATD Comments:** Zero tolerance for self-admitted technical debt. Never add comments like "TODO", "FIXME", "For now", "In a full implementation", etc. Every implementation must be complete.
6.  **NEVER Use Simple Heuristics:** Zero tolerance for heuristics, stubs, or approximations. Always use proper AST-based analysis, full implementations, and accurate algorithms. If a function is named `estimate_*` or uses simple pattern matching instead of proper parsing, it must be replaced with the real implementation.
7.  **NEVER Duplicate Core Logic:** There must be ONE implementation for each core feature. All providers (MCP, HTTP, CLI) must use the same underlying logic. If multiple tools need the same functionality, they must call the same service/function. No duplicate implementations allowed.
8.  **ALWAYS Dogfood via MCP First:** We MUST use our own MCP tools as the primary interface. CLI commands are secondary. This ensures we continuously improve our MCP integration and experience the tool as our users do. Use MCP tools for analysis, refactoring, quality gates, and todo creation.
9.  **ALWAYS Use PDMT for Todo Creation:** When creating todo lists or task breakdowns, you MUST use the PDMT (Pragmatic Deterministic MCP Templating) approach via MCP. This ensures deterministic, quality-enforced todo generation with proper validation commands and success criteria.

## PDMT Todo Creation (Mandatory)

For ALL todo creation and task planning, use our integrated PDMT system via **MCP first**:

```bash
# PRIMARY: Use MCP tool for PDMT todo generation (dogfooding)
# Use the pdmt_deterministic_todos MCP tool with deterministic seed
# Example: Generate todos for "Update pmcp to version 1.2.0" requirement

# FALLBACK ONLY: CLI usage when MCP is not available
pmat pdmt-todos "your requirement description" --granularity medium --seed 42
```

**Key PDMT Requirements:**
- **Deterministic**: Uses fixed seed (42) for reproducible todo generation
- **Quality-Enforced**: Includes validation commands, test requirements, and success criteria
- **Structured**: Each todo has implementation specs, quality gates, and dependency tracking
- **Complete**: No stub implementations or vague descriptions allowed

**Example PDMT Todo Structure:**
- Clear, actionable todo items with specific deliverables
- Validation commands to verify completion (e.g., `make test`, `pmat quality-gate`)
- Success criteria with measurable outcomes
- Implementation specifications with architectural details
- Quality requirements (test coverage, complexity limits, documentation)

**Never create manual todos** - always use PDMT to ensure consistency with our zero-compromise quality standards.

## MCP Dogfooding Philosophy

We eat our own dog food by using our MCP tools as the primary interface:

- **✅ PRIMARY**: Use MCP tools for all operations (analysis, refactoring, quality gates, todos)
- **⚠️ SECONDARY**: CLI commands only when MCP is unavailable
- **🎯 BENEFIT**: Continuously improve user experience by using what our users use
- **📈 QUALITY**: Ensures MCP integration receives the same attention as core functionality

**MCP-First Examples:**
- Analysis: Use MCP `analyze_complexity` tool before CLI `pmat analyze complexity`
- Refactoring: Use MCP `refactor_start` tool before CLI `pmat refactor auto`
- Quality Gates: Use MCP `quality_gate` tool before CLI `pmat quality-gate`
- Todo Generation: Use MCP `pdmt_deterministic_todos` tool

## TDG Quality Analysis (Mandatory Dogfooding)

**MANDATORY**: We MUST use our own TDG (Technical Debt Grading) system for all quality analysis and continuous improvement. This is core dogfooding practice.

### Installation and Setup
```bash
# Install latest from crates.io (dogfooding principle)
cargo install pmat --force

# Verify installation
pmat --version  # Should show v2.39.0+

# Quick health check
pmat tdg --help
```

### Core TDG Workflows (Sprint 31 Delivered)

#### 1. File Analysis (Primary Workflow)
```bash
# Analyze single file with detailed breakdown
pmat tdg server/src/tdg/analyzer_ast.rs

# Analyze with component breakdown
pmat tdg server/src/tdg/storage.rs --include-components

# Get top problematic files
pmat tdg . --top-files 10

# Export analysis results
pmat tdg . --format json > analysis.json
pmat tdg . --format csv > analysis.csv
pmat tdg . --format sarif > analysis.sarif
```

#### 2. Web Dashboard (Real-time Monitoring)
```bash
# Start web dashboard for real-time TDG monitoring
pmat tdg dashboard --port 8081 --open

# Access at: http://localhost:8081
# Features:
# - Real-time system metrics
# - Storage backend monitoring
# - Performance profiling
# - Interactive analysis
```

#### 3. MCP Integration (External Tools)
```bash
# Start MCP server for external tool integration
pmat mcp serve --port 3000

# Available MCP tools:
# - tdg_analyze_with_storage
# - tdg_system_diagnostics  
# - tdg_storage_management
# - tdg_performance_profiling
# - tdg_alert_management
# - tdg_export_data
```

#### 4. Advanced Features
```bash
# Performance profiling with flame graphs
pmat tdg profile server/src/tdg/ --flame-graph

# Alert system configuration
pmat tdg alerts --configure --threshold high

# Batch export all formats
pmat tdg export . --all-formats --output-dir ./tdg-reports/
```

### Quality Standards via TDG (v2.39.0)

**Mandatory Thresholds** (Toyota Way Zero-Defect):
- **Overall Grade**: Must maintain A- or higher (≥85 points)
- **Structural Complexity**: ≤20 per function (enforced)
- **Semantic Complexity**: Cognitive complexity ≤15 (enforced)
- **Duplication**: <10% code duplication (measured)
- **Documentation**: >70% coverage for public APIs (tracked)
- **Technical Debt**: Zero SATD comments (zero-tolerance)

**Enforcement Commands**:
```bash
# Run quality gate (fails build if standards not met)
pmat quality-gate --file <file.rs>

# Comprehensive project analysis
pmat tdg . --enforce-thresholds --fail-on-grade-below A-

# Integration with make commands
make lint    # Includes TDG quality checks
make test    # Includes TDG validation
```

### Daily Dogfooding Practice

**Before Every Commit**:
1. **TDG Analysis**: `pmat tdg <changed-files>` 
2. **Quality Gate**: `pmat quality-gate --file <changed-files>`
3. **Dashboard Check**: `pmat tdg dashboard` (verify no regressions)
4. **Standard Gates**: `make lint && make test`

**Weekly Quality Review**:
1. **Full Project Analysis**: `pmat tdg . --top-files 20`
2. **Trend Analysis**: `pmat tdg dashboard` (check performance trends)
3. **Export Reports**: `pmat tdg export . --format markdown --output weekly-report.md`
4. **Kaizen Planning**: Use worst-graded files for next improvement cycle

## The Kaizen Refactoring Loop (The "Kata")

This is the core workflow for improving the codebase. Treat it as a repeatable practice (a kata) to drive quality.

### Step 1: Find the Target (Genchi Genbutsu)

First, "go and see" the problems. Use our MCP tools (PRIMARY) to identify the most critical area for improvement:

-   **For General Quality Issues (MCP First):**
    - **✅ PRIMARY**: Use MCP `analyze_lint_hotspot` tool with `{"top_files": 5}`
    - **⚠️ FALLBACK**: `pmat analyze lint-hotspot --top-files 5`
-   **For High Complexity (MCP First):**
    - **✅ PRIMARY**: Use MCP `analyze_complexity` tool with `{"top_files": 5}`
    - **⚠️ FALLBACK**: `pmat analyze complexity --top-files 5`
-   **For Technical Debt (MCP First):**
    - **✅ PRIMARY**: Use MCP `analyze_satd` tool
    - **⚠️ FALLBACK**: `pmat analyze satd`
-   **For Unused Code (MCP First):**
    - **✅ PRIMARY**: Use MCP `analyze_dead_code` tool
    - **⚠️ FALLBACK**: `pmat analyze dead-code`

### Step 2: Create the Refactoring Plan (Jidoka)

Once you have identified a target file, use our MCP refactoring tools (PRIMARY) to generate an automated, AI-driven refactoring plan:

- **✅ PRIMARY**: Use MCP `refactor_start` tool with `{"file_path": "<path/to/target/file.rs>"}`
- **⚠️ FALLBACK**: `pmat refactor auto --file <path/to/target/file.rs>`

### Step 3: Verify the Improvement

After applying the refactoring, you **MUST** verify that the change improved quality and did not introduce regressions.

1.  **Run Quality Gate (MCP First):** Ensure the specific file now meets our zero-tolerance standards.
    - **✅ PRIMARY**: Use MCP `quality_gate` tool with `{"file_path": "<path/to/target/file.rs>"}`
    - **⚠️ FALLBACK**: `pmat quality-gate --file <path/to/target/file.rs>`
2.  **Run Fast Tests:** Confirm that the changes have not broken any existing functionality.
    ```bash
    make test-fast
    ```
3.  **Add doctest:** update or add doctest for added functionality

Once both checks pass, commit the changes and return to Step 1 to find the next target.

## Mandatory Checks Before Committing

After making **any** code changes, you **MUST** run the following commands from the project root. A commit will not be accepted otherwise.

```bash
make lint
make test
```

The `make test` command runs all required tests:
- `make test-fast` - Fast unit and integration tests
- `make test-doc` - All doctests
- `make test-property` - Property-based tests
- `make test-examples` - All cargo examples

**TOYOTA WAY PRINCIPLE: NO DEFECT IS MINOR**
- All compilation errors must be fixed immediately
- All warnings must be addressed (unused imports, dead code, type mismatches)
- All tests must pass across examples, integration tests, and unit tests
- Zero tolerance for "it compiles but has warnings" mentality
- Every defect represents a potential failure point that violates our quality standards

## Quality Standards (Toyota Way Excellence Achieved)

**✅ STATUS: PROJECT NOW MEETS ALL EXTREME QUALITY STANDARDS**

Following successful Toyota Way Kaizen refactoring and comprehensive verification, the project now achieves:

-   **Complexity:** **ACHIEVED** - All functions ≤20 complexity (current max: 0)
-   **Test Coverage:** **EXCEEDED & ENFORCED** - 80.2% achieved (Sprint 46), ≥80% enforced at pre-commit
-   **Technical Debt:** **ACHIEVED** - Zero SATD comments maintained (0 found)
-   **Linting:** **ACHIEVED** - All clippy violations eliminated (0 violations)
-   **Doctests:** **ACHIEVED** - All 72 failing doctests fixed (63+ passed, 0 failed, 141+ ignored)
-   **Property Tests:** **ACHIEVED** - All 3 failing property tests fixed (229+ passed, 0 failed, 3 ignored)
-   **Integration:** **EXCEEDED** - Full MCP, Quality Gates, and Context integration verified
-   **Code Quality:** **EXCEEDED** - 84% complexity reduction with -3,401 lines while improving functionality

### Comprehensive Quality Verification (Recent):
- **Analysis Handlers**: 11 doctests passing + comprehensive property tests
- **MCP Server**: 11 doctests passing + 14 integration tests passing  
- **SATD Detection**: 14 property tests passing + quality gate integration
- **Complexity Analysis**: 21 property tests passing + threshold filtering verified
- **Dead Code Analysis**: 9 property tests + full integration (CLI, MCP, Quality Gates)
- **Refactor Engine**: 10 property tests + state machine verification

### Toyota Way Success Metrics:
- **handle_refactor_auto**: 136 → 21 complexity (-84%)
- **handle_analyze_dead_code**: 244 → ~10 complexity (-96%)
- **Total violations**: 5,202 → 0 (-100%)
- **SATD comments**: 0 (maintained zero-tolerance)
- **Lint violations**: 0 (all fixed)
- **Failing doctests**: 72 → 0 (-100%)
- **Failing property tests**: 3 → 0 (-100%)
- **Property test coverage**: 64+ comprehensive property tests across all major components
- **Integration test coverage**: CLI + MCP + Quality Gates + Context all verified
- **Code reduction**: -3,401 lines net while enhancing features
- **Coverage Achievement Protection**: ≥80% minimum enforced via pre-commit hooks

### Sprint 46 Coverage Enforcement (NEW)
**Achievement Protection System**: Never drop below our 80.2% Sprint 46 achievement.

**Pre-commit Hook Enhancement:**
- Real-time coverage analysis using `cargo-llvm-cov` (faster, more accurate than tarpaulin)
- Blocks commits that drop below 80% minimum threshold
- Integrated with existing quality gate system  
- Located: `.git/hooks/pre-commit`

**Enforcement Mechanism:**
```bash
# Pre-commit hook runs:
1. Documentation synchronization check
2. PMAT complexity analysis (≤20 cyclomatic, ≤15 cognitive)  
3. SATD zero-tolerance enforcement
4. Clippy linting validation
5. ⭐ NEW: Test coverage ≥80% enforcement
```

**Installation Requirements:**
```bash
# RECOMMENDED: Fast, accurate LLVM-based coverage (2024+ standard)
cargo install cargo-llvm-cov

# Verification command
cargo llvm-cov --summary-only
# Should show ≥80% coverage

# FALLBACK: Slower tarpaulin (if llvm-cov unavailable)
cargo install cargo-tarpaulin
```

**Benefits:**
- 🎯 **Achievement Protection**: Prevents regression from Sprint 46 success
- 🏭 **Toyota Way**: Quality built-in at source (pre-commit)  
- 📊 **Real-time Feedback**: Coverage analysis during development
- 🚫 **Zero Regression**: Cannot commit code that reduces coverage

## Canonical Version Management (NEW - Prevents Version Regression)

**IMPORTANT**: We now use a canonical version management system that prevents version regression issues (like 2.3.0 → 2.0.1). 
Full specification: `docs/todo/canonical-version-updates-spec.md`

### Quick Release Commands

```bash
# Run pre-release checklist (recommended first step)
./scripts/release-checklist.sh

# Interactive release with recommendations
./scripts/release-checklist.sh --interactive

# Automatic version bump detection (recommended)
make release-auto

# Manual version bumps
make release-patch   # Bug fixes only (x.y.Z)
make release-minor   # New features (x.Y.z)  
make release-major   # Breaking changes (X.y.z)

# GitHub Actions workflow
gh workflow run canonical-release.yml -f bump_type=auto
```

### Release Process (Jidoka - Quality at Every Step)

The canonical release process enforces quality gates at every step:

#### Step 1: Pre-Release Validation
```bash
# Runs 12 quality checks automatically
make pre-release-checks
```
This validates:
- Version consistency across workspace
- All tests passing
- Zero SATD tolerance
- Security audit (cargo-audit)
- Outdated dependencies check
- SemVer compatibility (cargo-semver-checks)

#### Step 2: Determine Version Bump
```bash
# Auto-detect based on commits
make release-auto

# Or use checklist for recommendation
./scripts/release-checklist.sh
```
The system analyzes:
- Breaking changes → MAJOR
- Feature commits (feat:) → MINOR  
- Everything else → PATCH

#### Step 3: Execute Release
```bash
# This will:
# 1. Run all pre-release checks
# 2. Update versions in workspace
# 3. Update CHANGELOG.md
# 4. Create git commit and tag
# 5. Push to GitHub
# 6. Create GitHub release
# 7. Publish to crates.io (if configured)
make release-[patch|minor|major|auto]
```

#### Step 4: Verify Release
```bash
# Automatic verification
make release-verify

# Manual verification
cargo search pmat | head -1
cargo install pmat --force
pmat --version
```

### Release Quality Gates

Every release MUST pass these gates:
1. **Version Consistency**: All workspace members synchronized
2. **Quality Standards**: Zero SATD, complexity ≤20, all tests pass
3. **Security**: No critical vulnerabilities (cargo-audit)
4. **SemVer Compliance**: API compatibility verified (cargo-semver-checks)
5. **Documentation**: CHANGELOG.md updated with changes
6. **Dependencies**: No severely outdated dependencies

### Release Tools

The system uses industry-standard tools:
- **cargo-release**: Workspace-aware version management
- **cargo-semver-checks**: API breaking change detection
- **cargo-audit**: Security vulnerability scanning
- **cargo-outdated**: Dependency freshness checking

Install all tools:
```bash
make install-release-tools
```

### Recovery from Release Issues

If a wrong version is published:
1. **Cannot unpublish from crates.io** (by design)
2. Immediately publish a patch version with fix
3. Yank the bad version: `cargo yank --version x.y.z`
4. Document the issue in CHANGELOG.md

### Release Checklist
- [ ] Run `./scripts/release-checklist.sh` first
- [ ] All CI/CD workflows passing
- [ ] Pre-release checks pass (`make pre-release-checks`)
- [ ] CHANGELOG.md has unreleased changes documented
- [ ] Version bump type identified (patch/minor/major)
- [ ] Release created with `make release-auto`
- [ ] GitHub release verified
- [ ] crates.io publication verified
- [ ] Both installation methods tested

**Remember**: The canonical system prevents version regression and ensures every release meets our extreme quality standards.

- workspace project
- We use TDD. no code is written unless a test is written first.  new features require 80% coverage and passing pmat quality gates.  all code must be in roadmap and using a ticket that is updated when complete.  For tickets/bugs, we need to add doctests/property tests and cargo run --example.
- this is a workspace project, never cd into server.
- We practice the toyota way.  EVERY defect is our problem.  We never "allow them", we use five-whys, fix root cause, even if unrelated to a problem we are working on.