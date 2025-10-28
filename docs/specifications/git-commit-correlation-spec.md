# Git-Commit Correlation Specification

**Status**: Draft
**Type**: Specification
**Created**: 2025-10-28
**Priority**: P1 (High - Quality Evolution Tracking)
**Complexity**: High
**Sprint**: 65 (Proposed)
**Inspired By**: HGM (Huxley-Gödel Machine) quality tracking system

---

## Executive Summary

This specification defines a git-commit correlation system for PMAT's Technical Debt Grading (TDG) that links quality metrics to specific git commits, enabling users to answer critical questions like "Which commit broke quality?", "When did complexity spike?", and "What's the quality delta between releases?". The system extends PMAT's existing TDG time-series infrastructure with git context awareness, providing git bisect-style quality archaeology.

---

## 1. Problem Statement

### 1.1 Current State

**TDG Tracks Time, Not Git Context:**
- ✅ TDG has time-series tracking (server/src/tdg/metrics_aggregator.rs:14)
- ✅ TDG stores `AnalysisMetadata` with timestamps (server/src/tdg/storage.rs:42)
- ✅ TDG provides trending (Rising, Falling, Stable, Volatile)
- ❌ TDG does NOT track git commit SHA
- ❌ TDG does NOT track branch name
- ❌ TDG does NOT track author information
- ❌ TDG does NOT link quality metrics to code changes

**Questions Users CANNOT Answer:**
1. "Which commit caused our TDG grade to drop from A to B?"
2. "What's the quality impact of author X's last 10 commits?"
3. "Show me TDG evolution from v2.177.0 to v2.178.0"
4. "Which files regressed in the last sprint?"
5. "Who introduced the complexity spike in parser.rs?"

### 1.2 Desired State

**TDG Tracks Quality Per Commit:**
- ✅ Every TDG analysis stores git commit SHA, branch, author
- ✅ Query TDG by commit range (e.g., `HEAD~10..HEAD`)
- ✅ Compare TDG between git tags (e.g., `v2.177.0` vs `v2.178.0`)
- ✅ Visualize quality evolution alongside git history
- ✅ Git bisect-style quality archaeology ("find commit that broke quality")
- ✅ Per-author quality impact analysis
- ✅ Per-branch quality tracking (feature branches vs main)

**Questions Users CAN Answer:**
1. ✅ "Show TDG at commit abc123" → `pmat tdg history --commit abc123`
2. ✅ "Quality impact of last 10 commits" → `pmat tdg history --since HEAD~10`
3. ✅ "Delta between releases" → `pmat tdg compare v2.177.0..v2.178.0`
4. ✅ "Files that regressed" → `pmat tdg regressions --since main@{1.week.ago}`
5. ✅ "Author quality impact" → `pmat tdg by-author --author noah`

---

## 2. Design Principles

### 2.1 Core Principles

1. **Git-Native Integration**
   - Leverage git's existing commit SHA, author, branch metadata
   - No separate versioning system (git is the source of truth)
   - Support git tags, branches, ranges (e.g., `HEAD~5..HEAD`)

2. **Zero-Overhead Storage**
   - Git metadata adds minimal storage (~100 bytes per record)
   - Reuse existing TDG tiered storage (Hot/Warm/Cold)
   - Compress historical data with LZ4 (already implemented)

3. **Backward Compatible**
   - Existing TDG records without git context still work
   - Graceful degradation if not in a git repository
   - Migration path from time-only to git-linked records

4. **Query-Optimized**
   - Index by commit SHA for O(1) lookup
   - Index by timestamp for time-range queries
   - Index by author for per-developer analytics

5. **Toyota Way - Jidoka (Built-In Quality)**
   - Detect quality regressions automatically
   - Alert on TDG grade drops per commit
   - Integrate with CI/CD quality gates

---

## 3. Data Model

### 3.1 Git Context Metadata

**New: `GitContext` Struct**

```rust
// server/src/models/git_context.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Git context for a specific analysis run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitContext {
    /// Full commit SHA (40 hex chars)
    pub commit_sha: String,

    /// Short commit SHA (7 hex chars) for display
    pub commit_sha_short: String,

    /// Branch name (e.g., "main", "feature/tdg-git")
    pub branch: String,

    /// Commit author name
    pub author_name: String,

    /// Commit author email
    pub author_email: String,

    /// Commit timestamp (when code was committed)
    pub commit_timestamp: DateTime<Utc>,

    /// Commit message (first line only)
    pub commit_message: String,

    /// Git tags at this commit (e.g., ["v2.177.0"])
    pub tags: Vec<String>,

    /// Parent commit SHAs (for merge commits)
    pub parent_commits: Vec<String>,

    /// Repository remote URL (if available)
    pub remote_url: Option<String>,

    /// Is working directory clean? (false = uncommitted changes)
    pub is_clean: bool,

    /// Uncommitted file count (if is_clean = false)
    pub uncommitted_files: usize,
}

impl GitContext {
    /// Extract git context from the current working directory
    pub fn from_current_dir(repo_path: &Path) -> Result<Self, GitContextError>;

    /// Extract git context from a specific commit SHA
    pub fn from_commit_sha(repo_path: &Path, sha: &str) -> Result<Self, GitContextError>;

    /// Check if we're in a git repository
    pub fn is_git_repo(path: &Path) -> bool;

    /// Get git context or return None if not in a git repo
    pub fn try_from_current_dir(repo_path: &Path) -> Option<Self>;
}
```

### 3.2 Enhanced TDG Record

**Modified: `FullTdgRecord` (server/src/tdg/storage.rs:52)**

```rust
// server/src/tdg/storage.rs

use crate::models::git_context::GitContext;

/// Full TDG record for transactional storage (ENHANCED with git context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTdgRecord {
    pub identity: FileIdentity,
    pub score: TdgScore,
    pub components: ComponentScores,
    pub semantic_sig: SemanticSignature,
    pub metadata: AnalysisMetadata,

    /// NEW: Git context (None if not in a git repo)
    pub git_context: Option<GitContext>,
}
```

