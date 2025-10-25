# Sprint 53 Path Validation Fix for Polyglot Tools ✅ COMPLETED

## Overview

This task focuses on fixing the path validation issues in the polyglot tools implementation. The current code attempts to apply the logical NOT operator (`!`) to a `Result` type returned by `PathValidator::ensure_exists`, which causes compilation errors. This document outlines the necessary changes to fix these issues and ensure proper path validation throughout the polyglot AST framework.

## Current Issues

1. Incorrect usage of `PathValidator::ensure_exists` with `!` operator in `polyglot_tools.rs`
2. Potential inconsistencies in path validation across different modules
3. Missing proper error handling for path validation failures

## Goals

1. Fix path validation compilation errors in `polyglot_tools.rs`
2. Implement consistent path validation pattern throughout the polyglot framework
3. Improve error messages for path validation failures
4. Add comprehensive tests for path validation scenarios

## Implementation Details

### 1. Fix Immediate Compilation Errors

Update `server/src/mcp_integration/polyglot_tools.rs` to fix the incorrect path validation:

```rust
// Before:
if !PathValidator::ensure_exists(&path) || !path.is_dir() {
    return Err(McpError {
        code: crate::mcp_integration::error_codes::INVALID_PARAMS,
        message: format!("Path is not a directory: {}", path.display()),
        data: Some(json!({
            "path": path.display().to_string(),
            "suggestion": "Please provide a valid directory path"
        })),
    });
}

// After:
if PathValidator::ensure_exists(&path).is_err() || !path.is_dir() {
    return Err(McpError {
        code: crate::mcp_integration::error_codes::INVALID_PARAMS,
        message: format!("Path is not a directory: {}", path.display()),
        data: Some(json!({
            "path": path.display().to_string(),
            "suggestion": "Please provide a valid directory path"
        })),
    });
}
```

Apply this fix to both instances in `polyglot_tools.rs` (lines 85 and 323).

### 2. Create a Helper Function for Path Validation

Add a helper function in `server/src/ast/polyglot/utils.rs` for consistent path validation:

```rust
//! Utility functions for polyglot AST operations
//! 
//! This module provides common utility functions used throughout the polyglot AST framework.

use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use crate::utils::path_validator::PathValidator;

/// Validates a directory path for polyglot operations
/// 
/// Returns the validated path or an error if the path:
/// - Does not exist
/// - Is not a directory
/// - Does not have read permissions
pub fn validate_directory_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    
    // Check if path exists
    if let Err(e) = PathValidator::ensure_exists(path) {
        return Err(anyhow!("Path does not exist: {}: {}", path.display(), e));
    }
    
    // Check if path is a directory
    if !path.is_dir() {
        return Err(anyhow!("Path is not a directory: {}", path.display()));
    }
    
    // Convert to PathBuf and return
    Ok(path.to_path_buf())
}

/// Validates a file path for polyglot operations
/// 
/// Returns the validated path or an error if the path:
/// - Does not exist
/// - Is not a file
/// - Does not have read permissions
pub fn validate_file_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    
    // Check if path exists
    if let Err(e) = PathValidator::ensure_exists(path) {
        return Err(anyhow!("File does not exist: {}: {}", path.display(), e));
    }
    
    // Check if path is a file
    if !path.is_file() {
        return Err(anyhow!("Path is not a file: {}", path.display()));
    }
    
    // Convert to PathBuf and return
    Ok(path.to_path_buf())
}

/// Checks if a file can be processed by a language mapper
/// 
/// This is a lightweight check that only verifies if:
/// - The path exists and is a file
/// - The file extension matches one of the provided extensions
pub fn is_valid_language_file(path: impl AsRef<Path>, extensions: &[&str]) -> bool {
    let path = path.as_ref();
    
    // Check if path is a file
    if !path.is_file() {
        return false;
    }
    
    // Check if file has a matching extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return extensions.contains(&ext);
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_validate_directory_path() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();
        
        // Valid directory
        let result = validate_directory_path(dir_path);
        assert!(result.is_ok());
        
        // Non-existent directory
        let non_existent = dir_path.join("non_existent");
        let result = validate_directory_path(&non_existent);
        assert!(result.is_err());
        
        // File (not a directory)
        let file_path = dir_path.join("test.txt");
        File::create(&file_path).unwrap();
        let result = validate_directory_path(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }
    
    #[test]
    fn test_validate_file_path() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();
        
        // Create a test file
        let file_path = dir_path.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        
        // Valid file
        let result = validate_file_path(&file_path);
        assert!(result.is_ok());
        
        // Non-existent file
        let non_existent = dir_path.join("non_existent.txt");
        let result = validate_file_path(&non_existent);
        assert!(result.is_err());
        
        // Directory (not a file)
        let result = validate_file_path(dir_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a file"));
    }
    
    #[test]
    fn test_is_valid_language_file() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();
        
        // Create test files
        let java_file = dir_path.join("Test.java");
        File::create(&java_file).unwrap();
        
        let txt_file = dir_path.join("test.txt");
        File::create(&txt_file).unwrap();
        
        // Test valid extensions
        assert!(is_valid_language_file(&java_file, &["java", "kt"]));
        assert!(!is_valid_language_file(&txt_file, &["java", "kt"]));
        assert!(is_valid_language_file(&txt_file, &["txt"]));
        
        // Test directory
        assert!(!is_valid_language_file(dir_path, &["java"]));
        
        // Test non-existent file
        let non_existent = dir_path.join("NonExistent.java");
        assert!(!is_valid_language_file(&non_existent, &["java"]));
    }
}
```

