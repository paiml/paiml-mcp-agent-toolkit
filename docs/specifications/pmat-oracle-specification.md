# PMAT Oracle Specification v1.0

**Status**: Draft
**Version**: 1.0.0
**Last Updated**: 2025-12-07
**Author**: PAIML Engineering
**Classification**: Technical Specification

---

## Executive Summary

PMAT Oracle is a unified, closed-loop quality improvement system for Rust projects that implements the Plan-Do-Check-Act (PDCA) cycle with Compiler-In-The-Loop (CITL) learning. It synthesizes **all native Rust toolchain signals** (rustc, clippy, cargo test, cargo build) with **all PMAT quality signals** (TDG, complexity, SATD, dead code, rust-project-score, five-whys) to converge any Rust project toward a **"perfect" state**: 95%+ test coverage, zero defects, optimal performance, and A+ quality grade.

### Design Philosophy: Toyota Production System

This specification applies Toyota Way principles to software quality automation:

- **Jidoka (Automation with Human Intelligence)**: Oracle suggests fixes with confidence scores; humans approve high-impact changes
- **Kaizen (Continuous Improvement)**: Each PDCA cycle improves the pattern library and prediction models
- **Genchi Genbutsu (Go and See)**: Evidence-based decisions from actual compiler output, not heuristics
- **Andon (Stop-the-Line)**: Automatic halt when quality degrades or confidence drops below threshold
- **Muda Elimination**: Minimize wasted developer time on repetitive fixes

### Scientific Foundation

