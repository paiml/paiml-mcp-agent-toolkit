//! CachePredictor implementation
//!
//! Predictive cache warmer using ML-like patterns for cache optimization.

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Predictive cache warmer using ML-like patterns
pub struct CachePredictor<K>
where
    K: Clone + Eq + std::hash::Hash,
{
    /// Historical access sequences
    access_history: RwLock<VecDeque<K>>,
    /// Sequence patterns
    patterns: RwLock<FxHashMap<Vec<K>, f64>>,
    /// Prediction confidence threshold
    confidence_threshold: f64,
}

impl<K> CachePredictor<K>
where
    K: Clone + Eq + std::hash::Hash,
{
    #[must_use]
    pub fn new(confidence_threshold: f64) -> Self {
        Self {
            access_history: RwLock::new(VecDeque::new()),
            patterns: RwLock::new(FxHashMap::default()),
            confidence_threshold,
        }
    }

    pub fn record_access(&self, key: K) {
        let mut history = self.access_history.write();
        history.push_back(key);

        // Keep only recent history
        if history.len() > 1000 {
            history.pop_front();
        }

        // Update patterns
        self.update_patterns(&history);
    }

    pub fn predict_next(&self, current_sequence: &[K]) -> Vec<K> {
        debug_assert!(!current_sequence.is_empty(), "current_sequence must not be empty");
        let patterns = self.patterns.read();
        let mut predictions = Vec::new();

        for (pattern, confidence) in patterns.iter() {
            if *confidence > self.confidence_threshold
                && pattern.len() > current_sequence.len()
                && pattern.starts_with(current_sequence)
            {
                predictions.push(pattern[current_sequence.len()].clone());
            }
        }

        predictions
    }

    pub fn predict_value(&self, _key: &K) -> Option<()> {
        // Simplified prediction - in practice this would predict actual values
        None
    }

    fn update_patterns(&self, history: &VecDeque<K>) {
        let mut patterns = self.patterns.write();

        // Extract subsequences and update their frequencies
        for window_size in 2..=5.min(history.len()) {
            for window in history.iter().collect::<Vec<_>>().windows(window_size) {
                let pattern: Vec<K> = window.iter().map(|k| (*k).clone()).collect();
                *patterns.entry(pattern).or_insert(0.0) += 1.0;
            }
        }

        // Normalize frequencies
        let total_patterns = patterns.len() as f64;
        for confidence in patterns.values_mut() {
            *confidence /= total_patterns;
        }
    }

    /// Get access history length for testing
    #[cfg(test)]
    pub fn access_history_len(&self) -> usize {
        self.access_history.read().len()
    }
}
