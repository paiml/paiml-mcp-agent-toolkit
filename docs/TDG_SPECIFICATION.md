# Technical Debt Grading (TDG) Specification v2.0

## Problem Statement
The current TDG implementation has been identified as "vaporware" due to:
- Using simple string matching instead of AST-based analysis
- Heuristic-based complexity calculation that misses actual code structure
- No validation against real metrics
- Inaccurate scoring that doesn't reflect actual code quality

## Specification

### 1. Core Requirements

#### 1.1 AST-Based Analysis (REQUIRED)
- MUST use proper AST parsing for supported languages
- MUST NOT use string matching or heuristics
- MUST accurately count control flow statements
- MUST handle language-specific constructs correctly

#### 1.2 Supported Languages (MVP)
- Rust (via syn crate)
- Python (via rustpython-parser)
- JavaScript/TypeScript (via swc)
- Go (via tree-sitter)
- Java (via tree-sitter)

#### 1.3 Six Orthogonal Metrics

##### 1.3.1 Structural Complexity (25 points)
- Cyclomatic complexity (McCabe)
- Cognitive complexity
- Nesting depth
- Method length
- **Validation**: Must match `pmat analyze complexity` output

##### 1.3.2 Semantic Complexity (20 points)
- Number of parameters
- Return type complexity
- Generic/template usage
- Abstraction levels
- **Validation**: AST-based, not heuristic

##### 1.3.3 Duplication Ratio (20 points)
- Exact duplicates (hash-based)
- Near duplicates (AST similarity)
- Pattern repetition
- **Validation**: Must detect actual clones, not false positives

##### 1.3.4 Coupling Score (15 points)
- Import dependencies
- External calls
- Interface implementations
- Cohesion metrics
- **Validation**: Must trace actual dependencies

##### 1.3.5 Documentation Coverage (10 points)
- Function documentation
- Public API documentation
- Inline comments ratio
- **Validation**: Language-specific doc patterns

##### 1.3.6 Consistency Score (10 points)
- Naming conventions
- Code style adherence
- Indentation consistency
- **Validation**: Configurable rules

### 2. Grading Scale

| Grade | Total Score | Description |
|-------|-------------|-------------|
| A+    | 95-100      | Exceptional quality |
| A     | 90-94       | Excellent quality |
| A-    | 85-89       | Very good quality |
| B+    | 80-84       | Good quality |
| B     | 75-79       | Above average |
| B-    | 70-74       | Satisfactory |
| C+    | 65-69       | Below average |
| C     | 60-64       | Poor quality |
| C-    | 55-59       | Very poor |
| D     | 50-54       | Failing |
| F     | <50         | Critical issues |

### 3. Accuracy Requirements

#### 3.1 Complexity Calculation
- MUST match industry-standard tools within 5% margin
- Cyclomatic complexity MUST be exact (no estimation)
- Cognitive complexity MUST follow Sonar rules

#### 3.2 Test Coverage
- Unit tests for each metric calculation
- Integration tests with known codebases
- Property-based tests for edge cases
- Minimum 90% code coverage

#### 3.3 Performance
- Single file analysis: <100ms
- 1000 file project: <10 seconds
- Memory usage: <500MB for large projects

### 4. Output Requirements

#### 4.1 File-Level Analysis
```json
{
  "file": "path/to/file.rs",
  "language": "Rust",
  "tdg_score": {
    "structural_complexity": 22.5,
    "semantic_complexity": 18.0,
    "duplication_ratio": 19.5,
    "coupling_score": 14.0,
    "doc_coverage": 8.5,
    "consistency_score": 9.5,
    "total": 92.0,
    "grade": "A"
  },
  "details": {
    "cyclomatic_complexity": 15,
    "cognitive_complexity": 12,
    "duplicate_lines": 45,
    "dependencies": 8,
    "documented_functions": "18/20"
  }
}
```

#### 4.2 Project-Level Analysis
- Aggregate scores across all files
- Language distribution
- Hotspot identification
- Trend analysis

### 5. Validation Test Suite

#### 5.1 Known Good Examples
- Linux kernel subset: Expected B+ to A-
- Rust standard library: Expected A to A+
- Popular open source projects with known quality

#### 5.2 Known Bad Examples
- Obfuscated code: Expected F
- Legacy spaghetti code: Expected D to F
- High-complexity samples: Expected C or below

### 6. Migration Path

#### Phase 1: AST Integration (Sprint 25)
- Integrate existing AST parsers
- Replace heuristic methods
- Maintain backward compatibility

#### Phase 2: Validation (Sprint 26)
- Comprehensive test suite
- Benchmark against tools
- Fix accuracy issues

#### Phase 3: Production (Sprint 27)
- Performance optimization
- Documentation
- Release v3.0.0

## Success Criteria

1. **Accuracy**: Complexity metrics match established tools (±5%)
2. **Coverage**: 90%+ test coverage with property tests
3. **Performance**: Meet timing requirements
4. **Validation**: Pass all known good/bad examples
5. **User Trust**: No "vaporware" perception

## Implementation Priority

1. Fix cyclomatic complexity (currently using string matching)
2. Implement proper AST parsing for all metrics
3. Add comprehensive test suite
4. Validate against known codebases
5. Performance optimization