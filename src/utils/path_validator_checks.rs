impl PathValidator {
    /// Validate that a path exists
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let existing_path = Path::new("Cargo.toml");
    /// assert!(PathValidator::ensure_exists(existing_path).is_ok());
    /// ```
    pub fn ensure_exists(path: &Path) -> Result<(), PathValidationError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if !path.exists() {
            return Err(PathValidationError::NotFound {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Check if a path exists (returns boolean)
    ///
    /// Unlike ensure_exists, this returns a boolean instead of a Result
    /// This is useful in conditional expressions
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let existing_path = Path::new("Cargo.toml");
    /// assert!(PathValidator::path_exists(existing_path));
    /// ```
    #[must_use]
    pub fn path_exists(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.exists()
    }

    /// Validate that a path exists and is a file
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let file_path = Path::new("Cargo.toml");
    /// assert!(PathValidator::ensure_file(file_path).is_ok());
    /// ```
    pub fn ensure_file(path: &Path) -> Result<(), PathValidationError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self::ensure_exists(path)?;

        if !path.is_file() {
            return Err(PathValidationError::NotFile {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Check if a path exists and is a file (returns boolean)
    ///
    /// Unlike ensure_file, this returns a boolean instead of a Result
    /// This is useful in conditional expressions
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let file_path = Path::new("Cargo.toml");
    /// assert!(PathValidator::is_valid_file(file_path));
    /// ```
    #[must_use]
    pub fn is_valid_file(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.exists() && path.is_file()
    }

    /// Validate that a path exists and is a directory
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let dir_path = Path::new("src");
    /// assert!(PathValidator::ensure_directory(dir_path).is_ok());
    /// ```
    pub fn ensure_directory(path: &Path) -> Result<(), PathValidationError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self::ensure_exists(path)?;

        if !path.is_dir() {
            return Err(PathValidationError::NotDirectory {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Check if a path exists and is a directory (returns boolean)
    ///
    /// Unlike ensure_directory, this returns a boolean instead of a Result
    /// This is useful in conditional expressions
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let dir_path = Path::new("src");
    /// assert!(PathValidator::is_valid_directory(dir_path));
    /// ```
    #[must_use]
    pub fn is_valid_directory(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.exists() && path.is_dir()
    }

    /// Validate that a path exists and is readable
    pub fn ensure_readable(path: &Path) -> Result<(), PathValidationError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self::ensure_exists(path)?;

        // Check if we can read the file/directory
        match std::fs::metadata(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(PathValidationError::NotReadable {
                path: path.to_path_buf(),
            }),
        }
    }

    /// Get parent directory, validating it exists
    ///
    /// Returns the parent directory if the path is a file, or the path itself if it's a directory
    pub fn get_valid_parent(path: &Path) -> Result<&Path, PathValidationError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self::ensure_exists(path)?;

        if path.is_file() {
            path.parent().ok_or_else(|| PathValidationError::Invalid {
                path: path.to_path_buf(),
            })
        } else if path.is_dir() {
            Ok(path)
        } else {
            Err(PathValidationError::Invalid {
                path: path.to_path_buf(),
            })
        }
    }

    /// Check if path is a valid source file (with common extensions)
    #[must_use]
    pub fn is_source_file(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if !path.is_file() {
            return false;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                matches!(
                    ext,
                    "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp"
                )
            })
    }

    /// Validate path exists and return appropriate error for anyhow
    pub fn validate_exists_anyhow(path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if !path.exists() {
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }
        Ok(())
    }

    /// Validate path is file and return appropriate error for anyhow
    pub fn validate_file_anyhow(path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self::validate_exists_anyhow(path)?;

        if !path.is_file() {
            return Err(anyhow!("Path is not a file: {}", path.display()));
        }
        Ok(())
    }

    /// Validate path is directory and return appropriate error for anyhow
    pub fn validate_directory_anyhow(path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self::validate_exists_anyhow(path)?;

        if !path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", path.display()));
        }
        Ok(())
    }
}
