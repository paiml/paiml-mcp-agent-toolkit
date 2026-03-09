# Design Document: `pmat repo-score` Implementation

**Status:** Design Phase
**Sprint:** TBD
**Owner:** PAIML Engineering
**Created:** 2025-11-10
**Last Updated:** 2025-11-10

---

## 1. Executive Summary

This document specifies the implementation of `pmat repo-score`, a new PMAT command that evaluates repository health using the scoring system defined in `docs/specifications/components/repo-health.md`.

**Goals:**
- ✅ Provide automated repository quality assessment (0-100 score + bonus)
- ✅ Integrate with existing PMAT infrastructure (validate-readme, quality-gate, etc.)
- ✅ Generate actionable improvement recommendations
- ✅ Support CI/CD integration (JSON output, exit codes)
- ✅ Maintain EXTREME TDD standards (85%+ coverage, <5min test-fast)

**Non-Goals:**
- ❌ External service/API (purely local analysis)
- ❌ Historical trend tracking (future enhancement)
- ❌ Automated fixing (recommendation only)

---

## 2. Architecture Overview

### 2.1 Module Structure

```
server/src/
├── cli/
│   └── handlers/
│       └── repo_score.rs              # CLI command handler
├── services/
│   └── repo_score/
│       ├── mod.rs                      # Public API
│       ├── aggregator.rs               # Score aggregation logic
│       ├── scorers/
│       │   ├── mod.rs                  # Scorer trait + registry
│       │   ├── readme_scorer.rs        # Category A: Documentation
│       │   ├── precommit_scorer.rs     # Category B: Pre-commit hooks
│       │   ├── hygiene_scorer.rs       # Category C: Repository hygiene
│       │   ├── makefile_scorer.rs      # Category D: Build automation
│       │   ├── ci_scorer.rs            # Category E: CI/CD
│       │   └── pmat_scorer.rs          # Category F: PMAT compliance
│       ├── bonus/
│       │   ├── mod.rs                  # Bonus point detectors
│       │   ├── property_test_detector.rs
│       │   ├── fuzzing_detector.rs
│       │   ├── mutation_detector.rs
│       │   └── docs_detector.rs
│       └── models.rs                   # Data structures
└── tests/
    └── repo_score/
        ├── mod.rs
        ├── readme_scorer_tests.rs
        ├── precommit_scorer_tests.rs
        ├── hygiene_scorer_tests.rs
        ├── makefile_scorer_tests.rs
        ├── ci_scorer_tests.rs
        ├── pmat_scorer_tests.rs
        ├── bonus_tests.rs
        ├── aggregator_tests.rs
        └── integration_tests.rs
```

### 2.2 Data Flow

```
┌─────────────────┐
│  CLI Entry      │
│  pmat repo-score│
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│  RepoScoreOrchestrator  │
│  - Parse args           │
│  - Validate path        │
│  - Run scorers          │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  ScorerRegistry         │
│  - Register all scorers │
│  - Execute in parallel  │
└────────┬────────────────┘
         │
         ├──▶ ReadmeScorer (calls validate-readme)
         ├──▶ PrecommitScorer (checks .git/hooks)
         ├──▶ HygieneScorer (scans for cruft)
         ├──▶ MakefileScorer (calls bashrs)
         ├──▶ CiScorer (parses .github/workflows)
         └──▶ PmatScorer (calls quality-gate)
         │
         ▼
┌─────────────────────────┐
│  BonusDetector          │
│  - Property tests       │
│  - Fuzzing              │
│  - Mutation config      │
│  - Living docs          │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  ScoreAggregator        │
│  - Combine scores       │
│  - Apply weights        │
│  - Generate grade       │
│  - Create report        │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  OutputFormatter        │
│  - Text (default)       │
│  - JSON                 │
│  - JUnit XML            │
│  - Badge JSON           │
└────────┬────────────────┘
         │
         ▼
      stdout/file
```

---

## 3. Data Models

### 3.1 Core Structures

