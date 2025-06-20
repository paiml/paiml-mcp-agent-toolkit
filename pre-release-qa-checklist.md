# Pre-Release QA Checklist for v0.26.0

This checklist covers manual QA testing of all CLI features using the release binary.
Execute each test and mark with ✅ when verified or ❌ if failed.

## Prerequisites

- [ ] Build release binary: `cargo build --release --features all-languages`
- [ ] Binary location: `./target/release/pmat`
- [ ] Test project: Use this repository for self-testing
- [ ] Create test output directory: `mkdir -p qa-test-outputs`

## 1. Basic Commands

### Version and Help
- [ ] `./target/release/pmat --version` - Shows version 0.26.0
- [ ] `./target/release/pmat --help` - Shows all commands
- [ ] `./target/release/pmat analyze --help` - Shows all analysis subcommands

## 2. Context Generation

### Auto-detection
- [ ] `./target/release/pmat context` - Auto-detects Rust project
- [ ] `./target/release/pmat context --output qa-test-outputs/context.md` - Saves to file

### Format Options
- [ ] `./target/release/pmat context --format json > qa-test-outputs/context.json` - JSON output
- [ ] `./target/release/pmat context --format sarif > qa-test-outputs/context.sarif` - SARIF output
- [ ] `./target/release/pmat context --format llm-optimized` - LLM-optimized format

### Language-specific
- [ ] `./target/release/pmat context --toolchain rust` - Force Rust detection
- [ ] `./target/release/pmat context --toolchain kotlin -p test_kotlin_project` - Test Kotlin (create small test project first)

## 3. Code Analysis - Complexity

### Basic Complexity
- [ ] `./target/release/pmat analyze complexity` - Default summary
- [ ] `./target/release/pmat analyze complexity --format full` - Full report
- [ ] `./target/release/pmat analyze complexity --format json > qa-test-outputs/complexity.json` - JSON output
- [ ] `./target/release/pmat analyze complexity --max-cyclomatic 20` - With threshold
- [ ] `./target/release/pmat analyze complexity --top-files 5` - Top complex files

## 4. Code Analysis - Git-based

### Code Churn
- [ ] `./target/release/pmat analyze churn` - Default 30 days
- [ ] `./target/release/pmat analyze churn --days 7` - Last week
- [ ] `./target/release/pmat analyze churn --format json > qa-test-outputs/churn.json` - JSON output
- [ ] `./target/release/pmat analyze churn --top-files 10` - Top churned files

## 5. Code Analysis - Architecture

### Dependency Graphs (DAG)
- [ ] `./target/release/pmat analyze dag` - Default Mermaid output
- [ ] `./target/release/pmat analyze dag --target-nodes 25` - Limited nodes
- [ ] `./target/release/pmat analyze dag --output qa-test-outputs/dag.mmd` - Save to file
- [ ] `./target/release/pmat analyze dag --format dot` - GraphViz format

### Dead Code Detection
- [ ] `./target/release/pmat analyze dead-code` - Default analysis
- [ ] `./target/release/pmat analyze dead-code --format json > qa-test-outputs/dead-code.json`
- [ ] `./target/release/pmat analyze dead-code --confidence-threshold 80` - High confidence only

## 6. Code Quality Analysis

### SATD (Self-Admitted Technical Debt)
- [ ] `./target/release/pmat analyze satd` - Find all SATD
- [ ] `./target/release/pmat analyze satd --severity high` - High severity only
- [ ] `./target/release/pmat analyze satd --format sarif > qa-test-outputs/satd.sarif`

### Technical Debt Gradient (TDG)
- [ ] `./target/release/pmat analyze tdg` - Calculate TDG scores
- [ ] `./target/release/pmat analyze tdg --threshold 1.5` - Above threshold
- [ ] `./target/release/pmat analyze tdg --format json > qa-test-outputs/tdg.json`

## 7. Advanced Analysis

### Deep Context
- [ ] `./target/release/pmat analyze deep-context` - Comprehensive analysis
- [ ] `./target/release/pmat analyze deep-context --format json > qa-test-outputs/deep-context.json`
- [ ] `./target/release/pmat analyze deep-context --full` - Detailed report

### Makefile Linting
- [ ] `./target/release/pmat analyze makefile` - Lint Makefile
- [ ] `./target/release/pmat analyze makefile --fix` - Auto-fix issues
- [ ] `./target/release/pmat analyze makefile --format json > qa-test-outputs/makefile-lint.json`

### Big-O Analysis
- [ ] `./target/release/pmat analyze big-o` - Algorithmic complexity
- [ ] `./target/release/pmat analyze big-o --confidence-threshold 80` - High confidence
- [ ] `./target/release/pmat analyze big-o --format json > qa-test-outputs/big-o.json`

## 8. ML-based Analysis

### Defect Prediction
- [ ] `./target/release/pmat analyze defect-prediction` - Predict defects
- [ ] `./target/release/pmat analyze defect-prediction --high-risk-only` - High risk files
- [ ] `./target/release/pmat analyze defect-prediction --format sarif > qa-test-outputs/defects.sarif`

