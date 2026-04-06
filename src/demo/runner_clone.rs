impl DemoRunner {
    #[cfg(feature = "git-lib")]
    async fn clone_and_prepare(&self, url: &str) -> Result<PathBuf> {
        debug_assert!(!url.is_empty(), "url must not be empty");
        println!("🔄 Cloning repository: {url}");

        // Create a temporary directory for cloning
        let temp_dir = env::temp_dir().join(format!("paiml-demo-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await?;

        // Create git cloner with progress tracking
        let cloner = GitCloner::new(temp_dir.clone()).with_timeout(Duration::from_secs(120)); // 2 minute timeout

        // Monitor progress in background
        let progress_handle = {
            let cloner = cloner.clone();
            tokio::spawn(async move {
                let mut last_stage = String::with_capacity(1024);
                loop {
                    sleep(Duration::from_millis(500)).await;
                    let progress = cloner.get_progress().await;

                    if progress.stage != last_stage {
                        println!("   📦 {}", progress.stage);
                        last_stage = progress.stage.clone();
                    }

                    if progress.total > 0 {
                        let percent =
                            (progress.current as f64 / progress.total as f64 * 100.0) as u32;
                        print!(
                            "\r   ⏳ Progress: {}% ({}/{})",
                            percent, progress.current, progress.total
                        );
                        io::stdout().flush().ok();
                    }
                }
            })
        };

        // Clone the repository
        match cloner.clone_or_update(url).await {
            Ok(cloned) => {
                progress_handle.abort();
                println!("\r   ✅ Clone complete!                                         ");

                if cloned.cached {
                    println!("   📋 Using cached repository");
                }

                Ok(cloned.path)
            }
            Err(e) => {
                progress_handle.abort();
                println!("\r   ❌ Clone failed                                           ");

                // Clean up on failure
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;

                match e {
                    CloneError::Timeout => {
                        Err(anyhow!("Repository clone timed out after 2 minutes"))
                    }
                    CloneError::InvalidUrl(msg) => Err(anyhow!("Invalid GitHub URL: {msg}")),
                    CloneError::GitError(e) => Err(anyhow!("Git error: {e}")),
                    _ => Err(anyhow!("Failed to clone repository: {e}")),
                }
            }
        }
    }

    #[cfg(not(feature = "git-lib"))]
    async fn clone_and_prepare(&self, url: &str) -> Result<PathBuf> {
        debug_assert!(!url.is_empty(), "url must not be empty");
        use std::process::Command;

        println!("🔄 Cloning repository: {url}");

        // Extract repo name from URL
        let repo_name = url
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .unwrap_or("repo")
            .trim_end_matches(".git");

        // Create a temporary directory for cloning
        let temp_dir = env::temp_dir().join(format!("paiml-demo-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await?;

        let clone_path = temp_dir.join(repo_name);
        let clone_path_str = clone_path
            .to_str()
            .ok_or_else(|| anyhow!("Clone path contains invalid UTF-8"))?;

        // Clone using shell git command
        let output = Command::new("git")
            .args(["clone", "--depth", "1", url, clone_path_str])
            .output()
            .map_err(|e| anyhow!("Failed to run git clone: {e}"))?;

        if output.status.success() {
            println!("   ✅ Clone complete!");
            Ok(clone_path)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
            Err(anyhow!("Git clone failed: {}", stderr.trim()))
        }
    }
}
