//! State persistence layer for Claude Code Agent
//!
//! PMAT-7006: Provides persistent storage for monitoring state, project configurations,
//! and quality metrics history across agent restarts.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Persistent state for the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Version of the state format
    pub version: String,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,

    /// Currently monitored projects
    pub monitored_projects: HashMap<String, ProjectState>,

    /// Historical quality metrics
    pub quality_history: Vec<QualitySnapshot>,

    /// Agent configuration overrides
    pub config_overrides: HashMap<String, serde_json::Value>,

    /// Session statistics
    pub statistics: AgentStatistics,
}

/// State of a monitored project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    /// Project identifier
    pub id: String,

    /// Project path
    pub path: PathBuf,

    /// Monitoring start time
    pub started_at: DateTime<Utc>,

    /// Last analysis time
    pub last_analyzed: Option<DateTime<Utc>>,

    /// Current quality metrics
    pub current_metrics: QualityMetrics,

    /// Watch patterns
    pub watch_patterns: Vec<String>,

    /// Custom thresholds
    pub thresholds: QualityThresholds,
}

/// Quality metrics for a project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// Average complexity
    pub avg_complexity: f64,

    /// Maximum complexity
    pub max_complexity: u32,

    /// SATD count
    pub satd_count: usize,

    /// Dead code percentage
    pub dead_code_percentage: f64,

    /// Quality score (0-100)
    pub quality_score: f64,

    /// Total files analyzed
    pub files_analyzed: usize,

    /// Total violations
    pub total_violations: usize,
}

/// Quality thresholds for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub max_complexity: u32,
    pub satd_tolerance: usize,
    pub dead_code_max_percentage: f64,
    pub min_quality_score: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_complexity: 20, // Toyota Way standard
            satd_tolerance: 0,  // Zero tolerance
            dead_code_max_percentage: 10.0,
            min_quality_score: 80.0,
        }
    }
}

/// Historical quality snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,

    /// Project ID
    pub project_id: String,

    /// Metrics at this point in time
    pub metrics: QualityMetrics,

    /// Any violations detected
    pub violations: Vec<String>,
}

/// Agent statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStatistics {
    /// Total monitoring sessions
    pub sessions_count: u64,

    /// Total analyses performed
    pub analyses_performed: u64,

    /// Total violations detected
    pub violations_detected: u64,

    /// Total refactorings suggested
    pub refactorings_suggested: u64,

    /// Agent uptime seconds
    pub total_uptime_seconds: u64,
}

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
    pub async fn save(&self) -> Result<()> {
        let state = self.state.read().await;
        let json = serde_json::to_string_pretty(&*state)?;

        // Create parent directory if needed
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write atomically using temp file
        let temp_file = self.state_file.with_extension("tmp");
        fs::write(&temp_file, json).await?;
        fs::rename(&temp_file, &self.state_file).await?;

        Ok(())
    }

    /// Start auto-save task
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
    pub async fn add_project(&self, project: ProjectState) -> Result<()> {
        let mut state = self.state.write().await;
        state.monitored_projects.insert(project.id.clone(), project);
        state.last_updated = Utc::now();
        Ok(())
    }

    /// Remove monitored project
    pub async fn remove_project(&self, project_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.monitored_projects.remove(project_id);
        state.last_updated = Utc::now();
        Ok(())
    }

    /// Update project metrics
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
    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// Update statistics
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

impl Default for AgentState {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
            monitored_projects: HashMap::new(),
            quality_history: Vec::new(),
            config_overrides: HashMap::new(),
            statistics: AgentStatistics::default(),
        }
    }
}

impl AgentState {
    /// Convert to JSON string
    fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add a project
        let project = ProjectState {
            id: "test_project".to_string(),
            path: PathBuf::from("/test/path"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec!["*.rs".to_string()],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Save and verify
        persistence.save().await.unwrap();

        // Load from file
        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;

        assert!(state.monitored_projects.contains_key("test_project"));
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add project
        let project = ProjectState {
            id: "test".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Update metrics
        let metrics = QualityMetrics {
            avg_complexity: 5.5,
            max_complexity: 15,
            satd_count: 0,
            dead_code_percentage: 2.5,
            quality_score: 92.0,
            files_analyzed: 100,
            total_violations: 0,
        };

        persistence.update_metrics("test", metrics).await.unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.quality_history.len(), 1);
        assert_eq!(state.quality_history[0].project_id, "test");
    }
}
