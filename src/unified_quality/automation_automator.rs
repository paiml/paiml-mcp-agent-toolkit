impl ConservativeAutomator {
    /// Create a new conservative automator
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: AutomatorConfig) -> Self {
        Self {
            safe_transforms: Self::initialize_safe_transforms(),
            git: GitSafetyNet::new(PathBuf::from(".")),
            rollback: RollbackManager::new(),
            config,
        }
    }

    /// Automatically fix a violation if safe
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn auto_fix(&self, violation: &Violation) -> Result<Fix> {
        if !self.config.enabled {
            return Err(anyhow!("Automation is disabled"));
        }

        match violation.violation_type {
            // Dead code removal is deterministic
            ViolationType::DeadCode => {
                let fix = self.remove_dead_code(violation)?;
                if self.config.create_branches {
                    self.git.create_fix_branch(&fix)?;
                }
                Ok(fix)
            }

            // Unused imports are safe to remove
            ViolationType::UnusedImport => {
                let fix = self.remove_import(violation)?;
                if self.config.create_branches {
                    self.git.create_fix_branch(&fix)?;
                }
                Ok(fix)
            }

            // Format violations are safe
            ViolationType::Formatting => {
                let fix = self.run_rustfmt(violation)?;
                if self.config.create_branches {
                    self.git.create_fix_branch(&fix)?;
                }
                Ok(fix)
            }

            // Everything else needs human review
            _ => Err(anyhow!("Violation type requires human review")),
        }
    }

    /// Batch fix multiple violations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn batch_fix(&self, violations: Vec<Violation>) -> Result<AutomationResult> {
        let mut result = AutomationResult {
            successful: Vec::new(),
            failed: Vec::new(),
            pending_review: Vec::new(),
            branch_name: None,
        };

        // Create a branch for all fixes
        if self.config.create_branches {
            let branch_name = format!("auto-fix-{}", chrono::Utc::now().timestamp());
            self.git.create_branch(&branch_name)?;
            result.branch_name = Some(branch_name);
        }

        // Process violations in batches
        for chunk in violations.chunks(self.config.max_batch_size) {
            for violation in chunk {
                match self.auto_fix(violation).await {
                    Ok(fix) => {
                        // Apply the fix
                        match self.apply_fix(&fix) {
                            Ok(verified) => {
                                result.successful.push(AppliedFix {
                                    fix: fix.clone(),
                                    verification_passed: verified,
                                    commit_hash: None,
                                });
                            }
                            Err(e) => {
                                result.failed.push(FailedFix {
                                    fix,
                                    error: e.to_string(),
                                    can_retry: true,
                                });
                            }
                        }
                    }
                    Err(_) if self.config.require_review => {
                        // Can't auto-fix, needs review
                        if let Ok(fix) = self.suggest_fix(violation) {
                            result.pending_review.push(fix);
                        }
                    }
                    Err(e) => {
                        // Failed to create fix
                        eprintln!("Failed to create fix: {e}");
                    }
                }
            }
        }

        // Commit if configured
        if self.config.auto_commit && !result.successful.is_empty() {
            self.git.commit_fixes(&result.successful)?;
        }

        Ok(result)
    }

    /// Rollback the last automation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn rollback(&mut self) -> Result<()> {
        self.rollback.rollback_last()
    }

    /// Initialize safe transforms
    fn initialize_safe_transforms() -> Vec<SafeTransform> {
        vec![
            SafeTransform {
                id: "remove_dead_code".to_string(),
                name: "Remove Dead Code".to_string(),
                handles: vec![ViolationType::DeadCode],
                success_rate: 1.0,
                transform: |_violation| {
                    Ok(Fix {
                        file: PathBuf::from("test.rs"),
                        fix_type: FixType::DeadCodeRemoval,
                        change: Change {
                            before: " fn unused() {}".to_string(),
                            after: String::new(),
                            line_range: (1, 1),
                        },
                        verify_command: Some("cargo check".to_string()),
                        branch_name: "fix/remove-dead-code".to_string(),
                    })
                },
            },
            SafeTransform {
                id: "remove_unused_import".to_string(),
                name: "Remove Unused Import".to_string(),
                handles: vec![ViolationType::UnusedImport],
                success_rate: 1.0,
                transform: |_violation| {
                    Ok(Fix {
                        file: PathBuf::from("test.rs"),
                        fix_type: FixType::UnusedImportRemoval,
                        change: Change {
                            before: "use std::collections::HashMap;".to_string(),
                            after: String::new(),
                            line_range: (1, 1),
                        },
                        verify_command: Some("cargo check".to_string()),
                        branch_name: "fix/remove-unused-import".to_string(),
                    })
                },
            },
        ]
    }

    /// Remove dead code
    fn remove_dead_code(&self, violation: &Violation) -> Result<Fix> {
        // In production, would use syn to parse and remove dead code
        Ok(Fix {
            file: PathBuf::from(&violation.file),
            fix_type: FixType::DeadCodeRemoval,
            change: Change {
                before: "dead code".to_string(),
                after: String::new(),
                line_range: (1, 10),
            },
            verify_command: Some("cargo check".to_string()),
            branch_name: format!("fix/dead-code-{}", chrono::Utc::now().timestamp()),
        })
    }

    /// Remove unused import
    fn remove_import(&self, violation: &Violation) -> Result<Fix> {
        // In production, would use syn to parse and remove import
        Ok(Fix {
            file: PathBuf::from(&violation.file),
            fix_type: FixType::UnusedImportRemoval,
            change: Change {
                before: "use unused;".to_string(),
                after: String::new(),
                line_range: (1, 1),
            },
            verify_command: Some("cargo check".to_string()),
            branch_name: format!("fix/unused-import-{}", chrono::Utc::now().timestamp()),
        })
    }

    /// Run rustfmt
    fn run_rustfmt(&self, violation: &Violation) -> Result<Fix> {
        // In production, would actually run rustfmt
        Ok(Fix {
            file: PathBuf::from(&violation.file),
            fix_type: FixType::Formatting,
            change: Change {
                before: "unformatted code".to_string(),
                after: "formatted code".to_string(),
                line_range: (1, 100),
            },
            verify_command: Some("cargo fmt -- --check".to_string()),
            branch_name: format!("fix/formatting-{}", chrono::Utc::now().timestamp()),
        })
    }

    /// Suggest a fix for manual review
    fn suggest_fix(&self, violation: &Violation) -> Result<Fix> {
        Ok(Fix {
            file: PathBuf::from(&violation.file),
            fix_type: FixType::SimpleRefactor,
            change: Change {
                before: "complex code".to_string(),
                after: "simplified code".to_string(),
                line_range: (1, 50),
            },
            verify_command: Some("cargo test".to_string()),
            branch_name: format!("fix/suggestion-{}", chrono::Utc::now().timestamp()),
        })
    }

    /// Apply a fix to a file
    fn apply_fix(&self, fix: &Fix) -> Result<bool> {
        // In production, would actually modify the file
        // For now, just verify
        if let Some(cmd) = &fix.verify_command {
            let output = Command::new("sh").arg("-c").arg(cmd).output()?;
            Ok(output.status.success())
        } else {
            Ok(true)
        }
    }

    /// Get list of safe transformations
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_safe_transforms(&self) -> Vec<SafeTransform> {
        self.safe_transforms.clone()
    }
}