```rust
// server/src/services/repo_score/models.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Overall repository score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoScore {
    pub total_score: f64,          // 0-100 base score
    pub bonus_points: f64,         // 0-10 bonus
    pub final_score: f64,          // total + bonus (max 110)
    pub grade: Grade,              // A+, A, A-, B+, etc.
    pub categories: CategoryScores,
    pub bonus: BonusScores,
    pub recommendations: Vec<Recommendation>,
    pub metadata: ScoreMetadata,
}

/// Letter grade assignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Grade {
    APlus,   // 95-110
    A,       // 90-94
    AMinus,  // 85-89
    BPlus,   // 80-84
    B,       // 70-79
    C,       // 60-69
    D,       // 50-59
    F,       // 0-49
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 95.0 => Grade::APlus,
            s if s >= 90.0 => Grade::A,
            s if s >= 85.0 => Grade::AMinus,
            s if s >= 80.0 => Grade::BPlus,
            s if s >= 70.0 => Grade::B,
            s if s >= 60.0 => Grade::C,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::APlus => "A+",
            Grade::A => "A",
            Grade::AMinus => "A-",
            Grade::BPlus => "B+",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }
}

/// Category scores (base 100 points)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScores {
    pub documentation: CategoryScore,     // 20 points
    pub precommit_hooks: CategoryScore,   // 20 points
    pub repository_hygiene: CategoryScore, // 10 points
    pub build_test_automation: CategoryScore, // 25 points
    pub continuous_integration: CategoryScore, // 20 points
    pub pmat_compliance: CategoryScore,   // 5 points
}

impl CategoryScores {
    pub fn total(&self) -> f64 {
        self.documentation.score
            + self.precommit_hooks.score
            + self.repository_hygiene.score
            + self.build_test_automation.score
            + self.continuous_integration.score
            + self.pmat_compliance.score
    }
}

/// Individual category score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub score: f64,           // Earned points
    pub max_score: f64,       // Maximum possible
    pub percentage: f64,      // score/max_score * 100
    pub status: ScoreStatus,  // Pass, Warning, Fail
    pub subcategories: Vec<SubcategoryScore>,
    pub findings: Vec<Finding>,
}

/// Subcategory breakdown (e.g., A1, A2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcategoryScore {
    pub id: String,           // "A1", "A2", etc.
    pub name: String,         // "README Accuracy"
    pub score: f64,
    pub max_score: f64,
    pub findings: Vec<Finding>,
}

/// Bonus points (0-10 max)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusScores {
    pub property_tests: BonusItem,      // +3 max
    pub fuzzing: BonusItem,             // +2 max
    pub mutation_testing: BonusItem,    // +2 max
    pub living_docs: BonusItem,         // +3 max
}

impl BonusScores {
    pub fn total(&self) -> f64 {
        self.property_tests.points
            + self.fuzzing.points
            + self.mutation_testing.points
            + self.living_docs.points
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusItem {
    pub points: f64,
    pub max_points: f64,
    pub detected: bool,
    pub evidence: Vec<String>,
}

/// Finding (positive or negative)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub location: Option<String>,  // File path or line number
    pub impact_points: f64,        // Points lost/gained
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Success,   // ✅ Green - criterion met
    Warning,   // ⚠️  Yellow - partial compliance
    Error,     // ❌ Red - criterion failed
    Info,      // ℹ️  Blue - informational
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScoreStatus {
    Pass,      // ≥90% of max
    Warning,   // 70-89% of max
    Fail,      // <70% of max
}

/// Recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact_points: f64,        // Potential score improvement
    pub estimated_effort: String,  // "15 minutes", "2 hours", "1 week"
    pub commands: Vec<String>,     // Shell commands to execute
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,  // Blocks production readiness
    High,      // Important for quality
    Medium,    // Nice to have
    Low,       // Minor improvement
}

/// Metadata about the scoring run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetadata {
    pub timestamp: String,          // ISO 8601
    pub repository_path: PathBuf,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub pmat_version: String,
    pub spec_version: String,       // "1.0.0"
    pub execution_time_ms: u64,
}
```

### 3.2 Scorer Trait

```rust
// server/src/services/repo_score/scorers/mod.rs

use crate::services::repo_score::models::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Trait for all scoring modules
#[async_trait]
pub trait Scorer: Send + Sync {
    /// Name of the category (e.g., "Documentation Quality")
    fn category_name(&self) -> &str;

    /// Maximum points available for this category
    fn max_score(&self) -> f64;

    /// Execute scoring for this category
    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore>;
}

/// Configuration for scorers
#[derive(Debug, Clone)]
pub struct ScorerConfig {
    pub verbose: bool,
    pub timeout_seconds: u64,
    pub skip_slow_checks: bool,
}

/// Registry of all scorers
pub struct ScorerRegistry {
    scorers: Vec<Box<dyn Scorer>>,
}

impl ScorerRegistry {
    pub fn new() -> Self {
        Self {
            scorers: vec![
                Box::new(ReadmeScorer::new()),
                Box::new(PrecommitScorer::new()),
                Box::new(HygieneScorer::new()),
                Box::new(MakefileScorer::new()),
                Box::new(CiScorer::new()),
                Box::new(PmatScorer::new()),
            ],
        }
    }

    pub async fn score_all(
        &self,
        repo_path: &Path,
        config: &ScorerConfig,
    ) -> Result<CategoryScores> {
        // Execute all scorers in parallel
        let mut handles = vec![];
        for scorer in &self.scorers {
            let path = repo_path.to_path_buf();
            let cfg = config.clone();
            let scorer_clone = scorer.clone(); // Requires Clone on Scorer
            handles.push(tokio::spawn(async move {
                scorer_clone.score(&path, &cfg).await
            }));
        }

        // Collect results
        let results = futures::future::try_join_all(handles).await?;

        // Aggregate into CategoryScores
        Ok(CategoryScores {
            documentation: results[0].clone()?,
            precommit_hooks: results[1].clone()?,
            repository_hygiene: results[2].clone()?,
            build_test_automation: results[3].clone()?,
            continuous_integration: results[4].clone()?,
            pmat_compliance: results[5].clone()?,
        })
    }
}
```

