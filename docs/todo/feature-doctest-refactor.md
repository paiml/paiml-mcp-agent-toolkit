# Heuristic-Driven Doctest Coverage Improvement Specification

- **Status**: Proposed
- **Author**: PAIML Engineering
- **Created**: 2023-10-27
- **Last Updated**: 2025-01-04
- **Target Coverage**: 80% doctest coverage for all public APIs
- **Estimated Effort**: 120-160 developer hours

## 1. Executive Summary

This specification defines a systematic, heuristic-driven approach to achieving 80% doctest coverage across the `pmat` codebase while maintaining our Zero Tolerance Quality Standards. The approach leverages pmat's own analysis capabilities to identify high-value targets and verify improvements, implementing a feedback loop that ensures each contribution adds measurable value.

### Key Metrics
- **Current State**: ~45% doctest coverage (estimated from `cargo tarpaulin` runs)
- **Target State**: ≥80% sustained doctest coverage
- **Quality Gate**: Zero regression in cyclomatic complexity (<20 per function)
- **Performance Impact**: <1% increase in `cargo test --doc` execution time

## 2. Technical Architecture

### 2.1 Coverage Analysis Pipeline

The doctest improvement process integrates with pmat's existing analysis infrastructure:

```rust
// Integration with pmat's CodeAnalysisService
pub struct DoctestCoverageAnalyzer {
    ast_parser: AstParser,
    coverage_tracker: CoverageTracker,
    complexity_analyzer: ComplexityAnalyzer,
}

impl DoctestCoverageAnalyzer {
    pub async fn analyze_missing_doctests(&self, path: ProjectPath) -> Result<DoctestReport> {
        let ast = self.ast_parser.parse_project(&path).await?;
        let public_apis = self.extract_public_apis(&ast);
        let coverage_data = self.coverage_tracker.get_doctest_coverage(&public_apis).await?;
        
        // Correlate with complexity to prioritize high-value targets
        let complexity_scores = self.complexity_analyzer.analyze(&ast).await?;
        
        Ok(self.generate_prioritized_report(coverage_data, complexity_scores))
    }
}
```

### 2.2 Integration Points

1. **AST Analysis**: Leverages existing `syn` and `tree-sitter` parsers
2. **Coverage Tracking**: Integrates with `cargo-tarpaulin` for baseline metrics
3. **Quality Gates**: Hooks into `pmat quality-gate` for automated verification
4. **CI/CD Pipeline**: GitHub Actions workflow for continuous monitoring

## 3. Heuristic Framework

### 3.1 Priority Matrix

Each potential doctest target is scored using a weighted priority matrix:

```rust
#[derive(Debug, Clone)]
pub struct DoctestPriority {
    pub api_visibility_score: f64,    // Weight: 0.35
    pub complexity_score: f64,        // Weight: 0.25
    pub usage_frequency: f64,         // Weight: 0.20
    pub testability_score: f64,       // Weight: 0.15
    pub documentation_gap: f64,       // Weight: 0.05
}

impl DoctestPriority {
    pub fn calculate_priority(&self) -> f64 {
        self.api_visibility_score * 0.35 +
        self.complexity_score * 0.25 +
        self.usage_frequency * 0.20 +
        self.testability_score * 0.15 +
        self.documentation_gap * 0.05
    }
}
```

### 3.2 Heuristic Definitions

#### Heuristic 1: Critical Path APIs (Priority: P0)
**Rationale**: APIs on the critical execution path have the highest impact on system reliability.

**Implementation**:
```rust
/// Service initialization is critical path - must never fail
///
/// # Examples
///
/// ```
/// use pmat::services::code_analysis::CodeAnalysisService;
/// use pmat::types::ProjectPath;
///
/// let service = CodeAnalysisService::new();
/// assert!(service.is_initialized());
/// 
/// // Verify default configuration
/// let config = service.get_config();
/// assert_eq!(config.max_file_size, 10_485_760); // 10MB
/// assert_eq!(config.timeout_ms, 30_000);
/// ```
pub fn new() -> Self {
    Self {
        parser: AstParser::with_default_config(),
        cache: AnalysisCache::new(1000),
        metrics: Metrics::default(),
    }
}
```

**Identification Query**:
```sql
SELECT function_name, module_path, cyclomatic_complexity
FROM public_apis
WHERE is_critical_path = true
  AND doctest_coverage = 0
