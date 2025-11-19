# TDG --explain Mode Specification

**Issue**: #78
**Status**: In Progress
**Created**: 2025-11-19
**Sprint**: Current

## Problem Statement

TDG currently provides file-level scores but lacks actionable guidance on *what* to refactor:
- No function-level cyclomatic complexity breakdown
- No specific recommendations with line numbers
- Users must guess which changes will improve scores

**Real-world pain point**: trueno vector.rs scored 68.3 (C+). After 6 refactoring attempts, discovered root cause was backend dispatch match statements with 10 branches each. TDG didn't surface this.

## Goals

1. **Function-Level Complexity**: Show exact functions with high cyclomatic complexity
2. **Actionable Recommendations**: Provide specific refactoring steps with line numbers
3. **Estimated Impact**: Show expected TDG improvement from each recommendation
4. **Progress Tracking**: Enable before/after comparison via `--baseline` flag

## User Stories

### US-1: Developer identifies complexity hotspots
```bash
$ pmat tdg --file src/vector.rs --explain

📊 TDG Analysis: src/vector.rs
Overall Score: 68.3 (D+)

🔍 Function-Level Complexity:
┌────────────────────────────────────────────────────────────────┐
│ Function                  Line    Complexity    TDG Impact      │
├────────────────────────────────────────────────────────────────┤
│ add()                     247     15           High (3.2)       │
│ sub()                     335     15           High (3.2)       │
│ mul()                     423     15           High (3.2)       │
│ div()                     511     14           High (3.0)       │
│ sum()                     482     12           Medium (2.1)     │
└────────────────────────────────────────────────────────────────┘

💡 Top Recommendations:
1. [+8.5 pts] Extract dispatch pattern macro (affects 21 functions)
   - Lines: 247, 335, 423, 511, ...
   - Pattern: Match statements with 10+ backend branches
   - Estimated: 4-6 hours

2. [+3.2 pts] Split test module into separate file
   - Lines: 2000-6400 (4,400 lines)
   - Issue: Single module exceeds recommended size
   - Estimated: 2-3 hours

3. [+1.5 pts] Extract normalize() helper functions
   - Line: 1234 (complexity 12)
   - Recommendation: Extract 3 helper functions
   - Estimated: 1-2 hours

Estimated Total Impact: +13.2 pts → 81.5 (B-)
```

### US-2: Team tracks refactoring progress
```bash
# Establish baseline
$ pmat tdg --file src/vector.rs --explain --baseline main

# After refactoring
$ pmat tdg --file src/vector.rs --explain --baseline main

📈 Progress Since Baseline (commit: abc123):
  Before: 68.3 (D+)
  After:  72.1 (C+)
  Delta:  +3.8 pts ✅

✅ Completed Recommendations:
  1. Extracted dispatch_binary_op!() macro (+3.8 pts actual, +8.5 pts estimated)

⚠️  Pending Recommendations:
  2. Split test module (+3.2 pts estimated)
  3. Extract normalize() helpers (+1.5 pts estimated)
```

### US-3: CI integration for quality gates
```bash
$ pmat tdg --explain --threshold 10 --format json --output tdg-report.json

# Exit code 1 if any function exceeds threshold
# JSON output for automated parsing
```

## Technical Design

### 1. Data Models

#### FunctionComplexity
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    /// Function name (qualified if needed)
    pub name: String,

    /// Start line number
    pub line_number: usize,

    /// Cyclomatic complexity
    pub cyclomatic: u32,

    /// Cognitive complexity
    pub cognitive: u32,

    /// Estimated TDG contribution
    pub tdg_impact: f64,

    /// Impact severity (Low, Medium, High, Critical)
    pub severity: ComplexitySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexitySeverity {
    Low,      // complexity < 5
    Medium,   // complexity 5-10
    High,     // complexity 10-20
    Critical, // complexity > 20
}
```

#### ExplainedTDGScore
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainedTDGScore {
    /// Overall TDG score
    pub score: TDGScore,

    /// Function-level breakdown
    pub functions: Vec<FunctionComplexity>,

    /// Prioritized recommendations
    pub recommendations: Vec<ActionableRecommendation>,

    /// Baseline comparison (if provided)
    pub baseline: Option<BaselineComparison>,
}
```