---

## 4. Scorer Implementations

### 4.1 ReadmeScorer (Category A: 20 points)

**Responsibilities:**
- A1: README accuracy (10 points) - Uses existing `validate-readme` command
- A2: README comprehensiveness (10 points) - Checks for required sections

**Implementation Strategy:**

```rust
// server/src/services/repo_score/scorers/readme_scorer.rs

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::models::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;

pub struct ReadmeScorer;

impl ReadmeScorer {
    pub fn new() -> Self {
        Self
    }

    /// Check README accuracy by calling validate-readme
    async fn score_accuracy(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");
        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A1".to_string(),
                name: "README Accuracy".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: "README.md not found".to_string(),
                    location: Some(readme_path.display().to_string()),
                    impact_points: -10.0,
                }],
            });
        }

        // Call existing validate-readme logic
        // TODO: Refactor validate-readme to be library-callable
        let validation_result = self.run_validate_readme(repo_path).await?;

        let mut score = 10.0;
        let mut findings = vec![];

        // Deduct points for broken links
        if validation_result.broken_links > 0 {
            let deduction = (validation_result.broken_links as f64 * 0.5).min(5.0);
            score -= deduction;
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: format!("{} broken links found", validation_result.broken_links),
                location: Some("README.md".to_string()),
                impact_points: -deduction,
            });
        }

        // Deduct points for broken code examples
        if validation_result.broken_examples > 0 {
            let deduction = (validation_result.broken_examples as f64 * 1.0).min(5.0);
            score -= deduction;
            findings.push(Finding {
                severity: Severity::Error,
                category: "Documentation".to_string(),
                message: format!("{} broken code examples", validation_result.broken_examples),
                location: Some("README.md".to_string()),
                impact_points: -deduction,
            });
        }

        Ok(SubcategoryScore {
            id: "A1".to_string(),
            name: "README Accuracy".to_string(),
            score: score.max(0.0),
            max_score: 10.0,
            findings,
        })
    }

    /// Check README comprehensiveness
    async fn score_comprehensiveness(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");
        let content = tokio::fs::read_to_string(&readme_path).await?;

        let required_sections = vec![
            ("Project Description", r"(?i)## (overview|about|description)"),
            ("Installation", r"(?i)## install(ation)?"),
            ("Usage", r"(?i)## (usage|getting started|quick start)"),
            ("License", r"(?i)## license"),
            ("Contributing", r"(?i)## contribut(ing|e)"),
        ];

        let mut score = 0.0;
        let mut findings = vec![];

        for (section_name, regex) in required_sections {
            let re = regex::Regex::new(regex)?;
            if re.is_match(&content) {
                score += 2.0; // 5 sections × 2 points each = 10 points
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Documentation".to_string(),
                    message: format!("{} section found", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 2.0,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Documentation".to_string(),
                    message: format!("{} section missing", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 0.0,
                });
            }
        }

        Ok(SubcategoryScore {
            id: "A2".to_string(),
            name: "README Comprehensiveness".to_string(),
            score,
            max_score: 10.0,
            findings,
        })
    }

    async fn run_validate_readme(&self, repo_path: &Path) -> Result<ValidationResult> {
        // TODO: Call existing validate_readme logic
        // For now, return mock
        Ok(ValidationResult {
            broken_links: 0,
            broken_examples: 0,
        })
    }
}

#[async_trait]
impl Scorer for ReadmeScorer {
    fn category_name(&self) -> &str {
        "Documentation Quality"
    }

    fn max_score(&self) -> f64 {
        20.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        let a1 = self.score_accuracy(repo_path).await?;
        let a2 = self.score_comprehensiveness(repo_path).await?;

        let total_score = a1.score + a2.score;
        let percentage = (total_score / self.max_score()) * 100.0;

        let status = if percentage >= 90.0 {
            ScoreStatus::Pass
        } else if percentage >= 70.0 {
            ScoreStatus::Warning
        } else {
            ScoreStatus::Fail
        };

        let mut findings = a1.findings.clone();
        findings.extend(a2.findings.clone());

        Ok(CategoryScore {
            score: total_score,
            max_score: self.max_score(),
            percentage,
            status,
            subcategories: vec![a1, a2],
            findings,
        })
    }
}

#[derive(Debug)]
struct ValidationResult {
    broken_links: usize,
    broken_examples: usize,
}
```

