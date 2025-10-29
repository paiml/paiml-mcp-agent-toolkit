# TDG Score Enforcement System Specification

**Version**: 1.0
**Status**: 🔄 In Progress
**Sprint**: Sprint 66 (Post-v2.179.0)
**Date**: October 28, 2025

---

## Executive Summary

Implement a comprehensive TDG score enforcement system that uses content hashing (blake3) to automatically track quality scores for all files. This system ensures **zero tolerance for quality regressions** by making TDG tracking automatic and enforcement built into the development workflow.

**Key Principle (Toyota Way - Jidoka)**: Build quality into the process, detect defects immediately, stop the line when quality drops.

---

## Problem Statement

### Current State
- TDG analysis is manual (`pmat tdg <file>`)
- Scores stored only when explicitly requested (`--with-git-context`)
- No automatic tracking of all files in a project
- No quality gates to prevent regressions
- Developers can bypass quality checks

### Desired State
- **Automatic TDG tracking** for all source files
- **Content-hash based** deduplication (analyze once per content)
- **Git hook enforcement** (pre-commit quality gates)
- **CI/CD integration** (automated quality validation)
- **Quality trending** (track improvements/regressions over time)

---

## Solution Architecture

### 1. Automatic TDG Tracking

#### Content-Hash Based Analysis

```rust
// FileIdentity already has blake3 content_hash
pub struct FileIdentity {
    pub path: PathBuf,
    pub content_hash: Blake3Hash,  // ✅ Already implemented
    pub size_bytes: u64,
    pub modified_time: SystemTime,
}
```

**Workflow**:
1. On file save/commit, compute blake3 hash of content
2. Check if hash exists in TDG storage
3. If hash exists: Retrieve cached score (instant)
4. If hash not found: Analyze and store with hash
5. Store hash → score mapping for future lookups

**Benefits**:
- Analyze each unique content only once
- Instant score retrieval for unchanged files
- Works across branches (same content = same hash)
- Efficient for large codebases

### 2. Project-Wide TDG Baseline

#### Baseline Creation

```bash
# Create baseline for entire project
pmat tdg baseline create --path . --output .pmat/baseline.json

# Baseline stores:
# {
#   "version": "2.179.0",
#   "created_at": "2025-10-28T22:00:00Z",
#   "files": {
#     "src/lib.rs": {
#       "content_hash": "blake3_hash_here",
#       "score": {
#         "total": 95.5,
#         "grade": "A+",
#         ...
#       },
#       "git_context": {...}
#     }
#   },
#   "summary": {
#     "total_files": 150,
#     "avg_score": 88.2,
#     "grade_distribution": {"A+": 20, "A": 50, ...}
#   }
# }
```

#### Baseline Comparison

```bash
# Compare current state against baseline
pmat tdg baseline compare --baseline .pmat/baseline.json

# Output:
# ✅ Improved: 5 files
#    - src/lib.rs: B+ → A (85.2 → 92.1)
# ⚠️  Regressed: 2 files
#    - src/api.rs: A → B+ (91.0 → 85.5)
# ➡️  Unchanged: 143 files
```

### 3. Git Hook Integration

#### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
# Auto-installed via: pmat hooks install

set -e

echo "🔍 Running TDG quality gates..."

# Get list of staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.\(rs\|ts\|py\|go\|java\|cpp\)$' || true)

if [ -z "$STAGED_FILES" ]; then
  echo "✅ No source files staged, skipping TDG check"
  exit 0
fi

# Analyze staged files with git context
for file in $STAGED_FILES; do
  echo "  Analyzing $file..."
  pmat tdg "$file" --with-git-context --min-grade B+ 2>&1 | grep -E "(Overall Score|ERROR)" || true
done

# Check if any files regressed
if pmat tdg check-regression --staged; then
  echo "✅ TDG quality gates passed"
  exit 0
else
  echo "❌ TDG quality regression detected"
  echo "   Run 'pmat tdg baseline compare' to see details"
  echo "   Override with: git commit --no-verify"
  exit 1
