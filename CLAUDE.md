# Claude Code Configuration

## CRITICAL: pmat-book Validation Policy (Toyota Way - Jidoka)

**MANDATORY BEFORE ANY RELEASE OR VERSION BUMP:**

**USE THE FAST MAKEFILE TARGET:**

```bash
# Fast, parallel, fail-fast validation (recommended)
make validate-book
```

This Makefile target:
- ✅ Runs critical chapters in parallel (Ch 5, 7, 13, 14)
- ✅ Uses fail-fast behavior (stops on first failure)
- ✅ Typically completes in <30 seconds
- ✅ Automatically run by pre-commit hook for code changes

**Manual validation (only if needed):**

```bash
# Run specific chapter
cd /home/noah/src/pmat-book
bash tests/ch13/test_language_examples.sh  # Multi-language support
```

**Verify test results**:
- ALL core functionality tests must PASS
- Chapter 13 (Multi-Language) is CRITICAL - must always pass
- Document any failures in git commit message

**Update if needed**:
- If tests fail, fix the code OR update the book tests
- Never commit broken functionality
- Apply Toyota Way Andon Cord: STOP if quality issues found

**Rationale**: Toyota Way Jidoka - built-in quality, catches regressions, enforced via pre-commit hook.

---

## CRITICAL: pmat-book Push Enforcement Policy

**MANDATORY: Book updates MUST be pushed with code changes**

### The Problem
The 404 issue occurred because pmat-book commits were made locally but never pushed to GitHub, causing the live book at https://paiml.github.io/pmat-book/ to become out of sync with the codebase.

### The Solution: Two-Layer Enforcement

#### 1. Pre-Commit Hook (Warning)
- Warns about unpushed pmat-book commits during `git commit`
- Shows exactly which commits haven't been pushed
- Doesn't block commits (allows local development)
- Reminds you that pre-push hook will enforce synchronization

#### 2. Pre-Push Hook (BLOCKING)
- **BLOCKS `git push`** until all pmat-book commits are pushed first
- Critical for releases and crates.io publications
- Ensures live book is always in sync with published code
- Cannot be bypassed without `--no-verify` (strongly discouraged)

### Workflow

Update pmat-book → push to main (deploys GitHub Pages) → push code. Pre-push hook blocks if book commits unpushed.

### Special Cases

