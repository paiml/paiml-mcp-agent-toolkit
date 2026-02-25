// Default implementations and UnifiedConfig methods
// Included from config.rs - no `use` imports or `#!` inner attributes

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            mode: QualityMode::Observe,
            auto_progress: true,
            progress_after_days: 30,
            budget: BudgetConfig::default(),
            monitoring: MonitoringConfig::default(),
            automation: AutomationConfig::default(),
            research: ResearchConfig::default(),
        }
    }
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            complexity_points: 100,
            satd_allowance: 20,
            coverage_floor: 70.0,
            regeneration_daily: 5.0,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            incremental: true,
            cache_ast: true,
            dashboard_port: 8080,
            update_interval: 5,
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
            ],
        }
    }
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_review: true,
            safe_only: true,
            create_branches: false, // DISABLED: per CLAUDE.md zero-branching policy
        }
    }
}

impl UnifiedConfig {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get default configuration path
    #[must_use]
    pub fn default_path() -> PathBuf {
        PathBuf::from(".pmat/config.toml")
    }

    /// Check if should auto-progress
    #[must_use]
    pub fn should_progress(&self, days_in_mode: u32) -> bool {
        self.auto_progress && days_in_mode >= self.progress_after_days
    }

    /// Get next quality mode
    #[must_use]
    pub fn next_mode(&self) -> Option<QualityMode> {
        match self.mode {
            QualityMode::Observe => Some(QualityMode::Advise),
            QualityMode::Advise => Some(QualityMode::Guide),
            QualityMode::Guide => Some(QualityMode::Enforce),
            QualityMode::Enforce => Some(QualityMode::Extreme),
            QualityMode::Extreme => None,
        }
    }
}