fi
```

#### Post-Commit Hook

```bash
#!/bin/bash
# .git/hooks/post-commit
# Update TDG baseline after successful commit

echo "📊 Updating TDG baseline..."
pmat tdg baseline update --path . --commit HEAD
echo "✅ TDG baseline updated"
```

### 4. CI/CD Integration

#### GitHub Actions

```yaml
# .github/workflows/tdg-quality-gate.yml
name: TDG Quality Gate

on:
  pull_request:
    types: [opened, synchronize]
  push:
    branches: [main, master]

jobs:
  tdg-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for baseline comparison

      - name: Install PMAT
        run: cargo install pmat

      - name: Create baseline from base branch
        run: |
          git checkout ${{ github.base_ref }}
          pmat tdg baseline create --path . --output baseline-base.json

      - name: Analyze current branch
        run: |
          git checkout ${{ github.head_ref }}
          pmat tdg baseline create --path . --output baseline-current.json

      - name: Compare baselines
        run: |
          pmat tdg baseline compare \
            --baseline baseline-base.json \
            --current baseline-current.json \
            --fail-on-regression

      - name: Generate TDG report
        if: always()
        run: |
          pmat tdg baseline compare \
            --baseline baseline-base.json \
            --current baseline-current.json \
            --format markdown > tdg-report.md

      - name: Comment PR with TDG report
        if: always() && github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('tdg-report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

### 5. CLI Commands

#### New Commands

```bash
# Baseline management
pmat tdg baseline create --path <dir> --output <file>
pmat tdg baseline update --path <dir> --commit <sha>
pmat tdg baseline compare --baseline <file> [--current <file>]
pmat tdg baseline export --format json|csv|html

# Quality gates
pmat tdg check-regression --staged
pmat tdg check-regression --commit <sha>
pmat tdg check-quality --min-grade <grade> --path <dir>

# Hook management (already exists, extend)
pmat hooks install --tdg-enforcement
pmat hooks status --verbose
pmat hooks uninstall --tdg-only

# Batch analysis with deduplication
pmat tdg analyze-all --path <dir> --dedupe-by-hash
pmat tdg analyze-all --path <dir> --parallel --workers 8
```

### 6. Quality Gate Rules

#### Rule Configuration

```toml
# .pmat/tdg-rules.toml
[quality_gates]
# Minimum grades by file type
rust_min_grade = "B+"
typescript_min_grade = "B+"
python_min_grade = "B"

# Regression tolerance
max_score_drop = 5.0  # Max points drop allowed
allow_grade_drop = false  # Never allow grade drops

# Enforcement mode
mode = "strict"  # strict | warning | disabled
block_on_regression = true
block_on_new_files_below_threshold = true

[baseline]
# Auto-update baseline
auto_update_on_commit = true
auto_update_on_merge = true

# Baseline storage
baseline_path = ".pmat/baseline.json"
store_in_git = true  # Track baseline in git

[ci_cd]
# CI/CD settings
fail_fast = false
generate_reports = true
comment_on_pr = true
```

---

## Implementation Plan

### Phase 1: Baseline System (Sprint 66 Part 1)

**Tasks**:
1. Create `BaselineManager` struct
2. Implement `create()` - analyze all files, store with hashes
3. Implement `compare()` - diff two baselines
4. Implement `update()` - update baseline for specific commits
5. Add CLI commands: `pmat tdg baseline {create,compare,update}`
6. Add tests (15 RED tests → GREEN)

**Files**:
- `server/src/tdg/baseline.rs` (new, ~400 lines)
- `server/src/cli/handlers/tdg_baseline_handlers.rs` (new, ~300 lines)
- `server/src/cli/commands.rs` (extend TdgCommand enum)
- `server/tests/tdg_baseline_tests.rs` (new, ~250 lines)

**Estimated**: 3-4 hours

### Phase 2: Quality Gate System (Sprint 66 Part 2)

**Tasks**:
1. Create `QualityGate` trait
2. Implement `RegressionGate` - detect quality drops
3. Implement `MinimumGradeGate` - enforce minimum grades
4. Implement `NewFileGate` - enforce quality for new files
5. Add CLI commands: `pmat tdg check-{regression,quality}`
6. Add tests (12 RED tests → GREEN)

**Files**:
- `server/src/tdg/quality_gate.rs` (new, ~350 lines)
- `server/src/cli/handlers/tdg_gate_handlers.rs` (new, ~250 lines)
- `server/tests/tdg_quality_gate_tests.rs` (new, ~200 lines)

**Estimated**: 2-3 hours

### Phase 3: Git Hook Integration (Sprint 66 Part 3)

**Tasks**:
1. Extend `pmat hooks install` with `--tdg-enforcement`
2. Create pre-commit script for TDG checks
3. Create post-commit script for baseline updates
4. Add hook configuration in `.pmat/tdg-rules.toml`
5. Add tests (10 RED tests → GREEN)

**Files**:
- `server/src/cli/handlers/hooks_handlers.rs` (extend, +200 lines)
- `templates/hooks/pre-commit-tdg.sh` (new, ~80 lines)
- `templates/hooks/post-commit-tdg.sh` (new, ~40 lines)
- `server/tests/tdg_hooks_tests.rs` (new, ~150 lines)

**Estimated**: 2 hours

### Phase 4: CI/CD Templates (Sprint 66 Part 4)

**Tasks**:
1. Create GitHub Actions workflow template
2. Create GitLab CI template
3. Create Jenkins pipeline template
4. Add documentation with examples
5. Add CI/CD integration tests

**Files**:
- `templates/ci/github-actions-tdg.yml` (new, ~100 lines)
- `templates/ci/gitlab-ci-tdg.yml` (new, ~80 lines)
- `templates/ci/Jenkinsfile-tdg` (new, ~70 lines)
- `docs/guides/ci-cd-tdg-integration.md` (new, ~400 lines)

**Estimated**: 2 hours

---

## Data Structures

### Baseline Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdgBaseline {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub git_context: Option<GitContext>,
    pub files: HashMap<PathBuf, BaselineEntry>,
    pub summary: BaselineSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub content_hash: Blake3Hash,
    pub score: TdgScore,
    pub components: ComponentScores,
    pub git_context: Option<GitContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSummary {
    pub total_files: usize,
    pub avg_score: f32,
    pub grade_distribution: HashMap<Grade, usize>,
    pub languages: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub improved: Vec<FileComparison>,
    pub regressed: Vec<FileComparison>,
    pub unchanged: Vec<PathBuf>,
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComparison {
    pub path: PathBuf,
    pub old_score: TdgScore,
    pub new_score: TdgScore,
    pub delta: f32,
    pub grade_change: (Grade, Grade),
}
```

---

## Storage Schema

### Baseline Storage

**Location**: `~/.pmat/baselines/`

```
~/.pmat/
├── baselines/
│   ├── project_hash_baseline_v1.json
│   ├── project_hash_baseline_v2.json
│   └── ...
├── tdg-warm/  (existing)
└── tdg-cold/  (existing)
```

**Project Hash**: `blake3(git_repo_path)`

### Deduplication via Content Hash

**Existing storage already supports this**:
- `FileIdentity.content_hash` is blake3 hash
- `TieredStore` uses content_hash as key
- Same content across files/branches = single stored score

---

## Performance Considerations

### Caching Strategy

1. **Hot Cache (In-Memory)**
   - Content hash → TDG score mapping
   - LRU eviction (keep 1000 most recent)
   - <1ms lookup time

2. **Warm Storage (LZ4 Compressed)**
   - Recent TDG records with full metadata
   - <10ms lookup time
   - Already implemented

3. **Cold Storage (Uncompressed)**
   - Historical records
   - <50ms lookup time
   - Already implemented

### Batch Analysis Optimization

```rust
// Parallel analysis with deduplication
pub async fn analyze_project_dedupe(
    &self,
    root: &Path,
    parallelism: usize,
) -> Result<ProjectBaseline> {
    // 1. Discover all source files
    let files = discover_source_files(root)?;

    // 2. Compute content hashes (fast, parallel)
    let hashes: HashMap<PathBuf, Blake3Hash> = files
        .par_iter()
        .map(|f| (f.clone(), compute_hash(f)))
        .collect();

    // 3. Check which hashes are already analyzed
    let cached = self.storage.bulk_lookup(&hashes.values()).await?;

    // 4. Analyze only unique new hashes (parallel)
    let to_analyze: Vec<_> = hashes
        .iter()
        .filter(|(_, h)| !cached.contains_key(h))
        .collect();

    let new_scores: Vec<_> = to_analyze
        .par_iter()
        .map(|(path, _)| self.analyze_file(path))
        .collect();

    // 5. Assemble baseline from cached + new
    Ok(ProjectBaseline::new(cached, new_scores))
}
```

**Expected Performance**:
- 1000 files, all cached: <100ms
- 1000 files, 10% new: ~10 seconds (90% instant, 10% analyzed)
- 1000 files, all new: ~100 seconds (parallel analysis)

---

## Testing Strategy

### Unit Tests (37 total)

**Baseline Manager** (15 tests):
- `test_create_baseline_empty_dir`
- `test_create_baseline_with_files`
- `test_create_baseline_stores_hashes`
- `test_compare_baselines_no_changes`
- `test_compare_baselines_with_improvements`
- `test_compare_baselines_with_regressions`
- `test_compare_baselines_with_new_files`
- `test_compare_baselines_with_removed_files`
- `test_update_baseline_single_file`
- `test_baseline_serialization`
- `test_baseline_summary_calculations`
- `test_baseline_with_git_context`
- `test_baseline_deduplication_by_hash`
- `test_baseline_parallel_analysis`
- `test_baseline_incremental_update`

**Quality Gates** (12 tests):
- `test_regression_gate_detects_score_drop`
- `test_regression_gate_detects_grade_drop`
- `test_regression_gate_allows_improvements`
- `test_minimum_grade_gate_enforces_threshold`
- `test_minimum_grade_gate_allows_higher_grades`
- `test_new_file_gate_enforces_quality`
- `test_combined_gates_evaluation`
- `test_gate_configuration_loading`
- `test_gate_failure_messages`
- `test_gate_tolerance_settings`
- `test_gate_bypass_for_emergencies`
- `test_gate_reporting`

**Git Hooks** (10 tests):
- `test_pre_commit_hook_passes_clean_code`
- `test_pre_commit_hook_blocks_regressions`
- `test_pre_commit_hook_allows_bypass`
- `test_post_commit_hook_updates_baseline`
- `test_hook_install_creates_scripts`
- `test_hook_uninstall_removes_scripts`
- `test_hook_status_shows_state`
- `test_hook_configuration_loading`
- `test_hook_staged_files_detection`
- `test_hook_error_handling`

### Integration Tests (5 total)

- `test_full_workflow_create_baseline_make_changes_compare`
- `test_ci_cd_workflow_simulation`
- `test_multi_developer_workflow`
- `test_merge_conflict_baseline_resolution`
- `test_large_codebase_performance`

---

## User Experience

### Happy Path

```bash
# 1. Initialize project with TDG enforcement
$ pmat hooks install --tdg-enforcement
✅ Installed TDG quality gates
   - Pre-commit: Check for regressions
   - Post-commit: Update baseline
   Configuration: .pmat/tdg-rules.toml

# 2. Create initial baseline
$ pmat tdg baseline create --path .
🔍 Analyzing 150 files...
✅ Baseline created: .pmat/baseline.json
   Average score: 88.2 (A-)
   Grade distribution: A+:20, A:50, B+:40, B:30, C:10

# 3. Make code changes
$ vim src/api.rs
$ git add src/api.rs
$ git commit -m "Refactor API module"

# Pre-commit hook runs automatically:
🔍 Running TDG quality gates...
  Analyzing src/api.rs...
  Overall Score: 92.1 (A)
✅ TDG quality gates passed

# Post-commit hook runs:
📊 Updating TDG baseline...
✅ TDG baseline updated

# 4. Compare against baseline
$ pmat tdg baseline compare
✅ Improved: 1 file
   - src/api.rs: B+ (85.5) → A (92.1) [+6.6]
➡️  Unchanged: 149 files
```

### Regression Detection

```bash
# Make change that degrades quality
$ vim src/database.rs  # Introduce complex code
$ git add src/database.rs
$ git commit -m "Add feature"

# Pre-commit hook catches regression:
🔍 Running TDG quality gates...
  Analyzing src/database.rs...
  Overall Score: 72.3 (B)
❌ TDG quality regression detected
   - src/database.rs: A (91.0) → B (72.3) [-18.7]

   Options:
   1. Refactor to improve quality
   2. Override with: git commit --no-verify
   3. Update baseline: pmat tdg baseline update --accept-regression

# Developer refactors
$ vim src/database.rs  # Simplify
$ git add src/database.rs
$ git commit -m "Add feature (refactored)"
✅ TDG quality gates passed
```

---

## Configuration

### Project Configuration

```toml
# .pmat/tdg-rules.toml
[quality_gates]
mode = "strict"  # strict | warning | disabled

[quality_gates.thresholds]
min_grade_rust = "B+"
min_grade_typescript = "B+"
min_grade_python = "B"

[quality_gates.regression]
max_score_drop = 5.0
allow_grade_drop = false
tolerance_for_large_refactors = 10.0  # If >500 lines changed

[baseline]
path = ".pmat/baseline.json"
auto_update = true
track_in_git = true

[ci_cd]
enabled = true
fail_on_regression = true
comment_on_pr = true
```

---

## Benefits

### For Developers
- **Immediate Feedback**: Know quality instantly on commit
- **Prevent Regressions**: Can't accidentally degrade quality
- **Quality Tracking**: See improvements over time
- **CI/CD Integration**: Automated quality validation

### For Teams
- **Consistent Standards**: Everyone held to same quality bar
- **Quality Trends**: Track team quality over sprints
- **Code Review**: Quality scores in PR comments
- **Technical Debt Visibility**: Quantify debt accumulation

### For Organizations
- **Risk Mitigation**: Prevent quality degradation
- **Compliance**: Automated quality auditing
- **Metrics**: Track quality across projects
- **ROI**: Reduce bug fix costs via prevention

---

## Rollout Plan

### Phase 1: Internal Dogfooding (Week 1)
- Install on PMAT project itself
- Create baseline
- Enable git hooks
- Monitor for 1 week
- Document issues/improvements

### Phase 2: Beta Release (Week 2)
- Release as experimental feature
- Document in pmat-book
- Create tutorial video
- Gather user feedback

### Phase 3: Production Release (Week 3)
- Stabilize based on feedback
- Add to default installation
- Update documentation
- Announce feature

---

## Success Metrics

- **Adoption**: >50% of PMAT users enable TDG enforcement
- **Quality**: Average project TDG score improves by >5 points
- **Regressions**: <1% of commits degrade quality
- **Performance**: Baseline comparison <1 second for 1000 files
- **Satisfaction**: >80% user satisfaction with feature

---

## Next Steps

1. ✅ Review this specification
2. ⏳ Implement Phase 1 (Baseline System)
3. ⏳ Implement Phase 2 (Quality Gates)
4. ⏳ Implement Phase 3 (Git Hooks)
5. ⏳ Implement Phase 4 (CI/CD Templates)
6. ⏳ Dogfood on PMAT
7. ⏳ Release v2.180.0 with TDG Enforcement

---

**Status**: 🔄 Specification Complete - Ready for Implementation

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
