# Actionable Entropy Analysis Specification

## Problem Statement

Current entropy implementation produces 2255+ violations by checking character-level Shannon entropy on EVERY file, making it noisy and non-actionable. We need entropy analysis that identifies REAL code quality issues.

## Solution: AST-Based Pattern Entropy

Since PMAT has full project context via `pmat context` with complete AST analysis, we should measure entropy at the AST pattern level, not character level.

## Core Principle: Entropy Should Find Actionable Issues

Entropy analysis should identify:
1. **Repetitive Patterns** - Same code structure repeated with minor variations
2. **Inconsistent Patterns** - Similar operations done in wildly different ways
3. **Copy-Paste Evolution** - Code that started as copy-paste and diverged
4. **Missing Abstractions** - Patterns that should be extracted into functions/modules

## Implementation Design

### 1. AST Pattern Extraction
```rust
pub struct AstPattern {
    pub pattern_type: PatternType,
    pub frequency: usize,
    pub locations: Vec<Location>,
    pub variation_score: f64,  // How much patterns vary (0=identical, 1=very different)
}

pub enum PatternType {
    ErrorHandling,      // try/catch, Result handling patterns
    DataValidation,     // Input validation patterns
    ResourceManagement, // open/close, lock/unlock patterns
    ControlFlow,        // if/else chains, match statements
    DataTransformation, // map/filter/reduce patterns
    ApiCall,           // HTTP/RPC call patterns
}
```

### 2. Entropy Calculation Levels

#### Level 1: File-Level Pattern Entropy
- Count unique AST patterns in a file
- Flag files with >5 instances of similar patterns (suggests missing abstraction)
- Flag files with <0.3 pattern diversity (too repetitive)

#### Level 2: Module-Level Pattern Entropy  
- Compare patterns across files in same module
- Flag modules with >70% pattern overlap between files
- Identify cross-file duplication opportunities

#### Level 3: Project-Level Pattern Entropy
- Identify global patterns repeated across modules
- Flag inconsistent error handling across project
- Find project-wide abstraction opportunities

### 3. Actionable Thresholds

```rust
pub struct EntropyThresholds {
    // VIOLATIONS occur when:
    pub max_pattern_repetition: usize,     // Same pattern >5 times in file
    pub min_pattern_diversity: f64,        // File diversity <0.3
    pub max_cross_file_similarity: f64,    // Files >70% similar
    pub max_inconsistency_score: f64,      // Pattern variations >0.8
}

impl Default for EntropyThresholds {
    fn default() -> Self {
        Self {
            max_pattern_repetition: 5,
            min_pattern_diversity: 0.3,
            max_cross_file_similarity: 0.7,
            max_inconsistency_score: 0.8,
        }
    }
}
```

### 4. Output Format

```rust
pub struct EntropyReport {
    pub total_files_analyzed: usize,
    pub actionable_violations: Vec<ActionableViolation>,
    pub refactoring_opportunities: Vec<RefactoringHint>,
    pub pattern_summary: PatternSummary,
}

pub struct ActionableViolation {
    pub severity: Severity,
    pub pattern: AstPattern,
    pub message: String,
    pub fix_suggestion: String,
    pub estimated_loc_reduction: usize,
}

pub enum Severity {
    High,   // >10 repetitions or >80% similarity
    Medium, // 5-10 repetitions or 70-80% similarity  
    Low,    // 3-5 repetitions or 60-70% similarity
}
```

### 5. Example Violations and Fixes

#### Example 1: Repetitive Error Handling
```rust
// VIOLATION: Same error handling pattern repeated 8 times
// SUGGESTION: Extract to handle_api_error() function
// ESTIMATED REDUCTION: 56 lines

// Found in api_client.rs:
if let Err(e) = result {
    log::error!("API call failed: {}", e);
    metrics.increment_error_count();
    return Err(format!("Failed: {}", e));
}
// ... repeated 8 times with minor variations
```

#### Example 2: Inconsistent Validation
```rust
// VIOLATION: 3 different validation patterns for same data type
// SUGGESTION: Standardize with validate_user_input() 
// ESTIMATED REDUCTION: 120 lines

// Pattern A (5 instances):
if input.len() > 0 && input.len() < 100 { ... }

// Pattern B (3 instances):  
match input.len() {
    0 => Err("empty"),
    1..=100 => Ok(input),
    _ => Err("too long")
}

// Pattern C (4 instances):
input.chars().count().checked_sub(1)
    .filter(|&len| len < 100)
    .ok_or("invalid length")
```

