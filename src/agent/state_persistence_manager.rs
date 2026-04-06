/// State persistence manager
pub struct StatePersistence {
    /// Path to state file
    state_file: PathBuf,

    /// Current state
    state: Arc<RwLock<AgentState>>,

    /// Auto-save interval in seconds
    auto_save_interval: u64,
}

impl StatePersistence {
    /// Create new state persistence manager
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(state_dir: impl AsRef<Path>) -> Result<Self> {
        let state_file = state_dir.as_ref().join("agent_state.json");

        let state = if state_file.exists() {
            Self::load_from_file(&state_file)?
        } else {
            AgentState::default()
        };

        Ok(Self {
            state_file,
            state: Arc::new(RwLock::new(state)),
            auto_save_interval: 60, // Save every minute
        })
    }

    /// Load state from file
    fn load_from_file(path: &Path) -> Result<AgentState> {
        let contents = std::fs::read_to_string(path).context("Failed to read state file")?;

        serde_json::from_str(&contents).context("Failed to deserialize state")
    }

    /// Save current state to file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn save(&self) -> Result<()> {
        let state = self.state.read().await;
        let json = serde_json::to_string_pretty(&*state)?;

        // Create parent directory if needed
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Use safe two-phase write pattern with .tmp extension
        let temp_file = self.state_file.with_extension("tmp");
        fs::write(&temp_file, json).await?;
        fs::rename(&temp_file, &self.state_file).await?;

        Ok(())
    }

    /// Start auto-save task
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn start_auto_save(&self) {
        let state_file = self.state_file.clone();
        let state = self.state.clone();
        let interval = self.auto_save_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval.tick().await;

                // Save state
                if let Ok(state) = state.read().await.to_json() {
                    if let Err(e) = fs::write(&state_file, state).await {
                        tracing::error!("Failed to auto-save state: {}", e);
                    } else {
                        tracing::debug!("State auto-saved");
                    }
                }
            }
        });
    }

    /// Add or update monitored project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn add_project(&self, project: ProjectState) -> Result<()> {
        let mut state = self.state.write().await;
        state.monitored_projects.insert(project.id.clone(), project);
        state.last_updated = Utc::now();
        Ok(())
    }

    /// Remove monitored project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn remove_project(&self, project_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.monitored_projects.remove(project_id);
        state.last_updated = Utc::now();
        Ok(())
    }

    /// Update project metrics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn update_metrics(&self, project_id: &str, metrics: QualityMetrics) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(project) = state.monitored_projects.get_mut(project_id) {
            project.current_metrics = metrics.clone();
            project.last_analyzed = Some(Utc::now());

            // Add to history
            state.quality_history.push(QualitySnapshot {
                timestamp: Utc::now(),
                project_id: project_id.to_string(),
                metrics,
                violations: Vec::new(),
            });

            // Trim history to last 1000 entries
            if state.quality_history.len() > 1000 {
                let drain_count = state.quality_history.len() - 1000;
                state.quality_history.drain(0..drain_count);
            }

            state.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Get current state
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// Update statistics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn update_statistics<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AgentStatistics),
    {
        let mut state = self.state.write().await;
        updater(&mut state.statistics);
        state.last_updated = Utc::now();
        Ok(())
    }
}
