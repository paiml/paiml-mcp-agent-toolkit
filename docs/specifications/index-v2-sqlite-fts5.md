# Index v2: SQLite + FTS5 Backend

**Version**: 2.0.0
**Status**: Phase 3 In Progress (Phases 0-2 Complete)
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
2. No IDF weighting — TF-only scoring treats "self" same as "dispatch_event" — **fixed in Phase 1**
3. O(n) substring scan per query — no inverted index — **fixed in Phase 1**
4. Monolithic blob — must deserialize entire index for any query — **fixed in Phase 2**
5. Test function pollution — 73% of indexed functions are tests — **fixed in Phase 0**

## Architecture: SQLite + FTS5

Replace the monolithic LZ4+bincode blob with SQLite + FTS5:

```
Before (v1.x):                      After (v2.0):
┌──────────────┐                    ┌──────────────────────────┐
│ functions.lz4│ 58GB               │ context.db               │ 52MB (18K)
│  bincode blob│                    │  ├─ functions (table)     │ 252MB (90K)
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
| Inverted index | O(n) scan | O(1) per term [5] |
| Partial loading | All-or-nothing | Query what you need [7] |
| Incremental update | Rebuild entire index | INSERT/UPDATE rows |
| Stop words | None | Built-in tokenizer [3] |
| Memory | Load entire blob | SQLite paging/mmap [8] |
| IDF | Not computed | Automatic in FTS5 rank [6] |

## Phases

### Phase 0: Call Graph Fix (DONE)

**Ticket**: PMAT-159-P0
**Status**: Complete
**Commits**: `838f65f3`

- `is_generic_callee()` excludes 50+ common method names from call graph
- `is_test_chunk()` filters test functions at build time
- `corpus_lower` removed from serialization (lazy compute on load)
- `name_index` capped at 100 entries per name
- Index version bumped to 1.4.0

### Phase 1: SQLite Backend + FTS5 Search (DONE)

**Ticket**: PMAT-159-P1
**Status**: Complete
**Commits**: `4e736f0c`, `b140f289`

Dual-write `context.db` alongside `functions.lz4`. Query engine uses FTS5 BM25 when available, falls back to TF scan.

- `sqlite_backend.rs`: schema creation, insert, FTS5 BM25 search (11 tests)
- `save()` dual-writes blob + SQLite
- `load()` detects `context.db`, sets `db_path` on index
- `calculate_relevance_scores()` uses FTS5 when `db_path` available
- Standalone FTS5 (no content-sync) for simplicity [7]
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
) WITHOUT ROWID;

-- Graph metrics (PageRank, centrality) [4]
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
- `src/services/agent_context/function_index/sqlite_backend.rs` — SQLite backend (820 lines)
- `src/services/agent_context/function_index/build.rs` — save/load with SQLite
- `src/services/agent_context/function_index/mod.rs` — module declaration
- `src/services/agent_context/query/engine.rs` — FTS5 BM25 integration

### Phase 2: SQLite-First Load + Blob Deprecation (DONE)

**Ticket**: PMAT-159-P2
**Status**: Complete
**Commits**: `ae0b3422`

- `load()` prefers `context.db` over `functions.lz4` (try SQLite first, blob fallback)
- `load_from_sqlite()` reads all tables, rebuilds name_index + file_index in memory
- Auto-fallback: if SQLite load fails (stale schema, missing table), falls back to blob
- Sibling project indexes load via blob fallback (may not have SQLite yet)
- `AgentContextIndex` has `db_path: Option<PathBuf>` for downstream FTS5 queries

### Phase 3: Performance + Cleanup (IN PROGRESS)

**Ticket**: PMAT-159-P3
**Commits**: `97912f37`, `ced920cf`, `ed12056e`, `37be1e00`, `762530d3`

- [x] WAL mode for concurrent read/write (set in `open_db()` pragmas) [8]
- [x] Prepared statement caching (`prepare_cached()` used throughout)
- [x] Workspace cache freshness: `newest_index_mtime()` checks both `.db` and `.idx/manifest.json`
- [x] Stop writing blob format: `save()` writes only SQLite + manifest (no `functions.lz4`)
- [x] `discover_sibling_indexes()` checks for both `.pmat/context.db` and `.pmat/context.idx`
- [x] Skip corpus construction on SQLite load: `build_indices_without_corpus()` saves ~36MB
- [x] `find_similar()` builds corpus entry on-the-fly via `build_corpus_entry()`
- [x] `load_or_build_index()` accepts either `.db` or `.idx` paths
- [x] CB-130 compliance checks accept `.pmat/context.db` as valid index
- [~] File-level incremental SQLite updates: **deferred** — `build_incremental()` already
  handles file-level diffing via SHA256 checksums at the application layer. `try_incremental_update()`
  skips `save()` when no changes detected. True SQLite-level row upserts would require
  refactoring the ID mapping (array index → persistent rowid) across call graph, graph metrics,
  and FTS5 rowid references. ROI is low given 0.9s cached query.
- [ ] Benchmark: <100ms p95 query latency on depyler 230K function index (requires Mac SSH)

**Observed performance**:
- Local 18K functions: 0.58s query (SQLite load + FTS5 search)
- Workspace 90K functions: 0.9s cached (was 1.2s before corpus skip), 10.8s uncached
- FTS5 search itself: <10ms (dominant cost is SQLite I/O for 90K functions)
- Corpus skip: saves ~36MB allocation + ~300ms for 90K functions
- Disk: 18K → 52MB SQLite, 90K → 252MB SQLite (was 47MB blob + 52MB dual-write)

## Peer-Reviewed Citations

[1] **Robertson, S., & Zaragoza, H. (2009).** "The Probabilistic Relevance Framework: BM25 and Beyond." *Foundations and Trends in Information Retrieval*, 3(4), 333-389.
- BM25 ranking function used by FTS5's `rank` column
- Parameters: k1=1.2, b=0.75 (FTS5 defaults)
- Theoretical basis for IDF weighting that distinguishes rare terms from common ones

[2] **Cormack, G. V., Clarke, C. L., & Buettcher, S. (2009).** "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." *SIGIR '09*, 758-759.
- RRF fusion for combining BM25 + PageRank scores
- Used in `calculate_relevance_scores()` quality weighting

[3] **Porter, M. F. (1980).** "An Algorithm for Suffix Stripping." *Program*, 14(3), 130-137.
- Porter stemmer used by FTS5 tokenizer for morphological normalization
- `serialize`/`serializer`/`serialization` → same stem
- Reduces vocabulary size, improves recall without precision loss

[4] **Page, L., Brin, S., Motwani, R., & Winograd, T. (1999).** "The PageRank Citation Ranking: Bringing Order to the Web." *Stanford InfoLab Technical Report*.
- PageRank stored in `graph_metrics` table for importance ranking
- Functions called by many other important functions rank higher
- Applied to call graph edges in `compute_graph_metrics()`

[5] **Zobel, J., & Moffat, A. (2006).** "Inverted Files for Text Search Engines." *ACM Computing Surveys*, 38(2), Article 6.
- Inverted index theory underlying FTS5's posting lists
- O(1) per-term lookup replacing O(n) substring scan
- Segment B-tree structure for efficient merging

[6] **Manning, C. D., Raghavan, P., & Schutze, H. (2008).** "Introduction to Information Retrieval." *Cambridge University Press*, Ch. 2 (Stop words), Ch. 6 (Scoring/IDF).
- IDF weighting automatically applied by FTS5 BM25
- Stop word elimination via tokenizer configuration
- Document length normalization in BM25's `b` parameter

[7] **Hipp, D. R. (2020).** "SQLite FTS5 Extension." *sqlite.org/fts5.html*
- FTS5 architecture: shadow tables, segment B-trees, prefix indexes
- Standalone mode (not content-synced) chosen for simplicity — `identifiers` column not in `functions` table
- BM25 rank computation via `bm25()` auxiliary function

[8] **Hipp, D. R. (2010).** "Write-Ahead Logging." *sqlite.org/wal.html*
- WAL mode enables concurrent readers during writes
- `PRAGMA journal_mode = WAL` set in `open_db()`
- Memory-mapped I/O via `PRAGMA mmap_size` for large indexes

[9] **Salton, G., & Buckley, C. (1988).** "Term-Weighting Approaches in Automatic Text Retrieval." *Information Processing & Management*, 24(5), 513-523.
- TF-IDF foundation used in fallback `calculate_relevance_scores_tf()`
- Log-normalized term frequency: `tf = (1 + ln(count)) / doc_len_factor`

[10] **Broder, A. Z. (1997).** "On the Resemblance and Containment of Documents." *SEQUENCES '97*, 21-29.
- MinHash/LSH theory used by `detect_code_clones()` for duplicate detection
- Clone count stored in `clone_count` column for deduplication scoring

## Performance Targets

| Metric | v1.x (blob) | v2.0 (SQLite) | Actual | Method |
|--------|-------------|---------------|--------|--------|
| Index size (18K) | 47MB blob | <100MB | 52MB | SQLite |
| Index size (90K) | OOM/58GB | <300MB | 252MB | Call graph fix + SQLite |
| Query latency (cached) | 60s+ | <1s | 0.9s | FTS5 + corpus skip |
| FTS5 search | N/A | <10ms | <10ms | Inverted index |
| Memory (load) | OOM | <50MB | ~50MB | No corpus on SQLite path |
| Build time (18K) | N/A | <30s | ~15s | SQLite batch insert |

## Backward Compatibility

- v1.4.0 blob format still readable via `load_from_blob()` fallback
- `AgentContextIndex` public API unchanged
- `pmat query` flags unchanged
- `discover_sibling_indexes()` checks both `.pmat/context.db` and `.pmat/context.idx`
- CB-130 compliance accepts either format

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-07 | Initial spec: Phase 0 complete, Phase 1-3 defined |
| 1.1.0 | 2026-02-07 | Phase 1 complete: SQLite backend, FTS5 BM25 search, dual-write, TF fallback |
| 1.2.0 | 2026-02-07 | Phase 2 complete: SQLite-first load, blob fallback. Phase 3 started |
| 1.3.0 | 2026-02-07 | Phase 3: Stop writing blob, SQLite-only save, sibling discovery updated |
| 2.0.0 | 2026-02-07 | Major update: corpus skip, CB-130 acceptance, .db path discovery, perf numbers, citations [8-10] |
