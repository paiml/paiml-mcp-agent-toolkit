# PMAT Debug Command - Five Whys Root Cause Analysis

**Status**: Implementation (Sprint N+1)
**Created**: 2025-11-24
**Methodology**: EXTREME TDD + Toyota Way Principles

## Table of Contents

1. [Overview](#overview)
2. [Toyota Way Principles](#toyota-way-principles)
3. [Command Specification](#command-specification)
4. [Five Whys Algorithm](#five-whys-algorithm)
5. [PMAT Tool Integration](#pmat-tool-integration)
6. [Output Format](#output-format)
7. [Implementation Plan](#implementation-plan)
8. [Test Cases](#test-cases)
9. [Success Criteria](#success-criteria)

---

## Overview

Implement `pmat debug` command that applies Toyota's Five Whys methodology to systematically find root causes of software defects. Integrates with existing PMAT tooling (TDG, complexity, dead code, SATD detection) to provide evidence-based debugging guidance.

**Core Principle**: Five Whys is the ONLY acceptable debugging method (per CLAUDE.md policy).

### Problem Statement

Developers often use ineffective debugging approaches:
- ❌ Random print statements
- ❌ Guessing root cause
- ❌ Applying fixes without understanding

**Solution**: Structured Five Whys analysis with PMAT evidence.

---

## Toyota Way Principles

### 1. Genchi Genbutsu (Go and See)
- Analyze actual code using PMAT tools
- Gather evidence from TDG, complexity, SATD analysis
- Don't guess - measure

### 2. Jidoka (Built-in Quality)
- Automated analysis prevents hallucinations
- Evidence-based recommendations
- No speculation without data

### 3. Kaizen (Continuous Improvement)
- Each Five Whys session improves understanding
- Learn from root causes
- Document patterns

### 4. Nemawashi (Consensus Building)
- Output format supports team discussion
- Evidence visible to all stakeholders
- Reproducible analysis

---

## Command Specification

### Syntax

```bash
pmat debug <issue-description> [OPTIONS]
```

### Arguments

- `<issue-description>`: Text description of the issue (required)
  - Examples:
    - "thread panicked: stack overflow in test_multiple_parameter_types"
    - "API returns 500 on POST /users endpoint"
    - "Memory leak in background worker process"

### Options

- `--path <PATH>`: Project path (default: current directory)
- `--depth <N>`: Number of "why" iterations (default: 5, max: 10)
- `--format <FORMAT>`: Output format: text, json, markdown (default: text)
- `--output <FILE>`: Write analysis to file
- `--context <FILE>`: Use deep context file for enhanced analysis
- `--auto-analyze`: Automatically analyze suspected files with PMAT tools
- `--interactive`: Interactive mode with prompts for each "why"

### Examples

```bash
# Basic usage
pmat debug "Stack overflow in recursive parser"

# With auto-analysis
pmat debug "High memory usage in worker" --auto-analyze

# Generate markdown report
pmat debug "API timeout" --format markdown --output debug-report.md

# Interactive mode
pmat debug "Test failure in integration suite" --interactive

# With existing deep context
pmat debug "Performance regression" --context deep_context.md
```

---

## Five Whys Algorithm

### Core Process

For each iteration (1-5 or --depth):
1. **Ask "Why?"** - Why did this symptom occur?
2. **Gather Evidence** - Use PMAT tools to analyze
3. **Formulate Hypothesis** - Based on evidence, not guessing
4. **Validate** - Check hypothesis against codebase facts
5. **Document** - Record "why" and supporting evidence

### Evidence Gathering Strategy

For each "why" iteration, automatically gather:

**1. Code Complexity Analysis**
```bash
pmat analyze <suspected-files> --format json
```
- Cyclomatic complexity > 20 → Root cause: "Excessive complexity"
- Cognitive complexity → Indicates hard-to-understand code

**2. SATD Detection**
```bash
pmat satd <files> --format json
```
- TODO/FIXME/HACK comments → Known technical debt
- Link to defect likelihood

**3. Dead Code Analysis**
```bash
pmat dead-code <path> --format json
```
- Unused functions/variables → Maintenance burden
- Indicates poorly understood codebase

**4. Git Analysis**
```bash
pmat git-churn <path> --since 30d
```
- High churn → Unstable code
- Recent changes → Likely culprit

**5. TDG Score**
```bash
pmat tdg-score <file> --format json
```
- Low TDG → Poor test coverage
- Indicates fragile code

### Heuristics for Root Cause

| Symptom | Likely Root Cause (Why 5) | Evidence |
|---------|---------------------------|----------|
| Stack overflow | Unbounded recursion | Complexity > 50, no base case |
| Memory leak | Resource not freed | SATD: "TODO: cleanup", no Drop impl |
| Test flakiness | Race condition | High complexity + concurrency |
| API timeout | N+1 queries | TDG score < 50, no pagination |
| Compilation error | Dependency conflict | Cargo.lock changes, version mismatch |

---

## PMAT Tool Integration

### Service Dependencies

The debug command integrates with:

1. **`complexity` service** - Cyclomatic/cognitive complexity
2. **`satd_detector` service** - Technical debt detection
3. **`dead_code_analyzer` service** - Unused code detection
4. **`git_analysis` service** - Churn and history analysis
5. **`tdg_calculator` service** - Test quality scoring
6. **`deep_context` service** - Semantic code understanding

### Analysis Pipeline

```rust
struct DebugAnalysis {
    issue: String,
    whys: Vec<WhyIteration>,
    root_cause: Option<String>,
    recommendations: Vec<String>,
    evidence: AnalysisEvidence,
}

struct WhyIteration {
    depth: u8,
    question: String,
    hypothesis: String,
    evidence: Vec<Evidence>,
    confidence: f64, // 0.0-1.0
}

struct Evidence {
    source: EvidenceSource, // Complexity, SATD, DeadCode, Git, TDG
    file: PathBuf,
    metric: String,
    value: serde_json::Value,
    interpretation: String,
}

enum EvidenceSource {
    Complexity,
    SATD,
    DeadCode,
    GitChurn,
    TDG,
    ManualInspection,
}
```

### Example Analysis Flow

**Issue**: "Stack overflow in test_multiple_parameter_types"

**Why 1**: Why did the test crash?
- **Evidence**: Stack overflow error in recursive function
- **Hypothesis**: Recursion exceeded stack limit
- **Confidence**: 0.95

**Why 2**: Why did recursion exceed limit?
- **Evidence**: Complexity analysis shows cyclomatic = 48
- **Evidence**: No termination condition visible
- **Hypothesis**: Deep AST traversal without tail recursion
- **Confidence**: 0.85

**Why 3**: Why is AST traversal so deep?
- **Evidence**: Parser generates deeply nested nodes
- **Evidence**: Grammar allows unlimited nesting
- **Hypothesis**: Parser lacks depth validation
- **Confidence**: 0.75

**Why 4**: Why does grammar allow unlimited nesting?
- **Evidence**: SATD comment: "TODO: Add max depth check"
- **Evidence**: TDG score: 42/100 (low test coverage)
- **Hypothesis**: Missing requirement, not tested
- **Confidence**: 0.70

**Why 5**: Why was requirement missing?
- **Evidence**: No specification document found
- **Evidence**: High git churn (23 commits in 14 days)
- **Root Cause**: Parser implemented without depth constraints
- **Confidence**: 0.65

**Recommendation**: Add `max_depth: 1000` parameter to parser config

---

## Output Format

### Text Format (Default)

```
🔍 PMAT Five Whys Root Cause Analysis

Issue: Stack overflow in test_multiple_parameter_types

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Why 1: Why did the test crash?
   ❓ Question: What caused the stack overflow?
   💡 Hypothesis: Recursion exceeded stack limit
   📊 Evidence:
      • Stack trace shows recursive call depth > 10,000
      • File: server/src/parser/ast.rs:142
   ✅ Confidence: 95%

Why 2: Why did recursion exceed limit?
   ❓ Question: Why is recursion unbounded?
   💡 Hypothesis: Deep AST traversal without tail recursion
   📊 Evidence:
      • Complexity: cyclomatic = 48 (threshold: 20)
      • SATD: "FIXME: Optimize recursive calls" (line 135)
      • File: server/src/parser/ast.rs
   ✅ Confidence: 85%

Why 3: Why is AST traversal so deep?
   ❓ Question: Why does parser allow deep nesting?
   💡 Hypothesis: Parser lacks depth validation
   📊 Evidence:
      • Grammar allows unlimited nesting
      • No max_depth parameter in config
      • File: server/src/parser/grammar.pest
   ✅ Confidence: 75%

Why 4: Why does grammar allow unlimited nesting?
   ❓ Question: Why wasn't depth limit specified?
   💡 Hypothesis: Missing requirement, not tested
   📊 Evidence:
      • TDG Score: 42/100 (low test coverage)
      • No tests for deep nesting edge case
      • File: server/tests/parser_tests.rs
   ✅ Confidence: 70%

Why 5: Why was requirement missing?
   ❓ Question: Why wasn't edge case considered?
   💡 Root Cause: Parser implemented without depth constraints
   📊 Evidence:
      • No specification document found
      • High git churn: 23 commits in 14 days
      • SATD: "TODO: Add parser limits" (parser/mod.rs:12)
   ✅ Confidence: 65%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎯 Root Cause:
   Parser implemented without depth validation constraints

💡 Recommendations:
   1. Add max_depth: 1000 parameter to ParserConfig
   2. Implement depth tracking in AST traversal
   3. Add test case for deeply nested input (RED phase)
   4. Document parser limits in specification
   5. Add SATD resolution to next sprint

📁 Affected Files:
   • server/src/parser/ast.rs (primary)
   • server/src/parser/grammar.pest (config)
   • server/tests/parser_tests.rs (missing tests)

📊 Evidence Summary:
   • Complexity violations: 1 file
   • SATD markers: 2 instances
   • TDG score: 42/100 (needs improvement)
   • Git churn: HIGH (23 commits, 14 days)

⏱️  Analysis completed in 2.3 seconds
```

### JSON Format (--format json)

```json
{
  "issue": "Stack overflow in test_multiple_parameter_types",
  "timestamp": "2025-11-24T10:30:00Z",
  "duration_ms": 2300,
  "whys": [
    {
      "depth": 1,
      "question": "Why did the test crash?",
      "hypothesis": "Recursion exceeded stack limit",
      "confidence": 0.95,
      "evidence": [
        {
          "source": "StackTrace",
          "file": "server/src/parser/ast.rs",
          "line": 142,
          "metric": "recursive_depth",
          "value": 10000,
          "interpretation": "Stack overflow at depth > 10,000"
        }
      ]
    },
    {
      "depth": 2,
      "question": "Why did recursion exceed limit?",
      "hypothesis": "Deep AST traversal without tail recursion",
      "confidence": 0.85,
      "evidence": [
        {
          "source": "Complexity",
          "file": "server/src/parser/ast.rs",
          "metric": "cyclomatic_complexity",
          "value": 48,
          "threshold": 20,
          "interpretation": "Excessive complexity indicates complex control flow"
        },
        {
          "source": "SATD",
          "file": "server/src/parser/ast.rs",
          "line": 135,
          "metric": "fixme_marker",
          "value": "FIXME: Optimize recursive calls",
          "interpretation": "Known technical debt related to recursion"
        }
      ]
    }
  ],
  "root_cause": {
    "depth": 5,
    "description": "Parser implemented without depth validation constraints",
    "confidence": 0.65
  },
  "recommendations": [
    {
      "priority": "high",
      "action": "Add max_depth: 1000 parameter to ParserConfig",
      "file": "server/src/parser/config.rs"
    },
    {
      "priority": "high",
      "action": "Implement depth tracking in AST traversal",
      "file": "server/src/parser/ast.rs"
    },
    {
      "priority": "medium",
      "action": "Add test case for deeply nested input",
      "file": "server/tests/parser_tests.rs"
    }
  ],
  "affected_files": [
    "server/src/parser/ast.rs",
    "server/src/parser/grammar.pest",
    "server/tests/parser_tests.rs"
  ],
  "evidence_summary": {
    "complexity_violations": 1,
    "satd_markers": 2,
    "tdg_score": 42.0,
    "git_churn_high": true
  }
}
```

### Markdown Format (--format markdown)

```markdown
# Five Whys Root Cause Analysis

**Issue**: Stack overflow in test_multiple_parameter_types
**Date**: 2025-11-24
**Analysis Time**: 2.3 seconds

---

## Why 1: Why did the test crash?

**Hypothesis**: Recursion exceeded stack limit
**Confidence**: 95%

**Evidence**:
- Stack trace shows recursive call depth > 10,000
- Location: `server/src/parser/ast.rs:142`

---

## Why 2: Why did recursion exceed limit?

**Hypothesis**: Deep AST traversal without tail recursion
**Confidence**: 85%

**Evidence**:
- **Complexity**: cyclomatic = 48 (threshold: 20) ⚠️
- **SATD**: "FIXME: Optimize recursive calls" (line 135)
- Location: `server/src/parser/ast.rs`

---

## Why 5: Why was requirement missing?

**Root Cause**: Parser implemented without depth validation constraints
**Confidence**: 65%

**Evidence**:
- No specification document found
- High git churn: 23 commits in 14 days
- SATD: "TODO: Add parser limits" (`parser/mod.rs:12`)

---

## Recommendations

1. **HIGH**: Add `max_depth: 1000` parameter to ParserConfig
2. **HIGH**: Implement depth tracking in AST traversal
3. **MEDIUM**: Add test case for deeply nested input (RED phase)
4. **LOW**: Document parser limits in specification

---

## Evidence Summary

| Metric | Value | Status |
|--------|-------|--------|
| Complexity violations | 1 | ⚠️ |
| SATD markers | 2 | ⚠️ |
| TDG score | 42/100 | ❌ |
| Git churn | 23 commits (14d) | ⚠️ HIGH |

**Affected Files**:
- `server/src/parser/ast.rs` (primary)
- `server/src/parser/grammar.pest`
- `server/tests/parser_tests.rs`
```

---

## Implementation Plan

### Phase 1: RED - Tests First (EXTREME TDD)

**File**: `server/tests/debug_command_tests.rs`

Create 12+ tests covering:
1. Basic Five Whys execution (depth=5)
2. Custom depth (depth=3, depth=10)
3. Evidence gathering from each service
4. Output format validation (text, json, markdown)
5. File analysis integration
6. Confidence scoring algorithm
7. Recommendation generation
8. Error handling (no files found, service unavailable)
9. Interactive mode prompts
10. Auto-analyze flag behavior
11. Deep context integration
12. Git churn correlation

### Phase 2: GREEN - Minimal Implementation

**Files to create**:
1. `server/src/services/five_whys_analyzer.rs` (core logic)
2. `server/src/cli/commands/debug.rs` (CLI interface)
3. `server/src/cli/handlers/debug_handlers.rs` (handler)
4. `server/src/models/debug_analysis.rs` (data structures)

**Core Services**:

```rust
// server/src/services/five_whys_analyzer.rs

pub struct FiveWhysAnalyzer {
    complexity_service: Arc<dyn ComplexityAnalyzer>,
    satd_detector: Arc<SatdDetector>,
    dead_code_analyzer: Arc<DeadCodeAnalyzer>,
    git_analyzer: Arc<GitAnalysisService>,
    tdg_calculator: Arc<TdgCalculator>,
}

impl FiveWhysAnalyzer {
    pub async fn analyze(
        &self,
        issue: &str,
        path: &Path,
        depth: u8,
    ) -> Result<DebugAnalysis> {
        let mut whys = Vec::new();
        
        for i in 1..=depth {
            let why = self.iterate_why(issue, path, i, &whys).await?;
            whys.push(why);
            
            // Early termination if high confidence root cause found
            if why.confidence > 0.9 && i >= 3 {
                break;
            }
        }
        
        let root_cause = self.extract_root_cause(&whys)?;
        let recommendations = self.generate_recommendations(&whys, &root_cause)?;
        
        Ok(DebugAnalysis {
            issue: issue.to_string(),
            whys,
            root_cause: Some(root_cause),
            recommendations,
            evidence: self.summarize_evidence(&whys)?,
        })
    }
    
    async fn iterate_why(
        &self,
        issue: &str,
        path: &Path,
        depth: u8,
        previous_whys: &[WhyIteration],
    ) -> Result<WhyIteration> {
        // 1. Formulate question based on previous iteration
        let question = self.formulate_question(issue, depth, previous_whys)?;
        
        // 2. Gather evidence from PMAT services
        let evidence = self.gather_evidence(path).await?;
        
        // 3. Generate hypothesis based on evidence
        let hypothesis = self.generate_hypothesis(&question, &evidence)?;
        
        // 4. Calculate confidence score
        let confidence = self.calculate_confidence(&evidence)?;
        
        Ok(WhyIteration {
            depth,
            question,
            hypothesis,
            evidence,
            confidence,
        })
    }
    
    async fn gather_evidence(&self, path: &Path) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();
        
        // Parallel evidence gathering
        let (complexity, satd, dead_code, git_churn, tdg) = tokio::join!(
            self.analyze_complexity(path),
            self.detect_satd(path),
            self.find_dead_code(path),
            self.analyze_git_churn(path),
            self.calculate_tdg(path),
        );
        
        if let Ok(c) = complexity {
            evidence.extend(c);
        }
        if let Ok(s) = satd {
            evidence.extend(s);
        }
        if let Ok(d) = dead_code {
            evidence.extend(d);
        }
        if let Ok(g) = git_churn {
            evidence.extend(g);
        }
        if let Ok(t) = tdg {
            evidence.extend(t);
        }
        
        Ok(evidence)
    }
}
```

### Phase 3: REFACTOR - Integration & Polish

**Integrations**:
1. Add to `server/src/cli/commands/mod.rs`
2. Wire up handler in `server/src/cli/mod.rs`
3. Register service in `server/src/services/mod.rs`
4. Add clap subcommand in `server/src/cli/app.rs`

**Polish**:
1. Colored terminal output (termcolor)
2. Progress indicators for long analysis
3. Caching for repeated analysis
4. Export to file formats

---

## Test Cases

### Test 1: Basic Five Whys Execution
```rust
#[tokio::test]
async fn test_five_whys_basic_execution() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer.analyze(
        "Stack overflow in parser",
        Path::new("test_fixtures/parser"),
        5,
    ).await.unwrap();
    
    assert_eq!(result.whys.len(), 5);
    assert!(result.root_cause.is_some());
    assert!(!result.recommendations.is_empty());
}
```

### Test 2: Evidence Gathering
```rust
#[tokio::test]
async fn test_evidence_gathering_all_services() {
    let analyzer = FiveWhysAnalyzer::new();
    let evidence = analyzer.gather_evidence(
        Path::new("test_fixtures/complex_code")
    ).await.unwrap();
    
    // Should have evidence from all 5 services
    let sources: HashSet<_> = evidence.iter()
        .map(|e| e.source)
        .collect();
    
    assert!(sources.contains(&EvidenceSource::Complexity));
    assert!(sources.contains(&EvidenceSource::SATD));
    assert!(sources.contains(&EvidenceSource::DeadCode));
    assert!(sources.contains(&EvidenceSource::GitChurn));
    assert!(sources.contains(&EvidenceSource::TDG));
}
```

### Test 3: Confidence Scoring
```rust
#[tokio::test]
async fn test_confidence_scoring_high_evidence() {
    let analyzer = FiveWhysAnalyzer::new();
    
    let evidence = vec![
        Evidence {
            source: EvidenceSource::Complexity,
            metric: "cyclomatic".to_string(),
            value: json!({"value": 50, "threshold": 20}),
            interpretation: "High complexity".to_string(),
            ..Default::default()
        },
        Evidence {
            source: EvidenceSource::SATD,
            metric: "fixme_count".to_string(),
            value: json!(3),
            interpretation: "Multiple FIXME markers".to_string(),
            ..Default::default()
        },
    ];
    
    let confidence = analyzer.calculate_confidence(&evidence).unwrap();
    assert!(confidence > 0.7); // High evidence → high confidence
}
```

### Test 4: Output Format Validation
```rust
#[tokio::test]
async fn test_output_format_text() {
    let analysis = create_test_analysis();
    let output = format_text(&analysis).unwrap();
    
    assert!(output.contains("🔍 PMAT Five Whys"));
    assert!(output.contains("Why 1:"));
    assert!(output.contains("Root Cause:"));
    assert!(output.contains("Recommendations:"));
}

#[tokio::test]
async fn test_output_format_json() {
    let analysis = create_test_analysis();
    let output = format_json(&analysis).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    
    assert!(parsed["whys"].is_array());
    assert_eq!(parsed["whys"].as_array().unwrap().len(), 5);
}
```

### Test 5: Recommendation Generation
```rust
#[tokio::test]
async fn test_recommendation_generation() {
    let analyzer = FiveWhysAnalyzer::new();
    let whys = vec![
        create_why_with_evidence(EvidenceSource::Complexity, 50.0),
        create_why_with_evidence(EvidenceSource::SATD, 5.0),
    ];
    let root_cause = "High complexity without tests".to_string();
    
    let recommendations = analyzer.generate_recommendations(&whys, &root_cause).unwrap();
    
    assert!(!recommendations.is_empty());
    // Should recommend reducing complexity and adding tests
    assert!(recommendations.iter().any(|r| r.contains("complexity")));
    assert!(recommendations.iter().any(|r| r.contains("test")));
}
```

---

## Success Criteria

### Functional Requirements
- ✅ Command executes with issue description
- ✅ Generates 5 "why" iterations by default
- ✅ Gathers evidence from all 5 PMAT services
- ✅ Calculates confidence scores (0.0-1.0)
- ✅ Identifies root cause with supporting evidence
- ✅ Generates actionable recommendations
- ✅ Outputs in 3 formats (text, json, markdown)

### Quality Requirements
- ✅ Test coverage ≥95% (EXTREME TDD)
- ✅ Zero clippy warnings
- ✅ All tests passing
- ✅ TDG score ≥85/100
- ✅ Documentation complete (rustdoc)

### Performance Requirements
- ✅ Analysis completes in <5 seconds for small projects (<100 files)
- ✅ Analysis completes in <30 seconds for large projects (>1000 files)
- ✅ Parallel evidence gathering (tokio::join!)

### UX Requirements
- ✅ Clear, actionable output
- ✅ Confidence scores visible
- ✅ File locations linked
- ✅ Evidence interpretation human-readable
- ✅ Recommendations prioritized (HIGH/MEDIUM/LOW)

### Integration Requirements
- ✅ Works with existing PMAT services
- ✅ Uses O(1) cached metrics where possible
- ✅ No breaking changes to existing commands
- ✅ Documented in CLAUDE.md

---

## Files to Create/Modify

### New Files
1. `server/src/services/five_whys_analyzer.rs` (~500 lines)
2. `server/src/cli/commands/debug.rs` (~100 lines)
3. `server/src/cli/handlers/debug_handlers.rs` (~200 lines)
4. `server/src/models/debug_analysis.rs` (~150 lines)
5. `server/tests/debug_command_tests.rs` (~600 lines)
6. `docs/specifications/pmat-debug-five-whys.md` (this file)

### Modified Files
1. `server/src/services/mod.rs` (+1 line: pub mod)
2. `server/src/cli/commands/mod.rs` (+1 line: pub mod)
3. `server/src/cli/mod.rs` (+handler registration)
4. `server/src/cli/app.rs` (+clap subcommand)
5. `CLAUDE.md` (+usage documentation)

### Total Implementation Size
- ~1,550 lines of production code
- ~600 lines of tests
- **Test:Code ratio**: 0.39 (39% - EXTREME TDD)

---

**NEXT STEP**: Begin RED phase - write all tests first before any implementation.
