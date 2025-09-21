// Centrality metrics computation
// Placeholder - will implement in Sprint 2

use super::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CentralityMetrics {
    pub degree: Vec<f64>,
    pub betweenness: Vec<f64>,
    pub closeness: Vec<f64>,
    pub eigenvector: Vec<f64>,
    pub katz: Vec<f64>,
    pub harmonic: Vec<f64>,
}

pub struct CentralityComputer {
    pub normalize: bool,
    pub weighted: bool,
}

impl CentralityComputer {
    pub fn new(normalize: bool, weighted: bool) -> Self {
        CentralityComputer {
            normalize,
            weighted,
        }
    }

    pub fn compute_all(&self, _graph: &DependencyGraph) -> CentralityMetrics {
        todo!("Implement in Sprint 2")
    }
}