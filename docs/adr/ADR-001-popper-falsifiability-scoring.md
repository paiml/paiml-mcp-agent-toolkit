# ADR-001: Popper Falsifiability Scoring Architecture

**Status:** Accepted
**Date:** 2025-12-09
**Decision Makers:** @paiml/core

## Context

PMAT needed a scientific rigor assessment system that could evaluate whether software projects make testable, falsifiable claims. Existing quality metrics (coverage, complexity, linting) measure code properties but not the epistemological quality of project claims.

### Problem Statement

1. Projects often make vague claims ("fast", "robust", "scalable") that cannot be verified
2. No existing tool measures scientific rigor in software projects
3. Quality scores don't distinguish between projects with evidence vs. marketing claims

### Requirements

- R1: Score must reflect Popper's demarcation criterion (falsifiability)
- R2: Unfalsifiable projects must be distinguishable (gateway mechanism)
- R3: Must work with existing PMAT infrastructure (services, CLI, MCP)
- R4: Must support Cargo workspaces (code in members, docs at root)
- R5: Scoring must be deterministic and reproducible

## Decision

Implement a **100-point Popper Falsifiability Score** with:

### 1. Gateway Mechanism (Category A)

If Category A (Falsifiability & Testability) scores below 60%, the total score is automatically 0. This implements Popper's key insight: without falsifiable claims, no amount of other quality metrics matter.

```
Score = (Category_A >= 60%) ? Sum(Categories) : 0
```

### 2. Six Scoring Categories

| Category | Points | Focus |
|----------|--------|-------|
| A | 25 | Falsifiability & Testability (GATEWAY) |
| B | 25 | Reproducibility Infrastructure |
| C | 20 | Transparency & Openness |
| D | 15 | Statistical Rigor |
| E | 10 | Historical Integrity |
| F | 5/NA | ML/AI Reproducibility |

### 3. Workspace-Aware Scoring

```rust
// Check tests/benches in workspace members, docs/CI at root
pub fn get_code_paths(project_path: &Path) -> Vec<PathBuf> {
    let info = detect_workspace(project_path);
    info.members  // Returns all Cargo workspace members
}
```

### 4. Trait-Based Scorer Architecture

```rust
pub trait PopperScorer: Send + Sync {
    fn name(&self) -> &str;
    fn category_id(&self) -> char;
    fn max_points(&self) -> f64;
    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore>;
}
```

## Alternatives Considered

### Alternative 1: Extend repo-score

**Rejected.** repo-score focuses on repository health (commits, branches, issues). Falsifiability is orthogonal - a repo can be healthy but make unfalsifiable claims.

### Alternative 2: Single-score metric

**Rejected.** A single number loses the nuance of *why* a project fails. The category breakdown enables targeted improvements.

### Alternative 3: No gateway mechanism

**Rejected.** This would allow projects to score well by gaming non-falsifiability categories while making vague claims. The gateway enforces Popper's core principle.

## Consequences

### Positive

- **Scientific Rigor**: Enforces Popper's demarcation criterion
- **Actionable**: Category breakdown shows exactly what to improve
- **Workspace Support**: Works with monorepos and Cargo workspaces
- **Extensible**: New scorers can be added via the trait

### Negative

- **Harsh Grading**: Gateway can result in 0 score for otherwise good projects
- **Subjectivity**: "Falsifiable claim" detection uses heuristics
- **Maintenance**: 6 scorers must be kept in sync with best practices

### Risks

| Risk | Mitigation |
|------|------------|
| False positives (valid claims marked unfalsifiable) | Heuristics updated based on feedback |
| Gaming the system (adding fake benchmarks) | Score is advisory, not security-critical |
| Performance on large projects | Async scoring, file caching |

## Implementation

- **Location:** `server/src/services/popper_score/`
- **CLI:** `pmat popper-score [--verbose] [--format json|markdown|yaml]`
- **MCP Tool:** `popper_score` (tool ID: 19)
- **Tests:** 57 unit + integration tests

## References

- Popper, K. (1959). *The Logic of Scientific Discovery*
- [docs/specifications/popper-score-v1.1.md](../specifications/popper-score-v1.1.md)
- [Chapter 37: Popper Falsifiability Score](https://paiml.github.io/pmat-book/ch37-00-popper-score.html)