### 3. Update Polyglot Tools to Use Helper Functions

Modify `server/src/mcp_integration/polyglot_tools.rs` to use the new helper functions:

```rust
use crate::ast::polyglot::utils::{validate_directory_path, validate_file_path};

// Replace path validation in execute method
async fn execute(&self, params: Value) -> Result<Value, McpError> {
    // Extract parameters
    let path_str = params["path"]
        .as_str()
        .ok_or_else(|| McpError {
            code: crate::mcp_integration::error_codes::INVALID_PARAMS,
            message: "Missing path parameter".to_string(),
            data: None,
        })?;
        
    let path = PathBuf::from(path_str);
    
    // Validate path (using new helper)
    let validated_path = validate_directory_path(&path).map_err(|e| McpError {
        code: crate::mcp_integration::error_codes::INVALID_PARAMS,
        message: e.to_string(),
        data: Some(json!({
            "path": path.display().to_string(),
            "suggestion": "Please provide a valid directory path"
        })),
    })?;
    
    // Rest of the method...
}
```

Apply similar updates to `LanguageBoundaryTool::execute` method.

### 4. Update Language Mappers to Use Helper Functions

Update the `LanguageMapper` implementations to use the new helper functions:

```rust
// In JavaMapper implementation
async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
    // Validate the file path
    let validated_path = validate_file_path(path)?;
    
    // Additional validation
    if !is_valid_language_file(path, &["java"]) {
        return Err(anyhow!("Not a Java file: {}", path.display()));
    }
    
    // Read the file
    let source = fs::read_to_string(&validated_path).await?;
    
    // Map the source
    self.map_source(&source, &validated_path).await
}

async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
    // Validate the directory path
    let validated_path = validate_directory_path(path)?;
    
    // Continue with directory mapping
    // ...
}
```

### 5. Update Module Structure to Include Utils

Add the utils module to `server/src/ast/polyglot/mod.rs`:

```rust
// Module exports
pub mod unified_node;
pub mod language_mapper;
pub mod cross_language_dependencies;
pub mod language_mapper_factory;
pub mod stub_mapper;
pub mod utils;

// Re-exports
pub use unified_node::UnifiedNode;
pub use language_mapper::LanguageMapper;
pub use cross_language_dependencies::CrossLanguageDependencies;
pub use language_mapper_factory::LanguageMapperFactory;
pub use stub_mapper::StubMapper;
pub use utils::{validate_directory_path, validate_file_path, is_valid_language_file};
```

### 6. Integration Tests for Path Validation

Add integration tests in `server/tests/polyglot_path_validation_tests.rs`:

