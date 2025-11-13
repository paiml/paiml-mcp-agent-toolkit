# PMAT-UX-001: Progress Reporting for Red Team & Repo Score

**Sprint**: 45
**Status**: 🟢 GREEN PHASE (75% Complete)
**Estimated**: 2-3 hours
**Actual**: ~2 hours (in progress)

**Progress**:
- ✅ RED Phase (15 tests created - commit 913c6858)
- ✅ GREEN Phase Infrastructure (commit 485e6e54)
- ✅ Red Team CLI Integration (commit eaa3a669)
- ✅ Repo Score CLI Integration (commit 36e8cc5a)
- ⏳ Test Verification (tests compiling)
- ⏳ REFACTOR Phase (performance optimization, UX polish)

## 🎯 Objective

Add real-time progress reporting to `pmat red-team` and `pmat repo-score` commands to provide visibility into long-running repository analysis operations.

## 📋 Problem Statement

**Current Behavior:**
- `pmat red-team analyze` and `pmat repo-score` commands run silently during analysis
- Users have no feedback during multi-file/multi-commit analysis
- Operations >10s feel unresponsive, leading to premature cancellation
- No indication of progress through large repositories

**User Pain Points:**
```bash
# Current behavior - no feedback
$ pmat repo-score --path .
# ... 30 seconds of silence ...
Repository Score: 85/100

# Current behavior - no feedback
$ pmat red-team analyze --message "feat: Complete"
# ... 15 seconds of silence ...
✅ All claims verified
```

## 🎯 Requirements

### Red Team Progress Reporting

**Must Support:**
- File-level progress when analyzing git history
- Commit-level progress when scanning commit messages
- Evidence source progress (8 sources: GitHistory, TestExecution, CoverageReport, etc.)
- ETA estimation for large repositories

**Example Output:**
```bash
$ pmat red-team analyze --message "feat: All tests passing" --verbose

🔴 Red Team Mode: Analyzing commit message
📝 Message: feat: All tests passing

⠋ Extracting claims... (1/3)
✓ Found 1 testable claim

⠋ Gathering evidence... (2/3)
  ⠋ GitHistory: Checking commit history... [=====>    ] 50%
  ✓ GitHistory: 0 rollbacks found (0.5s)
  ⠋ TestExecution: Running test suite... [=======>  ] 75%
  ✓ TestExecution: 5 tests ignored (1.2s)
  ⠋ CoverageReport: Analyzing coverage... [=========] 100%
  ✓ CoverageReport: 85% coverage (0.8s)

⠋ Analyzing results... (3/3)
✓ Analysis complete (2.5s total)

🔴 HALLUCINATION DETECTED
...
```

### Repo Score Progress Reporting

**Must Support:**
- Category-level progress (6 categories: Code Quality, Testing, Documentation, etc.)
- File-level progress within each category
- Percentage complete for overall analysis
- Time elapsed and ETA

