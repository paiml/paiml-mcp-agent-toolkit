use crate::models::dag::DependencyGraph;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Analyzer for coupling metrics
pub struct CouplingAnalyzer;

/// Coupling metrics for a file or module
#[derive(Debug, Clone)]
pub struct CouplingMetrics {
    /// Number of modules that depend on this module (incoming dependencies)
    pub afferent_coupling: usize,
    /// Number of modules that this module depends on (outgoing dependencies)
    pub efferent_coupling: usize,
    /// Instability metric (efferent / (afferent + efferent))
    pub instability: f64,
}

/// Report containing coupling analysis results
pub struct CouplingReport {
    /// Coupling metrics for each file
    pub file_metrics: HashMap<PathBuf, CouplingMetrics>,
    /// Overall project coupling metrics
    pub project_metrics: ProjectCouplingMetrics,
}

/// Project-level coupling metrics
pub struct ProjectCouplingMetrics {
    /// Average afferent coupling
    pub avg_afferent: f64,
    /// Average efferent coupling
    pub avg_efferent: f64,
    /// Maximum afferent coupling
    pub max_afferent: usize,
    /// Maximum efferent coupling
    pub max_efferent: usize,
}

impl CouplingAnalyzer {
    /// Create a new coupling analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze coupling in a dependency graph
    pub async fn analyze(&self, graph: &DependencyGraph) -> Result<CouplingReport> {
        let mut file_metrics = HashMap::new();

        // Calculate coupling for each node
        for (node_id, node_info) in &graph.nodes {
            let path = PathBuf::from(&node_info.file_path);

            // Calculate in-degree (afferent coupling)
            let afferent = graph.edges.iter().filter(|e| &e.to == node_id).count();

            // Calculate out-degree (efferent coupling)
            let efferent = graph.edges.iter().filter(|e| &e.from == node_id).count();

            let total = afferent + efferent;
            let instability = if total > 0 {
                efferent as f64 / total as f64
            } else {
                0.0
            };

            file_metrics.insert(
                path,
                CouplingMetrics {
                    afferent_coupling: afferent,
                    efferent_coupling: efferent,
                    instability,
                },
            );
        }

        // Calculate project-level metrics
        let project_metrics = self.calculate_project_metrics(&file_metrics);

        Ok(CouplingReport {
            file_metrics,
            project_metrics,
        })
    }

    /// Extract file path from node key
    #[allow(dead_code)]
    fn extract_file_path(node_key: &str) -> Option<PathBuf> {
        // Simple extraction - assumes node key contains file path
        if node_key.contains("::") {
            // Format: "file_path::module_name"
            node_key.split("::").next().map(PathBuf::from)
        } else {
            // Direct file path
            Some(PathBuf::from(node_key))
        }
    }

    /// Calculate project-level metrics
    fn calculate_project_metrics(
        &self,
        file_metrics: &HashMap<PathBuf, CouplingMetrics>,
    ) -> ProjectCouplingMetrics {
        if file_metrics.is_empty() {
            return ProjectCouplingMetrics {
                avg_afferent: 0.0,
                avg_efferent: 0.0,
                max_afferent: 0,
                max_efferent: 0,
            };
        }

        let mut total_afferent = 0;
        let mut total_efferent = 0;
        let mut max_afferent = 0;
        let mut max_efferent = 0;

        for metrics in file_metrics.values() {
            total_afferent += metrics.afferent_coupling;
            total_efferent += metrics.efferent_coupling;
            max_afferent = max_afferent.max(metrics.afferent_coupling);
            max_efferent = max_efferent.max(metrics.efferent_coupling);
        }

        let count = file_metrics.len() as f64;
        ProjectCouplingMetrics {
            avg_afferent: total_afferent as f64 / count,
            avg_efferent: total_efferent as f64 / count,
            max_afferent,
            max_efferent,
        }
    }
}

impl Default for CouplingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
