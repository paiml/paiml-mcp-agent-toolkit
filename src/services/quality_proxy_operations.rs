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
        info!("Proxying {} operation for {}", operation_name(&request.operation), request.file_path);

        let content = self.get_operation_content(&request)?;

        // The gates are chosen from the language, not from a raw extension
        // string. This lookup used to end in `.unwrap_or("rs")`, so an
        // extensionless `Makefile` or `Dockerfile` was handed to `cargo clippy`
        // as Rust and rejected with fabricated parse errors; see
        // `proxy_language` for that and for the case-sensitivity that let a
        // `.RS` file skip every gate.
        let language = proxy_language(&request.file_path);
        // A client's `quality_config` may only TIGHTEN the project's own
        // `[quality]` (pmat.toml, found by walking up from the target file). It
        // used to be the whole gate: `{"max_complexity":9999,"allow_satd":true}`
        // flipped a failing verdict to passing with no config source consulted.
        let quality_config = effective_quality_config(
            &request.file_path,
            &request.quality_config,
            project_quality_floor(&request.file_path).as_ref(),
        );
        let outcome = self
            .analyze_content(&content, &request.file_path, language, &quality_config)
            .await?;
        let passed = outcome.passed;

        let (status, final_content, refactoring_applied, refactoring_plan) = if passed {
            (ProxyStatus::Accepted, content, false, None)
        } else {
            match request.mode {
                ProxyMode::Strict => (ProxyStatus::Rejected, String::new(), false, None),
                // Advisory returns the content either way, so the caller can
                // still proceed — but it must not LAUNDER a failing verdict as
                // `accepted`. It did: `passed` was not consulted, and a client
                // that only read `status` learned nothing (CRUX-10 B1, #1151).
                ProxyMode::Advisory => (ProxyStatus::Rejected, content, false, None),
                ProxyMode::AutoFix => {
                    self.auto_fix_decision(&content, &request.file_path, language, &quality_config)
                        .await?
                }
            }
        };
        Ok(ProxyResponse {
            status,
            quality_report: QualityReport {
                passed,
                metrics: outcome.metrics,
                violations: outcome.violations,
                language: outcome.language,
                gates_run: outcome.gates_run,
            },
            final_content,
            refactoring_applied,
            written: false,
            refactoring_plan,
        })
    }

    /// The auto-fix branch of a failing verdict: try the fix, re-grade it,
    /// and carry the plan on the rejection too. The plan is the only place
    /// the *reason* an auto-fix did nothing can surface — for a language with
    /// no auto-fix implemented, `auto_fix_content` records a "skipped" step
    /// naming it, and dropping the plan turned that back into a bare
    /// `refactoring_applied: false`, indistinguishable from "there was nothing
    /// to fix". `refactoring_applied` stays false on rejection: it is what was
    /// attempted, not what was applied.
    async fn auto_fix_decision(
        &self,
        content: &str,
        file_path: &str,
        language: Language,
        quality_config: &QualityConfig,
    ) -> Result<Decision> {
        let (fixed_content, plan) =
            match self.auto_fix_content(content, file_path, language, quality_config).await {
                Ok(fixed) => fixed,
                Err(e) => {
                    warn!("Auto-fix failed: {}", e);
                    return Ok((ProxyStatus::Rejected, String::new(), false, None));
                }
            };
        let fixed = self.analyze_content(&fixed_content, file_path, language, quality_config).await?;
        if fixed.passed {
            Ok((ProxyStatus::Modified, fixed_content, true, Some(plan)))
        } else {
            warn!("Auto-fix failed to meet quality standards");
            Ok((ProxyStatus::Rejected, String::new(), false, Some(plan)))
        }
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
                        // Appending to a file that does not exist yet creates
                        // it — but only somewhere it could actually be created.
                        // A path under a directory that does not exist was
                        // silently treated as "an append to the empty file" and
                        // came back accepted/passed:true, while `quality_gate`
                        // rejected the very same path with "File does not
                        // exist". Two tools in one session must not disagree
                        // about whether a path is real.
                        None => {
                            ensure_appendable(&request.file_path)?;
                            Ok(append_content.clone())
                        }
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

/// Refuse an append to a path that could not be created if it were performed.
///
/// The file itself may legitimately not exist yet; its directory may not.
fn ensure_appendable(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    match path.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) if !dir.is_dir() => anyhow::bail!(
            "Append rejected: {} does not exist and neither does its directory {}",
            file_path,
            dir.display()
        ),
        _ => Ok(()),
    }
}

