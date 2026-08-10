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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    /// Resolve the post-operation content the quality gates will judge.
    ///
    /// `request.file_path` used to be ignored entirely: with no inline `content`
    /// an Edit fell through to the replacement fragment alone and an Append to
    /// the appended text alone, so a three-line file came back with
    /// `final_content` holding one line and was graded on that fragment. Worse,
    /// nothing checked that `old_content` actually occurred anywhere, so an edit
    /// anchored to a string absent from the file was reported "accepted". Fall
    /// back to the file on disk, and refuse an anchor that occurs nowhere in it.
    /// (An anchor that occurs several times still replaces every occurrence —
    /// that is the semantic the proxy's property tests pin.)
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

                let existing = match &request.content {
                    Some(inline) => inline.clone(),
                    None => read_proxy_target(&request.file_path)?.with_context(|| {
                        format!(
                            "Edit operation on {} requires the file to exist or inline content",
                            request.file_path
                        )
                    })?,
                };

                if !existing.contains(old.as_str()) {
                    anyhow::bail!(
                        "Edit rejected: old_content does not occur in {}",
                        request.file_path
                    );
                }
                Ok(existing.replace(old, new))
            }
            ProxyOperation::Append => {
                let append_content = request
                    .content
                    .as_ref()
                    .context("Append operation requires content")?;

                match &request.old_content {
                    // Caller-supplied preceding text keeps its historical join.
                    Some(existing) => Ok(format!("{existing}\n{append_content}")),
                    None => match read_proxy_target(&request.file_path)? {
                        Some(existing) => Ok(join_appended(&existing, append_content)),
                        // Appending to a file that does not exist yet creates it.
                        None => Ok(append_content.clone()),
                    },
                }
            }
        }
    }
}

/// Read the file an operation targets; `None` when it does not exist yet.
fn read_proxy_target(file_path: &str) -> Result<Option<String>> {
    let path = Path::new(file_path);
    if !path.is_file() {
        return Ok(None);
    }
    std::fs::read_to_string(path)
        .map(Some)
        .with_context(|| format!("Failed to read {file_path} for quality proxy"))
}

/// Concatenate appended text without inventing or losing a line break.
fn join_appended(existing: &str, addition: &str) -> String {
    if existing.is_empty() || existing.ends_with('\n') {
        format!("{existing}{addition}")
    } else {
        format!("{existing}\n{addition}")
    }
}

#[cfg(test)]
mod proxy_file_path_tests {
    use super::*;

    const THREE_LINES: &str = "pub fn line_one() -> i32 { 1 }\n\
                               pub fn line_two() -> i32 { 2 }\n\
                               pub fn line_three() -> i32 { 3 }\n";

    fn request(op: ProxyOperation, path: &std::path::Path) -> ProxyRequest {
        ProxyRequest {
            operation: op,
            file_path: path.display().to_string(),
            content: None,
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        }
    }

    fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("multi.rs");
        std::fs::write(&path, THREE_LINES).expect("write fixture");
        (dir, path)
    }

    /// An edit must be graded on the whole file, not on the replacement alone.
    #[test]
    fn test_edit_applies_to_the_file_on_disk() {
        let (_dir, path) = fixture();
        let service = QualityProxyService::new();
        let mut req = request(ProxyOperation::Edit, &path);
        req.old_content = Some("pub fn line_two() -> i32 { 2 }".to_string());
        req.new_content = Some("pub fn line_two() -> i32 { 22 }".to_string());

        let content = service.get_operation_content(&req).expect("edit resolves");
        assert!(content.contains("line_one"), "{content}");
        assert!(content.contains("line_three"), "{content}");
        assert!(content.contains("{ 22 }"), "{content}");
        assert!(!content.contains("{ 2 }"), "{content}");
    }

    /// An anchor that occurs nowhere in the file is not an edit — it is an error.
    #[test]
    fn test_edit_with_absent_old_content_is_rejected() {
        let (_dir, path) = fixture();
        let service = QualityProxyService::new();
        let mut req = request(ProxyOperation::Edit, &path);
        req.old_content = Some("THIS STRING IS NOT PRESENT".to_string());
        req.new_content = Some("zzz".to_string());

        let err = service
            .get_operation_content(&req)
            .expect_err("an impossible edit must not be accepted");
        assert!(err.to_string().contains("does not occur"), "{err}");
    }

    /// A repeated anchor replaces every occurrence — in the file, not in a
    /// fragment (the proxy's property tests pin the replace-all semantic).
    #[test]
    fn test_edit_replaces_every_occurrence_in_the_file() {
        let (_dir, path) = fixture();
        let service = QualityProxyService::new();
        let mut req = request(ProxyOperation::Edit, &path);
        req.old_content = Some("-> i32".to_string());
        req.new_content = Some("-> u32".to_string());

        let content = service.get_operation_content(&req).expect("edit resolves");
        assert_eq!(content.matches("-> u32").count(), 3, "{content}");
        assert!(!content.contains("-> i32"), "{content}");
    }

    /// Append must keep the file it appends to.
    #[test]
    fn test_append_keeps_the_existing_file() {
        let (_dir, path) = fixture();
        let service = QualityProxyService::new();
        let mut req = request(ProxyOperation::Append, &path);
        req.content = Some("pub fn line_four() -> i32 { 4 }\n".to_string());

        let content = service.get_operation_content(&req).expect("append resolves");
        assert!(content.starts_with(THREE_LINES), "{content}");
        assert!(content.contains("line_four"), "{content}");
    }

    /// Appending to a path that does not exist yet still creates it.
    #[test]
    fn test_append_to_missing_file_is_just_the_addition() {
        let dir = tempfile::tempdir().expect("tempdir");
        let service = QualityProxyService::new();
        let mut req = request(ProxyOperation::Append, &dir.path().join("new.rs"));
        req.content = Some("pub fn only() {}\n".to_string());
        assert_eq!(
            service.get_operation_content(&req).expect("append resolves"),
            "pub fn only() {}\n"
        );
    }
}
