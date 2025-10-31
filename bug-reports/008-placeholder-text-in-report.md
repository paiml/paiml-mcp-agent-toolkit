# Bug Report: Placeholder Text in Context Report Sections

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium
**Component**: Context generation - report sections

## Description

When running `pmat context`, the final report shows multiple sections with placeholder text instead of actual analysis results. These sections appear to be templates that were never filled in with real data.

## Steps to Reproduce

```bash
pmat context
```

## Actual Output

```markdown
## Key Components
Key architectural components identified in the codebase.

## Big-O Complexity Analysis
Complexity analysis results integrated in function annotations above.

## Entropy Analysis
Code entropy and organization metrics.

## Provability Analysis
Formal verification and provability insights.

## Graph Metrics
Dependency graph and PageRank analysis.

## Technical Debt Gradient (TDG)
Technical debt progression and accumulation patterns.

## Dead Code Analysis
Unused code detection and removal recommendations.

## Self-Admitted Technical Debt (SATD)
TODO, FIXME, and HACK comments indicating technical debt.

## Quality Insights
Overall code quality assessment and trends.

## Recommendations
Actionable suggestions for code improvement.
```

## Expected Behavior

Each section should contain actual analysis results:

```markdown
## Key Components
- `ServerCore` (server/src/main.rs:45) - Entry point, handles request routing
- `LanguageAnalyzer` (server/src/cli/language_analyzer.rs:120) - Multi-language AST parsing
- `DeepContextService` (server/src/services/deep_context.rs:200) - Context generation

## Big-O Complexity Analysis
- **O(n²)**: 3 functions detected
  - `process_dependencies` (server/src/graph/builder.rs:156)
  - `analyze_cross_references` (server/src/services/context.rs:289)
- **O(n log n)**: 12 functions detected
- **O(n)**: 145 functions detected

## Entropy Analysis
- Average entropy: 3.45 bits
- High entropy files (>4.0 bits):
  - server/src/cli/mod.rs: 4.32 bits
  - server/src/services/mutation/mod.rs: 4.18 bits

... (actual data for all sections)
```

## Analysis

Possible causes:

1. **Template Not Filled**: Sections exist as placeholders but analysis not integrated
2. **Analysis Not Run**: Individual analyses (entropy, graph, etc.) not executed
3. **Data Not Aggregated**: Analyses run but results not collected for report
4. **Feature Incomplete**: These features planned but not yet implemented

## Impact

- Report provides no value in these sections
- Users expect comprehensive analysis but get empty placeholders
- Wastes report space and reduces credibility
- Makes PMAT appear incomplete or broken

## Files to Investigate

- `server/src/cli/handlers/context.rs` - Context generation orchestration
- `server/src/services/simple_deep_context.rs` or `server/src/services/deep_context.rs` - Report generation
- Individual analysis services:
  - `server/src/services/entropy.rs` (if exists)
  - `server/src/graph/metrics.rs` - Graph analysis
  - `server/src/services/provability.rs` (if exists)
  - `server/src/services/tdg.rs` - Technical debt gradient

## Suggested Fix

**Option 1: Fill in sections with actual data**
- Integrate existing analyses into report sections
- Aggregate metrics and generate summaries

**Option 2: Remove placeholder sections**
- Only show sections with actual data
- Use conditional rendering based on available analyses

**Option 3: Show "Not Available" message**
```markdown
## Entropy Analysis
_Analysis not yet implemented. Coming in future release._
```

Preferred: **Option 1** - Users expect these features based on section headers.

## Test Case

```rust
#[test]
fn test_context_report_no_placeholder_text() {
    let result = generate_context("./fixtures/rust-project");

    // Should not contain generic placeholder descriptions
    assert!(!result.contains("Key architectural components identified"));
    assert!(!result.contains("Complexity analysis results integrated in function annotations"));

    // Should contain actual metrics
    assert!(result.contains("Average entropy:"));
    assert!(result.contains("O(n²): "));
}
```