### 4.2 Other Scorers (High-Level Design)

**PrecommitScorer (Category B: 20 points)**
- Check for `.pre-commit-config.yaml` or `.git/hooks/pre-commit`
- Measure execution time (target: <30s)
- Check lint status (run `make lint` or equivalent)

**HygieneScorer (Category C: 10 points)**
- Scan for cruft files (`.swp`, `.tmp`, `.bak`, `.DS_Store`)
- Scan for team files (`SESSION*.md`, `defect-report-*.txt`)
- Check `.gitignore` coverage

**MakefileScorer (Category D: 25 points)**
- Check for `Makefile` existence
- Run `bashrs lint Makefile` (0-10 points based on warnings)
- Verify standard targets: `test-fast`, `test`, `lint`, `coverage`
- Time `make test-fast` (target: <5 min)
- Time `make coverage` (target: <10 min)

**CiScorer (Category E: 20 points)**
- Check for `.github/workflows/*.yml` files
- Parse workflow files for required elements
- Call GitHub API for build status (if authenticated)
- Count recent passing builds

**PmatScorer (Category F: 5 points)**
- Check for `.pmat-gates.toml` or `pmat-quality.toml`
- Run `pmat quality-gate --checks all` (if available)
- Parse complexity, coverage, SATD settings

---

## 5. CLI Interface

### 5.1 Command Signature

```bash
pmat repo-score [PATH] [OPTIONS]

ARGS:
    <PATH>    Path to repository (default: current directory)

OPTIONS:
    -v, --verbose              Show detailed scoring breakdown
    -o, --output <FORMAT>      Output format: text, json, junit, badge
                               (default: text)
    -f, --output-file <FILE>   Write output to file instead of stdout
    --min-score <SCORE>        Exit with error if score < threshold (0-100)
    --fail-on-grade <GRADE>    Exit with error if grade worse than threshold
                               (A+, A, A-, B+, B, C, D, F)
    --skip-slow                Skip slow checks (e.g., coverage runs)
    --timeout <SECONDS>        Max execution time per scorer (default: 300)
    --no-bonus                 Exclude bonus points from final score
    --recommendations          Generate improvement recommendations
    --help                     Print help information
```

### 5.2 Output Formats

**Text (default):**
```
Repository Score: 94/100 (A)

 ✅ A. Documentation Quality          18/20 (90%)
    ✅ A1. README Accuracy             8/10
    ✅ A2. README Comprehensiveness   10/10

 ⚠️  B. Pre-commit Hooks              18/20 (90%)
    ✅ B1. Best Practices              9/10
    ✅ B2. Performance                 9/10

 ⚠️  C. Repository Hygiene             8/10 (80%)
    ✅ C1. No Cruft                    5/5
    ⚠️  C2. No Team Files               3/5

 ✅ D. Build & Test Automation        22/25 (88%)
    ⚠️  D1. Makefile Quality            8/10
    ✅ D2. Test Performance            8/8
    ✅ D3. Coverage & Mutation         6/7

 ✅ E. Continuous Integration         18/20 (90%)
    ✅ E1. GitHub Actions             10/10
    ⚠️  E2. Build Status               8/10

 ✅ F. PMAT Compliance                 5/5 (100%)
    ✅ F1. Quality Gates               5/5

 🎁 Bonus Points                      +7/10
    ✅ Property-based testing          +3
    ✅ Mutation testing config         +2
    ✅ Living documentation            +2

Grade: A (94/100)
Status: PRODUCTION READY ✅

Recommendations: Run with --recommendations flag
```

**JSON:**
```json
{
  "total_score": 87.0,
  "bonus_points": 7.0,
  "final_score": 94.0,
  "grade": "A",
  "categories": {
    "documentation": {
      "score": 18.0,
      "max_score": 20.0,
      "percentage": 90.0,
      "status": "Pass",
      "subcategories": [
        {
          "id": "A1",
          "name": "README Accuracy",
          "score": 8.0,
          "max_score": 10.0,
          "findings": []
        }
      ]
    }
  },
  "metadata": {
    "timestamp": "2025-11-10T12:34:56Z",
    "repository_path": "/home/user/project",
    "git_branch": "main",
    "pmat_version": "2.192.0",
    "spec_version": "1.0.0",
    "execution_time_ms": 12453
  }
}
```

