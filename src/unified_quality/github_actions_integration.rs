impl GitHubActionsIntegration {
    /// Create new GitHub Actions integration
    #[must_use]
    pub fn new(
        monitor: QualityMonitor,
        enforcer: ErrorBudgetEnforcer,
        config: GitHubConfig,
    ) -> Self {
        Self {
            monitor,
            enforcer,
            config,
        }
    }

    /// Run quality analysis for pull request
    pub async fn analyze_pull_request(
        &mut self,
        _pr_number: u32,
        _base_ref: String,
        _head_ref: String,
        changed_files: Vec<PathBuf>,
    ) -> Result<WorkflowResult> {
        // Analyze changed files
        let mut total_complexity = 0;
        let mut total_satd = 0;
        let mut violations = Vec::new();

        for file in &changed_files {
            if let Some(metrics) = self.monitor.get_metrics(file) {
                total_complexity += metrics.complexity;
                total_satd += metrics.satd_count;

                // Check for violations
                if metrics.complexity > self.config.quality_thresholds.max_complexity_increase {
                    violations.push(QualityViolation {
                        file: file.to_string_lossy().to_string(),
                        violation_type: "complexity".to_string(),
                        severity: ViolationSeverity::Error,
                        message: format!(
                            "Complexity {} exceeds threshold {}",
                            metrics.complexity,
                            self.config.quality_thresholds.max_complexity_increase
                        ),
                        line: None,
                    });
                }
            }
        }

        // Create diff analysis for enforcer
        let diff = DiffAnalysis {
            complexity_change: total_complexity as i32, // Simplified - would need base comparison
            satd_change: total_satd as i32,
            coverage_change: 0.0, // Would need actual coverage analysis
            files_changed: changed_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        };

        // Get enforcement decision
        let team_id = self.extract_team_from_repository();
        let decision = self.enforcer.check_commit(&team_id, &diff);

        // Determine workflow status
        let status = match &decision {
            Decision::Approved => {
                if violations.is_empty() {
                    WorkflowStatus::Success
                } else {
                    WorkflowStatus::Warning
                }
            }
            Decision::Warning(_) => WorkflowStatus::Warning,
            Decision::RequiresApproval { .. } => WorkflowStatus::Warning,
            Decision::Blocked { .. } => WorkflowStatus::Failure,
        };

        // Create analysis summary
        let analysis = QualityAnalysis {
            files_analyzed: changed_files.len(),
            total_complexity,
            complexity_change: total_complexity as i32,
            satd_count: total_satd,
            satd_change: total_satd as i32,
            coverage: 0.8, // Would need actual coverage calculation
            coverage_change: 0.0,
            violations,
        };

        // Generate comment if configured
        let comment = if self.config.comments.post_summary {
            Some(self.generate_comment(&status, &analysis, &decision))
        } else {
            None
        };

        // Create workflow outputs
        let mut outputs = HashMap::new();
        outputs.insert("status".to_string(), format!("{status:?}"));
        outputs.insert("complexity".to_string(), total_complexity.to_string());
        outputs.insert("satd_count".to_string(), total_satd.to_string());
        outputs.insert(
            "files_analyzed".to_string(),
            changed_files.len().to_string(),
        );
        outputs.insert(
            "violations".to_string(),
            analysis.violations.len().to_string(),
        );

        Ok(WorkflowResult {
            status,
            analysis,
            decision,
            comment,
            outputs,
        })
    }

    /// Generate GitHub Actions workflow YAML
    #[must_use]
    pub fn generate_workflow_yaml(&self) -> String {
        let triggers = &self.config.triggers;
        let thresholds = &self.config.quality_thresholds;

        format!(
            r#"name: Quality Gate
on:
  pull_request:
    branches: [{}]
  push:
    branches: [{}]
  schedule:
    - cron: '{}'

jobs:
  quality-gate:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      checks: write

    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        profile: minimal
        override: true

    - name: Install PMAT
      run: cargo install pmat --version latest

    - name: Run Quality Analysis
      id: quality
      run: |
        # Run PMAT unified quality analysis
        pmat unified-quality analyze \
          --max-complexity {} \
          --max-satd {} \
          --min-coverage {} \
          --output-format github-actions \
          --changed-files-only ${{{{ github.event_name == 'pull_request' }}}}

    - name: Update PR Comment
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v7
      with:
        script: |
          const analysis = JSON.parse(process.env.QUALITY_ANALYSIS);
          const comment = process.env.QUALITY_COMMENT;

          // Find existing comment
          const comments = await github.rest.issues.listComments({{
            owner: context.repo.owner,
            repo: context.repo.repo,
            issue_number: context.issue.number,
          }});

          const existingComment = comments.data.find(
            comment => comment.body.includes('📊 Code Quality Report')
          );

          if (existingComment && {}) {{
            // Update existing comment
            await github.rest.issues.updateComment({{
              owner: context.repo.owner,
              repo: context.repo.repo,
              comment_id: existingComment.id,
              body: comment,
            }});
          }} else {{
            // Create new comment
            await github.rest.issues.createComment({{
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: comment,
            }});
          }}
      env:
        QUALITY_ANALYSIS: ${{{{ steps.quality.outputs.analysis }}}}
        QUALITY_COMMENT: ${{{{ steps.quality.outputs.comment }}}}

    - name: Set Status Check
      if: always()
      run: |
        status="${{{{ steps.quality.outputs.status }}}}"
        if [ "$status" = "Success" ]; then
          exit 0
        elif [ "$status" = "Warning" ]; then
          echo "::warning::Quality warnings detected"
          exit 0
        else
          echo "::error::Quality checks failed"
          exit 1
        fi
"#,
            triggers.branches.join(", "),
            triggers.branches.join(", "),
            triggers.on_schedule.as_deref().unwrap_or("0 6 * * 1"),
            thresholds.max_complexity_increase,
            thresholds.max_satd_increase,
            thresholds.min_coverage,
            self.config.comments.update_existing,
        )
    }

    /// Generate comment text based on analysis results
    fn generate_comment(
        &self,
        status: &WorkflowStatus,
        analysis: &QualityAnalysis,
        decision: &Decision,
    ) -> String {
        let template = &self.config.comments.template;
        let mut comment = format!("{}\n\n", template.header);

        match status {
            WorkflowStatus::Success => {
                comment.push_str(
                    &template
                        .success_template
                        .replace("{complexity}", &analysis.total_complexity.to_string())
                        .replace("{satd_count}", &analysis.satd_count.to_string())
                        .replace("{coverage}", &format!("{:.1}", analysis.coverage * 100.0)),
                );
            }
            WorkflowStatus::Warning => {
                let warnings = analysis
                    .violations
                    .iter()
                    .filter(|v| matches!(v.severity, ViolationSeverity::Warning))
                    .map(|v| format!("- {}: {}", v.file, v.message))
                    .collect::<Vec<_>>()
                    .join("\n");

                comment.push_str(
                    &template
                        .warning_template
                        .replace("{warnings}", &warnings)
                        .replace("{complexity}", &analysis.total_complexity.to_string())
                        .replace("{satd_count}", &analysis.satd_count.to_string())
                        .replace("{coverage}", &format!("{:.1}", analysis.coverage * 100.0)),
                );
            }
            WorkflowStatus::Failure => {
                let failures = analysis
                    .violations
                    .iter()
                    .filter(|v| {
                        matches!(
                            v.severity,
                            ViolationSeverity::Error | ViolationSeverity::Critical
                        )
                    })
                    .map(|v| format!("- {}: {}", v.file, v.message))
                    .collect::<Vec<_>>()
                    .join("\n");

                comment.push_str(
                    &template
                        .failure_template
                        .replace("{failures}", &failures)
                        .replace("{complexity}", &analysis.total_complexity.to_string())
                        .replace("{satd_count}", &analysis.satd_count.to_string())
                        .replace("{coverage}", &format!("{:.1}", analysis.coverage * 100.0)),
                );
            }
            WorkflowStatus::Error(e) => {
                comment.push_str(&format!("❌ **Error during quality analysis:**\n\n{e}"));
            }
        }

        // Add decision details
        match decision {
            Decision::Approved => {
                comment.push_str("\n\n✅ **Error budget status:** Approved");
            }
            Decision::Warning(msg) => {
                comment.push_str(&format!("\n\n⚠️ **Error budget status:** Warning\n{msg}"));
            }
            Decision::RequiresApproval { approvers, .. } => {
                comment.push_str(&format!(
                    "\n\n👥 **Error budget status:** Requires approval from: {}",
                    approvers.join(", ")
                ));
            }
            Decision::Blocked { suggestion, .. } => {
                comment.push_str(&format!(
                    "\n\n🚫 **Error budget status:** Blocked\n\n{suggestion}"
                ));
            }
        }

        comment.push_str(&format!(
            "\n\n---\n📊 **Summary:**\n- Files analyzed: {}\n- Complexity change: {:+}\n- SATD change: {:+}\n- Coverage: {:.1}%",
            analysis.files_analyzed,
            analysis.complexity_change,
            analysis.satd_change,
            analysis.coverage * 100.0
        ));

        comment
    }

    /// Extract team identifier from repository name
    fn extract_team_from_repository(&self) -> String {
        // Simple heuristic: use repository owner as team
        self.config
            .repository
            .split('/')
            .next()
            .unwrap_or("default")
            .to_string()
    }
}