**Modified: `AnalysisMetadata` (server/src/tdg/storage.rs:42)**

```rust
/// Analysis metadata for quality tracking (UNCHANGED - git context separate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analyzer_version: String,
    pub analysis_duration_ms: u64,
    pub language_confidence: f32,
    pub analysis_timestamp: SystemTime, // When analysis ran (may differ from commit time)
    pub cache_hit: bool,
}
```

**Rationale**: Keep `AnalysisMetadata` unchanged, add `git_context` as optional field to maintain backward compatibility.

### 3.3 Storage Schema

**Extended: `TieredStore` Indexes**

```rust
// server/src/tdg/storage.rs

use dashmap::DashMap;
use std::sync::Arc;

pub struct TieredStore {
    /// Hot cache - recent files (in-memory)
    hot: Arc<DashMap<Blake3Hash, HotCacheEntry>>,

    /// NEW: Git commit SHA index (commit_sha -> Vec<Blake3Hash>)
    commit_index: Arc<DashMap<String, Vec<Blake3Hash>>>,

    /// NEW: Author index (author_email -> Vec<Blake3Hash>)
    author_index: Arc<DashMap<String, Vec<Blake3Hash>>>,

    /// NEW: Branch index (branch_name -> Vec<Blake3Hash>)
    branch_index: Arc<DashMap<String, Vec<Blake3Hash>>>,

    /// NEW: Tag index (tag_name -> Vec<Blake3Hash>)
    tag_index: Arc<DashMap<String, Vec<Blake3Hash>>>,

    /// Warm storage - compressed recent records (backend-agnostic)
    warm_backend: Box<dyn StorageBackend>,

    /// Cold storage - full historical records (backend-agnostic)
    cold_backend: Box<dyn StorageBackend>,

    /// Archival configuration
    archive_after_days: u32,
}

impl TieredStore {
    /// NEW: Query records by commit SHA
    pub fn get_by_commit(&self, commit_sha: &str) -> Result<Vec<FullTdgRecord>>;

    /// NEW: Query records by author
    pub fn get_by_author(&self, author_email: &str) -> Result<Vec<FullTdgRecord>>;

    /// NEW: Query records by branch
    pub fn get_by_branch(&self, branch: &str) -> Result<Vec<FullTdgRecord>>;

    /// NEW: Query records by git tag
    pub fn get_by_tag(&self, tag: &str) -> Result<Vec<FullTdgRecord>>;

    /// NEW: Query records by commit range (e.g., "HEAD~10..HEAD")
    pub fn get_by_commit_range(&self, repo_path: &Path, range: &str) -> Result<Vec<FullTdgRecord>>;
}
```

**Index Update Strategy**:
- Indexes built incrementally on record insertion
- Indexes stored in memory (DashMap for concurrency)
- Indexes persisted to warm/cold storage for recovery
- Indexes rebuilt from cold storage on startup if missing

---

## 4. CLI Interface

### 4.1 New Commands

#### 4.1.1 `pmat tdg history` - Query TDG by Git History

```bash
# Show TDG for specific commit
pmat tdg history --commit abc123def456

# Show TDG for last 10 commits
pmat tdg history --since HEAD~10

# Show TDG between two commits
pmat tdg history --range v2.177.0..v2.178.0

# Show TDG for specific author
pmat tdg history --author noah@example.com

# Show TDG for specific branch
pmat tdg history --branch feature/new-parser

# Show TDG for specific file across commits
pmat tdg history --file server/src/lib.rs --since HEAD~20

# Output format
pmat tdg history --since HEAD~5 --format table
pmat tdg history --since HEAD~5 --format json
pmat tdg history --since HEAD~5 --format markdown
pmat tdg history --since HEAD~5 --format chart  # ASCII chart
```

**Output Example (table format)**:
```
Commit      Date        Author  Branch  Grade  Total  Complexity  Files Changed
abc123d     2025-10-28  Noah    main    A      92.3   8.2         server/src/lib.rs (+15, -3)
def456e     2025-10-27  Noah    main    B      85.1   12.5        server/src/parser.rs (+50, -10)
789abcd     2025-10-26  Alice   main    A      91.7   8.8         server/src/utils.rs (+5, -2)

Quality Trend: ↗ Improving (Grade B → A over 3 commits)
Top Contributor: Noah (2 commits, avg grade A-)
Risk Files: server/src/parser.rs (grade drop B → B)
```

#### 4.1.2 `pmat tdg compare` - Compare TDG Between Commits/Tags

```bash
# Compare two commits
pmat tdg compare abc123..def456

# Compare two tags
pmat tdg compare v2.177.0..v2.178.0

# Compare branch against main
pmat tdg compare main..feature/new-parser

# Compare with HEAD
pmat tdg compare HEAD~10..HEAD

# Show only regressions
pmat tdg compare v2.177.0..v2.178.0 --regressions-only

# Show per-file breakdown
pmat tdg compare v2.177.0..v2.178.0 --per-file
```

**Output Example**:
```
TDG Comparison: v2.177.0 → v2.178.0

Overall:
  Grade:      A- → A     (↗ Improved)
  Total:      88.5 → 92.3 (+3.8 points)
  Complexity: 10.2 → 8.1  (-2.1 points, ↗ Improved)

Files Improved (5):
  ✅ server/src/lib.rs          B+ → A  (+5.2 points)
  ✅ server/src/utils.rs        A  → A+ (+1.8 points)
  ✅ server/src/parser.rs       C  → B  (+8.3 points)
  ✅ server/src/complexity.rs   B  → A  (+4.1 points)
  ✅ server/src/analysis.rs     A- → A  (+2.3 points)

Files Regressed (1):
  ❌ server/src/mutation.rs     A  → B+ (-2.1 points)
     Reason: Complexity increased from 8.5 → 12.3
     Commit: def456e (Noah, 2025-10-27)
     Message: "feat: Add mutation operator registry"

New Files (2):
  🆕 server/src/registry.rs     B  (new file, 250 lines)
  🆕 server/src/operators.rs    A- (new file, 180 lines)

Deleted Files (0):

Authors:
  Noah:  6 commits, +1,200 lines, avg grade A-
  Alice: 2 commits, +150 lines, avg grade A
```