### Duplicate Detection
- [ ] `./target/release/pmat analyze duplicates` - Find duplicate code
- [ ] `./target/release/pmat analyze duplicates --min-lines 10` - Minimum size
- [ ] `./target/release/pmat analyze duplicates --format json > qa-test-outputs/duplicates.json`

## 9. Specialized Analysis

### Provability Analysis
- [ ] `./target/release/pmat analyze provability` - Formal verification
- [ ] `./target/release/pmat analyze provability --high-confidence-only` - High confidence
- [ ] `./target/release/pmat analyze provability --format json > qa-test-outputs/provability.json`

### Graph Metrics
- [ ] `./target/release/pmat analyze graph-metrics` - Centrality analysis
- [ ] `./target/release/pmat analyze graph-metrics --metric pagerank` - PageRank only
- [ ] `./target/release/pmat analyze graph-metrics --format json > qa-test-outputs/graph-metrics.json`

### Name Similarity
- [ ] `./target/release/pmat analyze name-similarity "analyze"` - Find similar names
- [ ] `./target/release/pmat analyze name-similarity "test" --threshold 0.8` - High similarity

### Symbol Table
- [ ] `./target/release/pmat analyze symbol-table` - Generate symbol table
- [ ] `./target/release/pmat analyze symbol-table --cross-references` - With x-refs
- [ ] `./target/release/pmat analyze symbol-table --format json > qa-test-outputs/symbols.json`

### Proof Annotations
- [ ] `./target/release/pmat analyze proof-annotations` - Collect annotations
- [ ] `./target/release/pmat analyze proof-annotations --source all` - All sources

### Incremental Coverage
- [ ] `./target/release/pmat analyze incremental-coverage --base-branch master` - Coverage analysis
- [ ] `./target/release/pmat analyze incremental-coverage --base-branch master --format json > qa-test-outputs/coverage.json`

## 10. Project Management

### Template Generation
- [ ] `./target/release/pmat list` - List all templates
- [ ] `./target/release/pmat generate rust-makefile` - Generate single template
- [ ] `./target/release/pmat generate rust-makefile --output qa-test-outputs/Makefile.test`

### Project Scaffolding
- [ ] `./target/release/pmat scaffold rust my-test-project --path qa-test-outputs/` - Scaffold project
- [ ] `./target/release/pmat scaffold rust my-test-project --templates makefile,readme,gitignore --path qa-test-outputs/`

### Template Search
- [ ] `./target/release/pmat search makefile` - Search templates
- [ ] `./target/release/pmat search rust --limit 5` - Limited results

## 11. Quality Gates and Reporting

### Quality Gate
- [ ] `./target/release/pmat quality-gate` - Run quality checks
- [ ] `./target/release/pmat quality-gate --strict` - Strict mode

### Enhanced Reports
- [ ] `./target/release/pmat report` - Generate report
- [ ] `./target/release/pmat report --format html` - HTML report
- [ ] `./target/release/pmat report --output qa-test-outputs/report.html`

## 12. Interactive Features

### Demo Mode
- [ ] `./target/release/pmat demo` - CLI demo
- [ ] `./target/release/pmat demo --web` - Web interface (check it opens browser)
- [ ] `./target/release/pmat demo --format json` - JSON output

### Refactoring
- [ ] `./target/release/pmat refactor interactive` - Interactive mode
- [ ] `./target/release/pmat refactor status` - Check status

## 13. Server Mode

### HTTP API
- [ ] `./target/release/pmat serve` - Start server (test with Ctrl+C)
- [ ] `./target/release/pmat serve --port 8080` - Custom port
- [ ] Test endpoint: `curl http://localhost:8080/health`

## 14. Diagnostics

### Self-diagnostics
- [ ] `./target/release/pmat diagnose` - Run diagnostics
- [ ] `./target/release/pmat diagnose --verbose` - Verbose output

## 15. Comprehensive Analysis

### All-in-one Analysis
- [ ] `./target/release/pmat analyze comprehensive` - Run all analyses
- [ ] `./target/release/pmat analyze comprehensive --output qa-test-outputs/comprehensive.json`

## 16. Edge Cases and Error Handling

### Invalid Inputs
- [ ] `./target/release/pmat context --toolchain invalid` - Should show error
- [ ] `./target/release/pmat analyze complexity --max-cyclomatic -1` - Should show error
- [ ] `./target/release/pmat generate non-existent-template` - Should show helpful error

### Empty/Missing Files
- [ ] `./target/release/pmat analyze makefile --path /tmp/no-makefile` - Should handle gracefully
- [ ] `./target/release/pmat context --path /tmp/empty-dir` - Should handle empty directory

## 17. Performance Tests

### Large Files
- [ ] `./target/release/pmat analyze complexity --path server/src` - Should complete < 10s
- [ ] `./target/release/pmat analyze dag` - Should handle large graphs

### Memory Usage
- [ ] Run `./target/release/pmat analyze deep-context` and monitor memory (should stay < 500MB)

## Summary

Total Tests: ~120
- [ ] All tests passed
- [ ] Performance acceptable
- [ ] Error handling appropriate
- [ ] Output formats correct

## Notes

Record any issues found here:

---

## Sign-off

- [ ] QA completed by: ________________
- [ ] Date: ________________
- [ ] Version tested: v0.26.0
- [ ] Ready for release: Yes / No