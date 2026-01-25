# PMAT Work Contract: Popperian Falsification-Based Quality Enforcement

**Status**: PROPOSED
**Author**: Claude Code (Refined by Dr. Karl Popper)
**Date**: 2026-01-25
**Version**: 1.1.0

## Executive Summary

This specification addresses critical quality regression issues in `pmat work` by implementing a **Popperian falsification-based contract system**. Every claim made by `pmat work complete` becomes falsifiable, and ANY successful falsification (finding a contradiction) blocks completion. To prevent "Immunizing Stratagems" (ad hoc overrides), all exceptions require a linked debt ticket.

## Problem Statement

### Current Issues

1. **TDG Regression Allowed**: Large files grow, quality scores dip without blocking
2. **Coverage Gaming**: CUDA/AVX files hidden via `#[cfg(not(coverage))]` or exclusion
3. **No Absolute Thresholds**: Coverage is trend-based (relative), not threshold-based (95%)
4. **Warnings-Only Falsification**: Popper checks report but don't block
5. **Optional Spec/Roadmap**: Documentation updates not enforced
6. **No GitHub Sync**: Changes not automatically pushed
7. **Ad Hoc Overrides**: Developers can bypass checks without accountability
8. **Supply Chain Blindness**: No checks for vulnerable dependencies added during work

### Evidence of Gaming Vectors

| Gaming Technique | Current Detection | Impact |
|------------------|-------------------|--------|
| `#[cfg(not(coverage))]` on CUDA code | None | Coverage inflated |
| Moving code to `#[ignore]` modules | None | Complexity hidden |
| Excluding files in `.codecov.yml` | None | Coverage inflated |
| Deleting tests to "fix" failures | None | Quality regression |
| Not updating spec/roadmap | None | Documentation drift |
| Modifying the falsifier itself | None | Silent failure of checks |

## Five Whys Root Cause Analysis

### Issue: `pmat work` allows quality regression

| Level | Why | Finding |
|-------|-----|---------|
| 1 | Quality regression allowed | Falsification is warnings-only |
| 2 | Warnings-only design | No absolute threshold (95%) enforcement |
| 3 | No threshold enforcement | No baseline capture at work start |
| 4 | No baseline capture | No anti-gaming detection |
| **5** | **ROOT CAUSE** | **Architecture is verification-based, not falsification-based** |

### Popperian Insight

Karl Popper's demarcation criterion: A claim is only scientific if it can be **falsified**.

Current `pmat work`:
- **Verification-based**: "Did tests pass?" (positive)
- **Should be**: "Can we find ANY evidence tests don't cover the changes?" (falsification)

## Solution: PMAT Work Contract

### Core Principle

> Every output of `pmat work complete` is a **CLAIM** that must be **FALSIFIABLE**.
> If ANY falsification succeeds (finds a contradiction), the claim is INVALID and work is BLOCKED.

### Contract Structure

```rust
/// Popperian Work Contract
///
/// Every claim made by `pmat work complete` must be falsifiable.
/// If ANY claim cannot be verified, work is BLOCKED.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkContract {
    // === IDENTITY ===
    pub work_item_id: String,
    pub created_at: DateTime<Utc>,

    // === BASELINE (captured at work start, immutable via git) ===
    pub baseline_commit: String,          // Git SHA for tamper-proof baseline
    pub baseline_tdg: f64,
    pub baseline_coverage: f64,
    pub baseline_rust_score: Option<f64>,
    pub baseline_file_manifest: FileManifest,

    // === THRESHOLDS (non-negotiable) ===
    pub thresholds: ContractThresholds,

    // === FALSIFICATION CLAIMS ===
    pub claims: Vec<FalsifiableClaim>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractThresholds {
    /// Minimum total coverage (absolute, not relative)
    pub min_coverage_pct: f64,           // Default: 95.0

    /// Maximum allowed TDG regression (0 = no regression)
    pub max_tdg_regression: f64,         // Default: 0.0

    /// Maximum cyclomatic complexity per function
    pub max_function_complexity: u32,    // Default: 20

    /// Maximum file size in lines
    pub max_file_lines: usize,           // Default: 500

    /// Minimum spec score for completion
    pub min_spec_score: u32,             // Default: 95

    /// Require GitHub push on completion
    pub require_github_sync: bool,       // Default: true

    /// Require spec update for feature work
    pub require_spec_update: bool,       // Default: true

    /// Require roadmap update (BLOCKING)
    pub require_roadmap_update: bool,    // Default: true
}

impl Default for ContractThresholds {
    fn default() -> Self {
        Self {
            min_coverage_pct: 95.0,
            max_tdg_regression: 0.0,
            max_function_complexity: 20,
            max_file_lines: 500,
            min_spec_score: 95,
            require_github_sync: true,
            require_spec_update: true,
            require_roadmap_update: true, // Now mandatory
        }
    }
}
```

