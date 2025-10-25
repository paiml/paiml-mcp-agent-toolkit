//! Path validation utilities for polyglot AST
//!
//! This module provides specialized path validation functions for the polyglot AST framework.
//! It builds on top of the central PathValidator to add language-specific validation and
//! helper methods specifically designed for cross-language analysis.

use crate::utils::path_validator::{PathValidator, PathValidationError};
use crate::ast::polyglot::Language;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Specialized path validation utilities for polyglot AST
pub struct PolyglotPathValidator;

impl PolyglotPathValidator {
    /// Validate a directory path for polyglot analysis
    ///
    /// # Arguments
    /// * `path` - Directory path to validate
    ///
    /// # Returns
    /// * `Ok(())` if path exists and is a directory
    /// * `Err` with descriptive message otherwise
    pub fn validate_directory_path(path: &Path) -> Result<()> {
        PathValidator::validate_directory_anyhow(path)
            .map_err(|e| anyhow!("Invalid polyglot directory path: {}", e))
    }

    /// Validate a file path for polyglot analysis
    ///
    /// # Arguments
    /// * `path` - File path to validate
    ///
    /// # Returns
    /// * `Ok(())` if path exists and is a file
    /// * `Err` with descriptive message otherwise
    pub fn validate_file_path(path: &Path) -> Result<()> {
        PathValidator::validate_file_anyhow(path)
            .map_err(|e| anyhow!("Invalid polyglot file path: {}", e))
    }

    /// Check if a path is a valid language file
    ///
    /// # Arguments
    /// * `path` - File path to check
    /// * `language` - Optional language to check against
    ///
    /// # Returns
    /// * `true` if path is a valid file for the specified language (or any supported language if None)
    /// * `false` otherwise
    pub fn is_valid_language_file(path: &Path, language: Option<Language>) -> bool {
        if !PathValidator::is_valid_file(path) {
            return false;
        }

        match language {
            Some(lang) => Self::is_file_for_language(path, lang),
            None => Self::is_any_supported_language_file(path),
        }
    }
    