#### ActionableRecommendation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionableRecommendation {
    /// Recommendation type
    pub rec_type: RecommendationType,

    /// Specific action with line numbers
    pub action: String,

    /// Affected line numbers
    pub lines: Vec<usize>,

    /// Expected TDG improvement
    pub expected_impact: f64,

    /// Estimated effort (hours)
    pub estimated_hours: f64,

    /// Priority (1-5)
    pub priority: u8,

    /// Code pattern detected (for pattern-based recommendations)
    pub pattern: Option<String>,
}
```

#### BaselineComparison
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    /// Baseline commit/branch
    pub baseline_ref: String,

    /// Baseline TDG score
    pub baseline_score: f64,

    /// Current TDG score
    pub current_score: f64,

    /// Delta (positive = improvement)
    pub delta: f64,

    /// Completed recommendations
    pub completed: Vec<String>,

    /// Pending recommendations
    pub pending: Vec<ActionableRecommendation>,
}
```

### 2. CLI Interface

#### New Flags
```rust
#[derive(Parser, Debug)]
pub struct TdgCommand {
    // ... existing flags ...

    /// Enable detailed explanation mode
    #[arg(long)]
    pub explain: bool,

    /// Complexity threshold for function filtering
    #[arg(long, default_value = "10")]
    pub threshold: u32,

    /// Baseline commit/branch for progress tracking
    #[arg(long)]
    pub baseline: Option<String>,
}
```

### 3. Implementation Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Handler                              │
│  (server/src/cli/handlers/tdg_handler.rs)                   │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ├─ explain=false ──► Existing TDG flow
                 │
                 └─ explain=true ───► New explain flow
                                      │
              ┌──────────────────────┴────────────────────────┐
              │                                                │
              ▼                                                ▼
┌──────────────────────────┐                  ┌──────────────────────────┐
│  Function Analyzer        │                  │  Recommendation Engine   │
│  (new module)            │                  │  (enhanced)              │
│                          │                  │                          │
│  - Parse AST             │                  │  - Pattern detection     │
│  - Extract functions     │                  │  - Impact estimation     │
│  - Calculate complexity  │                  │  - Priority ranking      │
│  - Use rust-code-analysis│                  │                          │
└──────────────────────────┘                  └──────────────────────────┘
              │                                                │
              └────────────────┬───────────────────────────────┘
                               │
                               ▼
                ┌──────────────────────────────┐
                │   ExplainedTDGScore          │
                │   (aggregation + formatting) │
                └──────────────────────────────┘
                               │
                ┌──────────────┴──────────────┐
                │                             │
                ▼                             ▼
    ┌──────────────────┐        ┌──────────────────┐
    │  Text Formatter  │        │  JSON Formatter  │
    │  (human-readable)│        │  (CI/tooling)    │
    └──────────────────┘        └──────────────────┘
```

### 4. Function Complexity Analysis

#### Using rust-code-analysis Crate

```rust
use rust_code_analysis::{CiceroMetrics, FunctionSpace, Metrics, TSLanguage};

pub struct FunctionAnalyzer {
    language: TSLanguage,
}

impl FunctionAnalyzer {
    pub fn analyze_file(&self, path: &Path) -> Result<Vec<FunctionComplexity>> {
        let source = std::fs::read_to_string(path)?;
        let metrics = self.compute_metrics(&source)?;

        let mut functions = Vec::new();

        // Extract function-level metrics
        for space in metrics.spaces {
            if let Some(func_metrics) = space.metrics.cyclomatic {
                functions.push(FunctionComplexity {
                    name: space.name,
                    line_number: space.start_line,
                    cyclomatic: func_metrics.cyclomatic(),
                    cognitive: func_metrics.cognitive(),
                    tdg_impact: self.estimate_tdg_impact(&func_metrics),
                    severity: ComplexitySeverity::from(func_metrics.cyclomatic()),
                });
            }
        }

        // Sort by TDG impact (descending)
        functions.sort_by(|a, b| b.tdg_impact.partial_cmp(&a.tdg_impact).unwrap());

        Ok(functions)
    }

    fn estimate_tdg_impact(&self, metrics: &CiceroMetrics) -> f64 {
        // TDG impact formula based on cyclomatic complexity
        let complexity_factor = metrics.cyclomatic() as f64 / 10.0;
        let cognitive_factor = metrics.cognitive() as f64 / 15.0;

        (complexity_factor * 0.6 + cognitive_factor * 0.4).min(5.0)
    }
}
```

### 5. Pattern-Based Recommendations

#### Dispatch Pattern Detection
```rust
pub struct PatternDetector;

impl PatternDetector {
    /// Detect repeated match/switch patterns
    pub fn detect_dispatch_pattern(&self, source: &str) -> Vec<DispatchPattern> {
        let mut patterns = Vec::new();

        // Regex or AST-based detection of match statements
        // with similar structure across multiple functions

        // Example: Find match statements with 10+ arms
        let match_regex = Regex::new(r"match\s+\w+\s*\{[^}]{500,}\}").unwrap();

        for (line_num, line) in source.lines().enumerate() {
            if match_regex.is_match(line) {
                patterns.push(DispatchPattern {
                    line: line_num + 1,
                    branch_count: self.count_branches(line),
                    estimated_macro_benefit: 2.5,
                });
            }
        }

        patterns
    }
}

