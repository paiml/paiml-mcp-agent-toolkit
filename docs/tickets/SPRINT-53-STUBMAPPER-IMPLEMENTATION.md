# Sprint 53 StubMapper Implementation

## Overview

This task focuses on creating a `StubMapper` implementation for the polyglot AST framework. The StubMapper will serve as a fallback implementation when a language-specific mapper is requested but the corresponding feature flag is not enabled. This allows for testing and integration while maintaining a modular feature flag architecture.

## Goals

1. Create a StubMapper that implements the LanguageMapper trait
2. Provide basic functionality for testing purposes
3. Return appropriate errors for unsupported operations
4. Include comprehensive documentation and tests

## Implementation Details

### 1. StubMapper Structure

Create a new file `server/src/ast/polyglot/stub_mapper.rs` with the following implementation:

```rust
//! Stub implementation of LanguageMapper for testing
//!
//! This module provides a stub implementation of the LanguageMapper trait
//! that can be used when language-specific mappers are not available.
//! It implements the required interface but returns appropriate errors
//! or empty results for actual operations.

use crate::ast::polyglot::{Language, UnifiedNode};
use crate::ast::polyglot::language_mapper::LanguageMapper;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// A stub mapper implementation for testing
///
/// This mapper implements the LanguageMapper trait but provides minimal
/// functionality. It is used as a fallback when language-specific mappers
/// are not available due to feature flags being disabled.
#[derive(Debug, Clone)]
pub struct StubMapper {
    language: Language,
}

impl StubMapper {
    /// Create a new stub mapper for the specified language
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

#[async_trait]
impl LanguageMapper for StubMapper {
    /// Returns the language this mapper handles
    fn language(&self) -> Language {
        self.language
    }
    
    /// Stub implementation that returns an error
    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        Err(anyhow!(
            "StubMapper for {:?} cannot map files. Enable the '{}-ast' feature to use actual implementation.",
            self.language,
            self.language.name().to_lowercase()
        ))
    }
    
    /// Stub implementation that returns an error
    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        Err(anyhow!(
            "StubMapper for {:?} cannot map source. Enable the '{}-ast' feature to use actual implementation.",
            self.language,
            self.language.name().to_lowercase()
        ))
    }
    
    /// Stub implementation that returns an empty vector
    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        tracing::warn!(
            "Using StubMapper for {:?}. Enable the '{}-ast' feature for actual functionality.",
            self.language,
            self.language.name().to_lowercase()
        );
        
        // Return empty vec rather than error for directory mapping
        // This allows for graceful fallback in polyglot analysis
        Ok(Vec::new())
    }
    
    /// Stub implementation that returns true only for special test files
    fn can_map_file(&self, path: &Path) -> bool {
        // Only return true for special test files with the "stub_test" extension
        if let Some(ext) = path.extension() {
            if ext == "stub_test" {
                return true;
            }
        }
        
        // Otherwise check if this is a standard file for the language
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.language.file_extensions().contains(&ext_str);
            }
        }
        
        false
    }
    
    /// Create a minimal test node for stub testing
    async fn create_test_node(&self, path: &Path) -> Result<UnifiedNode> {
        use crate::ast::polyglot::{NodeKind, unified_node::SourcePosition};
        use std::collections::HashMap;
        
        // Extract a basic name from the path
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        let name = if file_name.contains('.') {
            file_name.split('.').next().unwrap_or("unknown").to_string()
        } else {
            file_name
        };
        
        // Create a minimal node for testing
        Ok(UnifiedNode {
            id: format!("{}:stub:{}", self.language.name(), name),
            kind: NodeKind::Unknown,
            name: name.clone(),
            fqn: format!("stub.{}", name),
            language: self.language,
            file_path: path.to_path_buf(),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references: Vec::new(),
            type_info: None,
            signature: None,
            documentation: Some("Stub node created for testing".to_string()),
            original_item: None,
            metadata: {
                let mut map = HashMap::new();
                map.insert("stub".to_string(), "true".to_string());
                map.insert("testing".to_string(), "true".to_string());
                map
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stub_mapper_language() {
        let mapper = StubMapper::new(Language::Java);
        assert_eq!(mapper.language(), Language::Java);
    }
    
    #[tokio::test]
    async fn test_stub_mapper_map_file() {
        let mapper = StubMapper::new(Language::Java);
        let result = mapper.map_file(Path::new("/test/file.java")).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Enable the 'java-ast' feature"));
    }
    
    #[tokio::test]
    async fn test_stub_mapper_map_source() {
        let mapper = StubMapper::new(Language::Java);
        let result = mapper.map_source("class Test {}", Path::new("/test/file.java")).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Enable the 'java-ast' feature"));
    }
    
    #[tokio::test]
    async fn test_stub_mapper_map_directory() {
        let mapper = StubMapper::new(Language::Java);
        let result = mapper.map_directory(Path::new("/test/"), true).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_stub_mapper_can_map_file() {
        let mapper = StubMapper::new(Language::Java);
        assert!(mapper.can_map_file(Path::new("/test/file.java")));
        assert!(mapper.can_map_file(Path::new("/test/test.stub_test")));
        assert!(!mapper.can_map_file(Path::new("/test/file.rs")));
        assert!(!mapper.can_map_file(Path::new("/test/file.py")));
    }
    
    #[tokio::test]
    async fn test_stub_mapper_create_test_node() {
        let mapper = StubMapper::new(Language::Java);
        let node = mapper.create_test_node(Path::new("/test/TestClass.java")).await.unwrap();
        
        assert_eq!(node.name, "TestClass");
        assert_eq!(node.language, Language::Java);
        assert_eq!(node.id, "Java:stub:TestClass");
        assert_eq!(node.fqn, "stub.TestClass");
        assert_eq!(node.metadata.get("stub"), Some(&"true".to_string()));
        assert_eq!(node.documentation.as_deref(), Some("Stub node created for testing"));
    }
}
```

