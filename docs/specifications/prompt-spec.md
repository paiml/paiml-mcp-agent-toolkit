# PMAT Prompt System Specification

## Overview

The `pmat prompt` command provides pre-configured, reusable prompts for common development workflows that enforce EXTREME TDD and Toyota Way quality principles. Each prompt is stored in YAML format and can be emitted for use with AI assistants or CI/CD pipelines.

## Command Syntax

```bash
pmat prompt <prompt-name> [options]
```

### Options

- `--format <FORMAT>` - Output format: yaml, json, text (default: yaml)
- `--output <FILE>` - Write to file instead of stdout
- `--list` - List all available prompts
- `--show-variables` - Show prompt variables that can be customized
- `--set <KEY=VALUE>` - Override prompt variables (can be repeated)

## Default Assumptions & Language Support

### Default Target: Rust Projects

All prompts assume a **Rust project** by default, using:
- **PMAT commands** (universal, work on any language):
  - `pmat analyze` - code analysis
  - `pmat tdg` - technical debt grading
  - `pmat context` - deep context generation
  - `pmat mutate` - mutation testing
  - `pmat validate-docs` - documentation validation
  - `pmat validate-readme` - hallucination detection

- **Rust tooling** (default, can be overridden):
  - `cargo test` - run tests
  - `cargo clippy` - linting
  - `cargo fmt` - formatting
  - `cargo llvm-cov` - coverage
  - `make test-fast` - fast test suite
  - `make coverage` - coverage report

### Simple Case (90% of users)

Just use the prompt as-is for Rust+PMAT projects:

```bash
pmat prompt code-coverage       # Works immediately on Rust projects
pmat prompt continue            # Uses Rust tooling by default
pmat prompt debug               # Assumes Cargo/Rust environment
```

### Language Override (10% of users)

For non-Rust projects, override via variables:

```bash
# Python project
pmat prompt code-coverage \
  --set TEST_CMD="pytest" \
  --set COVERAGE_CMD="pytest --cov" \
  --set LINT_CMD="pylint"

# JavaScript project
pmat prompt code-coverage \
  --set TEST_CMD="npm test" \
  --set COVERAGE_CMD="jest --coverage" \
  --set LINT_CMD="eslint"

# Go project
pmat prompt code-coverage \
  --set TEST_CMD="go test ./..." \
  --set COVERAGE_CMD="go test -coverprofile=coverage.out" \
  --set LINT_CMD="golint"
```

### Why This Design?

1. **Simplicity First**: Most users are PMAT maintainers (Rust developers)
2. **PMAT is Universal**: `pmat analyze/tdg/context` work on ANY language
3. **Override When Needed**: Variables support other languages
4. **No Magic**: Explicit commands, no auto-detection complexity

### Supported Languages

PMAT analyzes **all these languages**:
- Rust, Python, JavaScript/TypeScript, Go, Java, C/C++, Ruby, PHP, Swift, Kotlin, C#, Bash, WASM

The prompts work with any of them - just override the build/test commands.

## Core Prompts

### 1. code-coverage

**Purpose**: Ensure all code coverage is greater than 85% using EXTREME TDD methodology.

**YAML Format**:
```yaml
name: code-coverage
description: Ensure code coverage >85% using EXTREME TDD
category: quality
priority: critical
methodology: EXTREME TDD
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
heuristics:
  - uncovered code first
  - low coverage with low TDG score
coverage_target: 85
testing_approaches:
  - mutation testing
  - property-based testing
  - cargo run --example
  - pmat tdg enhanced testing
prompt: |
  All code coverage must be greater than 85%. Continue next best recommended step or roadmap using EXTREME TDD
  (mutation/property/cargo run --example, pmat tdg enhanced testing) that respects (make coverage <10min,
  make test-fast under <5 min, and pre-commit test < 30 seconds).

  Use Heuristic:
  1. Uncovered code
  2. Low coverage with low TDG score

  If you spot a defect due to unimplemented or partially implemented functionality, STOP THE LINE and implement
  using EXTREME TDD. The concept of "pre-existing failure" is irrelevant, fix.
toyota_way_principles:
  - jidoka: stop_the_line
  - andon_cord: true
  - genchi_genbutsu: verify_actual_state
```

### 2. clean-repo-cruft

**Purpose**: Remove temporary files and cruft from repository root.