**JUnit XML (CI integration):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="repo-score" tests="6" failures="0" errors="2">
  <testsuite name="Documentation Quality" tests="2" failures="0">
    <testcase name="A1. README Accuracy" time="0.5">
      <system-out>Score: 8/10 (80%)</system-out>
    </testcase>
    <testcase name="A2. README Comprehensiveness" time="0.3">
      <system-out>Score: 10/10 (100%)</system-out>
    </testcase>
  </testsuite>
  <testsuite name="Repository Hygiene" tests="2" failures="1">
    <testcase name="C1. No Cruft" time="0.1" />
    <testcase name="C2. No Team Files" time="0.1">
      <failure message="2 team-specific files found">
        Found: SESSION-foo.md, defect-report-123.txt
      </failure>
    </testcase>
  </testsuite>
</testsuites>
```

**Badge JSON (shields.io):**
```json
{
  "schemaVersion": 1,
  "label": "repo score",
  "message": "94/100 (A)",
  "color": "brightgreen"
}
```

### 5.3 Exit Codes

```rust
pub enum ExitCode {
    Success = 0,           // Score meets threshold
    ScoreBelowThreshold = 1, // Score < --min-score
    GradeBelowThreshold = 2, // Grade < --fail-on-grade
    ExecutionError = 3,    // Internal error (panic, timeout)
    InvalidArguments = 4,  // Bad CLI arguments
}
```

---

## 6. Integration Points

### 6.1 Reuse Existing PMAT Components

**✅ Leverage existing code:**

1. **validate-readme** → `ReadmeScorer`
   - Location: `server/src/cli/handlers/validate_readme.rs`
   - Refactor to library function: `validate_readme_lib(path) -> ValidationResult`

2. **quality-gate** → `PmatScorer`
   - Location: `server/src/cli/handlers/quality_gate.rs`
   - Refactor to library function: `run_quality_gates(path) -> QualityGateResult`

3. **bashrs integration** → `MakefileScorer`
   - Current: CLI calls `bashrs lint Makefile`
   - Keep shell execution, parse output

4. **git utilities** → `ScoreMetadata`
   - Location: `server/src/utils/git.rs`
   - Use existing `get_current_branch()`, `get_current_commit()`

5. **file utilities** → `HygieneScorer`
   - Location: `server/src/utils/path_validator.rs`
   - Reuse directory traversal logic

### 6.2 New External Dependencies

```toml
# Add to server/Cargo.toml
[dependencies]
# Existing dependencies...
tokio = { version = "1", features = ["full"] }  # Already present
futures = "0.3"                                  # Already present
regex = "1.10"                                   # Already present
serde = { version = "1.0", features = ["derive"] } # Already present
serde_json = "1.0"                               # Already present

# New dependencies (minimal additions)
glob = "0.3"            # For pattern matching (cruft detection)
walkdir = "2.4"         # For directory traversal (already used)
```

**No new external dependencies needed!** All required functionality exists in PMAT.

---

## 7. Testing Strategy

### 7.1 Test Structure

```
server/src/tests/repo_score/
├── mod.rs                        # Test module setup
├── fixtures/                     # Test repositories
│   ├── perfect_repo/            # Score: 100/100
│   ├── good_repo/               # Score: 85-94
│   ├── average_repo/            # Score: 70-84
│   ├── poor_repo/               # Score: <70
│   └── empty_repo/              # Minimal structure
├── readme_scorer_tests.rs
├── precommit_scorer_tests.rs
├── hygiene_scorer_tests.rs
├── makefile_scorer_tests.rs
├── ci_scorer_tests.rs
├── pmat_scorer_tests.rs
├── bonus_tests.rs
├── aggregator_tests.rs
└── integration_tests.rs
```

### 7.2 Test Coverage Requirements

**Target: 85%+ coverage (PMAT standard)**

**Test Types:**

1. **Unit Tests** (70% of tests)
   - Each scorer in isolation
   - Mock file system where possible
   - Test edge cases (missing files, malformed content)

2. **Integration Tests** (25% of tests)
   - End-to-end scoring with fixture repositories
   - CLI argument parsing
   - Output format validation

3. **Property Tests** (5% of tests)
   - Score always 0-110
   - Grade matches score range
   - Sum of subcategories equals category score

### 7.3 Example Test

```rust
// server/src/tests/repo_score/readme_scorer_tests.rs

