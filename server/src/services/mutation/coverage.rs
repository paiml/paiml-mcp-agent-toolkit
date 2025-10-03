//! Coverage Tracking for Mutation Testing - Phase 4.1
//!
//! EXTREME TDD: Coverage instrumentation and tracking

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Coverage information for a single execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoverageInfo {
    /// Lines of code executed
    pub lines_covered: HashSet<u32>,

    /// Basic blocks executed
    pub blocks_covered: HashSet<u64>,

    /// Branches taken
    pub branches_taken: HashSet<(u64, u64)>,

    /// Total execution count (for edge coverage)
    pub execution_count: u64,
}

impl CoverageInfo {
    /// Create new empty coverage info
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate coverage percentage
    pub fn coverage_percentage(&self, total_lines: u32) -> f64 {
        if total_lines == 0 {
            return 0.0;
        }
        (self.lines_covered.len() as f64 / total_lines as f64) * 100.0
    }

    /// Check if this coverage is a superset of another
    pub fn covers(&self, other: &CoverageInfo) -> bool {
        other.lines_covered.is_subset(&self.lines_covered)
            && other.blocks_covered.is_subset(&self.blocks_covered)
            && other.branches_taken.is_subset(&self.branches_taken)
    }

    /// Calculate coverage increase from baseline
    pub fn coverage_increase(&self, baseline: &CoverageInfo) -> f64 {
        let new_lines = self.lines_covered.difference(&baseline.lines_covered).count();
        let new_blocks = self.blocks_covered.difference(&baseline.blocks_covered).count();
        let new_branches = self.branches_taken.difference(&baseline.branches_taken).count();

        // Simple weighted average
        let total_new = (new_lines * 2 + new_blocks * 3 + new_branches * 5) as f64;
        let baseline_size = (baseline.lines_covered.len() * 2
            + baseline.blocks_covered.len() * 3
            + baseline.branches_taken.len() * 5) as f64;

        if baseline_size == 0.0 {
            return if total_new > 0.0 { 1.0 } else { 0.0 };
        }

        total_new / baseline_size
    }

    /// Merge with another coverage info
    pub fn merge(&mut self, other: &CoverageInfo) {
        self.lines_covered.extend(&other.lines_covered);
        self.blocks_covered.extend(&other.blocks_covered);
        self.branches_taken.extend(&other.branches_taken);
        self.execution_count += other.execution_count;
    }

    /// Check if this is "interesting" (adds new coverage)
    pub fn is_interesting(&self, corpus: &[CoverageInfo]) -> bool {
        for existing in corpus {
            if existing.covers(self) {
                return false; // This coverage is already subsumed
            }
        }
        true
    }
}

/// Coverage-guided corpus for fuzzing
#[derive(Debug, Clone, Default)]
pub struct CoverageCorpus {
    /// Inputs that discovered new coverage
    pub interesting_inputs: Vec<(Vec<u8>, CoverageInfo)>,

    /// Baseline coverage without fuzzing
    pub baseline: CoverageInfo,

    /// Maximum coverage achieved
    pub max_coverage: CoverageInfo,
}

impl CoverageCorpus {
    /// Create new corpus with baseline coverage
    pub fn new(baseline: CoverageInfo) -> Self {
        Self {
            interesting_inputs: Vec::new(),
            baseline: baseline.clone(),
            max_coverage: baseline,
        }
    }

    /// Add input if it discovers new coverage
    pub fn add_if_interesting(&mut self, input: Vec<u8>, coverage: CoverageInfo) -> bool {
        if coverage.is_interesting(&self.interesting_inputs.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>()) {
            self.max_coverage.merge(&coverage);
            self.interesting_inputs.push((input, coverage));
            true
        } else {
            false
        }
    }

    /// Get total coverage increase from baseline
    pub fn total_coverage_increase(&self) -> f64 {
        self.max_coverage.coverage_increase(&self.baseline)
    }

    /// Get most interesting inputs for mutation
    pub fn get_seeds(&self, count: usize) -> Vec<Vec<u8>> {
        self.interesting_inputs
            .iter()
            .take(count)
            .map(|(input, _)| input.clone())
            .collect()
    }
}

/// Instrumentation point for coverage tracking
#[derive(Debug, Clone, Copy)]
pub struct InstrumentationPoint {
    pub line: u32,
    pub block_id: u64,
}

/// Simple coverage tracker (simulated for Phase 1)
#[derive(Debug, Default)]
pub struct CoverageTracker {
    current: CoverageInfo,
}

impl CoverageTracker {
    /// Create new coverage tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record line execution
    pub fn record_line(&mut self, line: u32) {
        self.current.lines_covered.insert(line);
    }