### 6. Integration with Quality Gates

```rust
// In pmat.toml
[entropy]
enabled = true
max_violations = 10           # Fail if >10 actionable violations
min_severity = "medium"       # Only report medium+ severity
pattern_types = ["all"]       # Or specific: ["ErrorHandling", "DataValidation"]
exclude_paths = ["tests/*"]  # Don't check test files
```

### 7. CLI Interface

```bash
# Analyze entropy with actionable output
pmat analyze entropy --actionable

# Output:
Entropy Analysis Results
========================
Files Analyzed: 234
Actionable Violations: 8

HIGH SEVERITY (2):
1. api_client.rs: Error handling repeated 12x
   Fix: Extract handle_api_error() - saves 84 lines
   
2. validators.rs: Same validation in 8 functions  
   Fix: Create ValidationRules trait - saves 156 lines

MEDIUM SEVERITY (6):
...

Total Potential Reduction: 423 lines (18% of analyzed code)
```

### 8. MCP Tool Interface

```typescript
{
  "tool": "analyze_entropy",
  "params": {
    "mode": "actionable",
    "severity": "medium",
    "pattern_types": ["ErrorHandling", "DataValidation"]
  }
}
```

## Implementation Plan (Sprint 83)

### Phase 1: AST Pattern Extraction (Day 1-2)
- [ ] Implement PatternExtractor using existing AST from `pmat context`
- [ ] Create pattern fingerprinting algorithm
- [ ] Build pattern similarity scoring

### Phase 2: Entropy Calculation (Day 3-4)
- [ ] Implement file-level entropy calculation
- [ ] Implement module-level entropy calculation  
- [ ] Implement project-level entropy calculation

### Phase 3: Actionable Reporting (Day 5-6)
- [ ] Create violation detection with severity levels
- [ ] Generate fix suggestions based on patterns
- [ ] Calculate estimated LOC reduction

### Phase 4: Integration (Day 7-8)
- [ ] Update quality gate to use new entropy
- [ ] Add CLI command with --actionable flag
- [ ] Update MCP tool interface
- [ ] Add configuration to pmat.toml

### Phase 5: Testing & Documentation (Day 9-10)
- [ ] Unit tests for pattern extraction
- [ ] Integration tests with real codebases
- [ ] Update CLAUDE.md to mandate entropy usage
- [ ] Create examples and documentation

## Success Criteria

1. **Actionability**: Every violation has a clear fix
2. **Accuracy**: <5% false positive rate
3. **Performance**: <10 seconds for 100K LOC
4. **Reduction**: Average 15-30% LOC reduction potential identified
5. **Severity**: Proper prioritization of issues
6. **Integration**: Works with existing quality gates

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_pattern_extraction() {
    let ast = parse_file("sample.rs");
    let patterns = extract_patterns(&ast);
    assert!(patterns.iter().any(|p| matches!(p.pattern_type, PatternType::ErrorHandling)));
}

#[test]
fn test_actionable_violation_generation() {
    let pattern = create_repetitive_pattern();
    let violation = generate_violation(&pattern);
    assert!(violation.fix_suggestion.contains("Extract"));
    assert!(violation.estimated_loc_reduction > 0);
}
```

### Integration Tests
- Test with PMAT's own codebase (dogfooding)
- Test with known repetitive codebases
- Test with well-refactored codebases (should have few violations)

## Configuration

```toml
# server/pmat.toml
[entropy]
enabled = true
mode = "actionable"              # Only actionable violations
max_violations = 10              # Quality gate threshold
min_severity = "medium"          # Ignore low severity
pattern_repetition_threshold = 5 # Flag >5 repetitions
diversity_threshold = 0.3        # Flag <30% diversity
similarity_threshold = 0.7       # Flag >70% similarity
exclude = ["tests/**", "examples/**"]
```

## Expected Outcome

Instead of 2255 noisy violations, we expect:
- 10-50 HIGH/MEDIUM severity actionable violations
- Each with clear fix suggestions
- Total potential for 15-30% code reduction
- Clear prioritization for refactoring work

## References

- PMAT Context System (full AST available)
- Clone Detection Research (Roy & Cordy, 2007)
- DRY Principle (Hunt & Thomas, 1999)
- Refactoring Patterns (Fowler, 2018)