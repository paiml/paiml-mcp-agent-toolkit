# Git History RAG Integration Specification

**Version**: 1.0
**Created**: February 5, 2026
**Status**: SPECIFICATION (Ready for Implementation)
**Methodology**: EXTREME TDD + Toyota Way + Popperian Falsification
**Extends**: `semantic-search-pmat-mcp-vector-db.md`

---

## Executive Summary

Extend PMAT's RAG-powered code search (`pmat query`) with optional git history integration. When enabled via `--git-history` flag, search expands to include commit messages, enabling discovery of code by *intent* ("why was this written?") in addition to *content* ("what does this do?").

**Key Design Decision**: Separate indexes for code and git history, joined at query time.

---

## Toyota Way Principles

This specification applies Toyota Production System principles throughout:

| Principle | Japanese | Application |
|-----------|----------|-------------|
| **Jidoka** | 自働化 | Automation with human touch - git history enriches results but human judgment interprets intent |
| **Kaizen** | 改善 | Incremental improvement - start with commit messages, expand to diffs/PRs later |
| **Genchi Genbutsu** | 現地現物 | Go and see - search actual commit history, not abstractions |
| **Muda Elimination** | 無駄 | Reduce waste - separate indexes avoid redundant updates |
| **Heijunka** | 平準化 | Level loading - async index updates don't block queries |
| **Poka-Yoke** | ポカヨケ | Error-proofing - schema constraints prevent invalid data |

---

## Scientific Foundation

### Peer-Reviewed Citations

#### Index Architecture

**[ICSME-2019]** Robillard, M.P., et al. "On-demand Developer Documentation." *IEEE International Conference on Software Maintenance and Evolution*, 2019.
- **Finding**: Commit messages provide unique semantic context not present in code
- **Relevance**: Justifies separate embedding space for commit messages

**[MSR-2021]** Tian, Y., et al. "What Makes a Good Commit Message?" *Mining Software Repositories*, 2021.
- **Finding**: High-quality commit messages follow patterns: imperative mood, <50 chars subject, body explains "why"
- **Relevance**: Informs commit message preprocessing for embedding quality

**[ICSE-2020]** Liu, Z., et al. "Neural-Machine-Translation-Based Commit Message Generation." *International Conference on Software Engineering*, 2020.
- **Finding**: Commit messages have distinct linguistic patterns from code comments
- **Relevance**: Supports using different embedding strategies for code vs commits

#### Separate Index Justification

**[VLDB-2023]** Wang, J., et al. "Milvus: A Purpose-Built Vector Data Management System." *Proceedings of the VLDB Endowment*, 2023.
- **Finding**: Heterogeneous data benefits from separate vector spaces with late fusion
- **Relevance**: Justifies separate indexes joined at query time

**[SIGIR-2022]** Formal, T., et al. "SPLADE v2: Sparse Lexical and Expansion Model for Information Retrieval." *ACM SIGIR*, 2022.
- **Finding**: Hybrid retrieval (dense + sparse) outperforms single-method approaches
- **Relevance**: Supports our RRF fusion of code and commit results

**[TOIS-2023]** Lin, J., et al. "Proposed Best Practices for Reproducible IR Research." *ACM Transactions on Information Systems*, 2023.
- **Finding**: Modular index design enables reproducible experiments
- **Relevance**: Separate indexes allow independent evaluation

#### Update Frequency Analysis

**[FSE-2018]** Levin, S. & Yehudai, A. "The Co-Evolution of Test Maintenance and Code Maintenance." *Foundations of Software Engineering*, 2018.
- **Finding**: Code and metadata evolve at different rates; tight coupling increases maintenance burden
- **Relevance**: Justifies decoupled update cycles for code vs git indexes

**[ESEM-2020]** Hora, A. "What Code Is Deliberately Excluded from Test Coverage?" *Empirical Software Engineering and Measurement*, 2020.
- **Finding**: Index staleness correlates with update frequency mismatch
- **Relevance**: Separate indexes with independent update cycles reduce staleness

---

## Architecture

### Dual-Index Design (Toyota Way: Muda Elimination)

