// Temporary directory RAII wrapper - included by temp_file.rs

impl TempDir {
    /// Create a new temporary directory with a given prefix
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefix for the directory name
    ///
    /// # Returns
    ///
    /// A new TempDir instance
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    pub async fn new(prefix: &str) -> Result<Self> {
        let path = create_temp_dir(prefix).await?;

        Ok(Self {
            path,
            cleaned_up: false,
        })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Clean up the temporary directory and all its contents
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails
    pub async fn cleanup(&mut self) -> Result<()> {
        if self.cleaned_up {
            return Ok(());
        }

        if fs::try_exists(&self.path).await.unwrap_or(false) {
            fs::remove_dir_all(&self.path).await.with_context(|| {
                format!(
                    "Failed to remove temporary directory: {}",
                    self.path.display()
                )
            })?;
        }

        self.cleaned_up = true;
        Ok(())
    }

    /// Mark the directory as cleaned up without actually cleaning it up
    pub fn mark_cleaned_up(&mut self) {
        self.cleaned_up = true;
    }

    /// Create a path inside this temporary directory
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the file or directory to create
    ///
    /// # Returns
    ///
    /// PathBuf to the created path
    pub fn child<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if self.cleaned_up {
            return;
        }

        // Use blocking FS to ensure cleanup even during runtime shutdown
        if std::path::Path::new(&self.path).exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.path) {
                eprintln!("Error removing temporary directory in drop: {}", e);
            }
        }
    }
}
