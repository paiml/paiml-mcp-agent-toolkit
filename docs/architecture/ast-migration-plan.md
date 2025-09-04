# AST Migration Plan - Sprint 49 Phase 2

## Current State

### Old AST Files (31 files to be deleted)

#### Core Files (28)
1. `server/src/models/unified_ast.rs` - Old unified AST model
2. `server/src/services/ast_based_dependency_analyzer.rs` - Dependency analysis
3. `server/src/services/ast_based_dependency_analyzer_tests.rs` - Tests
4. `server/src/services/ast_c.rs` - C language parsing
5. `server/src/services/ast_c_dispatch.rs` - C dispatch logic
6. `server/src/services/ast_cpp.rs` - C++ language parsing
7. `server/src/services/ast_cpp_dispatch.rs` - C++ dispatch logic
8. `server/src/services/ast_kotlin.rs` - Kotlin language parsing
9. `server/src/services/ast_python.rs` - Python language parsing
10. `server/src/services/ast_rust.rs` - Rust language parsing
11. `server/src/services/ast_rust_property_tests.rs` - Rust property tests
12. `server/src/services/ast_rust_unified.rs` - Unified Rust AST
13. `server/src/services/ast_strategies.rs` - Strategy pattern implementation
14. `server/src/services/ast_strategies_temp.rs` - Temporary strategies
15. `server/src/services/ast_strategies_tests.rs` - Strategy tests
16. `server/src/services/ast_typescript.rs` - TypeScript parsing
17. `server/src/services/ast_typescript_dispatch.rs` - TypeScript dispatch
18. `server/src/services/ast_typescript_property_tests.rs` - TypeScript tests
19. `server/src/services/ast_import_tests.rs` - Import tests
20. `server/src/services/ast_import_property_tests.rs` - Import property tests
21. `server/src/services/unified_ast_engine.rs` - Old AST engine
22. `server/src/services/unified_ast_parser.rs` - Old parser
23. `server/src/services/makefile_linter/ast.rs` - Makefile AST
24. `server/src/services/cache/cache_property_tests_fast.rs` - Cache tests
25. `server/src/tdg/analyzer_ast.rs` - TDG AST analyzer
26. `server/src/tests/ast_e2e.rs` - E2E tests
27. `server/src/tests/ast_regression_test.rs` - Regression tests
28. `server/src/tests/property_expansion/ast_parser_properties.rs` - Property tests

#### Backup Files (3)
1. `server/src/services/ast_rust_unified.rs.backup`
2. `server/src/services/ast_strategies_temp.rs.backup`
3. `server/src/tests/ast_e2e.rs.backup`

### Services Using Old AST (20 files need updates)

#### High Priority (Core Services)
1. `server/src/services/quality_proxy.rs` - Uses `ast_rust::analyze_rust_file_with_complexity`
2. `server/src/services/complexity.rs` - Core complexity analysis
3. `server/src/services/deep_context.rs` - Deep context analysis
4. `server/src/services/context.rs` - Context extraction
5. `server/src/services/refactor_engine.rs` - Refactoring engine

#### Medium Priority (Analysis Tools)
6. `server/src/cli/analysis_utilities.rs` - CLI analysis utilities
7. `server/src/services/tdg_calculator.rs` - TDG calculation
8. `server/src/services/artifact_writer.rs` - Artifact generation
9. `server/src/cli/handlers/refactor_handlers.rs` - Refactor handlers
10. `server/src/handlers/tools.rs` - Tool handlers

#### Low Priority (Tests and Demo)
11. `server/src/demo/mod.rs` - Demo code
12. `server/src/services/complexity_fix_tests.rs` - Complexity tests
13. `server/src/tests/ast_e2e.rs` - E2E tests
14. `server/src/services/mod.rs` - Module exports

#### Already Migrating (Self-referential)
15-20. Various AST files that reference each other

## Migration Strategy

### Phase 1: Foundation (COMPLETE ✅)
- Created new `server/src/ast/` module structure
- Implemented language strategies for Rust, Python, TypeScript, C/C++
- Module compiles successfully

### Phase 2: Service Compatibility Layer (CURRENT)
Create compatibility shims to allow gradual migration:

```rust
// In server/src/services/ast_rust.rs (temporary compatibility)
pub use crate::ast::languages::rust::*;

// Add compatibility function
pub async fn analyze_rust_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    // Delegate to new AST module
    // ... implementation
}
```

### Phase 3: Service Migration
Migrate services in priority order:

1. **Week 1**: Core services (quality_proxy, complexity, deep_context)
2. **Week 2**: Analysis tools (CLI utilities, TDG calculator)
3. **Week 3**: Tests and demo code

### Phase 4: Cleanup
Once all services are migrated:
1. Remove compatibility shims
2. Delete all 31 old AST files
3. Update module exports
4. Run full test suite

## Migration Checklist

### For Each Service
- [ ] Identify old AST imports
- [ ] Map to new AST module equivalents
- [ ] Update import statements
- [ ] Verify functionality
- [ ] Run service-specific tests

### Example Migration

#### Before
```rust
use crate::services::ast_rust::analyze_rust_file_with_complexity;
use crate::models::unified_ast::UnifiedAstNode;
```

#### After
```rust
use crate::ast::languages::rust::RustStrategy;
use crate::ast::core::UnifiedAstNode;
use crate::ast::LanguageStrategy;
```

## Risk Mitigation

1. **Gradual Migration**: Use compatibility layer to avoid breaking changes
2. **Test Coverage**: Ensure tests pass after each migration
3. **Rollback Plan**: Keep old files until migration is verified
4. **Documentation**: Update all documentation as services migrate

## Success Criteria

- [ ] All 20 services migrated to new AST module
- [ ] All tests passing
- [ ] 31 old AST files deleted
- [ ] Documentation updated
- [ ] No performance regression
- [ ] Code coverage maintained at ≥80%

## Timeline

- **Sprint 49**: Foundation and planning (DONE)
- **Sprint 50**: Service migration (Week 1-2)
- **Sprint 51**: Cleanup and validation (Week 3)
- **Sprint 52**: Documentation and optimization

## Benefits After Migration

1. **80% File Reduction**: 31 files → 6 files
2. **Cleaner Architecture**: Single source of truth for AST
3. **Easier Maintenance**: All AST logic in one module
4. **Better Extensibility**: Add new languages easily
5. **Improved Performance**: Optimized data structures
6. **Type Safety**: Stronger typing across boundaries