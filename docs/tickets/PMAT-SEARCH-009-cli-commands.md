# PMAT-SEARCH-009: CLI Commands for Semantic Search

**Sprint**: 31
**Phase**: RED → GREEN → REFACTOR (EXTREME TDD)
**Estimate**: 3 hours
**Priority**: HIGH

## Objective

Implement command-line interface for semantic search operations: embedding synchronization, semantic querying, code clustering, and topic analysis.

## Background

With the semantic search engine, clustering, and topic modeling complete, we need user-facing CLI commands. This enables:
- **Developers**: Use semantic search from terminal/scripts
- **CI/CD**: Integrate into automated workflows
- **Scripts**: Batch processing of codebases
- **Interactive**: Explore codebase semantics

## Requirements

### Functional Requirements

1. **Embedding Commands** (`pmat embed`)
   - `pmat embed sync <directory>` - Synchronize embeddings for directory
   - `pmat embed status` - Show embedding statistics
   - `pmat embed clear` - Clear all embeddings

2. **Search Commands** (`pmat semantic`)
   - `pmat semantic search <query>` - Semantic search with options
   - `pmat semantic similar <file>` - Find similar code

3. **Analysis Commands** (`pmat analyze`)
   - `pmat analyze cluster` - Cluster code by similarity
   - `pmat analyze topics` - Extract semantic topics

### Non-Functional Requirements

- **User Experience**: Clear error messages, progress indicators
- **Performance**: Commands complete in <10 seconds for typical codebases
- **Robustness**: Handle missing files, invalid arguments gracefully
- **Testability**: 30 unit tests (RED phase)

## Technical Design

### Command Structure

```
pmat
├── embed
│   ├── sync <dir> [--language rust|python|typescript]
│   ├── status
│   └── clear [--confirm]
├── semantic
│   ├── search <query> [--mode hybrid|vector|keyword] [--limit 10] [--language rust]
│   └── similar <file> [--limit 5]
└── analyze
    ├── cluster [--method kmeans|hierarchical|dbscan] [--k 5]
    └── topics [--num-topics 10] [--language rust]
```

### Data Structures

```rust
// CLI argument structures
#[derive(Parser)]
#[command(name = "pmat")]
#[command(about = "Pragmatic AST & Semantic Search Toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Embed(EmbedCommand),
    Semantic(SemanticCommand),
    Analyze(AnalyzeCommand),
}

#[derive(Parser)]
pub struct EmbedCommand {
    #[command(subcommand)]
    pub action: EmbedAction,
}

#[derive(Subcommand)]
pub enum EmbedAction {
    Sync {
        directory: PathBuf,
        #[arg(long)]
        language: Option<String>,
    },
    Status,
    Clear {
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Parser)]
pub struct SemanticCommand {
    #[command(subcommand)]
    pub action: SemanticAction,
}

#[derive(Subcommand)]
pub enum SemanticAction {
    Search {
        query: String,
        #[arg(long, default_value = "hybrid")]
        mode: String,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        language: Option<String>,
    },
    Similar {
        file: PathBuf,
        #[arg(long, default_value = "5")]
        limit: usize,
    },
}

#[derive(Parser)]
pub struct AnalyzeCommand {
    #[command(subcommand)]
    pub action: AnalyzeAction,
}

#[derive(Subcommand)]
pub enum AnalyzeAction {
    Cluster {
        #[arg(long, default_value = "kmeans")]
        method: String,
        #[arg(long)]
        k: Option<usize>,
    },
    Topics {
        #[arg(long, default_value = "10")]
        num_topics: usize,
        #[arg(long)]
        language: Option<String>,
    },
}
```

### Handler Functions

```rust
impl Cli {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Embed(cmd) => handle_embed(cmd).await,
            Commands::Semantic(cmd) => handle_semantic(cmd).await,
            Commands::Analyze(cmd) => handle_analyze(cmd).await,
        }
    }
}

async fn handle_embed(cmd: &EmbedCommand) -> Result<(), Box<dyn std::error::Error>> {
    match &cmd.action {
        EmbedAction::Sync { directory, language } => {
            // 1. Initialize engines
            // 2. Index directory
            // 3. Show progress
            // 4. Print statistics
        }
        EmbedAction::Status => {
            // Query database for embedding counts
        }
        EmbedAction::Clear { confirm } => {
            // Clear embeddings with confirmation
        }
    }
}

async fn handle_semantic(cmd: &SemanticCommand) -> Result<(), Box<dyn std::error::Error>> {
    match &cmd.action {
        SemanticAction::Search { query, mode, limit, language } => {
            // Execute hybrid search
        }
        SemanticAction::Similar { file, limit } => {
            // Find similar code
        }
    }
}

async fn handle_analyze(cmd: &AnalyzeCommand) -> Result<(), Box<dyn std::error::Error>> {
    match &cmd.action {
        AnalyzeAction::Cluster { method, k } => {
            // Perform clustering
        }
        AnalyzeAction::Topics { num_topics, language } => {
            // Extract topics
        }
    }
}
```