#### 4.1.3 `pmat tdg regressions` - Find Quality Regressions

```bash
# Find regressions since last week
pmat tdg regressions --since main@{1.week.ago}

# Find regressions in last 10 commits
pmat tdg regressions --since HEAD~10

# Find regressions by specific author
pmat tdg regressions --author noah@example.com --since HEAD~20

# Find files that dropped grade
pmat tdg regressions --min-grade-drop 1  # Dropped at least 1 letter grade

# Output with git blame
pmat tdg regressions --since HEAD~10 --show-blame
```

**Output Example**:
```
Quality Regressions (last 10 commits)

server/src/mutation.rs:
  Grade:      A → B+ (regression)
  Complexity: 8.5 → 12.3 (+3.8 points)
  Commit:     def456e (2025-10-27)
  Author:     Noah <noah@example.com>
  Message:    "feat: Add mutation operator registry"
  Files:      +50 lines, -10 lines

  Root Cause Analysis:
    • Added 3 new nested loops (cognitive complexity +5)
    • Function `register_operator` has cyclomatic complexity 15 (threshold: 10)
    • Suggested fix: Extract method for validation logic

server/src/parser.rs:
  Grade:      B+ → B (minor regression)
  Duplication: 12% → 18% (+6 percentage points)
  Commit:     789abcd (2025-10-26)
  Author:     Alice <alice@example.com>
  Message:    "fix: Handle edge case in expression parsing"
  Files:      +35 lines, -5 lines

  Root Cause Analysis:
    • Duplicated error handling pattern across 4 functions
    • Suggested fix: Extract error handling into helper function
```

#### 4.1.4 `pmat tdg by-author` - Per-Author Quality Analytics

```bash
# Show quality metrics by author
pmat tdg by-author --since HEAD~50

# Show specific author
pmat tdg by-author --author noah@example.com

# Show leaderboard
pmat tdg by-author --leaderboard --since main@{1.month.ago}

# Show impact per author (lines changed weighted by grade)
pmat tdg by-author --show-impact
```

**Output Example**:
```
Author Quality Analytics (last 50 commits)

Noah <noah@example.com>:
  Commits:     32 commits (64%)
  Lines:       +2,500 / -800 (net +1,700)
  Avg Grade:   A- (89.2 / 100)
  Best Grade:  A+ (95.3) - server/src/utils.rs
  Worst Grade: B  (82.1) - server/src/mutation.rs
  Trend:       ↗ Improving (+2.3 points over period)

  Grade Distribution:
    A+ ████████ 8 files
    A  ██████████████ 14 files
    B+ ████ 4 files
    B  ██ 2 files

  Quality Impact Score: +142.5
    (lines changed × grade improvement)

Alice <alice@example.com>:
  Commits:     18 commits (36%)
  Lines:       +900 / -200 (net +700)
  Avg Grade:   A (91.7 / 100)
  Best Grade:  A+ (96.8) - server/src/analysis.rs
  Worst Grade: B+ (87.5) - server/src/parser.rs
  Trend:       → Stable (±0.5 points over period)

  Grade Distribution:
    A+ ██████ 6 files
    A  ████████ 8 files
    B+ ██ 2 files

  Quality Impact Score: +78.3

Leaderboard (by avg grade):
  1. Alice  - A  (91.7) 🥇
  2. Noah   - A- (89.2) 🥈
```

#### 4.1.5 `pmat tdg bisect` - Git Bisect for Quality

```bash
# Find commit that broke quality (interactive)
pmat tdg bisect --good v2.177.0 --bad HEAD

# Find commit where file dropped below grade B
pmat tdg bisect --file server/src/lib.rs --threshold "grade:B"

# Find commit where complexity spiked
pmat tdg bisect --file server/src/parser.rs --threshold "complexity:10"
```

**Output Example** (interactive session):
```
$ pmat tdg bisect --good v2.177.0 --bad HEAD

Bisecting: 15 revisions left to test after this (roughly 4 steps)
[abc123d] Checking commit abc123d...

server/src/lib.rs: Grade A (92.3) ✅ GOOD
Mark this commit as good or bad? [g/b/skip]: b

Bisecting: 7 revisions left to test after this (roughly 3 steps)
[def456e] Checking commit def456e...

server/src/lib.rs: Grade B+ (87.1) ✅ GOOD
Mark this commit as good or bad? [g/b/skip]: b

Bisecting: 3 revisions left to test after this (roughly 2 steps)
[789abcd] Checking commit 789abcd...

server/src/lib.rs: Grade B (82.5) ❌ BAD
Mark this commit as good or bad? [g/b/skip]: g

Bisecting: 1 revision left to test after this (roughly 1 step)
[111aaaa] Checking commit 111aaaa...

server/src/lib.rs: Grade B- (79.8) ❌ BAD

✅ First bad commit: 111aaaa
Author:  Noah <noah@example.com>
Date:    2025-10-27 14:32:11
Message: feat: Add complex mutation logic

Quality Regression:
  Grade:      A → B- (dropped 2 letter grades)
  Complexity: 8.2 → 15.7 (+7.5 points)

Suggested fixes:
  1. Extract nested loops into separate functions
  2. Reduce cyclomatic complexity in `apply_mutation()`
  3. Split 250-line function into smaller units
```

### 4.2 Modified Existing Commands

#### 4.2.1 `pmat tdg` - Store Git Context by Default

```bash
# Default behavior: Store TDG WITH git context (if in git repo)
pmat tdg server/src/lib.rs

# Explicit flag to store git context
pmat tdg server/src/lib.rs --with-git-context

# Skip git context (time-only, backward compatible)
pmat tdg server/src/lib.rs --no-git-context
```

