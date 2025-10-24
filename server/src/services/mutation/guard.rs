//! RAII-based mutant file restoration guard
//!
//! Provides a safe file backup and restoration mechanism for mutation testing,
//! ensuring that original files are always restored even on error or process
//! termination.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// RAII guard that ensures a file is restored on scope exit
///
/// Uses a combination of async operations for normal execution
/// and sync operations in Drop to ensure restoration even on panic.
pub struct MutantGuard {
    /// Original file path that was modified
    original_path: PathBuf,
    
    /// Path to the backup file
    backup_path: PathBuf,
    
    /// Whether backup has already been restored
    restored: bool,
}

impl MutantGuard {
    /// Create a new guard by backing up a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file that will be modified
    ///
    /// # Returns
    ///
    /// A new guard instance that will restore the file when dropped
    ///
    /// # Errors
    ///
    /// Returns an error if the backup cannot be created
    pub async fn new(path: &Path) -> Result<Self> {
        // Create backup path with a unique suffix
        let backup_path = path.with_extension(format!(
            "pmat_backup_{}", 
            std::process::id()
        ));
        
        // Create backup of the original file
        fs::copy(path, &backup_path)
            .await
            .with_context(|| format!("Failed to create backup of {}", path.display()))?;
        
        Ok(Self {
            original_path: path.to_path_buf(),
            backup_path,
            restored: false,
        })
    }
    
    /// Restore the file from backup
    ///
    /// # Returns
    ///
    /// Ok(()) if the restoration was successful, or an error otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the restoration fails
    pub async fn restore(&mut self) -> Result<()> {
        if self.restored {
            return Ok(());
        }
        
        // Copy backup back to original file
        fs::copy(&self.backup_path, &self.original_path)
            .await
            .with_context(|| "Failed to restore file from backup".to_string())?;
            
        // Remove backup file
        fs::remove_file(&self.backup_path)
            .await
            .with_context(|| "Failed to remove backup file".to_string())?;
            
        self.restored = true;
        Ok(())
    }
    
    /// Mark this guard as restored without actually restoring
    ///
    /// This is useful when another mechanism has already restored the file
    /// and we don't want the Drop handler to attempt restoration again.
    pub fn mark_restored(&mut self) {
        self.restored = true;
    }
    
    /// Get the path to the backup file
    pub fn backup_path(&self) -> &Path {
        &self.backup_path
    }
    
    /// Get the path to the original file
    pub fn original_path(&self) -> &Path {
        &self.original_path
    }
}

impl Drop for MutantGuard {
    fn drop(&mut self) {
        // Skip if already restored
        if self.restored {
            return;
        }
        
        // Use blocking operations to ensure files are restored
        // even in the case of tokio runtime shutdown
        if let Err(e) = std::fs::copy(&self.backup_path, &self.original_path) {
            eprintln!("ERROR: Failed to restore file in MutantGuard::drop: {}", e);
        }
        
        if let Err(e) = std::fs::remove_file(&self.backup_path) {
            eprintln!("ERROR: Failed to remove backup file in MutantGuard::drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    // No external imports needed
    
    #[tokio::test]
    async fn test_guard_creation_and_restoration() -> Result<()> {
        // Create a temp file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_owned();
        
        // Write initial content
        fs::write(&temp_path, "initial content")?;
        
        // Create guard
        let mut guard = MutantGuard::new(&temp_path).await?;
        
        // Verify backup exists
        assert!(guard.backup_path().exists());
        assert_eq!(fs::read_to_string(guard.backup_path())?, "initial content");
        
        // Modify the file
        fs::write(&temp_path, "modified content")?;
        assert_eq!(fs::read_to_string(&temp_path)?, "modified content");
        
        // Restore the file
        guard.restore().await?;
        
        // Verify content was restored
        assert_eq!(fs::read_to_string(&temp_path)?, "initial content");
        
        // Verify backup was deleted
        assert!(!guard.backup_path().exists());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_guard_drop_restores_file() -> Result<()> {
        // Create a temp file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_owned();
        
        // Write initial content
        fs::write(&temp_path, "initial content")?;
        
        // Use a block to force drop
        {
            // Create guard
            let _guard = MutantGuard::new(&temp_path).await?;
            
            // Modify the file
            fs::write(&temp_path, "modified by test")?;
            assert_eq!(fs::read_to_string(&temp_path)?, "modified by test");
            
            // Guard is dropped here
        }
        
        // Verify content was restored by Drop
        assert_eq!(fs::read_to_string(&temp_path)?, "initial content");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_mark_restored_prevents_restoration() -> Result<()> {
        // Create a temp file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_owned();
        
        // Write initial content
        fs::write(&temp_path, "initial content")?;
        
        // Create guard
        let mut guard = MutantGuard::new(&temp_path).await?;
        
        // Modify the file
        fs::write(&temp_path, "modified by test")?;
        
        // Mark as restored (without actually restoring)
        guard.mark_restored();
        
        // Guard is dropped here, but shouldn't restore
        drop(guard);
        
        // Verify content was NOT restored
        assert_eq!(fs::read_to_string(&temp_path)?, "modified by test");
        
        Ok(())
    }
}