## Test Plan (RED Phase - 30 tests)

### Embed Command Tests (10 tests)
1. `test_embed_sync_basic` - Sync directory successfully
2. `test_embed_sync_invalid_directory` - Error for non-existent directory
3. `test_embed_sync_with_language_filter` - Filter by language
4. `test_embed_status_empty` - Status with no embeddings
5. `test_embed_status_with_data` - Status shows counts
6. `test_embed_clear_requires_confirm` - Clear needs --confirm flag
7. `test_embed_clear_with_confirm` - Clear succeeds with flag
8. `test_embed_sync_progress` - Shows progress during sync
9. `test_embed_sync_incremental` - Only syncs changed files
10. `test_embed_sync_statistics` - Reports accurate statistics

### Semantic Command Tests (10 tests)
11. `test_semantic_search_basic` - Search returns results
12. `test_semantic_search_with_mode` - Respects mode parameter
13. `test_semantic_search_with_limit` - Respects limit parameter
14. `test_semantic_search_with_language` - Filters by language
15. `test_semantic_search_empty_query` - Error for empty query
16. `test_semantic_similar_basic` - Find similar code
17. `test_semantic_similar_invalid_file` - Error for missing file
18. `test_semantic_similar_limit` - Respects limit
19. `test_semantic_search_output_format` - Proper output formatting
20. `test_semantic_search_no_results` - Handles no results gracefully

### Analyze Command Tests (10 tests)
21. `test_analyze_cluster_kmeans` - K-means clustering works
22. `test_analyze_cluster_hierarchical` - Hierarchical clustering works
23. `test_analyze_cluster_dbscan` - DBSCAN clustering works
24. `test_analyze_cluster_requires_k` - K-means requires k parameter
25. `test_analyze_cluster_output_format` - Proper cluster output
26. `test_analyze_topics_basic` - Topic extraction works
27. `test_analyze_topics_with_language` - Filter by language
28. `test_analyze_topics_invalid_count` - Error for invalid topic count
29. `test_analyze_topics_output_format` - Proper topic output
30. `test_analyze_topics_empty_database` - Handles empty database

## Implementation Steps

### RED Phase (45 minutes)
1. Create `server/src/cli/semantic_cli.rs`
2. Write all 30 failing tests
3. Verify tests fail with clear error messages
4. Run: `cargo test cli_semantic -- --nocapture`

### GREEN Phase (90 minutes)
1. Implement CLI argument parsing with clap
2. Implement embed command handlers
3. Implement semantic command handlers
4. Implement analyze command handlers
5. Add progress indicators and output formatting
6. Run: `cargo test` - all tests pass

### REFACTOR Phase (45 minutes)
1. Extract common formatting functions
2. Add colored output with termcolor/colored
3. Improve error messages
4. Add bash completion generation
5. Run: `cargo clippy` - zero warnings
6. Run: `cargo test` - all tests still pass

## Acceptance Criteria

- [ ] All 30 tests pass
- [ ] All commands work as specified
- [ ] Clear error messages for all failure cases
- [ ] Progress indicators for long operations
- [ ] Zero clippy warnings
- [ ] Code coverage ≥ 95%
- [ ] Cyclomatic complexity ≤ 10 per function

## Dependencies

- `clap` v4 for CLI parsing
- `SemanticSearchEngine`, `HybridSearchEngine` for search
- `ClusteringEngine` for clustering
- `TopicEngine` for topic modeling
- `colored` or `termcolor` for output formatting

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Complex argument parsing | Use clap derive macros |
| Long-running operations | Add progress bars with indicatif |
| Poor UX | Test with real users, iterate |
| Inconsistent output | Create formatting module |

## Future Enhancements

- JSON output mode for scripting
- Interactive TUI with ratatui
- Watch mode for continuous sync
- Export results to CSV/JSON
- Shell completion scripts

## References

- clap documentation: https://docs.rs/clap
- indicatif: https://docs.rs/indicatif
- colored: https://docs.rs/colored

---

**EXTREME TDD**: RED → GREEN → REFACTOR