### 2. Integration with Language Mapper Factory

Update `server/src/ast/polyglot/language_mapper_factory.rs` to use the StubMapper:

```rust
use crate::ast::polyglot::{Language, unified_node::UnifiedNode};
use crate::ast::polyglot::language_mapper::LanguageMapper;
use crate::ast::polyglot::stub_mapper::StubMapper;
use anyhow::{Result, anyhow};

pub struct LanguageMapperFactory;

impl LanguageMapperFactory {
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
            
            // When no language-specific mapper is available, use the StubMapper
            _ => {
                tracing::warn!(
                    "No language mapper implementation available for {:?}. Using StubMapper instead.",
                    language
                );
                Ok(Box::new(StubMapper::new(language)))
            }
        }
    }
}
```

### 3. Update Module Exports

Add the StubMapper module to `server/src/ast/polyglot/mod.rs`:

```rust
// Module exports
pub mod unified_node;
pub mod language_mapper;
pub mod cross_language_dependencies;
pub mod language_mapper_factory;
pub mod stub_mapper;

// Re-exports
pub use unified_node::UnifiedNode;
pub use language_mapper::LanguageMapper;
pub use cross_language_dependencies::CrossLanguageDependencies;
pub use language_mapper_factory::LanguageMapperFactory;
pub use stub_mapper::StubMapper;
```

### 4. Add Special Testing Method to LanguageMapper Trait

Update `server/src/ast/polyglot/language_mapper.rs` to add a testing method:

```rust
/// A trait for mapping language-specific code to unified nodes
#[async_trait]
pub trait LanguageMapper: Send + Sync {
    /// Returns the language this mapper handles
    fn language(&self) -> Language;
    
    /// Map a file to unified nodes
    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>>;
    
    /// Map source code to unified nodes
    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>>;
    
    /// Map a directory of files to unified nodes
    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        // Default implementation that walks the directory and maps each file
        let mut nodes = Vec::new();
        
        // Walk the directory
        if recursive {
            // Recursive directory walk
            let mut files_to_process = Vec::new();
            let entries = fs::read_dir(path).await?;
            
            // Process entries
            let mut entries_vec = Vec::new();
            let mut entry_stream = tokio_stream::wrappers::ReadDirStream::new(entries);
            while let Some(entry) = entry_stream.next().await {
                if let Ok(entry) = entry {
                    entries_vec.push(entry);
                }
            }
            
            for entry in entries_vec {
                let entry_path = entry.path();
                if entry_path.is_file() && self.can_map_file(&entry_path) {
                    files_to_process.push(entry_path);
                } else if entry_path.is_dir() {
                    // Recursively process subdirectory
                    let sub_nodes = self.map_directory(&entry_path, true).await?;
                    nodes.extend(sub_nodes);
                }
            }
            
            // Map files in parallel
            let results = futures::future::join_all(
                files_to_process.into_iter().map(|file_path| {
                    let mapper = self.clone_box();
                    async move {
                        mapper.map_file(&file_path).await
                    }
                })
            ).await;
            
            // Collect results
            for result in results {
                if let Ok(file_nodes) = result {
                    nodes.extend(file_nodes);
                }
            }
        } else {
            // Non-recursive directory walk
            let entries = fs::read_dir(path).await?;
            let mut entry_stream = tokio_stream::wrappers::ReadDirStream::new(entries);
            
            while let Some(entry) = entry_stream.next().await {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_file() && self.can_map_file(&entry_path) {
                        let file_nodes = self.map_file(&entry_path).await?;
                        nodes.extend(file_nodes);
                    }
                }
            }
        }
        
        Ok(nodes)
    }
    
    /// Check if this mapper can map the given file
    fn can_map_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.language().file_extensions().contains(&ext_str);
            }
        }
        
        false
    }
    
    /// Create a test node for testing purposes
    ///
    /// This is primarily used by StubMapper and in tests.
    /// Language-specific mappers can override this if needed.
    async fn create_test_node(&self, _path: &Path) -> Result<UnifiedNode> {
        Err(anyhow!("create_test_node not implemented for this mapper"))
    }
    
    /// Clone this mapper (box implementation)
    fn clone_box(&self) -> Box<dyn LanguageMapper>;
}
```

