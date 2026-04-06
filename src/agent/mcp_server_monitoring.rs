// MCP Server Monitoring Handlers - included by mcp_server.rs
// Contains: start/stop/status monitoring request handlers.

impl ClaudeCodeAgentMcpServer {
    /// Handle start monitoring request
    async fn handle_start_monitoring(&mut self, params: &Value) -> Result<Value> {
        let project_path = params["project_path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("project_path parameter required"))?;

        let path = PathBuf::from(project_path);
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Project path does not exist: {project_path}"
            ));
        }

        let project_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let watch_patterns = params["watch_patterns"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| self.config.watch_patterns.clone());

        let complexity_threshold = params["complexity_threshold"]
            .as_u64()
            .unwrap_or(u64::from(self.config.complexity_threshold))
            as u32;

        info!(
            "Starting quality monitoring for project: {} at {}",
            project_name, project_path
        );

        let project = MonitoredProject {
            path: path.clone(),
            name: project_name.clone(),
            watch_patterns: watch_patterns.clone(),
            complexity_threshold,
            last_analysis: None,
            started_at: std::time::SystemTime::now(),
        };

        self.monitored_projects
            .insert(project_name.clone(), project);

        if let Some(ref monitor) = self.quality_monitor {
            let command = QualityMonitorCommand::StartMonitoring {
                project_path: path,
                config: Box::new(self.monitored_projects[&project_name].clone()),
            };
            let _ = monitor.send(command).await;
        }

        Ok(json!({
            "type": "text",
            "text": format!("Started quality monitoring for project '{}'\nPath: {}\nComplexity threshold: {}\nWatch patterns: {:?}",
                project_name, project_path, complexity_threshold, watch_patterns)
        }))
    }

    /// Handle stop monitoring request
    async fn handle_stop_monitoring(&mut self, params: &Value) -> Result<Value> {
        let project_id = params["project_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("project_id parameter required"))?;

        info!("Stopping quality monitoring for project: {}", project_id);
        self.monitored_projects.remove(project_id);

        if let Some(ref monitor) = self.quality_monitor {
            let command = QualityMonitorCommand::StopMonitoring {
                project_id: project_id.to_string(),
            };
            let _ = monitor.send(command).await;
        }

        Ok(json!({
            "type": "text",
            "text": format!("Stopped quality monitoring for project: {}", project_id)
        }))
    }

    /// Handle get status request
    async fn handle_get_status(&self, params: &Value) -> Result<Value> {
        let project_id = params["project_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("project_id parameter required"))?;

        let project = self.monitored_projects.get(project_id)
            .ok_or_else(|| anyhow::anyhow!("Project '{project_id}' is not being monitored"))?;

        // Try to get real status from quality monitor channel
        if let Some(ref monitor) = self.quality_monitor {
            if let Some(status) = request_monitor_status(monitor, project_id).await {
                return Ok(format_status_response(project_id, &status));
            }
        }

        // Fall back to stored project info
        if let Some(last_analysis) = &project.last_analysis {
            return Ok(format_status_response(project_id, last_analysis));
        }

        let pending_status = ProjectAnalysisResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            quality_score: 0.0,
            files_analyzed: 0,
            functions_analyzed: 0,
            avg_complexity: 0.0,
            hotspot_functions: 0,
            satd_issues: 0,
            quality_gate_status: "PENDING".to_string(),
            recommendations: vec!["Analysis pending...".to_string()],
        };

        Ok(format_status_response(project_id, &pending_status))
    }
}

async fn request_monitor_status(
    monitor: &mpsc::Sender<QualityMonitorCommand>,
    project_id: &str,
) -> Option<ProjectAnalysisResult> {
    let (tx, rx) = oneshot::channel();
    let command = QualityMonitorCommand::GetStatus {
        project_id: project_id.to_string(),
        response_tx: Box::new(tx),
    };
    if monitor.send(command).await.is_ok() {
        if let Ok(Some(status)) = rx.await {
            return Some(status);
        }
    }
    None
}

fn format_status_response(project_id: &str, status: &ProjectAnalysisResult) -> Value {
    json!({
        "type": "text",
        "text": format!("Quality Status for {}: {}", project_id,
            serde_json::to_string_pretty(status).unwrap_or_default())
    })
}
