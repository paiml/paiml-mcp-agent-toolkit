# Cross-Language Analysis Implementation Plan

## Overview

This document outlines the implementation plan for the cross-language analysis feature in the PMAT toolkit. The feature will enable analysis of code spanning multiple programming languages, detection of cross-language dependencies, and visualization of language boundaries.

## Current Status

Integration tests have been created that define the expected behavior of the cross-language analysis feature. However, the core implementation has several issues that need to be addressed before the tests can run successfully.

## Key Components

1. **Unified AST Framework**
   - `UnifiedNode`: Language-agnostic representation of code elements
   - `LanguageMapper`: Interface for converting language-specific ASTs to unified representation
   - `CrossLanguageDependencies`: Detection and analysis of relationships between nodes in different languages

2. **Language Mappers**
   - `JavaMapper`: Maps Java code to unified nodes
   - `KotlinMapper`: Maps Kotlin code to unified nodes
   - `ScalaMapper`: Maps Scala code to unified nodes
   - `TypeScriptMapper`: Maps TypeScript code to unified nodes
   - `JavaScriptMapper`: Maps JavaScript code to unified nodes

3. **MCP Tools**
   - `PolyglotAnalysisTool`: Analyzes entire projects spanning multiple languages
   - `LanguageBoundaryTool`: Detects and analyzes language boundaries in a project

## Implementation Issues to Fix

### Import and Dependency Issues

1. **Missing HashSet Import**:
   - File: `server/src/ast/polyglot/cross_language_dependencies.rs`
   - Fix: Add `use std::collections::HashSet;`

2. **Missing tokio::fs Import**:
   - File: `server/src/ast/polyglot/language_mapper.rs`
   - Fix: Add `use tokio::fs;`

3. **Missing AstItem Import**:
   - Files: `server/src/ast/polyglot/language_mapper_factory.rs` and others
   - Fix: Add `use crate::services::context::AstItem;`

4. **Missing NodeKind Import**:
   - File: `server/src/ast/polyglot/language_mapper_factory.rs`
   - Fix: Add `use crate::ast::polyglot::NodeKind;`

### Feature Flag Issues

1. **Language Visitor Imports**:
   - Issue: Language-specific AST visitors (e.g., `KotlinAstVisitor`) are either missing or behind feature flags
   - Fix:
     ```rust
     #[cfg(feature = "kotlin-ast")]
     use crate::services::languages::kotlin::KotlinAstVisitor;
     ```

2. **Language Module Issues**:
   - Issue: Some language modules (`scala`, `typescript`, `javascript`) appear to be missing
   - Fix: Create these modules under `server/src/services/languages/` and gate them with feature flags

### AstItem and NodeKind Mismatches

1. **Missing NodeKind Variants**:
   - Issue: Some `NodeKind` variants referenced in code don't exist
   - Fix: Add missing variants or update references

2. **AstItem Variants Mismatch**:
   - Issue: Code references `AstItem` variants that don't exist in the current implementation
   - Fix: Update `from_ast_item` to handle only existing variants or add missing variants

### Path Validation Issues

1. **Incorrect Path Validation**:
   - File: `server/src/mcp_integration/polyglot_tools.rs`
   - Issue: Applying `!` to `Result<(), PathValidationError>`
   - Fix: Change to:
     ```rust
     if PathValidator::ensure_exists(&path).is_err() || !path.is_dir() {
     ```

### Unused Variable Warnings

1. **Unused Variables**:
   - Issue: Several unused variables causing compilation failures
   - Fix: Prefix with underscore (e.g., `_source`) or use them appropriately

## Feature Flags to Add

Add the following feature flags to `server/Cargo.toml`:

```toml
[features]
# Language-specific features
java-ast = []
kotlin-ast = []
scala-ast = []
typescript-ast = []
javascript-ast = []

# Meta-feature for all languages
polyglot-ast = [
    "java-ast",
    "kotlin-ast", 
    "scala-ast", 
    "typescript-ast",
    "javascript-ast"
]
```

## Implementation Plan

### Phase 1: Basic Framework and Fixes

1. Fix all compilation issues in existing code
2. Add missing language-specific visitor stubs and feature flags
3. Update `NodeKind` to align with `AstItem` variants
4. Implement basic `UnifiedNode` functionality

### Phase 2: Language Mapper Implementation

1. Implement `JavaMapper` (Java → Unified)
2. Implement `KotlinMapper` (Kotlin → Unified)
3. Implement `ScalaMapper` (Scala → Unified)
4. Implement `TypeScriptMapper` (TypeScript → Unified)
5. Implement `JavaScriptMapper` (JavaScript → Unified)

### Phase 3: Cross-Language Analysis

1. Complete `CrossLanguageDependencies` implementation
   - Reference resolution
   - Dependency detection
   - Visualization generation

### Phase 4: MCP Tools and Integration

1. Complete `PolyglotAnalysisTool` implementation
2. Complete `LanguageBoundaryTool` implementation
3. Connect to existing AST infrastructure

### Phase 5: Testing and Validation

1. Enable tests in `server/tests/polyglot_integration.rs`
2. Run tests with different language combinations
3. Address any issues discovered during testing

## Expected Outcome

Once completed, the cross-language analysis feature will enable:

1. Analysis of projects spanning multiple languages
2. Detection of cross-language dependencies (inheritance, usage, etc.)
3. Visualization of language boundaries and relationships
4. Recommendations for improved cross-language architecture

## Timeline

- Phase 1: 1 week
- Phase 2: 2 weeks
- Phase 3: 1 week
- Phase 4: 1 week
- Phase 5: 1 week

Total: 6 weeks (Sprint 52-57)