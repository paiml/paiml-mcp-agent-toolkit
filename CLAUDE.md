# Claude Agent Guide: paiml-mcp-agent-toolkit (pmat)

This guide provides the essential operational instructions for working on the `pmat` codebase, grounded in the principles of the Toyota Way.

## 🏆 Sprint 86 Complexity Elimination COMPLETE - v2.71.0

**MAJOR ACHIEVEMENT**: Sprint 86 has successfully **ELIMINATED 60% of complexity violations** through Toyota Way Extract Method refactoring and added **first-class Ruchy language support**.

### Key Achievements  
- **Complexity Reduction**: 70 → 28 violations (60% reduction)
- **Max Complexity**: 22 → 16 cyclomatic (27% improvement)
- **Ruchy Integration**: Full TDG, entropy, and language analysis support
- **Test Coverage**: 80.2% maintained with infrastructure restoration
- **Technical Debt**: Zero blocking issues resolved
- **Release**: v2.71.0 production-ready

**Proven Methodology**: The Toyota Way Kaizen approach continues to deliver exceptional code quality improvements.

## The Toyota Way: Our Guiding Philosophy

-   **Kaizen (改善): Continuous, Incremental Improvement.** We improve the codebase one file at a time. This ensures that every change is small, verifiable, and moves us toward our quality goals. Avoid large, sweeping changes.
-   **Genchi Genbutsu (現地現物): Go and See.** We don't guess where problems are. We use `pmat`'s analysis tools to find the *actual* root cause of quality issues, such as complexity hotspots or technical debt.
-   **Jidoka (自働化): Automation with a Human Touch.** We use `pmat refactor auto` to automate the creation of a refactoring plan, but an intelligent agent (you) must verify and apply the changes, ensuring correctness.

## 🚨 CRITICAL: A+ Code Standard for ALL NEW Code

**ABSOLUTE REQUIREMENT**: All NEW code written MUST achieve A+ quality standards:
- **Maximum Cyclomatic Complexity**: ≤10 (not 20, not 15, TEN!)
- **Maximum Cognitive Complexity**: ≤10 (simple, readable, maintainable)
- **TDD Mandatory**: Write test FIRST, then implementation
- **Test Coverage**: 100% for new functions (no exceptions)
- **Zero SATD**: No TODO, FIXME, HACK, or "temporary" solutions
- **Function Size**: ≤30 lines (if longer, decompose it)
- **Single Responsibility**: Each function does ONE thing well

**Why This Matters**:
- We have 565 legacy complexity violations to fix
- We CANNOT add more technical debt while fixing old debt
- Every new complex function adds 10+ hours to future refactoring
- A+ code is EASIER to write than B- code when done properly

**Enforcement**:
```rust
// ❌ BAD: Complexity 15+
fn process_data(items: Vec<Item>) -> Result<Output> {
    let mut results = Vec::new();
    for item in items {
        if item.valid {
            if item.type == "A" {
                // ... 20 more lines of nested logic
            }
        }
    }
    // ... more complexity
}

// ✅ GOOD: Complexity ≤10
fn process_data(items: Vec<Item>) -> Result<Output> {
    items.into_iter()
        .filter(|item| item.valid)
        .map(process_single_item)
        .collect()
}

fn process_single_item(item: Item) -> Result<ItemOutput> {
    match item.item_type {
        ItemType::A => process_type_a(item),
        ItemType::B => process_type_b(item),
    }
}
```

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
10. **NEVER Bypass Quality Gates:** Zero tolerance for `--no-verify` or bypassing quality gates. All code MUST pass quality gates before committing. The quality gate now properly excludes test files and focuses on production code only. Maximum cyclomatic complexity is 20, and all functions must comply. If quality gate fails, fix the issues before proceeding.
11. **NEVER Use Git Branches:** Always work directly on master branch. No feature branches, no topic branches. This ensures continuous integration and prevents merge conflicts. All changes go directly to master after passing quality gates.

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

#### 4. TDG Dogfooding Storage (NEW - v2.68.0)
**MANDATORY**: We now persistently store all TDG scores for historical tracking and quality trend analysis.