**YAML Format**:
```yaml
name: clean-repo-cruft
description: Remove all temporary files from repository root
category: maintenance
priority: medium
prompt: |
  Remove all temporary files from root of repository and add these patterns to .gitignore:
  - defect-report-*.txt
  - defect-report-*.json
  - defect-report-*.csv
  - defect-report-*.md
  - *.tmp
  - *.bak
  - .DS_Store
  - Thumbs.db

  Steps:
  1. Identify all temporary files in repo root and server/ directory
  2. Review files to ensure they're safe to delete
  3. Delete temporary files
  4. Update .gitignore with patterns
  5. Verify git status shows clean working tree
  6. Commit and push all changes

  Quality Gate: Verify no temporary files remain after cleanup.
validation:
  - git status --short | wc -l == 0
  - git ls-files --others --exclude-standard | wc -l == 0
```

### 3. continue

**Purpose**: Continue with next best recommended step using EXTREME TDD.

**YAML Format**:
```yaml
name: continue
description: Continue next best recommended step using EXTREME TDD
category: workflow
priority: high
methodology: EXTREME TDD
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
prompt: |
  Continue next best recommended step or roadmap using EXTREME TDD (mutation/property/cargo run --example,
  pmat tdg enhanced testing) that respects (make coverage <10min, make test-fast under <5 min, and
  pre-commit test < 30 seconds).

  If you spot a defect due to unimplemented or partially implemented functionality, STOP THE LINE and
  implement using EXTREME TDD. The concept of "pre-existing failure" is irrelevant, fix.

  Workflow:
  1. Run pmat analyze to identify issues
  2. Run pmat tdg to find highest debt
  3. Prioritize using heuristics (uncovered, low-coverage+low-TDG)
  4. Implement fix using RED-GREEN-REFACTOR
  5. Verify all quality gates pass
  6. Commit with descriptive message
toyota_way_principles:
  - kaizen: continuous_improvement
  - pdca_cycle: plan_do_check_act
```

### 4. assert-cmd-testing

**Purpose**: Verify all CLI options, flags, and arguments are covered with assert_cmd tests.

**YAML Format**:
```yaml
name: assert-cmd-testing
description: Count and verify all CLI variations are tested with assert_cmd
category: testing
priority: high
methodology: EXTREME TDD
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
heuristics:
  - uncovered code first
  - low coverage with low TDG score
prompt: |
  Count all variations of CLI options, flags and arguments, then verify all are covered using assert_cmd
  style testing. Use EXTREME TDD (mutation/property/cargo run --example, pmat tdg enhanced testing) that
  respects (make coverage <10min, make test-fast under <5 min, and pre-commit test < 30 seconds).

  Use Heuristic:
  1. Uncovered code
  2. Low coverage with low TDG score

  Steps:
  1. Parse CLI definition (clap, structopt, etc.)
  2. Count total combinations (commands × subcommands × flags × options)
  3. Count existing assert_cmd tests
  4. Generate RED tests for missing coverage
  5. Implement GREEN code to pass tests
  6. Refactor and optimize

  If you spot a defect due to unimplemented or partially implemented functionality, STOP THE LINE and
  implement using EXTREME TDD. The concept of "pre-existing failure" is irrelevant, fix.
quality_gates:
  - all CLI paths tested
  - 100% command coverage
  - 100% flag/option coverage
```

### 5. documentation

**Purpose**: Update and validate all documentation for accuracy and completeness.

**YAML Format**:
```yaml
name: documentation
description: Update all documentation and verify accuracy with pmat validate-docs
category: documentation
priority: medium
prompt: |
  Update all documentation, roadmap, tickets, books, README.md and verify all are up to date and accurate
  and do not contain missing information or broken links by using pmat validate-docs.

  Steps:
  1. Update README.md with latest features/changes
  2. Update ROADMAP.md with completed/in-progress items
  3. Update CHANGELOG.md with recent changes
  4. Update docs/ directory with specification changes
  5. Update pmat-book if CLI changes were made
  6. Run pmat validate-docs --fail-on-broken-links
  7. Run pmat validate-readme --fail-on-contradiction
  8. Fix any hallucinations or broken references
  9. Commit and push documentation changes
  10. If pmat-book was updated, push book changes FIRST

  Quality Gates:
  - pmat validate-docs passes (no broken links)
  - pmat validate-readme passes (no hallucinations)
  - All references point to valid files/URLs
  - pmat-book validation passes (if book updated)
validation_tools:
  - pmat validate-docs
  - pmat validate-readme
  - make validate-book
zero_tolerance:
  - broken_links: false
  - hallucinations: false
  - 404_errors: false
```

