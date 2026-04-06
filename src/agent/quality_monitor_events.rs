// File system event handling for QualityMonitorEngine
// Included by quality_monitor.rs - shares parent module scope

impl QualityMonitorEngine {
    /// Handle file system events
    async fn handle_file_system_event(
        project_id: &str,
        event: Event,
        project_path: &Path,
        config: &QualityMonitorConfig,
        metrics: &Arc<RwLock<HashMap<String, QualityMetrics>>>,
        event_sender: &Option<mpsc::Sender<QualityEvent>>,
    ) -> Result<()> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        debug_assert!(!project_id.is_empty(), "project_id must not be empty");
        debug!("File system event: {:?}", event);

        // Filter relevant events
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                // Process changed files
                for path in &event.paths {
                    if let Ok(relative_path) = path.strip_prefix(project_path) {
                        if Self::should_analyze_file(relative_path, &config.watch_patterns) {
                            info!("File changed: {:?}, triggering analysis", relative_path);

                            // Debounce: wait a bit to avoid excessive analysis
                            tokio::time::sleep(config.debounce_interval).await;

                            // Perform incremental analysis for the changed file
                            if let Err(e) = Self::analyze_changed_file(
                                project_id,
                                path,
                                relative_path,
                                &event.kind,
                                metrics,
                                event_sender,
                            )
                            .await
                            {
                                error!("Failed to analyze changed file {:?}: {}", relative_path, e);
                            }
                        }
                    }
                }
            }
            _ => {
                // Ignore other event types
            }
        }

        Ok(())
    }

    /// Analyze a changed file and update metrics
    async fn analyze_changed_file(
        project_id: &str,
        file_path: &PathBuf,
        relative_path: &Path,
        event_kind: &EventKind,
        metrics: &Arc<RwLock<HashMap<String, QualityMetrics>>>,
        event_sender: &Option<mpsc::Sender<QualityEvent>>,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(relative_path.exists(), "relative_path must exist: {}", relative_path.display());
        debug_assert!(!project_id.is_empty(), "project_id must not be empty");
        info!(
            "Analyzing changed file: {:?} (event: {:?})",
            relative_path, event_kind
        );

        // Get current file metrics
        let file_metrics = Self::analyze_file_metrics(file_path, relative_path).await?;

        // Update project metrics
        {
            let mut metrics_map = metrics.write().await;
            if let Some(project_metrics) = metrics_map.get_mut(project_id) {
                let file_path_str = relative_path.to_string_lossy().to_string();

                // Check if this is a new file or updated file
                let is_new_file = !project_metrics.file_metrics.contains_key(&file_path_str);
                let old_metrics = project_metrics.file_metrics.get(&file_path_str).cloned();

                // Update file metrics
                project_metrics
                    .file_metrics
                    .insert(file_path_str.clone(), file_metrics.clone());
                project_metrics.last_updated = SystemTime::now();

                // Update aggregate metrics
                Self::update_aggregate_metrics(project_metrics);

                // Send event notification
                if let Some(sender) = event_sender {
                    let event = if is_new_file {
                        QualityEvent::FileAnalyzed {
                            project_id: project_id.to_string(),
                            file_path: file_path_str,
                            metrics: file_metrics,
                        }
                    } else if let Some(old) = old_metrics {
                        // Check for quality changes
                        let changes =
                            Self::detect_quality_changes(&old, &file_metrics, &file_path_str);
                        QualityEvent::MetricsUpdated {
                            project_id: project_id.to_string(),
                            metrics: project_metrics.clone(),
                            changes,
                        }
                    } else {
                        QualityEvent::FileAnalyzed {
                            project_id: project_id.to_string(),
                            file_path: file_path_str,
                            metrics: file_metrics,
                        }
                    };

                    if let Err(e) = sender.try_send(event) {
                        warn!("Failed to send quality event: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a file should be analyzed based on patterns
    fn should_analyze_file(file_path: &Path, patterns: &[String]) -> bool {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let file_str = file_path.to_string_lossy();

        // Check if file matches any watch pattern
        for pattern in patterns {
            if pattern.contains("**") {
                // Simple glob pattern matching
                let extension = pattern.strip_prefix("**/").unwrap_or(pattern);
                if file_str.ends_with(extension.strip_prefix("*").unwrap_or(extension)) {
                    return true;
                }
            } else if file_str.contains(pattern) {
                return true;
            }
        }

        false
    }
}