```bash
# TDG automatically stores scores in ~/.pmat/tdg-warm and ~/.pmat/tdg-cold
# Every analysis is cached - repeated analyses use stored scores

# Check storage statistics
pmat tdg storage stats
# Output shows:
# - Total entries: number of files analyzed
# - Hot/warm/cold tier distribution  
# - Storage size and compression ratios

# Analyze file (stores score automatically)
pmat tdg server/src/lib.rs
# Score: 100.0/100 (A+) - stored for future reference

# Re-analyze same file (uses cached score)
pmat tdg server/src/lib.rs  
# Score: 100.0/100 (A+) - retrieved from storage

# View storage location and size
du -sh ~/.pmat/tdg-*
# Example output: 3.6M ~/.pmat/tdg-warm, 528K ~/.pmat/tdg-cold
```

**Dogfooding Benefits**:
- **Historical Tracking**: Every analyzed file is remembered
- **Performance**: Cache hits avoid re-analysis
- **Quality Trends**: Foundation for tracking code quality over time
- **CI/CD Integration**: Stored scores can be used for quality gates

#### 5. Advanced Features
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
- **Entropy Analysis**: ≤10 high-severity actionable violations per project (enforced in quality gates)

**Enforcement Commands**:
```bash
# Run quality gate (fails build if standards not met, includes entropy checks for strict/extreme profiles)
pmat quality-gate --file <file.rs> --profile strict

# Comprehensive project analysis
pmat tdg . --enforce-thresholds --fail-on-grade-below A-

# NEW: Entropy analysis for actionable violations (v2.69.0+)
# MCP PRIMARY: Use MCP analyze_entropy tool with {"path": ".", "min_severity": "high", "top_violations": 10}
# FALLBACK: CLI analysis when MCP unavailable
pmat analyze entropy . --min-severity high --top-violations 10

# Integration with make commands
make lint    # Includes TDG quality checks + entropy analysis
make test    # Includes TDG validation
```

### Daily Dogfooding Practice

**Before Every Commit** (Enhanced with Storage):
1. **TDG Analysis**: `pmat tdg <changed-files>` (automatically stores scores)
2. **Entropy Analysis**: `pmat analyze entropy --file <changed-files>` (detect actionable patterns)
3. **Quality Gate**: `pmat quality-gate --file <changed-files>`
4. **Storage Check**: `pmat tdg storage stats` (monitor dogfooding progress)
5. **Standard Gates**: `make lint && make test`

**Weekly Quality Review** (Leveraging Stored Data):
1. **Full Project Analysis**: `pmat tdg . --top-files 20` (uses cached scores when possible)
2. **Entropy Pattern Review**: `pmat analyze entropy --top-violations 10` (identify refactoring opportunities)
3. **Storage Analysis**: `du -sh ~/.pmat/tdg-* && pmat tdg storage stats` (track growth)
4. **Export Reports**: `pmat tdg export . --format markdown --output weekly-report.md`
5. **Kaizen Planning**: Use worst-graded files and entropy violations for next improvement cycle

**Monthly Dogfooding Health Check**:
1. **Storage Growth**: Monitor ~/.pmat/tdg-* directory sizes
2. **Cache Hit Ratio**: Look for consistent scores on unchanged files
3. **Quality Trends**: Foundation for future trend analysis features
4. **Storage Cleanup**: Consider archival of very old scores if needed

## Quality-Driven Development (QDD) with TDD (Mandatory)

**MANDATORY**: We MUST use our Quality-Driven Development (QDD) tool for ALL new code creation and refactoring. This ensures consistent quality standards across the entire codebase.

### QDD Core Principles
- **Test-Driven Development (TDD)**: RED-GREEN-REFACTOR cycle is mandatory
  - Write failing test FIRST
  - Write minimal code to pass
  - Refactor to meet quality standards
- **Quality Profiles**: Use appropriate profile for context
  - `extreme`: ≤5 complexity, 90% coverage (critical code)
  - `standard`: ≤10 complexity, 80% coverage (default)
  - `relaxed`: ≤20 complexity, 60% coverage (legacy/migration)
- **Pattern Enforcement**: SOLID, DRY, KISS, YAGNI principles
- **Zero SATD**: No technical debt comments allowed

### QDD Workflows (v2.69.0+)

