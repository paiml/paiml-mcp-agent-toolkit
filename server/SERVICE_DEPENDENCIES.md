# Service Dependencies in stubs.rs

This document maps the service dependencies used in `cli/stubs.rs` to facilitate modular refactoring.

## Overview
- **File**: `server/src/cli/stubs.rs`
- **Size**: 7,522 lines
- **Functions**: 210
- **Max Complexity**: Cyclomatic 26, Cognitive 53

## Core Service Dependencies

### 1. Analysis Services
- **complexity**: Complexity analysis and metrics
  - `analyze_file_complexity()`
  - `ComplexityReport`
  - `ComplexityMetrics`
  
- **dead_code_analyzer**: Dead code detection
  - `DeadCodeAnalyzer`
  - `analyze_with_ranking()`
  - `DeadCodeAnalysisConfig`

- **satd_detector**: Self-admitted technical debt detection
  - `SATDDetector`
  - `TechnicalDebt`
  - `Severity`

### 2. AST Services
- **ast_rust**: Rust AST parsing
  - AST node analysis
  - Syntax tree traversal
  
- **ast_typescript**: TypeScript AST parsing
- **ast_python**: Python AST parsing

### 3. Quality Services
- **defect_probability**: Defect prediction
  - Risk assessment
  - Probability calculation
  
- **tdg_calculator**: Technical Debt Gradient
  - TDG scoring
  - Metrics aggregation

### 4. Infrastructure Services
- **git_analysis**: Git history analysis
  - Commit analysis
  - Churn metrics
  
- **makefile_linter**: Makefile validation
  - Syntax checking
  - Best practices

### 5. Specialized Analyzers
- **lightweight_provability_analyzer**: Proof analysis
  - `ProofSummary`
  - Provability metrics

## Dependency Graph

```
stubs.rs (7,522 lines)
    ├── Analysis Layer
    │   ├── complexity_analyzer
    │   ├── dead_code_analyzer  
    │   └── satd_detector
    ├── AST Layer
    │   ├── ast_rust
    │   ├── ast_typescript
    │   └── ast_python
    ├── Quality Layer
    │   ├── defect_probability
    │   └── tdg_calculator
    └── Infrastructure Layer
        ├── git_analysis
        └── makefile_linter
```

## Refactoring Strategy

### Phase 1: Extract Service Interfaces
1. Define trait-based interfaces for each service
2. Create service registry for dependency injection
3. Implement service facades for simplified access

### Phase 2: Modularize Command Handlers
1. Extract complexity analysis handlers → `handlers/analysis/complexity.rs`
2. Extract SATD handlers → `handlers/analysis/satd.rs`
3. Extract dead code handlers → `handlers/analysis/dead_code.rs`
4. Extract TDG handlers → `handlers/analysis/tdg.rs`

### Phase 3: Create Service Layer
1. Implement service orchestration layer
2. Add caching and performance optimizations
3. Implement proper error handling and recovery

### Phase 4: Testing & Validation
1. Create unit tests for each extracted module
2. Add integration tests for service interactions
3. Performance benchmarking

## Implementation Priority

1. **High Priority** (Core functionality)
   - complexity_analyzer
   - dead_code_analyzer
   - satd_detector

2. **Medium Priority** (Quality features)
   - tdg_calculator
   - defect_probability
   - ast_* parsers

3. **Low Priority** (Supporting features)
   - git_analysis
   - makefile_linter
   - lightweight_provability_analyzer

## Migration Checklist

- [ ] Create service trait definitions
- [ ] Implement service registry
- [ ] Extract complexity handlers
- [ ] Extract SATD handlers
- [ ] Extract dead code handlers
- [ ] Extract TDG handlers
- [ ] Create service facades
- [ ] Add comprehensive tests
- [ ] Update documentation
- [ ] Performance validation

## Notes

- Each service should be independently testable
- Use dependency injection for flexibility
- Maintain backward compatibility during migration
- Follow Toyota Way principles: incremental improvements