**Output Enhanced with Git Info**:
```
Analyzing: server/src/lib.rs

Git Context:
  Commit:  abc123def456 (abc123d)
  Branch:  main
  Author:  Noah <noah@example.com>
  Date:    2025-10-28 15:20:45 +0100
  Message: docs: Update roadmap and project state summary (v2.178.0)
  Tags:    v2.178.0
  Clean:   ✅ Yes (no uncommitted changes)

TDG Analysis:
  Grade:      A
  Total:      92.3 / 100
  Complexity: 8.2
  [... rest of output unchanged ...]

✅ Stored with git context: commit abc123d
```

#### 4.2.2 `pmat tdg dashboard` - Show Git Timeline

```bash
# Start dashboard with git timeline view
pmat tdg dashboard --with-git-timeline

# Dashboard port and host
pmat tdg dashboard --port 8080 --host 127.0.0.1 --open
```

**Dashboard Enhancements**:
- New tab: "Git Timeline" (quality evolution graph)
- Commit markers on timeline graph
- Click commit → show files changed and TDG delta
- Filter by author, branch, date range
- Export timeline as PNG/SVG

---

## 5. Implementation Plan

### 5.1 Phase 1: Core Data Model (Week 1)

**File**: `server/src/models/git_context.rs` (NEW)

**Tasks**:
1. Define `GitContext` struct (see Section 3.1)
2. Implement `GitContext::from_current_dir()` using `git2-rs` crate
3. Implement `GitContext::from_commit_sha()` for historical queries
4. Add unit tests for git context extraction
5. Add property tests for edge cases (merge commits, detached HEAD, dirty working tree)

**Dependencies**:
- Add `git2 = "0.18"` to `Cargo.toml`
- Add `chrono` (already in project)

**Tests**:
- ✅ Extract git context from clean repo
- ✅ Extract git context from dirty repo (uncommitted changes)
- ✅ Extract git context from detached HEAD
- ✅ Extract git context from merge commit (multiple parents)
- ✅ Handle git tags (multiple tags at same commit)
- ✅ Handle non-git directory (return None)
- ✅ Handle bare repository

**Acceptance Criteria**:
- All tests passing (100% coverage for git_context.rs)
- Zero panics on edge cases
- Graceful degradation if git command fails

### 5.2 Phase 2: Storage Schema Extension (Week 1-2)

**Files**:
- `server/src/tdg/storage.rs` (MODIFY)
- `server/src/models/mod.rs` (MODIFY)

**Tasks**:
1. Add `git_context: Option<GitContext>` to `FullTdgRecord`
2. Implement indexes (commit, author, branch, tag)
3. Add `TieredStore::get_by_commit()` query method
4. Add `TieredStore::get_by_author()` query method
5. Add `TieredStore::get_by_branch()` query method
6. Add `TieredStore::get_by_commit_range()` query method
7. Implement index persistence (warm/cold storage)
8. Add migration logic (old records without git_context)

**Storage Overhead**:
```
GitContext struct size: ~200 bytes (uncompressed)
LZ4 compressed:         ~100 bytes (50% compression ratio)
Per-file overhead:      ~100 bytes
10,000 files:           ~1 MB total
```

**Tests**:
- ✅ Store record with git context
- ✅ Store record without git context (backward compat)
- ✅ Query by commit SHA (O(1) lookup)
- ✅ Query by author (returns all commits by author)
- ✅ Query by branch (returns all commits on branch)
- ✅ Query by commit range (e.g., `HEAD~10..HEAD`)
- ✅ Index persistence and recovery
- ✅ Migration from old schema (records without git_context)

**Acceptance Criteria**:
- All tests passing (100% coverage for storage changes)
- No performance regression (<5% overhead for index updates)
- Backward compatibility maintained (old records load correctly)

### 5.3 Phase 3: CLI Commands (Week 2-3)

**Files**:
- `server/src/cli/commands.rs` (MODIFY)
- `server/src/cli/handlers/tdg_git_handlers.rs` (NEW)

**Tasks**:
1. Implement `pmat tdg history` command
2. Implement `pmat tdg compare` command
3. Implement `pmat tdg regressions` command
4. Implement `pmat tdg by-author` command
5. Implement `pmat tdg bisect` command (interactive)
6. Add git range parsing (e.g., `HEAD~10..HEAD`, `v2.177.0..v2.178.0`)
7. Add output formatting (table, JSON, markdown, chart)
8. Add ASCII chart rendering for quality trends

**Tests**:
- ✅ `pmat tdg history --commit <sha>` returns correct record
- ✅ `pmat tdg history --since HEAD~10` returns last 10 commits
- ✅ `pmat tdg history --range v1..v2` returns range
- ✅ `pmat tdg compare v1..v2` shows delta
- ✅ `pmat tdg regressions` detects grade drops
- ✅ `pmat tdg by-author` aggregates by author
- ✅ `pmat tdg bisect` finds first bad commit (mock interactive input)

**Acceptance Criteria**:
- All commands have `--help` documentation
- All output formats working (table, JSON, markdown)
- Zero panics on invalid git ranges
- Helpful error messages (e.g., "commit abc123 not found in TDG storage")

### 5.4 Phase 4: Dashboard Integration (Week 3-4)

**Files**:
- `server/src/tdg/web_dashboard.rs` (MODIFY)
- `server/src/tdg/export.rs` (MODIFY)

**Tasks**:
1. Add "Git Timeline" tab to dashboard
2. Implement quality evolution line chart (Chart.js or similar)
3. Add commit markers on timeline
4. Implement drill-down (click commit → show files changed)
5. Add filters (author, branch, date range)
6. Add export (PNG, SVG, CSV)
7. Add real-time updates (WebSocket)

**Tests**:
- ✅ Dashboard renders git timeline
- ✅ Timeline shows commit markers
- ✅ Click commit → shows TDG delta
- ✅ Filter by author works
- ✅ Export to PNG works
- ✅ Real-time updates on new TDG analysis

**Acceptance Criteria**:
- Dashboard works in Chrome, Firefox, Safari
- Timeline renders smoothly (60 FPS)
- Export formats validated (PNG, SVG, CSV)

### 5.5 Phase 5: Documentation & Examples (Week 4)

