# Repository Health Scoring Adjustments

**Status**: Draft Specification
**Created**: 2025-11-11
**Author**: Claude Code (AI Assistant)
**Purpose**: Address false positives and improve accuracy of `pmat repo-score` hygiene scoring

---

## Executive Summary

During real-world testing of the `pmat repo-score` command, we discovered that the **Repository Hygiene** scorer produces false positives by penalizing repositories for having build artifacts (e.g., `target/`) during active development, even when these files are properly gitignored.

**Current Behavior**: Repository scores **5.0/10.0** (50%) on hygiene despite being **git-clean** with all cruft patterns properly configured in `.gitignore`.

**Root Cause**: Hygiene scorer scans physical filesystem without respecting `.gitignore`, conflating "git-clean" with "filesystem-clean".

**Impact**: Misleading scores for actively developed repositories, reducing trust in the scoring system.

---

## Current Implementation Analysis

### Hygiene Scorer Behavior (server/src/services/repo_score/scorers/hygiene_scorer.rs)

**C1: No Cruft Files (5 points)**
```rust
let cruft_patterns = vec![
    // Build artifacts
    "target/", "dist/", "build/", "out/", "*.pyc", "__pycache__/",
    "node_modules/", ".next/", ".cache/",
    // Temp files
    "*.tmp", "*.swp", "*.swo", "*~",
    // OS files
    ".DS_Store", "Thumbs.db", "desktop.ini",
    // Editor backups
    "*.bak", "*.orig",
];
```

**Scanning Logic**:
```rust
for entry in WalkDir::new(repo_path)
    .max_depth(5)
    .into_iter()
{
    // Scans physical filesystem
    // Does NOT check .gitignore
    // Does NOT check git tracking status
}
```

**C2: No Team-Specific Files (5 points)**
- Scans for `.idea/`, `.vscode/`, etc.
- Same issue: ignores .gitignore status

### Real-World Test Results

**Test Case**: PMAT repository (this codebase)

**Git Status**:
```bash
$ git status
On branch master
nothing to commit, working tree clean
```

**Score Result**:
```
❌ Repository Hygiene: 5.0/10.0 (50%)
     • Cruft file found: ./target/release/build/zstd-safe-*/build_script_build-*
     • Cruft file found: ./target/release/build/syn-*/output
     ... (10 files from target/ directory)
```

**Analysis**:
- Repository is **git-clean** (100% compliant)
- All cruft patterns are in `.gitignore` (lines 11-12, 68, 83, 180-183)
- Build artifacts exist during active development (expected)
- Score penalizes normal development workflow

---

## Problem Statement

### False Positive Scenarios

1. **Active Development**
   - Developer runs `cargo build` → creates `target/`
   - Repo scores 50% hygiene despite perfect git hygiene
   - Score would be 100% after `cargo clean`, then 50% after next build

2. **CI/CD Environments**
   - CI builds create temporary artifacts
   - Repo scores poorly during CI run
   - Same repo scores perfectly on fresh clone

3. **Mutation Testing**
   - `mutants.out/` directory created during testing
   - Already gitignored (line 180-183)
   - Penalized anyway

4. **Coverage Analysis**
   - `target/llvm-cov*/` created during `make coverage`
   - Already gitignored (line 130-133)
   - Penalized anyway

### User Experience Issues

1. **Confusing Messaging**
   - Recommendation: "Remove cruft files (.tmp, .bak) and team-specific files (.idea/, .vscode/). Add them to .gitignore."
   - Reality: Already in .gitignore, files are properly ignored
   - User action: None needed, but score says otherwise

2. **Score Volatility**
   - Score changes based on build state, not code quality
   - `git clean -fdx` → 100% hygiene
   - `cargo build` → 50% hygiene
   - Undermines trust in scoring system

3. **Misleading Priorities**
   - Hygiene score suggests urgent fixes needed
   - No actual repository health issue exists
   - Wastes developer time investigating false positives

---

## Proposed Solutions

### Solution 1: Respect .gitignore (Recommended)

**Approach**: Check if files are gitignored before scoring.

**Implementation**:
```rust
use ignore::WalkBuilder;

// Replace WalkDir with ignore::WalkBuilder
for entry in WalkBuilder::new(repo_path)
    .max_depth(Some(5))
    .hidden(false)           // Don't auto-skip hidden files
    .parents(true)           // Respect parent .gitignore files
    .ignore(true)            // Respect .gitignore
    .git_ignore(true)        // Respect .git/info/exclude
    .git_global(true)        // Respect global gitignore
    .git_exclude(true)       // Respect .git/info/exclude
    .build()
{
    // Only scans git-tracked or not-ignored files
}
```

