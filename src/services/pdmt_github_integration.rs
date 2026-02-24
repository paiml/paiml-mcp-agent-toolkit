#![cfg_attr(coverage_nightly, coverage(off))]
//! PDMT GitHub Issue Template Generation Service
//!
//! This module provides deterministic GitHub issue template generation using the
//! PDMT (Pragmatic Deterministic MCP Templating) approach. It creates structured,
//! quality-enforced issue descriptions with embedded metadata, validation commands,
//! and success criteria.
//!
//! # Features
//!
//! - **Deterministic Generation**: Uses seed 42 for reproducible issue templates
//! - **Issue Type Support**: Feature, bug, enhancement, refactor issue types
//! - **Quality Integration**: Embedded quality requirements and validation commands
//! - **PDMT Metadata**: Structured metadata for automated processing
//! - **Toyota Way Standards**: Zero SATD tolerance and complete specifications
//!
//! # Usage
//!
//! ```rust
//! use pmat::services::pdmt_github_integration::{PdmtGitHubService, IssueType, PdmtIssueRequest};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let service = PdmtGitHubService::new();
//!
//! let request = PdmtIssueRequest {
//!     title: "Implement GraphQL API endpoint".to_string(),
//!     description: "Add GraphQL support for better API flexibility".to_string(),
//!     issue_type: IssueType::Feature,
//!     priority: Priority::High,
//!     complexity_estimate: Some(15),
//!     assignees: vec![],
//!     custom_labels: vec![],
//!     config: None,
//! };
//!
//! let issue_template = service.generate_issue_template(&request)?;
//! assert!(issue_template.body.contains("## PDMT Configuration"));
//! # Ok(())
//! # }
//! ```

use crate::services::github_issues::{GitHubIssue, IssueRequest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

include!("pdmt_github_integration_types.rs");

/// Main service for PDMT GitHub integration
///
/// Provides deterministic GitHub issue template generation with embedded
/// quality requirements, validation commands, and PDMT metadata.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::pdmt_github_integration::{PdmtGitHubService, PdmtIssueRequest, IssueType, Priority};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let service = PdmtGitHubService::new();
///
/// let request = PdmtIssueRequest {
///     title: "Add user authentication system".to_string(),
///     description: "Implement secure user login and registration".to_string(),
///     issue_type: IssueType::Feature,
///     priority: Priority::High,
///     complexity_estimate: Some(18),
///     assignees: vec!["developer".to_string()],
///     custom_labels: vec!["security".to_string()],
///     config: None,
/// };
///
/// let template = service.generate_issue_template(&request)?;
/// assert!(template.body.contains("## Quality Requirements"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PdmtGitHubService {
    config: PdmtConfig,
}

impl PdmtGitHubService {
    /// Create a new PDMT GitHub service with default configuration
    pub fn new() -> Self {
        Self {
            config: PdmtConfig::default(),
        }
    }

    /// Create a new PDMT GitHub service with custom configuration
    pub fn with_config(config: PdmtConfig) -> Self {
        Self { config }
    }

    /// Generate a PDMT-compliant issue template
    pub fn generate_issue_template(
        &self,
        request: &PdmtIssueRequest,
    ) -> Result<PdmtIssueTemplate, PdmtGitHubError> {
        let config = request.config.as_ref().unwrap_or(&self.config);

        self.validate_request(request)?;
        let metadata = self.generate_metadata(request, config)?;
        let title = self.generate_title(request);
        let body = self.generate_issue_body(request, &metadata)?;

        let mut labels = request.issue_type.default_labels();
        labels.push(request.priority.label());
        labels.extend(request.custom_labels.clone());
        labels.push("pdmt".to_string());

        if let Some(complexity) = request.complexity_estimate {
            labels.push(format!("complexity:{}", self.complexity_category(complexity)));
        }

        Ok(PdmtIssueTemplate {
            title,
            body,
            labels,
            assignees: request.assignees.clone(),
            metadata,
        })
    }

    /// Convert PDMT template to GitHub issue request
    pub fn to_github_request(&self, template: &PdmtIssueTemplate) -> IssueRequest {
        IssueRequest {
            title: template.title.clone(),
            body: template.body.clone(),
            labels: template.labels.clone(),
            assignees: template.assignees.clone(),
        }
    }

    /// Extract PDMT metadata from GitHub issue
    pub fn extract_metadata(&self, issue: &GitHubIssue) -> Option<PdmtMetadata> {
        let body = issue.body.as_ref()?;

        if !body.contains("## PDMT Configuration") {
            return None;
        }

        Some(PdmtMetadata {
            seed: 42,
            quality_level: QualityLevel::Strict,
            granularity: Granularity::High,
            issue_type: self.extract_issue_type_from_title(&issue.title),
            priority: Priority::Medium,
            complexity_estimate: None,
            generated_at: issue.created_at.clone(),
            validation_commands: self.extract_validation_commands(body),
            success_criteria: self.extract_success_criteria(body),
            quality_requirements: QualityRequirements::default(),
        })
    }
}

impl Default for PdmtGitHubService {
    fn default() -> Self {
        Self::new()
    }
}

include!("pdmt_github_integration_generation.rs");

include!("pdmt_github_integration_tests.rs");
