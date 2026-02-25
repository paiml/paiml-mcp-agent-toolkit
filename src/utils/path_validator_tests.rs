#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_exists_valid_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;
        assert!(PathValidator::ensure_exists(&file_path).is_ok());
        Ok(())
    }

    #[test]
    fn test_ensure_exists_invalid_file() {
        let path = Path::new("/tmp/nonexistent_file_that_definitely_does_not_exist_12345.txt");
        assert!(PathValidator::ensure_exists(path).is_err());
    }

    #[test]
    fn test_ensure_file_valid() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;
        assert!(PathValidator::ensure_file(&file_path).is_ok());
        Ok(())
    }

    #[test]
    fn test_ensure_directory_valid() -> Result<()> {
        let temp_dir = TempDir::new()?;
        assert!(PathValidator::ensure_directory(temp_dir.path()).is_ok());
        Ok(())
    }

    #[test]
    fn test_get_valid_parent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;

        let parent = PathValidator::get_valid_parent(&file_path)?;
        assert_eq!(parent, temp_dir.path());

        Ok(())
    }

    #[test]
    fn test_boolean_path_validators() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create a file
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;

        // Create a directory
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;

        // Test non-existent path
        let non_existent = temp_dir.path().join("does_not_exist");

        // Test path_exists
        assert!(PathValidator::path_exists(&file_path));
        assert!(PathValidator::path_exists(&dir_path));
        assert!(!PathValidator::path_exists(&non_existent));

        // Test is_valid_file
        assert!(PathValidator::is_valid_file(&file_path));
        assert!(!PathValidator::is_valid_file(&dir_path));
        assert!(!PathValidator::is_valid_file(&non_existent));

        // Test is_valid_directory
        assert!(PathValidator::is_valid_directory(&dir_path));
        assert!(!PathValidator::is_valid_directory(&file_path));
        assert!(!PathValidator::is_valid_directory(&non_existent));

        Ok(())
    }

    #[test]
    fn test_is_source_file() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test Rust file
        let rs_file = temp_dir.path().join("test.rs");
        fs::write(&rs_file, "fn main() {}")?;
        assert!(PathValidator::is_source_file(&rs_file));

        // Test non-source file
        let txt_file = temp_dir.path().join("test.txt");
        fs::write(&txt_file, "text content")?;
        assert!(!PathValidator::is_source_file(&txt_file));

        Ok(())
    }

    #[test]
    fn test_validate_anyhow_methods() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test with a file (should exist)
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;
        assert!(PathValidator::validate_exists_anyhow(&file_path).is_ok());
        assert!(PathValidator::validate_file_anyhow(&file_path).is_ok());

        // Test with a directory
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;
        assert!(PathValidator::validate_directory_anyhow(&dir_path).is_ok());

        // Test boolean methods
        assert!(PathValidator::path_exists(&file_path));
        assert!(PathValidator::is_valid_file(&file_path));
        assert!(PathValidator::is_valid_directory(&dir_path));
        assert!(!PathValidator::is_valid_file(&dir_path));
        assert!(!PathValidator::is_valid_directory(&file_path));

        // Test with nonexistent path
        let bad_path = temp_dir.path().join("nonexistent_123456");
        assert!(PathValidator::validate_exists_anyhow(&bad_path).is_err());

        Ok(())
    }

    #[test]
    fn test_ensure_file_with_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;

        // Trying to validate a directory as a file should fail
        let result = PathValidator::ensure_file(&dir_path);
        assert!(result.is_err());
        match result {
            Err(PathValidationError::NotFile { path }) => {
                assert_eq!(path, dir_path);
            }
            _ => panic!("Expected NotFile error"),
        }
        Ok(())
    }

    #[test]
    fn test_ensure_directory_with_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;

        // Trying to validate a file as a directory should fail
        let result = PathValidator::ensure_directory(&file_path);
        assert!(result.is_err());
        match result {
            Err(PathValidationError::NotDirectory { path }) => {
                assert_eq!(path, file_path);
            }
            _ => panic!("Expected NotDirectory error"),
        }
        Ok(())
    }

    #[test]
    fn test_ensure_readable_valid() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("readable.txt");
        fs::write(&file_path, "content")?;

        assert!(PathValidator::ensure_readable(&file_path).is_ok());
        Ok(())
    }

    #[test]
    fn test_ensure_readable_nonexistent() {
        let path = Path::new("/tmp/nonexistent_readable_file_12345.txt");
        let result = PathValidator::ensure_readable(path);
        assert!(result.is_err());
        match result {
            Err(PathValidationError::NotFound { .. }) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_get_valid_parent_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // For a directory, get_valid_parent should return the directory itself
        let parent = PathValidator::get_valid_parent(temp_dir.path())?;
        assert_eq!(parent, temp_dir.path());
        Ok(())
    }

    #[test]
    fn test_get_valid_parent_nonexistent() {
        let path = Path::new("/tmp/nonexistent_parent_test_12345");
        let result = PathValidator::get_valid_parent(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_anyhow_with_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;

        // Directory should fail validation as file
        let result = PathValidator::validate_file_anyhow(&dir_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not a file"));
        Ok(())
    }

    #[test]
    fn test_validate_directory_anyhow_with_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "content")?;

        // File should fail validation as directory
        let result = PathValidator::validate_directory_anyhow(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not a directory"));
        Ok(())
    }

    #[test]
    fn test_path_validation_error_display() {
        let path = PathBuf::from("/test/path");

        let err = PathValidationError::NotFound { path: path.clone() };
        assert!(err.to_string().contains("/test/path"));
        assert!(err.to_string().contains("does not exist"));

        let err = PathValidationError::NotFile { path: path.clone() };
        assert!(err.to_string().contains("not a file"));

        let err = PathValidationError::NotDirectory { path: path.clone() };
        assert!(err.to_string().contains("not a directory"));

        let err = PathValidationError::NotReadable { path: path.clone() };
        assert!(err.to_string().contains("not readable"));

        let err = PathValidationError::Invalid { path };
        assert!(err.to_string().contains("Invalid path"));
    }

    #[test]
    fn test_path_validation_error_clone() {
        let path = PathBuf::from("/test/path");
        let err = PathValidationError::NotFound { path };
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_path_validation_error_debug() {
        let path = PathBuf::from("/test/path");
        let err = PathValidationError::NotFound { path };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_is_source_file_all_extensions() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test all supported source extensions
        let extensions = ["rs", "py", "js", "ts", "go", "c", "cpp", "h", "hpp"];
        for ext in extensions {
            let file_path = temp_dir.path().join(format!("test.{}", ext));
            fs::write(&file_path, "content")?;
            assert!(
                PathValidator::is_source_file(&file_path),
                "Failed for extension: {}",
                ext
            );
        }

        // Test non-source extensions
        let non_source = ["md", "json", "toml", "yaml"];
        for ext in non_source {
            let file_path = temp_dir.path().join(format!("test.{}", ext));
            fs::write(&file_path, "content")?;
            assert!(
                !PathValidator::is_source_file(&file_path),
                "Should not be source file: {}",
                ext
            );
        }

        Ok(())
    }

    #[test]
    fn test_is_source_file_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("src");
        fs::create_dir(&dir_path)?;

        // Directories should not be considered source files
        assert!(!PathValidator::is_source_file(&dir_path));
        Ok(())
    }

    #[test]
    fn test_is_source_file_no_extension() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("Makefile");
        fs::write(&file_path, "content")?;

        // Files without extensions should not be source files
        assert!(!PathValidator::is_source_file(&file_path));
        Ok(())
    }
}