**Files**:
- `docs/guides/tdg-git-correlation.md` (NEW)
- `docs/guides/mutation-testing.md` (UPDATE - add git correlation section)
- `README.md` (UPDATE - add git correlation section)
- `CLAUDE.md` (UPDATE - add git correlation section)

**Tasks**:
1. Write user guide for git correlation features
2. Add CLI examples for each command
3. Add troubleshooting section
4. Add FAQ (10 questions)
5. Add best practices guide
6. Update README with git correlation section

**Acceptance Criteria**:
- All commands documented with examples
- FAQ covers common use cases
- Cross-links validated (no 404s)

---

## 6. Technical Architecture

### 6.1 Git Integration Layer

```rust
// server/src/services/git_integration.rs (NEW)

use git2::{Repository, Commit, Oid};
use anyhow::Result;

pub struct GitIntegration;

impl GitIntegration {
    /// Open git repository at path
    pub fn open_repo(path: &Path) -> Result<Repository>;

    /// Get current commit SHA
    pub fn get_current_commit_sha(repo: &Repository) -> Result<String>;

    /// Get commit by SHA
    pub fn get_commit(repo: &Repository, sha: &str) -> Result<Commit>;

    /// Get commits in range (e.g., "HEAD~10..HEAD")
    pub fn get_commits_in_range(repo: &Repository, range: &str) -> Result<Vec<Commit>>;

    /// Get commit diff (files changed, additions, deletions)
    pub fn get_commit_diff(repo: &Repository, commit: &Commit) -> Result<CommitDiff>;

    /// Check if working directory is clean
    pub fn is_clean(repo: &Repository) -> Result<bool>;

    /// Get tags at commit
    pub fn get_tags_at_commit(repo: &Repository, commit: &Commit) -> Result<Vec<String>>;
}
```

### 6.2 Query Engine

```rust
// server/src/tdg/query_engine.rs (NEW)

use crate::models::git_context::GitContext;
use crate::tdg::storage::{FullTdgRecord, TieredStore};
use anyhow::Result;

pub struct TdgQueryEngine {
    store: Arc<TieredStore>,
    git_integration: GitIntegration,
}

impl TdgQueryEngine {
    /// Query TDG records by commit SHA
    pub fn query_by_commit(&self, commit_sha: &str) -> Result<Vec<FullTdgRecord>>;

    /// Query TDG records by commit range
    pub fn query_by_range(&self, repo_path: &Path, range: &str) -> Result<Vec<FullTdgRecord>>;

    /// Query TDG records by author
    pub fn query_by_author(&self, author_email: &str) -> Result<Vec<FullTdgRecord>>;

    /// Query TDG records by branch
    pub fn query_by_branch(&self, branch: &str) -> Result<Vec<FullTdgRecord>>;

    /// Find quality regressions in commit range
    pub fn find_regressions(&self, repo_path: &Path, range: &str, threshold: f32) -> Result<Vec<QualityRegression>>;

    /// Compare TDG between two commits
    pub fn compare_commits(&self, commit1: &str, commit2: &str) -> Result<TdgComparison>;

    /// Aggregate TDG by author
    pub fn aggregate_by_author(&self, range: &str) -> Result<HashMap<String, AuthorStats>>;
}
```

### 6.3 Regression Detector

```rust
// server/src/tdg/regression_detector.rs (NEW)

use crate::tdg::TdgScore;

pub struct RegressionDetector;

impl RegressionDetector {
    /// Detect if TDG grade dropped between two records
    pub fn is_regression(before: &TdgScore, after: &TdgScore) -> bool;

    /// Calculate regression severity (0.0 = no regression, 1.0 = critical)
    pub fn regression_severity(before: &TdgScore, after: &TdgScore) -> f32;

    /// Diagnose root cause of regression
    pub fn diagnose_regression(before: &FullTdgRecord, after: &FullTdgRecord) -> RegressionDiagnosis;

    /// Suggest fixes for regression
    pub fn suggest_fixes(diagnosis: &RegressionDiagnosis) -> Vec<String>;
}

#[derive(Debug)]
pub struct RegressionDiagnosis {
    pub component: String,        // Which component regressed (complexity, duplication, etc.)
    pub delta: f32,                // How much it regressed
    pub root_cause: String,        // Human-readable explanation
    pub affected_functions: Vec<String>,
}
```

---

## 7. Performance Considerations

### 7.1 Index Performance

**Index Sizes** (10,000 files, 1,000 commits):
- Commit index: ~1,000 entries × 40 bytes = 40 KB
- Author index: ~10 authors × (10 commits × 40 bytes) = 4 KB
- Branch index: ~5 branches × (200 commits × 40 bytes) = 40 KB
- **Total index size**: ~100 KB (in-memory)

**Query Performance**:
- Lookup by commit SHA: O(1) (DashMap)
- Lookup by author: O(1) → O(n) iteration over author's commits
- Lookup by range: O(m) where m = commits in range
- **Target**: <10ms for single commit, <100ms for 100-commit range

### 7.2 Storage Overhead

**Per-Record Overhead**:
- GitContext struct: ~200 bytes (uncompressed)
- LZ4 compressed: ~100 bytes
- **Increase**: ~10% of current TDG record size

**Disk Space** (10,000 files):
- Without git context: 10,000 × 1 KB = 10 MB
- With git context: 10,000 × 1.1 KB = 11 MB
- **Overhead**: 1 MB (10% increase)

### 7.3 Git Command Performance

**Git Operations**:
- `git log`: ~50ms for 1,000 commits
- `git show`: ~10ms per commit
- `git diff`: ~20ms per commit

**Mitigation**:
- Cache git repository handle (avoid repeated opens)
- Batch git operations where possible
- Use libgit2 (git2-rs) instead of shelling out

---

## 8. Quality Gates

### 8.1 Pre-Commit Checks

**Hook**: `.git/hooks/pre-commit`

