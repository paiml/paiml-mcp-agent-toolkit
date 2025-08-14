# Documentation Coverage Improvement Plan

This document outlines a comprehensive plan to increase documentation coverage from the current 52% to 80%+.

## Current Status
- Total Rust files: 419 (excluding target)
- Files with documentation: 218 (52.02%)
- Files with doctests: 65 (15.51%)
- Total doctests: 508

## Target Goals
- Documentation coverage: 80%+ (335+ files)
- Doctest coverage: 30%+ (125+ files)
- All public APIs must have examples

## Priority 1: Core Module Documentation (CRITICAL)

### Module-Level Documentation (//!)
- [ ] `src/unified_protocol/mod.rs` - Document the unified protocol architecture
- [ ] `src/models/mod.rs` - Document the core data models
- [ ] `src/services/mod.rs` - Document the services architecture
- [ ] `src/demo/mod.rs` - Document the demo/reporting system
- [ ] `src/utils/mod.rs` - Document utility functions
- [ ] `src/mcp_server/mod.rs` - Enhance MCP server documentation
- [ ] `src/unified_protocol/adapters/mod.rs` - Document protocol adapters
- [ ] `src/services/cache/mod.rs` - Document caching strategy
- [ ] `src/services/makefile_linter/mod.rs` - Document Makefile linting

### Core Service Documentation
- [ ] `src/services/complexity.rs` - Add examples for complexity analysis
- [ ] `src/services/satd_detector.rs` - Add examples for SATD detection
- [ ] `src/services/dead_code_detector.rs` - Add examples for dead code detection
- [ ] `src/services/file_classifier.rs` - Add examples for file classification
- [ ] `src/services/duplicate_detector.rs` - Add examples for duplicate detection
- [ ] `src/services/deep_context.rs` - Add examples for deep context analysis
- [ ] `src/services/refactor_engine.rs` - Add examples for refactoring
- [ ] `src/services/quality_gate.rs` - Add examples for quality gates

## Priority 2: Public API Documentation

### Analysis APIs
- [ ] `src/cli/analyze.rs` - Document all analysis commands with examples
- [ ] `src/unified_protocol/handlers.rs` - Document all protocol handlers
- [ ] `src/mcp_server/handlers.rs` - Document all MCP handlers
- [ ] `src/services/dag_analyzer.rs` - Document DAG analysis with examples

### Context Generation APIs
- [ ] `src/cli/context.rs` - Document context generation commands
- [ ] `src/services/context_generator.rs` - Add comprehensive examples
- [ ] `src/services/git_operations.rs` - Document git integration

### Refactoring APIs
- [ ] `src/cli/refactor.rs` - Document refactoring commands
- [ ] `src/models/refactor.rs` - Document refactor state machine

## Priority 3: Test Documentation

### Property Tests Documentation
- [ ] `src/services/*_property_tests.rs` - Document property test strategies
- [ ] `src/models/dag_property_tests.rs` - Document DAG property tests

### Integration Tests Documentation
- [ ] `src/tests/mcp_protocol.rs` - Document MCP protocol tests
- [ ] `src/tests/cli_basic_tests.rs` - Document CLI test scenarios
- [ ] `src/tests/analyze_cli_tests.rs` - Document analysis test cases

## Priority 4: Utility and Helper Documentation

### Template System
- [ ] `src/services/template_engine.rs` - Document template engine with examples
- [ ] `src/tests/template_resources.rs` - Document template testing

### Error Handling
- [ ] `src/unified_protocol/error.rs` - Document error types with examples
- [ ] `src/unified_protocol/error_tests.rs` - Document error test cases

### Configuration
- [ ] `src/config/mod.rs` - Document configuration system
- [ ] `src/utils/metrics.rs` - Document metrics collection

## Implementation Strategy

### Phase 1: Core Modules (Week 1)
1. Add module-level documentation to all `mod.rs` files
2. Ensure all public structs/enums have documentation
3. Add at least one doctest per module

### Phase 2: Service Documentation (Week 2)
1. Document all service modules with usage examples
2. Add doctests for main service functions
3. Include error handling examples

### Phase 3: API Documentation (Week 3)
1. Document all CLI commands with examples
2. Document all protocol handlers
3. Add integration examples

### Phase 4: Test Documentation (Week 4)
1. Document test strategies and patterns
2. Add examples of how to write tests
3. Document test utilities

## Doctest Guidelines

### Good Doctest Example
```rust
/// Analyzes code complexity for a given file.
///
/// # Examples
///
/// ```
/// use pmat::services::complexity::analyze_complexity;
/// use std::path::Path;
///
/// let result = analyze_complexity(Path::new("src/main.rs")).unwrap();
/// assert!(result.complexity >= 0);
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn analyze_complexity(path: &Path) -> Result<ComplexityResult> {
    // ...
}
```

### Documentation Checklist
- [ ] Function purpose clearly stated
- [ ] All parameters documented
- [ ] Return value documented
- [ ] Error conditions documented
- [ ] At least one working example
- [ ] Example includes necessary imports
- [ ] Example demonstrates typical usage

## Metrics to Track
1. Files with documentation: Target 335+ (80%)
2. Files with doctests: Target 125+ (30%)
3. Public API coverage: Target 100%
4. Module documentation: Target 100%

## Tools to Use
- `cargo doc --no-deps --open` - Generate and view documentation
- `cargo test --doc` - Run all doctests
- `cargo rustdoc -- -Z unstable-options --show-coverage` - Show coverage stats