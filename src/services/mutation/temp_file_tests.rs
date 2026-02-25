// Tests for temp_file module - included by temp_file.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_worker_temp_file_creation() {
        let temp_file = WorkerTempFile::new(1, 2, None);
        assert!(temp_file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("pmat_w1_2.rs"));
    }

    #[tokio::test]
    async fn test_worker_temp_file_with_custom_extension() {
        let temp_file = WorkerTempFile::new(1, 2, Some("txt"));
        assert!(temp_file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("pmat_w1_2.txt"));
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