ORDER BY usage_frequency DESC
LIMIT 10;
```

#### Heuristic 2: Pure Functions with Invariants (Priority: P1)
**Rationale**: Pure functions with mathematical properties provide high-confidence test cases.

**Example with Property Testing**:
```rust
/// Converts identifier to snake_case while preserving semantics
///
/// # Invariants
/// - Idempotent: `to_snake_case(to_snake_case(s)) == to_snake_case(s)`
/// - Reversible modulo case: original semantic meaning preserved
///
/// # Examples
///
/// ```
/// use pmat::utils::naming::to_snake_case;
///
/// // Basic conversions
/// assert_eq!(to_snake_case("PascalCase"), "pascal_case");
/// assert_eq!(to_snake_case("camelCase"), "camel_case");
/// assert_eq!(to_snake_case("kebab-case"), "kebab_case");
/// assert_eq!(to_snake_case("SCREAMING_SNAKE"), "screaming_snake");
///
/// // Edge cases
/// assert_eq!(to_snake_case("IOError"), "io_error");
/// assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
/// assert_eq!(to_snake_case("getValue2"), "get_value_2");
///
/// // Idempotency
/// let input = "ComplexHTTPSHandler";
/// assert_eq!(
///     to_snake_case(&to_snake_case(input)),
///     to_snake_case(input)
/// );
/// ```
pub fn to_snake_case(s: &str) -> String {
    // Implementation with O(n) complexity
}
```

#### Heuristic 3: Error Path Documentation (Priority: P1)
**Rationale**: Error handling paths are often undertested but critical for robustness.

```rust
/// Analyzes file with comprehensive error handling
///
/// # Examples
///
/// ```
/// use pmat::services::code_analysis::{CodeAnalysisService, AnalysisError};
/// use pmat::types::ProjectPath;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let service = CodeAnalysisService::new();
/// let path = ProjectPath::new("src/main.rs");
///
/// // Success case
/// let result = service.analyze_file(&path).await?;
/// assert!(result.metrics.lines_of_code > 0);
///
/// // File not found
/// let missing = ProjectPath::new("nonexistent.rs");
/// match service.analyze_file(&missing).await {
///     Err(AnalysisError::FileNotFound(p)) => {
///         assert_eq!(p.as_ref(), "nonexistent.rs");
///     }
///     _ => panic!("Expected FileNotFound error"),
/// }
///
/// // File too large (>10MB)
/// let large_file = ProjectPath::new("assets/large.bin");
/// match service.analyze_file(&large_file).await {
///     Err(AnalysisError::FileTooLarge { size, max }) => {
///         assert!(size > max);
///         assert_eq!(max, 10_485_760);
///     }
///     _ => panic!("Expected FileTooLarge error"),
/// }
/// # Ok(())
/// # }
/// ```
pub async fn analyze_file(&self, path: &ProjectPath) -> Result<Analysis, AnalysisError> {
    // Implementation with proper error boundaries
}
```

#### Heuristic 4: Configuration and Builder Patterns (Priority: P2)
**Rationale**: Configuration structs define system behavior contracts.

```rust
/// Configuration for deep context analysis with semantic validation
///
/// # Examples
///
/// ```
/// use pmat::services::deep_context::{DeepContextConfig, ValidationError};
///
/// // Default configuration
/// let config = DeepContextConfig::default();
/// assert_eq!(config.max_depth, 10);
/// assert_eq!(config.dead_code_threshold, 0.9);
/// assert!(config.include_test_files);
///
/// // Builder pattern with validation
/// let custom = DeepContextConfig::builder()
///     .max_depth(15)
///     .dead_code_threshold(0.95)
///     .exclude_test_files()
///     .timeout_seconds(60)
///     .build()?;
/// 
/// assert_eq!(custom.max_depth, 15);
/// assert!(!custom.include_test_files);
///
/// // Validation errors
/// assert!(matches!(
///     DeepContextConfig::builder()
///         .dead_code_threshold(1.5) // Invalid: >1.0
///         .build(),
///     Err(ValidationError::InvalidThreshold(_))
/// ));
/// # Ok::<(), ValidationError>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepContextConfig {
    pub max_depth: usize,
    pub dead_code_threshold: f64,
    pub include_test_files: bool,
    // ...
}
```

#### Heuristic 5: Performance-Critical Hot Paths (Priority: P2)
**Rationale**: Document performance characteristics and complexity guarantees.

```rust
/// Binary search for symbol resolution with O(log n) complexity
///
/// # Performance
/// - Time Complexity: O(log n)
/// - Space Complexity: O(1)
/// - Cache-friendly for arrays up to 10K elements
///
/// # Examples
///
/// ```
/// use pmat::analysis::symbol_table::SymbolIndex;
/// use std::time::Instant;
///
/// let mut index = SymbolIndex::new();
/// 
/// // Populate with 10K symbols
/// for i in 0..10_000 {
///     index.insert(format!("symbol_{}", i), i);
/// }
/// index.finalize(); // Sorts internally
///
/// // Lookup performance
/// let start = Instant::now();
/// let result = index.binary_search("symbol_5000");
/// let elapsed = start.elapsed();
/// 
/// assert_eq!(result, Some(&5000));
/// assert!(elapsed.as_micros() < 10); // <10μs lookup
///
/// // Edge cases
/// assert_eq!(index.binary_search("symbol_0"), Some(&0));
/// assert_eq!(index.binary_search("symbol_9999"), Some(&9999));
/// assert_eq!(index.binary_search("nonexistent"), None);
/// ```
pub fn binary_search(&self, key: &str) -> Option<&SymbolData> {
    // Implementation with guaranteed O(log n)
}
```

## 4. Implementation Workflow

### 4.1 Automated Discovery Pipeline

```bash
#!/bin/bash
# scripts/find-doctest-targets.sh