### 6. debug

**Purpose**: Debug issues using Five Whys root cause analysis and implement permanent fixes.

**YAML Format**:
```yaml
name: debug
description: Debug using Five Whys and implement root cause fix with EXTREME TDD
category: debugging
priority: critical
methodology: EXTREME TDD + Five Whys
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
heuristics:
  - uncovered code first
  - low coverage with low TDG score
prompt: |
  Debug this issue using Five Whys root cause analysis and a permanent fix that solves root cause using
  EXTREME TDD (mutation/property/cargo run --example, pmat tdg enhanced testing) that respects
  (make coverage <10min, make test-fast under <5 min, and pre-commit test < 30 seconds).

  Use Heuristic:
  1. Uncovered code
  2. Low coverage with low TDG score

  Five Whys Process:
  1. Why did this problem occur? [Surface symptom]
  2. Why did that happen? [Immediate cause]
  3. Why did that happen? [Underlying cause]
  4. Why did that happen? [Systemic issue]
  5. Why did that happen? [ROOT CAUSE]

  Implementation:
  1. Write RED test that reproduces the issue
  2. Apply Five Whys to find root cause
  3. Implement GREEN fix that addresses root cause (not symptom)
  4. REFACTOR to prevent recurrence
  5. Add regression test
  6. Document root cause in commit message

  Quality Gate: Issue must not recur under any circumstances.
toyota_way_principles:
  - five_whys: true
  - root_cause_analysis: required
  - genchi_genbutsu: go_to_the_source
```

## Additional Prompts (Based on Git History & PMAT Capabilities)

### 7. mutation-testing

**Purpose**: Run mutation testing on high-complexity or low-coverage code.

**YAML Format**:
```yaml
name: mutation-testing
description: Run mutation testing on high-complexity/low-coverage code
category: testing
priority: high
methodology: EXTREME TDD + Mutation Testing
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
prompt: |
  Run mutation testing on code with high complexity or low coverage using EXTREME TDD approach.

  Steps:
  1. Run pmat analyze to find high-complexity functions
  2. Run pmat tdg to identify low TDG scores
  3. Prioritize files with complexity >10 or TDG <50
  4. Run pmat mutate --files <high-priority-files>
  5. For each surviving mutant:
     a. Write RED test that kills the mutant
     b. Verify GREEN test kills mutant
     c. REFACTOR for clarity
  6. Repeat until mutation score >80%

  Quality Gates:
  - Mutation score >80%
  - All critical paths have mutant-killing tests
  - make test-fast <5min

  If you spot a defect due to unimplemented or partially implemented functionality, STOP THE LINE and
  implement using EXTREME TDD. The concept of "pre-existing failure" is irrelevant, fix.
mutation_targets:
  - complexity: ">10"
  - coverage: "<85%"
  - tdg_score: "<50"
mutation_score_target: 80
```

### 8. performance-optimization

**Purpose**: Speed up compilation and test execution using Five Whys analysis.

**YAML Format**:
```yaml
name: performance-optimization
description: Optimize compilation and test performance using Five Whys
category: performance
priority: high
methodology: Five Whys + EXTREME TDD
constraints:
  - make coverage <10min (TARGET)
  - make test-fast <5min (TARGET)
  - pre-commit test <30s (TARGET)
prompt: |
  Speed up compilation and test execution using Five Whys root cause analysis.

  Current Targets:
  - make coverage: <10 minutes
  - make test-fast: <5 minutes (ideally <3 minutes)
  - pre-commit test: <30 seconds

  Five Whys Process:
  1. Why is compilation/testing slow?
  2. Why is that happening?
  3. Why is that happening?
  4. Why is that happening?
  5. Why is that happening? [ROOT CAUSE]

  Common Optimizations:
  1. Exclude slow tests (>60s) from test-fast using #[cfg(not(feature = "skip-slow-tests"))]
  2. Enable mold linker in .cargo/config.toml
  3. Reduce codegen-units for faster linking
  4. Use cargo-nextest for parallel test execution
  5. Mark integration tests as #[ignore] if they require binary
  6. Split large test files into smaller modules
  7. Use feature flags to exclude optional heavy dependencies

  Implementation:
  1. Measure baseline: time make test-fast
  2. Profile to find bottlenecks
  3. Apply optimizations one at a time
  4. Measure improvement after each change
  5. Document optimization in commit message

  Quality Gate: All targets must be met, all tests must still pass.
optimization_targets:
  - compilation_time: "<2min"
  - test_execution_time: "<3min"
  - pre_commit_time: "<30s"
tools:
  - cargo-nextest
  - mold linker
  - feature flags
```

