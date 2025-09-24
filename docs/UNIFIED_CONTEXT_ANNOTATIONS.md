# Unified Context with Advanced Annotations

The `pmat context` command now provides comprehensive analysis with advanced annotations in a single unified output.

## Overview

The unified context command integrates all PMAT analysis types into one comprehensive report, providing a complete view of your codebase quality, complexity, and technical debt.

## Usage

```bash
# Generate unified context with all annotations
pmat context

# Save to file
pmat context --output context.md

# Skip expensive analysis for faster results
pmat context --skip-expensive-metrics
```

## Advanced Annotations Included

### 1. Big-O Complexity Analysis
Analyzes algorithmic complexity of functions to identify performance bottlenecks:

```markdown
## Big-O Complexity Analysis

- `sort_function`: O(n log n)
- `nested_loops`: O(n²)
- `simple_function`: O(1)
```

### 2. Entropy Analysis
Measures pattern entropy and code duplication for actionable quality improvements:

```markdown
## Entropy Analysis

- **Pattern Entropy**: 0.750
- **Code Duplication**: 16.4%
- **Structural Entropy**: 0.820

### Actionable Improvements:
- Refactor duplicated logic in authentication modules
- Extract common patterns to utility functions
```

### 3. Provability Analysis
Uses abstract interpretation to analyze correctness properties:

```markdown
## Provability Analysis

### Invariants
- Array bounds are always checked before access
- Null pointers are validated before dereferencing

### Pre-conditions
- Input validation completed before processing
- Resources initialized before use

### Post-conditions
- Memory properly deallocated after operations
- Error states properly handled

### Verification Status: ✓ Verified
```

### 4. Graph Metrics
Analyzes dependency and call graph structure with centrality measures:

```markdown
## Graph Metrics

### Centrality Measures
- **Betweenness Centrality**: 0.342
- **Closeness Centrality**: 0.678
- **Degree Centrality**: 0.445

### Graph Structure
- **Nodes**: 127
- **Edges**: 256
- **Density**: 0.032

### Critical Paths
- main -> auth -> database -> query
- handler -> validation -> business_logic
```

### 5. Technical Debt Gradient (TDG)
Quantifies and prioritizes technical debt for refactoring decisions:

```markdown
## Technical Debt Gradient (TDG)

### Overall TDG Score: 7.32

### File-level TDG Scores
- `src/auth.rs`: 8.94
- `src/database.rs`: 6.12
- `src/utils.rs`: 3.45

### Debt Hotspots
- Authentication module (Score: 8.94)
- Legacy parser code (Score: 7.23)

### Refactoring Priorities
1. Refactor authentication module
2. Modernize legacy parser
3. Extract common utilities
```

### 6. Dead Code Analysis
Identifies unreachable code, unused variables, and unnecessary imports:

```markdown
## Dead Code Analysis

### Unreachable Functions
- `legacy_handler`
- `deprecated_utility`

### Unused Variables
- `temp_buffer` in process_data()
- `cache_size` in main()

### Unused Imports
- `std::collections::HashMap` in utils.rs
- `serde_json` in config.rs
```

### 7. Self-Admitted Technical Debt (SATD)
Analyzes TODO, FIXME, and HACK comments to track acknowledged technical debt:

```markdown
## Self-Admitted Technical Debt (SATD)

### Total SATD Comments: 23

### TODO Comments (15)
- src/auth.rs:45: TODO: implement proper error handling
- src/db.rs:123: TODO: add connection pooling
- src/utils.rs:67: TODO: optimize this algorithm

### FIXME Comments (6)
- src/parser.rs:234: FIXME: memory leak in edge case
- src/api.rs:89: FIXME: race condition under load

### HACK Comments (2)
- src/legacy.rs:12: HACK: temporary workaround for API compatibility

### Debt Categories
- **Design Debt**: 8
- **Code Debt**: 12
- **Test Debt**: 2
- **Documentation Debt**: 1
```

### 8. Quality Insights
Provides automated analysis and insights based on all metrics:

```markdown
## Quality Insights

- **Codebase Size**: 57 functions across 13 files
- **Average Functions per File**: 4.4
- ⚠ High complexity functions: 12
- ⚠ High function density - consider modularization
```

### 9. Recommendations
Actionable recommendations based on comprehensive analysis:

```markdown
## Recommendations

- Review and address identified technical debt
- Refactor high-complexity functions (TDG > 8.0)
- Remove dead code to improve maintainability
- Monitor TDG scores over time to track improvement
- Consider breaking down large modules
- Enable all analysis features for comprehensive insights
```

## Implementation Details

### Architecture
The unified context uses the `AdvancedUnifiedContextBuilder` which:

- Integrates with existing analysis engines (`pmat analyze big-o`, `entropy`, etc.)
- Provides stub implementations ready for actual analyzer integration
- Uses extreme TDD with property-based testing
- Supports configurable analysis features

### Language Support
- **Rust**: Full AST analysis with complexity metrics
- **TypeScript/JavaScript**: Enhanced AST parsing with SWC
- **WASM/WAT**: WebAssembly module analysis
- **Other languages**: Heuristic-based analysis with extensible framework

### Performance
- Basic analysis: ~1-2 seconds
- Full analysis with all annotations: ~5-10 seconds
- Use `--skip-expensive-metrics` for faster results (disables provability and graph metrics)

## Configuration

The unified context can be configured through:

```rust
let mut builder = AdvancedUnifiedContextBuilder::new(&project_path);

// Disable expensive analysis
builder.enable_provability = false;
builder.enable_graph_metrics = false;

// Enable specific features
builder.enable_big_o = true;
builder.enable_entropy = true;
builder.enable_tdg = true;
```

## Testing

The implementation includes comprehensive test coverage:

- **TDD Test Suites**: RED-GREEN-REFACTOR for all annotations
- **Property-Based Tests**: Consistency and edge case validation
- **Integration Tests**: End-to-end validation with real codebases
- **Performance Tests**: Scalability verification

## Future Enhancements

1. **Real Analysis Integration**: Replace stubs with actual analysis engines
2. **ML-Enhanced Insights**: Machine learning for smarter recommendations
3. **Historical Tracking**: Track metrics over time for trend analysis
4. **Custom Annotations**: User-defined analysis rules and metrics
5. **IDE Integration**: VS Code extension for real-time insights

## Examples

See the generated context for the comprehensive language test project:

```bash
cd comprehensive_language_test
pmat context --output example_output.md
```

This showcases all annotations working together to provide a complete picture of codebase quality and technical debt.