/// Concatenate appended text without inventing or losing a line break.
fn join_appended(existing: &str, addition: &str) -> String {
    if existing.is_empty() || existing.ends_with('\n') {
        format!("{existing}{addition}")
    } else {
        format!("{existing}\n{addition}")
    }
}
/// What a graded request resolves to: status, the content handed back,
/// whether a refactoring was applied, and the plan (also on rejection).
type Decision = (ProxyStatus, String, bool, Option<Vec<HashMap<String, serde_json::Value>>>);

fn operation_name(op: &ProxyOperation) -> &'static str {
    match op {
        ProxyOperation::Write => "write",
        ProxyOperation::Edit => "edit",
        ProxyOperation::Append => "append",
    }
}


/// The project's own `[quality]` floor for `file_path`, if a `pmat.toml` sits
/// in any ancestor directory. `None` when there is no project config to
/// honour — the client's config is then the only one, exactly as before.
pub(crate) fn project_quality_floor(file_path: &str) -> Option<QualityConfig> {
    use crate::services::configuration_service::ConfigurationService;
    let start = Path::new(file_path);
    let mut dir = if start.is_absolute() {
        start.parent().map(Path::to_path_buf)
    } else {
        std::env::current_dir().ok().map(|cwd| cwd.join(start)).and_then(|p| p.parent().map(Path::to_path_buf))
    }?;
    loop {
        let candidate = dir.join("pmat.toml");
        if candidate.is_file() {
            let q = ConfigurationService::new(Some(candidate)).get_config().ok()?.quality;
            return Some(QualityConfig {
                max_complexity: q.max_complexity,
                allow_satd: q.allow_satd,
                require_docs: q.require_docs,
                // `[quality]` has no auto_format; the client's value stands.
                auto_format: true,
            });
        }
        dir = dir.parent()?.to_path_buf();
    }
}

/// Merge a client's `quality_config` onto the project's floor so the client
/// can only make the gate STRICTER. Pure, so the rule is testable on its own:
/// the lower complexity ceiling wins; SATD is allowed only if BOTH allow it;
/// docs are required if EITHER requires them; `auto_format` is the client's.
pub(crate) fn effective_quality_config(
    _file_path: &str,
    client: &QualityConfig,
    floor: Option<&QualityConfig>,
) -> QualityConfig {
    // A project with no pmat.toml has the default config as its floor; a
    // client is not allowed to loosen past it either (CRUX-10 B2 measured
    // `allow_satd:true` on a file outside any project being honoured).
    let default_floor = QualityConfig::default();
    let f = floor.unwrap_or(&default_floor);
    QualityConfig {
        max_complexity: client.max_complexity.min(f.max_complexity),
        allow_satd: client.allow_satd && f.allow_satd,
        require_docs: client.require_docs || f.require_docs,
        auto_format: client.auto_format,
    }
}

#[cfg(test)]
mod proxy_file_path_tests {
    use super::*;
    /// CRUX-10 B1: advisory mode must not launder a failing verdict. Fixed
    /// input (the spec's `.bad` fixture) so the failing branch is exercised
    /// on every run without depending on a generator. Named mutation: the
    /// advisory `else` arm returning `Accepted` fails this test and the
    /// property test `test_advisory_mode_reports_the_verdict_and_returns_the_content`.
    #[test]
    fn advisory_rejects_failing_content_and_returns_it_unwritten() {
        use crate::models::proxy::{ProxyMode, ProxyOperation, ProxyRequest, ProxyStatus, QualityConfig};
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let bad = "// TODO: x\npub fn f(){}\n".to_string();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "/nonexistent/project/B1.rs".to_string(),
            content: Some(bad.clone()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Advisory,
            quality_config: QualityConfig::default(),
        };
        let r = rt.block_on(crate::services::quality_proxy::QualityProxyService::new().proxy_operation(request)).expect("advisory never errors");
        assert!(!r.quality_report.passed, "the fixture must fail the default gate (SATD)");
        assert!(matches!(r.status, ProxyStatus::Rejected), "advisory laundered passed=false as {:?}", r.status);
        assert_eq!(r.final_content, bad, "advisory returns the content for the client to decide on");
        assert!(!r.written);
    }