### File Manifest (Anti-Gaming)

```rust
/// Immutable file manifest captured at work start
///
/// Detects file hiding/exclusion gaming by tracking ALL source files.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileManifest {
    /// All source files with metadata
    pub files: HashMap<PathBuf, FileEntry>,

    /// Files that MUST be included in coverage
    pub coverage_required: Vec<PathBuf>,

    /// Checksum of entire manifest (tamper detection)
    pub manifest_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    /// SHA256 of file content at baseline
    pub content_hash: String,

    /// Line count at baseline
    pub lines: usize,

    /// Function count at baseline
    pub functions: usize,

    /// Maximum complexity at baseline
    pub max_complexity: u32,

    /// File category for coverage requirements
    pub category: FileCategory,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileCategory {
    /// Standard Rust code - must be covered
    RustSource,
    /// CUDA kernels - must be covered (no hiding allowed)
    CudaKernel,
    /// SIMD/AVX code - must be covered (no hiding allowed)
    SimdAvx,
    /// Test code - excluded from coverage
    TestCode,
    /// Build scripts - optional coverage
    BuildScript,
    /// Generated code - excluded
    Generated,
}
```

### Falsifiable Claims

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct FalsifiableClaim {
    /// Human-readable claim
    pub hypothesis: String,

    /// Method to attempt falsification
    pub falsification_method: FalsificationMethod,

    /// Evidence required to validate
    pub evidence_required: EvidenceType,

    /// Result of falsification attempt
    pub result: Option<FalsificationResult>,

    /// Optional override (requires justification AND ticket)
    pub override_info: Option<OverrideInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverrideInfo {
    pub reason: String,
    pub ticket_id: String, // MANDATORY: Future work to fix debt
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FalsificationMethod {
    /// Try to find files in baseline missing from completion
    ManifestIntegrity,

    /// Try to find uncovered lines in changed code
    DifferentialCoverage,

    /// Try to find total coverage below threshold
    AbsoluteCoverage,

    /// Try to find TDG score regression
    TdgRegression,

    /// Try to find complexity regression
    ComplexityRegression,

    /// Try to find file size regression
    FileSizeRegression,

    /// Try to find spec score below threshold
    SpecQuality,

    /// Try to find roadmap not updated
    RoadmapUpdate,

    /// Try to find unpushed commits
    GitHubSync,

    /// Try to find `#[cfg(not(coverage))]` gaming
    CoverageGaming,

    /// Try to find vulnerable dependencies added
    SupplyChainIntegrity,

    /// Try to find flaws in the falsifier itself (Meta-Check)
    MetaFalsification,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Numeric comparison (actual vs threshold)
    NumericComparison { actual: f64, threshold: f64 },

    /// File list (missing/added/modified)
    FileList(Vec<PathBuf>),

    /// Concrete counter-example (better than boolean)
    CounterExample { details: String },

    /// Git state
    GitState { unpushed_commits: usize, dirty_files: usize },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Did falsification succeed (found a problem)?
    pub falsified: bool,

    /// Evidence that caused falsification
    pub evidence: Option<EvidenceType>,

    /// Human-readable explanation
    pub explanation: String,
}
```

## Workflow Changes

### Phase 1: `pmat work start`

```
pmat work start <id> [--spec <spec-path>]

CURRENT:
  1. Create/load work item
  2. Optional: Create spec template

NEW (Contract Creation):
  1. Create/load work item
  2. Capture immutable baseline:
     a. Run `pmat tdg` → baseline_tdg
     b. Run `make coverage` (fast) → baseline_coverage
     c. Run `pmat rust-project-score` → baseline_rust_score
     d. Generate file manifest (all source files)
  3. Create work contract with claims
  4. Commit baseline to git (immutable):
     git add .pmat-work/<id>/contract.json
     git commit -m "chore(work): baseline for <id>"
  5. Create/update spec (if --spec or feature work)
```

### Phase 2: During Work

```
Developer works on changes...

MONITORING (optional, recommended):
  pmat work status <id>
  → Shows current metrics vs baseline
  → Warns of potential falsification failures
  → Suggests fixes before completion
```

### Phase 3: `pmat work complete`

```
pmat work complete <id>

CURRENT:
  1. Run quality gates (warnings only)
  2. Capture metrics
  3. Mark complete

NEW (Contract Validation):
  1. Load contract from baseline commit (tamper-proof)
  2. Run ALL falsification tests (11 Checks):

     [1/11] Manifest Integrity
            Hypothesis: "All baseline files still exist"
            Falsification: Find files in baseline missing now
            → PASS: All 142 files present
            → FAIL: Missing src/cuda/kernels.cu (BLOCKED)

     [2/11] Meta-Falsification (Self-Test)
            Hypothesis: "The falsifier is active and detecting"
            Falsification: Inject dummy failure, ensure detection
            → PASS: Dummy gaming pattern detected
            → FAIL: Dummy pattern IGNORED (SYSTEM BROKEN - BLOCKED)

     [3/11] Coverage Gaming Detection
            Hypothesis: "No coverage exclusion gaming"
            Falsification: Find `#[cfg(not(coverage))]` patterns
            → PASS: No gaming patterns found
            → FAIL: Found gaming in src/simd/avx.rs:45 (BLOCKED)

     [4/11] Differential Coverage
            Hypothesis: "All changed lines are covered"
            Falsification: Find uncovered lines in git diff
            → PASS: 100% of 234 changed lines covered
            → FAIL: 12 uncovered lines in src/new_feature.rs (BLOCKED)

     [5/11] Absolute Coverage
            Hypothesis: "Total coverage >= 95%"
            Falsification: Measure coverage < 95%
            → PASS: Coverage is 96.2%
            → FAIL: Coverage is 89.1% (BLOCKED)

     [6/11] TDG Regression
            Hypothesis: "TDG score >= baseline"
            Falsification: Find TDG drop
            → PASS: 85.2 >= 84.0 (baseline)
            → FAIL: 79.1 < 84.0 (BLOCKED)

     [7/11] Complexity Regression
            Hypothesis: "No function exceeds complexity 20"
            Falsification: Find function with complexity > 20
            → PASS: Max complexity is 18
            → FAIL: process_data() has complexity 27 (BLOCKED)

     [8/11] Supply Chain Integrity
            Hypothesis: "No vulnerable dependencies added"
            Falsification: Run `cargo deny check`
            → PASS: No bans/advisories
            → FAIL: Found RUSTSEC-2026-001 (BLOCKED)

     [9/11] File Size Regression
            Hypothesis: "No file exceeds 500 lines"
            Falsification: Find file > 500 lines
            → PASS: Largest file is 423 lines
            → FAIL: src/handlers.rs is 612 lines (BLOCKED)

     [10/11] Spec & Roadmap Quality
             Hypothesis: "Spec score >= 95 and Roadmap updated"
             Falsification: Run pmat spec score < 95 OR check roadmap diff
             → PASS: Spec score 97/100, Roadmap updated
             → FAIL: Roadmap unchanged (BLOCKED - Where are we going?)

     [11/11] GitHub Sync
             Hypothesis: "All changes pushed"
             Falsification: Find unpushed commits
             → PASS: All commits pushed
             → FAIL: 3 unpushed commits (BLOCKED)

  3. If ANY falsification succeeds → BLOCK completion
     Print: "Work blocked: <N> falsification(s) found"
     Print each failed claim with evidence
     Print: "Fix issues and retry: pmat work complete <id>"

  4. If ALL falsifications fail (claims hold) → COMPLETE
     Update spec/roadmap
     Push to GitHub
     Mark work item complete
     Print success summary with metrics
```

## Anti-Gaming Detection

### Coverage Gaming Patterns

```rust
/// Detect coverage gaming patterns
pub fn detect_coverage_gaming(project_path: &Path) -> Vec<GamingViolation> {
    let mut violations = Vec::new();

    // Pattern 1: `#[cfg(not(coverage))]` or `#[cfg(not(tarpaulin))]`
    let cfg_patterns = grep_recursive(
        project_path,
        r"#\[cfg\(not\(coverage|tarpaulin|llvm_cov\)\)\]"
    );
    for match_ in cfg_patterns {
        violations.push(GamingViolation {
            file: match_.path,
            line: match_.line,
            pattern: GamingPattern::CfgNotCoverage,
            severity: Severity::Critical,
        });
    }

    // Pattern 2: Suspicious `#[ignore]` on test modules
    let ignore_patterns = grep_recursive(
        project_path,
        r"#\[ignore\].*mod.*test"
    );
    // ... validate if module existed in baseline

    // Pattern 3: `.codecov.yml` exclusions added during work
    let codecov_path = project_path.join(".codecov.yml");
    if codecov_path.exists() {
        let codecov_changed = git_file_changed_since_baseline(&codecov_path);
        if codecov_changed {
            let new_exclusions = diff_codecov_exclusions(baseline, current);
            for exclusion in new_exclusions {
                violations.push(GamingViolation {
                    file: codecov_path.clone(),
                    line: 0,
                    pattern: GamingPattern::NewCodecovExclusion(exclusion),
                    severity: Severity::Critical,
                });
            }
        }
    }

    // Pattern 4: Test deletion (file existed in baseline, gone now)
    for (path, entry) in &baseline_manifest.files {
        if entry.category == FileCategory::TestCode {
            if !path.exists() {
                violations.push(GamingViolation {
                    file: path.clone(),
                    line: 0,
                    pattern: GamingPattern::TestFileDeletion,
                    severity: Severity::Critical,
                });
            }
        }
    }

    violations
}

#[derive(Debug)]
pub enum GamingPattern {
    /// `#[cfg(not(coverage))]` to exclude code
    CfgNotCoverage,
    /// New exclusion added to `.codecov.yml`
    NewCodecovExclusion(String),
    /// Test file deleted during work
    TestFileDeletion,
    /// Test module marked `#[ignore]` during work
    TestModuleIgnored,
    /// CUDA/AVX file removed from manifest
    CriticalFileRemoved,
}
```

### CUDA/AVX File Protection

```rust
/// Detect CUDA/AVX file categories automatically
pub fn categorize_file(path: &Path) -> FileCategory {
    let extension = path.extension().and_then(|e| e.to_str());
    let content = fs::read_to_string(path).unwrap_or_default();

    match extension {
        // CUDA files
        Some("cu" | "cuh") => FileCategory::CudaKernel,

        // Rust files with SIMD
        Some("rs") if contains_simd_patterns(&content) => FileCategory::SimdAvx,

        // Test files
        Some("rs") if is_test_file(path) => FileCategory::TestCode,

        // Regular Rust
        Some("rs") => FileCategory::RustSource,

        // Build scripts
        Some("rs") if path.ends_with("build.rs") => FileCategory::BuildScript,

        _ => FileCategory::Generated,
    }
}

fn contains_simd_patterns(content: &str) -> bool {
    let patterns = [
        "#[target_feature(enable",
        "std::arch::x86_64",
        "std::arch::aarch64",
        "_mm256_", "_mm512_", "_mm_",  // AVX intrinsics
        "vld1q_", "vst1q_",            // NEON intrinsics
        "is_x86_feature_detected!",
    ];
    patterns.iter().any(|p| content.contains(p))
}
```

## Configuration

### Project-Level Configuration

```toml
# .pmat-work.toml (project root)

[contract]
# Override default thresholds
min_coverage_pct = 95.0
max_tdg_regression = 0.0
max_function_complexity = 20
max_file_lines = 500
min_spec_score = 95

[contract.enforcement]
# Which checks are blocking vs warning
manifest_integrity = "block"
coverage_gaming = "block"
differential_coverage = "block"
absolute_coverage = "block"
tdg_regression = "block"
complexity_regression = "block"
file_size_regression = "warn"     # Allow warning for large files
spec_quality = "block"
roadmap_update = "block"          # CHANGED: Now blocking
supply_chain = "block"            # NEW: Blocking
meta_check = "block"              # NEW: Blocking

[contract.coverage]
# Files that MUST be included in coverage (no exclusion allowed)
protected_patterns = [
    "src/**/*.rs",
    "cuda/**/*.cu",
    "simd/**/*.rs",
]

# Files that MAY be excluded
excludable_patterns = [
    "build.rs",
    "benches/**/*.rs",
    "examples/**/*.rs",
]

[contract.gaming_detection]
# Enable/disable specific gaming checks
detect_cfg_not_coverage = true
detect_codecov_changes = true
detect_test_deletion = true
detect_ignore_additions = true
```

## CLI Changes

### New Commands

```bash
# View current contract status
pmat work contract <id>
  → Shows baseline vs current metrics
  → Predicts which falsifications would fail
  → Suggests fixes

# Validate without completing
pmat work validate <id>
  → Runs all falsification tests (including self-test)
  → Reports results without marking complete

# Override threshold (requires justification AND ticket)
pmat work complete <id> \
  --override complexity \
  --reason "Legacy code debt" \
  --ticket "DEBT-123"
  → Allows completion with documented exception
  → Verifies ticket ID format
  → Exception recorded in contract
```

### Updated Commands

```bash
# Start now captures baseline
pmat work start <id> [--spec <path>] [--skip-baseline]
  → --skip-baseline: For quick fixes (still validates on complete)

# Complete now enforces contract
pmat work complete <id> [--force] [--override <check> --reason <text> --ticket <id>]
  → --force: Skip ALL checks (emergency only, requires confirmation)
  → --override: Skip specific check with documented reason & ticket
```

## Adoption via `pmat comply`

To facilitate the transition to this rigorous standard, `pmat comply` will be updated with an upgrade command.

### Command: `pmat comply upgrade`

```bash
pmat comply upgrade --target popperian
```

**Actions:**
1.  **Configuration Injection**: Creates `.pmat-work.toml` with the strict "Block" settings defined above.
2.  **Baseline Capture**: Runs the "Phase 1: Baseline Capture" logic immediately to establish the "Day 0" contract for the repository.
3.  **Debt Recognition**: Scans for existing violations (e.g., coverage < 95%) and automatically creates "Legacy Debt" tickets in `.pmat-tickets/`. This prevents the project from being immediately blocked, converting "errors" into "managed debt".
    *   *Example*: If coverage is 80%, it creates `DEBT-001: Coverage Gap (80% < 95%)` and adds an override to the contract linked to `DEBT-001`.
4.  **Hook Installation**: Installs `pre-push` and `pre-commit` hooks that enforce `pmat work complete`.

**Verification**:
After upgrade, running `pmat comply check` should pass (due to the debt tickets), but `pmat work complete` on *new* work will enforce the strict standard.

## Implementation Phases

### Phase 1: Baseline Capture (Week 1)

1. Add `WorkContract` struct to `work_handlers/`
2. Implement baseline capture at `work start`:
   - TDG score capture
   - Coverage capture (fast mode)
   - File manifest generation
3. Store contract in `.pmat-work/<id>/contract.json`
4. Commit baseline to git

**Deliverables**:
- `src/cli/handlers/work_contract.rs`
- `src/services/file_manifest.rs`
- Tests for baseline capture

### Phase 2: Falsification Framework (Week 2)

1. Implement `FalsifiableClaim` system
2. Implement each falsification method:
   - ManifestIntegrity
   - DifferentialCoverage
   - AbsoluteCoverage
   - TdgRegression
   - ComplexityRegression
   - SupplyChainIntegrity (New)
   - MetaFalsification (New)
3. Integrate into `work complete`

**Deliverables**:
- `src/cli/handlers/work_falsification.rs`
- Tests for each falsification method

### Phase 3: Anti-Gaming Detection (Week 3)

1. Implement gaming pattern detection:
   - `#[cfg(not(coverage))]` scanner
   - `.codecov.yml` change detector
   - Test deletion detector
2. Implement CUDA/AVX file categorization
3. Add protected file enforcement

**Deliverables**:
- `src/services/gaming_detector.rs`
- `src/services/file_categorizer.rs`
- Tests for gaming detection

### Phase 4: Spec/Roadmap/GitHub Integration (Week 4)

1. Implement mandatory spec update check
2. Implement roadmap update check (Blocking)
3. Implement GitHub sync validation
4. Add override mechanism with ticket validation

**Deliverables**:
- Integration with `spec_handlers`
- Integration with `roadmap_service`
- Override audit logging

## Success Criteria

### Quantitative

| Metric | Target | Measurement |
|--------|--------|-------------|
| Coverage gaming detected | 100% | Test suite with gaming patterns |
| TDG regression blocked | 100% | Test suite with regression scenarios |
| False positive rate | <5% | Real-world usage tracking |
| Baseline capture time | <30s | Benchmark on large projects |
| Falsification time | <60s | Benchmark on large projects |

### Qualitative

- [ ] No work can complete with coverage <95% without ticketed override
- [ ] No work can complete with TDG regression without ticketed override
- [ ] All CUDA/AVX files protected from exclusion
- [ ] All overrides require documented justification AND future ticket
- [ ] All contracts stored immutably in git history
- [ ] Meta-verification ensures the falsifier itself is not broken

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Baseline capture too slow | Developer friction | Fast mode default, full mode opt-in |
| False positives block valid work | Developer frustration | Override mechanism with audit |
| Gaming detection too aggressive | Blocks legitimate patterns | Allowlist for known patterns |
| Git history bloat | Repository size | Prune old contracts after 90 days |
| Falsifier broken | False sense of security | Mandatory Meta-Falsification step |

## References

1. Popper, K. (1959). *The Logic of Scientific Discovery*. Routledge.
2. IEEE 730-2014. *Standard for Software Quality Assurance Processes*.
3. Toyota Production System. *Jidoka (Autonomation)*.
4. CLAUDE.md. *Zero Tolerance for Defects, 95% Minimum Coverage*.

## Appendix A: Example Contract JSON

```json
{
  "work_item_id": "feature-123",
  "created_at": "2026-01-25T10:30:00Z",
  "baseline_commit": "abc123def456",
  "baseline_tdg": 84.5,
  "baseline_coverage": 94.2,
  "baseline_rust_score": 112.0,
  "baseline_file_manifest": {
    "files": {
      "src/lib.rs": {
        "content_hash": "sha256:abc...",
        "lines": 245,
        "functions": 12,
        "max_complexity": 15,
        "category": "RustSource"
      },
      "src/cuda/kernels.cu": {
        "content_hash": "sha256:def...",
        "lines": 180,
        "functions": 8,
        "max_complexity": 12,
        "category": "CudaKernel"
      }
    },
    "coverage_required": [
      "src/lib.rs",
      "src/cuda/kernels.cu"
    ],
    "manifest_hash": "sha256:xyz..."
  },
  "thresholds": {
    "min_coverage_pct": 95.0,
    "max_tdg_regression": 0.0,
    "max_function_complexity": 20,
    "max_file_lines": 500,
    "min_spec_score": 95,
    "require_github_sync": true,
    "require_spec_update": true,
    "require_roadmap_update": true
  },
  "claims": [
    {
      "hypothesis": "All baseline files still exist",
      "falsification_method": "ManifestIntegrity",
      "evidence_required": { "type": "FileList", "files": [] },
      "result": null
    },
    {
      "hypothesis": "Total coverage >= 95%",
      "falsification_method": "AbsoluteCoverage",
      "evidence_required": { "type": "NumericComparison", "actual": 0, "threshold": 95.0 },
      "result": null
    }
  ]
}
```

## Appendix B: Falsification Output Example

```
pmat work complete feature-123

Loading contract from baseline commit abc123...

Running Popperian Falsification (11 claims to validate)

[1/11] Manifest Integrity
      Hypothesis: "All baseline files still exist"
      Falsification: Searching for missing files...
      Result: PASSED (142/142 files present)

[2/11] Meta-Falsification
      Hypothesis: "Falsifier is working"
      Falsification: Injecting dummy failure...
      Result: PASSED (Detected dummy failure)

[3/11] Coverage Gaming Detection
      Hypothesis: "No coverage exclusion gaming"
      Falsification: Scanning for gaming patterns...
      Result: FAILED
      Evidence: Found `#[cfg(not(coverage))]` at src/simd/avx.rs:45

... (rest of the output)

[11/11] GitHub Sync
      Hypothesis: "All changes pushed"
      Falsification: Checking git status...
      Result: PASSED (0 unpushed commits)

FALSIFICATION RESULT: BLOCKED (2 failures, 1 warning)

Failures (must fix):
  - [3/11] Coverage Gaming: Remove `#[cfg(not(coverage))]` from src/simd/avx.rs:45
  - [7/11] Complexity: Refactor process_data() (complexity 27 > 20)

Warnings (should fix):
  - [9/11] File Size: Consider splitting src/handlers.rs (512 > 500 lines)

Fix issues and retry: pmat work complete feature-123

Or override with justification and TICKET:
  pmat work complete feature-123 \
    --override complexity \
    --reason "Legacy code debt" \
    --ticket "DEBT-456"
```