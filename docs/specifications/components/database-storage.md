# Database & Storage

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 8

## Storage Architecture

```
.pmat/
├── context.db          # SQLite + FTS5 (primary, 52 MB for 18K functions)
├── context.idx         # Legacy LZ4 blob (fallback)
├── coverage-cache.json # LLVM coverage data
├── workspace.db        # Cross-project workspace index (275 MB for 90K functions)
├── workspace.idx       # Legacy workspace blob
├── dead-code-cache.json
└── bug-hunter-cache/
    └── *.json          # Batuta findings by project hash

.pmat-metrics/
├── commit-<sha>-meta.json  # TDG/repo scores per commit
├── commit-<sha>-tests.json # Test pass/fail counts (EvoScore)
└── memory-baseline.json    # dhat-rs memory baseline
```

## SQLite + FTS5 Backend

### Dual-Write Pattern

On `save()`, writes both SQLite and LZ4 for backward compatibility.
On `load()`, prefers `context.db` over `functions.lz4`.

### FTS5 Standalone Mode

FTS5 table is NOT content-synced — `identifiers` column is not in `functions` table.
This avoids schema coupling but requires separate inserts.

### Prepared Statement Pattern

Wrap in block to drop borrow before transaction commit:
```rust
{
    let mut stmt = tx.prepare("INSERT INTO functions_fts VALUES (?1, ?2, ?3)")?;
    for func in &functions {
        stmt.execute(params![func.name, func.signature, func.source])?;
    }
} // stmt drops here, releasing borrow on tx
tx.commit()?;
```

## Trueno-DB Columnar Storage

### Integration

Trueno-DB provides:
- Columnar storage for analytics workloads
- SIMD-accelerated aggregations
- Compressed column encoding

### Schema

| Column | Type | Purpose |
|--------|------|---------|
| function_name | String | Function identifier |
| file_path | String | Source file |
| complexity | u32 | Cyclomatic complexity |
| tdg_score | f32 | Technical debt score |
| pagerank | f32 | Centrality score |
| coverage | f32 | Line coverage ratio |

## CSR Graph Database (trueno-graph)

### Architecture

Dual storage pattern:
- HashMap for O(1) lookups by node ID
- CSR (Compressed Sparse Row) for PageRank computation
- Bidirectional NodeId mapping

### Key Insight

CSR graphs only track nodes with edges. Always use `node_map.len()`
not `graph.num_nodes()` for total node count.

## Cache Invalidation

All caches keyed on git HEAD hash:
- Coverage cache: invalidated when HEAD changes
- Context index: rebuilt when source files change
- Metric files: timestamped per-commit

## Key Files

| File | Purpose |
|------|---------|
| `src/services/sqlite_backend.rs` | SQLite + FTS5 backend |
| `src/services/context_graph.rs` | trueno-graph CSR database |
| `src/services/cache.rs` | SessionCacheManager |

## References

- Consolidated from: trueno-db-integration, trueno-db-integration-v2,
  trueno-db-integration-review-response, trueno-integration-spec
