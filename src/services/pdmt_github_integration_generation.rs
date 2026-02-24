impl PdmtGitHubService {
    /// Validate PDMT issue request
    pub(crate) fn validate_request(
        &self,
        request: &PdmtIssueRequest,
    ) -> Result<(), PdmtGitHubError> {
        if request.title.is_empty() {
            return Err(PdmtGitHubError::InvalidConfig {
                message: "Title cannot be empty".to_string(),
            });
        }

        if request.description.is_empty() {
            return Err(PdmtGitHubError::InvalidConfig {
                message: "Description cannot be empty".to_string(),
            });
        }

        if let Some(complexity) = request.complexity_estimate {
            if complexity > 50 {
                return Err(PdmtGitHubError::InvalidConfig {
                    message: "Complexity estimate too high (max 50)".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Generate PDMT metadata
    pub(crate) fn generate_metadata(
        &self,
        request: &PdmtIssueRequest,
        config: &PdmtConfig,
    ) -> Result<PdmtMetadata, PdmtGitHubError> {
        let now = chrono::Utc::now().to_rfc3339();

        let validation_commands = generate_validation_commands_for_type(&request.issue_type);
        let success_criteria = generate_success_criteria_for_type(&request.issue_type);
        let quality_requirements = generate_quality_requirements_for_type(&request.issue_type);

        Ok(PdmtMetadata {
            seed: config.seed,
            quality_level: config.quality_level.clone(),
            granularity: config.granularity.clone(),
            issue_type: request.issue_type.clone(),
            priority: request.priority.clone(),
            complexity_estimate: request.complexity_estimate,
            generated_at: now,
            validation_commands,
            success_criteria,
            quality_requirements,
        })
    }

    /// Generate formatted issue title with type prefix
    pub(crate) fn generate_title(&self, request: &PdmtIssueRequest) -> String {
        format!("{} {}", request.issue_type.title_prefix(), request.title)
    }

    /// Generate structured issue body with PDMT format
    pub(crate) fn generate_issue_body(
        &self,
        request: &PdmtIssueRequest,
        metadata: &PdmtMetadata,
    ) -> Result<String, PdmtGitHubError> {
        let mut body = String::new();

        body.push_str("## Description\n\n");
        body.push_str(&request.description);
        body.push_str("\n\n");

        body.push_str("## PDMT Configuration\n\n");
        body.push_str(&format!("- **Seed**: {} (deterministic)\n", metadata.seed));
        body.push_str(&format!(
            "- **Quality Level**: {:?}\n",
            metadata.quality_level
        ));
        body.push_str(&format!("- **Granularity**: {:?}\n", metadata.granularity));
        body.push_str(&format!("- **Issue Type**: {:?}\n", metadata.issue_type));
        body.push_str(&format!("- **Priority**: {:?}\n", metadata.priority));

        if let Some(complexity) = metadata.complexity_estimate {
            body.push_str(&format!("- **Estimated Complexity**: {}\n", complexity));
        }

        body.push('\n');

        body.push_str("## Quality Requirements\n\n");
        let reqs = &metadata.quality_requirements;
        body.push_str(&format!(
            "- **Test Coverage**: ≥{}%\n",
            reqs.test_coverage
        ));
        body.push_str(&format!(
            "- **Max Complexity**: ≤{} per function\n",
            reqs.max_complexity
        ));
        body.push_str(&format!(
            "- **SATD Tolerance**: {} (zero tolerance)\n",
            reqs.satd_tolerance
        ));
        body.push_str(&format!(
            "- **Documentation Required**: {}\n",
            reqs.documentation_required
        ));
        body.push_str(&format!(
            "- **Property Tests Required**: {}\n",
            reqs.property_tests_required
        ));
        body.push('\n');

        body.push_str("## Validation Commands\n\n");
        body.push_str("```bash\n");
        for command in &metadata.validation_commands {
            body.push_str(&format!("{}\n", command));
        }
        body.push_str("```\n\n");

        body.push_str("## Success Criteria\n\n");
        for criterion in &metadata.success_criteria {
            body.push_str(&format!("- [ ] {}\n", criterion));
        }
        body.push('\n');

        body.push_str("## Toyota Way Standards\n\n");
        body.push_str("- **Kaizen**: Continuous improvement through quality gates\n");
        body.push_str("- **Genchi Genbutsu**: Verify implementation meets requirements\n");
        body.push_str("- **Jidoka**: Automated quality enforcement with human oversight\n");
        body.push('\n');

        body.push_str("---\n");
        body.push_str("*Generated using PDMT (Pragmatic Deterministic MCP Templating)*\n");
        body.push_str(&format!(
            "*Seed: {} | Quality: {:?} | Standards: Zero SATD*",
            metadata.seed, metadata.quality_level
        ));

        Ok(body)
    }

    /// Get complexity category label
    pub(crate) fn complexity_category(&self, complexity: u8) -> &'static str {
        match complexity {
            0..=5 => "trivial",
            6..=10 => "low",
            11..=15 => "medium",
            16..=25 => "high",
            _ => "very-high",
        }
    }

    /// Extract issue type from title prefix
    pub(crate) fn extract_issue_type_from_title(&self, title: &str) -> IssueType {
        if title.starts_with("feat:") {
            IssueType::Feature
        } else if title.starts_with("fix:") {
            IssueType::Bug
        } else if title.starts_with("enhance:") {
            IssueType::Enhancement
        } else if title.starts_with("refactor:") {
            IssueType::Refactor
        } else if title.starts_with("docs:") {
            IssueType::Documentation
        } else if title.starts_with("test:") {
            IssueType::Testing
        } else {
            IssueType::Feature // Default
        }
    }

    /// Extract validation commands from issue body
    pub(crate) fn extract_validation_commands(&self, body: &str) -> Vec<String> {
        let mut commands = Vec::new();
        let mut in_code_block = false;

        for line in body.lines() {
            if line.starts_with("```bash") {
                in_code_block = true;
                continue;
            }
            if line.starts_with("```") && in_code_block {
                in_code_block = false;
                continue;
            }
            if in_code_block && !line.trim().is_empty() {
                commands.push(line.to_string());
            }
        }

        commands
    }

    /// Extract success criteria from issue body
    pub(crate) fn extract_success_criteria(&self, body: &str) -> Vec<String> {
        let mut criteria = Vec::new();

        for line in body.lines() {
            if line.starts_with("- [ ]") {
                criteria.push(line[5..].trim().to_string());
            }
        }

        criteria
    }
}

/// Generate validation commands based on issue type
fn generate_validation_commands_for_type(issue_type: &IssueType) -> Vec<String> {
    let mut commands = vec![
        "cargo test --package pmat".to_string(),
        "pmat analyze satd --strict".to_string(),
        "pmat analyze complexity --max-complexity 20".to_string(),
        "make lint".to_string(),
    ];

    match issue_type {
        IssueType::Feature => {
            commands.push("pmat quality-gate --project-wide".to_string());
            commands.push("cargo doc --package pmat --no-deps".to_string());
        }
        IssueType::Bug => {
            commands.push("cargo test --package pmat --test integration".to_string());
        }
        IssueType::Enhancement => {
            commands.push("pmat analyze complexity --top-files 5".to_string());
        }
        IssueType::Refactor => {
            commands.push("pmat refactor auto --validate".to_string());
            commands.push("pmat quality-gate --strict".to_string());
        }
        IssueType::Documentation => {
            commands.push("cargo test --doc".to_string());
        }
        IssueType::Testing => {
            commands.push("make test-property".to_string());
            commands.push("cargo test --package pmat --test property".to_string());
        }
    }

    commands
}

/// Generate success criteria based on issue type
fn generate_success_criteria_for_type(issue_type: &IssueType) -> Vec<String> {
    let mut criteria = vec![
        "All tests pass successfully".to_string(),
        "Zero SATD comments in implementation".to_string(),
        "All functions ≤20 cyclomatic complexity".to_string(),
        "No lint violations".to_string(),
    ];

    match issue_type {
        IssueType::Feature => {
            criteria.push("Complete API documentation".to_string());
            criteria.push("Integration tests covering main workflows".to_string());
            criteria.push("Quality gate passes for all new code".to_string());
        }
        IssueType::Bug => {
            criteria.push("Root cause identified and documented".to_string());
            criteria.push("Regression test added".to_string());
            criteria.push("No similar issues exist".to_string());
        }
        IssueType::Enhancement => {
            criteria.push("Performance improvement demonstrated".to_string());
            criteria.push("Backward compatibility maintained".to_string());
        }
        IssueType::Refactor => {
            criteria.push("Code quality metrics improved".to_string());
            criteria.push("Functionality unchanged (tests pass)".to_string());
            criteria.push("Documentation updated".to_string());
        }
        IssueType::Documentation => {
            criteria.push("All examples tested and working".to_string());
            criteria.push("Documentation coverage improved".to_string());
        }
        IssueType::Testing => {
            criteria.push("Test coverage increased".to_string());
            criteria.push("Property tests cover edge cases".to_string());
        }
    }

    criteria
}

/// Generate quality requirements based on issue type
fn generate_quality_requirements_for_type(issue_type: &IssueType) -> QualityRequirements {
    let base_requirements = QualityRequirements::default();

    match issue_type {
        IssueType::Feature => QualityRequirements {
            test_coverage: 90,
            property_tests_required: true,
            ..base_requirements
        },
        IssueType::Bug => QualityRequirements {
            test_coverage: 95,
            ..base_requirements
        },
        IssueType::Testing => QualityRequirements {
            test_coverage: 100,
            property_tests_required: true,
            ..base_requirements
        },
        _ => base_requirements,
    }
}