    /// Record block execution
    pub fn record_block(&mut self, block_id: u64) {
        self.current.blocks_covered.insert(block_id);
    }

    /// Record branch taken
    pub fn record_branch(&mut self, from: u64, to: u64) {
        self.current.branches_taken.insert((from, to));
    }

    /// Get current coverage and reset
    pub fn take_coverage(&mut self) -> CoverageInfo {
        std::mem::take(&mut self.current)
    }

    /// Simulate coverage for given input (Phase 1)
    pub fn simulate_coverage(input: &[u8]) -> CoverageInfo {
        let mut coverage = CoverageInfo::new();

        // Simulate coverage based on input characteristics
        let len = input.len();

        // Basic coverage: lines 1-10
        for line in 1..=10 {
            coverage.lines_covered.insert(line);
        }

        // Additional coverage based on input length
        if len > 10 {
            for line in 11..=20 {
                coverage.lines_covered.insert(line);
            }
        }

        if len > 50 {
            for line in 21..=30 {
                coverage.lines_covered.insert(line);
            }
        }

        if len > 100 {
            for line in 31..=40 {
                coverage.lines_covered.insert(line);
            }
        }

        // Simulate blocks based on first bytes
        for (idx, &byte) in input.iter().take(5).enumerate() {
            coverage.blocks_covered.insert(byte as u64 + idx as u64 * 256);
        }

        // Simulate branches based on patterns
        if input.contains(&0xAA) {
            coverage.branches_taken.insert((1, 2));
        }
        if input.contains(&0xBB) {
            coverage.branches_taken.insert((2, 3));
        }
        if input.contains(&0xCC) {
            coverage.branches_taken.insert((3, 4));
        }

        coverage.execution_count = 1;
        coverage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_info_creation() {
        let coverage = CoverageInfo::new();
        assert_eq!(coverage.lines_covered.len(), 0);
        assert_eq!(coverage.blocks_covered.len(), 0);
    }

    #[test]
    fn test_coverage_percentage() {
        let mut coverage = CoverageInfo::new();
        coverage.lines_covered.insert(1);
        coverage.lines_covered.insert(2);
        coverage.lines_covered.insert(3);

        assert_eq!(coverage.coverage_percentage(10), 30.0);
    }

    #[test]
    fn test_coverage_increase() {
        let mut baseline = CoverageInfo::new();
        baseline.lines_covered.insert(1);
        baseline.lines_covered.insert(2);

        let mut new_coverage = CoverageInfo::new();
        new_coverage.lines_covered.insert(1);
        new_coverage.lines_covered.insert(2);
        new_coverage.lines_covered.insert(3);
        new_coverage.lines_covered.insert(4);

        let increase = new_coverage.coverage_increase(&baseline);
        assert!(increase > 0.0);
    }

    #[test]
    fn test_coverage_merge() {
        let mut coverage1 = CoverageInfo::new();
        coverage1.lines_covered.insert(1);
        coverage1.lines_covered.insert(2);

        let mut coverage2 = CoverageInfo::new();
        coverage2.lines_covered.insert(3);
        coverage2.lines_covered.insert(4);

        coverage1.merge(&coverage2);
        assert_eq!(coverage1.lines_covered.len(), 4);
    }

    #[test]
    fn test_coverage_corpus() {
        let baseline = CoverageInfo::new();
        let mut corpus = CoverageCorpus::new(baseline);

        let input1 = vec![1, 2, 3];
        let mut cov1 = CoverageInfo::new();
        cov1.lines_covered.insert(1);

        let added = corpus.add_if_interesting(input1, cov1);
        assert!(added);
        assert_eq!(corpus.interesting_inputs.len(), 1);
    }

    #[test]
    fn test_simulate_coverage_length() {
        let short_input = vec![0; 5];
        let long_input = vec![0; 150];

        let short_cov = CoverageTracker::simulate_coverage(&short_input);
        let long_cov = CoverageTracker::simulate_coverage(&long_input);

        assert!(long_cov.lines_covered.len() > short_cov.lines_covered.len());
    }

    #[test]
    fn test_simulate_coverage_patterns() {
        let input_with_pattern = vec![0xAA, 0xBB, 0xCC];
        let input_without_pattern = vec![0x00, 0x11, 0x22];

        let cov_with = CoverageTracker::simulate_coverage(&input_with_pattern);
        let cov_without = CoverageTracker::simulate_coverage(&input_without_pattern);

        assert!(cov_with.branches_taken.len() > cov_without.branches_taken.len());
    }
}
