# Index v2: SQLite + FTS5 Backend

**Version**: 1.0.0
**Status**: In Progress
**Issue**: [#159](https://github.com/paiml/paiml-mcp-agent-toolkit/issues/159)
**Author**: PAIML Team
**Created**: 2026-02-07

---

## Problem

`pmat query` produces a 58GB index for the depyler repo (230K functions, 2.2M LOC Rust). The monolithic LZ4+bincode blob architecture cannot scale:

| Metric | Expected | Actual |
|--------|----------|--------|
| Index size | <100MB | 58GB |
| Query latency | <100ms | 60s+ timeout |
| Function count | ~10K | 230,486 |
| Memory on load | <500MB | OOM |

**Root causes** (Toyota Five Whys):
1. O(n^2) call graph from common names (`new`, `clone` → 500M edges) — **fixed in Phase 0**
2. No IDF weighting — TF-only scoring treats "self" same as "dispatch_event"
3. O(n) substring scan per query — no inverted index
4. Monolithic blob — must deserialize entire index for any query
5. Test function pollution — 73% of indexed functions are tests

## Architecture: SQLite + FTS5

Replace the monolithic LZ4+bincode blob with SQLite + FTS5:

```
Current (v1.x):                     Target (v2.0):
┌──────────────┐                    ┌──────────────────────────┐
│ functions.lz4│ 58GB               │ context.db               │ <100MB
│  bincode blob│                    │  ├─ functions (table)     │
│  all-or-nothing                   │  ├─ functions_fts (FTS5)  │ BM25 built-in
│  O(n) scan   │                    │  ├─ call_graph (table)    │
│  no IDF      │                    │  ├─ graph_metrics (table) │
└──────────────┘                    │  └─ metadata (table)      │
                                    └──────────────────────────┘
```

**Why SQLite + FTS5:**

| Capability | LZ4 blob | SQLite + FTS5 |
|------------|----------|---------------|
| BM25 scoring | Manual TF-only | Built-in [1][2] |
| Inverted index | O(n) scan | O(1) per term |
| Partial loading | All-or-nothing | Query what you need |
| Incremental update | Rebuild entire index | INSERT/UPDATE rows |
| Stop words | None | Built-in tokenizer [3] |
| Memory | Load entire blob | SQLite paging/mmap |
| IDF | Not computed | Automatic in FTS5 rank |

## Phases

### Phase 0: Call Graph Fix (DONE)

**Ticket**: PMAT-159-P0
**Status**: Complete

- `is_generic_callee()` excludes 50+ common method names from call graph
- `is_test_chunk()` filters test functions at build time
- `corpus_lower` removed from serialization (lazy compute on load)
- `name_index` capped at 100 entries per name
- Index version bumped to 1.4.0

### Phase 1: SQLite Backend + FTS5 Search (DONE)

**Ticket**: PMAT-159-P1
**Status**: Complete

Dual-write `context.db` alongside `functions.lz4`. Query engine uses FTS5 BM25 when available, falls back to TF scan.

- `sqlite_backend.rs`: schema creation, insert, FTS5 BM25 search (11 tests)
- `save()` dual-writes blob + SQLite
- `load()` detects `context.db`, sets `db_path` on index
- `calculate_relevance_scores()` uses FTS5 when `db_path` available
- Standalone FTS5 (no content-sync) for simplicity
- Verified: 18K functions → 52MB, 90K workspace → 275MB

**Schema:**
```sql
-- Core function data
CREATE TABLE functions (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    function_name TEXT NOT NULL,
    signature TEXT NOT NULL,
    definition_type TEXT NOT NULL DEFAULT 'Function',
    doc_comment TEXT,
    source TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    language TEXT NOT NULL,
    checksum TEXT NOT NULL,
    -- Quality metrics (denormalized for speed)
    tdg_score REAL NOT NULL DEFAULT 0.0,
    tdg_grade TEXT NOT NULL DEFAULT 'A',
    complexity INTEGER NOT NULL DEFAULT 1,
    cognitive_complexity INTEGER NOT NULL DEFAULT 1,
    big_o TEXT NOT NULL DEFAULT 'O(1)',
    satd_count INTEGER NOT NULL DEFAULT 0,
    loc INTEGER NOT NULL DEFAULT 0,
    -- Cached annotations
    commit_count INTEGER NOT NULL DEFAULT 0,
    churn_score REAL NOT NULL DEFAULT 0.0,
    clone_count INTEGER NOT NULL DEFAULT 0,
    pattern_diversity REAL NOT NULL DEFAULT 0.0,
    fault_annotations TEXT NOT NULL DEFAULT '[]'
);

-- FTS5 virtual table for BM25 search [1][2] (standalone, not content-synced)
CREATE VIRTUAL TABLE functions_fts USING fts5(
    function_name,
    signature,
    doc_comment,
    file_path,
    identifiers,          -- extracted identifiers from source (not in functions table)
    tokenize='porter unicode61 remove_diacritics 2'  -- [3] stemming + unicode
);

-- Call graph edges
CREATE TABLE call_graph (
    caller_id INTEGER NOT NULL REFERENCES functions(id),
    callee_id INTEGER NOT NULL REFERENCES functions(id),
    PRIMARY KEY (caller_id, callee_id)
);

-- Graph metrics (PageRank, centrality)
CREATE TABLE graph_metrics (
    function_id INTEGER PRIMARY KEY REFERENCES functions(id),
    pagerank REAL NOT NULL DEFAULT 0.0,
    centrality REAL NOT NULL DEFAULT 0.0,
    in_degree INTEGER NOT NULL DEFAULT 0,
    out_degree INTEGER NOT NULL DEFAULT 0
);

-- Index metadata
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Performance indexes
CREATE INDEX idx_functions_file ON functions(file_path);
CREATE INDEX idx_functions_name ON functions(function_name);
CREATE INDEX idx_functions_lang ON functions(language);
CREATE INDEX idx_functions_grade ON functions(tdg_grade);
CREATE INDEX idx_call_graph_callee ON call_graph(callee_id);
```

**FTS5 BM25 Query:**
```sql
-- Semantic search with BM25 ranking (replaces O(n) substring scan)
SELECT f.*, functions_fts.rank AS bm25_score
FROM functions_fts
JOIN functions f ON f.id = functions_fts.rowid
WHERE functions_fts MATCH ?
ORDER BY functions_fts.rank
LIMIT ?;

-- With quality weighting
SELECT f.*,
    (functions_fts.rank * 0.7 + (1.0 - f.tdg_score / 10.0) * 0.3) AS score
FROM functions_fts
JOIN functions f ON f.id = functions_fts.rowid
WHERE functions_fts MATCH ?
ORDER BY score
LIMIT ?;
```

**Implementation files:**
- `src/services/agent_context/function_index/sqlite_backend.rs` — new SQLite backend
- `src/services/agent_context/function_index/build.rs` — dual save (blob + SQLite)
- `src/services/agent_context/function_index/mod.rs` — feature-gate SQLite path

### Phase 2: Migration + Deprecate Blob

**Ticket**: PMAT-159-P2

- `load()` prefers `context.db` over `functions.lz4`
- `save()` writes only SQLite (stop writing blob)
- Auto-migration: if only blob exists, convert to SQLite on first load
- Remove `corpus`, `corpus_lower` from `AgentContextIndex` (FTS5 handles search)
- Remove `calls`/`called_by` HashMaps (query `call_graph` table)

### Phase 3: Incremental + Performance

**Ticket**: PMAT-159-P3

- File-level incremental updates via `checksum` column
- WAL mode for concurrent read/write
- Prepared statement caching
- Benchmark: <100ms p95 query latency on 230K function index

## Peer-Reviewed Citations

[1] **Robertson, S., & Zaragoza, H. (2009).** "The Probabilistic Relevance Framework: BM25 and Beyond." *Foundations and Trends in Information Retrieval*, 3(4), 333-389.
- BM25 ranking function used by FTS5's `rank` column
- Parameters: k1=1.2, b=0.75 (FTS5 defaults)

[2] **Cormack, G. V., Clarke, C. L., & Buettcher, S. (2009).** "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." *SIGIR '09*, 758-759.
- RRF fusion for combining BM25 + PageRank scores in Phase 2

[3] **Porter, M. F. (1980).** "An Algorithm for Suffix Stripping." *Program*, 14(3), 130-137.
- Porter stemmer used by FTS5 tokenizer for morphological normalization
- `serialize`/`serializer`/`serialization` → same stem

[4] **Page, L., Brin, S., Motwani, R., & Winograd, T. (1999).** "The PageRank Citation Ranking: Bringing Order to the Web." *Stanford InfoLab Technical Report*.
- PageRank stored in `graph_metrics` table for importance ranking

[5] **Zobel, J., & Moffat, A. (2006).** "Inverted Files for Text Search Engines." *ACM Computing Surveys*, 38(2), Article 6.
- Inverted index theory underlying FTS5's posting lists
- O(1) per-term lookup replacing O(n) substring scan

[6] **Manning, C. D., Raghavan, P., & Schutze, H. (2008).** "Introduction to Information Retrieval." *Cambridge University Press*, Ch. 2 (Stop words), Ch. 6 (Scoring/IDF).
- IDF weighting automatically applied by FTS5 BM25
- Stop word elimination via tokenizer configuration

[7] **Hipp, D. R. (2020).** "SQLite FTS5 Extension." *sqlite.org/fts5.html*
- FTS5 architecture: shadow tables, segment B-trees, prefix indexes
- Content-sync mode (`content=functions`) avoids data duplication

## Performance Targets

| Metric | v1.x (blob) | v2.0 (SQLite) | Method |
|--------|-------------|---------------|--------|
| Index size (depyler) | 58GB | <100MB | Call graph fix + SQLite |
| Query latency | 60s+ | <100ms | FTS5 inverted index |
| Memory (load) | OOM | <50MB | SQLite paging |
| Incremental update | Full rebuild | O(changed files) | Checksum-based upsert |
| Build time (depyler) | >10min | <2min | Skip tests + efficient insert |

## Backward Compatibility

- v1.4.0 blob format still readable (auto-migrate to SQLite)
- `AgentContextIndex` public API unchanged
- `pmat query` flags unchanged
- Old `.pmat/context.idx/` directory migrated to `.pmat/context.db`

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-07 | Initial spec: Phase 0 complete, Phase 1-3 defined |
| 1.1.0 | 2026-02-07 | Phase 1 complete: SQLite backend, FTS5 BM25 search, dual-write, TF fallback |
