# GitHub Issue: Complete C/C++ Language Analyzer Integration with Deep Context

## Description

During Sprint 49, we implemented new C and C++ language analyzers and partially integrated them with deep_context.rs. However, several compilation issues need to be resolved before they can be fully enabled and included in a release.

## Current Status

We have successfully implemented:

1. C language analyzer (`services/ast/languages/c.rs`)
2. C++ language analyzer (`services/ast/languages/cpp.rs`) 
3. Thread-local caching for C/C++ analysis results in deep_context.rs
4. Updated documentation in Sprint 49 progress reports

## Compilation Issues

When trying to compile with the new language analyzers, we're seeing several issues:

1. **Import conflicts**:
   - `ComplexityMetrics` import issues in C/C++ analyzers
   - Missing modules in unified_context_builder.rs:
     - entropy
     - provability
     - graph_metrics
     - tdg
     - dead_code

2. **Trait implementation issues**:
   - CStrategy and CppStrategy need to implement services::ast::AstStrategy
   - The AST strategy pattern needs updating to support the new analyzers

3. **Variable usage issues**:
   - Unused variables in language analyzers
   - Variables prefixed with underscore but still used

4. **Integration issues**:
   - The analyze_file_by_language function has been updated but dependent code needs adjustments

## Tasks

- [ ] Fix import conflicts in unified_context_builder.rs
- [ ] Implement services::ast::AstStrategy trait for CStrategy and CppStrategy
- [ ] Resolve variable usage issues in language analyzers
- [ ] Add integration tests for the new language analyzers
- [ ] Update feature flags to enable the new analyzers
- [ ] Ensure backward compatibility with existing code
- [ ] Add comprehensive test coverage

## Technical Details

The implementation requires understanding several parts of the codebase:

1. **AST Module Structure**:
   - server/src/ast/ - Core AST module
   - server/src/services/ast/ - Service-level AST module
   - The relationship between these modules needs clarification

2. **Strategy Pattern**:
   - CStrategy and CppStrategy are defined in ast/languages/c_cpp.rs
   - They need to implement the AstStrategy trait from services/ast/mod.rs

3. **Language Analysis**:
   - Multiple implementations of language analyzers exist
   - Need to ensure consistent approach across languages

4. **Deep Context Integration**:
   - deep_context.rs uses analyze_file_by_language function
   - This function needs proper delegation to language-specific analyzers

## Related Changes

- PR #xxx: WebAssembly Disassembly Implementation
- PR #yyy: Sprint 49 Documentation Updates

## Priority

High - This completes a key technical debt reduction task from Sprint 49

## Labels

bug, enhancement, technical-debt

## Milestone

v2.171.0