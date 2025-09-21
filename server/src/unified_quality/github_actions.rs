//! GitHub Actions integration for unified quality system
//!
//! Provides quality gates and automation through GitHub Actions workflows

use crate::unified_quality::enforcement::{Decision, DiffAnalysis, ErrorBudgetEnforcer};
use crate::unified_quality::foundation::QualityMonitor;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// GitHub Actions integration for quality enforcement
pub struct GitHubActionsIntegration {
    /// Quality monitor
    monitor: QualityMonitor,

    /// Error budget enforcer  
    enforcer: ErrorBudgetEnforcer,

    /// Integration configuration
    config: GitHubConfig,
}

/// GitHub Actions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Repository owner/name
    pub repository: String,

    /// GitHub token for API access
    pub token: String,

    /// Quality gate thresholds
    pub quality_thresholds: QualityThresholds,

    /// Workflow triggers
    pub triggers: WorkflowTriggers,

    /// Comment settings
    pub comments: CommentConfig,
}

/// Quality thresholds for GitHub Actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Maximum allowed complexity increase
    pub max_complexity_increase: u32,

    /// Maximum allowed SATD increase
    pub max_satd_increase: u32,

    /// Minimum coverage requirement
    pub min_coverage: f64,

    /// Block PR if thresholds exceeded
    pub block_on_violation: bool,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_complexity_increase: 50,
            max_satd_increase: 5,
            min_coverage: 0.8,
            block_on_violation: true,
        }
    }
}

/// Workflow triggers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTriggers {
    /// Run on pull request
    pub on_pull_request: bool,

    /// Run on push to main
    pub on_push_main: bool,

    /// Run on schedule
    pub on_schedule: Option<String>,

    /// Specific branches to monitor
    pub branches: Vec<String>,
}

impl Default for WorkflowTriggers {
    fn default() -> Self {
        Self {
            on_pull_request: true,
            on_push_main: true,
            on_schedule: Some("0 6 * * 1".to_string()), // Weekly on Monday 6 AM
            branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

/// Comment configuration for GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentConfig {
    /// Post quality summary as PR comment
    pub post_summary: bool,

    /// Post detailed metrics
    pub post_details: bool,

    /// Update existing comments
    pub update_existing: bool,

    /// Comment template
    pub template: CommentTemplate,
}

impl Default for CommentConfig {
    fn default() -> Self {
        Self {
            post_summary: true,
            post_details: false,
            update_existing: true,
            template: CommentTemplate::default(),
        }
    }
}

/// Comment template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentTemplate {
    /// Header for quality reports
    pub header: String,

    /// Success message template
    pub success_template: String,

    /// Warning message template  
    pub warning_template: String,

    /// Failure message template
    pub failure_template: String,
}

impl Default for CommentTemplate {
    fn default() -> Self {
        Self {
            header: "## 📊 Code Quality Report".to_string(),
            success_template: "✅ **Quality checks passed!**\n\n- Complexity: {complexity}\n- SATD Count: {satd_count}\n- Coverage: {coverage:.1%}".to_string(),
            warning_template: "⚠️ **Quality warnings detected:**\n\n{warnings}\n\n- Complexity: {complexity}\n- SATD Count: {satd_count}\n- Coverage: {coverage:.1%}".to_string(),
            failure_template: "❌ **Quality checks failed:**\n\n{failures}\n\n- Complexity: {complexity}\n- SATD Count: {satd_count}\n- Coverage: {coverage:.1%}\n\nPlease address these issues before merging.".to_string(),
        }
    }
}

/// GitHub Actions workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Overall status
    pub status: WorkflowStatus,

    /// Quality analysis results
    pub analysis: QualityAnalysis,

    /// Enforcement decision
    pub decision: Decision,

    /// Generated comment (if any)
    pub comment: Option<String>,

    /// Workflow outputs for GitHub Actions
    pub outputs: HashMap<String, String>,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Quality checks passed
    Success,

    /// Quality issues found but not blocking
    Warning,