    /// CRUX-10 B2, outside any project: the default config is the floor, so
    /// `allow_satd:true` on a bare temp file must still not be honoured.
    /// RED before the fix: the no-floor arm returned the client's config as is.
    #[test]
    fn without_a_pmat_toml_the_default_config_is_the_floor() {
        use crate::models::proxy::QualityConfig;
        let client = QualityConfig {
            max_complexity: 9999,
            allow_satd: true,
            require_docs: false,
            auto_format: false,
        };
        let eff = effective_quality_config("/nonexistent/dir/x.rs", &client, None);
        let d = QualityConfig::default();
        assert_eq!(eff.max_complexity, d.max_complexity.min(9999));
        assert!(!eff.allow_satd || d.allow_satd, "the client loosened allow_satd past the default floor");
        assert_eq!(eff.require_docs, d.require_docs);
        assert!(!eff.auto_format, "auto_format is the client's");
    }

    /// CRUX-10 B2: a client's config may only tighten the project's floor.
    #[test]
    fn client_quality_config_can_only_tighten_the_project_floor() {
        use crate::models::proxy::QualityConfig;
        let floor = QualityConfig {
            max_complexity: 20,
            allow_satd: false,
            require_docs: true,
            auto_format: true,
        };
        let loosening = QualityConfig {
            max_complexity: 9999,
            allow_satd: true,
            require_docs: false,
            auto_format: false,
        };
        let e = effective_quality_config("x.rs", &loosening, Some(&floor));
        assert_eq!(e.max_complexity, 20, "cannot raise the ceiling");
        assert!(!e.allow_satd, "cannot allow SATD the project forbids");
        assert!(e.require_docs, "cannot drop a docs requirement");
        assert!(!e.auto_format, "auto_format is the client's own");
        let tightening = QualityConfig {
            max_complexity: 5,
            allow_satd: false,
            require_docs: true,
            auto_format: true,
        };
        let e = effective_quality_config("x.rs", &tightening, Some(&floor));
        assert_eq!(e.max_complexity, 5, "tightening is honoured");
        // no project floor: the client's config is the whole gate, as before
        let e = effective_quality_config("x.rs", &loosening, None);
        // No pmat.toml: the default config is the floor, not "anything goes".
        let d = QualityConfig::default();
        assert_eq!(e.max_complexity, d.max_complexity.min(9999));
        assert_eq!(e.allow_satd, d.allow_satd);
    }

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

    /// `quality_proxy` accepted edit/append on a path that cannot exist while
    /// `quality_gate`, in the same session, rejected it with "File does not
    /// exist". Both must refuse it.
    #[test]
    fn test_operations_on_an_impossible_path_are_rejected() {
        let service = QualityProxyService::new();
        let missing = std::path::Path::new("/does/not/exist/never_existed.rs");

        let mut edit = request(ProxyOperation::Edit, missing);
        edit.old_content = Some("x".to_string());
        edit.new_content = Some("y".to_string());
        let err = service
            .get_operation_content(&edit)
            .expect_err("editing a nonexistent file must fail loudly");
        assert!(
            err.to_string().contains("requires the file to exist"),
            "{err}"
        );

        let mut append = request(ProxyOperation::Append, missing);
        append.content = Some("pub fn added() {}\n".to_string());
        let err = service
            .get_operation_content(&append)
            .expect_err("appending under a nonexistent directory must fail loudly");
        assert!(err.to_string().contains("Append rejected"), "{err}");
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