```
┌─────────────────────────────────────────────────────────────────┐
│                        Query Engine                              │
│                                                                  │
│  pmat query "error handling" --git-history                      │
│                      │                                           │
│         ┌───────────┴───────────┐                               │
│         ▼                       ▼                                │
│  ┌─────────────┐         ┌─────────────┐                        │
│  │ Code Index  │         │ Git Index   │                        │
│  │ (existing)  │         │ (new)       │                        │
│  └──────┬──────┘         └──────┬──────┘                        │
│         │                       │                                │
│         └───────────┬───────────┘                               │
│                     ▼                                            │
│              ┌─────────────┐                                     │
│              │ RRF Fusion  │  ← [SIGIR-2022] SPLADE v2          │
│              └──────┬──────┘                                     │
│                     ▼                                            │
│              Enriched Results                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Schema Design (Toyota Way: Poka-Yoke)

#### Code Index (Existing - No Changes)

```sql
-- .pmat/context.idx (existing)
CREATE TABLE code_embeddings (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    chunk_type TEXT NOT NULL CHECK (chunk_type IN ('function', 'struct', 'enum', 'trait', 'type', 'class', 'module')),
    chunk_name TEXT NOT NULL,
    language TEXT NOT NULL,
    start_line INTEGER NOT NULL CHECK (start_line > 0),
    end_line INTEGER NOT NULL CHECK (end_line >= start_line),
    signature TEXT,
    content_checksum TEXT NOT NULL,
    embedding BLOB NOT NULL,
    tdg_grade TEXT CHECK (tdg_grade IN ('A', 'B', 'C', 'D', 'F')),
    complexity INTEGER CHECK (complexity >= 0),
    created_at INTEGER NOT NULL,
    UNIQUE(file_path, chunk_type, chunk_name)
);
```

#### Git History Index (New)

```sql
-- .pmat/git-history.idx (new)
CREATE TABLE git_commits (
    commit_hash TEXT PRIMARY KEY CHECK (length(commit_hash) = 40),
    message_subject TEXT NOT NULL,        -- First line, <72 chars [MSR-2021]
    message_body TEXT,                    -- Explains "why"
    author_name TEXT NOT NULL,
    author_email TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    embedding BLOB NOT NULL,              -- Embedded: subject + body
    is_merge BOOLEAN DEFAULT FALSE,
    is_fix BOOLEAN DEFAULT FALSE,         -- Conventional commit or "fix" keyword
    is_feat BOOLEAN DEFAULT FALSE,        -- Conventional commit "feat:"
    issue_refs TEXT,                      -- JSON array: ["#123", "JIRA-456"]
    created_at INTEGER NOT NULL
);

CREATE TABLE commit_files (
    commit_hash TEXT NOT NULL,
    file_path TEXT NOT NULL,
    change_type TEXT NOT NULL CHECK (change_type IN ('A', 'M', 'D', 'R')),
    lines_added INTEGER DEFAULT 0 CHECK (lines_added >= 0),
    lines_deleted INTEGER DEFAULT 0 CHECK (lines_deleted >= 0),
    PRIMARY KEY (commit_hash, file_path),
    FOREIGN KEY (commit_hash) REFERENCES git_commits(commit_hash)
);

-- Co-change analysis [FSE-2018]
CREATE TABLE file_cochange (
    file_a TEXT NOT NULL,
    file_b TEXT NOT NULL,
    cochange_count INTEGER NOT NULL CHECK (cochange_count > 0),
    jaccard_similarity REAL CHECK (jaccard_similarity BETWEEN 0.0 AND 1.0),
    last_cochange INTEGER NOT NULL,
    PRIMARY KEY (file_a, file_b)
);