This specification synthesizes methods from 15 peer-reviewed publications spanning fault localization, automated program repair, weak supervision, and continuous integration (see [References](#references)).

---

## 1. Problem Statement

### 1.1 Current State: Fragmented Quality Signals

Rust developers face a fragmented quality landscape:

| Signal Source | What It Detects | Integration Status |
|---------------|-----------------|-------------------|
| `rustc` | Type errors, ownership violations, lifetime issues | Manual interpretation |
| `cargo clippy` | Lint warnings, code smells, anti-patterns | Manual --fix or ignore |
| `cargo test` | Test failures, assertion violations | Manual debugging |
| `cargo build` | Compilation errors, missing dependencies | Manual resolution |
| `pmat TDG` | Technical debt grade per file | Informational only |
| `pmat complexity` | Cyclomatic/cognitive complexity | Threshold warnings |
| `pmat SATD` | TODO/FIXME/HACK markers | Report generation |
| `pmat dead-code` | Unused functions, modules | Manual removal |
| `pmat rust-project-score` | 106-point project scoring | Periodic assessment |
| `pmat five-whys` | Root cause analysis | Manual investigation |

**Problem**: These signals are siloed. No system synthesizes them into a unified improvement loop.

### 1.2 Target State: Converged Quality

PMAT Oracle converges projects toward:

| Quality Dimension | Target | Measurement |
|-------------------|--------|-------------|
| **Test Coverage** | ≥95% | cargo llvm-cov |
| **Mutation Score** | ≥85% | cargo mutants |
| **Compiler Errors** | 0 | rustc |
| **Clippy Warnings** | 0 | cargo clippy |
| **Test Failures** | 0 | cargo test |
| **TDG Grade** | A+ (≥95) | pmat analyze tdg |
| **Rust Project Score** | ≥90/106 | pmat rust-project-score |
| **SATD Markers** | 0 | pmat analyze satd |
| **Dead Code** | 0 | pmat dead-code |
| **Cyclomatic Complexity** | ≤15 per function | pmat complexity |
| **Cognitive Complexity** | ≤25 per function | pmat complexity |
| **Build Time** | ≤60s incremental | cargo build --timings |

---

## 2. Architecture

### 2.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PMAT ORACLE SYSTEM                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      SIGNAL AGGREGATION LAYER                        │   │
│  │  ┌──────────────────────────────────────────────────────────────┐   │   │
│  │  │ NATIVE RUST SIGNALS                                          │   │   │
│  │  │ • rustc errors (E0308, E0382, E0499, E0597, ...)            │   │   │
│  │  │ • clippy warnings (complexity, suspicious, correctness)      │   │   │
│  │  │ • cargo test failures (assertion, panic, timeout)            │   │   │
│  │  │ • cargo build errors (dependency, feature, target)           │   │   │
│  │  │ • cargo llvm-cov (line/branch/function coverage)             │   │   │
│  │  │ • cargo mutants (mutation score, surviving mutants)          │   │   │
│  │  └──────────────────────────────────────────────────────────────┘   │   │
│  │  ┌──────────────────────────────────────────────────────────────┐   │   │
│  │  │ PMAT SIGNALS                                                 │   │   │
│  │  │ • TDG score (0-100, per-file and aggregate)                  │   │   │
│  │  │ • Complexity (cyclomatic, cognitive, Halstead)               │   │   │
│  │  │ • SATD annotations (TODO, FIXME, HACK, XXX counts)           │   │   │
│  │  │ • Dead code (unused functions, modules, imports)             │   │   │
│  │  │ • Rust project score (106-point breakdown)                   │   │   │
│  │  │ • Five-whys analysis (root cause chains)                     │   │   │
│  │  │ • Churn analysis (commit frequency, change coupling)         │   │   │
│  │  └──────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    UNIFIED DEFECT SCHEMA (UDS)                       │   │
│  │  DefectReport {                                                      │   │
│  │    id: UUID,                                                         │   │
│  │    category: DefectCategory,     // 18 categories from OIP           │   │
│  │    severity: Severity,           // Critical/High/Medium/Low         │   │
│  │    confidence: f32,              // 0.0-1.0 prediction confidence    │   │
│  │    location: CodeLocation,       // file, line, column, span         │   │
│  │    signals: Vec<SignalEvidence>, // contributing signals             │   │
│  │    suggested_fixes: Vec<Fix>,    // ranked by confidence             │   │
│  │    root_cause: Option<FiveWhys>, // causal chain if available        │   │
│  │  }                                                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      ORACLE DECISION ENGINE                          │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐     │   │
│  │  │ Pattern Store  │  │ Ensemble       │  │ RAG Knowledge      │     │   │
│  │  │ (entrenar .apr)│  │ Predictor (EM) │  │ Base (trueno-rag)  │     │   │
│  │  │                │  │                │  │                    │     │   │
│  │  │ • 10K+ patterns│  │ • 5 signals    │  │ • Historical bugs  │     │   │
│  │  │ • Error→Fix    │  │ • Weak super.  │  │ • Fix patterns     │     │   │
│  │  │ • Cross-project│  │ • EM weights   │  │ • Hybrid retrieval │     │   │
│  │  └────────────────┘  └────────────────┘  └────────────────────┘     │   │
│  │                              │                                       │   │
│  │                              ▼                                       │   │
│  │  ┌──────────────────────────────────────────────────────────────┐   │   │
│  │  │ DECISION OUTPUT                                              │   │   │
│  │  │ • Fix suggestion with confidence score                       │   │   │
│  │  │ • Auto-apply threshold (default: confidence ≥ 0.9)           │   │   │
│  │  │ • Human review queue (0.7 ≤ confidence < 0.9)                │   │   │
│  │  │ • Skip threshold (confidence < 0.7)                          │   │   │
│  │  └──────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         PDCA EXECUTION LOOP                          │   │
│  │                                                                      │   │
│  │   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐     │   │
│  │   │   PLAN   │───▶│    DO    │───▶│  CHECK   │───▶│   ACT    │     │   │
│  │   │          │    │          │    │          │    │          │     │   │
│  │   │ Analyze  │    │ Apply    │    │ Verify   │    │ Learn    │     │   │
│  │   │ defects  │    │ fixes    │    │ fixes    │    │ patterns │     │   │
│  │   │          │    │          │    │          │    │          │     │   │
│  │   └──────────┘    └──────────┘    └──────────┘    └────┬─────┘     │   │
│  │        ▲                                               │            │   │
│  │        └───────────────────────────────────────────────┘            │   │
│  │                      (iterate until converged)                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Integration Matrix

| Component | Source | Role in Oracle |
|-----------|--------|----------------|
| **entrenar** | PAIML | Pattern storage (.apr files), fix templates |
| **trueno-rag** | PAIML | Hybrid retrieval (BM25 + dense), pattern matching |
| **OIP** | PAIML | Ensemble predictor, Tarantula SBFL, CITL mappings |
| **verificar** | PAIML | Mutation testing, semantic verification |
| **batuta** | PAIML | Stack orchestration, knowledge graph, backend selection |
| **aprender** | PAIML | RandomForest classifier, k-NN, clustering |
| **trueno** | PAIML | SIMD acceleration, vector operations, graph analytics |

---

## 3. Signal Taxonomy

### 3.1 Native Rust Signals

#### 3.1.1 Rustc Error Codes (Exhaustive Mapping)

Based on CITL mappings from OIP (`src/citl.rs`):

| Error Code | Description | DefectCategory | Confidence |
|------------|-------------|----------------|------------|
| E0308 | Mismatched types | TypeErrors | 0.95 |
| E0382 | Use of moved value | OwnershipBorrow | 0.90 |
| E0502 | Cannot borrow as mutable | OwnershipBorrow | 0.95 |
| E0503 | Cannot use value after mutable borrow | OwnershipBorrow | 0.95 |
| E0505 | Cannot move out of borrowed content | OwnershipBorrow | 0.95 |
| E0507 | Cannot move out of borrowed content | MemorySafety | 0.90 |
| E0499 | Cannot borrow as mutable more than once | OwnershipBorrow | 0.95 |
| E0597 | Borrowed value does not live long enough | OwnershipBorrow | 0.90 |
| E0716 | Temporary value dropped while borrowed | OwnershipBorrow | 0.90 |
| E0277 | Trait bound not satisfied | TraitBounds | 0.95 |
| E0412 | Cannot find type in scope | TypeErrors | 0.90 |
| E0425 | Cannot find value in scope | StdlibMapping | 0.85 |
| E0433 | Failed to resolve module path | StdlibMapping | 0.85 |
| E0599 | No method found for type | ASTTransform | 0.85 |
| E0615 | Attempted to access field on non-struct | OperatorPrecedence | 0.80 |
| E0658 | Unstable feature | Configuration | 0.75 |
| E0133 | Unsafe block required | MemorySafety | 0.90 |
| E0515 | Cannot return reference to local variable | OwnershipBorrow | 0.90 |

#### 3.1.2 Clippy Lint Categories

| Clippy Category | Lint Examples | DefectCategory | Auto-fixable |
|-----------------|---------------|----------------|--------------|
| correctness | `clippy::eq_op`, `clippy::erasing_op` | LogicErrors | Yes |
| suspicious | `clippy::suspicious_else_formatting` | LogicErrors | Partial |
| complexity | `clippy::cognitive_complexity` | PerformanceIssues | No |
| perf | `clippy::needless_collect`, `clippy::large_enum_variant` | PerformanceIssues | Yes |
| style | `clippy::redundant_clone`, `clippy::needless_return` | CodeStyle | Yes |
| pedantic | `clippy::cast_possible_truncation` | TypeErrors | Partial |
| restriction | `clippy::unwrap_used`, `clippy::expect_used` | ApiMisuse | No |
| nursery | `clippy::cognitive_complexity` | PerformanceIssues | No |

#### 3.1.3 Test Failure Signals

| Failure Type | Detection Method | DefectCategory |
|--------------|------------------|----------------|
| Assertion failure | `assert!`, `assert_eq!` panic | LogicErrors |
| Panic | `panic!`, `unreachable!` | ApiMisuse |
| Timeout | Test exceeds time limit | PerformanceIssues |
| Stack overflow | Recursive depth exceeded | MemorySafety |
| Deadlock | Timeout in concurrent tests | Concurrency |
| Property violation | `proptest`, `quickcheck` failure | LogicErrors |

### 3.2 PMAT Signals

#### 3.2.1 TDG (Technical Debt Gradient)

```rust
pub struct TdgScore {
    pub file_path: PathBuf,
    pub total: f32,           // 0-100 composite score
    pub grade: TdgGrade,      // A++, A+, A, A-, B+, B, B-, C+, C, C-, D, F
    pub components: TdgComponents,
}

pub struct TdgComponents {
    pub complexity_score: f32,      // Cyclomatic + cognitive
    pub test_coverage_score: f32,   // Line + branch coverage
    pub documentation_score: f32,   // Rustdoc completeness
    pub satd_penalty: f32,          // TODO/FIXME deductions
    pub churn_factor: f32,          // Recent change frequency
}
```

**Grade Thresholds**:
- A++ (≥98): Exceptional quality
- A+ (≥95): Near-perfect
- A (≥90): Excellent
- A- (≥85): Very good
- B+ (≥80): Good
- B (≥75): Acceptable
- B- (≥70): Needs improvement
- C+ (≥65): Below standard
- C (≥60): Poor
- C- (≥55): Very poor
- D (≥50): Critical
- F (<50): Failing

#### 3.2.2 Rust Project Score (106 Points)

From `rust-project-score-v1.1-update.md`:

| Category | Max Points | Components |
|----------|------------|------------|
| Rust Tooling Compliance | 25 | Clippy (tiered), rustfmt, cargo-audit, cargo-deny |
| Code Quality | 26 | Complexity (3), unsafe (9), mutation (8), build time (4), dead code (2) |
| Testing Excellence | 20 | Coverage (8), integration (4), doc tests (3), mutation (5) |
| Documentation | 15 | Rustdoc (7), README (5), changelog (3) |
| Performance & Benchmarking | 10 | Criterion (5), profiling (5) |
| Dependency Health | 12 | Count (5), features (4), pruning (3) |

#### 3.2.3 Five-Whys Root Cause Chain

```rust
pub struct FiveWhysAnalysis {
    pub symptom: String,
    pub why_chain: Vec<WhyLevel>,
    pub root_cause: RootCause,
    pub confidence: f32,
    pub evidence: Vec<Evidence>,
}

pub struct WhyLevel {
    pub question: String,
    pub hypothesis: String,
    pub evidence: Vec<Evidence>,
    pub confidence: f32,
}

pub struct RootCause {
    pub category: DefectCategory,
    pub description: String,
    pub recommended_fixes: Vec<Fix>,
    pub prevention_strategy: String,
}
```

---

## 4. Unified Defect Schema (UDS)

### 4.1 Schema Definition

```rust
/// Unified schema for all defect types across signal sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectReport {
    /// Unique identifier
    pub id: Uuid,

    /// Timestamp of detection
    pub detected_at: DateTime<Utc>,

    /// Primary defect category (18 types from OIP)
    pub category: DefectCategory,

    /// Severity level
    pub severity: Severity,

    /// Confidence in classification (0.0-1.0)
    pub confidence: f32,

    /// Code location
    pub location: CodeLocation,

    /// Contributing signal evidence
    pub signals: Vec<SignalEvidence>,

    /// Suggested fixes ranked by confidence
    pub suggested_fixes: Vec<SuggestedFix>,

    /// Root cause analysis (if available)
    pub root_cause: Option<FiveWhysAnalysis>,

    /// Historical similar defects (from RAG)
    pub similar_defects: Vec<SimilarDefect>,

    /// Oracle decision
    pub decision: OracleDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefectCategory {
    // Memory & Concurrency (from OIP)
    MemorySafety,
    Concurrency,
    OwnershipBorrow,

    // Type System
    TypeErrors,
    TypeAnnotationGap,
    TraitBounds,
    OperatorPrecedence,

    // Performance & Security
    PerformanceIssues,
    Security,
    Configuration,

    // API & Integration
    ApiMisuse,
    IntegrationFailure,
    StdlibMapping,

    // Code Quality
    DocumentationGap,
    TestingGap,

    // Rust-specific
    ASTTransform,
    ComprehensionBug,
    IteratorChain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,  // Blocks compilation or causes runtime crash
    High,      // Major functionality impact
    Medium,    // Moderate impact, workaround exists
    Low,       // Minor issue, cosmetic or style
}

#[derive(Debug, Clone)]
pub struct SignalEvidence {
    pub source: SignalSource,
    pub raw_message: String,
    pub parsed_data: serde_json::Value,
    pub weight: f32,  // Contribution to final confidence
}

#[derive(Debug, Clone, Copy)]
pub enum SignalSource {
    Rustc,
    Clippy,
    CargoTest,
    CargoBuild,
    LlvmCov,
    CargoMutants,
    PmatTdg,
    PmatComplexity,
    PmatSatd,
    PmatDeadCode,
    PmatRustProjectScore,
    PmatFiveWhys,
    PmatChurn,
}
```

### 4.2 Signal Fusion Algorithm

Based on ensemble weak supervision from OIP [1, 2]:

```rust
/// Fuse multiple signals into unified confidence score
/// Uses Expectation-Maximization with learned weights
pub fn fuse_signals(signals: &[SignalEvidence]) -> (f32, Vec<f32>) {
    // Initialize weights uniformly
    let n = signals.len();
    let mut weights = vec![1.0 / n as f32; n];

    // EM iterations (from OIP ensemble_predictor.rs)
    for _ in 0..100 {
        // E-step: Estimate latent labels
        let latent_labels = estimate_latent_labels(signals, &weights);

        // M-step: Update weights based on agreement
        weights = update_weights(signals, &latent_labels);

        // Check convergence
        if converged(&weights) {
            break;
        }
    }

    // Compute final confidence as weighted sum
    let confidence = signals.iter()
        .zip(weights.iter())
        .map(|(s, w)| s.weight * w)
        .sum::<f32>()
        .clamp(0.0, 1.0);

    (confidence, weights)
}
```

---

## 5. PDCA Execution Loop

### 5.1 Phase 1: PLAN (Defect Analysis)

```rust
pub struct PlanPhase {
    /// Maximum defects to analyze per iteration
    pub batch_size: usize,

    /// Minimum confidence to include in plan
    pub confidence_threshold: f32,

    /// Priority ordering strategy
    pub priority: PriorityStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum PriorityStrategy {
    /// Highest severity first (Critical → Low)
    SeverityFirst,

    /// Highest confidence first (most certain fixes)
    ConfidenceFirst,

    /// Highest impact first (based on churn + centrality)
    ImpactFirst,

    /// Fastest fixes first (quick wins)
    QuickWins,

    /// Blocking issues first (compilation errors)
    BlockersFirst,
}

impl PlanPhase {
    pub async fn execute(&self, project: &Project) -> PlanResult {
        // 1. Collect all signals
        let rustc_errors = self.collect_rustc_errors(project).await?;
        let clippy_warnings = self.collect_clippy_warnings(project).await?;
        let test_failures = self.collect_test_failures(project).await?;
        let pmat_issues = self.collect_pmat_issues(project).await?;

        // 2. Unify into DefectReports
        let defects = self.unify_signals(
            rustc_errors,
            clippy_warnings,
            test_failures,
            pmat_issues,
        )?;

        // 3. Prioritize and batch
        let prioritized = self.prioritize(defects, self.priority);
        let batch = prioritized.into_iter().take(self.batch_size).collect();

        // 4. Query oracle for fix suggestions
        let fixes = self.query_oracle(&batch).await?;

        Ok(PlanResult { defects: batch, fixes })
    }

    async fn collect_rustc_errors(&self, project: &Project) -> Result<Vec<RustcError>> {
        let output = Command::new("cargo")
            .args(["build", "--message-format=json"])
            .current_dir(&project.path)
            .output()
            .await?;

        parse_rustc_json(&output.stdout)
    }

    async fn collect_clippy_warnings(&self, project: &Project) -> Result<Vec<ClippyWarning>> {
        let output = Command::new("cargo")
            .args(["clippy", "--message-format=json", "--", "-D", "warnings"])
            .current_dir(&project.path)
            .output()
            .await?;

        parse_clippy_json(&output.stdout)
    }

    async fn collect_pmat_issues(&self, project: &Project) -> Result<PmatAnalysis> {
        // Collect TDG
        let tdg = Command::new("pmat")
            .args(["analyze", "tdg", "--format", "json", "--path"])
            .arg(&project.path)
            .output()
            .await?;

        // Collect complexity
        let complexity = Command::new("pmat")
            .args(["analyze", "complexity", "--format", "json", "--path"])
            .arg(&project.path)
            .output()
            .await?;

        // Collect SATD
        let satd = Command::new("pmat")
            .args(["analyze", "satd", "--format", "json", "--path"])
            .arg(&project.path)
            .output()
            .await?;

        // Collect dead code
        let dead_code = Command::new("pmat")
            .args(["dead-code", "--format", "json", "--path"])
            .arg(&project.path)
            .output()
            .await?;

        // Collect rust-project-score
        let score = Command::new("pmat")
            .args(["rust-project-score", "--format", "json", "--path"])
            .arg(&project.path)
            .output()
            .await?;

        PmatAnalysis::parse(tdg, complexity, satd, dead_code, score)
    }
}
```

### 5.2 Phase 2: DO (Fix Application)

```rust
pub struct DoPhase {
    /// Auto-apply threshold (confidence ≥ this value)
    pub auto_apply_threshold: f32,

    /// Maximum fixes to apply per iteration
    pub max_fixes_per_iteration: usize,

    /// Backup strategy before applying fixes
    pub backup: BackupStrategy,
}

#[derive(Debug, Clone)]
pub enum BackupStrategy {
    /// Git stash before applying
    GitStash,

    /// Create patch files
    PatchFiles(PathBuf),

    /// Copy to backup directory
    CopyBackup(PathBuf),

    /// No backup (dangerous)
    None,
}

impl DoPhase {
    pub async fn execute(&self, plan: &PlanResult) -> DoResult {
        // 1. Create backup
        self.create_backup(&plan.project).await?;

        // 2. Filter fixes by confidence threshold
        let auto_fixes: Vec<_> = plan.fixes.iter()
            .filter(|f| f.confidence >= self.auto_apply_threshold)
            .take(self.max_fixes_per_iteration)
            .collect();

        let review_fixes: Vec<_> = plan.fixes.iter()
            .filter(|f| f.confidence < self.auto_apply_threshold && f.confidence >= 0.7)
            .collect();

        // 3. Apply auto-fixes
        let mut applied = Vec::new();
        for fix in auto_fixes {
            match self.apply_fix(fix).await {
                Ok(result) => applied.push((fix.clone(), result)),
                Err(e) => {
                    // Rollback on failure
                    self.rollback(&applied).await?;
                    return Err(e);
                }
            }
        }

        // 4. Queue review fixes for human approval
        let review_queue = self.queue_for_review(review_fixes).await?;

        Ok(DoResult { applied, review_queue })
    }

    async fn apply_fix(&self, fix: &SuggestedFix) -> Result<ApplyResult> {
        match fix.fix_type {
            FixType::ClippyAutoFix => {
                // Use clippy --fix for applicable lints
                Command::new("cargo")
                    .args(["clippy", "--fix", "--allow-dirty", "--allow-staged"])
                    .current_dir(&fix.project_path)
                    .output()
                    .await?;
            }

            FixType::DiffPatch(ref diff) => {
                // Apply unified diff patch
                apply_unified_diff(&fix.location, diff)?;
            }

            FixType::Replacement { ref old, ref new } => {
                // Simple text replacement
                let content = fs::read_to_string(&fix.location.file_path)?;
                let updated = content.replace(old, new);
                fs::write(&fix.location.file_path, updated)?;
            }

            FixType::InsertAfter { ref anchor, ref content } => {
                insert_after_line(&fix.location.file_path, anchor, content)?;
            }

            FixType::DeleteLines { start, end } => {
                delete_lines(&fix.location.file_path, start, end)?;
            }
        }

        Ok(ApplyResult::Success)
    }
}
```

### 5.3 Phase 3: CHECK (Verification)

```rust
pub struct CheckPhase {
    /// Run cargo build to verify compilation
    pub verify_build: bool,

    /// Run cargo test to verify tests pass
    pub verify_tests: bool,

    /// Run cargo clippy to verify no new warnings
    pub verify_clippy: bool,

    /// Check that metrics improved
    pub verify_metrics: bool,

    /// Timeout for verification commands
    pub timeout: Duration,
}

impl CheckPhase {
    pub async fn execute(&self, do_result: &DoResult, baseline: &Metrics) -> CheckResult {
        let mut checks = Vec::new();

        // 1. Verify compilation
        if self.verify_build {
            let build_result = self.check_build().await?;
            checks.push(Check::Build(build_result));

            if !build_result.success {
                return CheckResult::Failed { checks, reason: "Build failed".into() };
            }
        }

        // 2. Verify tests
        if self.verify_tests {
            let test_result = self.check_tests().await?;
            checks.push(Check::Test(test_result));

            if !test_result.success {
                return CheckResult::Failed { checks, reason: "Tests failed".into() };
            }
        }

        // 3. Verify clippy
        if self.verify_clippy {
            let clippy_result = self.check_clippy().await?;
            checks.push(Check::Clippy(clippy_result));

            // Allow same or fewer warnings, not more
            if clippy_result.warning_count > baseline.clippy_warnings {
                return CheckResult::Failed {
                    checks,
                    reason: format!(
                        "Clippy warnings increased: {} → {}",
                        baseline.clippy_warnings,
                        clippy_result.warning_count
                    )
                };
            }
        }

        // 4. Verify metrics improved
        if self.verify_metrics {
            let current_metrics = self.collect_metrics().await?;
            checks.push(Check::Metrics(current_metrics.clone()));

            // Check for regression
            if let Some(regression) = self.detect_regression(&baseline, &current_metrics) {
                return CheckResult::Failed { checks, reason: regression };
            }
        }

        CheckResult::Passed { checks }
    }

    fn detect_regression(&self, baseline: &Metrics, current: &Metrics) -> Option<String> {
        // Coverage must not decrease
        if current.test_coverage < baseline.test_coverage - 0.01 {
            return Some(format!(
                "Coverage decreased: {:.1}% → {:.1}%",
                baseline.test_coverage * 100.0,
                current.test_coverage * 100.0
            ));
        }

        // TDG must not decrease
        if current.tdg_score < baseline.tdg_score - 1.0 {
            return Some(format!(
                "TDG decreased: {:.1} → {:.1}",
                baseline.tdg_score,
                current.tdg_score
            ));
        }

        // Rust project score must not decrease
        if current.rust_project_score < baseline.rust_project_score - 1 {
            return Some(format!(
                "Rust project score decreased: {} → {}",
                baseline.rust_project_score,
                current.rust_project_score
            ));
        }

        None
    }
}
```

### 5.4 Phase 4: ACT (Learning)

```rust
pub struct ActPhase {
    /// Pattern store for successful fixes
    pub pattern_store: PatternStore,

    /// RAG knowledge base for historical bugs
    pub knowledge_base: RagKnowledgeBase,

    /// Ensemble predictor for weight updates
    pub ensemble: EnsemblePredictor,
}

impl ActPhase {
    pub async fn execute(
        &self,
        do_result: &DoResult,
        check_result: &CheckResult,
    ) -> ActResult {
        match check_result {
            CheckResult::Passed { .. } => {
                // Successful fix: capture pattern
                for (fix, _) in &do_result.applied {
                    self.capture_successful_pattern(fix).await?;
                }

                // Update ensemble weights (positive feedback)
                self.ensemble.update_weights(Feedback::Positive).await?;

                ActResult::PatternsCaptured(do_result.applied.len())
            }

            CheckResult::Failed { reason, .. } => {
                // Failed fix: downweight pattern
                for (fix, _) in &do_result.applied {
                    self.downweight_pattern(fix).await?;
                }

                // Update ensemble weights (negative feedback)
                self.ensemble.update_weights(Feedback::Negative).await?;

                // Record failure for future learning
                self.knowledge_base.record_failure(reason.clone()).await?;

                ActResult::PatternsDownweighted(do_result.applied.len())
            }
        }
    }

    async fn capture_successful_pattern(&self, fix: &SuggestedFix) -> Result<()> {
        let pattern = FixPattern {
            error_code: fix.defect.signals[0].source.to_error_code(),
            context: extract_ast_context(&fix.location)?,
            fix_template: fix.to_template(),
            confidence: fix.confidence,
            times_applied: 1,
            success_rate: 1.0,
        };

        // Store in entrenar pattern store (.apr file)
        self.pattern_store.insert(pattern).await?;

        // Index in RAG knowledge base
        let doc = BugDocument {
            id: Uuid::new_v4().to_string(),
            title: format!("Fix for {:?}", fix.defect.category),
            description: fix.description.clone(),
            fix_commit: get_current_commit()?,
            fix_diff: fix.to_diff(),
            affected_files: vec![fix.location.file_path.display().to_string()],
            category: fix.defect.category,
            severity: fix.defect.severity as u8,
            symptoms: extract_symptoms(&fix.defect.signals),
            root_cause: fix.defect.root_cause.as_ref().map(|r| r.description.clone()).unwrap_or_default(),
            fix_pattern: fix.to_template(),
        };

        self.knowledge_base.index_document(doc).await?;

        Ok(())
    }
}
```

---

## 6. Convergence Criteria

### 6.1 Quality Gates

```rust
#[derive(Debug, Clone)]
pub struct ConvergenceTargets {
    /// Test coverage target (default: 0.95)
    pub test_coverage: f32,

    /// Mutation score target (default: 0.85)
    pub mutation_score: f32,

    /// Maximum compiler errors (default: 0)
    pub max_compiler_errors: usize,

    /// Maximum clippy warnings (default: 0)
    pub max_clippy_warnings: usize,

    /// Maximum test failures (default: 0)
    pub max_test_failures: usize,

    /// Minimum TDG grade (default: A+)
    pub min_tdg_grade: TdgGrade,

    /// Minimum rust-project-score (default: 90)
    pub min_rust_project_score: u32,

    /// Maximum SATD markers (default: 0)
    pub max_satd_markers: usize,

    /// Maximum dead code items (default: 0)
    pub max_dead_code: usize,

    /// Maximum cyclomatic complexity (default: 15)
    pub max_cyclomatic_complexity: u32,

    /// Maximum cognitive complexity (default: 25)
    pub max_cognitive_complexity: u32,

    /// Maximum incremental build time (default: 60s)
    pub max_build_time: Duration,
}

impl Default for ConvergenceTargets {
    fn default() -> Self {
        Self {
            test_coverage: 0.95,
            mutation_score: 0.85,
            max_compiler_errors: 0,
            max_clippy_warnings: 0,
            max_test_failures: 0,
            min_tdg_grade: TdgGrade::APlus,
            min_rust_project_score: 90,
            max_satd_markers: 0,
            max_dead_code: 0,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 25,
            max_build_time: Duration::from_secs(60),
        }
    }
}

impl ConvergenceTargets {
    pub fn is_converged(&self, metrics: &Metrics) -> ConvergenceStatus {
        let mut failures = Vec::new();

        if metrics.test_coverage < self.test_coverage {
            failures.push(format!(
                "Coverage: {:.1}% < {:.1}%",
                metrics.test_coverage * 100.0,
                self.test_coverage * 100.0
            ));
        }

        if metrics.mutation_score < self.mutation_score {
            failures.push(format!(
                "Mutation score: {:.1}% < {:.1}%",
                metrics.mutation_score * 100.0,
                self.mutation_score * 100.0
            ));
        }

        if metrics.compiler_errors > self.max_compiler_errors {
            failures.push(format!(
                "Compiler errors: {} > {}",
                metrics.compiler_errors,
                self.max_compiler_errors
            ));
        }

        if metrics.clippy_warnings > self.max_clippy_warnings {
            failures.push(format!(
                "Clippy warnings: {} > {}",
                metrics.clippy_warnings,
                self.max_clippy_warnings
            ));
        }

        // ... additional checks ...

        if failures.is_empty() {
            ConvergenceStatus::Converged
        } else {
            ConvergenceStatus::NotConverged { remaining: failures }
        }
    }
}
```

### 6.2 Iteration Limits and Safeguards

```rust
pub struct OracleConfig {
    /// Maximum PDCA iterations before giving up
    pub max_iterations: usize,

    /// Minimum progress per iteration (prevents infinite loops)
    pub min_progress_per_iteration: f32,

    /// Stagnation threshold (stop if no progress for N iterations)
    pub stagnation_threshold: usize,

    /// Andon cord: halt on critical regression
    pub andon_enabled: bool,

    /// Human approval required for high-impact changes
    pub require_human_approval_above: Option<usize>,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            min_progress_per_iteration: 0.001,  // 0.1% improvement
            stagnation_threshold: 5,
            andon_enabled: true,
            require_human_approval_above: Some(10),  // Changes affecting >10 files
        }
    }
}
```

---

## 7. Oracle Decision Engine

### 7.1 Pattern Store Integration (entrenar)

```rust
/// Query entrenar pattern store for fix suggestions
pub async fn query_pattern_store(
    defect: &DefectReport,
    store: &DecisionPatternStore,
) -> Vec<SuggestedFix> {
    let error_code = extract_error_code(&defect.signals);
    let context = extract_ast_context(&defect.location)?;

    // Query patterns matching error code + context
    let patterns = store.query(Query {
        error_code: Some(error_code),
        context_hash: Some(context.hash()),
        min_confidence: 0.7,
        limit: 10,
    })?;

    patterns.into_iter()
        .map(|p| SuggestedFix {
            fix_type: p.to_fix_type(),
            confidence: p.confidence * p.success_rate,
            pattern_id: p.id,
            description: p.description,
            source: FixSource::PatternStore,
        })
        .collect()
}
```

### 7.2 RAG Knowledge Base Integration (trueno-rag)

```rust
/// Query RAG for similar historical bugs and fixes
pub async fn query_rag_knowledge_base(
    defect: &DefectReport,
    rag: &RagPipeline,
) -> Vec<SimilarDefect> {
    let query = format!(
        "{:?} in {} at line {}: {}",
        defect.category,
        defect.location.file_path.display(),
        defect.location.line,
        defect.signals[0].raw_message
    );

    let (results, context) = rag.query_with_context(&query, 10).await?;

    results.into_iter()
        .map(|r| SimilarDefect {
            id: r.id,
            similarity: r.score,
            category: r.metadata.get("category").map(|c| DefectCategory::from(c)),
            fix_pattern: r.metadata.get("fix_pattern").cloned(),
            symptoms: r.metadata.get("symptoms").map(|s| serde_json::from_str(s).unwrap_or_default()),
        })
        .collect()
}
```

### 7.3 Ensemble Predictor Integration (OIP)

```rust
/// Use ensemble weak supervision for confidence calibration
pub fn calibrate_confidence(
    defect: &DefectReport,
    ensemble: &EnsemblePredictor,
) -> f32 {
    let features = FileFeatures {
        sbfl_score: defect.fault_localization_score.unwrap_or(0.0),
        tdg_score: 1.0 - (defect.tdg_score.unwrap_or(0.0) / 100.0),
        churn_score: defect.churn_score.unwrap_or(0.0),
        complexity_score: defect.complexity_score.unwrap_or(0.0),
        rag_similarity: defect.rag_similarity.unwrap_or(0.0),
    };

    // Use learned weights from EM
    ensemble.predict_confidence(&features)
}
```

### 7.4 Fault Localization Integration (OIP Tarantula)

```rust
/// Use Tarantula SBFL for precise fault localization
pub async fn localize_fault(
    test_failures: &[TestFailure],
    coverage: &LcovCoverage,
) -> Vec<SuspiciousStatement> {
    let tarantula = TarantulaSbfl::new(Formula::Ochiai);

    // Parse coverage data
    let passed_coverage = coverage.filter_passed();
    let failed_coverage = coverage.filter_failed();

    // Score each statement
    let rankings = tarantula.rank_statements(
        &passed_coverage,
        &failed_coverage,
        10,  // top-10
    )?;

    rankings.into_iter()
        .map(|r| SuspiciousStatement {
            file: r.file,
            line: r.line,
            suspiciousness: r.score,
            formula: Formula::Ochiai,
        })
        .collect()
}
```

---

## 8. CLI Interface

### 8.1 Commands

```bash
# Full PDCA loop until convergence
pmat oracle fix --path . --max-iterations 50

# Single iteration (for CI/CD)
pmat oracle fix --path . --iterations 1

# Dry-run (show plan without applying)
pmat oracle fix --path . --dry-run

# Custom thresholds
pmat oracle fix --path . \
    --coverage-target 0.90 \
    --tdg-target 85 \
    --rust-score-target 80

# Auto-apply with confidence threshold
pmat oracle fix --path . --auto-apply-threshold 0.95

# Interactive mode (human approval for each fix)
pmat oracle fix --path . --interactive

# Show current convergence status
pmat oracle status --path .

# Export fix plan to JSON
pmat oracle plan --path . --format json --output plan.json

# Replay specific fixes from plan
pmat oracle apply --plan plan.json --fixes 1,3,5
```

### 8.2 Configuration File

```toml
# .pmat-oracle.toml

[convergence]
test_coverage = 0.95
mutation_score = 0.85
max_compiler_errors = 0
max_clippy_warnings = 0
max_test_failures = 0
min_tdg_grade = "A+"
min_rust_project_score = 90
max_satd_markers = 0
max_dead_code = 0
max_cyclomatic_complexity = 15
max_cognitive_complexity = 25
max_build_time_secs = 60

[execution]
max_iterations = 100
min_progress_per_iteration = 0.001
stagnation_threshold = 5
andon_enabled = true
require_human_approval_above = 10

[thresholds]
auto_apply = 0.9
human_review = 0.7
skip_below = 0.5

[pattern_store]
path = ".pmat/patterns.apr"
cross_project_import = true
import_sources = ["~/.pmat/global-patterns.apr"]

[rag]
index_path = ".pmat/rag-index"
chunk_size = 512
chunk_overlap = 50
fusion_strategy = "rrf"

[reporting]
format = "markdown"
output = "pmat-oracle-report.md"
include_evidence = true
include_diffs = true
```

### 8.3 Rich Report Format (PMAT-ORACLE-REPORT-V1)

The Oracle supports a `--format rich` option that produces comprehensive TEXT-only reports with:

#### 8.3.1 Report Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     PMAT ORACLE QUALITY REPORT                          │
│                   Project: my-rust-project                              │
│                   Timestamp: 2025-12-07 14:30:00                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ╔════════════════════════════════════════════════════════════════════╗ │
│  ║  ANDON STATUS: 🔴 RED (3 blocking defects)                         ║ │
│  ╚════════════════════════════════════════════════════════════════════╝ │
│                                                                         │
│  ════════════════════════════════════════════════════════════════════   │
│  EXECUTIVE SUMMARY                                                      │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  Iterations Completed: 5/50                                             │
│  Convergence:          62.3%                                            │
│  Defects Fixed:        24                                               │
│  Defects Remaining:    18                                               │
│  Pattern Hit Rate:     78.4%                                            │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ QUALITY PROGRESSION                                              │   │
│  │                                                                  │   │
│  │ Coverage    ███████████░░░░░ 72.4%  (target: 95%)               │   │
│  │ TDG Score   ██████████████░░ 87.2   (target: 95)                │   │
│  │ Rust Score  █████████████░░░ 82/106 (target: 90)                │   │
│  │ Complexity  ████████████████ 100%   (target: ≤15)               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ════════════════════════════════════════════════════════════════════   │
│  DEFECT CLASSIFICATION BY CATEGORY (K-means, k=4, silhouette=0.72)      │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │ Cluster 0: Type System Errors (8 defects)                      │     │
│  │   E0308 (mismatched types)     ████████████ 5                  │     │
│  │   E0277 (trait not satisfied)  ██████░░░░░░ 3                  │     │
│  │                                                                │     │
│  │ Cluster 1: Ownership/Borrow (6 defects)                        │     │
│  │   E0382 (use after move)       ██████████░░ 4                  │     │
│  │   E0502 (borrow conflict)      █████░░░░░░░ 2                  │     │
│  │                                                                │     │
│  │ Cluster 2: Lint Warnings (3 defects)                           │     │
│  │   clippy::unwrap_used          ████████████ 2                  │     │
│  │   clippy::missing_docs         ██████░░░░░░ 1                  │     │
│  │                                                                │     │
│  │ Cluster 3: Test Failures (1 defect)                            │     │
│  │   assertion failure            ████████████ 1                  │     │
│  └────────────────────────────────────────────────────────────────┘     │
│                                                                         │
│  ════════════════════════════════════════════════════════════════════   │
│  PAGERANK-RANKED DEFECT CENTRALITY                                      │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  Rank │ File                          │ PageRank │ Defects │ Category  │
│  ─────┼───────────────────────────────┼──────────┼─────────┼───────────│
│  #1   │ src/services/parser.rs:145    │ 0.0842   │ 3       │ Type      │
│  #2   │ src/models/user.rs:67         │ 0.0621   │ 2       │ Borrow    │
│  #3   │ src/handlers/api.rs:234       │ 0.0534   │ 2       │ Lint      │
│  #4   │ tests/integration.rs:89       │ 0.0412   │ 1       │ Test      │
│  #5   │ src/utils/convert.rs:12       │ 0.0387   │ 1       │ Type      │
│                                                                         │
│  ════════════════════════════════════════════════════════════════════   │
│  LOUVAIN COMMUNITY DETECTION (modularity=0.68)                          │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  Community 1 (6 files): src/services/* - High coupling, refactor target │
│  Community 2 (4 files): src/models/* - Clean separation                 │
│  Community 3 (3 files): tests/* - Test isolation complete               │
│                                                                         │
│  ════════════════════════════════════════════════════════════════════   │
│  SEMANTIC DOMAIN CLASSIFICATION                                         │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  Core Business Logic:     ████████████████ 12 files                     │
│  Infrastructure/Stdlib:   ████████░░░░░░░░  6 files                     │
│  External Dependencies:   ████░░░░░░░░░░░░  3 files                     │
│  Test Code:               ██████░░░░░░░░░░  4 files                     │
│                                                                         │
│  ════════════════════════════════════════════════════════════════════   │
│  SUGGESTED REMEDIATION (sorted by impact × confidence)                  │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  1. [AUTO-APPLY] Fix E0308 in parser.rs:145 (conf: 0.97)               │
│     └─ Pattern: type_coercion_fix_001                                   │
│                                                                         │
│  2. [AUTO-APPLY] Fix E0382 in user.rs:67 (conf: 0.94)                  │
│     └─ Pattern: clone_before_move_001                                   │
│                                                                         │
│  3. [REVIEW] Refactor api.rs:234 for clippy::unwrap_used (conf: 0.72)  │
│     └─ Suggested: Replace .unwrap() with .ok_or_else()                  │
│                                                                         │
│  4. [SKIP] Low-confidence fix for convert.rs:12 (conf: 0.45)           │
│     └─ Reason: Multiple valid approaches, human judgment needed         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 8.3.2 Rich Report Modules

The rich report is generated by these pure-Rust components (no JavaScript):

| Module | Purpose | Algorithm |
|--------|---------|-----------|
| `semantic.rs` | Semantic domain classification | Import graph analysis, stdlib detection |
| `clustering.rs` | K-means defect clustering | K-means with elbow method, silhouette scoring |
| `graph.rs` | PageRank and community detection | PageRank for centrality, Louvain for communities |
| `report.rs` | TEXT-only report generator | ASCII art bars, box-drawing tables, owo-colors |

#### 8.3.3 CLI Integration

```bash
# Generate rich report (TEXT output, colored for terminal)
pmat oracle fix --path . --format rich

# Export rich report to file (no colors, ASCII-safe)
pmat oracle fix --path . --format rich --output oracle-report.txt

# Rich report with specific sections
pmat oracle fix --path . --format rich \
    --include-pagerank \
    --include-clustering \
    --include-communities
```

#### 8.3.4 Andon Status Colors

| Status | Condition | Action |
|--------|-----------|--------|
| 🟢 **GREEN** | All quality gates pass | Continue to next iteration |
| 🟡 **YELLOW** | Minor issues, no blockers | Review warnings, proceed cautiously |
| 🔴 **RED** | Blocking defects detected | Stop-the-line (Andon principle), escalate |

---

## 9. Metrics and Observability

### 9.1 Dashboard Metrics

```rust
pub struct OracleMetrics {
    // Progress metrics
    pub iterations_completed: u64,
    pub defects_fixed: u64,
    pub defects_remaining: u64,
    pub convergence_percentage: f32,

    // Confidence metrics
    pub avg_fix_confidence: f32,
    pub pattern_hit_rate: f32,
    pub rag_retrieval_rate: f32,

    // Performance metrics
    pub avg_iteration_time: Duration,
    pub total_run_time: Duration,
    pub fixes_per_hour: f32,

    // Quality progression
    pub coverage_progression: Vec<f32>,
    pub tdg_progression: Vec<f32>,
    pub rust_score_progression: Vec<u32>,

    // Learning metrics
    pub patterns_captured: u64,
    pub patterns_downweighted: u64,
    pub ensemble_weight_updates: u64,
}
```

### 9.2 Prometheus Export

```
# HELP pmat_oracle_defects_total Total defects detected
# TYPE pmat_oracle_defects_total counter
pmat_oracle_defects_total{category="OwnershipBorrow",severity="high"} 42

# HELP pmat_oracle_fixes_applied_total Total fixes applied
# TYPE pmat_oracle_fixes_applied_total counter
pmat_oracle_fixes_applied_total{source="pattern_store"} 35
pmat_oracle_fixes_applied_total{source="rag"} 12

# HELP pmat_oracle_convergence_ratio Current convergence ratio
# TYPE pmat_oracle_convergence_ratio gauge
pmat_oracle_convergence_ratio 0.87

# HELP pmat_oracle_iteration_duration_seconds Duration of each PDCA iteration
# TYPE pmat_oracle_iteration_duration_seconds histogram
pmat_oracle_iteration_duration_seconds_bucket{le="1"} 5
pmat_oracle_iteration_duration_seconds_bucket{le="5"} 15
pmat_oracle_iteration_duration_seconds_bucket{le="30"} 45
```

---

## 10. Security and Safety

### 10.1 Sandboxing

All fix applications run in a sandboxed environment:

1. **Git stash backup** before any changes
2. **Atomic rollback** on CHECK failure
3. **File permission preservation**
4. **No network access during fix application**
5. **Timeout enforcement** on all commands

### 10.2 Human-in-the-Loop

For high-risk fixes:

- Changes affecting >10 files require approval
- Security-related fixes (unsafe blocks) require approval
- Dependency changes require approval
- Public API changes require approval

### 10.3 Audit Trail

All oracle actions are logged:

```rust
pub struct OracleAuditLog {
    pub timestamp: DateTime<Utc>,
    pub action: OracleAction,
    pub defect_id: Uuid,
    pub fix_id: Option<Uuid>,
    pub confidence: f32,
    pub approved_by: Option<String>,  // "auto" or human identifier
    pub result: ActionResult,
    pub metrics_before: Metrics,
    pub metrics_after: Option<Metrics>,
}
```

---

## 11. Implementation Phases

### Phase 1: Foundation (4 weeks)

- [ ] Unified Defect Schema (UDS) implementation
- [ ] Signal collectors for all sources (rustc, clippy, cargo test, pmat)
- [ ] Basic PDCA loop without learning
- [ ] CLI interface (`pmat oracle fix --dry-run`)

### Phase 2: Pattern Store (3 weeks)

- [ ] entrenar integration for .apr pattern files
- [ ] Pattern matching engine
- [ ] Fix template system
- [ ] Pattern import/export

### Phase 3: RAG Integration (3 weeks)

- [ ] trueno-rag integration
- [ ] Knowledge base indexing pipeline
- [ ] Hybrid retrieval (BM25 + dense)
- [ ] Fix suggestion ranking

### Phase 4: Ensemble Learning (2 weeks)

- [ ] OIP ensemble predictor integration
- [ ] Weak supervision EM implementation
- [ ] Weight update pipeline
- [ ] Confidence calibration

### Phase 5: Fault Localization (2 weeks)

- [ ] Tarantula SBFL integration
- [ ] LCOV coverage parsing
- [ ] SZZ bug-introducing commit analysis
- [ ] Hybrid fault localizer

### Phase 6: Convergence & Observability (2 weeks)

- [ ] Convergence criteria enforcement
- [ ] Prometheus metrics export
- [ ] Dashboard integration
- [ ] Audit logging

### Phase 7: Production Hardening (2 weeks)

- [ ] Sandboxing and safety rails
- [ ] Human-in-the-loop workflows
- [ ] Performance optimization
- [ ] Documentation and examples

---

## 12. References

### Fault Localization

1. Jones, J.A., Harrold, M.J., & Stasko, J. (2002). "Visualization of Test Information to Assist Fault Localization." *ICSE 2002*, pp. 467-477. [Tarantula original paper]

2. Abreu, R., Zoeteweij, P., & Van Gemund, A.J.C. (2007). "On the Accuracy of Spectrum-based Fault Localization." *TAICPART 2007*, pp. 89-98. [Ochiai formula]

3. Wong, W.E., Debroy, V., Gao, R., & Li, Y. (2014). "The DStar Method for Effective Software Fault Localization." *IEEE TSE*, 40(1), pp. 1-17.

### Automated Program Repair

4. Le Goues, C., Nguyen, T., Forrest, S., & Weimer, W. (2012). "GenProg: A Generic Method for Automatic Software Repair." *IEEE TSE*, 38(1), pp. 54-72.

5. Monperrus, M. (2018). "Automatic Software Repair: A Bibliography." *ACM Computing Surveys*, 51(1), Article 17.

6. Gazzola, L., Micucci, D., & Mariani, L. (2019). "Automatic Software Repair: A Survey." *IEEE TSE*, 45(1), pp. 34-67.

### Weak Supervision & ML

7. Ratner, A., Bach, S.H., Ehrenberg, H., Fries, J., Wu, S., & Ré, C. (2017). "Snorkel: Rapid Training Data Creation with Weak Supervision." *VLDB 2017*, pp. 269-282. [Weak supervision EM]

8. Fu, M., & Tantithamthavorn, C. (2022). "LineVul: A Transformer-based Line-Level Vulnerability Prediction." *MSR 2022*, pp. 608-620.

### Bug Prediction & History Mining

9. Śliwerski, J., Zimmermann, T., & Zeller, A. (2005). "When Do Changes Induce Fixes?" *MSR 2005*, pp. 1-5. [SZZ algorithm]

10. Kim, S., Zimmermann, T., Whitehead Jr, E.J., & Zeller, A. (2007). "Predicting Faults from Cached History." *ICSE 2007*, pp. 489-498.

### Continuous Integration & Testing

11. Spieker, H., Gotlieb, A., Marijan, D., & Mossige, M. (2017). "Reinforcement Learning for Automatic Test Case Prioritization and Selection in Continuous Integration." *ISSTA 2017*, pp. 12-22. [RL prioritization]

12. Groce, A., Zhang, C., Eide, E., Chen, Y., & Regehr, J. (2012). "Swarm Testing." *ISSTA 2012*, pp. 78-88. [Swarm testing]

### Code Quality & Technical Debt

13. Bavota, G., & Russo, B. (2016). "A Large-scale Empirical Study on Self-admitted Technical Debt." *MSR 2016*, pp. 315-326. [SATD]

14. Lenarduzzi, V., Saarimäki, N., & Taibi, D. (2019). "The Technical Debt Dataset." *PROMISE 2019*, Article 2.

### Retrieval-Augmented Generation

15. Lewis, P., Perez, E., Piktus, A., Petroni, F., Karpukhin, V., Goyal, N., Küttler, H., Lewis, M., Yih, W., Rocktäschel, T., Riedel, S., & Kiela, D. (2020). "Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks." *NeurIPS 2020*. [RAG foundation]

---

## Appendix A: Error Code → Fix Pattern Mapping

Complete mapping from rustc/clippy error codes to fix templates:

| Code | Pattern ID | Fix Template | Confidence |
|------|------------|--------------|------------|
| E0308 | type_mismatch_cast | `{expr} as {target_type}` | 0.85 |
| E0308 | type_mismatch_into | `{expr}.into()` | 0.90 |
| E0382 | use_after_move_clone | `{var}.clone()` | 0.95 |
| E0382 | use_after_move_borrow | `&{var}` | 0.85 |
| E0502 | mutable_borrow_scope | Split into separate scopes | 0.80 |
| E0499 | multiple_mut_borrow | Use RefCell or restructure | 0.70 |
| E0597 | lifetime_extend | Add lifetime annotation | 0.75 |
| E0597 | lifetime_owned | Return owned value | 0.85 |
| E0277 | trait_impl_missing | `impl {Trait} for {Type}` | 0.60 |
| E0277 | trait_derive | `#[derive({Trait})]` | 0.90 |

---

## Appendix B: PAIML Ecosystem Integration

| Tool | Integration Point | Data Flow |
|------|-------------------|-----------|
| **entrenar** | Pattern store | Read/write .apr fix patterns |
| **trueno-rag** | Knowledge base | Index and query historical bugs |
| **OIP** | Ensemble predictor | Weak supervision for confidence |
| **OIP** | Tarantula SBFL | Fault localization |
| **verificar** | Semantic verification | Mutation testing validation |
| **batuta** | Stack orchestration | Cross-project pattern sharing |
| **aprender** | ML models | RandomForest bug prediction |
| **trueno** | SIMD acceleration | Vector operations for embeddings |

---

## Appendix C: Toyota Way Mapping

| Toyota Principle | Oracle Implementation |
|------------------|----------------------|
| **Jidoka** | Auto-apply with confidence threshold; halt on regression |
| **Kaizen** | Pattern capture from successful fixes; weight updates |
| **Genchi Genbutsu** | Evidence from actual compiler output, not heuristics |
| **Andon** | Stop-the-line on critical regression |
| **Muda** | Eliminate repetitive manual fix cycles |
| **Muri** | Batch size limits prevent overload |
| **Mura** | Consistent quality gates across iterations |
| **Heijunka** | Prioritization smooths fix workload |
| **Nemawashi** | Human review for high-impact changes |
| **Hansei** | Audit logging for retrospective analysis |

---

## Appendix D: Peer-Reviewed Research → Toyota Way Alignment

The following 10 peer-reviewed methodologies form the scientific foundation of the PMAT Oracle, annotated with their alignment to Toyota Way principles:

### 1. Jones et al. (2002) - Tarantula (Fault Localization)
**Principle: Genchi Genbutsu (Go and See)**

By visualizing test execution paths to color-code suspicious statements, Tarantula enables developers to "go and see" the actual location of faults based on empirical evidence rather than speculation, grounding debugging in reality.

### 2. Abreu et al. (2007) - Ochiai (Spectrum-based Fault Localization)
**Principle: Genchi Genbutsu (Facts over Data)**

The Ochiai coefficient provides a statistically rigorous metric for suspiciousness. Using this formula ensures that the "Go and See" process is guided by objective data (facts) derived from test spectrums, minimizing bias in fault identification.

### 3. Le Goues et al. (2012) - GenProg (Automated Program Repair)
**Principle: Jidoka (Autonomation)**

GenProg embodies Jidoka by granting the machine the "intelligence" to propose and verify fixes autonomously. It detects a defect (stop) and attempts to repair it (fix) without immediate human intervention, only escalating when a valid fix is found or if it fails.

### 4. Ratner et al. (2017) - Snorkel (Weak Supervision)
**Principle: Muda (Waste Elimination)**

Manually labeling training data for defect prediction is "muda" (waste). Snorkel's weak supervision eliminates this waste by programmatically generating labels from noisy signals (heuristics, linters), allowing the Oracle to learn efficiently without expensive manual effort.

### 5. Śliwerski et al. (2005) - SZZ Algorithm (Bug Prediction)
**Principle: Hansei (Relentless Reflection)**

The SZZ algorithm analyzes the history of changes to identify bug-inducing commits. This is automated "Hansei"—looking back at past actions (commits) to understand the root cause of current defects and prevent recurrence.

### 6. Bavota & Russo (2016) - Self-Admitted Technical Debt (SATD)
**Principle: Mieruka (Visual Control)**

SATD analysis detects TODOs and FIXMEs, making hidden technical debt visible. This "Visual Control" ensures that debt doesn't accumulate unnoticed, allowing the team to manage it proactively rather than reacting to it later.

### 7. Lewis et al. (2020) - RAG (Retrieval-Augmented Generation)
**Principle: Yokoten (Horizontal Deployment)**

RAG retrieves relevant fix patterns from a knowledge base of historical bugs. This facilitates "Yokoten"—sharing best practices and solutions across the codebase or organization—ensuring that a solution found in one context is available to solve similar problems elsewhere.

### 8. Spieker et al. (2017) - RL for Test Prioritization
**Principle: Heijunka (Leveling)**

Reinforcement Learning prioritizes test cases to detect faults earlier. This levels the testing workload by ensuring the most critical tests run first, smoothing the flow of feedback to developers and preventing "batch and queue" delays.

### 9. Wong et al. (2014) - DStar (Fault Localization)
**Principle: Kaizen (Continuous Improvement)**

DStar represents a refinement over previous coefficients like Ochiai, designed to be more effective in specific contexts (like multiple faults). Integrating DStar demonstrates "Kaizen" applied to the tooling itself—continuously improving the Oracle's diagnostic accuracy.

### 10. Groce et al. (2012) - Swarm Testing
**Principle: Poka-Yoke (Mistake Proofing)**

Swarm testing runs diverse configurations to uncover edge cases that standard suites miss. This acts as a "Poka-Yoke" mechanism, mistake-proofing the verification process by ensuring that even subtle, configuration-dependent bugs are caught before deployment.

---

*Document generated by PAIML Engineering. For questions, contact the pmat-oracle maintainers.*