### 9. quality-enforcement

**Purpose**: Run all quality gates and enforce extreme quality standards.

**YAML Format**:
```yaml
name: quality-enforcement
description: Run all quality gates and enforce extreme quality standards
category: quality
priority: critical
methodology: EXTREME TDD + Toyota Way
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
prompt: |
  Run all quality gates and enforce extreme quality standards using EXTREME TDD methodology.

  Quality Gates:
  1. Compilation: cargo build --all-features
  2. Linting: cargo clippy --all-targets --all-features -- -D warnings
  3. Formatting: cargo fmt -- --check
  4. Tests: make test-fast (must pass 100%)
  5. Coverage: make coverage (must be >85%)
  6. Mutation: pmat mutate (score >80%)
  7. Complexity: pmat analyze (max complexity <15)
  8. TDG: pmat tdg (average score >60)
  9. Documentation: pmat validate-docs (no broken links)
  10. README: pmat validate-readme (no hallucinations)
  11. Book: make validate-book (all tests pass)
  12. Bash: bashrs lint Makefile scripts/*.sh

  Enforcement Process:
  1. Run all quality gates
  2. If ANY gate fails, STOP THE LINE (Toyota Way - Andon Cord)
  3. Apply Five Whys to find root cause
  4. Fix using EXTREME TDD (RED-GREEN-REFACTOR)
  5. Re-run all gates
  6. Repeat until ALL gates pass
  7. Only then commit and push

  Zero Tolerance Policy:
  - Compilation warnings: 0
  - Clippy warnings: 0
  - Test failures: 0
  - Coverage: <85% rejected
  - Broken links: 0
  - Hallucinations: 0

  Quality Gate: ALL gates must pass before any commit.
toyota_way_principles:
  - jidoka: built_in_quality
  - andon_cord: stop_if_quality_issue
  - poka_yoke: mistake_proofing
  - genchi_genbutsu: verify_actual_state
quality_gates:
  - compilation: required
  - clippy: required
  - fmt: required
  - test_fast: required
  - coverage: required
  - mutation: recommended
  - complexity: required
  - tdg: required
  - docs: required
  - bash_lint: required
```

### 10. refactor-hotspots

**Purpose**: Refactor code with high TDG scores or low coverage using EXTREME TDD.

**YAML Format**:
```yaml
name: refactor-hotspots
description: Refactor high-TDG/low-coverage code using EXTREME TDD
category: refactoring
priority: high
methodology: EXTREME TDD + TDG Analysis
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
heuristics:
  - high TDG score (>80)
  - low coverage (<85%)
  - high complexity (>15)
prompt: |
  Refactor code hotspots using EXTREME TDD methodology.

  Hotspot Identification:
  1. Run pmat tdg to find high TDG scores (>80)
  2. Run pmat analyze to find high complexity (>15)
  3. Run make coverage to find low coverage (<85%)
  4. Prioritize files that are in ALL three categories

  Refactoring Process (EXTREME TDD):
  1. Write RED tests for current behavior (characterization tests)
  2. Verify GREEN (tests pass with current implementation)
  3. REFACTOR in small steps:
     a. Extract function
     b. Run tests (must stay GREEN)
     c. Extract another function
     d. Run tests (must stay GREEN)
     e. Simplify logic
     f. Run tests (must stay GREEN)
  4. Add property-based tests for invariants
  5. Run mutation testing to verify test quality
  6. Verify TDG score improved by at least 20 points

  Quality Gates:
  - All tests pass throughout refactoring
  - Coverage increases or stays at 85%+
  - Complexity decreases by at least 30%
  - TDG score improves by at least 20 points
  - make test-fast <5min

  If you spot a defect due to unimplemented or partially implemented functionality, STOP THE LINE and
  implement using EXTREME TDD. The concept of "pre-existing failure" is irrelevant, fix.
refactoring_targets:
  - tdg_score: ">80"
  - coverage: "<85%"
  - complexity: ">15"
improvement_goals:
  - tdg_improvement: "+20 points"
  - complexity_reduction: "-30%"
  - coverage_target: ">85%"
```