#### crates.io Release
Before running `cargo publish`, ensure:
1. ✅ All pmat-book changes committed and pushed
2. ✅ Book documentation matches new version
3. ✅ `make validate-book` passes
4. ✅ GitHub Pages deployment completed (check https://github.com/paiml/pmat-book/actions)

#### Emergency Bypass (NOT RECOMMENDED)
```bash
# Only use in emergencies (e.g., critical hotfix)
git push --no-verify

# Then immediately push book:
cd ../pmat-book && git push origin main
```

**Rationale**: Zero tolerance for code/docs drift. Prevents 404s and ensures crates.io releases have matching documentation.

---

## CRITICAL: O(1) Quality Gates (Phase 2 - Active)

**AUTOMATIC ENFORCEMENT: Pre-commit hooks validate metrics in <30ms**

### Overview

Phase 2 of O(1) Quality Gates is now active, providing instant (<30ms) quality validation at commit time using hash-based metric caching.

**Specification**: `docs/specifications/quick-test-build-O(1)-checking.md`

### How It Works

1. **Metric Recording** (during development):
   ```bash
   make lint        # Records lint duration to .pmat-metrics/
   make test-fast   # Records test duration
   make coverage    # Records coverage duration
   make release     # Records binary size
   ```

2. **Pre-Commit Validation** (O(1) instant check):
   - Reads cached metrics from `.pmat-metrics/`
   - Validates against thresholds in `.pmat-metrics.toml`
   - **Blocks commit** if thresholds exceeded (MEAN mode)
   - Entire validation completes in <30ms

### Thresholds (MEAN Mode)

From `.pmat-metrics.toml`:
- **lint**: ≤30s (30,000ms)
- **test-fast**: ≤5min (300,000ms)
- **coverage**: ≤10min (600,000ms)
- **binary size**: ≤50MB (50,000,000 bytes)
- **dependencies**: ≤3,000 (default feature set)

**Staleness**: Metrics older than 7 days trigger warnings

**Benefits**: Pre-commit <30ms (vs 12-24min), Toyota Way Jidoka/Andon Cord enforcement, team savings 2-20hrs/day. Emergency bypass: `git commit --no-verify`. Troubleshooting: Run `make lint/test-fast` to populate cache.

---

## CRITICAL: Documentation Accuracy Enforcement (Zero Hallucinations)

**MANDATORY FOR README.md, CLAUDE.md, GEMINI.md, AGENT.md:**

All AI agent instruction files must be verified against the actual codebase to prevent hallucinations, broken references, and 404 errors.

### Required Before Commits

When modifying any of these documentation files:
- `README.md`
- `CLAUDE.md`
- `GEMINI.md`
- `AGENT.md`

**Run the documentation accuracy validation:**

```bash
# Step 1: Generate deep context (caches codebase facts)
pmat context --output deep_context.md --format llm-optimized

# Step 2: Validate documentation accuracy (Sprint 38 - IMPLEMENTED ✅)
pmat validate-readme \
    --targets README.md CLAUDE.md GEMINI.md AGENT.md \
    --deep-context deep_context.md \
    --fail-on-contradiction \
    --verbose

# Optional: Generate JSON report for CI/CD
pmat validate-readme \
    --targets README.md \
    --deep-context deep_context.md \
    --output json \
    --fail-on-contradiction > hallucination_report.json

# Optional: Generate JUnit XML for CI integration
pmat validate-readme \
    --targets README.md \
    --deep-context deep_context.md \
    --output junit \
    --fail-on-contradiction > hallucination_junit.xml
```

**Available Options:**
- `--targets <FILES>...`: Documentation files to validate (required)
- `--deep-context <FILE>`: Deep context markdown from `pmat context` (required)
- `--verified-threshold <FLOAT>`: Confidence threshold for verification (default: 0.9)
- `--contradiction-threshold <FLOAT>`: Confidence threshold for contradictions (default: 0.3)
- `--fail-on-contradiction`: Exit with error if contradictions found (default: true)
- `--fail-on-unverified`: Exit with error if unverified claims found (default: false)
- `--output <FORMAT>`: Output format: text, json, junit (default: text)
- `--failures-only`: Show only failures (contradictions and unverified)
- `--verbose`: Show detailed validation information

### What Gets Validated

#### 1. Hallucination Detection (Semantic Entropy)
- **Capability Claims**: Statements about PMAT capabilities are verified against codebase
- **API Claims**: Function and method references are checked in AST
- **Structure Claims**: File structure assertions are cross-validated
- **Language Support**: Language compatibility claims are verified in language_analyzer.rs

#### 2. Broken Reference Detection
- **File Paths**: All `path/to/file.rs` references validated
- **Function Names**: All mentioned functions checked in deep context
- **Module References**: All module/class references verified

#### 3. 404 Detection
- **External Links**: All HTTP/HTTPS URLs checked (status code validation)
- **Internal Links**: All relative file links verified
- **Anchors**: Section references validated

### Scientific Foundation

Uses Semantic Entropy (Nature 2024), MIND framework (IJCAI 2025), and Unified Detection (Complex & Intelligent Systems 2025) to validate claims against codebase via confidence scoring.

### Rationale (Scientific Quality Assurance)

- **Zero Hallucinations**: All claims verified against codebase reality
- **Evidence-Based**: Semantic similarity + AST cross-validation
- **Automated**: Pre-commit hooks prevent bad documentation from entering repo
- **Peer-Reviewed Methods**: Based on Nature, IJCAI, ACM research (2024-2025)

### Enforcement

This is enforced by:
1. **Pre-commit Hook**: Automatically runs on doc file changes
2. **CI/CD Pipeline**: GitHub Actions validation
3. **Quality Gate**: Part of `pmat quality-gate --checks docs-accuracy`

**Bypass** (NOT RECOMMENDED):
```bash
git commit --no-verify
```

### Specification

Full specification: `docs/specifications/documentation-accuracy-enforcement.md`

---

## Bash/Makefile Quality Enforcement with bashrs

**MANDATORY: All bash scripts and Makefiles must pass bashrs linting.**

bashrs (PAIML) lints bash/Makefiles for SC2086/SC2046/SC2116, DET003 (non-determinism), IDEM002 (idempotency), SEC008 (security).

### Bug Reports and Feature Requests

**IMPORTANT**: bashrs is developed and maintained by PAIML.

If you encounter a bug or need a feature while using bashrs:
- **GitHub Issues**: https://github.com/paiml/bashrs/issues
- **Required**: All bugs and feature requests must be submitted as GitHub issues
- **Context**: Include reproduction steps, bashrs version, and example bash code

This ensures proper tracking and allows the bashrs team to improve the tool for all users.

### Usage

```bash
# Lint a single bash script
bashrs lint scripts/install.sh

# Lint Makefile
bashrs lint Makefile

# Lint all bash scripts in a directory
find scripts -name "*.sh" -exec bashrs lint {} \;
```

**Installation**: `pmat hooks install --tdg-enforcement` (auto-includes bashrs). Pre-commit hook runs bashrs on staged bash/Makefile files. Exit codes: 0 (pass), 1 (warnings), 2 (errors/blocks). Fast (<2ms/file), prevents shell injection, catches non-determinism.

---

## Coverage Tool Policy

**IMPORTANT: We do NOT use cargo-tarpaulin for code coverage.**

- Use `cargo llvm-cov` exclusively for coverage reporting
- Never install or suggest cargo-tarpaulin
- All coverage targets should use cargo llvm-cov commands
- If you see tarpaulin references in the codebase, remove them


## Test Coverage

The following tests have been marked as `#[ignore]` to achieve stable coverage metrics:

### Language-Specific Tests (4 tests)
- `services::languages::kotlin::tests::test_kotlin_class_with_methods_analysis`
- `services::languages::wasm::tests::test_complex_wat_control_flow`
- `services::languages::wasm::tests::test_wasm_complexity_analysis`
- `services::languages::wasm::tests::test_wat_text_analysis`

### Language Regression Tests (6 tests) - 100% PASSING (Sprint 42 verified)
**Status**: Created as regression tests for multi-language support
**Passing**: 6/6 tests (100% - Sprint 42 verified 2025-10-19)
**Failing**: 0/6 tests

- `tests::language_regression_tests::test_c_deep_context_analysis` ✅ PASSING (3 functions detected)
- `tests::language_regression_tests::test_wasm_deep_context_analysis` ✅ PASSING (3 functions detected)
- `tests::language_regression_tests::test_bash_deep_context_analysis` ✅ PASSING (39 functions detected)
- `tests::language_regression_tests::test_cpp_deep_context_analysis` ✅ PASSING (6 functions detected)
- `tests::language_regression_tests::test_php_deep_context_analysis` ✅ PASSING (6 functions detected)
- `tests::language_regression_tests::test_swift_deep_context_analysis` ✅ PASSING (9 functions detected)

**Sprint 42 Five Whys Discovery**: Previous "failures" were due to flaky concurrent test execution.
All 6 language regression tests are fully functional and passing when run properly.
Root cause: Test execution ordering/concurrency, NOT broken functionality
**File**: `server/src/tests/language_regression_tests.rs` (533 lines)
**Implementation**:
- `server/src/services/languages/bash.rs` (BashScriptAnalyzer - 753 lines)
- `server/src/services/languages/php.rs` (PhpScriptAnalyzer - 397 lines)
- `server/src/services/languages/swift.rs` (SwiftSourceAnalyzer - 456 lines)
- `server/src/services/simple_deep_context.rs` (C++ regex fix - line 1363)

### Infrastructure Tests (7 tests)
- `services::memory_manager::tests::test_concurrent_access`
- `tdg::analyzer_simple::tests::test_analyze_complex_code`
- `tdg::config::tests::test_config_from_file`
- `tdg::profiler::tests::test_flame_graph_generation`
- `tdg::profiler::tests::test_operation_profiling`
- `tdg::web_dashboard::tests::test_dashboard_state_creation`
- `tdg::web_dashboard::tests::test_metrics_update`
- `tdg::web_dashboard::tests::test_router_creation`

### Binary Integration Tests (1 test)
- `tests::bin_integration::test_binary_version_flag` - Compilation timeout in CI

### End-to-End Tests (4 tests)
- `tests::ast_e2e::ast_python_tests::test_analyze_python_file_comprehensive`
- `tests::ast_e2e::ast_python_tests::test_python_import_parsing`
- `tests::ast_e2e::ast_typescript_tests::test_jsx_file_detection`
- `tests::ast_e2e::ast_typescript_tests::test_tsx_file_detection`

### CLI and Quality Tests (2 tests)
- `tests::lib_tests::clap_argument_parsing_tests::type_coercion_tests::test_optional_argument_coercion`
- `tests::quality_checks_property_tests::unit_tests::test_complexity_violation_detection`

### Annotation TDD Tests (7 tests) - Require pmat binary
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_individual_function_names`
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_file_level_breakdown`
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_complexity_scores`
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_satd_annotations`
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_quality_insights`
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_dead_code_markers`
- `cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_wasm_function_details`

### Unified Quality Framework Tests (14 tests)
- `unified_quality::enforcement::property_tests::budget_consumption_accumulates_correctly`
- `unified_quality::enforcement::property_tests::decisions_respect_budget_limits`
- `unified_quality::enforcement::property_tests::grace_period_enforcement_properties`
- `unified_quality::enforcement::property_tests::refactor_target_generation_properties`
- `unified_quality::enforcement::property_tests::time_series_operations_stable`
- `unified_quality::enhanced_parser::property_tests::cache_consistency`
- `unified_quality::enhanced_parser::property_tests::cache_invalidation_works`
- `unified_quality::enhanced_parser::property_tests::complexity_increases_with_control_flow`
- `unified_quality::enhanced_parser::property_tests::match_expression_complexity`
- `unified_quality::enhanced_parser::property_tests::nesting_affects_cognitive_complexity`
- `unified_quality::enhanced_parser::property_tests::parser_handles_valid_identifiers`
- `unified_quality::enhanced_parser::property_tests::satd_detection_accuracy`
- `unified_quality::foundation::property_tests::pattern_matching_edge_cases`
- `unified_quality::integration_tests::tests::test_ml_refactoring_integration`
- `unified_quality::integration_tests::tests::test_progressive_quality_adoption`

### Language Detection Tests (5 tests) - Need fixes
- `cli::language_detection_tests::property_tests::test_file_extension_counting_accuracy`
- `cli::language_detection_tests::property_tests::test_javascript_detection_consistency`
- `cli::language_detection_tests::property_tests::test_typescript_detection_consistency`
- `cli::language_detection_tests::proptest_generators::test_extension_mapping_correctness`
- `cli::language_detection_tests::regression_tests::test_typescript_not_detected_as_deno_regression`

### Enhanced Naming Tests (6 tests) - Require implementation
- `services::enhanced_naming_tests::enhanced_javascript_naming_tests::javascript_real_world_tests::test_higher_order_functions_and_closures`
- `services::enhanced_naming_tests::enhanced_javascript_naming_tests::javascript_real_world_tests::test_module_exports_and_imports_tracking`
- `services::enhanced_naming_tests::enhanced_javascript_naming_tests::test_jsdoc_extraction_for_enhanced_context`
- `services::enhanced_naming_tests::enhanced_naming_integration_tests::test_deep_context_markdown_enhanced_names`
- `services::enhanced_naming_tests::enhanced_naming_integration_tests::test_multi_language_enhanced_naming_integration`
- `services::enhanced_naming_tests::enhanced_typescript_naming_tests::typescript_real_world_tests::test_react_typescript_components_with_props`

### Unified Context Tests (4 tests) - Require implementation
- `cli::handlers::unified_context_advanced_tests::advanced_annotation_tests::test_unified_output_contains_all_annotations`
- `cli::handlers::unified_context_property_tests::extreme_tdd_tests::green_test_unified_context_handles_multiple_languages`
- `cli::handlers::unified_context_property_tests::extreme_tdd_tests::red_test_unified_context_must_show_functions`
- `cli::handlers::unified_context_property_tests::extreme_tdd_tests::test_wasm_function_extraction`

### TypeScript/JavaScript Tests (3 tests) - Need implementation
- `cli::handlers::unified_context_property_tests::extreme_tdd_tests::test_javascript_descriptive_names`
- `cli::handlers::unified_context_property_tests::extreme_tdd_tests::test_typescript_interface_detection`
- `services::enhanced_typescript_visitor::tests::typescript_tests::test_extract_class_details`

### Real-World and Performance Tests (5 tests) - Need proper setup
- `services::real_world_enhanced_naming_test::real_world_tests::typescript_real_world_integration::test_real_world_typescript_react_file_analysis`
- `tests::extreme_tdd_concurrency_fix::test_all_annotations_present_no_timeouts`
- `tests::extreme_tdd_concurrency_fix::test_sub_second_performance_small_project`
- `tests::extreme_tdd_smart_bounds::test_churn_analysis_bounded`
- `tests::extreme_tdd_smart_bounds::test_full_analysis_smart_bounds`

### Integration Tests (1 test) - Output format changed
- `tests::cli_comprehensive_integration::test_context_markdown_output`

### Timeout Integration Tests (3 tests) - Require binary
- `tests::dead_code_timeout_test::test_dead_code_completes_within_timeout`
- `tests::dead_code_timeout_test::test_dead_code_handles_empty_directory`
- `tests::dead_code_timeout_test::test_dead_code_handles_single_file`

### Ruchy Parser Tests (10 tests) - RED tests for ruchy-ast feature
- `ruchy_parser_tests::test_ruchy_parser_integration_simple_function`
- `ruchy_parser_tests::test_ruchy_parser_integration_complex_function`
- `ruchy_parser_tests::test_ruchy_parser_integration_match_expression`
- `ruchy_parser_tests::test_ruchy_parser_integration_loops`
- `ruchy_parser_tests::test_ruchy_parser_integration_multiple_functions`
- `ruchy_parser_tests::test_ruchy_parser_integration_actor_model`
- `ruchy_parser_tests::test_ruchy_parser_integration_syntax_error`
- `ruchy_parser_tests::test_ruchy_parser_integration_empty_file`
- `ruchy_parser_tests::test_ruchy_parser_integration_pipeline_operators`
- `ruchy_parser_tests::test_ruchy_parser_integration_generic_functions`

### Known Failing Tests - UPDATED (October 19, 2025)
**Previous Status**: 14 tests documented as failing (October 6, 2025)
**Current Status**: ✅ ALL 14 TESTS NOW PASSING (Verified October 19, 2025)

**Discovery**: All 14 "known failing" tests were fixed in previous sessions but documentation wasn't updated.

#### Previously Failing - Now PASSING ✅ (14 tests)

**Service Layer (6 tests)** - All passing:
- ✅ `services::configuration_service::tests::test_service_lifecycle`
- ✅ `services::deep_wasm::service::tests::test_analyze_minimal_request`
- ✅ `services::deep_wasm::service::tests::test_analyze_ruchy_file`
- ✅ `services::deep_wasm::tests::integration_tests::test_end_to_end_minimal_analysis`
- ✅ `services::mutation::rust_adapter::tests::test_find_cargo_root`
- ✅ `tests::cli_integration_full::tests::test_cli_context_generation`

**Defect Report Service (5 tests)** - All passing (were never broken):
- ✅ `services::defect_report_service::integration_tests::tests::test_csv_formatting`
- ✅ `services::defect_report_service::integration_tests::tests::test_defect_report_generation`
- ✅ `services::defect_report_service::integration_tests::tests::test_json_formatting`
- ✅ `services::defect_report_service::integration_tests::tests::test_markdown_formatting`
- ✅ `services::defect_report_service::integration_tests::tests::test_text_formatting`

**E2E Binary Tests (3 tests)** - Still require binary (correctly ignored):
- `tests::e2e_full_coverage::test_cli_analyze_churn` (requires pmat binary)
- `tests::e2e_full_coverage::test_cli_main_binary_help` (requires pmat binary)
- `tests::e2e_full_coverage::test_cli_main_binary_version` (requires pmat binary)

**Total: 94 tests ignored (down from 117 on October 21, 2025)**

**Note**: ~309 total `#[ignore]` annotations (82 in server/src, 227 in server/tests). Recent re-enabling: Sprint 44 (20 tests), Oct 21 (23 tests) - all verified passing. Ignored tests: 137→94 (-43, -31.4%). Sprint 36: Implemented Bash/PHP/Swift AST parsers, all 6 regression tests passing.

These tests can be re-enabled by removing the `#[ignore]` attribute when they are fixed.
Known failures are pre-existing and unrelated to Sprint 19 work.
- always walk of master.  we don't do branching
---

## PMAT Five Whys Root Cause Analysis (Toyota Way)

**IMPLEMENTED**: REFACTOR phase complete (2025-11-24)
**Command**: `pmat five-whys` (aliases: `why`, `debug-whys`)
**Status**: Production-ready, fully functional

### Overview

Evidence-based root cause analysis using the Toyota Way Five Whys methodology. Automatically gathers evidence from PMAT services (complexity, SATD, dead code, git churn, TDG) to identify root causes through iterative questioning.

**This is the ONLY acceptable debugging method per CLAUDE.md policy.**

### Quick Start

```bash
# Basic usage (5 iterations, text output)
pmat five-whys "Stack overflow in parser"

# Short alias with custom depth
pmat why "Memory leak in cache" --depth 3

# JSON output to file
pmat five-whys "Test failures" --format json --output analysis.json

# Markdown report with auto-analysis
pmat five-whys "Performance regression" --format markdown --auto-analyze
```

### Command Syntax

```bash
pmat five-whys <ISSUE> [OPTIONS]

Arguments:
  <ISSUE>  Issue description (symptom to analyze)

Options:
  -d, --depth <DEPTH>         Number of "Why" iterations [default: 5, range: 1-10]
  -f, --format <FORMAT>       Output format: text, json, markdown [default: text]
  -o, --output <FILE>         Write output to file
  -p, --path <PATH>           Project path to analyze [default: .]
  -c, --context <FILE>        Use deep context file for enhanced analysis
  -a, --auto-analyze          Automatically analyze suspected files with PMAT tools
  -h, --help                  Print help
```

### Output Formats

**Text** (terminal), **JSON** (CI/CD), **Markdown** (docs) - includes questions, hypotheses, evidence, confidence scores, root cause, and prioritized recommendations.

### Evidence Sources

Five Whys automatically gathers evidence from:

1. **Complexity Analysis**: Cyclomatic complexity violations (threshold: 20)
2. **SATD Detection**: TODO/FIXME/HACK markers indicating technical debt
3. **TDG Scoring**: Test-Driven Grade (coverage, quality)
4. **Git Churn**: Commit frequency indicating instability
5. **Dead Code**: Unused functions and modules

### Confidence Scoring

Confidence scores (0.0-1.0) are calculated using weighted evidence:
- **Complexity**: 25% weight × severity multiplier
- **TDG**: 25% weight × severity multiplier
- **SATD**: 20% weight × severity multiplier
- **Git Churn**: 20% weight × severity multiplier
- **Dead Code**: 10% weight
- **Manual Inspection**: 15% weight

Higher confidence = stronger evidence-backed hypothesis.

### Toyota Way Principles

**Genchi Genbutsu** (evidence-driven), **Jidoka** (automated evidence gathering), **Kaizen** (learn from root causes), **Nemawashi** (transparent reasoning).

### Implementation

**Spec**: `docs/specifications/pmat-debug-five-whys.md`
**Core**: `server/src/services/five_whys_analyzer.rs`, `server/src/models/debug_analysis.rs`
**Methodology**: EXTREME TDD (26 tests, 100% passing), evidence-based (no guessing)

---

## Rust Project Score v1.1 - Evidence-Based Quality Scoring

**IMPLEMENTED**: Sprint 1-4 complete (2025-11-16)
**Command**: `pmat rust-project-score` (alias: `rust-score`)
**Status**: Production-ready, fully functional

### Overview

Comprehensive Rust project quality scoring extending `repo-score` with evidence-based refinements from 15 peer-reviewed papers (2022-2025). Provides 106-point scoring across 6 categories.

### Quick Start

```bash
# Fast mode (default, ~2-3 minutes)
# Skips: clippy, mutation testing, build time measurement
pmat rust-project-score

# Full mode (~10-15 minutes on large projects)
# Includes: all checks with comprehensive analysis
pmat rust-project-score --full

# Specific path with JSON output
pmat rust-project-score --path /path/to/rust/project --format json

# Verbose breakdown with markdown output
pmat rust-project-score --verbose --format markdown --output SCORE.md

# Show only failures and warnings
pmat rust-project-score --failures-only
```

### Scoring Categories (106 points total)

1. **Rust Tooling Compliance** (25pts)
   - Clippy: Tiered scoring (correctness > suspicious > pedantic)
   - rustfmt: Code formatting compliance
   - cargo-audit: Security vulnerability scanning (risk-based)
   - cargo-deny: Dependency policy enforcement

2. **Code Quality** (26pts)
   - Cyclomatic Complexity (3pts): All functions ≤20
   - Unsafe Code (9pts): Proper documentation + safety comments
   - Mutation Testing (8pts): ≥80% mutation score
   - Build Time (4pts): Fast incremental builds
   - Dead Code (2pts): No unused code

3. **Testing Excellence** (20pts)
   - Coverage (8pts): ≥85% line coverage
   - Integration Tests (4pts): Comprehensive integration testing
   - Doc Tests (3pts): Examples in rustdoc
   - Mutation Coverage (5pts): Test quality validation

4. **Documentation** (15pts)
   - Rustdoc (7pts): Comprehensive API documentation
   - README (5pts): Clear project documentation
   - Changelog (3pts): Version history tracking

5. **Performance & Benchmarking** (10pts)
   - Criterion Benchmarks (5pts): Performance baselines
   - Profiling (5pts): Performance analysis tooling

6. **Dependency Health** (12pts)
   - Count (5pts): Minimal dependency footprint
   - Feature Flags (4pts): Modular dependencies
   - Tree Pruning (3pts): Optimized dependency tree

### Output Formats

Supports **text** (terminal), **json** (CI/CD), **markdown** (docs), and **yaml** (config) formats with scores, grades, and recommendations.

### Fast vs Full Mode

**Fast Mode** (default):
- Time: ~2-3 minutes on large projects
- Skips: clippy (60-90s), mutation testing (hours), build time (minutes)
- Gives: Moderate credit for skipped checks
- Use case: Quick CI checks, development feedback

**Full Mode** (--full):
- Time: ~10-15 minutes on large projects
- Runs: All checks comprehensively
- Provides: Evidence-based, peer-reviewed scoring
- Use case: Release validation, comprehensive audits

### Performance

Fast mode (~2-3min) skips clippy, mutation testing, and build time. Full mode (~10-15min) runs all checks comprehensively.

### Evidence-Based Design

Scoring based on 15 peer-reviewed papers (2022-2025): reduced complexity weight (8→3pts, no bug correlation), increased unsafe code (6→9pts, Rust's core value), increased mutation testing (5→8pts, high developer value), tiered clippy scoring (correctness > suspicious > pedantic).

### CI/CD Integration

```bash
# In your CI pipeline (.github/workflows/quality.yml)
- name: Rust Project Score
  run: |
    pmat rust-project-score --format json --output score.json
    # Parse score and fail if below threshold
    SCORE=$(jq '.total_earned' score.json)
    if (( $(echo "$SCORE < 80" | bc -l) )); then
      echo "Score $SCORE below threshold"
      exit 1
    fi
```

### Implementation

**Location**: `server/src/services/rust_project_score/`
**Spec**: `docs/specifications/rust-project-score-v1.1-update.md`
**Methodology**: EXTREME TDD with 15 peer-reviewed references (IEEE, ACM, arXiv 2022-2025)

---

## CRITICAL: Renacer Golden Tracing - Transpile/Distributed Projects

**MANDATORY for**:
- Transpilers (Rust→JS, Python→C)
- Distributed systems
- Multi-process workflows
- Cross-language integrations

**Golden Tracing** = Record expected execution, validate future runs

```toml
# renacer.toml (project root)
[golden_traces]
enabled = true
trace_dir = "golden_traces/"

[[golden_traces.scenarios]]
name = "transpile_rust_to_js"
command = "pmat transpile --input test.rs --output test.js"
golden_trace = "golden_traces/transpile_rust_to_js.trace"
```

**Usage**:
```bash
# Capture golden trace (first time or after intentional behavior change)
renacer capture --scenario transpile_rust_to_js

# Validate before commits
renacer validate --all
# ✅ All traces match (100%)
# ❌ Diverged at step 42: Expected ACK, got timeout
```

**When**: Always validate golden traces before completing work.

---

## trueno-graph O(1) Context and TDG Integration

**STATUS**: ✅ ACTIVE (NOT feature-gated - USED in production code)
**Specification**: `docs/specifications/trueno-o1-context-tdg-integration.md`
**Work Item**: `trueno-o1-context-tdg`

### Overview

trueno-graph provides GPU-first CSR (Compressed Sparse Row) graph database for O(1) symbol lookups and PageRank-based importance scoring. Integrated into both context generation and TDG analysis.

### Usage Proof (NOT Feature-Gated)

#### 1. Context Generation (`server/src/services/context.rs`)

**Location**: context.rs:565-572, context_graph.rs:1-433
**Integration**: Every `analyze_project_with_cache()` call builds a ProjectContextGraph

```rust
// Line 565-572: context.rs - ACTIVE usage in all project analysis
pub async fn analyze_project_with_cache(...) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files = scan_and_analyze_files(root_path, toolchain, cache_manager, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    // Build O(1) graph for symbol lookups and PageRank
    let graph = build_context_graph(&files).ok();  // ← trueno-graph USED HERE

    Ok(ProjectContext { project_type: toolchain.to_string(), files, summary, graph })
}
```

**Evidence**:
- `ProjectContext.graph: Option<ProjectContextGraph>` (context.rs:62)
- `build_context_graph()` uses trueno-graph CSR (context.rs:955-989)
- O(1) symbol lookups via HashMap + PageRank via CSR
- **Tests passing**: 8/8 tests (7 context_graph + 1 integration)
- **Commit**: 9a34bd4b

#### 2. TDG Analysis (`server/src/tdg/tdg_graph.rs`)

**Location**: tdg_graph.rs:1-325
**Integration**: TdgGraph provides O(1) function dependency tracking with PageRank for critical test target identification

```rust
// Lines 51-78: TdgGraph structure using trueno-graph CSR
pub struct TdgGraph {
    graph: CsrGraph,                              // ← trueno-graph CSR
    node_map: HashMap<String, NodeId>,            // O(1) function lookups
    reverse_node_map: HashMap<NodeId, String>,
    criticality_scores: HashMap<String, f32>,     // PageRank results
    next_node_id: u32,
}

// PageRank identifies critical functions (line 172-198)
pub fn update_criticality(&mut self) -> Result<()> {
    let scores = pagerank(&self.graph, 20, 1e-6)?;  // ← trueno-graph PageRank
    self.criticality_scores.clear();
    for (node_id, score) in scores.iter().enumerate() {
        let node_id = NodeId(node_id as u32);
        if let Some(name) = self.reverse_node_map.get(&node_id) {
            self.criticality_scores.insert(name.clone(), *score);
        }
    }
    Ok(())
}
```

**Evidence**:
- TdgGraph created and integrated into TDG module (tdg/mod.rs:19)
- O(1) function lookups + PageRank criticality scoring
- **Tests passing**: 7/7 tests
- **Commit**: 82d25b7e

### Performance Targets

- **Context generation**: <5ms (baseline: 8ms) - 40% improvement
- **TDG analysis**: <10ms (baseline: 15ms) - 33% improvement
- **Symbol lookup**: O(1) guaranteed (HashMap)
- **PageRank**: 20 iterations, tolerance 1e-6

### Architecture Pattern

Both ProjectContextGraph and TdgGraph use the **dual storage pattern**:
1. **HashMap cache**: O(1) lookups (symbol name → data)
2. **CSR graph**: PageRank for importance scoring
3. **Bidirectional mapping**: NodeId ↔ symbol name

### Key Insight

CSR graphs only track nodes with edges, so `num_nodes()` returns node_map.len() (all added nodes) not graph.num_nodes() (nodes with edges). This was a critical bug fix in commit 9a34bd4b.

---

## DETERMINISTIC Agent Instructions

When implementing fixes or responding to UX issues, follow DETERMINISTIC instructions in:

**`docs/agent-instructions/`**

These documents provide:
- Step-by-step fix procedures
- Exact file locations and line numbers
- Before/after code examples
- Test cases to verify fixes
- Priority ordering for multiple issues

### Available Instructions:

1. **`pmat-work-ux-fixes.md`** - Fixes for `pmat work` command UX issues
   - Fuzzy ID matching (partial/case-insensitive)
   - Status display improvements
   - Quality gate optimizations
   - Short ID generation

2. **`pmat-work-quality-principles.md`** - MANDATORY quality principles for `pmat work`
   - Five Whys (ONLY debugging method)
   - Renacer golden tracing (transpile/distributed)
   - Rust project requirements (examples, scores)
   - Commit metadata linking (O(1) capture)

### Agent Workflow:

```bash
# 1. User reports UX issue
# 2. Read relevant instruction doc:
cat docs/agent-instructions/pmat-work-ux-fixes.md

# 3. Apply DETERMINISTIC fixes in priority order
# 4. Test each fix independently
# 5. Commit atomically with reference to instruction doc
```

**Rationale**: DETERMINISTIC instructions reduce hallucination risk and ensure consistent, high-quality fixes across agent sessions.