```bash
#!/bin/bash
# Check if TDG regressed compared to HEAD~1

echo "Checking TDG quality regression..."

# Run TDG on staged files
pmat tdg . --format json > /tmp/tdg_current.json

# Get TDG from HEAD~1 (previous commit)
git stash  # Stash uncommitted changes
pmat tdg . --format json > /tmp/tdg_previous.json
git stash pop

# Compare
pmat tdg compare HEAD~1..HEAD --regressions-only --format text

if [ $? -ne 0 ]; then
    echo "❌ TDG quality regressed. Fix issues or use 'git commit --no-verify' to bypass."
    exit 1
fi

echo "✅ TDG quality maintained or improved."
exit 0
```

### 8.2 CI/CD Integration

**GitHub Actions Example**:

```yaml
# .github/workflows/tdg-quality-gate.yml

name: TDG Quality Gate

on:
  pull_request:
    branches: [main, master]

jobs:
  tdg-regression-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Fetch full history for git range queries

      - name: Install PMAT
        run: cargo install pmat

      - name: Check TDG Regressions
        run: |
          # Compare PR branch against base branch
          pmat tdg compare origin/${{ github.base_ref }}..HEAD --regressions-only --format markdown > tdg_report.md

      - name: Post PR Comment
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('tdg_report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## TDG Quality Report\n\n${report}`
            });

      - name: Fail if Regressions
        run: |
          pmat tdg compare origin/${{ github.base_ref }}..HEAD --regressions-only --fail-on-regression
```

---

## 9. Use Cases

### 9.1 Developer Workflow

**Scenario**: Developer wants to check quality impact before pushing

```bash
# Before committing
git add .
pmat tdg . --format table

# Check if quality regressed compared to HEAD
pmat tdg compare HEAD..WORKING_DIR --regressions-only

# If regression found, diagnose
pmat tdg regressions --since HEAD --show-blame

# Fix issues, then commit
git commit -m "refactor: Reduce complexity in parser.rs"

# Verify fix improved quality
pmat tdg compare HEAD~1..HEAD
```

### 9.2 Code Review Workflow

**Scenario**: Reviewer wants to check PR quality impact

```bash
# Checkout PR branch
git checkout feature/new-parser

# Compare against main
pmat tdg compare main..feature/new-parser --per-file --format markdown > pr_tdg_report.md

# Check if any files regressed
pmat tdg regressions --since main --min-grade-drop 1

# Post report as PR comment
gh pr comment 123 --body-file pr_tdg_report.md
```

### 9.3 Release Quality Assessment

**Scenario**: Release manager wants to verify quality improved in release

```bash
# Compare two releases
pmat tdg compare v2.177.0..v2.178.0 --format markdown > release_quality_report.md

# Check for any critical regressions
pmat tdg regressions --range v2.177.0..v2.178.0 --min-grade-drop 2

# Generate author impact report
pmat tdg by-author --range v2.177.0..v2.178.0 --show-impact

# Add to release notes
cat release_quality_report.md >> RELEASE_NOTES.md
```

### 9.4 Quality Archaeology

**Scenario**: Developer wants to find when complexity spiked

```bash
# Find commit where parser.rs complexity exceeded 10
pmat tdg bisect --file server/src/parser.rs --threshold "complexity:10" --good v2.170.0 --bad HEAD

# Output:
# First bad commit: abc123d
# Author: Alice <alice@example.com>
# Date:   2025-10-15 14:22:33
# Message: feat: Add support for nested expressions
#
# Complexity spike: 8.2 → 15.7 (+7.5 points)
# Suggested fix: Extract nested logic into helper functions
```

### 9.5 Team Performance Analytics

**Scenario**: Engineering manager wants to track team quality contributions

```bash
# Generate monthly team report
pmat tdg by-author --since main@{1.month.ago} --format markdown > team_quality_report.md

# Show leaderboard
pmat tdg by-author --leaderboard --since main@{1.month.ago}

# Output:
# Quality Leaderboard (last month)
# 1. Alice  - A  (92.5 avg) 🥇
# 2. Bob    - A- (89.8 avg) 🥈
# 3. Carol  - B+ (87.1 avg) 🥉
#
# Team Stats:
#   Avg Grade:    A- (90.1)
#   Total Commits: 127
#   Quality Trend: ↗ Improving (+3.2 points)
```

---

## 10. Migration Strategy

### 10.1 Backward Compatibility

**Approach**: Optional git context (graceful degradation)

```rust
// Old records without git_context still work
pub struct FullTdgRecord {
    pub identity: FileIdentity,
    pub score: TdgScore,
    pub components: ComponentScores,
    pub semantic_sig: SemanticSignature,
    pub metadata: AnalysisMetadata,
    pub git_context: Option<GitContext>,  // None for old records
}

// Queries handle missing git_context gracefully
impl TdgQueryEngine {
    pub fn query_by_commit(&self, commit_sha: &str) -> Result<Vec<FullTdgRecord>> {
        // If record.git_context.is_none(), skip it (can't query by commit)
        self.store.get_by_commit(commit_sha)
            .into_iter()
            .filter(|r| r.git_context.is_some())
            .collect()
    }
}
```

### 10.2 Migration Path

**Step 1**: Install PMAT v2.179.0 (with git correlation)

```bash
cargo install pmat --version 2.179.0
```

**Step 2**: Existing TDG records continue to work

```bash
# Old records (without git context) still queryable by time
pmat tdg history --since 2025-10-01  # Works (queries by timestamp)

# Git-based queries only work for new records
pmat tdg history --since HEAD~10  # Only returns records with git_context
```

**Step 3**: Re-analyze to backfill git context (optional)

```bash
# Re-analyze all files to add git context
pmat tdg . --recursive --with-git-context