### 11. security-audit

**Purpose**: Run security analysis and fix vulnerabilities using EXTREME TDD.

**YAML Format**:
```yaml
name: security-audit
description: Run security analysis and fix vulnerabilities with EXTREME TDD
category: security
priority: critical
methodology: EXTREME TDD + Security Analysis
constraints:
  - make coverage <10min
  - make test-fast <5min
  - pre-commit test <30s
prompt: |
  Run security audit and fix all vulnerabilities using EXTREME TDD methodology.

  Security Checks:
  1. Run cargo audit to check for known vulnerabilities
  2. Run bashrs lint to check for shell injection issues
  3. Run pmat analyze to find potential security issues:
     - SQL injection points
     - Command injection
     - Path traversal
     - Unsafe deserialization
     - Unvalidated input
  4. Run clippy with security lints enabled

  Fix Process (EXTREME TDD):
  1. For each vulnerability:
     a. Write RED test that exploits the vulnerability
     b. Verify test fails (demonstrates vulnerability)
     c. Implement GREEN fix (secure code)
     d. Verify test passes (vulnerability fixed)
     e. REFACTOR for clarity
     f. Add fuzzing/property tests for security properties
  2. Update dependencies: cargo update
  3. Re-run all security checks
  4. Document security fixes in CHANGELOG

  Security Standards:
  - Zero known vulnerabilities (cargo audit)
  - Zero shell injection issues (bashrs)
  - All user input validated
  - All file paths sanitized
  - All SQL/command execution parameterized
  - All sensitive data encrypted
  - All secrets in environment variables (not code)

  Quality Gate: Zero security vulnerabilities allowed.
security_tools:
  - cargo audit
  - bashrs lint
  - clippy security lints
  - cargo-crev
  - cargo-geiger
vulnerability_tolerance: 0
```

## Implementation Plan

### Phase 1: Core Infrastructure (Sprint 1)

1. **Create prompt storage system**
   - Define YAML schema for prompts
   - Implement prompt loading/parsing
   - Add validation for prompt structure
   - Test: RED test for invalid YAML, GREEN for valid

2. **Implement `pmat prompt` command**
   - Add CLI subcommand to main.rs
   - Parse command-line options
   - Wire up to handler
   - Test: assert_cmd tests for all flags

3. **Add prompt handler**
   - Create `server/src/cli/handlers/prompt_handler.rs`
   - Implement list functionality
   - Implement show functionality
   - Test: Unit tests for handler logic

### Phase 2: Prompt Management (Sprint 2)

4. **Store prompts in embedded resources**
   - Create `server/prompts/` directory
   - Add YAML files for all 11 prompts
   - Use `include_str!` or similar for embedding
   - Test: Verify all prompts load correctly

5. **Add variable substitution**
   - Support `${VAR}` syntax in prompts
   - Implement `--set KEY=VALUE` option
   - Add `--show-variables` option
   - Test: Property tests for variable substitution

6. **Add output formats**
   - YAML output (default)
   - JSON output (`--format json`)
   - Plain text output (`--format text`)
   - Test: Verify each format is valid

### Phase 3: Integration & Testing (Sprint 3)

7. **Integration with existing tools**
   - pmat analyze
   - pmat tdg
   - pmat mutate
   - pmat validate-docs
   - Test: E2E tests for prompt workflows

8. **Documentation**
   - Add to README.md
   - Add to pmat-book
   - Add examples
   - Test: pmat validate-docs passes

9. **Quality gates**
   - Coverage >85%
   - All tests pass
   - Mutation score >80%
   - make test-fast <5min
   - Test: All quality gates pass

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_prompt_success() {
        let prompt = Prompt::load("code-coverage").unwrap();
        assert_eq!(prompt.name, "code-coverage");
        assert!(prompt.coverage_target == 85);
    }

    #[test]
    fn test_load_prompt_not_found() {
        let result = Prompt::load("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_substitution() {
        let mut prompt = Prompt::load("code-coverage").unwrap();
        prompt.set_variable("coverage_target", "90");
        assert!(prompt.render().contains("90"));
    }

    #[test]
    fn test_list_prompts() {
        let prompts = Prompt::list();
        assert!(prompts.len() >= 11);
        assert!(prompts.contains(&"code-coverage".to_string()));
    }
}
```

### Integration Tests (assert_cmd)

```rust
#[test]
fn test_pmat_prompt_code_coverage() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["prompt", "code-coverage"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: code-coverage"))
        .stdout(predicate::str::contains("coverage_target: 85"));
}