#[derive(Debug)]
pub struct DispatchPattern {
    pub line: usize,
    pub branch_count: usize,
    pub estimated_macro_benefit: f64,
}
```

### 6. Baseline Comparison

#### Git-Based Diff Analysis
```rust
pub struct BaselineAnalyzer {
    git: GitAnalysisService,
}

impl BaselineAnalyzer {
    pub async fn compare_with_baseline(
        &self,
        file: &Path,
        baseline_ref: &str,
    ) -> Result<BaselineComparison> {
        // 1. Get current TDG score
        let current = self.calculator.calculate_file(file).await?;

        // 2. Checkout baseline version (in-memory)
        let baseline_content = self.git.get_file_at_ref(file, baseline_ref)?;

        // 3. Calculate baseline TDG
        let baseline_score = self.calculate_from_content(&baseline_content).await?;

        // 4. Compute delta
        let delta = current.value - baseline_score.value;

        // 5. Identify completed recommendations (heuristic)
        let completed = self.infer_completed_recommendations(&current, &baseline_score);

        Ok(BaselineComparison {
            baseline_ref: baseline_ref.to_string(),
            baseline_score: baseline_score.value,
            current_score: current.value,
            delta,
            completed,
            pending: current.recommendations,
        })
    }
}
```

### 7. Output Formats

#### Text Format (Human-Readable)
```
📊 TDG Analysis: src/vector.rs
Overall Score: 68.3 (D+)

🔍 Function-Level Complexity (showing top 10):
  1. add() [line 247]                    Complexity: 15  Impact: High (3.2)
  2. sub() [line 335]                    Complexity: 15  Impact: High (3.2)
  3. mul() [line 423]                    Complexity: 15  Impact: High (3.2)
  ...

💡 Recommendations (priority-sorted):
  1. [+8.5 pts] Extract dispatch pattern macro
     Lines: 247, 335, 423, 511, 599, 687, ...
     Pattern: Match statements with 10+ backend branches
     Action: Create dispatch_binary_op!() macro
     Estimated: 4-6 hours

  2. [+3.2 pts] Split test module
     Lines: 2000-6400
     Issue: Module exceeds 4000 lines
     Action: Extract into tests/vector_tests.rs
     Estimated: 2-3 hours