# Or incrementally: re-analyze on next TDG run
# Git context automatically added for new analyses
```

---

## 11. Testing Strategy

### 11.1 Unit Tests

**Coverage Target**: 100% for new code

**Files**:
- `server/src/models/git_context.rs` (NEW)
- `server/src/tdg/query_engine.rs` (NEW)
- `server/src/tdg/regression_detector.rs` (NEW)
- `server/src/services/git_integration.rs` (NEW)

**Test Cases**:
- ✅ Extract git context from clean repo
- ✅ Extract git context from dirty repo
- ✅ Extract git context from detached HEAD
- ✅ Extract git context from merge commit
- ✅ Handle git tags
- ✅ Handle non-git directory
- ✅ Query by commit SHA
- ✅ Query by commit range
- ✅ Query by author
- ✅ Detect regressions
- ✅ Compare commits
- ✅ Aggregate by author

### 11.2 Integration Tests

**Test Scenarios**:
1. End-to-end: Store TDG with git context → Query by commit
2. End-to-end: Compare two commits → Verify delta calculation
3. End-to-end: Find regressions → Verify detection accuracy
4. End-to-end: Bisect → Verify first bad commit found
5. CLI integration: All commands work with real git repo

**Test Repository**:
- Create test git repo with known history
- 10 commits with known TDG grades
- 2 authors, 2 branches
- 1 merge commit
- 2 tags (v1.0.0, v2.0.0)

### 11.3 Property Tests

**Properties to Test**:
- Query by commit always returns consistent results
- Regression detection is commutative (compare(A, B) = -compare(B, A))
- Index consistency (commit_index matches actual stored records)
- Git range parsing is idempotent

**Framework**: `proptest` (already in project)

### 11.4 Performance Tests

**Benchmarks**:
- Query 1 commit: <10ms
- Query 100 commits: <100ms
- Query 1,000 commits: <1s
- Index update (1,000 records): <50ms
- Storage overhead: <10% increase

**Framework**: `criterion` (already in project)

---

## 12. Documentation Requirements

### 12.1 User Documentation

**Files**:
- `docs/guides/tdg-git-correlation.md` (NEW, ~2,000 lines)
  - Introduction and motivation
  - Getting started (first git correlation analysis)
  - CLI command reference (all 5 new commands)
  - Use cases (10 scenarios)
  - Troubleshooting (10 common issues)
  - FAQ (10 questions)
  - Best practices

**Sections**:
1. What is Git-Commit Correlation?
2. Why Track Quality Per Commit?
3. Getting Started
4. CLI Commands
5. Use Cases
6. Dashboard
7. CI/CD Integration
8. Troubleshooting
9. FAQ
10. Best Practices

### 12.2 API Documentation

**Rustdoc Coverage**: 100% for new public APIs

**Examples in Rustdoc**:
- Every public function has at least 1 example
- Complex APIs have multiple examples
- Edge cases documented

### 12.3 Changelog

**Entry for v2.179.0**:

```markdown
## [2.179.0] - 2025-11-XX

### Added - Git-Commit Correlation (Sprint 65)

**NEW: Track TDG Quality Per Git Commit**

Link TDG quality metrics to git commits, enabling git bisect-style quality archaeology and per-author/per-release quality tracking.

**New Commands** (5):
- `pmat tdg history` - Query TDG by git history (commit, range, author, branch)
- `pmat tdg compare` - Compare TDG between commits/tags/branches
- `pmat tdg regressions` - Find quality regressions in commit range
- `pmat tdg by-author` - Per-author quality analytics and leaderboard
- `pmat tdg bisect` - Git bisect for quality (find commit that broke quality)

**Enhanced Commands**:
- `pmat tdg` - Now stores git context by default (commit SHA, author, branch, tags)
- `pmat tdg dashboard` - New "Git Timeline" tab with quality evolution graph

**Key Features**:
- ✅ Query TDG by commit SHA, range, author, branch, tag
- ✅ Compare quality between releases (e.g., `v2.177.0..v2.178.0`)
- ✅ Detect quality regressions with root cause analysis
- ✅ Per-author quality impact and leaderboard
- ✅ Git bisect for quality (find first bad commit)
- ✅ Dashboard with interactive git timeline
- ✅ CI/CD integration (GitHub Actions, GitLab CI)
- ✅ Backward compatible (old records still work)

**Storage**:
- Git context stored with every TDG analysis (~100 bytes per record)
- Indexed by commit, author, branch, tag (O(1) lookup)
- Hot/Warm/Cold tiered storage (same as TDG)

**Use Cases**:
- "Which commit broke quality?" → `pmat tdg bisect`
- "Quality impact of this PR?" → `pmat tdg compare main..feature-branch`
- "Team quality leaderboard?" → `pmat tdg by-author --leaderboard`
- "Quality delta between releases?" → `pmat tdg compare v1..v2`

**Documentation**:
- User guide: `docs/guides/tdg-git-correlation.md` (~2,000 lines)
- CLI examples for all commands
- 10 use case scenarios
- CI/CD integration guides
- Troubleshooting and FAQ