#[test]
fn test_pmat_prompt_list() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["prompt", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("code-coverage"))
        .stdout(predicate::str::contains("clean-repo-cruft"))
        .stdout(predicate::str::contains("continue"));
}

#[test]
fn test_pmat_prompt_set_variable() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["prompt", "code-coverage", "--set", "coverage_target=90"])
        .assert()
        .success()
        .stdout(predicate::str::contains("coverage_target: 90"));
}

#[test]
fn test_pmat_prompt_json_format() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["prompt", "code-coverage", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains("\"name\": \"code-coverage\""));
}
```

### Property-Based Tests

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_all_prompts_load_successfully(
            prompt_name in prop::sample::select(Prompt::list())
        ) {
            let result = Prompt::load(&prompt_name);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_variable_substitution_idempotent(
            key in "[a-z_]+",
            value in "[a-zA-Z0-9_-]+"
        ) {
            let mut prompt = Prompt::load("code-coverage").unwrap();
            prompt.set_variable(&key, &value);
            let rendered1 = prompt.render();
            let rendered2 = prompt.render();
            prop_assert_eq!(rendered1, rendered2);
        }
    }
}
```

## File Structure

```
server/
├── prompts/                          # Embedded prompt YAML files
│   ├── code-coverage.yaml
│   ├── clean-repo-cruft.yaml
│   ├── continue.yaml
│   ├── assert-cmd-testing.yaml
│   ├── documentation.yaml
│   ├── debug.yaml
│   ├── mutation-testing.yaml
│   ├── performance-optimization.yaml
│   ├── quality-enforcement.yaml
│   ├── refactor-hotspots.yaml
│   └── security-audit.yaml
├── src/
│   ├── cli/
│   │   ├── handlers/
│   │   │   ├── prompt_handler.rs    # Main prompt handler
│   │   │   └── prompt_handler_tests.rs
│   │   └── commands.rs               # Add prompt subcommand
│   ├── models/
│   │   └── prompt.rs                 # Prompt data model
│   └── services/
│       └── prompt_service.rs         # Prompt loading/rendering
└── tests/
    └── prompt_integration_tests.rs   # assert_cmd tests
```

## Quality Gates

Before marking this feature complete, ALL of the following must pass:

1. **Compilation**: `cargo build --all-features` (zero warnings)
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: `make test-fast` (<5 min, 100% passing)
4. **Coverage**: `make coverage` (>85%, <10 min)
5. **Integration**: All 11 prompts load and render correctly
6. **CLI Tests**: All assert_cmd tests pass
7. **Documentation**: pmat-book updated with prompt examples
8. **Validation**: `pmat validate-docs` passes (no broken links)

## Toyota Way Principles Applied

- **Jidoka (Built-in Quality)**: Each prompt enforces quality gates
- **Andon Cord**: "STOP THE LINE" language in prompts
- **Five Whys**: Built into debug and performance-optimization prompts
- **Genchi Genbutsu**: Prompts encourage going to actual source
- **Kaizen**: Continue prompt enables continuous improvement
- **PDCA Cycle**: Plan-Do-Check-Act embedded in workflow

## Success Criteria

1. ✅ All 11 prompts implemented and tested
2. ✅ `pmat prompt <name>` works for all prompts
3. ✅ Variable substitution works correctly
4. ✅ All output formats (YAML, JSON, text) work
5. ✅ All quality gates pass
6. ✅ Documentation complete and accurate
7. ✅ make test-fast <5 min
8. ✅ make coverage <10 min
9. ✅ Coverage >85%

## Future Enhancements

1. **Custom Prompts**: Allow users to define custom prompts in `.pmat/prompts/`
2. **Prompt Composition**: Combine multiple prompts (e.g., `code-coverage` + `mutation-testing`)
3. **Interactive Mode**: `pmat prompt --interactive` to select prompt with TUI
4. **Prompt History**: Track which prompts were used and outcomes
5. **Prompt Metrics**: Measure effectiveness of each prompt
6. **AI Integration**: Direct integration with Claude Code/Gemini/ChatGPT APIs