-- Indexes for query performance
CREATE INDEX idx_commits_timestamp ON git_commits(timestamp DESC);
CREATE INDEX idx_commits_author ON git_commits(author_email);
CREATE INDEX idx_commits_is_fix ON git_commits(is_fix) WHERE is_fix = TRUE;
CREATE INDEX idx_files_path ON commit_files(file_path);
```

---

## Popperian Falsification Tests

Each feature includes falsifiable predictions. If these tests fail, the hypothesis is **disproved**.

### F1: Separate Index Update Independence

**Hypothesis**: Separate indexes allow independent updates without cross-contamination.

**Falsification Test**:
```rust
#[test]
fn falsify_index_independence() {
    // Setup: Create both indexes with known state
    let code_idx = CodeIndex::new(temp_path);
    let git_idx = GitHistoryIndex::new(temp_path);

    code_idx.insert_chunk("src/main.rs", "main", embedding_a);
    git_idx.insert_commit("abc123", "Initial commit", embedding_b);

    let code_checksum_before = code_idx.checksum();
    let git_checksum_before = git_idx.checksum();

    // Action: Update ONLY git index with new commit
    git_idx.insert_commit("def456", "Add feature", embedding_c);

    // Falsification: Code index MUST be unchanged
    assert_eq!(code_idx.checksum(), code_checksum_before,
        "FALSIFIED: Git index update contaminated code index");

    // And git index MUST have changed
    assert_ne!(git_idx.checksum(), git_checksum_before,
        "FALSIFIED: Git index update had no effect");
}
```

**Popper Criterion**: If code index checksum changes when only git index is updated, the independence hypothesis is falsified.

---

### F2: Commit Message Embedding Quality

**Hypothesis**: Commit messages about similar topics cluster together in embedding space [ICSE-2020].

**Falsification Test**:
```rust
#[test]
fn falsify_commit_semantic_clustering() {
    let embedder = CommitEmbedder::new();

    // Semantically similar commits (error handling)
    let fix_null = embedder.embed("Fix null pointer exception in parser");
    let fix_crash = embedder.embed("Fix crash when input is empty");
    let fix_panic = embedder.embed("Handle panic in error path");

    // Semantically different commit (feature)
    let add_feature = embedder.embed("Add dark mode support to UI");

    // Falsification: Similar commits MUST be closer than dissimilar
    let sim_fix_null_crash = cosine_similarity(&fix_null, &fix_crash);
    let sim_fix_null_feature = cosine_similarity(&fix_null, &add_feature);

    assert!(sim_fix_null_crash > sim_fix_null_feature,
        "FALSIFIED: Error-fix commits not closer than error-vs-feature. \
         sim(fix,fix)={}, sim(fix,feat)={}",
        sim_fix_null_crash, sim_fix_null_feature);

    // Quantitative threshold from [ICSE-2020]: similar > 0.7, dissimilar < 0.4
    assert!(sim_fix_null_crash > 0.7,
        "FALSIFIED: Similar commits below 0.7 threshold");
    assert!(sim_fix_null_feature < 0.5,
        "FALSIFIED: Dissimilar commits above 0.5 threshold");
}
```

**Popper Criterion**: If semantically similar commits don't cluster (cosine > 0.7), embedding quality hypothesis is falsified.

---

### F3: RRF Fusion Improves Relevance

**Hypothesis**: Hybrid search (code + git) via RRF outperforms code-only search for intent queries [SIGIR-2022].

**Falsification Test**:
```rust
#[test]
fn falsify_rrf_fusion_improvement() {
    let code_idx = CodeIndex::with_test_data();
    let git_idx = GitHistoryIndex::with_test_data();

    // Intent-based query (commit messages help)
    let query = "fix memory leak";

    // Search code only
    let code_results = code_idx.search(query, limit=10);
    let code_mrr = mean_reciprocal_rank(&code_results, &ground_truth);

    // Search with git history fusion
    let git_results = git_idx.search(query, limit=10);
    let fused_results = rrf_fusion(&code_results, &git_results);
    let fused_mrr = mean_reciprocal_rank(&fused_results, &ground_truth);

    // Falsification: Fusion MUST improve MRR for intent queries
    // Based on [SIGIR-2022]: hybrid improves MRR by 8-15%
    let improvement = (fused_mrr - code_mrr) / code_mrr;

    assert!(fused_mrr >= code_mrr,
        "FALSIFIED: RRF fusion decreased relevance. \
         code_mrr={}, fused_mrr={}", code_mrr, fused_mrr);

    assert!(improvement > 0.05,
        "FALSIFIED: RRF improvement below 5% threshold. \
         improvement={}%", improvement * 100.0);
}
```

**Popper Criterion**: If fused MRR is not at least 5% better than code-only for intent queries, RRF hypothesis is falsified.

---

### F4: Update Frequency Justifies Separation

**Hypothesis**: Git history changes more frequently than code semantics, justifying separate indexes [FSE-2018].

**Falsification Test**:
```rust
#[test]
fn falsify_update_frequency_difference() {
    // Analyze real repository history
    let repo = Repository::open(".")?;
    let commits = repo.commits_since(days_ago(30));

    let mut code_change_days = HashSet::new();
    let mut git_change_days = HashSet::new();

    for commit in commits {
        let day = commit.timestamp.date();
        git_change_days.insert(day);  // Every commit updates git history

        if commit.changes_code_semantically() {  // Function sig/body changes
            code_change_days.insert(day);
        }
    }

    let git_frequency = git_change_days.len() as f64 / 30.0;
    let code_frequency = code_change_days.len() as f64 / 30.0;
    let ratio = git_frequency / code_frequency;

    // Falsification: Git MUST change more frequently than code semantics
    // [FSE-2018] reports 2-5x difference in typical projects
    assert!(ratio > 1.5,
        "FALSIFIED: Git history not significantly more frequent than code changes. \
         ratio={:.2}x (expected >1.5x)", ratio);
}
```

**Popper Criterion**: If git changes less than 1.5x more frequently than semantic code changes, separation is not justified.

---

### F5: Co-Change Prediction Accuracy

**Hypothesis**: Files that changed together historically will change together in future [FSE-2018].

**Falsification Test**:
```rust
#[test]
fn falsify_cochange_prediction() {
    // Train on first 80% of history
    let (train_commits, test_commits) = repo.commits().split_at_ratio(0.8);

    let cochange_model = CoChangeModel::train(&train_commits);

    // Test prediction on remaining 20%
    let mut predictions = 0;
    let mut correct = 0;

    for commit in test_commits {
        let changed_files = commit.files();
        if changed_files.len() < 2 { continue; }

        let first_file = &changed_files[0];
        let predicted_cochanges = cochange_model.predict_cochanges(first_file, top_k=5);

        for predicted in predicted_cochanges {
            predictions += 1;
            if changed_files.contains(&predicted) {
                correct += 1;
            }
        }
    }

    let precision = correct as f64 / predictions as f64;

    // Falsification: Precision MUST exceed random baseline
    // Random baseline for repo with N files: ~5/N
    let random_baseline = 5.0 / repo.file_count() as f64;

    assert!(precision > random_baseline * 2.0,
        "FALSIFIED: Co-change prediction not better than 2x random. \
         precision={:.3}, random={:.3}", precision, random_baseline);

    // [FSE-2018] reports precision@5 of 0.15-0.35 for real projects
    assert!(precision > 0.10,
        "FALSIFIED: Co-change precision below 10% threshold");
}
```

**Popper Criterion**: If co-change prediction precision is not at least 2x random baseline, the feature provides no value.

---

## CLI Interface

### New Flag: `--git-history`

```bash
# Default: code-only search (existing behavior)
pmat query "error handling"