**Inspired by**: HGM (Huxley-Gödel Machine) quality tracking system
**Files Changed**: 15 new/modified files, ~3,500 lines added
**Tests**: 100% coverage (unit + integration + property tests)
```

---

## 13. Success Metrics

### 13.1 Functional Metrics

**Definition of Done**:
- ✅ All 5 new CLI commands working
- ✅ All tests passing (100% coverage for new code)
- ✅ Documentation complete (user guide + API docs)
- ✅ Dashboard "Git Timeline" tab working
- ✅ CI/CD integration guides published
- ✅ Zero regressions in existing TDG functionality
- ✅ Backward compatibility validated (old records work)

### 13.2 Performance Metrics

**Targets**:
- ✅ Query by commit: <10ms (90th percentile)
- ✅ Query by range (100 commits): <100ms
- ✅ Storage overhead: <10% increase
- ✅ Index update overhead: <5% per TDG analysis
- ✅ Dashboard timeline render: <200ms

### 13.3 Quality Metrics

**Code Quality**:
- ✅ Zero clippy warnings
- ✅ Zero RUSTSEC warnings
- ✅ 100% test coverage for new code
- ✅ All property tests passing
- ✅ All benchmarks passing (performance targets met)

**Documentation Quality**:
- ✅ All CLI commands have `--help`
- ✅ All public APIs have rustdoc
- ✅ User guide validated (no 404s, no hallucinations)
- ✅ Examples tested (run successfully)

### 13.4 User Adoption Metrics

**Tracking** (post-release):
- Number of `pmat tdg history` invocations
- Number of `pmat tdg compare` invocations
- Number of dashboard "Git Timeline" views
- GitHub Issues related to git correlation
- Community feedback (GitHub Discussions)

**Targets** (3 months post-release):
- 100+ users trying git correlation features
- <5 critical bugs reported
- >80% user satisfaction (survey)

---

## 14. Risk Assessment

### 14.1 Technical Risks

**Risk 1: Git Integration Complexity**
- **Likelihood**: Medium
- **Impact**: High
- **Mitigation**: Use battle-tested `git2-rs` crate, extensive testing, fallback to shell commands if git2 fails

**Risk 2: Storage Overhead**
- **Likelihood**: Low
- **Impact**: Medium
- **Mitigation**: LZ4 compression (~50% savings), optional git context (can disable), tiered storage

**Risk 3: Index Performance**
- **Likelihood**: Low
- **Impact**: Medium
- **Mitigation**: DashMap for concurrent access, index persistence, lazy index loading

**Risk 4: Backward Compatibility**
- **Likelihood**: Low
- **Impact**: High
- **Mitigation**: Optional `git_context` field, graceful degradation, extensive migration testing

### 14.2 User Experience Risks

**Risk 1: Confusing CLI**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**: Comprehensive `--help`, examples in docs, consistent naming

**Risk 2: Slow Queries**
- **Likelihood**: Low
- **Impact**: High
- **Mitigation**: Performance tests, benchmarks, optimized indexes

**Risk 3: Dashboard Complexity**
- **Likelihood**: Medium
- **Impact**: Low
- **Mitigation**: Incremental rollout (start simple, add features), user testing

### 14.3 Project Risks

**Risk 1: Scope Creep**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**: Strict scope definition (5 commands + dashboard), phased rollout

**Risk 2: Timeline Slip**
- **Likelihood**: Medium
- **Impact**: Low
- **Mitigation**: 4-week plan with buffer, MVP first (core commands), dashboard optional

---

## 15. Alternatives Considered

### 15.1 Alternative: External Tool (git-tdg)

**Approach**: Build separate CLI tool `git-tdg` (like `git-lfs`)

**Pros**:
- Separation of concerns
- Easier testing (isolated from PMAT)
- Could be reused by other tools

**Cons**:
- Fragmentation (users need 2 tools)
- Duplication (TDG logic duplicated)
- Maintenance burden (2 projects)

**Decision**: ❌ Rejected - Keep everything in PMAT for better UX

### 15.2 Alternative: Database Backend (SQLite)

**Approach**: Store TDG + git context in SQLite database

**Pros**:
- Powerful queries (SQL)
- Mature indexing (B-trees)
- Standard tooling

**Cons**:
- Overkill for simple queries
- Dependency on SQLite
- Slower than in-memory indexes for hot data
- Complexity for tiered storage (Hot/Warm/Cold)

**Decision**: ❌ Rejected - Current tiered storage + DashMap is sufficient

### 15.3 Alternative: Git Hooks Only

**Approach**: Store git context via git hooks (pre-commit, post-commit)

**Pros**:
- Automatic (no user action needed)
- Always up-to-date

**Cons**:
- Requires git hook installation
- Fragile (hooks can be bypassed)
- Doesn't work for historical analysis

**Decision**: ❌ Rejected - Hybrid approach (store on `pmat tdg` invocation + optional hooks)

---

## 16. Future Enhancements

### 16.1 Phase 2 (Sprint 66+)

**Machine Learning Integration**:
- Predict quality regressions before commit (ML model trained on historical data)
- Suggest refactorings based on similar commits that improved quality
- Auto-detect patterns (e.g., "Friday commits have lower quality")

**Advanced Visualizations**:
- Heatmap (files × time, color = grade)
- Sunburst diagram (directory structure, color = grade)
- Network graph (file dependencies, edge weight = coupling)

**GitLab/GitHub Integration**:
- GitHub App: Post TDG reports as PR comments
- GitLab MR widget: Show TDG delta in merge request UI
- Status checks: Block merge if quality regressed

### 16.2 Phase 3 (Future)

**Team Analytics**:
- Team dashboard (quality KPIs, trends, alerts)
- Slack/Discord notifications (quality milestones, regressions)
- Gamification (badges, achievements, streaks)

**Historical Analysis**:
- Import git history (analyze all commits retroactively)
- Time-series forecasting (predict future quality)
- Anomaly detection (flag unusual quality changes)

---

## 17. Appendix

### 17.1 Related Work

**Git Bisect**:
- Inspiration for `pmat tdg bisect`
- Binary search for first bad commit
- Interactive workflow

**HGM (Huxley-Gödel Machine)**:
- Inspiration for git-linked quality tracking
- Stores performance metrics per git commit
- Compares agent versions via git history

**Code Climate**:
- Commercial tool with git integration
- Shows quality evolution over time
- Per-commit quality reports

### 17.2 References

**Git2-rs Documentation**:
- https://docs.rs/git2/latest/git2/

**PMAT TDG Documentation**:
- `docs/guides/tdg-overview.md`
- `server/src/tdg/mod.rs`

**HGM Paper**:
- arXiv 2510.21614: "Huxley: A Self-Improving AI Coding Agent"

### 17.3 Glossary

**Terms**:
- **TDG**: Technical Debt Grading (PMAT's quality scoring system)
- **Git Context**: Git metadata (commit SHA, author, branch, tags)
- **Tiered Storage**: Hot/Warm/Cold storage strategy (in-memory → compressed → archival)
- **Quality Regression**: Drop in TDG grade between commits
- **Git Bisect**: Binary search for first bad commit
- **Git Range**: Commit range notation (e.g., `HEAD~10..HEAD`, `v1..v2`)

---

**End of Specification**

**Document Version**: 1.0
**Last Updated**: 2025-10-28
**Author**: Claude (AI Pair Programmer)
**Reviewed By**: [Pending]
**Status**: Draft (awaiting approval)
