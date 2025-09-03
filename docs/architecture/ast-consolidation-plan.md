# AST Architecture Consolidation Plan - Sprint 49

## Current State Analysis

### File Inventory (31 files)
- **Core Model**: 1 file (`models/unified_ast.rs`)
- **Language Implementations**: 9 files (rust, python, typescript, c, cpp, kotlin, etc.)
- **Dispatch Files**: 3 files (c, cpp, typescript dispatch)
- **Strategy Pattern**: 3 files (strategies implementation)
- **Test Files**: 8 files
- **Unified/Engine**: 4 files (parser, engine implementations)

### Architecture Issues
1. **Duplication**: Multiple dispatch files with similar logic
2. **Scattered Logic**: AST parsing spread across 9+ language files
3. **Complex Dependencies**: Cross-references between strategies, dispatchers, and implementations
4. **Maintenance Burden**: 31 files for what should be ~5 core files

## Target Architecture

### Proposed Structure
```
server/src/ast/
├── mod.rs              # Public API and trait definitions
├── core.rs             # Core AST types (UnifiedAstNode, AstDag, etc.)
├── languages/
│   ├── mod.rs          # Language registry and trait
│   ├── rust.rs         # Rust implementation
│   ├── python.rs       # Python implementation
│   ├── typescript.rs   # TypeScript/JavaScript implementation
│   └── others.rs       # C, C++, Kotlin, etc.
├── parser.rs           # Unified parser orchestration
└── engine.rs           # AST engine and utilities
```

### Consolidation Strategy

#### Phase 1: Core Module Setup
1. Create `server/src/ast/` directory structure
2. Move `models/unified_ast.rs` → `ast/core.rs`
3. Define clean trait hierarchy in `ast/mod.rs`

#### Phase 2: Language Consolidation
1. Merge dispatch files into language implementations
2. Unify strategy pattern into `languages/mod.rs`
3. One file per language family (e.g., TypeScript handles JS too)

#### Phase 3: Parser/Engine Unification
1. Merge `unified_ast_parser.rs` and `unified_ast_engine.rs`
2. Create single parser orchestrator
3. Remove duplicate traversal logic

#### Phase 4: Service Integration
1. Update all imports to use new `ast::` module
2. Remove old files
3. Verify no functionality lost

## Implementation Plan

### Step 1: Create New Structure (Non-Breaking)
```rust
// server/src/ast/mod.rs
pub mod core;
pub mod languages;
pub mod parser;
pub mod engine;

pub use core::*;
pub use parser::UnifiedParser;
pub use engine::AstEngine;
```

### Step 2: Migrate Core Types
- Move `UnifiedAstNode`, `AstDag`, `Language` enum
- Keep backward compatibility exports

### Step 3: Consolidate Languages
- Merge dispatch logic into implementations
- Use trait objects for polymorphism
- Remove redundant strategy files

### Step 4: Update Services
- Change imports from scattered files to `ast::`
- Test each service after update
- Maintain test coverage

## Benefits

### Before: 31 files, complex dependencies
### After: ~6 files, clean architecture

1. **80% File Reduction**: 31 → 6 files
2. **Clear Separation**: Core, Languages, Parser, Engine
3. **Better Maintainability**: Single place for each concept
4. **Easier Extension**: Add new language = add one file
5. **Performance**: Preserved (no algorithmic changes)

## Risk Mitigation

1. **Backward Compatibility**: Keep old exports during transition
2. **Incremental Migration**: One language at a time
3. **Test Coverage**: Run full test suite after each step
4. **Performance Monitoring**: Benchmark before/after

## Success Criteria

- [ ] All 31 AST files consolidated to ~6
- [ ] All tests passing
- [ ] No performance regression
- [ ] Cleaner import structure
- [ ] Documentation updated
- [ ] Zero functional changes

## Timeline

- **Day 1**: Core module setup and type migration
- **Day 2**: Language consolidation
- **Day 3**: Parser/Engine unification
- **Day 4**: Service integration
- **Day 5**: Testing and documentation

## Next Steps

1. Create `server/src/ast/` directory
2. Begin migrating `unified_ast.rs`
3. Set up trait hierarchy
4. Start with Rust language as pilot