set -euo pipefail

# Use pmat's own analysis to find targets
pmat analyze deep-context \
    --format json \
    --include ast,complexity,symbols | \
jq -r '
    .symbols[] |
    select(.visibility == "public" and .has_doctest == false) |
    "\(.complexity_score)\t\(.file_path)\t\(.line_number)\t\(.name)"
' | sort -rn | head -20
```

### 4.2 Verification Protocol

Each doctest addition must pass through three quality gates:

1. **Syntax Verification**:
   ```bash
   cargo test --doc --package pmat -- path::to::module
   ```

2. **Complexity Check**:
   ```bash
   pmat quality-gate --file path/to/file.rs --max-complexity 20
   ```

3. **Performance Regression**:
   ```bash
   hyperfine --warmup 3 \
       'cargo test --doc --package pmat' \
       'git checkout HEAD~1 && cargo test --doc --package pmat'
   ```

### 4.3 CI/CD Integration

```yaml
# .github/workflows/doctest-coverage.yml
name: Doctest Coverage Monitor

on:
  pull_request:
    paths:
      - 'server/src/**/*.rs'

jobs:
  coverage-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Calculate Doctest Coverage
        run: |
          cargo tarpaulin --doc --out Json > coverage.json
          
      - name: Verify Coverage Improvement
        run: |
          NEW_COVERAGE=$(jq '.coverage' coverage.json)
          BASE_COVERAGE=$(curl -s $BASE_COVERAGE_URL | jq '.coverage')
          
          if (( $(echo "$NEW_COVERAGE <= $BASE_COVERAGE" | bc -l) )); then
            echo "Coverage regression detected: $NEW_COVERAGE <= $BASE_COVERAGE"
            exit 1
          fi