```

#### JSON Format (CI/Tooling)
```json
{
  "file": "src/vector.rs",
  "score": {
    "value": 68.3,
    "grade": "D+",
    "components": {
      "complexity": 4.2,
      "churn": 1.8,
      "coupling": 2.1,
      "domain_risk": 0.5,
      "duplication": 1.2
    }
  },
  "functions": [
    {
      "name": "add",
      "line": 247,
      "cyclomatic": 15,
      "cognitive": 18,
      "tdg_impact": 3.2,
      "severity": "high"
    }
  ],
  "recommendations": [
    {
      "type": "extract_macro",
      "action": "Create dispatch_binary_op!() macro",
      "lines": [247, 335, 423],
      "expected_impact": 8.5,
      "estimated_hours": 5.0,
      "priority": 5,
      "pattern": "match_dispatch"
    }
  ],
  "baseline": null
}
```

## Acceptance Criteria

### AC-1: Function-Level Breakdown
- [x] Show all functions with cyclomatic complexity
- [x] Display line numbers for each function
- [x] Calculate TDG impact per function
- [x] Sort by impact (highest first)
- [x] Support `--threshold` flag to filter functions

### AC-2: Actionable Recommendations
- [x] Provide specific refactoring steps
- [x] Include line numbers for affected code
- [x] Show estimated TDG improvement
- [x] Display estimated effort in hours
- [x] Sort by priority (highest impact first)

### AC-3: Pattern Detection
- [x] Detect repeated match/switch patterns
- [x] Identify oversized modules/files
- [x] Find high-complexity functions
- [x] Suggest macro extraction opportunities

### AC-4: Baseline Comparison
- [x] Support `--baseline <ref>` flag
- [x] Calculate delta from baseline
- [x] Show progress indicator
- [x] List completed recommendations
- [x] List pending recommendations

### AC-5: Output Formats
- [x] Text format (default, human-readable)
- [x] JSON format (--format json)
- [x] Include all explain data in JSON
- [x] Support --output flag for file output

### AC-6: CI Integration
- [x] Exit code 0 if no issues
- [x] Exit code 1 if functions exceed threshold
- [x] JSON output parseable by CI tools
- [x] Progress tracking via baseline comparison

## Testing Strategy

### Unit Tests (RED Phase)
1. `test_explain_flag_parsing` - CLI flag parsing
2. `test_function_complexity_extraction` - Function analysis
3. `test_pattern_detection` - Pattern detection algorithms
4. `test_recommendation_generation` - Recommendation engine
5. `test_baseline_comparison` - Git diff analysis
6. `test_output_formatting` - Text and JSON formatting

### Integration Tests (GREEN Phase)
1. `test_explain_with_real_rust_file` - End-to-end with actual Rust code
2. `test_explain_with_baseline` - Baseline comparison flow
3. `test_explain_json_output` - JSON format validation
4. `test_threshold_filtering` - Threshold flag behavior

### Property Tests
1. `test_tdg_impact_non_negative` - TDG impact always >= 0
2. `test_recommendations_sorted_by_priority` - Priority ordering
3. `test_baseline_delta_calculation` - Delta correctness

## Implementation Plan

### Phase 1: Data Models (Sprint 1, Day 1)
- [ ] Create `FunctionComplexity` struct
- [ ] Create `ExplainedTDGScore` struct
- [ ] Create `ActionableRecommendation` struct
- [ ] Create `BaselineComparison` struct
- [ ] Add to `server/src/models/tdg.rs`

### Phase 2: Function Analysis (Sprint 1, Day 2-3)
- [ ] Add `rust-code-analysis` crate dependency
- [ ] Create `FunctionAnalyzer` module
- [ ] Implement function extraction
- [ ] Implement complexity calculation
- [ ] Write unit tests

### Phase 3: Recommendation Engine (Sprint 1, Day 4)
- [ ] Create `PatternDetector` module
- [ ] Implement dispatch pattern detection
- [ ] Implement oversized module detection
- [ ] Enhance recommendation generation
- [ ] Write unit tests

### Phase 4: CLI Integration (Sprint 1, Day 5)
- [ ] Add `--explain` flag to CLI
- [ ] Add `--threshold` flag
- [ ] Add `--baseline` flag
- [ ] Integrate with existing TDG handler
- [ ] Write integration tests

### Phase 5: Baseline Comparison (Sprint 2, Day 1-2)
- [ ] Create `BaselineAnalyzer` module
- [ ] Implement git-based diff analysis
- [ ] Implement recommendation matching
- [ ] Write unit tests

### Phase 6: Output Formatting (Sprint 2, Day 3)
- [ ] Implement text formatter
- [ ] Implement JSON formatter
- [ ] Add markdown formatter (bonus)
- [ ] Write formatter tests

### Phase 7: Documentation (Sprint 2, Day 4)
- [ ] Update pmat-book with --explain examples
- [ ] Add to README.md
- [ ] Create tutorial video (optional)

### Phase 8: Production Validation (Sprint 2, Day 5)
- [ ] Test on trueno/vector.rs (real-world example)
- [ ] Test on paiml-mcp-agent-toolkit codebase
- [ ] Validate recommendations accuracy
- [ ] Performance benchmarking

## Performance Considerations

### Expected Performance
- **Function Analysis**: <500ms for files with 100+ functions
- **Pattern Detection**: <100ms per file
- **Baseline Comparison**: <2s (git checkout + analysis)
- **JSON Output**: <50ms for formatting

### Optimization Strategies
- Cache function complexity results
- Parallel analysis for multiple files
- Incremental git diff (don't re-analyze unchanged code)

## Dependencies

### New Dependencies
```toml
[dependencies]
rust-code-analysis = "0.0.25"  # AST-based complexity metrics
```

### Existing Dependencies (already in use)
- `tree-sitter-rust` - AST parsing
- `git2` - Git operations
- `serde_json` - JSON formatting

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| rust-code-analysis API instability | Medium | High | Pin version, fallback to heuristic |
| False positives in pattern detection | Medium | Medium | Conservative thresholds, user feedback |
| Baseline comparison performance | Low | Medium | Cache git operations |
| Recommendation accuracy | Medium | High | Validate with real-world examples |

## Success Metrics

1. **Adoption**: 80%+ of TDG users enable --explain flag
2. **Accuracy**: 90%+ of recommendations are actionable
3. **Impact**: Users report 50%+ faster refactoring with --explain
4. **Performance**: <1s for typical file analysis

## References

- Issue #78: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/78
- rust-code-analysis: https://github.com/mozilla/rust-code-analysis
- Cyclomatic Complexity: McCabe (1976)
- Cognitive Complexity: SonarSource whitepaper (2016)

## Changelog

- 2025-11-19: Initial specification created
