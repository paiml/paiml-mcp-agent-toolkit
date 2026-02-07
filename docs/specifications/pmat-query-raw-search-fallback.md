# pmat query: Raw Search Fallback (PMAT-480)

## Problem

`pmat query` currently searches over a pre-built AST-extracted function index. This makes it excellent for semantic code search but leaves 5 critical gaps that prevent it from replacing `grep`/`rg` for AI coding agents (Claude Code, Cline, etc.):

1. **Non-code files invisible** — Cargo.toml, Makefile, YAML, JSON, Markdown, .env are unsearchable
2. **Module-level items not indexed** — `use`, `const`, `static`, `impl` blocks, macro calls, feature flags
3. **No line-level results** — returns entire functions, not specific lines
4. **Index-only search** — `--regex`/`--literal` only search indexed items, not raw files
5. **Index staleness** — edits not visible until `--rebuild-index`

## Solution: Raw Search Fallback Mode

When `--regex` or `--literal` is used, add a **raw file search** pass that searches file contents directly (like `rg`), then merges results with the indexed search. This makes `pmat query` a **superset of rg** rather than a parallel tool.

### Architecture

```text
pmat query "pattern" --regex
    │
    ├─► [1] Index Search (existing)
    │     AST-extracted functions matching pattern
    │     Returns: function-level results with quality metrics
    │
    ├─► [2] Raw Search (NEW)
    │     rg-style line search across ALL files
    │     Returns: file:line results (like grep)
    │     Respects .gitignore, .pmatignore
    │
    └─► [3] Merge & Deduplicate
          Index results get priority (have quality metrics)
          Raw results fill gaps (non-code files, non-indexed items)
          Dedup: if a raw match is inside an indexed function, prefer the indexed result
```

### Behavior Matrix

| Mode | Index Search | Raw Search | Output |
|------|-------------|------------|--------|
| Semantic (default) | Yes | No | Function-level with quality metrics |
| `--regex` | Yes | Yes | Merged: functions + raw lines |
| `--literal` | Yes | Yes | Merged: functions + raw lines |
| `--raw` (NEW) | No | Yes | Raw lines only (pure rg replacement) |

### New CLI Flag

```
--raw    Raw file search only, skip index (like rg, searches all file types)
```

### Raw Search Implementation

The raw search uses the `grep` crate (same engine as ripgrep) for:
- Regex and literal pattern matching
- Respecting `.gitignore` via the `ignore` crate
- Respecting `.pmatignore` for project-specific exclusions
- File type filtering via `--language` (maps to extensions)
- Binary file detection and skipping

### Output Format for Raw Results

Raw results use a new `RawSearchResult` struct:

```rust
pub struct RawSearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub context_before: Vec<String>,  // -B lines
    pub context_after: Vec<String>,   // -A lines
}
```

For `--format text` (default), raw results display as:
```
src/config.toml:15:  timeout = 30
src/config.toml:28:  max_retries = 3
```

For `--format json`, raw results include a `"type": "raw"` field to distinguish from indexed results.

### Merged Output

When both index and raw results exist (in `--regex`/`--literal` mode):

1. **Indexed results first** — with full quality metrics (TDG, complexity, coverage, etc.)
2. **Raw results second** — under a "Raw matches" section, for non-indexed matches
3. **Deduplication** — if a raw match falls within `start_line..end_line` of an indexed function, suppress the raw result (the indexed result is strictly better)

### File Discovery

Raw search walks the project tree using the `ignore` crate (same as rg):
- Respects `.gitignore` (already standard)
- Respects `.pmatignore` (pmat-specific exclusions)
- Skips binary files
- Skips `.git/`, `target/`, `node_modules/`, etc.
- Honors `--exclude-file` glob patterns
- Honors `--language` filter (maps to file extensions)

### Performance

- Raw search is O(files * lines), same as rg
- Index search is O(index_size), already fast
- Parallel: raw search runs concurrently with index search
- For `--raw` mode (no index), skip index loading entirely for instant startup

## Files to Modify

### 1. NEW: `src/services/agent_context/query/raw_search.rs`

Core raw search module:
- `RawSearchResult` struct
- `raw_search(pattern, project_path, options) -> Vec<RawSearchResult>`
- Uses `grep_regex` + `grep_searcher` + `ignore` crates for rg-compatible search
- `merge_results(indexed, raw) -> MergedResults` for deduplication

### 2. `src/services/agent_context/query/types.rs`

- Add `RawSearchResult` struct
- Add `MergedQueryResults` enum or wrapper

### 3. `src/services/agent_context/query/mod.rs`

- Add `mod raw_search;` and re-exports

### 4. `src/cli/commands/mod.rs`

- Add `--raw` flag to Query variant

### 5. `src/cli/command_dispatcher/mod.rs` + `command_structure.rs`

- Pass `raw` flag through to handler

### 6. `src/cli/handlers/query_handler.rs`

- When `--regex`/`--literal` + not `--raw`: run both index + raw search, merge
- When `--raw`: skip index, raw search only
- Format raw results in text/json/markdown output

### 7. `src/services/agent_context/query/formatters.rs`

- Add `format_raw_results()` for text/json/markdown output of raw matches
- Add merged output formatting

## Dependencies

Use existing crates already in the Rust ecosystem (check if already in Cargo.toml):
- `grep-regex` — regex engine (same as ripgrep)
- `grep-searcher` — file searching
- `ignore` — .gitignore-aware file walking

If not available, use `std::fs` + `regex` crate (already a dependency) + manual .gitignore parsing via the `ignore` crate.

**Check batuta stack first**: No batuta crate covers raw file search. This is infrastructure-level, external deps acceptable.

## Verification

1. `pmat query --regex "TODO" --raw` — finds TODOs in all files including TOML, MD, YAML
2. `pmat query --literal "OPENAI_API_KEY"` — finds in .env, Cargo.toml, AND indexed functions
3. `pmat query --regex "use serde" --files-with-matches` — finds import statements
4. `pmat query --regex "timeout" -A 2 -B 1` — shows context lines for raw matches
5. `pmat query --regex "fn main" --raw --format json` — pure rg replacement with JSON output
6. `pmat query "error handling" --coverage` — unchanged (semantic mode, no raw search)