use crate::services::repo_score::scorers::{ReadmeScorer, Scorer, ScorerConfig};
use crate::services::repo_score::models::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_readme_scorer_missing_file() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    let scorer = ReadmeScorer::new();
    let config = ScorerConfig {
        verbose: false,
        timeout_seconds: 60,
        skip_slow_checks: false,
    };

    // ACT
    let result = scorer.score(repo_path, &config).await.unwrap();

    // ASSERT
    assert_eq!(result.score, 0.0);
    assert_eq!(result.max_score, 20.0);
    assert_eq!(result.status, ScoreStatus::Fail);
    assert!(result.findings.iter().any(|f| f.message.contains("README.md not found")));
}

#[tokio::test]
async fn test_readme_scorer_perfect_readme() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create perfect README
    let readme_content = r#"
# Project Name

## Overview
This is a test project.

## Installation
```bash
cargo install test
```

## Usage
Run `test --help`

## Contributing
See CONTRIBUTING.md

## License
MIT License
"#;
    std::fs::write(repo_path.join("README.md"), readme_content).unwrap();

    let scorer = ReadmeScorer::new();
    let config = ScorerConfig {
        verbose: false,
        timeout_seconds: 60,
        skip_slow_checks: false,
    };

    // ACT
    let result = scorer.score(repo_path, &config).await.unwrap();

    // ASSERT
    assert_eq!(result.score, 20.0);
    assert_eq!(result.status, ScoreStatus::Pass);
}

#[tokio::test]
async fn test_readme_scorer_partial_content() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create README with only some sections
    let readme_content = r#"
# Project Name

## Installation
```bash
cargo install test
```
"#;
    std::fs::write(repo_path.join("README.md"), readme_content).unwrap();

    let scorer = ReadmeScorer::new();
    let config = ScorerConfig {
        verbose: false,
        timeout_seconds: 60,
        skip_slow_checks: false,
    };

    // ACT
    let result = scorer.score(repo_path, &config).await.unwrap();

    // ASSERT
    // Should get 10 points for accuracy (no broken links)
    // Should get 2-4 points for partial comprehensiveness
    assert!(result.score >= 12.0 && result.score <= 14.0);
    assert_eq!(result.status, ScoreStatus::Warning);
}
```

### 7.4 Test Fixtures

```bash
# Create test fixture repositories
server/src/tests/repo_score/fixtures/perfect_repo/
├── README.md               # Complete documentation
├── Makefile                # All targets, bashrs clean
├── .pmat-gates.toml        # Quality gates configured
├── .github/workflows/
│   └── ci.yml             # CI configured
└── .git/hooks/
    └── pre-commit         # Hooks installed

