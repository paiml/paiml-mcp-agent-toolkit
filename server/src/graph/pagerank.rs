// PageRank algorithm implementation
// Following Google's original algorithm with power iteration
// Complexity: All functions ≤ 9

use super::*;

pub struct PageRankComputer {
    pub damping: f64,
    pub tolerance: f64,
    pub max_iterations: usize,
}

impl Default for PageRankComputer {
    fn default() -> Self {
        PageRankComputer {
            damping: 0.85,
            tolerance: 1e-6,
            max_iterations: 100,
        }
    }
}

impl PageRankComputer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_damping(mut self, damping: f64) -> Self {
        self.damping = damping;
        self
    }

    /// Compute PageRank using power iteration
    /// Complexity: 9 (initialization + iteration loop)
    pub fn compute(&self, matrices: &GraphMatrices) -> Vec<f64> {
        let n = matrices.node_count;
        if n == 0 {
            return Vec::new();
        }

        // Initialize PageRank values uniformly
        let mut pr = vec![1.0 / n as f64; n];
        let mut new_pr = vec![0.0; n];

        // Compute out-degree for each node
        let out_degrees = self.compute_out_degrees(matrices);

        // Power iteration
        for _ in 0..self.max_iterations {
            // Store old values for convergence check
            let old_pr = pr.clone();

            // Compute new PageRank values
            self.iterate_pagerank(&pr, &mut new_pr, &out_degrees, matrices);

            // Swap vectors
            std::mem::swap(&mut pr, &mut new_pr);

            // Check convergence
            if self.has_converged(&pr, &old_pr) {
                break;
            }
        }

        pr
    }

    /// Compute out-degree for each node
    /// Complexity: 3
    fn compute_out_degrees(&self, matrices: &GraphMatrices) -> Vec<usize> {
        let mut out_degrees = vec![0; matrices.node_count];

        for (from, _to, _weight) in &matrices.edges {
            out_degrees[*from] += 1;
        }

        out_degrees
    }

    /// Single PageRank iteration
    /// Complexity: 8
    fn iterate_pagerank(
        &self,
        pr: &[f64],
        new_pr: &mut [f64],
        out_degrees: &[usize],
        matrices: &GraphMatrices,
    ) {
        let n = pr.len();

        // Reset new PageRank values
        for i in 0..n {
            new_pr[i] = (1.0 - self.damping) / n as f64;
        }

        // Sum of dangling node contributions
        let mut dangling_sum = 0.0;
        for i in 0..n {
            if out_degrees[i] == 0 {
                dangling_sum += pr[i];
            }
        }
        dangling_sum *= self.damping / n as f64;

        // Add dangling node contribution to all nodes
        for i in 0..n {
            new_pr[i] += dangling_sum;
        }

        // Add edge contributions
        for (from, to, weight) in &matrices.edges {
            if out_degrees[*from] > 0 {
                // Normalize by out-degree and apply damping
                let contribution = self.damping * pr[*from] * weight / out_degrees[*from] as f64;
                new_pr[*to] += contribution;
            }
        }
    }

    /// Check convergence using L1 norm
    /// Complexity: 3
    fn has_converged(&self, pr: &[f64], old_pr: &[f64]) -> bool {
        let mut diff = 0.0;

        for i in 0..pr.len() {
            diff += (pr[i] - old_pr[i]).abs();
        }

        diff < self.tolerance
    }
}