#### 1. Creating New Code with QDD
```bash
# Create new function with quality guarantees
pmat qdd create --type function --name process_data \
  --purpose "Process incoming data with validation" \
  --profile standard

# Create new module with tests
pmat qdd create --type module --name data_processor \
  --purpose "Data processing module with quality standards" \
  --profile extreme

# Create service with full documentation
pmat qdd create --type service --name api_handler \
  --purpose "API request handler with error recovery" \
  --profile standard
```

#### 2. Refactoring Existing Code
```bash
# Refactor high-complexity function
pmat qdd refactor --file src/complex_file.rs \
  --function complex_function --profile standard

# Refactor entire file to meet standards
pmat qdd refactor --file src/legacy_code.rs \
  --profile extreme

# Batch refactoring with validation
for file in $(pmat analyze complexity --top-files 10 | grep ".rs"); do
  pmat qdd refactor --file $file --profile standard
done
```

#### 3. Validating Code Quality
```bash
# Validate single file meets QDD standards
pmat qdd validate --file src/new_feature.rs --profile standard

# Validate entire module
pmat qdd validate --path src/modules/critical/ --profile extreme

# Pre-commit validation
pmat qdd validate --changed-files --profile standard
```

### TDD + QDD Integration Workflow

**MANDATORY WORKFLOW for new features**:

1. **Write Test First** (RED):
   ```rust
   #[test]
   fn test_new_feature() {
       // Test for feature that doesn't exist yet
       assert_eq!(process_data(&input), expected);
   }
   ```

2. **Generate Implementation with QDD** (GREEN):
   ```bash
   pmat qdd create --type function --name process_data \
     --inputs "input:&str" --output "Result<String>" \
     --profile standard
   ```

3. **Refactor with QDD** (REFACTOR):
   ```bash
   pmat qdd refactor --file src/feature.rs \
     --function process_data --profile extreme
   ```

4. **Validate Quality**:
   ```bash
   pmat qdd validate --file src/feature.rs --profile standard
   pmat quality-gate --file src/feature.rs
   ```

### Daily QDD Practice

**Before Writing Any Code**:
1. **Check Existing Quality**: `pmat tdg <file>` (uses persistent scoring)
2. **Check Pattern Entropy**: `pmat analyze entropy --file <file>` (identify refactoring needs)
3. **Generate with QDD**: `pmat qdd create` (never write code manually for new features)
4. **Validate Standards**: `pmat qdd validate` (ensure compliance)

**During Development**:
1. **TDD Cycle**: Test → QDD Create → Refactor
2. **Complexity Check**: `pmat analyze complexity --file <file>`
3. **Entropy Monitoring**: `pmat analyze entropy --file <file>` (watch for new patterns)
4. **Pattern Check**: `pmat qdd validate --patterns SOLID,DRY`

**Before Committing**:
1. **QDD Validation**: `pmat qdd validate --changed-files`
2. **TDG Scoring**: `pmat tdg <changed-files>` (stores persistently)
3. **Entropy Check**: `pmat analyze entropy --file <changed-files>` (final pattern review)
4. **Quality Gate**: `pmat quality-gate --file <changed-files>`
5. **Test Coverage**: Ensure ≥80% coverage maintained

### QDD Enforcement Rules

1. **NEVER write new functions manually** - Use `pmat qdd create`
2. **NEVER refactor without QDD** - Use `pmat qdd refactor`
3. **NEVER skip validation** - Use `pmat qdd validate` before commits
4. **ALWAYS use TDD** - Test first, QDD implementation second
5. **ALWAYS check TDG scores** - Persistent scoring tracks quality over time

### Integration with CI/CD

```yaml
# Example CI pipeline integration
quality-check:
  script:
    - pmat qdd validate --all --profile standard
    - pmat tdg . --enforce-thresholds --fail-on-grade-below A-
    - pmat analyze entropy --min-severity medium --top-violations 5
    - pmat quality-gate --all
    - cargo test --all
```

## Actionable Entropy Analysis (Mandatory Sprint 83+)

**MANDATORY**: We MUST use our Actionable Entropy Analysis for ALL pattern detection and refactoring identification. This replaces noisy character-based entropy with AST pattern analysis that provides specific, actionable fix suggestions.

