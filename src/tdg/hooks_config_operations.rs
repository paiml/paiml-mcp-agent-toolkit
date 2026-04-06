// Method implementations for hooks_config types.
// Included from hooks_config.rs — no `use` imports or inner attributes allowed.

impl TdgHooksConfig {
    /// Load configuration from .pmat/tdg-rules.toml
    pub fn load(project_root: &Path) -> Result<Self> {
        debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
        let config_path = project_root.join(".pmat").join("tdg-rules.toml");

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .context(format!("Failed to read config file: {:?}", config_path))?;

        let config: TdgHooksConfig = toml::from_str(&contents)
            .context(format!("Failed to parse config file: {:?}", config_path))?;

        Ok(config)
    }

    /// Create default configuration file at .pmat/tdg-rules.toml
    pub fn create_default(project_root: &Path) -> Result<()> {
        debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
        let pmat_dir = project_root.join(".pmat");
        let config_path = pmat_dir.join("tdg-rules.toml");

        // Create .pmat directory if it doesn't exist
        if !pmat_dir.exists() {
            fs::create_dir_all(&pmat_dir)?;
        }

        // Don't overwrite existing config
        if config_path.exists() {
            return Ok(());
        }

        let default_config = Self::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        fs::write(&config_path, toml_string)
            .context(format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }
}

impl QualityGatesConfig {
    /// Get minimum grade for a language (with fallback to deprecated fields)
    pub fn get_min_grade(&self, language: &str) -> Option<&str> {
        debug_assert!(!language.is_empty(), "language must not be empty");
        // Try new min_grades map first
        if let Some(grade) = self.min_grades.get(language) {
            return Some(grade.as_str());
        }

        // Fallback to deprecated fields for backward compatibility
        match language.to_lowercase().as_str() {
            "rust" => self.rust_min_grade.as_deref(),
            "typescript" | "javascript" => self.typescript_min_grade.as_deref(),
            "python" => self.python_min_grade.as_deref(),
            _ => None,
        }
    }

    /// Get default minimum grade (B+ for most languages)
    pub fn get_default_min_grade(&self) -> &str {
        "B+"
    }
}

impl std::fmt::Display for EnforcementMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Warning => write!(f, "warning"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}