server/src/tests/repo_score/fixtures/poor_repo/
├── README.txt             # Wrong filename
├── cruft.swp              # Cruft file
└── SESSION-123.md         # Team file
```

---

## 8. Performance Requirements

### 8.1 Execution Time

**Target: <30 seconds for full repo-score run**

Breakdown:
- ReadmeScorer: <5s (validate-readme already fast)
- PrecommitScorer: <3s (file checks only)
- HygieneScorer: <2s (directory scan)
- MakefileScorer: <10s (bashrs lint + target checks)
- CiScorer: <5s (file parsing, optional API call)
- PmatScorer: <5s (read config files)
- BonusDetector: <5s (pattern matching)
- Aggregation: <1s

**Optimizations:**
- ✅ Parallel execution of scorers (tokio::spawn)
- ✅ Skip slow checks with `--skip-slow` flag
- ✅ Timeout per scorer (default: 300s, configurable)
- ✅ Cache results for repeated runs (future enhancement)

### 8.2 Memory Usage

**Target: <100 MB peak memory**

- Streaming file reads (no full content buffering)
- Limit concurrent scorers (6 max)
- Drop findings after aggregation

---

## 9. Error Handling

### 9.1 Error Types

```rust
// server/src/services/repo_score/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoScoreError {
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Not a git repository: {0}")]
    NotGitRepository(String),

    #[error("Scorer '{0}' timed out after {1}s")]
    ScorerTimeout(String, u64),

    #[error("Scorer '{0}' failed: {1}")]
    ScorerFailed(String, String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("External command failed: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, RepoScoreError>;
```

### 9.2 Graceful Degradation

**Philosophy: Score what we can, report what we can't**

```rust
impl ScorerRegistry {
    pub async fn score_all_with_fallback(
        &self,
        repo_path: &Path,
        config: &ScorerConfig,
    ) -> CategoryScores {
        let mut scores = CategoryScores::default();

        for scorer in &self.scorers {
            match scorer.score(repo_path, config).await {
                Ok(score) => {
                    // Success - use score
                    scores.set_category(scorer.category_name(), score);
                }
                Err(e) => {
                    // Failure - award 0 points, log error
                    eprintln!("⚠️  Scorer '{}' failed: {}", scorer.category_name(), e);
                    scores.set_category(scorer.category_name(), CategoryScore {
                        score: 0.0,
                        max_score: scorer.max_score(),
                        percentage: 0.0,
                        status: ScoreStatus::Fail,
                        subcategories: vec![],
                        findings: vec![Finding {
                            severity: Severity::Error,
                            category: scorer.category_name().to_string(),
                            message: format!("Scorer failed: {}", e),
                            location: None,
                            impact_points: -scorer.max_score(),
                        }],
                    });
                }
            }
        }

        scores
    }
}
```

---

## 10. Implementation Plan

### Phase 1: Foundation (Week 1)

**Goal: Data models + CLI skeleton**

- [ ] Create `server/src/services/repo_score/` module structure
- [ ] Implement all data models in `models.rs`
- [ ] Implement `Grade::from_score()` logic
- [ ] Create CLI command skeleton in `cli/handlers/repo_score.rs`
- [ ] Add basic argument parsing (clap)
- [ ] Write unit tests for models (100% coverage)

**Deliverable:** `pmat repo-score .` prints "Not implemented"

### Phase 2: Core Scorers (Week 2-3)

**Goal: Implement all 6 base scorers**

Week 2:
- [ ] Implement `ReadmeScorer` (use existing validate-readme)
- [ ] Implement `HygieneScorer` (file scanning)
- [ ] Implement `MakefileScorer` (bashrs integration)
- [ ] Write unit tests for each (85%+ coverage)

Week 3:
- [ ] Implement `PrecommitScorer` (hook checks)
- [ ] Implement `CiScorer` (GitHub Actions parsing)
- [ ] Implement `PmatScorer` (quality-gate integration)
- [ ] Write unit tests for each (85%+ coverage)

**Deliverable:** `pmat repo-score .` prints base score (0-100)

### Phase 3: Bonus Points + Aggregation (Week 4)

**Goal: Bonus detection + final scoring**

- [ ] Implement `BonusDetector` module
- [ ] Implement property test detector
- [ ] Implement fuzzing detector
- [ ] Implement mutation test detector
- [ ] Implement living docs detector
- [ ] Implement `ScoreAggregator`
- [ ] Write integration tests with fixtures

**Deliverable:** `pmat repo-score .` prints final score with bonus

### Phase 4: Output Formats + Recommendations (Week 5)

**Goal: Professional output + CI integration**

- [ ] Implement text formatter (default)
- [ ] Implement JSON formatter
- [ ] Implement JUnit XML formatter
- [ ] Implement badge JSON formatter
- [ ] Implement recommendation generator
- [ ] Add `--min-score` and `--fail-on-grade` logic
- [ ] Write end-to-end CLI tests

**Deliverable:** Full `pmat repo-score` functionality

### Phase 5: Documentation + Polish (Week 6)

**Goal: Production-ready release**

- [ ] Add pmat-book chapter (Chapter 15: Repository Scoring)
- [ ] Update README.md with repo-score examples
- [ ] Add CI/CD integration guide
- [ ] Add example GitHub Actions workflow
- [ ] Conduct dogfood testing (score PMAT itself)
- [ ] Performance profiling + optimization
- [ ] Final QA + bug fixes

**Deliverable:** v2.193.0 release with repo-score

---

## 11. Open Questions

### Q1: Should scorers be plugin-based?

**Decision:** NO (for MVP)
- Rationale: 6 scorers is manageable, plugin system adds complexity
- Future: Could add plugin system in v2.0 if users request custom scorers

### Q2: Should we cache scoring results?

**Decision:** NO (for MVP)
- Rationale: Scoring is fast (<30s), caching adds complexity
- Future: Could add `.pmat/repo-score-cache.json` if needed

### Q3: Should we support historical trend tracking?

**Decision:** NO (for MVP)
- Rationale: TDG history already exists, avoid duplication
- Future: Could integrate with `pmat tdg history` in v2.0

### Q4: Should we call GitHub API for build status?

**Decision:** YES (optional)
- Rationale: Build status is important, but API requires auth
- Implementation: Try API call, fall back to local checks if unauthenticated

### Q5: Should we auto-fix issues?

**Decision:** NO
- Rationale: Recommendation-only keeps scope manageable
- Future: Could add `pmat repo-score --auto-fix` in v3.0

---

## 12. Success Metrics

### 12.1 Development Metrics

- [ ] **Test Coverage:** ≥85% for all repo_score modules
- [ ] **Test Performance:** `make test-fast` still <5 minutes
- [ ] **Code Quality:** Zero clippy warnings, complexity ≤10
- [ ] **Documentation:** Full pmat-book chapter + API docs

### 12.2 User Metrics (Post-Launch)

- [ ] **Adoption:** 50+ GitHub repos using `pmat repo-score` (3 months)
- [ ] **CI Integration:** 20+ repos with repo-score in CI (6 months)
- [ ] **Badge Usage:** 10+ repos displaying repo-score badges (6 months)
- [ ] **Community:** 5+ external contributions to scoring logic (12 months)

### 12.3 Quality Metrics

- [ ] **Dogfooding:** paiml-mcp-agent-toolkit scores ≥90 (A)
- [ ] **Consistency:** Score variance <5 points across 10 runs
- [ ] **Performance:** <30s execution time on 10,000-file repo

---

## 13. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Scoring takes >60s | Medium | High | Parallel execution, timeouts, --skip-slow |
| bashrs not installed | Low | Medium | Graceful degradation, suggest installation |
| GitHub API rate limits | Medium | Low | Optional, fall back to local checks |
| Test fixtures too large | Low | Low | Keep fixtures minimal (<100 files each) |
| Spec changes after implementation | Medium | High | Implement against spec v1.0.0, version carefully |

---

## 14. Future Enhancements (Out of Scope)

**v2.1.0+ (6-12 months)**

1. **Plugin System**
   - Custom scorers via `~/.pmat/scorers/`
   - Example: `OrganizationPolicyScorer`

2. **Historical Trends**
   - Integration with `pmat tdg history`
   - Visualize score over time

3. **Auto-Fix Mode**
   - `pmat repo-score --auto-fix`
   - Apply recommendations automatically

4. **Team Dashboard**
   - Web UI showing all repos' scores
   - Compare across organization

5. **Badge Service**
   - Hosted service: `https://pmat.io/badge/USER/REPO`
   - Real-time score updates

6. **Benchmark Mode**
   - Compare against top 100 GitHub repos
   - Industry percentile ranking

---

## Appendix A: File Locations

**New Files to Create:**

```
server/src/
├── cli/handlers/repo_score.rs                        # CLI handler
├── services/repo_score/
│   ├── mod.rs                                         # Public API
│   ├── models.rs                                      # Data structures (600 lines)
│   ├── error.rs                                       # Error types
│   ├── aggregator.rs                                  # Score aggregation
│   ├── orchestrator.rs                                # Main logic
│   ├── formatters/
│   │   ├── mod.rs
│   │   ├── text.rs                                    # Text output
│   │   ├── json.rs                                    # JSON output
│   │   ├── junit.rs                                   # JUnit XML
│   │   └── badge.rs                                   # Badge JSON
│   ├── scorers/
│   │   ├── mod.rs                                     # Scorer trait + registry
│   │   ├── readme_scorer.rs                           # Category A (400 lines)
│   │   ├── precommit_scorer.rs                        # Category B (300 lines)
│   │   ├── hygiene_scorer.rs                          # Category C (200 lines)
│   │   ├── makefile_scorer.rs                         # Category D (350 lines)
│   │   ├── ci_scorer.rs                               # Category E (300 lines)
│   │   └── pmat_scorer.rs                             # Category F (150 lines)
│   └── bonus/
│       ├── mod.rs                                     # Bonus detector
│       ├── property_test_detector.rs                  # +3 points
│       ├── fuzzing_detector.rs                        # +2 points
│       ├── mutation_detector.rs                       # +2 points
│       └── docs_detector.rs                           # +3 points
└── tests/repo_score/
    ├── mod.rs
    ├── fixtures/                                      # Test repos
    ├── readme_scorer_tests.rs                         # 200 lines
    ├── precommit_scorer_tests.rs                      # 150 lines
    ├── hygiene_scorer_tests.rs                        # 150 lines
    ├── makefile_scorer_tests.rs                       # 200 lines
    ├── ci_scorer_tests.rs                             # 150 lines
    ├── pmat_scorer_tests.rs                           # 100 lines
    ├── bonus_tests.rs                                 # 150 lines
    ├── aggregator_tests.rs                            # 150 lines
    └── integration_tests.rs                           # 300 lines

Total: ~5,000 lines of new code
```

**Modified Files:**

```
server/src/
├── cli/mod.rs                                         # Add repo_score module
├── cli/handlers/mod.rs                                # Export repo_score handler
└── main.rs                                            # Register repo-score command

server/Cargo.toml                                      # Add dependencies (if any)

docs/
├── specifications/components/repo-health.md           # Already created ✅
└── design/repo-score-implementation.md                # This document ✅
```

---

**Document Status:** ✅ READY FOR REVIEW
**Next Step:** TDD test writing (Phase 2)
**Estimated Implementation Time:** 6 weeks (1 engineer full-time)
**Risk Level:** LOW (leverages existing PMAT infrastructure)