### 5. Add Clone Implementation for StubMapper

Implement Clone for StubMapper:

```rust
impl Clone for StubMapper {
    fn clone(&self) -> Self {
        Self {
            language: self.language,
        }
    }
}

impl StubMapper {
    // Implement clone_box for the LanguageMapper trait
    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}
```

## Integration Tests

Create integration tests for the StubMapper in `server/tests/stub_mapper_integration.rs`:

```rust
//! Integration tests for the StubMapper implementation
//!
//! These tests verify that the StubMapper works correctly when language-specific
//! mappers are not available due to feature flags being disabled.

use pmat::ast::polyglot::{Language, LanguageMapper, StubMapper, UnifiedNode};
use std::path::{Path, PathBuf};
use anyhow::Result;

#[tokio::test]
async fn test_stub_mapper_integration() -> Result<()> {
    // Create a stub mapper for Java
    let mapper = StubMapper::new(Language::Java);
    
    // Verify language reporting
    assert_eq!(mapper.language(), Language::Java);
    
    // Test file detection
    assert!(mapper.can_map_file(Path::new("test.java")));
    assert!(!mapper.can_map_file(Path::new("test.rs")));
    
    // Test mapping errors
    let file_result = mapper.map_file(Path::new("test.java")).await;
    assert!(file_result.is_err());
    assert!(file_result.unwrap_err().to_string().contains("java-ast"));
    
    // Test directory mapping (should return empty vec)
    let dir_result = mapper.map_directory(Path::new("."), false).await?;
    assert!(dir_result.is_empty());
    
    // Test creating a test node
    let test_node = mapper.create_test_node(Path::new("TestClass.java")).await?;
    assert_eq!(test_node.name, "TestClass");
    assert_eq!(test_node.language, Language::Java);
    
    Ok(())
}

#[tokio::test]
async fn test_stub_mapper_metadata() -> Result<()> {
    // Create a stub mapper and test node
    let mapper = StubMapper::new(Language::Kotlin);
    let node = mapper.create_test_node(Path::new("User.kt")).await?;
    
    // Check metadata
    assert_eq!(node.metadata.get("stub"), Some(&"true".to_string()));
    assert_eq!(node.metadata.get("testing"), Some(&"true".to_string()));
    
    // Check documentation
    assert!(node.documentation.unwrap().contains("testing"));
    
    Ok(())
}
```

## Success Criteria

1. The StubMapper successfully implements the LanguageMapper trait
2. Tests pass even when language-specific features are not enabled
3. The StubMapper provides useful error messages that guide users to enable the required features
4. Integration tests verify the StubMapper's behavior

## Estimated Effort

- Implementation: 0.5 day
- Testing: 0.5 day
- Documentation: 0.5 day

Total: 1.5 days

## Dependencies

- Requires updated LanguageMapper trait with clone_box and create_test_node methods
- Should be implemented after or alongside the language-specific feature flags

## Next Steps After Completion

1. Implement conditionally compiled language-specific mappers
2. Complete the cross-language dependency detection
3. Update polyglot tools to work with the StubMapper