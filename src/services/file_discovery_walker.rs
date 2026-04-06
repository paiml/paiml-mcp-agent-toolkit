// File discovery walker: discover_files() and create_walk_builder() methods
// Included by file_discovery.rs - no `use` imports or `#!` inner attributes allowed

impl ProjectFileDiscovery {
    /// Discover all analyzable files in the project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn discover_files(&self) -> Result<Vec<PathBuf>> {
        let start = std::time::Instant::now();
        debug!("Starting file discovery at: {}", self.root.display());

        let mut builder = self.create_walk_builder()?;

        // Apply custom ignore patterns using process-specific ignore file
        if !self.config.custom_ignore_patterns.is_empty() {
            let temp_ignore_file =
                std::env::temp_dir().join(format!("paiml_ignore_{}", std::process::id()));
            if let Ok(mut file) = std::fs::File::create(&temp_ignore_file) {
                use std::io::Write;
                for pattern in &self.config.custom_ignore_patterns {
                    let _ = writeln!(file, "{pattern}");
                }
                let _ = file.flush();
                builder.add_ignore(&temp_ignore_file);
                // Process-specific ignore file is automatically cleaned on exit
            }
        }

        // Apply additional ignore patterns using process-specific ignore file
        if !ADDITIONAL_IGNORE_PATTERNS.is_empty() {
            let temp_ignore_file2 = std::env::temp_dir()
                .join(format!("paiml_additional_ignore_{}", std::process::id()));
            if let Ok(mut file) = std::fs::File::create(&temp_ignore_file2) {
                use std::io::Write;
                for pattern in ADDITIONAL_IGNORE_PATTERNS.iter() {
                    let _ = writeln!(file, "{pattern}");
                }
                let _ = file.flush();
                builder.add_ignore(&temp_ignore_file2);
                // Process-specific ignore file is automatically cleaned on exit
            }
        }

        let walker = builder.build_parallel();
        let mut files = Vec::new();
        let max_files = self.config.max_files.unwrap_or(usize::MAX);

        // Use parallel walker for performance
        let (tx, rx) = crossbeam_channel::unbounded();
        let filter_external = self.config.filter_external_repos;

        walker.run(|| {
            let tx = tx.clone();
            let classifier = self.classifier.clone();

            Box::new(move |result| {
                if let Ok(entry) = result {
                    if Self::should_include_entry(&entry, filter_external, &classifier) {
                        let _ = tx.send(entry.into_path());
                    }
                }

                // Stop if we've found enough files
                if tx.len() >= max_files {
                    return WalkState::Quit;
                }

                WalkState::Continue
            })
        });

        drop(tx); // Close sender

        // Collect results
        while let Ok(path) = rx.recv() {
            files.push(path);
            if files.len() >= max_files {
                debug!("Reached maximum file limit: {}", max_files);
                break;
            }
        }

        let elapsed = start.elapsed();
        debug!(
            "File discovery completed in {:?}. Found {} files",
            elapsed,
            files.len()
        );

        // Sort for deterministic output (using unstable for performance)
        files.sort_unstable();

        Ok(files)
    }

    /// Create the `WalkBuilder` with appropriate configuration
    fn create_walk_builder(&self) -> Result<WalkBuilder> {
        let mut builder = WalkBuilder::new(&self.root);

        // Configure ripgrep-style filtering
        builder
            .standard_filters(true) // Enables .gitignore, .ignore, etc.
            .hidden(!self.config.follow_links) // Skip hidden files unless following links
            .parents(true) // Check parent directories for ignore files
            .ignore(self.config.respect_gitignore)
            .git_ignore(self.config.respect_gitignore)
            .git_global(self.config.respect_gitignore)
            .git_exclude(self.config.respect_gitignore)
            .follow_links(self.config.follow_links)
            .max_depth(self.config.max_depth)
            // UX Fix: Support both .pmatignore (users expect this) AND .paimlignore (legacy)
            .add_custom_ignore_filename(".pmatignore")
            .add_custom_ignore_filename(".paimlignore");

        // Add build artifact filters
        builder.filter_entry(|entry| !Self::is_build_artifact(entry.path()));

        Ok(builder)
    }
}