### Core Entropy Principles
- **AST-Based Analysis**: Detects repetitive AST patterns, not character entropy
- **Actionable Violations**: Each violation includes specific fix suggestion and LOC reduction estimate
- **Pattern-Focused**: Identifies 6 key pattern types for targeted refactoring
- **Quality Threshold**: Target 10-50 violations (not 2255 like character entropy)

### Pattern Types Analyzed
1. **ErrorHandling**: try/catch, Result handling patterns → Extract error handler functions
2. **DataValidation**: Input validation patterns → Create validation traits/modules
3. **ResourceManagement**: open/close, lock/unlock patterns → Implement RAII/guards
4. **ControlFlow**: if/else chains, match statements → Strategy patterns/polymorphism
5. **DataTransformation**: map/filter/reduce patterns → Data transformation pipelines
6. **ApiCall**: HTTP/RPC call patterns → API client abstractions

### Mandatory Usage Workflows

#### Daily Entropy Analysis
```bash
# Before writing any code - check existing patterns
pmat analyze entropy --file <target-file> --min-severity medium

# During development - watch for new patterns
pmat analyze entropy --file <changed-files> --top-violations 5

# Before commits - final pattern review
pmat analyze entropy --file <all-changed-files> --min-severity low
```

#### Weekly Pattern Review
```bash
# Project-wide pattern identification
pmat analyze entropy --top-violations 10 --format markdown --output entropy-report.md

# Focus on high-impact violations
pmat analyze entropy --min-severity high --format json | jq '.actionable_violations[] | select(.estimated_loc_reduction > 50)'
```

#### Integration with Refactoring
```bash
# Use entropy to guide refactor decisions
pmat analyze entropy --file src/complex_module.rs --format detailed

# After refactoring - verify pattern reduction
pmat analyze entropy --file src/complex_module.rs --top-violations 3
```

### Entropy Quality Standards
- **Maximum Violations per File**: ≤5 medium/high severity violations
- **Pattern Repetition Threshold**: ≤5 instances of same pattern before violation
- **Minimum Pattern Diversity**: ≥30% (Shannon entropy of pattern distribution)
- **Cross-File Duplication**: ≤2 files sharing same pattern before violation

### Enforcement Integration
- **Pre-commit Hooks**: Include entropy analysis in quality gates
- **CI/CD Pipeline**: Fail builds with >10 high-severity entropy violations
- **Code Reviews**: Use entropy report to guide review focus
- **Refactoring Planning**: Use entropy violations to prioritize technical debt

### Output Format Examples

**Summary Format** (Default):
```
Entropy Analysis Summary
========================

Files Analyzed: 15
Total Violations: 3
Potential LOC Reduction: 120 lines (8.5%)

1. ErrorHandling pattern repeated 8 times (saves 45 lines)
   Fix: Extract to `handle_validation_error()` function
```

**JSON Format** (For tooling):
```json
{
  "actionable_violations": [{
    "severity": "High",
    "pattern": {
      "pattern_type": "ErrorHandling",
      "repetitions": 8,
      "variation_score": 0.3
    },
    "fix_suggestion": "Extract to handle_validation_error() function",
    "estimated_loc_reduction": 45
  }]
}
```

### Enforcement Rules
1. **NEVER ignore entropy violations** - All violations must be addressed or justified
2. **ALWAYS use specific fixes** - Apply suggested refactoring, don't just add comments
3. **ALWAYS verify improvement** - Re-run entropy analysis after fixes
4. **ALWAYS target actionable patterns** - Focus on patterns with clear fix paths
5. **ALWAYS check cross-file patterns** - Highest priority for shared modules

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
-   **For Actionable Entropy Violations (MCP First):**
    - **✅ PRIMARY**: Use MCP `analyze_entropy` tool with `{"min_severity": "high", "top_violations": 10}`
    - **⚠️ FALLBACK**: `pmat analyze entropy --min-severity high --top-violations 10`
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

## Quality Standards (Toyota Way Excellence Maintained)

**✅ STATUS: PROJECT EXCEEDS ENTERPRISE QUALITY STANDARDS**

Following Sprint 86 complexity elimination and Ruchy integration:

