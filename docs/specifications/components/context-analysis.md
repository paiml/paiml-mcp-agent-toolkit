# Context & Analysis

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 6

## Deep Context Analysis

### Architecture

`DeepContextAnalyzer::analyze_project()` orchestrates 9 parallel analysis phases:

```
Phase 1 (sequential): AST analysis
    |-> Populates DashMap caches
    |-> Builds Arc<ProjectContext>
    v
Phase 2 (parallel, 8 phases):
    ├── Complexity (reads from DashMap cache)
    ├── Churn (git log analysis, 3s timeout)
    ├── Duplicate detection (MinHash/LSH)
    ├── SATD detection (comment analysis)
    ├── Dead code (lightweight static analysis)
    ├── Provability (reuses Arc<ProjectContext>)
    ├── DAG construction (reuses Arc<ProjectContext>)
    └── Big-O analysis (pattern matching)
```

### Two-Phase Execution Model

AST runs first because:
1. Provability and DAG need `ProjectContext` — avoids parsing twice
2. Complexity needs file metrics — DashMap cache eliminates redundant I/O
3. Sequential AST + parallel remainder is faster than all-parallel with contention

### Memory Optimization

- `Arc<ProjectContext>` shared across Provability and DAG (saves ~2 GB)
- DashMap caches for complexity metrics (saves ~1.2 GB)
- Test file exclusion reduces parsing by ~475 MB
- In-place PageRank scoring (saves ~49 MB peak)

### File Discovery

```rust
ProjectFileDiscovery::new(path)
    .with_config(FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(10_000),
    })
    .discover_files()
```

### Test File Filtering

Skip `*_tests.rs`, `*_test.rs`, `test_*.rs`, `tests/*` from analysis:
- Reduces syn parsing allocations by ~475 MB
- Eliminates noise from complexity/provability/DAG phases

## RAG-Powered Context

### Agent Context Index

```
pmat context --output deep_context.md --format llm-optimized
```

Generates LLM-optimized project context including:
- File structure with categorization
- Function signatures with TDG grades
- Dependency graph summary
- Quality metrics overview

### ProjectContext Structure

```rust
struct ProjectContext {
    project_type: String,        // "rust", "python", etc.
    files: Vec<FileContext>,     // Per-file AST data
    summary: ProjectSummary,    // Aggregate counts
    graph: Option<ProjectContextGraph>,  // trueno-graph CSR
}
```

## Stack Visualization

Terminal-based diagnostics with ASCII graphics:
- Flame graph approximation for allocation hotspots
- Bar charts for per-phase timing
- Table formatting for metric summaries

## Key Files

| File | Purpose |
|------|---------|
| `src/services/deep_context/` | Deep context analyzer |
| `src/services/deep_context/analyzer_core/spawn.rs` | Phase orchestration |
| `src/services/deep_context/analysis_helpers.rs` | File discovery, test filtering |
| `src/services/deep_context/analysis_functions/` | Per-phase analysis functions |
| `src/services/context.rs` | ProjectContext AST parsing |
| `src/services/context_graph.rs` | trueno-graph integration |

## References

- Consolidated from: current-deep-context-design-profiling, improve-context,
  trueno-o1-context-tdg-integration, stack-visualization-diagnostics-reporting
