//! Temporary file management for distributed mutation testing
//!
//! Provides RAII-based temporary file handling for distributed mutation
//! testing to ensure proper cleanup, even in error cases.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// RAII wrapper for temporary files in distributed mutation testing
///
/// Ensures temporary files are cleaned up when they go out of scope,
/// even in the case of errors or panics.
pub struct WorkerTempFile {
    /// Path to the temporary file
    path: PathBuf,
    
    /// Whether the file has been manually cleaned up
    cleaned_up: bool,
    
    /// Whether to use sync cleanup (for Drop)
    use_sync_cleanup: bool,
}

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
    pub fn with_path(path: PathBuf) -> Self {
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
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Mark the file as cleaned up without actually cleaning it up
    ///
    /// This is useful if you want to keep the file around after the
    /// WorkerTempFile is dropped.
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
    pub async fn copy_to(&self, dest: &Path) -> Result<()> {
        fs::copy(&self.path, dest)
            .await
            .with_context(|| {
                format!("Failed to copy temporary file from {} to {}",
                    self.path.display(), dest.display())
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
            eprintln!("Warning: Temporary file not cleaned up: {}", self.path.display());
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
pub async fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("{}_{}", prefix, uuid::Uuid::new_v4()));
    
    fs::create_dir_all(&temp_dir)
        .await
        .with_context(|| format!("Failed to create temporary directory: {}", temp_dir.display()))?;
        
    Ok(temp_dir)
}

/// RAII wrapper for temporary directories
///
/// Ensures temporary directories are cleaned up when they go out of scope,
/// even in the case of errors or panics.
pub struct TempDir {
    /// Path to the temporary directory
    path: PathBuf,
    
    /// Whether the directory has been manually cleaned up
    cleaned_up: bool,
}

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
            fs::remove_dir_all(&self.path)
                .await
                .with_context(|| {
                    format!("Failed to remove temporary directory: {}", self.path.display())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_worker_temp_file_creation() {
        let temp_file = WorkerTempFile::new(1, 2, None);
        assert!(temp_file.path().file_name().unwrap().to_string_lossy().contains("pmat_w1_2.rs"));
    }
    
    #[tokio::test]
    async fn test_worker_temp_file_with_custom_extension() {
        let temp_file = WorkerTempFile::new(1, 2, Some("txt"));
        assert!(temp_file.path().file_name().unwrap().to_string_lossy().contains("pmat_w1_2.txt"));
    }
    
    #[tokio::test]
    async fn test_worker_temp_file_with_path() {
        let path = std::env::temp_dir().join("custom_temp_file.txt");
        let temp_file = WorkerTempFile::with_path(path.clone());
        assert_eq!(temp_file.path(), &path);
    }
    
    #[tokio::test]
    async fn test_worker_temp_file_write_read() -> Result<()> {
        let temp_file = WorkerTempFile::new(3, 4, None);
        
        // Write content
        temp_file.write("test content").await?;
        
        // Verify file exists
        assert!(temp_file.exists().await);
        
        // Read content
        let content = temp_file.read_to_string().await?;
        assert_eq!(content, "test content");
        
        // Cleanup
        let mut temp_file = temp_file;
        temp_file.cleanup().await?;
        
        // Verify file doesn't exist
        assert!(!temp_file.exists().await);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_worker_temp_file_copy_to() -> Result<()> {
        let temp_file = WorkerTempFile::new(5, 6, None);
        temp_file.write("test content").await?;
        
        // Create destination file
        let dest_file = std::env::temp_dir().join("pmat_copy_test.txt");
        temp_file.copy_to(&dest_file).await?;
        
        // Verify destination file exists
        assert!(fs::try_exists(&dest_file).await?);
        
        // Verify content
        let content = fs::read_to_string(&dest_file).await?;
        assert_eq!(content, "test content");
        
        // Clean up
        let mut temp_file = temp_file;
        temp_file.cleanup().await?;
        fs::remove_file(&dest_file).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_temp_dir_creation() -> Result<()> {
        let temp_dir = TempDir::new("pmat_test").await?;
        
        // Verify directory exists
        assert!(fs::try_exists(temp_dir.path()).await?);
        
        // Clean up
        let mut temp_dir = temp_dir;
        temp_dir.cleanup().await?;
        
        // Verify directory doesn't exist
        assert!(!fs::try_exists(temp_dir.path()).await.unwrap_or(false));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_temp_dir_child_paths() -> Result<()> {
        let temp_dir = TempDir::new("pmat_test").await?;
        
        // Create child path
        let child = temp_dir.child("test.txt");
        
        // Write to child
        let mut file = tokio::fs::File::create(&child).await?;
        file.write_all(b"test content").await?;
        file.flush().await?;
        
        // Verify file exists and has correct content
        assert!(fs::try_exists(&child).await?);
        let content = fs::read_to_string(&child).await?;
        assert_eq!(content, "test content");
        
        // Clean up
        let mut temp_dir = temp_dir;
        temp_dir.cleanup().await?;
        
        // Verify directory and file don't exist
        assert!(!fs::try_exists(temp_dir.path()).await.unwrap_or(false));
        assert!(!fs::try_exists(&child).await.unwrap_or(false));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_mark_cleaned_up() -> Result<()> {
        // Test temp file
        let mut temp_file = WorkerTempFile::new(7, 8, None);
        temp_file.write("test content").await?;
        
        // Mark as cleaned up without cleaning
        temp_file.mark_cleaned_up();
        
        // File should still exist
        assert!(fs::try_exists(temp_file.path()).await?);
        
        // After drop, file should still exist since we marked it as cleaned up
        drop(temp_file);
        
        // Clean up manually for test
        let path = std::env::temp_dir().join("pmat_w7_8.rs");
        if fs::try_exists(&path).await? {
            fs::remove_file(&path).await?;
        }
        
        // Test temp dir
        let mut temp_dir = TempDir::new("pmat_test").await?;
        let dir_path = temp_dir.path().to_path_buf();
        
        // Mark as cleaned up without cleaning
        temp_dir.mark_cleaned_up();
        
        // Directory should still exist
        assert!(fs::try_exists(&dir_path).await?);
        
        // After drop, directory should still exist since we marked it as cleaned up
        drop(temp_dir);
        
        // Clean up manually for test
        if fs::try_exists(&dir_path).await? {
            fs::remove_dir_all(&dir_path).await?;
        }
        
        Ok(())
    }
}