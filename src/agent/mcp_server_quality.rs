// MCP Server Quality Gates & Monitor - included by mcp_server.rs

fn format_quality_json(target_path: &str, all_passed: bool, quality_result: &QualityGateOutput) -> Value {
    debug_assert!(!target_path.is_empty(), "target_path must not be empty");
    let checks_status: HashMap<String, String> = quality_result
        .results
        .iter()
        .map(|r| (r.check.clone(), if r.passed { "PASSED" } else { "FAILED" }.to_string()))
        .collect();

    json!({
        "type": "text",
        "text": json!({
            "status": if all_passed { "PASSED" } else { "FAILED" },
            "target": target_path,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "checks": checks_status,
            "summary": quality_result.summary,
            "failures": quality_result.results.iter()
                .filter(|r| !r.passed)
                .map(|r| format!("{}: {}", r.check, r.message))
                .collect::<Vec<_>>()
        }).to_string()
    })
}

fn format_quality_markdown(target_path: &str, all_passed: bool, quality_result: &QualityGateOutput) -> Value {
    debug_assert!(!target_path.is_empty(), "target_path must not be empty");
    let status_icon = if all_passed { "✅" } else { "❌" };
    let status_text = if all_passed { "PASSED" } else { "FAILED" };
    let mut text = format!("# Quality Gates Report\n\n**Target**: {target_path}\n");
    text.push_str(&format!("**Status**: {status_icon} {status_text}\n"));
    text.push_str(&format!("**Timestamp**: {}\n\n## Checks\n", chrono::Utc::now().to_rfc3339()));

    for result in &quality_result.results {
        let icon = if result.passed { "✅" } else { "❌" };
        let msg = if result.passed { "PASSED" } else { &result.message };
        text.push_str(&format!("- {icon} {}: {msg}\n", result.check));
    }

    text.push_str(&format!(
        "\n**Summary**: {} of {} checks passed",
        quality_result.summary.passed_checks, quality_result.summary.total_checks
    ));

    json!({ "type": "text", "text": text })
}

fn format_quality_claude(target_path: &str, all_passed: bool, quality_result: &QualityGateOutput) -> Value {
    debug_assert!(!target_path.is_empty(), "target_path must not be empty");
    let status_icon = if all_passed { "✅" } else { "❌" };
    let status_text = if all_passed { "PASSED" } else { "FAILED" };
    let mut text = format!("🎯 Quality Gates Report for {target_path}\n\nStatus: {status_icon} {status_text}\n");

    if all_passed {
        text.push_str("All Toyota Way standards met!\n\n");
    } else {
        text.push_str("Quality issues detected!\n\n");
    }

    text.push_str("Checks completed:\n");
    for result in &quality_result.results {
        let icon = if result.passed { "✅" } else { "❌" };
        let msg = if result.passed { "PASSED" } else { &result.message };
        text.push_str(&format!("• {}: {icon} {msg}\n", result.check));
    }

    if all_passed {
        text.push_str("\nThe codebase meets all quality standards. Great work! 🚀");
    } else {
        text.push_str("\n⚠️ Please address the quality issues above.");
    }

    json!({ "type": "text", "text": text })
}

impl ClaudeCodeAgentMcpServer {
    /// Handle quality gates request
    #[allow(dead_code)]
    async fn handle_quality_gates(&self, params: &Value) -> Result<Value> {
        let target_path = params["target_path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("target_path parameter required"))?;

        let output_format = params["output_format"]
            .as_str()
            .unwrap_or("claude-friendly");

        info!("Running quality gates on: {}", target_path);

        let path = PathBuf::from(target_path);
        let input = QualityGateInput {
            path: path.clone(),
            checks: vec![
                QualityCheck::Complexity { max: 20 },
                QualityCheck::Satd { tolerance: 0 },
                QualityCheck::DeadCode { max_percentage: 10.0 },
                QualityCheck::Lint,
            ],
            strict: true,
        };

        let quality_result = self.quality_gate_service.process(input).await?;
        let all_passed = quality_result.results.iter().all(|r| r.passed);

        let result = match output_format {
            "json" => format_quality_json(target_path, all_passed, &quality_result),
            "markdown" => format_quality_markdown(target_path, all_passed, &quality_result),
            _ => format_quality_claude(target_path, all_passed, &quality_result),
        };

        Ok(result)
    }

    /// Run quality monitoring background task
    async fn run_quality_monitor(
        mut self,
        mut rx: mpsc::Receiver<QualityMonitorCommand>,
    ) -> Result<()> {
        info!("Starting quality monitoring background task");

        let mut monitoring_tasks: HashMap<String, tokio::task::JoinHandle<()>> = HashMap::new();

        while let Some(command) = rx.recv().await {
            match command {
                QualityMonitorCommand::StartMonitoring {
                    project_path,
                    config,
                } => {
                    info!("Monitor: Starting monitoring for {:?}", project_path);
                    self.monitored_projects
                        .insert(config.name.clone(), (*config).clone());

                    let project_id = config.name.clone();
                    let path = project_path.clone();
                    let analysis_service = self.analysis_service.clone();
                    let _quality_gate_service = self.quality_gate_service.clone();

                    let task = tokio::spawn(async move {
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(60));
                        loop {
                            interval.tick().await;
                            let input = AnalysisInput {
                                operation: AnalysisOperation::All,
                                path: path.clone(),
                                options: AnalysisOptions::default(),
                            };
                            if let Ok(_result) = analysis_service.process(input).await {
                                info!("Monitor: Analysis completed for {}", project_id);
                            }
                        }
                    });

                    monitoring_tasks.insert(config.name.clone(), task);
                }
                QualityMonitorCommand::StopMonitoring { project_id } => {
                    info!("Monitor: Stopping monitoring for {}", project_id);
                    self.monitored_projects.remove(&project_id);
                    if let Some(task) = monitoring_tasks.remove(&project_id) {
                        task.abort();
                    }
                }
                QualityMonitorCommand::GetStatus {
                    project_id,
                    response_tx,
                } => {
                    debug!("Monitor: Getting status for {}", project_id);
                    let result = self
                        .monitored_projects
                        .get(&project_id)
                        .and_then(|p| p.last_analysis.clone());
                    let _ = (*response_tx).send(result);
                }
                QualityMonitorCommand::Shutdown => {
                    info!("Monitor: Shutting down");
                    for (_id, task) in monitoring_tasks {
                        task.abort();
                    }
                    break;
                }
            }
        }

        Ok(())
    }
}
