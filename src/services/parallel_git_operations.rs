// Parallel git executor operations - included by parallel_git.rs
// NO `use` imports or `#!` inner attributes - shares parent module scope

impl ParallelGitExecutor {
    #[must_use]
    pub fn new(project_root: PathBuf) -> Self {
        debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
        Self::with_config(project_root, ParallelGitConfig::default())
    }

    #[must_use]
    pub fn with_config(project_root: PathBuf, config: ParallelGitConfig) -> Self {
        debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        let cache = Arc::new(RwLock::new(rustc_hash::FxHashMap::default()));

        Self {
            config,
            semaphore,
            cache,
            project_root,
        }
    }

    /// Execute a single git command with caching
    pub async fn execute_command(&self, args: Vec<&str>) -> Result<String> {
        debug_assert!(!args.is_empty(), "args must not be empty");
        // Generate cache key
        let cache_key = format!("git_{}", args.join("_"));

        // Check cache if enabled
        if self.config.enable_caching {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if entry.timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                    debug!("Cache hit for git command: {:?}", args);
                    return Ok(entry.result.clone());
                }
            }
        }

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await?;

        // Execute git command
        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(TemplateError::Io)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git command failed: {error_msg}"));
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        // Cache result if enabled
        if self.config.enable_caching {
            let mut cache = self.cache.write().await;
            cache.insert(
                cache_key,
                CacheEntry {
                    result: result.clone(),
                    timestamp: std::time::Instant::now(),
                },
            );
        }

        Ok(result)
    }

    /// Execute multiple git commands in parallel
    pub async fn execute_batch(&self, commands: Vec<Vec<&str>>) -> Result<Vec<String>> {
        debug_assert!(!commands.is_empty(), "commands must not be empty");
        let futures: Vec<_> = commands
            .into_iter()
            .map(|args| {
                let executor = self.clone();
                async move { executor.execute_command(args).await }
            })
            .collect();

        let results = join_all(futures).await;

        // Collect results, propagating first error
        let mut outputs = Vec::new();
        for result in results {
            outputs.push(result?);
        }

        Ok(outputs)
    }

    /// Get file history for multiple files in parallel
    pub async fn get_file_histories(
        &self,
        files: Vec<PathBuf>,
        max_commits: usize,
    ) -> Result<Vec<(PathBuf, Vec<CommitInfo>)>> {
        let commands: Vec<Vec<String>> = files
            .iter()
            .map(|file| {
                vec![
                    "log".to_string(),
                    "--follow".to_string(),
                    format!("-{}", max_commits),
                    "--pretty=format:%H|%an|%aI|%s".to_string(),
                    "--".to_string(),
                    file.to_str().unwrap_or("").to_string(),
                ]
            })
            .collect();

        let results = self.execute_batch_owned(commands).await?;

        Ok(files
            .into_iter()
            .zip(results)
            .map(|(file, output)| {
                let commits = Self::parse_commit_log(&output);
                (file, commits)
            })
            .collect())
    }

    /// Execute batch with owned strings (helper for complex commands)
    async fn execute_batch_owned(&self, commands: Vec<Vec<String>>) -> Result<Vec<String>> {
        debug_assert!(!commands.is_empty(), "commands must not be empty");
        let futures: Vec<_> = commands
            .into_iter()
            .map(|args| {
                let executor = self.clone();
                async move {
                    let args_refs: Vec<&str> =
                        args.iter().map(std::string::String::as_str).collect();
                    executor.execute_command(args_refs).await
                }
            })
            .collect();

        let results = join_all(futures).await;

        let mut outputs = Vec::new();
        for result in results {
            outputs.push(result?);
        }

        Ok(outputs)
    }

    /// Get blame information for multiple files in parallel
    pub async fn get_file_blames(&self, files: Vec<PathBuf>) -> Result<Vec<(PathBuf, String)>> {
        let commands: Vec<Vec<&str>> = files
            .iter()
            .map(|file| vec!["blame", "--line-porcelain", file.to_str().unwrap_or("")])
            .collect();

        let results = self.execute_batch(commands).await?;

        Ok(files.into_iter().zip(results).collect())
    }

    /// Get diff statistics for multiple file pairs in parallel
    pub async fn get_diff_stats(
        &self,
        file_pairs: Vec<(PathBuf, String, String)>, // (file, from_commit, to_commit)
    ) -> Result<Vec<DiffStats>> {
        let mut owned_args: Vec<Vec<String>> = Vec::new();

        for (file, from, to) in &file_pairs {
            let args = vec![
                "diff".to_string(),
                "--numstat".to_string(),
                format!("{}..{}", from, to),
                "--".to_string(),
                file.to_string_lossy().to_string(),
            ];
            owned_args.push(args);
        }

        let results = self.execute_batch_owned(owned_args).await?;

        Ok(results
            .into_iter()
            .zip(file_pairs)
            .map(|(output, (file, _, _))| Self::parse_diff_stats(&file, &output))
            .collect())
    }

    /// Parse commit log output
    fn parse_commit_log(output: &str) -> Vec<CommitInfo> {
        debug_assert!(!output.is_empty(), "output must not be empty");
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    Some(CommitInfo {
                        hash: parts[0].to_string(),
                        author: parts[1].to_string(),
                        date: parts[2].to_string(),
                        message: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Parse diff stats output
    fn parse_diff_stats(file: &Path, output: &str) -> DiffStats {
        debug_assert!(file.exists(), "file must exist: {}", file.display());
        let mut additions = 0;
        let mut deletions = 0;

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(add), Ok(del)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    additions += add;
                    deletions += del;
                }
            }
        }

        DiffStats {
            file: file.to_path_buf(),
            additions,
            deletions,
        }
    }

    /// Clear the command cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Git command cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        let size = cache.len();
        let memory = size * std::mem::size_of::<(String, CacheEntry)>();
        (size, memory)
    }
}

impl Clone for ParallelGitExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            cache: Arc::clone(&self.cache),
            project_root: self.project_root.clone(),
        }
    }
}
