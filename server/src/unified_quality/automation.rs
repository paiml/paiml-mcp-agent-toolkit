//! Automation Layer: Conservative Automation
//! 
//! Phase 4 Implementation (Months 10-12)
//! Safe, deterministic automation for simple fixes

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::unified_quality::metrics::{Violation, ViolationType};

/// Safe, deterministic automation for simple fixes
pub struct ConservativeAutomator {
    /// Only transformations with 100% success rate
    safe_transforms: Vec<SafeTransform>,
    
    /// Git integration for safety
    git: GitSafetyNet,
    
    /// Rollback capability
    rollback: RollbackManager,
    
    /// Configuration
    config: AutomatorConfig,
}

/// A safe, deterministic transformation
#[derive(Debug, Clone)]
pub struct SafeTransform {
    /// Transform identifier
    pub id: String,
    
    /// Transform name
    pub name: String,
    
    /// Violation types this transform handles
    pub handles: Vec<ViolationType>,
    
    /// Success rate (must be 1.0 for safe transforms)
    pub success_rate: f64,
    
    /// Transform function
    pub transform: TransformFn,
}

/// Transform function type
pub type TransformFn = fn(&Violation) -> Result<Fix>;

/// A fix to be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    /// File to fix
    pub file: PathBuf,
    
    /// Fix type
    pub fix_type: FixType,
    
    /// The actual change
    pub change: Change,
    
    /// Verification command
    pub verify_command: Option<String>,
    
    /// Branch name for the fix
    pub branch_name: String,
}

/// Types of fixes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FixType {
    DeadCodeRemoval,
    UnusedImportRemoval,
    Formatting,
    SimpleRefactor,
    DocumentationFix,
}

/// The actual change to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Original content
    pub before: String,
    
    /// Fixed content
    pub after: String,
    
    /// Line range affected
    pub line_range: (usize, usize),
}

/// Git safety net for automated changes
pub struct GitSafetyNet {
    /// Working directory
    work_dir: PathBuf,
    
    /// Current branch
    original_branch: Option<String>,
}

/// Rollback manager for undoing changes
pub struct RollbackManager {
    /// Rollback points
    rollback_points: Vec<RollbackPoint>,
    
    /// Maximum rollback history
    max_history: usize,
}

/// A rollback point
#[derive(Debug, Clone)]
struct RollbackPoint {
    /// Timestamp
    timestamp: std::time::SystemTime,
    
    /// Branch name
    branch: String,
    
    /// Commit hash
    commit: String,
    
    /// Files changed
    files: Vec<PathBuf>,
}

/// Automator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatorConfig {
    /// Enable automation
    pub enabled: bool,
    
    /// Require human review
    pub require_review: bool,
    
    /// Only apply safe transforms
    pub safe_only: bool,
    
    /// Create branches for fixes
    pub create_branches: bool,
    
    /// Auto-commit fixes
    pub auto_commit: bool,
    
    /// Maximum files per batch
    pub max_batch_size: usize,
}

impl Default for AutomatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_review: true,
            safe_only: true,
            create_branches: true,
            auto_commit: false,
            max_batch_size: 10,
        }
    }
}

/// Result of automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationResult {
    /// Fixes applied successfully
    pub successful: Vec<AppliedFix>,
    
    /// Fixes that failed
    pub failed: Vec<FailedFix>,
    
    /// Fixes requiring review
    pub pending_review: Vec<Fix>,
    
    /// Branch created (if any)
    pub branch_name: Option<String>,
}

/// Successfully applied fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedFix {
    pub fix: Fix,
    pub verification_passed: bool,
    pub commit_hash: Option<String>,
}

/// Failed fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedFix {
    pub fix: Fix,
    pub error: String,
    pub can_retry: bool,
}

impl ConservativeAutomator {
    /// Create a new conservative automator
    pub fn new(config: AutomatorConfig) -> Self {
        Self {
            safe_transforms: Self::initialize_safe_transforms(),
            git: GitSafetyNet::new(PathBuf::from(".")),
            rollback: RollbackManager::new(),
            config,
        }
    }
    
    /// Automatically fix a violation if safe
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
            _ => Err(anyhow!("Violation type requires human review"))
        }
    }
    
    /// Batch fix multiple violations
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
                        eprintln!("Failed to create fix: {}", e);
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
                            before: "#[allow(dead_code)] fn unused() {}".to_string(),
                            after: "".to_string(),
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
                            after: "".to_string(),
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
                after: "".to_string(),
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
                after: "".to_string(),
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
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()?;
            Ok(output.status.success())
        } else {
            Ok(true)
        }
    }
    
    /// Get list of safe transformations
    pub fn get_safe_transforms(&self) -> Vec<SafeTransform> {
        self.safe_transforms.clone()
    }
}

impl GitSafetyNet {
    fn new(work_dir: PathBuf) -> Self {
        Self {
            work_dir,
            original_branch: None,
        }
    }
    
    fn create_branch(&self, name: &str) -> Result<()> {
        Command::new("git")
            .current_dir(&self.work_dir)
            .args(&["checkout", "-b", name])
            .output()?;
        Ok(())
    }
    
    fn create_fix_branch(&self, fix: &Fix) -> Result<()> {
        self.create_branch(&fix.branch_name)
    }
    
    fn commit_fixes(&self, fixes: &[AppliedFix]) -> Result<()> {
        let message = format!("Auto-fix: {} violations", fixes.len());
        Command::new("git")
            .current_dir(&self.work_dir)
            .args(&["commit", "-m", &message])
            .output()?;
        Ok(())
    }
}

impl RollbackManager {
    fn new() -> Self {
        Self {
            rollback_points: Vec::new(),
            max_history: 10,
        }
    }
    
    fn add_rollback_point(&mut self, branch: String, commit: String, files: Vec<PathBuf>) {
        let point = RollbackPoint {
            timestamp: std::time::SystemTime::now(),
            branch,
            commit,
            files,
        };
        
        self.rollback_points.push(point);
        
        // Keep only recent history
        if self.rollback_points.len() > self.max_history {
            self.rollback_points.remove(0);
        }
    }
    
    fn rollback_last(&mut self) -> Result<()> {
        if let Some(point) = self.rollback_points.pop() {
            Command::new("git")
                .args(&["checkout", &point.branch])
                .output()?;
            Command::new("git")
                .args(&["reset", "--hard", &point.commit])
                .output()?;
            Ok(())
        } else {
            Err(anyhow!("No rollback points available"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservative_automator_creation() {
        let config = AutomatorConfig::default();
        let automator = ConservativeAutomator::new(config);
        assert!(!automator.safe_transforms.is_empty());
    }

    #[tokio::test]
    async fn test_auto_fix_dead_code() {
        let config = AutomatorConfig {
            enabled: true,
            ..Default::default()
        };
        let automator = ConservativeAutomator::new(config);
        
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::DeadCode,
            severity: crate::unified_quality::metrics::Severity::Low,
            value: 1.0,
            threshold: 0.0,
        };
        
        let result = automator.auto_fix(&violation).await;
        assert!(result.is_ok());
        
        let fix = result.unwrap();
        assert_eq!(fix.fix_type, FixType::DeadCodeRemoval);
    }

    #[test]
    fn test_rollback_manager() {
        let mut manager = RollbackManager::new();
        manager.add_rollback_point(
            "main".to_string(),
            "abc123".to_string(),
            vec![PathBuf::from("test.rs")],
        );
        assert_eq!(manager.rollback_points.len(), 1);
    }
}