# KAIZEN-0013: Cross-phase AST cache to eliminate redundant syn::parse_file()

**Source paper:** arxiv:2506.15655 — "cAST: Enhancing Code Retrieval-Augmented Generation with Structural Chunking via AST" (2025)
**Category:** performance
**Priority:** medium
**Effort:** M

## Problem
PMAT project memory (MEMORY.md, "Memory Profiling" section) records: *"deep context path: 8.7 GB total allocs, 104 MB peak — dominated by syn::parse_file() across parallel analysis phases"*. We already know the fix: a cross-phase AST cache. The cAST paper validates AST reuse as a first-class primitive for code-serving systems.

## Proposed improvement
Add `AstCache` keyed by `(file_path, mtime, content_hash)` that:
1. Parses each Rust file at most once per `analyze_project` call (across AST / complexity / dead-code / duplicate / context phases).
2. Survives across contract-validation runs via memory-mapped cache at `.pmat/ast-cache/`.
3. Chunks on AST boundaries (cAST's split-then-merge) for downstream consumers.

## Impact
- Direct line to the known 8.7 GB allocation bottleneck. Target: 3–4x reduction.
- cAST paper reports substantial retrieval quality gains from AST-aligned chunks.

## Implementation sketch
1. `server/src/services/ast_cache.rs` — Arc<HashMap<AstKey, Arc<syn::File>>>.
2. Plumb through `analyze_project` phases; refactor each phase to accept `&dyn AstProvider`.
3. Optional disk-backed cache for cross-process reuse (bincode + mmap).
4. Benchmark script in `examples/bench_ast_cache.rs`.

## Acceptance criteria
- dhat allocations on `context` mode drop by ≥3x.
- All existing tests pass; no regression in analysis fidelity.
- Benchmark in CI compares before/after.
- **No MCP schema change.**
