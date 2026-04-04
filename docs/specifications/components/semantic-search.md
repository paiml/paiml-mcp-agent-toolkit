# Semantic Search & Indexing

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 5

## Architecture

```
pmat query "error handling" --limit 10
    |
    v
[SQLite FTS5 BM25] --> ranked results
    |                      |
    v                      v
[TF-IDF semantic]    [enrichment flags]
    |                      |
    v                      v
[PageRank rerank]    [--churn, --duplicates, --entropy, --faults, -G]
```

## Index Backend: SQLite + FTS5

### Schema

```sql
CREATE TABLE functions (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER,
    signature TEXT,
    complexity INTEGER,
    tdg_grade TEXT,
    tdg_score REAL,
    pagerank REAL
);

CREATE VIRTUAL TABLE functions_fts USING fts5(
    name, signature, source,
    content='',  -- standalone (not content-synced)
    tokenize='porter unicode61'
);
```

### Performance

| Operation | SQLite | Legacy LZ4 |
|-----------|--------|-----------|
| Index load | ~150ms | ~1200ms |
| Semantic query | ~300ms | ~1400ms |
| File size (18K functions) | 52 MB | 47 MB |

### Lazy Loading

- `load_functions_lightweight()`: no source column, empty call graphs
- Source backfilled on-demand via `load_source_by_location()`
- Call graph queried on-demand via `get_calls()`/`get_called_by()`
- `load_all_source()` bulk-loads for regex/literal modes
- `ensure_call_graph()` eagerly loads for PTX flow and cross-project ranking

## Search Modes

### Semantic (default)

TF-IDF cosine similarity + BM25 fusion + PageRank reranking.

### Regex (`--regex`)

```bash
pmat query --regex "fn\s+handle_\w+" --limit 10
```

### Literal (`--literal`)

```bash
pmat query --literal "unwrap()" --limit 10
```

### Coverage Gaps (`--coverage-gaps`)

```bash
pmat query --coverage-gaps --rank-by impact --limit 20
```

Impact score: `missed_lines * pagerank / complexity`

## Enrichment Flags

### `-G` / `--git-history`

Fuses git commit history via Reciprocal Rank Fusion (RRF):
- TF-IDF embeddings on commit messages (128-dim vocabulary)
- SQLite in-memory DB for commit search
- Returns: commit hash, author, changed files
- Fixed: HashMap iteration order determinism (sort by document frequency)

### `--churn`

Git volatility metrics (90-day window):
- Commit count and churn score (0.0-1.0)
- Hot files (>50% churn) flagged

### `--duplicates`

Code clone detection via MinHash + LSH:
- Clone count and similarity score
- Identifies DRY violations

### `--entropy`

Pattern diversity metrics:
- Low (<30%) = repetitive boilerplate
- High (>80%) = unique code

### `--faults`

Batuta fault pattern annotations:
- `unwrap`, `panic`, `unsafe`, `todo!`, `expect`

### `--coverage`

LLVM line coverage enrichment:
- Per-function covered/total lines
- Coverage fault annotations: NO_COVERAGE, LOW_COVERAGE

## Git History Search

### CommitEmbedder

TF-IDF with 128-dim vocabulary. Critical fix: vocabulary term selection sorted
by document frequency descending for deterministic HashMap iteration.

### GitHistoryIndex

- `GitHistoryIndex::in_memory()` creates SQLite in-memory DB
- `insert_commits()` requires `&mut self` (transaction)
- `search()` requires `&self`
- `search_git_history_profiled()` returns `(results, profile, all_commits)`

## Cached Data

| Path | Format | Purpose |
|------|--------|---------|
| `.pmat/context.db` | SQLite | Function index (preferred) |
| `.pmat/context.idx` | LZ4 blob | Legacy function index |
| `.pmat/coverage-cache.json` | JSON | LLVM coverage data |
| `.pmat/workspace.db` | SQLite | Cross-project workspace index |

## Key Files

| File | Purpose |
|------|---------|
| `src/services/agent_context/function_index/sqlite_backend/` | SQLite + FTS5 backend (module) |
| `src/cli/handlers/query_handler/` | pmat query command handler (module) |
| `src/services/git_history/` | Git history search (module) |
| `src/services/git_history/commit_embedder.rs` | TF-IDF commit embeddings |

## References

- Consolidated from: semantic-search-pmat-mcp-vector-db, semantic-search-feature,
  index-v2-sqlite-fts5, git-commit-correlation-spec, git-history-rag-integration,
  falsify-rag, pmat-query-raw-search-fallback
