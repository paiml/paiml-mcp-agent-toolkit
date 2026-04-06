// Worker temporary file operations - included by temp_file.rs

impl WorkerTempFile {
    /// Create a new temporary file for a specific worker and mutant
    ///
    /// # Arguments
    ///
    /// * `worker_id` - ID of the worker
    /// * `mutant_id` - ID of the mutant
    /// * `extension` - File extension to use (defaults to "rs")
    ///
    /// # Returns
    ///
    /// A new WorkerTempFile instance
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(worker_id: usize, mutant_id: usize, extension: Option<&str>) -> Self {
        let ext = extension.unwrap_or("rs");
        let filename = format!("pmat_w{}_{}.{}", worker_id, mutant_id, ext);
        let path = std::env::temp_dir().join(filename);

        Self {
            path,
            cleaned_up: false,
            use_sync_cleanup: true,
        }
    }

    /// Create a new temporary file with a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the temporary file
    ///
    /// # Returns
    ///
    /// A new WorkerTempFile instance
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn with_path(path: PathBuf) -> Self {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self {
            path,
            cleaned_up: false,
            use_sync_cleanup: true,
        }
    }

    /// Set whether to use synchronous cleanup in Drop
    ///
    /// By default, the Drop implementation uses synchronous file operations
    /// to ensure cleanup even if the async runtime is shutting down.
    /// Set this to false if you want to disable that behavior.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_sync_cleanup(mut self, use_sync: bool) -> Self {
        self.use_sync_cleanup = use_sync;
        self
    }

    /// Write content to the temporary file
    ///
    /// # Arguments
    ///
    /// * `content` - Content to write to the file
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn write(&self, content: impl AsRef<[u8]>) -> Result<()> {
        fs::write(&self.path, content)
            .await
            .with_context(|| format!("Failed to write to temporary file: {}", self.path.display()))
    }

    /// Read content from the temporary file
    ///
    /// # Returns
    ///
    /// The file content as a byte vector
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn read(&self) -> Result<Vec<u8>> {
        fs::read(&self.path)
            .await
            .with_context(|| format!("Failed to read temporary file: {}", self.path.display()))
    }

    /// Read content from the temporary file as string
    ///
    /// # Returns
    ///
    /// The file content as a string
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or if content is not valid UTF-8
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn read_to_string(&self) -> Result<String> {
        fs::read_to_string(&self.path)
            .await
            .with_context(|| format!("Failed to read temporary file: {}", self.path.display()))
    }

    /// Check if the temporary file exists
    ///
    /// # Returns
    ///
    /// true if the file exists, false otherwise
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn exists(&self) -> bool {
        fs::try_exists(&self.path).await.unwrap_or(false)
    }

    /// Clean up the temporary file
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn cleanup(&mut self) -> Result<()> {
        if self.cleaned_up || !self.exists().await {
            return Ok(());
        }

        fs::remove_file(&self.path)
            .await
            .with_context(|| format!("Failed to remove temporary file: {}", self.path.display()))?;

        self.cleaned_up = true;
        Ok(())
    }

    /// Get the path to the temporary file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Mark the file as cleaned up without actually cleaning it up
    ///
    /// This is useful if you want to keep the file around after the
    /// WorkerTempFile is dropped.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn mark_cleaned_up(&mut self) {
        self.cleaned_up = true;
    }

    /// Copy this temporary file to another path
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination path
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if copying fails
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn copy_to(&self, dest: &Path) -> Result<()> {
        debug_assert!(dest.exists(), "dest must exist: {}", dest.display());
        fs::copy(&self.path, dest).await.with_context(|| {
            format!(
                "Failed to copy temporary file from {} to {}",
                self.path.display(),
                dest.display()
            )
        })?;

        Ok(())
    }
}

impl Drop for WorkerTempFile {
    fn drop(&mut self) {
        if self.cleaned_up {
            return;
        }

        if self.use_sync_cleanup {
            // Use blocking FS to ensure cleanup even during runtime shutdown
            if std::path::Path::new(&self.path).exists() {
                if let Err(e) = std::fs::remove_file(&self.path) {
                    eprintln!("Error removing temporary file in drop: {}", e);
                }
            }
        } else {
            // Just log that cleanup was not performed
            // This might happen if the async runtime is shutting down
            eprintln!(
                "Warning: Temporary file not cleaned up: {}",
                self.path.display()
            );
        }
    }
}

/// Utility to create a unique temporary directory
///
/// Creates a unique temporary directory with the given prefix and
/// returns a PathBuf to it.
///
/// # Arguments
///
/// * `prefix` - Prefix for the directory name
///
/// # Returns
///
/// PathBuf to the created directory
///
/// # Errors
///
/// Returns an error if directory creation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("{}_{}", prefix, uuid::Uuid::new_v4()));

    fs::create_dir_all(&temp_dir).await.with_context(|| {
        format!(
            "Failed to create temporary directory: {}",
            temp_dir.display()
        )
    })?;

    Ok(temp_dir)
}
