# Sprint 53 Language-Specific Feature Flags Implementation

## Overview

This task focuses on implementing the feature flag architecture for language-specific implementations in the polyglot AST framework. This is a critical foundational step for the cross-language analysis feature, as it will allow for conditional compilation of language-specific components.

## Goals

1. Add feature flags for all supported languages in the polyglot AST framework
2. Fix compilation issues in the core polyglot AST module
3. Implement a clean conditional compilation strategy
4. Ensure backward compatibility with existing code

## Implementation Details

### 1. Feature Flag Architecture

Add the following feature flags to `server/Cargo.toml`:

```toml
[features]
# Existing features
# ...

# Language-specific AST features
java-ast = ["dep:tree-sitter-java"]
kotlin-ast = ["dep:tree-sitter-kotlin-ng"]
scala-ast = ["dep:tree-sitter-scala"]
typescript-ast = ["dep:tree-sitter-typescript"]
javascript-ast = ["dep:tree-sitter-javascript"]

# Meta-feature for all language AST support
polyglot-ast = [
    "java-ast",
    "kotlin-ast", 
    "scala-ast", 
    "typescript-ast",
    "javascript-ast"
]
```

### 2. Module Structure and Imports

Update imports in polyglot modules to use conditional compilation:

```rust
// In server/src/ast/polyglot/mod.rs
#[cfg(feature = "java-ast")]
pub mod java;

#[cfg(feature = "kotlin-ast")]
pub mod kotlin;

#[cfg(feature = "scala-ast")]
pub mod scala;

#[cfg(feature = "typescript-ast")]
pub mod typescript;

#[cfg(feature = "javascript-ast")]
pub mod javascript;
```

### 3. Language Mapper Factory

Update the language mapper factory to conditionally create mappers:

```rust
// In server/src/ast/polyglot/language_mapper_factory.rs
pub fn create(language: Language) -> Result<Box<dyn LanguageMapper>> {
    match language {
        #[cfg(feature = "java-ast")]
        Language::Java => Ok(Box::new(JavaMapper::new())),
        
        #[cfg(feature = "kotlin-ast")]
        Language::Kotlin => Ok(Box::new(KotlinMapper::new())),
        
        #[cfg(feature = "scala-ast")]
        Language::Scala => Ok(Box::new(ScalaMapper::new())),
        
        #[cfg(feature = "typescript-ast")]
        Language::TypeScript => Ok(Box::new(TypeScriptMapper::new())),
        
        #[cfg(feature = "javascript-ast")]
        Language::JavaScript => Ok(Box::new(JavaScriptMapper::new())),
        
        #[cfg(not(any(
            feature = "java-ast",
            feature = "kotlin-ast",
            feature = "scala-ast",
            feature = "typescript-ast",
            feature = "javascript-ast"
        )))]
        _ => Ok(Box::new(StubMapper::new(language))),
        
        // When features are enabled but we don't have a mapper for this language
        _ => Err(anyhow!("No language mapper available for {:?}", language)),
    }
}
```

### 4. Fix Import Issues

Fix the import issues in:

- `server/src/ast/polyglot/cross_language_dependencies.rs`
- `server/src/ast/polyglot/language_mapper.rs`
- `server/src/ast/polyglot/language_mapper_factory.rs`
- `server/src/mcp_integration/polyglot_tools.rs`

Example fix for `cross_language_dependencies.rs`:

```rust
use crate::services::context::AstItem;
use crate::ast::polyglot::{Language, NodeKind};
use crate::ast::polyglot::unified_node::ReferenceKind;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use serde::{Serialize, Deserialize};
```

### 5. Path Validation Fix

Fix the path validation in `polyglot_tools.rs`:

```rust
// Before
if !PathValidator::ensure_exists(&path) || !path.is_dir() {
    // ...
}

// After
if PathValidator::ensure_exists(&path).is_err() || !path.is_dir() {
    // ...
}
```

### 6. Test Updates

Update the tests to work with the feature flags:

```rust
// In server/tests/polyglot_integration.rs
#[cfg_attr(not(feature = "polyglot-ast"), ignore = "Requires polyglot-ast feature")]
#[tokio::test]
async fn test_polyglot_analyzer_with_fixtures() -> Result<()> {
    // ...
}
```

## Success Criteria

1. The codebase compiles successfully with `cargo build`
2. Basic tests run successfully with `cargo test`
3. Feature flags correctly gate language-specific functionality
4. Polyglot tests are properly ignored when features are not enabled
5. Documentation reflects the feature flag requirements

## Estimated Effort

- Code changes: 1-2 days
- Testing: 1 day
- Documentation: 0.5 day

Total: 2-3 days

## Dependencies

- None. This is a foundational task that other cross-language analysis work depends on.

## Next Steps After Completion

1. Implement the actual language mappers (Java, Kotlin, Scala)
2. Fix AstItem and NodeKind mismatches
3. Complete the cross-language dependency detection logic