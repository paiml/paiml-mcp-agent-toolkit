# TDG Systems Comparison

## Overview

The codebase contains **two distinct TDG (Technical Debt Gradient) scoring systems** that serve different purposes and use different scoring scales.

## System 1: Grade-Based TDG (0-100 Scale)

**Location**: `server/src/tdg/mod.rs`, `server/src/tdg/analyzer_ast.rs`

**Scoring Scale**: 0-100 (higher is better)

**Output**: Letter grades (A+, A, A-, B+, B, B-, C+, C, C-, D, F)

### Components (Total: 110 points max, normalized to 100)
- **Structural Complexity**: 0-25 points
- **Semantic Complexity**: 0-20 points
- **Duplication**: 0-20 points
- **Coupling**: 0-15 points
- **Documentation**: 0-10 points
- **Consistency**: 0-10 points
- **Entropy**: 0-10 points (added for pattern analysis)

### Normalization Strategy
```rust
// Components are clamped to their max weights
structural_complexity = value.clamp(0.0, 25.0)
// ... other components ...

// Sum components and normalize to 0-100
if raw_total <= 100.0 {
    total = raw_total.clamp(0.0, 100.0)
} else {
    // Scale down when entropy pushes total > 100
    total = (raw_total / 110.0 * 100.0).clamp(0.0, 100.0)
}
```

### Grade Boundaries
- **A+**: >= 95.0
- **A**: >= 90.0
- **A-**: >= 85.0
- **B+**: >= 80.0
- **B**: >= 75.0
- **B-**: >= 70.0
- **C+**: >= 65.0
- **C**: >= 60.0
- **C-**: >= 55.0
- **D**: >= 50.0
- **F**: < 50.0

### Use Cases
- **AST-based analysis**: Full syntax tree parsing for accurate metrics
- **Detailed quality reporting**: Comprehensive breakdown of code quality factors
- **IDE integration**: Real-time quality feedback with letter grades
- **Quality gates**: Enforce minimum grade requirements (e.g., "no files below C+")

### Key Files
- `server/src/tdg/mod.rs`: Core TDG score structures
- `server/src/tdg/analyzer_ast.rs`: AST-based analysis implementation
- `server/src/tdg/scorers/`: Individual metric scorers (complexity, coupling, etc.)
- `server/src/tdg/normalization_tests.rs`: Normalization validation tests
- `server/src/tdg/complexity_entropy_integration_tests.rs`: Integration tests

## System 2: Severity-Based TDG (0-5 Scale)

**Location**: `server/src/services/tdg_calculator.rs`, `server/src/models/tdg.rs`

**Scoring Scale**: 0-5 (higher is worse - represents technical debt level)

**Output**: Severity levels (Normal, Warning, Critical)

### Components (Weighted)
- **Complexity**: Weight 0.30 (30%)
- **Churn**: Weight 0.35 (35%)
- **Coupling**: Weight 0.15 (15%)
- **Domain Risk**: Weight 0.10 (10%)
- **Duplication**: Weight 0.10 (10%)

### Normalization Strategy
```rust
// Each component is normalized to 0-5 range
complexity_factor = score.min(5.0)
churn_factor = normalized.min(5.0)
// ... other components ...

// Weighted sum with provability adjustment
base_weighted = complexity * 0.30 + churn * 0.35 + coupling * 0.15
              + domain_risk * 0.10 + duplication * 0.10
adjusted = base_weighted * (1.0 - provability_factor * 0.2)

// Final clamp to 0-5
tdg_value = adjusted.clamp(0.0, 5.0)
```

### Severity Thresholds
- **Normal**: TDG < 1.5 (low technical debt)
- **Warning**: TDG 1.5-2.5 (elevated debt requiring attention)
- **Critical**: TDG > 2.5 (critical debt requiring immediate action)

### Use Cases
- **Project-wide analysis**: Fast heuristic-based scoring for large codebases
- **Hotspot detection**: Identify high-risk files needing refactoring
- **CI/CD integration**: Quick quality checks without AST parsing overhead
- **Trend analysis**: Track technical debt trends over time
- **Refactoring prioritization**: Rank files by TDG score for remediation planning

### Key Files
- `server/src/services/tdg_calculator.rs`: Main TDG calculation logic
- `server/src/models/tdg.rs`: TDG data structures and severity levels

## Why Two Systems?

### Historical Context
1. **System 1 (Grade-Based)** was developed first as part of the AST-based quality analysis framework
2. **System 2 (Severity-Based)** was added later to provide lightweight analysis for CI/CD pipelines

### Complementary Strengths

| Aspect | Grade-Based (0-100) | Severity-Based (0-5) |
|--------|---------------------|----------------------|
| **Speed** | Slower (AST parsing) | Faster (heuristics) |
| **Accuracy** | High (full AST) | Moderate (patterns) |
| **Detail** | 7 components | 5 components |
| **Output** | Letter grade | Severity level |
| **Best For** | Local development, IDE | CI/CD, large-scale analysis |
| **Dependencies** | tree-sitter, syn | Minimal (git, fs) |
| **Use Case** | Deep analysis | Quick scans |

### Integration Strategy

**Recommended Usage**:
1. **Development Time**: Use Grade-Based TDG (0-100) for detailed feedback
2. **Pre-Commit**: Use Severity-Based TDG (0-5) for fast quality gates
3. **CI Pipeline**: Use Severity-Based TDG (0-5) for build blocking
4. **Code Review**: Use Grade-Based TDG (0-100) for comprehensive reports
5. **Refactoring**: Use Severity-Based TDG (0-5) for hotspot prioritization

## Future Considerations

### Potential Unification
While having two systems provides flexibility, there are opportunities to unify:

1. **Shared Metrics**: Extract common metric calculations into reusable modules
2. **Configurable Output**: Single calculation engine with multiple output formats
3. **Performance Tiers**: Offer "fast" and "thorough" modes of same system
4. **Unified Storage**: Store both scores in same data structure for correlation analysis

### Migration Path
If unification is desired:
1. Keep Grade-Based (0-100) as primary system
2. Add severity thresholds to Grade-Based system
3. Deprecate Severity-Based system gradually
4. Provide compatibility layer for existing integrations

## Testing

Both systems have comprehensive test coverage:

**Grade-Based System**:
- `server/src/tdg/normalization_tests.rs`: 8 tests validating 0-100 normalization
- `server/src/tdg/complexity_entropy_integration_tests.rs`: 7 integration tests
- Property-based tests with proptest

**Severity-Based System**:
- Unit tests in `server/src/services/tdg_calculator.rs`
- Integration tests validate 0-5 clamping
- Component-level normalization tests

## Conclusion

Having two TDG systems is intentional and provides value:
- **Grade-Based**: Deep, accurate analysis for development
- **Severity-Based**: Fast, actionable metrics for automation

Both systems are properly normalized and tested. Choose the appropriate system based on your use case and performance requirements.