    /// Check if a file path is for a specific language
    ///
    /// # Arguments
    /// * `path` - File path to check
    /// * `language` - Language to check against
    ///
    /// # Returns
    /// * `true` if the file extension matches the language
    /// * `false` otherwise
    pub fn is_file_for_language(path: &Path, language: Language) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                language.file_extensions()
                    .iter()
                    .any(|&lang_ext| lang_ext.eq_ignore_ascii_case(ext))
            })
            .unwrap_or(false)
    }

    /// Check if a file path is for any supported language
    ///
    /// # Arguments
    /// * `path` - File path to check
    ///
    /// # Returns
    /// * `true` if the file extension matches any supported language
    /// * `false` otherwise
    pub fn is_any_supported_language_file(path: &Path) -> bool {
        Language::from_path(path).is_some()
    }
    
    /// Get all files with a specific language in a directory
    ///
    /// # Arguments
    /// * `directory` - Directory to search
    /// * `language` - Language to filter by
    /// * `recursive` - Whether to search recursively
    ///
    /// # Returns
    /// * Vector of paths to files matching the language
    pub fn get_language_files_in_dir(
        directory: &Path,
        language: Language,
        recursive: bool,
    ) -> Result<Vec<PathBuf>> {
        Self::validate_directory_path(directory)?;
        
        let mut result = Vec::new();
        Self::collect_language_files_in_dir(directory, language, recursive, &mut result)?;
        Ok(result)
    }
    
    /// Helper function to recursively collect language files
    fn collect_language_files_in_dir(
        directory: &Path,
        language: Language,
        recursive: bool,
        results: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if !directory.is_dir() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && Self::is_file_for_language(&path, language) {
                results.push(path.clone());
            } else if recursive && path.is_dir() {
                Self::collect_language_files_in_dir(&path, language, recursive, results)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_validate_directory_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        assert!(PolyglotPathValidator::validate_directory_path(temp_dir.path()).is_ok());
        
        let non_existent = temp_dir.path().join("non_existent");
        assert!(PolyglotPathValidator::validate_directory_path(&non_existent).is_err());
        
        Ok(())
    }

    #[test]
    fn test_validate_file_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.java");
        fs::write(&file_path, "public class Test {}")?;
        
        assert!(PolyglotPathValidator::validate_file_path(&file_path).is_ok());
        
        let non_existent = temp_dir.path().join("non_existent.java");
        assert!(PolyglotPathValidator::validate_file_path(&non_existent).is_err());
        
        Ok(())
    }

    #[test]
    fn test_is_valid_language_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create various language files
        let java_file = temp_dir.path().join("Test.java");
        let kotlin_file = temp_dir.path().join("Test.kt");
        let txt_file = temp_dir.path().join("readme.txt");
        
        fs::write(&java_file, "public class Test {}")?;
        fs::write(&kotlin_file, "data class Test(val name: String)")?;
        fs::write(&txt_file, "This is a text file")?;
        
        // Test with specific languages
        assert!(PolyglotPathValidator::is_valid_language_file(&java_file, Some(Language::Java)));
        assert!(!PolyglotPathValidator::is_valid_language_file(&java_file, Some(Language::Kotlin)));
        assert!(PolyglotPathValidator::is_valid_language_file(&kotlin_file, Some(Language::Kotlin)));
        assert!(!PolyglotPathValidator::is_valid_language_file(&txt_file, Some(Language::Java)));
        
        // Test for any supported language
        assert!(PolyglotPathValidator::is_valid_language_file(&java_file, None));
        assert!(PolyglotPathValidator::is_valid_language_file(&kotlin_file, None));
        assert!(!PolyglotPathValidator::is_valid_language_file(&txt_file, None));
        
        // Test with non-existent file
        let non_existent = temp_dir.path().join("non_existent.java");
        assert!(!PolyglotPathValidator::is_valid_language_file(&non_existent, Some(Language::Java)));
        
        Ok(())
    }

    #[test]
    fn test_is_file_for_language() {
        assert!(PolyglotPathValidator::is_file_for_language(
            Path::new("test.java"), 
            Language::Java
        ));
        
        assert!(PolyglotPathValidator::is_file_for_language(
            Path::new("test.kt"), 
            Language::Kotlin
        ));
        
        assert!(!PolyglotPathValidator::is_file_for_language(
            Path::new("test.java"), 
            Language::Kotlin
        ));
        
        assert!(!PolyglotPathValidator::is_file_for_language(
            Path::new("test.txt"), 
            Language::Java
        ));
    }

    #[test]
    fn test_get_language_files_in_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create nested directory structure
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir)?;
        
        // Create various language files
        let java_file1 = temp_dir.path().join("Test1.java");
        let java_file2 = temp_dir.path().join("Test2.java");
        let java_file3 = nested_dir.join("Test3.java");
        let kotlin_file = temp_dir.path().join("Test.kt");
        
        fs::write(&java_file1, "public class Test1 {}")?;
        fs::write(&java_file2, "public class Test2 {}")?;
        fs::write(&java_file3, "public class Test3 {}")?;
        fs::write(&kotlin_file, "data class Test(val name: String)")?;
        
        // Test non-recursive search
        let java_files = PolyglotPathValidator::get_language_files_in_dir(
            temp_dir.path(),
            Language::Java,
            false
        )?;
        
        assert_eq!(java_files.len(), 2);
        assert!(java_files.contains(&java_file1));
        assert!(java_files.contains(&java_file2));
        assert!(!java_files.contains(&java_file3)); // Should not include nested file
        
        // Test recursive search
        let java_files_recursive = PolyglotPathValidator::get_language_files_in_dir(
            temp_dir.path(),
            Language::Java,
            true
        )?;
        
        assert_eq!(java_files_recursive.len(), 3);
        assert!(java_files_recursive.contains(&java_file1));
        assert!(java_files_recursive.contains(&java_file2));
        assert!(java_files_recursive.contains(&java_file3)); // Should include nested file
        
        // Test with a different language
        let kotlin_files = PolyglotPathValidator::get_language_files_in_dir(
            temp_dir.path(),
            Language::Kotlin,
            true
        )?;
        
        assert_eq!(kotlin_files.len(), 1);
        assert!(kotlin_files.contains(&kotlin_file));
        
        Ok(())
    }
}