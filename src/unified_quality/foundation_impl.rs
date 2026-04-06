impl QualityMonitor {
    /// Create a new quality monitor
    pub fn new(config: MonitorConfig) -> Result<Self> {
        let (tx, _rx) = crossbeam_channel::bounded(1000);

        Ok(Self {
            watcher: Arc::new(ParkingLotRwLock::new(None)),
            parser: Arc::new(std::sync::Mutex::new(EnhancedParser::new())),
            metrics: Arc::new(dashmap::DashMap::new()),
            events: tx,
            config,
        })
    }

    /// Start monitoring a directory
    #[cfg(feature = "watch")]
    pub async fn start_monitoring(&mut self, path: PathBuf) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        info!("Starting quality monitoring for: {:?}", path);

        // Create file system watcher
        let events = self.events.clone();
        let metrics = self.metrics.clone();
        let parser = self.parser.clone();
        let config = self.config.clone();

        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<Event>| {
                if let Ok(event) = result {
                    Self::handle_fs_event(event, &events, &metrics, &parser, &config);
                }
            },
            Config::default(),
        )?;

        // Start watching the directory
        watcher.watch(&path, RecursiveMode::Recursive)?;

        // Store watcher
        {
            let mut guard = self.watcher.write();
            *guard = Some(watcher);
        }

        // Perform initial analysis
        self.analyze_directory(&path).await?;

        Ok(())
    }

    /// Start monitoring a directory (stub when watch feature disabled)
    #[cfg(not(feature = "watch"))]
    pub async fn start_monitoring(&mut self, path: PathBuf) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        info!(
            "Starting quality monitoring for: {:?} (watch disabled)",
            path
        );
        self.analyze_directory(&path).await?;
        Ok(())
    }

    /// Analyze incremental changes with O(log n) complexity
    pub fn analyze_incremental(&self, change: FileChange) -> Result<Metrics> {
        // Use real tree-sitter incremental parsing
        let mut parser = self.parser.lock().expect("internal error");
        parser.parse_incremental(&change.path, &change.content)
    }

    /// Get current metrics for a file
    #[must_use]
    pub fn get_metrics(&self, path: &Path) -> Option<Metrics> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.metrics.get(path).map(|entry| entry.clone())
    }

    /// Get all metrics
    #[must_use]
    pub fn get_all_metrics(&self) -> HashMap<PathBuf, Metrics> {
        self.metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Subscribe to quality events
    #[must_use]
    pub fn subscribe(&self) -> crossbeam_channel::Receiver<QualityEvent> {
        let (_tx, rx) = crossbeam_channel::bounded(100);
        rx
    }

    /// Handle file system events
    #[cfg(feature = "watch")]
    fn handle_fs_event(
        event: Event,
        events: &crossbeam_channel::Sender<QualityEvent>,
        metrics: &Arc<dashmap::DashMap<PathBuf, Metrics>>,
        parser: &Arc<std::sync::Mutex<EnhancedParser>>,
        config: &MonitorConfig,
    ) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    Self::handle_file_changed(path, events, metrics, parser, &config.watch_patterns);
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    Self::handle_file_removed(path, events, metrics);
                }
            }
            _ => {}
        }
    }

    /// Process a single file create/modify event
    #[cfg(feature = "watch")]
    fn handle_file_changed(
        path: PathBuf,
        events: &crossbeam_channel::Sender<QualityEvent>,
        metrics: &Arc<dashmap::DashMap<PathBuf, Metrics>>,
        parser: &Arc<std::sync::Mutex<EnhancedParser>>,
        watch_patterns: &[String],
    ) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if !Self::should_analyze(&path, watch_patterns) {
            return;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };

        let _change = FileChange {
            path: path.clone(),
            content: content.clone(),
            old_tree: None,
            timestamp: SystemTime::now(),
        };

        let Ok(mut parser_lock) = parser.lock() else {
            return;
        };
        let Ok(new_metrics) = parser_lock.parse_incremental(&path, &content) else {
            return;
        };

        let old_metrics = metrics.insert(path.clone(), new_metrics.clone());
        let event = Self::build_quality_event(path, old_metrics, new_metrics);
        let _ = events.try_send(event);
    }

    /// Process a single file removal event
    #[cfg(feature = "watch")]
    fn handle_file_removed(
        path: PathBuf,
        events: &crossbeam_channel::Sender<QualityEvent>,
        metrics: &Arc<dashmap::DashMap<PathBuf, Metrics>>,
    ) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some((_, last_metrics)) = metrics.remove(&path) {
            let _ = events.try_send(QualityEvent::FileRemoved {
                path,
                last_metrics,
            });
        }
    }

    /// Build the appropriate quality event based on whether metrics existed before
    #[cfg(feature = "watch")]
    fn build_quality_event(
        path: PathBuf,
        old_metrics: Option<Metrics>,
        new_metrics: Metrics,
    ) -> QualityEvent {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        match old_metrics {
            Some(old) => QualityEvent::MetricsUpdated {
                path,
                old_metrics: old,
                new_metrics,
            },
            None => QualityEvent::FileAdded {
                path,
                metrics: new_metrics,
            },
        }
    }

    /// Check if file should be analyzed
    fn should_analyze(path: &Path, patterns: &[String]) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let path_str = path.to_string_lossy();
        patterns.iter().any(|pattern| {
            if pattern.contains("**") {
                let ext = pattern.strip_prefix("**/").unwrap_or(pattern);
                path_str.ends_with(ext.strip_prefix("*").unwrap_or(ext))
            } else {
                path_str.contains(pattern)
            }
        })
    }

    /// Analyze entire directory
    async fn analyze_directory(&self, path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        use walkdir::WalkDir;

        let mut batch = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.is_file() && Self::should_analyze(path, &self.config.watch_patterns) {
                batch.push(path.to_path_buf());

                if batch.len() >= self.config.max_batch_size {
                    self.analyze_batch(&batch).await?;
                    batch.clear();
                }
            }
        }

        if !batch.is_empty() {
            self.analyze_batch(&batch).await?;
        }

        Ok(())
    }

    /// Analyze a batch of files
    async fn analyze_batch(&self, paths: &[PathBuf]) -> Result<()> {
        // use rayon::prelude::*; // Currently unused

        let results: Vec<_> = paths
            .iter()
            .filter_map(|path| {
                std::fs::read_to_string(path).ok().map(|content| {
                    if let Ok(mut parser) = self.parser.lock() {
                        (path.clone(), parser.parse_incremental(path, &content))
                    } else {
                        (path.clone(), Err(anyhow::anyhow!("Failed to lock parser")))
                    }
                })
            })
            .collect();

        for (path, result) in results {
            if let Ok(metrics) = result {
                self.metrics.insert(path, metrics);
            }
        }

        Ok(())
    }
}
