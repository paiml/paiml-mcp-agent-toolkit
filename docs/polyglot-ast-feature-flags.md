# Polyglot AST Feature Flags

## Overview

PMAT supports cross-language code analysis through a unified Abstract Syntax Tree (AST) framework. This document describes the feature flag system that enables customized builds with specific language support.

## Feature Flag System

The polyglot AST system uses Rust's conditional compilation feature flags to provide flexible language support. There are two levels of features:

1. **Meta-feature**: `polyglot-ast` - The main feature that enables the cross-language analysis framework
2. **Language-specific features**: Individual language support that depends on the meta-feature

### Available Feature Flags

#### Meta-feature

- `polyglot-ast`: Enables the core cross-language analysis framework

#### Language-specific features

Each language-specific feature depends on the `polyglot-ast` meta-feature:

- `polyglot-java`: Java language support
- `polyglot-kotlin`: Kotlin language support
- `polyglot-scala`: Scala language support
- `polyglot-typescript`: TypeScript language support
- `polyglot-javascript`: JavaScript language support
- `polyglot-csharp`: C# language support
- `polyglot-ruby`: Ruby language support

## Usage

### Default Configuration

By default, the `polyglot-ast` meta-feature and all language-specific features are enabled through the `default` feature set:

```toml
# In Cargo.toml
[features]
default = ["all-languages", "demo", "polyglot-ast"]
```

### Customizing Your Build

To build with only specific language support, you can disable the default features and enable only what you need:

```bash
# Build with only Java and Kotlin support
cargo build --no-default-features --features="polyglot-java,polyglot-kotlin"

# Build with all polyglot features but disable other default features
cargo build --no-default-features --features="polyglot-ast"
```

### In Library Dependencies

When using PMAT as a library, you can specify which language features to include:

```toml
# In your project's Cargo.toml
[dependencies]
pmat = { version = "2.158.0", default-features = false, features = ["polyglot-java", "polyglot-typescript"] }
```

## Implementation Details

### Feature Flag Architecture

The feature flag system works at multiple levels:

1. **Cargo.toml**: Defines the features and their dependencies
2. **Language Mapper Factory**: Uses conditional compilation to include/exclude language-specific mappers
3. **Individual Language Modules**: Use feature flags to conditionally compile language-specific code

### Code Examples

#### Language Mapper Factory

```rust
// In language_mapper_factory.rs
pub fn create(language: Language) -> Result<Arc<dyn LanguageMapper>> {
    match language {
        #[cfg(feature = "polyglot-java")]
        Language::Java => Ok(Arc::new(JavaMapper::new())),
        
        #[cfg(feature = "polyglot-kotlin")]
        Language::Kotlin => Ok(Arc::new(KotlinMapper::new())),
        
        // Default case for unsupported languages
        _ => Ok(Arc::new(StubMapper::new(language)))
    }
}
```

#### Language Module

```rust
// In typescript.rs
#[cfg(feature = "polyglot-typescript")]
pub fn analyze_ast(source: &str) -> Result<Vec<UnifiedNode>> {
    // TypeScript-specific analysis code
}

// Always available fallback
#[cfg(not(feature = "polyglot-typescript"))]
pub fn analyze_ast(source: &str) -> Result<Vec<UnifiedNode>> {
    // Use StubMapper for minimal functionality
    let mapper = StubMapper::new(Language::TypeScript);
    // Basic analysis with limited functionality
}
```

## Benefits

- **Reduced Binary Size**: Build with only the languages you need
- **Faster Compilation**: Skip compilation of unnecessary language parsers
- **Customizable Deployment**: Create specialized builds for specific use cases
- **Testing Flexibility**: Test specific language combinations in isolation

## Limitations

- Features are compile-time only; you cannot enable/disable languages at runtime
- Some analysis capabilities might be limited when specific language features are disabled
- The `StubMapper` provides minimal functionality for unsupported languages

## Future Enhancements

- Dynamic language loading for runtime language extension
- More granular feature flags for specific analysis capabilities
- Improved StubMapper for better graceful degradation

## Related Documentation

- [Language Support Documentation](/docs/languages.md)
- [Cross-Language Analysis Guide](/docs/cross-language-analysis.md)
- [MCP Integration for Polyglot Analysis](/docs/mcp/polyglot-tools.md)