**Pros**:
- Eliminates false positives for gitignored files
- Aligns with git workflow (if it's ignored, it doesn't matter)
- Respects developer's explicit ignore patterns
- Score matches git-clean status

**Cons**:
- Requires new dependency: `ignore` crate (already used in codebase for other features)
- Won't catch gitignored files that *should* be tracked but aren't

**Risk**: Low (dependency already in use, well-maintained by ripgrep author)

### Solution 2: Dual Scoring (Alternative)

**Approach**: Provide two separate hygiene scores.

**Implementation**:
```rust
pub struct HygieneScore {
    pub git_tracked_score: f64,      // Only git-tracked files
    pub filesystem_score: f64,        // All files (current behavior)
    pub total_score: f64,             // git_tracked_score (primary)
}
```

**Output**:
```
✅ Repository Hygiene (Git-Tracked)    10.0/10.0 (100%)
     • No cruft in version control
⚠️ Repository Hygiene (Filesystem)     5.0/10.0 (50%)
     • Build artifacts present (target/)
     • Note: Ignored by git, safe during development
```

**Pros**:
- Provides both perspectives
- Helps distinguish development vs production readiness
- Educational for users

**Cons**:
- More complex output
- Two scores might confuse users
- Requires UI/UX design decisions

**Risk**: Medium (complexity, user confusion)

### Solution 3: Build Directory Exclusion (Quick Fix)

**Approach**: Hardcode exclusions for common build directories.

**Implementation**:
```rust
fn should_skip_directory(path: &Path) -> bool {
    let exclusions = [
        "target",           // Rust
        "node_modules",     // Node.js
        "dist",             // General build output
        "build",            // General build output
        ".next",            // Next.js
        "__pycache__",      // Python
        "mutants.out",      // Mutation testing
        "coverage",         // Coverage output
    ];

    path.components()
        .any(|c| exclusions.contains(&c.as_os_str().to_str().unwrap_or("")))
}
```

**Pros**:
- Quick to implement
- No new dependencies
- Solves 90% of false positives