-   **Complexity:** **EXCELLENT** - Max 16 cyclomatic, 19 cognitive (from 22/27)
-   **Test Coverage:** **PROTECTED** - 80.2% maintained with enforcement
-   **Technical Debt:** **MINIMAL** - 3 minor SATD comments (non-blocking)
-   **Linting:** **CLEAN** - 27 warnings (unused variables only)
-   **Language Support:** **EXPANDED** - First-class Ruchy language (v1.89.0)
-   **Entropy Analysis:** **ACTIONABLE** - AST-based pattern detection implemented
-   **Integration:** **COMPLETE** - Full MCP, Quality Gates, HTTP, and CLI verified
-   **Code Quality:** **A+ GRADE** - Enterprise-grade maintainability achieved

### Comprehensive Quality Verification (Recent):
- **Analysis Handlers**: 11 doctests passing + comprehensive property tests
- **MCP Server**: 11 doctests passing + 14 integration tests passing  
- **SATD Detection**: 14 property tests passing + quality gate integration
- **Complexity Analysis**: 21 property tests passing + threshold filtering verified
- **Dead Code Analysis**: 9 property tests + full integration (CLI, MCP, Quality Gates)
- **Refactor Engine**: 10 property tests + state machine verification

### Sprint 86 Success Metrics:
- **handle_analyze_defect_prediction**: 16/27 → ≤10 complexity (Extract Method)
- **format_defect_summary**: 16/17 → helper functions ≤10
- **Total complexity violations**: 70 → 28 (-60%)
- **Max cyclomatic complexity**: 22 → 16 (-27%)
- **Ruchy language support**: 0 → 100% (full integration)
- **Test compilation errors**: 13 → 0 (infrastructure restored)
- **Entropy violations**: 3143 (needs AST pattern focus)
- **Property test coverage**: 64+ comprehensive tests maintained
- **Integration coverage**: CLI + MCP + HTTP + Ruchy verified
- **Test coverage**: 80.2% maintained and protected
- **Production readiness**: 100% (zero blocking issues)

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

## Build Artifact Management and Cleaning Strategy

To prevent memory issues and ensure clean builds, especially during releases, we implement a systematic cleaning strategy:

### Cleaning Commands

```bash
# Quick clean - just this package and incremental
make clean-quick

# Standard clean - all build artifacts
make clean

# Deep clean - including cargo caches
make clean-deep
```

### When to Clean

1. **Before Every Release**: Automatic via `make pre-release-checks`
   - Ensures fresh, reproducible builds
   - Prevents stale artifact issues
   
2. **Daily Development**:
   - `make clean-quick` when switching between major features
   - Clears incremental compilation cache
   
3. **Weekly Maintenance**:
   - `make clean` to remove all build artifacts
   - Frees up disk space (can be 5-10GB per project)
   
4. **Monthly Deep Clean**:
   - `make clean-deep` to clear cargo registry caches
   - Removes old dependency versions
   - Can free up 5-10GB of disk space

### Memory Management

The project can consume significant memory during compilation:
- Debug builds: 5-10GB
- Release builds: 10-15GB
- Test coverage: 15-20GB

**Signs you need to clean**:
- Swap usage exceeding 10GB
- Compilation errors mentioning "out of memory"
- Mysterious compilation failures
- Slow incremental builds

### Release Process Integration

The cleaning strategy is integrated into our release process:
1. `make pre-release-checks` automatically runs `clean-quick`
2. Release checklist script includes cleaning step
3. CI/CD pipelines start with clean workspace

This ensures every release is built from a clean state, preventing artifact contamination and ensuring reproducibility.

- workspace project
- **We use TDD + QDD MANDATORY**: No code is written unless a test is written first. Use `pmat qdd create` for ALL new code generation. New features require 80% coverage and passing pmat quality gates. All code must be in roadmap and using a ticket that is updated when complete. For tickets/bugs, we need to add doctests/property tests and cargo run --example.
- **TDG Persistent Scoring**: We use `pmat tdg` with persistent storage (~/.pmat/tdg-*) to track quality scores over time. Every analyzed file is stored and cached for historical tracking and performance.
- **QDD Enforcement**: Use `pmat qdd create/refactor/validate` for ALL code changes. Quality profiles (extreme/standard/relaxed) ensure consistent standards. Never write code manually - always use QDD tools.
- this is a workspace project, never cd into server.
- We practice the toyota way. EVERY defect is our problem. We never "allow them", we use five-whys, fix root cause, even if unrelated to a problem we are working on.