```

## 5. Quality Metrics and Success Criteria

### 5.1 Primary Metrics
- **Doctest Coverage**: ≥80% for public APIs
- **Execution Time**: `cargo test --doc` <30 seconds
- **Doctest Quality Score**: ≥0.8 (based on assertion density and edge case coverage)

### 5.2 Secondary Metrics
- **Mean Time to Document (MTTD)**: <15 minutes per function
- **Doctest Failure Rate**: <2% in CI/CD over 30 days
- **Developer Satisfaction**: >4.0/5.0 in quarterly surveys

### 5.3 Anti-Patterns to Avoid

```rust
// ❌ BAD: Trivial doctest that adds no value
/// ```
/// let x = 5;
/// assert_eq!(x, 5);
/// ```

// ❌ BAD: Doctest that doesn't test the actual function
/// ```
/// // This doesn't even call the function!
/// assert!(true);
/// ```

// ✅ GOOD: Comprehensive doctest with edge cases
/// ```
/// use pmat::utils::validate_identifier;
/// 
/// assert!(validate_identifier("valid_name"));
/// assert!(validate_identifier("_private"));
/// assert!(!validate_identifier("123invalid"));
/// assert!(!validate_identifier(""));
/// assert!(!validate_identifier("kebab-case"));
/// ```
```

## 6. Rollout Schedule

### Phase 1: Foundation (Weeks 1-2)
- Implement `DoctestCoverageAnalyzer`
- Create automated discovery scripts
- Establish baseline metrics

### Phase 2: Critical APIs (Weeks 3-6)
- Target P0 heuristics (critical path APIs)
- Achieve 60% coverage milestone
- Refine priority matrix based on results

### Phase 3: Comprehensive Coverage (Weeks 7-10)
- Target P1 and P2 heuristics
- Achieve 80% coverage milestone
- Implement continuous monitoring

### Phase 4: Maintenance Mode (Ongoing)
- Automated PR checks for new public APIs
- Quarterly coverage audits
- Community contribution guidelines

## 7. Risk Mitigation

### 7.1 Technical Risks
- **Risk**: Doctest execution time regression
- **Mitigation**: Parallel test execution, caching compiled examples

### 7.2 Process Risks
- **Risk**: Developer resistance to documentation burden
- **Mitigation**: AI-assisted doctest generation, clear value demonstration

### 7.3 Quality Risks
- **Risk**: Low-quality doctests that don't catch regressions
- **Mitigation**: Automated quality scoring, peer review requirements

## 8. Appendices

### A. Doctest Quality Scoring Algorithm

```rust
pub fn score_doctest_quality(doctest: &str) -> f64 {
    let mut score = 0.0;
    
    // Assertion density (assertions per line)
    let assertion_count = doctest.matches("assert").count();
    let line_count = doctest.lines().count();
    score += (assertion_count as f64 / line_count as f64).min(0.3) * 0.3;
    
    // Edge case coverage
    if doctest.contains("// Edge case") || doctest.contains("empty") {
        score += 0.2;
    }
    
    // Error case coverage
    if doctest.contains("Err(") || doctest.contains("panic!") {
        score += 0.2;
    }
    
    // Performance validation
    if doctest.contains("Instant::now()") || doctest.contains("elapsed") {
        score += 0.15;
    }
    
    // Property testing
    if doctest.contains("quickcheck") || doctest.contains("proptest") {
        score += 0.15;
    }
    
    score
}
```

### B. Integration with `pmat refactor auto`

The doctest improvement process integrates with the AI-powered refactoring engine:

```bash
# Generate AI-suggested doctests for a module
pmat refactor auto \
    --single-file-mode \
    --file src/services/code_analysis.rs \
    --refactor-type add-doctests \
    --quality-threshold 0.8
```

This leverages the LLM to generate high-quality doctests that follow the established patterns while ensuring they compile and provide meaningful coverage.