**Cons**:
- Not comprehensive (can't cover all build systems)
- Hardcoded list needs maintenance
- Doesn't respect project-specific ignore patterns
- Band-aid solution

**Risk**: Low (simple implementation, immediate improvement)

### Solution 4: Configuration Option (User Control)

**Approach**: Let users choose scanning mode.

**Implementation**:
```bash
# Default: respect gitignore
pmat repo-score --path .

# Scan filesystem (strict mode, current behavior)
pmat repo-score --path . --strict-filesystem

# Only git-tracked files
pmat repo-score --path . --git-tracked-only
```

**Configuration** (`.pmat/repo-score.toml`):
```toml
[hygiene]
scan_mode = "respect-gitignore"  # or "filesystem" or "git-tracked-only"
skip_directories = ["target", "dist", "build"]
```

**Pros**:
- User control over behavior
- Supports different use cases (dev vs CI vs release)
- Backward compatible with flag

**Cons**:
- More configuration complexity
- Requires documentation
- Default behavior decision needed

**Risk**: Low (optional feature, doesn't break existing behavior)

---

## Recommended Implementation Plan

### Phase 1: Quick Win (Sprint 48.1)
**Implement Solution 3 (Build Directory Exclusion)**

**Changes**:
- Add `should_skip_directory()` helper
- Update both C1 and C2 scoring to skip build directories
- Add test case for PMAT repository

**Effort**: 1 hour
**Impact**: Eliminates 90% of false positives immediately
**Risk**: Minimal

**Expected Result**: PMAT repository scores **10.0/10.0 (100%)** on hygiene.

### Phase 2: Proper Solution (Sprint 49)
**Implement Solution 1 (Respect .gitignore)**

**Changes**:
- Replace `WalkDir` with `ignore::WalkBuilder`
- Add integration tests with gitignored files
- Update documentation

**Effort**: 4 hours
**Impact**: Eliminates all false positives for gitignored files
**Risk**: Low (dependency already in use)

**Expected Result**: All repositories with proper .gitignore score accurately.

### Phase 3: Enhancement (Sprint 50)
**Implement Solution 4 (Configuration Option)**

**Changes**:
- Add `--scan-mode` flag
- Add `.pmat/repo-score.toml` support
- Update documentation with use cases

**Effort**: 6 hours
**Impact**: Provides flexibility for different scenarios
**Risk**: Low (additive feature)

**Expected Result**: Users can choose appropriate scanning mode for their workflow.

---

## Testing Strategy

### Unit Tests

**Test Case 1: Gitignored Build Artifacts**
```rust
#[tokio::test]
async fn test_hygiene_respects_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create .gitignore
    fs::write(repo_path.join(".gitignore"), "target/\n*.tmp\n").unwrap();

    // Create gitignored files
    create_file(repo_path, "target/release/libfoo.rlib");
    create_file(repo_path, "test.tmp");

    // Create clean files
    create_file(repo_path, "src/main.rs");

    let scorer = HygieneScorer::new();
    let config = ScorerConfig::default();

    let result = scorer.score(repo_path, &config).await.unwrap();

    // Should score 100% because gitignored files are excluded
    assert_eq!(result.score, 10.0);
    assert_eq!(result.percentage, 100.0);
}
```

**Test Case 2: Build Directory Exclusion**
```rust
#[tokio::test]
async fn test_hygiene_excludes_build_directories() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create files in excluded directories
    create_file(repo_path, "target/debug/libfoo.rlib");
    create_file(repo_path, "node_modules/package/index.js");
    create_file(repo_path, "dist/bundle.js");

    // Create clean files
    create_file(repo_path, "src/main.rs");

    let scorer = HygieneScorer::new();
    let config = ScorerConfig::default();

    let result = scorer.score(repo_path, &config).await.unwrap();

    // Should score 100% because build dirs are excluded
    assert_eq!(result.score, 10.0);
}
```

### Integration Tests

**Test Case 3: PMAT Repository Real-World Test**
```rust
#[test]
fn test_pmat_repository_hygiene_score() {
    let repo_path = std::env::current_dir().unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    let output = cmd
        .args(["repo-score", "--path", repo_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // PMAT repository should score 100% hygiene (git-clean)
    assert!(stdout.contains("✅ Repository Hygiene") ||
            stdout.contains("Repository Hygiene        10.0/10.0"));
}
```

**Test Case 4: Fresh Clone vs Development**
```bash
# Test that fresh clone and development repo have similar scores
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git fresh-clone
cd fresh-clone
pmat repo-score --path . > fresh-score.txt

cd ../paiml-mcp-agent-toolkit
cargo build --release
pmat repo-score --path . > dev-score.txt

# Scores should be identical (both 100% hygiene)
diff fresh-score.txt dev-score.txt
```

### Regression Tests

**Test Case 5: Ensure Real Cruft Still Detected**
```rust
#[tokio::test]
async fn test_hygiene_still_detects_tracked_cruft() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create TRACKED cruft files (not in .gitignore)
    create_file(repo_path, "backup.bak");  // Not gitignored
    create_file(repo_path, "test.tmp");    // Not gitignored

    let scorer = HygieneScorer::new();
    let config = ScorerConfig::default();

    let result = scorer.score(repo_path, &config).await.unwrap();

    // Should lose points for tracked cruft
    assert!(result.score < 10.0);
    assert!(result.findings.iter().any(|f| f.message.contains("Cruft file")));
}
```

---

## Documentation Updates

### User-Facing Documentation

**pmat-book Chapter: Repository Health Scoring**

Add section:

```markdown
### Understanding Hygiene Scores

The Repository Hygiene category checks for cruft and team-specific files.

**Important**: The hygiene scorer respects your `.gitignore` file. Files that are
properly gitignored will NOT be penalized, even if they exist on your filesystem.

#### Expected Behavior During Development

- **Before Build**: 10.0/10.0 (100%) - Clean filesystem
- **After Build**: 10.0/10.0 (100%) - Build artifacts gitignored
- **With Cruft**: < 10.0/10.0 - Tracked cruft files detected

#### What Gets Scored

✅ **Ignored by Hygiene Scorer**:
- Files matching `.gitignore` patterns
- Common build directories (`target/`, `node_modules/`, `dist/`)
- Coverage output (`target/llvm-cov*/`, `coverage/`)
- Mutation testing output (`mutants.out/`)

❌ **Penalized by Hygiene Scorer**:
- Tracked cruft files (`.tmp`, `.bak` not in `.gitignore`)
- Team-specific files (`.idea/`, `.vscode/` tracked in git)
- OS files (`.DS_Store`, `Thumbs.db` tracked in git)

#### Best Practices

1. **Add to .gitignore**: All build artifacts, temp files, IDE configs
2. **Run git status**: Ensure working tree is clean
3. **Score reflects git hygiene**: Not filesystem state
```

### CLI Help Updates

**pmat repo-score --help**

Update description:

```
Repository health scoring (0-110 scale)

Scores your repository across 6 categories + bonus points:
- Documentation (20 pts): README, contributing guides
- Pre-commit Hooks (20 pts): Validation, formatting, linting
- Repository Hygiene (10 pts): No tracked cruft or team files
- Build/Test Automation (25 pts): Makefile, test targets
- Continuous Integration (20 pts): GitHub Actions, workflows
- PMAT Compliance (5 pts): PMAT hooks, validation

Bonus points (10 pts max):
- Property Testing (+3), Fuzzing (+2), Mutation Testing (+2)
- Living Documentation (+3)

Note: Hygiene scoring respects .gitignore. Build artifacts
(target/, node_modules/) are excluded if properly gitignored.
```

---

## Migration Path

### Backward Compatibility

**Current Behavior** (v2.194.0):
- Scans all files (ignores .gitignore)
- PMAT scores 50% hygiene

**New Behavior** (v2.195.0+):
- Respects .gitignore by default
- PMAT scores 100% hygiene

**Breaking Change**: No (improvement, not regression)

**User Impact**: Positive (more accurate scores)

### Rollout Strategy

1. **v2.195.0-beta**: Implement Phase 1 (build directory exclusion)
2. **User Testing**: Community feedback (1 week)
3. **v2.195.0**: Implement Phase 2 (respect .gitignore)
4. **v2.196.0**: Implement Phase 3 (configuration options)

### Communication Plan

**Changelog Entry**:
```markdown
## [2.195.0] - 2025-11-XX

### Fixed
- **repo-score**: Hygiene scorer now respects .gitignore patterns
- **repo-score**: Build directories (target/, node_modules/) excluded by default
- **repo-score**: False positives eliminated for gitignored files

### Changed
- **repo-score**: Hygiene scores now reflect git-tracked file hygiene, not filesystem state
- **repo-score**: More accurate scores for actively developed repositories

### Impact
Repositories with proper .gitignore will now score 100% hygiene (previously 50% during development).
This better reflects actual repository health and git hygiene practices.
```

---

## Metrics and Success Criteria

### Success Metrics

1. **Accuracy**: 100% of git-clean repos score 100% hygiene
2. **False Positive Rate**: < 5% (down from current ~50%)
3. **User Satisfaction**: Positive community feedback
4. **Score Stability**: Score doesn't change with build state

### Monitoring

**After v2.195.0 release**:
- Track community feedback on GitHub issues
- Monitor usage of new flags (if implemented)
- Collect hygiene score distribution (anonymous telemetry)

### Rollback Criteria

If any of these occur, consider rollback:
- False negative rate > 10% (missing real cruft)
- Community feedback overwhelmingly negative
- Performance regression > 2x slowdown

---

## Open Questions

1. **Should we scan symlinks?**
   - Current: Follows symlinks
   - Proposal: Skip symlinks (avoid false positives)
   - Decision needed: Sprint 48.1

2. **Should .git/ directory be excluded?**
   - Current: Excluded implicitly by WalkDir filters
   - Proposal: Explicitly exclude (performance)
   - Decision needed: Sprint 48.1

3. **Should git-lfs files be scored?**
   - Current: Treated as regular files
   - Proposal: Exclude large files (> 10MB)
   - Decision needed: Sprint 49

4. **Should we provide --strict-filesystem flag?**
   - Use case: Pre-deployment checks
   - Value: Catch truly stale artifacts
   - Decision needed: Sprint 50

---

## Related Work

### Similar Tools

**ShellCheck**: Respects pragmas to disable checks
- Lesson: User control over strictness is valuable

**Clippy**: Has `--allow` flags for specific warnings
- Lesson: Configuration reduces false positives

**PyLint**: Respects `.pylintrc` for exclusions
- Lesson: Project-specific config reduces noise

### Industry Standards

**Git Best Practices**:
- If it's in .gitignore, it doesn't exist (from git's perspective)
- Our scoring should align with this principle

**Clean Code**:
- Focus on what's tracked, not ephemeral build artifacts
- Our scoring should prioritize version-controlled hygiene

---

## Conclusion

The current hygiene scorer produces false positives by penalizing repositories for having build artifacts during active development. This undermines trust in the scoring system and wastes developer time.

**Recommended Path Forward**:
1. **Sprint 48.1** (1 hour): Implement build directory exclusion (quick win)
2. **Sprint 49** (4 hours): Respect .gitignore patterns (proper solution)
3. **Sprint 50** (6 hours): Add configuration options (flexibility)

**Expected Outcome**:
- PMAT repository: 50% → 100% hygiene score
- User trust: Restored through accurate scoring
- False positives: Eliminated for gitignored files

---

## Appendix A: Code Examples

### Current Implementation (False Positives)

```rust
// server/src/services/repo_score/scorers/hygiene_scorer.rs:38-66
for entry in WalkDir::new(repo_path)
    .max_depth(5)
    .into_iter()
{
    if let Ok(entry) = entry {
        if entry.depth() > 0 {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.starts_with('.') && file_name != ".gitignore" {
                continue;  // Skip hidden files
            }
        }

        // Problem: This scans ALL files, including gitignored ones
        for pattern in &cruft_patterns {
            if matches_pattern(&path_str, pattern) {
                cruft_found.push(path_str.to_string());
                deductions += 0.5;
                break;
            }
        }
    }
}
```

### Proposed Implementation (Respects .gitignore)

```rust
use ignore::WalkBuilder;

// Build walker that respects .gitignore
let walker = WalkBuilder::new(repo_path)
    .max_depth(Some(5))
    .hidden(false)
    .ignore(true)        // ✅ Respect .gitignore
    .git_ignore(true)    // ✅ Respect .git/info/exclude
    .git_global(true)    // ✅ Respect global gitignore
    .git_exclude(true)
    .build();

for entry in walker {
    if let Ok(entry) = entry {
        // Only scans non-ignored files
        let path = entry.path();
        let path_str = path.to_string_lossy();

        for pattern in &cruft_patterns {
            if matches_pattern(&path_str, pattern) {
                // Will only trigger for tracked cruft
                cruft_found.push(path_str.to_string());
                deductions += 0.5;
                break;
            }
        }
    }
}
```

---

## Appendix B: Real-World Test Results

### Before Fix (v2.194.0)

```bash
$ git status
On branch master
nothing to commit, working tree clean

$ pmat repo-score --path .
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  Repository Health Score
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📌  Summary
  Total Score:  92.0/100
  Bonus Points: 7.0/10
  Final Score:  99.0/110
  Grade:        A+

📂  Categories
  ✅ Documentation             20.0/20.0 (100.0%)
  ⚠️ Pre-commit Hooks          17.0/20.0 (85.0%)
  ❌ Repository Hygiene        5.0/10.0 (50.0%)   ← FALSE POSITIVE
  ✅ Build/Test Automation     25.0/25.0 (100.0%)
  ✅ Continuous Integration    20.0/20.0 (100.0%)
  ✅ PMAT Compliance           5.0/5.0 (100.0%)

💡  Recommendations
  🟡 Repository Hygiene: Remove cruft files (.tmp, .bak) and
     team-specific files (.idea/, .vscode/). Add them to .gitignore.

  ← Misleading: Already in .gitignore!
```

### After Fix (v2.195.0 Expected)

```bash
$ git status
On branch master
nothing to commit, working tree clean

$ pmat repo-score --path .
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  Repository Health Score
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📌  Summary
  Total Score:  97.0/100
  Bonus Points: 7.0/10
  Final Score:  104.0/110
  Grade:        A+

📂  Categories
  ✅ Documentation             20.0/20.0 (100.0%)
  ⚠️ Pre-commit Hooks          17.0/20.0 (85.0%)
  ✅ Repository Hygiene        10.0/10.0 (100.0%)  ← FIXED!
  ✅ Build/Test Automation     25.0/25.0 (100.0%)
  ✅ Continuous Integration    20.0/20.0 (100.0%)
  ✅ PMAT Compliance           5.0/5.0 (100.0%)

💡  Recommendations
  🟢 Bonus: Living Documentation: Set up mdBook for living
     documentation (+3 bonus points)
```

**Improvement**: 99/110 → 104/110 (+5 points)
**Hygiene**: 50% → 100% (false positive eliminated)

---

**Document Status**: Draft
**Next Steps**: Review with team, approve implementation plan
**Target Release**: v2.195.0 (Sprint 48.1)