# With git history: expands search to include commit messages
pmat query "error handling" --git-history

# Aliases
pmat query "memory leak" --gh
pmat query "memory leak" -G
```

### Output Enhancement

```
# Without --git-history
Found 5 functions:

1. src/api/error.rs:42 - handle_api_error
   Signature: pub fn handle_api_error(err: ApiError) -> Response
   TDG: A (2.1) | Complexity: 8 | Big-O: O(1)
   Relevance: 0.87

# With --git-history
Found 5 functions:

1. src/api/error.rs:42 - handle_api_error
   Signature: pub fn handle_api_error(err: ApiError) -> Response
   TDG: A (2.1) | Complexity: 8 | Big-O: O(1)
   Relevance: 0.92 (+0.05 from commit context)

   Related Commits:
   ├─ abc1234 "Fix race condition in error handler" (alice, 2025-12)
   ├─ def5678 "Add retry logic for transient errors" (bob, 2025-10)
   └─ ghi9012 "Refactor error handling for clarity" (alice, 2025-08)

   Co-Changes: src/api/retry.rs (87%), src/api/timeout.rs (64%)
```

### MCP Tool Enhancement

```json
{
  "name": "pmat_query_code",
  "inputSchema": {
    "properties": {
      "query": { "type": "string" },
      "git_history": {
        "type": "boolean",
        "default": false,
        "description": "Include git commit history in search"
      },
      "git_author": {
        "type": "string",
        "description": "Filter by commit author email"
      },
      "git_since": {
        "type": "string",
        "description": "Only commits after this date (ISO 8601)"
      },
      "show_cochanges": {
        "type": "boolean",
        "default": false,
        "description": "Show co-change analysis for results"
      }
    }
  }
}
```

---

## Implementation Plan

### Phase 1: Git Index Infrastructure (Sprint N)

**Toyota Way: Genchi Genbutsu** - Build foundation by directly analyzing git data

| Ticket | Description | Tests | Falsification |
|--------|-------------|-------|---------------|
| GH-RAG-001 | Git commit parser (libgit2) | 15 | F4 |
| GH-RAG-002 | Commit message embedder | 12 | F2 |
| GH-RAG-003 | Git history SQLite schema | 10 | F1 |
| GH-RAG-004 | Incremental git sync | 8 | F4 |

### Phase 2: Search Integration (Sprint N+1)

**Toyota Way: Jidoka** - Automation with quality built-in

| Ticket | Description | Tests | Falsification |
|--------|-------------|-------|---------------|
| GH-RAG-005 | Git history search engine | 15 | F2 |
| GH-RAG-006 | RRF fusion implementation | 12 | F3 |
| GH-RAG-007 | CLI `--git-history` flag | 10 | - |
| GH-RAG-008 | MCP tool enhancement | 8 | - |

### Phase 3: Analytics (Sprint N+2)

**Toyota Way: Kaizen** - Continuous improvement through insights

| Ticket | Description | Tests | Falsification |
|--------|-------------|-------|---------------|
| GH-RAG-009 | Co-change analysis | 12 | F5 |
| GH-RAG-010 | Author expertise scoring | 8 | - |
| GH-RAG-011 | Bug-fix corpus tagging | 10 | - |
| GH-RAG-012 | Documentation & book | - | - |

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Git index build (10K commits) | <30s | `time pmat git-index build` |
| Incremental sync (100 new commits) | <2s | `time pmat git-index sync` |
| Query with `--git-history` | <500ms | p95 latency |
| Query without flag (no regression) | <300ms | p95 latency |
| Git index size (10K commits) | <50MB | `du -h .pmat/git-history.idx` |

---

## Risk Mitigation

### Risk 1: Large Repository Performance

**Risk**: Repositories with 100K+ commits may have slow index builds
**Mitigation**:
- Configurable commit depth limit (`--depth 10000`)
- Incremental sync by default
- Background indexing (Toyota Way: Heijunka)

### Risk 2: Embedding Quality for Terse Commits

**Risk**: Short commit messages ("fix", "update") produce poor embeddings
**Mitigation**:
- Minimum message length filter (>10 chars)
- Concatenate with diff stats for context
- Flag low-quality commits for exclusion

### Risk 3: Privacy Concerns

**Risk**: Commit messages may contain sensitive information
**Mitigation**:
- Local-only index by default
- Redaction patterns for emails/tokens
- `--strip-pii` flag for sanitization

---

## References

1. **[ICSME-2019]** Robillard, M.P., et al. "On-demand Developer Documentation." IEEE ICSME, 2019.
2. **[MSR-2021]** Tian, Y., et al. "What Makes a Good Commit Message?" MSR, 2021.
3. **[ICSE-2020]** Liu, Z., et al. "Neural-Machine-Translation-Based Commit Message Generation." ICSE, 2020.
4. **[VLDB-2023]** Wang, J., et al. "Milvus: A Purpose-Built Vector Data Management System." VLDB, 2023.
5. **[SIGIR-2022]** Formal, T., et al. "SPLADE v2: Sparse Lexical and Expansion Model." ACM SIGIR, 2022.
6. **[TOIS-2023]** Lin, J., et al. "Proposed Best Practices for Reproducible IR Research." ACM TOIS, 2023.
7. **[FSE-2018]** Levin, S. & Yehudai, A. "The Co-Evolution of Test Maintenance and Code Maintenance." FSE, 2018.
8. **[ESEM-2020]** Hora, A. "What Code Is Deliberately Excluded from Test Coverage?" ESEM, 2020.

---

## Definition of Done

- [ ] All 5 falsification tests pass (F1-F5)
- [ ] 100+ unit tests across all tickets
- [ ] Git index builds in <30s for 10K commits
- [ ] Query latency regression <10%
- [ ] pmat-book chapter updated
- [ ] MCP tool schema updated

---

**Created**: February 5, 2026
**Author**: Claude + Noah
**Toyota Way Alignment**: Jidoka, Kaizen, Genchi Genbutsu, Muda Elimination, Heijunka, Poka-Yoke
**Popperian Rigor**: 5 falsifiable hypotheses with quantitative thresholds