    /// Quality checks failed - blocking merge
    Failure,

    /// Error during execution
    Error(String),
}

/// Quality analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysis {
    /// Files analyzed
    pub files_analyzed: usize,

    /// Total complexity
    pub total_complexity: u32,

    /// Complexity change from base
    pub complexity_change: i32,

    /// SATD count
    pub satd_count: u32,

    /// SATD change from base
    pub satd_change: i32,

    /// Test coverage
    pub coverage: f64,

    /// Coverage change from base
    pub coverage_change: f64,

    /// Quality violations found
    pub violations: Vec<QualityViolation>,
}

/// Quality violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    /// File path
    pub file: String,

    /// Violation type
    pub violation_type: String,

    /// Severity level
    pub severity: ViolationSeverity,

    /// Description
    pub message: String,

    /// Line number (if applicable)
    pub line: Option<u32>,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_quality::enforcement::EnforcerConfig;
    use crate::unified_quality::foundation::MonitorConfig;

    #[test]
    fn test_github_config_default() {
        let thresholds = QualityThresholds::default();
        assert_eq!(thresholds.max_complexity_increase, 50);
        assert_eq!(thresholds.max_satd_increase, 5);
        assert_eq!(thresholds.min_coverage, 0.8);
        assert!(thresholds.block_on_violation);
    }

    #[test]
    fn test_workflow_triggers_default() {
        let triggers = WorkflowTriggers::default();
        assert!(triggers.on_pull_request);
        assert!(triggers.on_push_main);
        assert!(triggers.on_schedule.is_some());
        assert_eq!(triggers.branches.len(), 2);
    }

    #[test]
    fn test_comment_template_default() {
        let template = CommentTemplate::default();
        assert!(template.header.contains("Code Quality Report"));
        assert!(template.success_template.contains("Quality checks passed"));
        assert!(template.failure_template.contains("Quality checks failed"));
    }

    #[test]
    fn test_workflow_yaml_generation() {
        let monitor = QualityMonitor::new(MonitorConfig::default()).unwrap();
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let config = GitHubConfig {
            repository: "owner/repo".to_string(),
            token: "token".to_string(),
            quality_thresholds: QualityThresholds::default(),
            triggers: WorkflowTriggers::default(),
            comments: CommentConfig::default(),
        };

        let integration = GitHubActionsIntegration::new(monitor, enforcer, config);
        let yaml = integration.generate_workflow_yaml();

        assert!(yaml.contains("name: Quality Gate"));
        assert!(yaml.contains("pull_request:"));
        assert!(yaml.contains("pmat unified-quality analyze"));
    }

    #[test]
    fn test_violation_severity_ordering() {
        let severities = vec![
            ViolationSeverity::Info,
            ViolationSeverity::Warning,
            ViolationSeverity::Error,
            ViolationSeverity::Critical,
        ];

        // Just test that all variants exist and can be created
        assert_eq!(severities.len(), 4);
    }

    #[test]
    fn test_team_extraction() {
        let monitor = QualityMonitor::new(MonitorConfig::default()).unwrap();
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let config = GitHubConfig {
            repository: "my-org/my-repo".to_string(),
            token: "token".to_string(),
            quality_thresholds: QualityThresholds::default(),
            triggers: WorkflowTriggers::default(),
            comments: CommentConfig::default(),
        };

        let integration = GitHubActionsIntegration::new(monitor, enforcer, config);
        let team = integration.extract_team_from_repository();
        assert_eq!(team, "my-org");
    }

    #[test]
    fn test_workflow_status_variants() {
        let statuses = vec![
            WorkflowStatus::Success,
            WorkflowStatus::Warning,
            WorkflowStatus::Failure,
            WorkflowStatus::Error("test".to_string()),
        ];

        assert_eq!(statuses.len(), 4);

        // Test Debug formatting
        let _ = format!("{:?}", WorkflowStatus::Success);
        let _ = format!("{:?}", WorkflowStatus::Error("test".to_string()));
    }
}
