impl ProjectMetadata {
    /// Create new project metadata with current PMAT version
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(pmat_version: impl Into<String>) -> Self {
        Self {
            pmat: PmatMetadata {
                version: pmat_version.into(),
                last_compliance_check: Some(chrono::Utc::now().to_rfc3339()),
                schema_version: default_schema_version(),
            },
            compliance: ComplianceMetadata::default(),
        }
    }

    /// Load project metadata from .pmat/project.toml
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load(project_path: &Path) -> Result<Self> {
        let path = Self::get_path(project_path);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Save project metadata to .pmat/project.toml
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, project_path: &Path) -> Result<()> {
        let path = Self::get_path(project_path);

        // Ensure .pmat directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content =
            toml::to_string_pretty(self).context("Failed to serialize project metadata")?;

        fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))
    }

    /// Check if project metadata exists
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn exists(project_path: &Path) -> bool {
        Self::get_path(project_path).exists()
    }

    /// Get path to project.toml
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn get_path(project_path: &Path) -> PathBuf {
        project_path.join(".pmat").join("project.toml")
    }

    /// Update last compliance check timestamp
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_compliance_check(&mut self) {
        self.pmat.last_compliance_check = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Record a migration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_migration(&mut self, from: String, to: String, success: bool) {
        self.compliance.migration_history.push(MigrationRecord {
            from_version: from,
            to_version: to.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            success,
        });

        if success {
            self.pmat.version = to;
            self.compliance.last_migration = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    /// Accept a breaking change
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn accept_breaking_change(&mut self, version: String) {
        if !self.compliance.breaking_changes_accepted.contains(&version) {
            self.compliance.breaking_changes_accepted.push(version);
        }
    }

    /// Check if a breaking change has been accepted
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_breaking_change_accepted(&self, version: &str) -> bool {
        self.compliance
            .breaking_changes_accepted
            .contains(&version.to_string())
    }
}
