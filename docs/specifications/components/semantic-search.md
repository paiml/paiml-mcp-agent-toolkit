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
    pagerank REAL,
    -- Contract verification (populated at index build time)
    contract_level TEXT,     -- 'L0'..'L5' or NULL
    contract_equation TEXT,  -- equation name from #[contract] attr
    contract_yaml TEXT       -- YAML file name
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

### Browse Mode (empty query)

When query is empty, results are sorted by PageRank for enrichment-only flags
(e.g., `pmat query --churn --limit 10`). All QueryOptions filters
(`--exclude-file`, `--exclude`, `--language`, etc.) are applied via
`passes_filters()` — fixed in v3.11.1 (previously bypassed).

### Coverage Gaps (`--coverage-gaps`)

```bash
pmat query --coverage-gaps --rank-by impact --limit 20
```

Impact score: `missed_lines * pagerank / complexity`

Test fixture directories (`comprehensive_language_test/`, `fixtures/`, `testdata/`)
are excluded from coverage-gaps results to prevent non-project files from ranking
above actual source code.

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

### `--contracts` (Planned)

Aprender-contracts verification enrichment. Surfaces contract metadata
alongside TDG grade for every function — O(1) from pre-built index.

```bash
pmat query "score" --contracts --limit 5
# Output per result:
#   src/scoring.rs:42  score_range  TDG:A  Contract:L3  Eq:score_range
#   src/scoring.rs:88  calculate    TDG:A  Contract:L2  Eq:check_compliance
#   src/scoring.rs:120 normalize    TDG:B  Contract:—   (no contract)
```

**Data model**: Three new fields on `FunctionEntry` / `QualityMetrics`,
populated at index build time (O(1) query, no runtime contract scan):

```rust
// In QualityMetrics
pub contract_level: Option<String>,  // "L0".."L5" or None
pub contract_equation: Option<String>,  // "score_range", "check_compliance"
pub contract_yaml: Option<String>,  // "pmat-core.yaml"
```

**Index build**: During `analyze_project_with_cache()`, scan each function's
preceding attributes for `#[provable_contracts_macros::contract("yaml", equation = "eq")]`.
Extract yaml name and equation. Look up verification level from
`contracts/binding.yaml` if present. Store in SQLite `functions` table.

```sql
ALTER TABLE functions ADD COLUMN contract_level TEXT;
ALTER TABLE functions ADD COLUMN contract_equation TEXT;
```

**Query display**: When `--contracts` flag is set, append contract info
to each result line. Color-coded: L4-L5 green, L2-L3 yellow, L0-L1 red,
no contract gray.

**O(1) guarantee**: Contract data is pre-indexed at build time, stored
in SQLite, and read with the function record. No YAML parsing or
filesystem scanning at query time.

**Coverage metric**: `--contract-gaps` shows functions WITHOUT contract
annotations, ranked by PageRank (highest-impact uncovered functions first).
Analogous to `--coverage-gaps` for test coverage.

```bash
pmat query --contract-gaps --limit 10
# Functions with no #[contract] annotation, ranked by importance
```

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

## Contract Enrichment Architecture

### Why O(1)

Contract data (YAML name, equation, verification level) is extracted once
at index build time by scanning `#[provable_contracts_macros::contract(...)]`
attributes in the AST. The data is stored in the SQLite `functions` table
alongside TDG grade and complexity. At query time, it's read with the
function record — zero additional I/O.

### Build-Time Pipeline

```
source.rs → AST parse → extract #[contract] attrs → lookup binding.yaml
    → (yaml_name, equation, level) → store in functions table
```

The contract attribute parser uses the same AST pass that computes
complexity and TDG — no additional file reads. For projects without
`contracts/binding.yaml`, the level defaults to the annotation-implied
level (L2 for `#[contract]` without Lean proof, L4 with `lean_theorem`).

### Display Integration

Contract grade displayed inline with TDG in all output modes:

| Mode | Example |
|------|---------|
| Default | `fn score_range  TDG:A  PV:L3` |
| `--contracts` | `fn score_range  TDG:A  PV:L3  Eq:score_range  YAML:pmat-core` |
| `--contract-gaps` | `fn normalize  TDG:B  PV:—  (no contract, PageRank: 0.023)` |
| JSON | `"contract": {"level": "L3", "equation": "score_range"}` |

### Relationship to TDG

TDG measures code quality (complexity, churn, test coverage).
Contract level measures **verification depth** (what's been proven):
- L0: No contract
- L1: YAML spec exists
- L2: `#[contract]` annotation (runtime debug_assert)
- L3: Property tests (proptest/quickcheck)
- L4: Bounded model checking (Kani)
- L5: Full theorem proving (Lean)

A function can have TDG:A (high quality code) but PV:L0 (unverified),
or TDG:C (complex) but PV:L4 (formally verified despite complexity).

## Workspace Indexing Performance

### Current Bottleneck

| Operation | Time | Size | Issue |
|-----------|------|------|-------|
| Index aprender (78K fn) | ~86s | 346 MB | Full re-index on every query |
| Workspace merge (115K fn) | ~90s | 608 MB | Re-merges on every query |
| Query execution | <5ms | — | Fast (FTS5 BM25) |
| Total first-query | ~180s | — | Dominated by index I/O |

The query itself is fast (5ms BM25 lookup). The bottleneck is
**loading + merging** the workspace index on every invocation.

### Planned Improvements

**1. Incremental Workspace Index** (O(1) staleness check)

Check workspace index mtime vs member Cargo.toml mtimes. If no member
changed, skip rebuild. Current: always rebuilds workspace SQLite.

```
if workspace.db.mtime > max(member.Cargo.toml.mtime for member in members):
    load workspace.db directly  # O(1) — ~150ms
else:
    rebuild workspace.db        # O(n) — ~90s
```

**2. Lazy Member Loading**

Don't merge all sibling projects into workspace by default. Only merge
when `--workspace` or `--cross-project` flag is used. Default: local
project only (20K functions, ~150ms load).

**3. Parallel Index Build**

Build per-member SQLite indexes in parallel (rayon). Currently sequential.
With 4 sibling projects: 4x speedup on merge phase.

**4. Memory-Mapped SQLite**

Use `mmap_size` pragma for large indexes (>100MB). Avoids reading
entire DB into memory. Benchmark: 608MB workspace.db → ~50ms mmap
vs ~2s full read.

```sql
PRAGMA mmap_size = 1073741824;  -- 1GB mmap window
```

**5. Tiered Index Architecture**

```
.pmat/context.db        # Local project (20K fn, 70MB) — always loaded
.pmat/workspace.db      # Workspace merge (115K fn, 608MB) — lazy
.pmat/fleet.db          # Multi-repo fleet (future) — on-demand
```

### Target Performance

| Operation | Current | Target | Improvement |
|-----------|---------|--------|-------------|
| Cold query (local) | 86s | <3s | Incremental check |
| Cold query (workspace) | 180s | <10s | Lazy + mmap |
| Warm query (cached) | <1s | <500ms | Already fast |
| Index rebuild (incremental) | 86s | <5s | Mtime-based skip |

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
