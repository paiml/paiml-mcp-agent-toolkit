# Project Scaffolding and Maintenance Specification

**Version**: 1.0.0
**Status**: Approved
**Date**: 2025-10-05
**Authors**: PMAT Development Team

---

## Table of Contents

1. [Overview](#overview)
2. [Core Rules](#core-rules)
3. [Project Scaffolding](#project-scaffolding)
4. [Project Maintenance](#project-maintenance)
5. [Quality Gates](#quality-gates)
6. [Implementation Architecture](#implementation-architecture)
7. [Testing Strategy](#testing-strategy)
8. [Success Metrics](#success-metrics)

---

## Overview

This specification defines a comprehensive system for scaffolding new projects and maintaining existing projects with extreme quality standards. The system enforces Toyota Way principles and extreme TDD methodologies across all projects.

### Design Principles

1. **Roadmap-Driven Development**: All work tracked in `roadmap.md` or `ROADMAP.md`
2. **Ticket-Based Execution**: All features implemented through linked tickets
3. **Extreme TDD**: Complexity <10, no SATD, >80% coverage, mutation + property testing
4. **Automation First**: Pre-commit hooks enforce quality gates
5. **Living Documentation**: All specs, tickets, and roadmaps are living documents

---

## Core Rules

### Rule A: Always Use Roadmap

**Requirements**:
- Every project MUST have a `roadmap.md` or `ROADMAP.md` at the project root
- Roadmap MUST contain:
  - Current sprint/milestone status
  - Planned work with priority ordering
  - Completed work with links to commits/PRs
  - Success metrics and KPIs
  - Risk assessment

**Roadmap Structure**:
```markdown
# Project Roadmap

## Current Status: [Version] - [Brief Description]

## Active Sprint: [Sprint Name]
- **Status**: [Not Started | In Progress | Complete]
- **Focus**: [Primary objective]
- **Tickets**:
  - [ ] TICKET-XXX: [Description]
  - [x] TICKET-YYY: [Description] (commit: abc123)

## Planned Sprints
1. **Sprint N**: [Objective] - [Estimated Days]
2. **Sprint N+1**: [Objective] - [Estimated Days]

## Completed Releases
### v1.2.0 - [Feature Name] (YYYY-MM-DD)
- ✅ [Achievement 1]
- ✅ [Achievement 2]
- **Quality**: Coverage X%, Complexity <Y, 0 SATD
- **Commits**: abc123, def456
```

**Validation**:
- `pmat analyze roadmap --validate` - Verify roadmap structure
- Pre-commit hook checks roadmap exists and is up-to-date

---

### Rule B: Always Have Tickets for Work Linked in Roadmap

**Requirements**:
- All roadmap items MUST link to ticket files in `docs/tickets/`
- Ticket format: `TICKET-[PROJECT]-[NUMBER].md` (e.g., `TICKET-PMAT-1001.md`)
- Each ticket MUST contain:
  - Clear objective and success criteria
  - Estimated complexity and time
  - Dependencies on other tickets
  - Test strategy (unit, integration, property tests)
  - Quality gates to pass

**Ticket Template**:
```markdown
# TICKET-[PROJECT]-[NUMBER]: [Title]

**Status**: [RED | GREEN | REFACTOR | COMPLETE]
**Priority**: [P0 | P1 | P2]
**Complexity**: [1-10]
**Estimated Time**: [X hours/days]
**Dependencies**: [List of TICKET-IDs]

## Objective
[Clear description of what needs to be done]

## Success Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] All quality gates pass

## Test Strategy
### Unit Tests
- [ ] Test case 1
- [ ] Test case 2

### Property Tests
- [ ] Property 1: [Description]
- [ ] Property 2: [Description]

### Integration Tests
- [ ] Integration scenario 1

## Quality Gates
- [ ] Complexity <10 for all functions
- [ ] Coverage >80%
- [ ] 0 SATD violations
- [ ] Mutation score >90%
- [ ] No lint/test failures

## Implementation Notes
[Design decisions, trade-offs, etc.]

## Verification
```bash
# Commands to verify completion
cargo test --test ticket_[number]_*
cargo llvm-cov report
pmat analyze complexity --path src/[module].rs
```
```

---

### Rule C: Extreme TDD with High Quality Standards

**Quality Requirements**:

#### 1. Complexity Limits
- **Cyclomatic Complexity**: ≤10 per function
- **Cognitive Complexity**: ≤15 per function
- **Nesting Depth**: ≤4 levels
- **Function Length**: ≤50 lines

**Validation**: `pmat analyze complexity --max-cyclomatic 10 --max-cognitive 15`

#### 2. No SATD (Self-Admitted Technical Debt)
- **Zero tolerance**: No TODO, FIXME, HACK, XXX comments
- **Alternative**: Create tickets for future work
- **Exception**: Short-term RED phase markers (removed in GREEN phase)

**Validation**: `pmat analyze satd --strict`

#### 3. Entropy Management
- **Watch entropy score**: Monitor code duplication and pattern violations
- **Threshold**: Entropy violations flagged in CI
- **Refactoring**: Extract patterns into reusable components

**Validation**: `pmat analyze entropy --threshold 8.0`

#### 4. Big-O Analysis
- **Document algorithmic complexity** in function docstrings
- **Target**: O(1), O(log n), or O(n) for most operations
- **Justify**: Document and justify O(n²) or worse

**Example**:
```rust
/// Finds matching items in the registry.
///
/// # Complexity
/// - Time: O(n) where n is number of registered items
/// - Space: O(1) - no allocations
///
/// # Performance
/// Benchmarked at <100ns for n≤1000 using FxHash
pub fn find(&self, key: &str) -> Option<&Item> {
    self.map.get(key)
}
```

#### 5. Provability
- **Formal properties**: Use property-based testing (proptest)
- **Invariants**: Document and test invariants
- **Contracts**: Pre/post-conditions in docstrings

**Example**:
```rust
#[test]
fn prop_registry_lookup_consistent() {
    proptest!(|(name: String, value: u32)| {
        let mut registry = Registry::new();
        registry.insert(name.clone(), value);

        // Property: Lookup always returns inserted value
        assert_eq!(registry.get(&name), Some(&value));

        // Property: Multiple lookups are idempotent
        assert_eq!(registry.get(&name), registry.get(&name));
    });
}
```

#### 6. Test Coverage
- **Minimum**: 80% line coverage
- **Target**: 90% line coverage, 95% branch coverage
- **Exclusions**: Only generated code and infallible panic paths

**Validation**: `cargo llvm-cov --fail-under-lines 80`

#### 7. Mutation Testing
- **Minimum**: 85% mutation score
- **Target**: 90%+ mutation score
- **Strategy**: Property tests catch more mutants than example-based tests

**Validation**: `pmat analyze mutate --min-score 85`

#### 8. Lint and Test Failures
- **Zero tolerance**: All clippy warnings as errors
- **All tests pass**: No ignored tests in main branch
- **Format**: `cargo fmt --check` passes

**Pre-commit Hook**:
```bash
#!/bin/bash
set -e
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
pmat analyze complexity --max-cyclomatic 10
pmat analyze satd --strict
```

---

## Project Scaffolding

### Agent Project Scaffolding (using pforge)

**Reference Implementation**: `../pforge` (from crates.io)

**Workflow**:
```bash
# Scaffold new MCP agent using pforge best practices
pmat scaffold agent --name my-agent --template pforge

# Generates:
# my-agent/
# ├── Cargo.toml (workspace with pforge-runtime dependency)
# ├── pforge.yaml (agent configuration)
# ├── ROADMAP.md (initial roadmap)
# ├── docs/
# │   ├── specifications/
# │   └── tickets/
# ├── src/
# │   ├── lib.rs
# │   └── handlers/
# ├── tests/
# │   ├── unit/
# │   ├── integration/
# │   └── properties/
# ├── .git/hooks/pre-commit (quality gates)
# └── CLAUDE.md (agent instructions)
```

**Generated Structure**:
1. **Workspace configuration** with pforge-runtime
2. **pforge.yaml** with tool, resource, and prompt definitions
3. **ROADMAP.md** with initial sprints
4. **Pre-commit hooks** enforcing quality gates
5. **Test scaffolding** with examples
6. **CLAUDE.md** with project-specific instructions

**Best Practices from pforge**:
- Handler trait implementation pattern
- FxHash-based registry (83-90ns dispatch)
- Middleware chain for cross-cutting concerns
- SIMD JSON parsing (16x faster via pmcp)
- Zero-copy deserialization
- Lock-free concurrency with DashMap

### WASM Project Scaffolding (using wasm-labs)

**Reference Implementation**: `../wasm-labs` (Rust WASM best practices)

**Workflow**:
```bash
# Scaffold new WASM project using wasm-labs patterns
pmat scaffold wasm --name my-wasm-lib --template wasm-labs

# Generates:
# my-wasm-lib/
# ├── Cargo.toml (wasm32-unknown-unknown target)
# ├── Makefile (wasm-full, quality gates)
# ├── ROADMAP.md (initial roadmap)
# ├── docs/
# │   ├── specifications/
# │   └── tickets/
# ├── src/
# │   ├── lib.rs
# │   ├── vfs/ (if needed - persistent data structures)
# │   └── context/ (if needed - deterministic execution)
# ├── tests/
# │   ├── unit/
# │   ├── properties/ (proptest)
# │   └── wasm_quality.rs (size/import checks)
# ├── benches/
# │   └── performance.rs (criterion)
# ├── .git/hooks/pre-commit
# └── CLAUDE.md
```

**Best Practices from wasm-labs**:
- Pure functional design (explicit state threading)
- `wasm32-unknown-unknown` target (no WASI imports)
- Persistent data structures (im-rs for O(1) cloning)
- Deterministic execution (seeded RNG, simulated clock)
- WASM quality gates:
  - Binary size <500KB uncompressed, <100KB gzipped
  - No WASI imports (verified with wasm-objdump)
  - Complexity ≤20 per function
- Property-based testing (10K inputs per test)
- Coverage targets: 85%+ line, 90%+ branch
- Mutation testing: 90%+ kill rate

**Makefile Targets (from wasm-labs)**:
```makefile
make wasm-full      # Complete WASM pipeline with report
make quality        # Fast quality gates (test, lint, complexity, SATD)
make quality-full   # All gates + mutation testing
make coverage       # Generate coverage report (cargo llvm-cov)
make mutation       # Mutation testing (cargo-mutants or pmat)
make dev            # Live development server with auto-rebuild
```

---

## Project Maintenance

### Living Documentation

**Requirements**:
- All specifications in `docs/specifications/`
- All tickets in `docs/tickets/`
- Roadmap updated with each commit
- ADRs (Architecture Decision Records) for major decisions

**Update Workflow**:
```bash
# After completing a ticket
1. Mark ticket as COMPLETE in file
2. Update ROADMAP.md with completion
3. Link commit hash in roadmap
4. Update success metrics (coverage, complexity, etc.)
5. Commit with format: "[TICKET-XXX] Brief description"
```

### Pre-commit Hook Management

**Auto-generated pre-commit hook**:
```bash
#!/bin/bash
# Generated by PMAT scaffold system
set -e

echo "🔬 Running quality gates..."

# Fast checks (<30s)
cargo fmt --check || { echo "❌ Format check failed"; exit 1; }
cargo clippy --all-targets -- -D warnings || { echo "❌ Clippy failed"; exit 1; }
cargo test --lib || { echo "❌ Tests failed"; exit 1; }

# Complexity check
pmat analyze complexity \
  --max-cyclomatic 10 \
  --max-cognitive 15 \
  --path $(pwd) \
  || { echo "❌ Complexity check failed"; exit 1; }

# SATD check (only staged files)
git diff --cached --name-only | \
  grep -E '\.(rs|md|toml)$' | \
  xargs pmat analyze satd --strict \
  || { echo "❌ SATD check failed"; exit 1; }

# Roadmap validation
if [ -f "ROADMAP.md" ] || [ -f "roadmap.md" ]; then
  pmat analyze roadmap --validate || { echo "⚠️ Roadmap may need updating"; }
fi

echo "✅ All quality gates passed"
```

**Installation**:
- Automatically installed by `pmat scaffold`
- Manual installation: `pmat install-hooks`
- Bypass (not recommended): `git commit --no-verify`

### Continuous Integration

**GitHub Actions Workflow** (auto-generated):
```yaml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install PMAT
        run: cargo install pmat

      - name: Format Check
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Tests
        run: cargo test --all-features

      - name: Coverage
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --fail-under-lines 80

      - name: Complexity Analysis
        run: pmat analyze complexity --max-cyclomatic 10

      - name: SATD Check
        run: pmat analyze satd --strict

      - name: Mutation Testing (on main)
        if: github.ref == 'refs/heads/main'
        run: pmat analyze mutate --min-score 85
```

### Roadmap Synchronization

**Auto-update roadmap on commit**:
```bash
# Post-commit hook (optional)
#!/bin/bash

# Extract ticket ID from commit message
TICKET=$(git log -1 --pretty=%B | grep -oE 'TICKET-[A-Z]+-[0-9]+')

if [ -n "$TICKET" ]; then
  # Mark ticket as complete in roadmap
  python3 scripts/update_roadmap.py --ticket "$TICKET" --commit $(git rev-parse HEAD)
fi
```

---

## Quality Gates

### Gate 1: Complexity (P0 - Blocking)
```bash
pmat analyze complexity \
  --max-cyclomatic 10 \
  --max-cognitive 15 \
  --fail-on-violation
```

**Failures**: Extract functions, simplify logic, use strategy pattern

### Gate 2: SATD (P0 - Blocking)
```bash
pmat analyze satd --strict --fail-on-violation
```

**Failures**: Create tickets, remove comments, justify exceptions

### Gate 3: Coverage (P0 - Blocking)
```bash
cargo llvm-cov --fail-under-lines 80 --fail-under-branches 90
```

**Failures**: Add tests, use property testing for edge cases

### Gate 4: Mutation Score (P1 - Warning)
```bash
pmat analyze mutate --min-score 85 --warn-only
```

**Failures**: Add property tests, improve assertions, test edge cases

### Gate 5: Lint/Format (P0 - Blocking)
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

**Failures**: Run `cargo fmt`, fix clippy suggestions

### Gate 6: Entropy (P1 - Warning)
```bash
pmat analyze entropy --threshold 8.0 --warn-only
```

**Failures**: Refactor duplicated code, extract common patterns

---

## Implementation Architecture

### Module Structure
```
server/src/
├── scaffold/
│   ├── mod.rs (pub struct ScaffoldEngine)
│   ├── agent.rs (pforge-based agent scaffolding)
│   ├── wasm.rs (wasm-labs-based WASM scaffolding)
│   ├── templates/ (template files)
│   └── tests.rs (unit + integration tests)
│
├── maintain/
│   ├── mod.rs (pub struct MaintainEngine)
│   ├── roadmap.rs (roadmap parsing and validation)
│   ├── tickets.rs (ticket management)
│   ├── quality_gates.rs (quality gate execution)
│   └── tests.rs (unit + integration tests)
│
└── cli/
    └── handlers/
        ├── scaffold_handlers.rs (CLI commands)
        └── maintain_handlers.rs (CLI commands)
```

### Core Types

```rust
/// Scaffolding configuration
pub struct ScaffoldConfig {
    pub project_name: String,
    pub template: Template,
    pub features: Vec<Feature>,
    pub quality_gates: QualityGateConfig,
}

pub enum Template {
    Agent { based_on: AgentFramework },
    Wasm { based_on: WasmFramework },
    Library,
    Custom { path: PathBuf },
}

pub enum AgentFramework {
    Pforge,  // Reference: ../pforge
}

pub enum WasmFramework {
    WasmLabs,  // Reference: ../wasm-labs
    PureWasm,
}

/// Roadmap structure
pub struct Roadmap {
    pub current_status: String,
    pub active_sprint: Sprint,
    pub planned_sprints: Vec<Sprint>,
    pub completed_releases: Vec<Release>,
}

pub struct Sprint {
    pub name: String,
    pub status: SprintStatus,
    pub focus: String,
    pub tickets: Vec<TicketRef>,
}

pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: TicketStatus,
    pub priority: Priority,
    pub complexity: u8,  // 1-10
    pub estimated_time: Duration,
    pub dependencies: Vec<String>,
    pub test_strategy: TestStrategy,
    pub quality_gates: Vec<QualityGate>,
}

pub enum QualityGate {
    Complexity { max_cyclomatic: u8, max_cognitive: u8 },
    Coverage { min_line: f32, min_branch: f32 },
    Satd { strict: bool },
    MutationScore { min_score: f32 },
    NoLintFailures,
    NoTestFailures,
}
```

### Scaffolding Algorithm

```rust
impl ScaffoldEngine {
    /// Scaffold a new project
    pub fn scaffold(&self, config: ScaffoldConfig) -> Result<PathBuf> {
        // 1. Validate configuration
        self.validate_config(&config)?;

        // 2. Create project directory
        let project_dir = self.create_directory(&config.project_name)?;

        // 3. Generate from template
        match config.template {
            Template::Agent { based_on: AgentFramework::Pforge } => {
                self.scaffold_pforge_agent(&project_dir, &config)?;
            }
            Template::Wasm { based_on: WasmFramework::WasmLabs } => {
                self.scaffold_wasm_labs(&project_dir, &config)?;
            }
            _ => unimplemented!(),
        }

        // 4. Initialize git repository
        self.init_git(&project_dir)?;

        // 5. Install pre-commit hooks
        self.install_hooks(&project_dir, &config.quality_gates)?;

        // 6. Generate initial roadmap
        self.generate_roadmap(&project_dir, &config)?;

        // 7. Create initial tickets
        self.create_initial_tickets(&project_dir, &config)?;

        // 8. Run initial quality check
        self.verify_scaffold(&project_dir)?;

        Ok(project_dir)
    }
}
```

---

## Testing Strategy

### Unit Tests (per function)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_creates_directory() {
        let config = ScaffoldConfig {
            project_name: "test-project".into(),
            template: Template::Agent { based_on: AgentFramework::Pforge },
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        let engine = ScaffoldEngine::new();
        let result = engine.scaffold(config);

        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }
}
```

### Property Tests (invariants)
```rust
#[test]
fn prop_scaffold_always_creates_valid_structure() {
    proptest!(|(name: String, template: Template)| {
        let config = ScaffoldConfig {
            project_name: name.clone(),
            template,
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        let engine = ScaffoldEngine::new();
        let project_dir = engine.scaffold(config).unwrap();

        // Property: Always creates Cargo.toml
        assert!(project_dir.join("Cargo.toml").exists());

        // Property: Always creates ROADMAP.md
        assert!(project_dir.join("ROADMAP.md").exists() ||
                project_dir.join("roadmap.md").exists());

        // Property: Always installs pre-commit hook
        assert!(project_dir.join(".git/hooks/pre-commit").exists());
    });
}
```

### Integration Tests (end-to-end)
```rust
#[test]
fn integration_scaffold_agent_and_build() {
    let temp_dir = tempdir().unwrap();

    // Scaffold agent project
    let config = ScaffoldConfig {
        project_name: "test-agent".into(),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![Feature::Logging, Feature::Metrics],
        quality_gates: QualityGateConfig::extreme_tdd(),
    };

    let engine = ScaffoldEngine::new();
    let project_dir = engine.scaffold(config).unwrap();

    // Verify structure
    assert!(project_dir.join("pforge.yaml").exists());
    assert!(project_dir.join("src/handlers").exists());

    // Build project
    let output = Command::new("cargo")
        .current_dir(&project_dir)
        .args(&["build", "--all-features"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Run tests
    let output = Command::new("cargo")
        .current_dir(&project_dir)
        .args(&["test", "--all-features"])
        .output()
        .unwrap();

    assert!(output.status.success());
}
```

### Mutation Tests (test quality)
```rust
// Verify that tests catch mutants
#[test]
fn mutation_test_scaffold_validation() {
    // Test catches mutants in validation logic
    // Run with: pmat analyze mutate --path src/scaffold/mod.rs
    // Expected: >90% mutation score
}
```

---

## Success Metrics

### Scaffolding Success Metrics

1. **Time to First Build**: <5 minutes from `pmat scaffold` to `cargo build` success
2. **Time to First Test**: <10 minutes from `pmat scaffold` to passing tests
3. **Quality Gate Pass Rate**: 100% on freshly scaffolded projects
4. **Developer Satisfaction**: Survey score >4.5/5

### Maintenance Success Metrics

1. **Roadmap Accuracy**: >90% of planned work tracked in tickets
2. **Quality Gate Pass Rate**: >95% of commits pass pre-commit hooks
3. **Complexity Trend**: Average complexity decreases over time
4. **Coverage Trend**: Coverage increases or stable >80%
5. **SATD Trend**: SATD count decreases over time (goal: 0)
6. **Mutation Score Trend**: Mutation score increases or stable >85%

### Project Health Score

Composite metric (0-100):
```
Health Score = (
  0.3 * (100 - avg_complexity/20 * 100) +      // Complexity (inverted)
  0.2 * coverage_percentage +                   // Coverage
  0.2 * mutation_score +                        // Mutation score
  0.1 * (100 - satd_count/50 * 100) +          // SATD (inverted)
  0.1 * (tests_passing_rate * 100) +           // Test success
  0.1 * (roadmap_accuracy * 100)               // Roadmap tracking
)
```

**Thresholds**:
- **Excellent**: >90
- **Good**: 80-90
- **Acceptable**: 70-80
- **Needs Attention**: <70

---

## Dogfooding

This specification will be implemented using the extreme TDD methodology it describes:

1. **This spec becomes TICKET-PMAT-5000** (P0 priority)
2. **Roadmap updated** with new sprint for scaffold/maintain implementation
3. **Implementation follows RED → GREEN → REFACTOR** with property tests
4. **All quality gates enforced** during implementation
5. **Success verified** by scaffolding a new test project and maintaining PMAT itself

---

## References

### Internal
- `../pforge` - Agent scaffolding reference implementation
- `../wasm-labs` - WASM project reference implementation
- `CLAUDE.md` - Project-specific instructions
- `ROADMAP.md` - Current PMAT roadmap

### External
- [Toyota Way (Genchi Genbutsu)](https://en.wikipedia.org/wiki/Toyota_Way)
- [Extreme Programming (XP)](http://www.extremeprogramming.org/)
- [Property-Based Testing](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Mutation Testing](https://en.wikipedia.org/wiki/Mutation_testing)

---

**Status**: Ready for Implementation
**Next Step**: Add to ROADMAP.md as next sprint series
**Implementation Start**: 2025-10-05
