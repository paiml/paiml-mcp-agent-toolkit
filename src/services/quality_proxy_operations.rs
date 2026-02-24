impl QualityProxyService {
    /// Proxies a code operation through quality gates.
    ///
    /// # Arguments
    ///
    /// * `request` - The proxy request containing operation details
    ///
    /// # Returns
    ///
    /// A proxy response with quality report and final content
    ///
    /// # Example
    ///
    /// ```
    /// use pmat::services::quality_proxy::QualityProxyService;
    /// use pmat::models::proxy::{ProxyRequest, ProxyOperation, ProxyMode, QualityConfig};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let service = QualityProxyService::new();
    /// let request = ProxyRequest {
    ///     operation: ProxyOperation::Write,
    ///     file_path: "example.rs".to_string(),
    ///     content: Some("/// Example function\nfn example() {}".to_string()),
    ///     old_content: None,
    ///     new_content: None,
    ///     mode: ProxyMode::Advisory,
    ///     quality_config: QualityConfig::default(),
    /// };
    ///
    /// let response = service.proxy_operation(request).await?;
    /// println!("Status: {:?}", response.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn proxy_operation(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        info!(
            "Proxying {} operation for {}",
            match request.operation {
                ProxyOperation::Write => "write",
                ProxyOperation::Edit => "edit",
                ProxyOperation::Append => "append",
            },
            request.file_path
        );

        let content = self.get_operation_content(&request)?;
        let file_extension = Path::new(&request.file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("rs");

        let ((quality_metrics, passed), violations) = self
            .analyze_content(
                &content,
                &request.file_path,
                file_extension,
                &request.quality_config,
            )
            .await?;

        let (status, final_content, refactoring_applied, refactoring_plan) = match request.mode {
            ProxyMode::Strict => {
                if passed {
                    (ProxyStatus::Accepted, content, false, None)
                } else {
                    (ProxyStatus::Rejected, String::new(), false, None)
                }
            }
            ProxyMode::Advisory => (ProxyStatus::Accepted, content, false, None),
            ProxyMode::AutoFix => {
                if passed {
                    (ProxyStatus::Accepted, content, false, None)
                } else {
                    match self
                        .auto_fix_content(
                            &content,
                            &request.file_path,
                            file_extension,
                            &request.quality_config,
                        )
                        .await
                    {
                        Ok((fixed_content, plan)) => {
                            let ((_, fixed_passed), _) = self
                                .analyze_content(
                                    &fixed_content,
                                    &request.file_path,
                                    file_extension,
                                    &request.quality_config,
                                )
                                .await?;

                            if fixed_passed {
                                (ProxyStatus::Modified, fixed_content, true, Some(plan))
                            } else {
                                warn!("Auto-fix failed to meet quality standards");
                                (ProxyStatus::Rejected, String::new(), false, None)
                            }
                        }
                        Err(e) => {
                            warn!("Auto-fix failed: {}", e);
                            (ProxyStatus::Rejected, String::new(), false, None)
                        }
                    }
                }
            }
        };

        Ok(ProxyResponse {
            status,
            quality_report: QualityReport {
                passed,
                metrics: quality_metrics,
                violations,
            },
            final_content,
            refactoring_applied,
            refactoring_plan,
        })
    }

    fn get_operation_content(&self, request: &ProxyRequest) -> Result<String> {
        match request.operation {
            ProxyOperation::Write => request
                .content
                .clone()
                .context("Write operation requires content"),
            ProxyOperation::Edit => {
                let old = request
                    .old_content
                    .as_ref()
                    .context("Edit operation requires old_content")?;
                let new = request
                    .new_content
                    .as_ref()
                    .context("Edit operation requires new_content")?;

                if let Some(existing_content) = &request.content {
                    Ok(existing_content.replace(old, new))
                } else {
                    Ok(new.clone())
                }
            }
            ProxyOperation::Append => {
                let append_content = request
                    .content
                    .as_ref()
                    .context("Append operation requires content")?;

                if let Some(existing) = &request.old_content {
                    Ok(format!("{existing}\n{append_content}"))
                } else {
                    Ok(append_content.clone())
                }
            }
        }
    }
}