```rust
//! Integration tests for polyglot path validation
//!
//! This module contains tests for the path validation functionality used throughout
//! the polyglot AST framework.

use pmat::ast::polyglot::utils::{validate_directory_path, validate_file_path, is_valid_language_file};
use std::path::Path;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;
use anyhow::Result;

#[test]
fn test_integration_validate_directory_path() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = temp_dir.path();
    
    // Test with valid directory
    let result = validate_directory_path(dir_path);
    assert!(result.is_ok());
    
    // Create a subdirectory
    let sub_dir = dir_path.join("sub_dir");
    std::fs::create_dir(&sub_dir)?;
    let result = validate_directory_path(&sub_dir);
    assert!(result.is_ok());
    
    // Test with invalid directory
    let invalid_dir = dir_path.join("invalid_dir");
    let result = validate_directory_path(&invalid_dir);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_integration_validate_file_path() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = temp_dir.path();
    
    // Create test files
    let java_file = dir_path.join("Test.java");
    let mut file = File::create(&java_file)?;
    writeln!(file, "public class Test {}")?;
    
    // Test with valid file
    let result = validate_file_path(&java_file);
    assert!(result.is_ok());
    
    // Test with invalid file
    let invalid_file = dir_path.join("invalid.java");
    let result = validate_file_path(&invalid_file);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_integration_is_valid_language_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = temp_dir.path();
    
    // Create test files for various languages
    let java_file = dir_path.join("Test.java");
    File::create(&java_file)?;
    
    let kotlin_file = dir_path.join("Test.kt");
    File::create(&kotlin_file)?;
    
    let scala_file = dir_path.join("Test.scala");
    File::create(&scala_file)?;
    
    let ts_file = dir_path.join("Test.ts");
    File::create(&ts_file)?;
    
    // Test language-specific validation
    assert!(is_valid_language_file(&java_file, &["java"]));
    assert!(!is_valid_language_file(&java_file, &["kt", "scala"]));
    
    assert!(is_valid_language_file(&kotlin_file, &["kt"]));
    assert!(is_valid_language_file(&scala_file, &["scala"]));
    assert!(is_valid_language_file(&ts_file, &["ts", "tsx"]));
    
    // Test multiple extensions
    assert!(is_valid_language_file(&java_file, &["java", "kt", "scala", "ts"]));
    
    Ok(())
}

#[tokio::test]
async fn test_language_mapper_path_validation() -> Result<()> {
    use pmat::ast::polyglot::{Language, StubMapper, LanguageMapper};
    
    let temp_dir = tempdir()?;
    let dir_path = temp_dir.path();
    
    // Create a mapper
    let mapper = StubMapper::new(Language::Java);
    
    // Create a valid Java file
    let java_file = dir_path.join("Test.java");
    File::create(&java_file)?;
    
    // Create a non-Java file
    let txt_file = dir_path.join("test.txt");
    File::create(&txt_file)?;
    
    // Test can_map_file
    assert!(mapper.can_map_file(&java_file));
    assert!(!mapper.can_map_file(&txt_file));
    
    // Test map_file (should error with StubMapper, but validate paths first)
    let map_result = mapper.map_file(&java_file).await;
    assert!(map_result.is_err());
    // Error should be about StubMapper, not the path validation
    assert!(map_result.unwrap_err().to_string().contains("StubMapper"));
    
    // Test with non-existent file
    let non_existent = dir_path.join("NonExistent.java");
    let map_result = mapper.map_file(&non_existent).await;
    assert!(map_result.is_err());
    
    Ok(())
}
```

## Success Criteria

1. All compilation errors related to path validation are resolved
2. Consistent path validation is used throughout the polyglot framework
3. Error messages are clear and helpful
4. Tests pass for various path validation scenarios

## Estimated Effort

- Implementation: 0.5 day ✅
- Testing: 0.5 day ✅
- Integration: 0.5 day ✅

Total: 1.5 days ✅ COMPLETED

## Dependencies

- Should be implemented early in the process, as it fixes a critical compilation error ✅
- Other components like StubMapper and language mappers will use these path validation utilities ✅

## Implementation Status

✅ **COMPLETED**: Path validation fixes have been implemented and integrated.

The implementation is documented in detail in the following file:
- [SPRINT-53-PATH-VALIDATION-FIX-SUMMARY.md](./SPRINT-53-PATH-VALIDATION-FIX-SUMMARY.md)

## Next Steps

1. Complete feature flag implementation
2. Implement the StubMapper
3. Fix AstItem and NodeKind mismatches