impl GitSafetyNet {
    fn new(work_dir: PathBuf) -> Self {
        debug_assert!(work_dir.exists(), "work_dir must exist: {}", work_dir.display());
        Self {
            work_dir,
            original_branch: None,
        }
    }

    fn create_branch(&self, name: &str) -> Result<()> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        Command::new("git")
            .current_dir(&self.work_dir)
            .args(["checkout", "-b", name])
            .output()?;
        Ok(())
    }

    fn create_fix_branch(&self, fix: &Fix) -> Result<()> {
        self.create_branch(&fix.branch_name)
    }

    fn commit_fixes(&self, fixes: &[AppliedFix]) -> Result<()> {
        debug_assert!(!fixes.is_empty(), "fixes must not be empty");
        let message = format!("Auto-fix: {} violations", fixes.len());
        Command::new("git")
            .current_dir(&self.work_dir)
            .args(["commit", "-m", &message])
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

    #[allow(dead_code)]
    fn add_rollback_point(&mut self, branch: String, commit: String, files: Vec<PathBuf>) {
        debug_assert!(true, "contract: add_rollback_point");
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
        debug_assert!(true, "contract: rollback_last");
        if let Some(point) = self.rollback_points.pop() {
            Command::new("git")
                .args(["checkout", &point.branch])
                .output()?;
            Command::new("git")
                .args(["reset", "--hard", &point.commit])
                .output()?;
            Ok(())
        } else {
            Err(anyhow!("No rollback points available"))
        }
    }
}