**Example Output:**
```bash
$ pmat repo-score --path . --verbose

🔍 Repository Health Score Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⠋ Analyzing repository structure...
✓ Found 1,234 files (0.1s)

⠋ Category 1/6: Code Quality
  ⠋ Analyzing complexity... [=====>    ] 150/300 files (50%)
  ✓ Complexity score: 78/100 (4.2s)
  ⠋ Analyzing duplication... [=========>] 280/300 files (93%)
  ✓ Duplication score: 92/100 (2.8s)

⠋ Category 2/6: Testing
  ⠋ Analyzing test coverage... [==>       ] 50/300 files (17%)
  ...

Overall: [=========>        ] 45% complete (8.5s elapsed, ~11s remaining)

✓ Analysis complete (19.2s total)

Repository Score: 85/100 (Grade: B)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Common Requirements

**Progress Indicator Features:**
- Spinner animation during indeterminate operations
- Progress bar with percentage for deterministic operations
- Step indicators (e.g., "2/5") for multi-stage operations
- Time elapsed display
- ETA calculation for long operations
- CI/TTY detection (disable in non-interactive environments)
- Respect `--quiet` flag (no progress in quiet mode)
- Respect `NO_COLOR` environment variable
- Graceful degradation in non-TTY environments

## 🔴 RED Phase: Tests First

### Test Suite Location
- `server/tests/progress_reporting_tests.rs`

### Red Team Progress Tests

```rust
#[test]
fn test_red_team_shows_progress_in_tty() {
    // Mock TTY environment
    std::env::set_var("TERM", "xterm-256color");

    let mut cmd = Command::new("pmat");
    cmd.args(&["red-team", "analyze", "--message", "feat: Test", "--verbose"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show progress indicators
    assert!(stdout.contains("⠋") || stdout.contains("Extracting claims"));
    assert!(stdout.contains("Gathering evidence"));
}

#[test]
fn test_red_team_no_progress_in_quiet_mode() {
    let mut cmd = Command::new("pmat");
    cmd.args(&["red-team", "analyze", "--message", "feat: Test", "--quiet"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should NOT show progress indicators
    assert!(!stdout.contains("⠋"));
    assert!(!stdout.contains("Extracting claims"));
}

#[test]
fn test_red_team_progress_updates() {
    // Test that progress updates as work completes
    let progress = RedTeamProgress::new();

    progress.set_stage(Stage::ExtractingClaims);
    assert_eq!(progress.current_stage(), "Extracting claims");

    progress.set_stage(Stage::GatheringEvidence);
    assert_eq!(progress.current_stage(), "Gathering evidence");
}
```

### Repo Score Progress Tests

```rust
#[test]
fn test_repo_score_shows_category_progress() {
    std::env::set_var("TERM", "xterm-256color");

    let mut cmd = Command::new("pmat");
    cmd.args(&["repo-score", "--path", ".", "--verbose"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show category progress
    assert!(stdout.contains("Category 1/6") || stdout.contains("Code Quality"));
    assert!(stdout.contains("Category 2/6") || stdout.contains("Testing"));
}

#[test]
fn test_repo_score_shows_file_progress() {
    let mut cmd = Command::new("pmat");
    cmd.args(&["repo-score", "--path", ".", "--verbose"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show file-level progress
    assert!(stdout.contains("files") || stdout.contains("["));
    assert!(stdout.contains("%") || stdout.contains("complete"));
}

#[test]
fn test_progress_respects_ci_environment() {
    std::env::set_var("CI", "true");

    let progress = ProgressIndicator::new("Testing");

    // Should not show progress in CI
    assert!(!progress.is_enabled());
}
```

**Total Tests**: 15
- Red Team progress: 5 tests
- Repo Score progress: 5 tests
- Progress indicator infrastructure: 5 tests

## 🟢 GREEN Phase: Implementation

### Phase 1: Infrastructure (30 minutes)

**File**: `server/src/cli/progress.rs` (extend existing)

**New Functions:**
```rust
/// Multi-stage progress indicator with ETA
pub struct MultiStageProgress {
    stages: Vec<String>,
    current_stage: usize,
    pb: Option<ProgressBar>,
    start_time: Instant,
}

impl MultiStageProgress {
    pub fn new(stages: Vec<String>) -> Self;
    pub fn next_stage(&mut self, message: &str);
    pub fn set_progress(&mut self, current: u64, total: u64);
    pub fn finish(&self, message: &str);
    pub fn get_eta(&self) -> Duration;
}

/// Category-based progress (for repo-score)
pub struct CategoryProgress {
    categories: Vec<String>,
    current_category: usize,
    pb: Option<ProgressBar>,
}

impl CategoryProgress {
    pub fn new(categories: Vec<String>) -> Self;
    pub fn next_category(&mut self, name: &str);
    pub fn set_file_progress(&mut self, current: usize, total: usize);
    pub fn finish(&self);
}
```

### Phase 2: Red Team Integration (45 minutes)

**File**: `server/src/cli/handlers/red_team.rs`

**Changes:**
```rust
impl RedTeamCmd {
    pub fn execute(&self) -> anyhow::Result<ExitCode> {
        match &self.command {
            RedTeamCommands::Analyze { message, path, format, verbose } => {
                // Create multi-stage progress
                let progress = if *verbose {
                    Some(MultiStageProgress::new(vec![
                        "Extracting claims".to_string(),
                        "Gathering evidence".to_string(),
                        "Analyzing results".to_string(),
                    ]))
                } else {
                    None
                };

                // Stage 1: Extract claims
                if let Some(ref p) = progress {
                    p.set_message("Extracting claims...");
                }
                let handler = RedTeamHandler::new();
                let context = RepositoryContext::new_mock();

                // Stage 2: Gather evidence
                if let Some(ref p) = progress {
                    p.next_stage("Gathering evidence...");
                }
                let result = handler.analyze_commit_message(message, &context);

                // Stage 3: Analyze
                if let Some(ref p) = progress {
                    p.next_stage("Analyzing results...");
                }

                // Finish
                if let Some(ref p) = progress {
                    p.finish("Analysis complete");
                }

                // ... format output ...
            }
        }
    }
}
```

**File**: `server/src/red_team/evidence_gatherer.rs`

**Add progress callbacks:**
```rust
pub struct EvidenceGatherer {
    // ... existing fields ...
    progress_callback: Option<Box<dyn Fn(&str, f32)>>,
}

impl EvidenceGatherer {
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, f32) + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    pub fn gather_evidence(&self, claim: &Claim, context: &RepositoryContext) -> Vec<EvidenceResult> {
        let total_sources = 8;
        let mut results = Vec::new();

        for (idx, source) in EVIDENCE_SOURCES.iter().enumerate() {
            if let Some(ref cb) = self.progress_callback {
                cb(source.name(), (idx as f32 / total_sources as f32));
            }

            // ... gather evidence from source ...

            results.push(result);
        }

        results
    }
}
```

### Phase 3: Repo Score Integration (45 minutes)

**File**: `server/src/cli/handlers/repo_score_handlers.rs`

**Changes:**
```rust
pub async fn handle_repo_score(
    path: &Path,
    format: RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
    update_badge: bool,
) -> Result<()> {
    // Create category progress
    let progress = if verbose {
        Some(CategoryProgress::new(vec![
            "Code Quality".to_string(),
            "Testing".to_string(),
            "Documentation".to_string(),
            "Security".to_string(),
            "Performance".to_string(),
            "Maintainability".to_string(),
        ]))
    } else {
        None
    };

    // Run scoring with progress
    let aggregator = ScoreAggregator::new();

    if let Some(ref p) = progress {
        aggregator.set_progress_callback(move |category, current, total| {
            p.set_file_progress(current, total);
        });
    }

    let score = aggregator
        .aggregate(path, &config)
        .await
        .context("Failed to calculate repository score")?;

    if let Some(ref p) = progress {
        p.finish();
    }

    // ... format output ...
}
```

**File**: `server/src/services/repo_score/aggregator.rs`

**Add progress support:**
```rust
pub struct ScoreAggregator {
    // ... existing fields ...
    progress_callback: Option<Box<dyn Fn(&str, usize, usize) + Send + Sync>>,
}

impl ScoreAggregator {
    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, usize, usize) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
    }

    pub async fn aggregate(&self, path: &Path, config: &ScorerConfig) -> Result<RepoScore> {
        let categories = vec![
            "Code Quality",
            "Testing",
            "Documentation",
            "Security",
            "Performance",
            "Maintainability",
        ];

        for (idx, category) in categories.iter().enumerate() {
            if let Some(ref cb) = self.progress_callback {
                cb(category, idx, categories.len());
            }

            // ... run category scoring ...
        }

        // ... aggregate results ...
    }
}
```

## 🔵 REFACTOR Phase: Quality

### Performance
- Progress updates should not slow down analysis >5%
- Use async callbacks to avoid blocking
- Batch progress updates (max 10 updates/second)

### UX Polish
- Use colored output (respecting NO_COLOR)
- Add Unicode spinner animations (fallback to ASCII)
- Show ETA for operations >10 seconds
- Show time elapsed for completed operations
- Clear progress indicators on completion

### Testing
- Test TTY detection
- Test CI environment detection
- Test quiet mode suppression
- Test progress accuracy
- Test ETA calculation

## 📊 Success Criteria

**Functional:**
- ✅ Progress indicators appear in TTY mode
- ✅ No progress in CI/quiet mode
- ✅ Accurate progress percentages
- ✅ ETA updates in real-time
- ✅ All existing tests pass
- ✅ 15 new progress tests pass

**Non-Functional:**
- ✅ Progress overhead <5% of total runtime
- ✅ Progress updates smooth (no flickering)
- ✅ Graceful degradation in non-TTY
- ✅ Respects accessibility (NO_COLOR)

## 🔗 Related Work

**Leverage Existing:**
- `server/src/cli/progress.rs` - ProgressIndicator infrastructure
- `indicatif` crate - Already in dependencies
- TTY detection logic - Already implemented

**Reference Implementations:**
- `server/src/cli/handlers/health_handler.rs` - Uses ProgressIndicator
- `server/src/cli/handlers/generation_handlers.rs` - Progress examples

**Environment Variables:**
- `CI` - Disable progress in CI
- `NO_COLOR` - Disable colors
- `PMAT_QUIET` - Quiet mode

## 📝 Implementation Notes

### Phase Breakdown

**Phase 1: Infrastructure (30 min)**
- Extend `progress.rs` with MultiStageProgress and CategoryProgress
- Add unit tests for new progress types

**Phase 2: Red Team (45 min)**
- Integrate progress into RedTeamCmd::execute()
- Add progress callbacks to EvidenceGatherer
- Test with verbose mode

**Phase 3: Repo Score (45 min)**
- Integrate progress into handle_repo_score()
- Add progress callbacks to ScoreAggregator
- Test with large repositories

**Phase 4: Polish & Test (30 min)**
- Add ETA calculation
- Add elapsed time display
- Run full test suite
- Manual testing on large repos

### Future Enhancements (Out of Scope)

- Interactive progress (user can pause/resume)
- Progress persistence (resume interrupted operations)
- Detailed sub-task breakdown
- Progress API endpoint for web UI
- JSON progress output for tooling integration

## 🎯 Definition of Done

- [ ] All 15 tests passing
- [ ] CI/CD pipeline green
- [ ] Progress visible in TTY mode
- [ ] No progress in CI/quiet mode
- [ ] Manual testing on 3 different repos
- [ ] Code review approved
- [ ] Documentation updated
- [ ] User feedback collected

## 📚 References

- **Existing Progress**: `server/src/cli/progress.rs`
- **Indicatif Docs**: https://docs.rs/indicatif/latest/indicatif/
- **TTY Detection**: TICKET-PMAT-6006 (UX improvements)
- **Red Team Mode**: `server/src/cli/handlers/red_team.rs`
- **Repo Score**: `server/src/cli/handlers/repo_score_handlers.rs`
