//! Cache Strategy Orchestrator for PMAT
//!
//! This module provides a unified orchestrator that manages multiple caching strategies
//! and automatically selects the optimal strategy based on workload characteristics,
//! access patterns, and performance requirements.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  Cache Strategy Orchestrator                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Workload Analyzer  │  Strategy Router  │  Performance Monitor  │
//! ├─────────────────────┼───────────────────┼───────────────────────┤
//! │   Adaptive Cache    │   Multi-Tier      │    Predictive Cache   │
//! │   LRU/LFU/TTL      │   L1/L2/L3        │    ML Predictions     │
//! └─────────────────────┴───────────────────┴───────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Dynamic Strategy Selection**: Chooses optimal caching strategy per workload
//! - **Performance Monitoring**: Real-time cache effectiveness measurement
//! - **Adaptive Optimization**: Continuously improves based on usage patterns
//! - **Resource Management**: Intelligent memory and storage allocation
//! - **Fault Tolerance**: Graceful degradation and recovery

use crate::services::cache::{
    advanced_strategies::{CacheTier, EvictionPolicy},
};
use anyhow::Result;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Workload characteristics that influence cache strategy selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    /// Average request rate (requests/second)
    pub request_rate: f64,
    /// Working set size (bytes)
    pub working_set_size: u64,
    /// Temporal locality factor (0.0-1.0)
    pub temporal_locality: f64,
    /// Spatial locality factor (0.0-1.0)
    pub spatial_locality: f64,
    /// Read/write ratio
    pub read_write_ratio: f64,
    /// Cache hit rate target
    pub target_hit_rate: f64,
    /// Latency sensitivity (0.0-1.0)
    pub latency_sensitivity: f64,
}

/// Cache strategy recommendation from the orchestrator
#[derive(Debug, Clone)]
pub struct StrategyRecommendation {
    /// Recommended eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Recommended tier configuration
    pub tier_config: TierConfiguration,
    /// Expected performance improvement
    pub expected_improvement: f64,
    /// Confidence in recommendation (0.0-1.0)
    pub confidence: f64,
}

/// Configuration for cache tiers
#[derive(Debug, Clone)]
pub struct TierConfiguration {
    /// Memory allocation per tier (bytes)
    pub tier_allocations: FxHashMap<CacheTier, u64>,
    /// Enable/disable specific tiers
    pub enabled_tiers: FxHashMap<CacheTier, bool>,
    /// Inter-tier promotion thresholds
    pub promotion_thresholds: FxHashMap<CacheTier, f64>,
}

/// Performance metrics for strategy evaluation
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Current hit rate
    pub hit_rate: f64,
    /// Average latency
    pub avg_latency: Duration,
    /// Memory utilization
    pub memory_utilization: f64,
    /// Throughput (operations/second)
    pub throughput: f64,
    /// Cache effectiveness score
    pub effectiveness_score: f64,
}

/// Cache strategy orchestrator
pub struct CacheOrchestrator {
    /// Current workload profile
    workload_profile: RwLock<WorkloadProfile>,
    /// Active cache strategies
    strategies: RwLock<FxHashMap<String, Box<dyn CacheStrategy + Send + Sync>>>,
    /// Performance monitoring
    metrics: RwLock<PerformanceMetrics>,
    /// Strategy evaluation history
    evaluation_history: RwLock<Vec<StrategyEvaluation>>,
    /// Configuration
    config: OrchestratorConfig,
    /// Performance counters
    counters: PerformanceCounters,
}

/// Configuration for the cache orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Enable automatic strategy switching
    pub auto_strategy_switching: bool,
    /// Performance evaluation interval
    pub evaluation_interval: Duration,
    /// Minimum improvement threshold for strategy switch
    pub min_improvement_threshold: f64,
    /// Strategy evaluation window size
    pub evaluation_window: usize,
    /// Enable performance prediction
    pub enable_prediction: bool,
}

/// Historical strategy evaluation
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StrategyEvaluation {
    /// Performance achieved
    performance: PerformanceMetrics,
    /// Evaluation timestamp
    timestamp: Instant,
}

#[cfg(test)]
impl StrategyEvaluation {
    pub fn score(&self) -> f64 {
        self.performance.hit_rate * self.performance.throughput
    }
    
    pub fn is_valid(&self) -> bool {
        self.timestamp.elapsed().as_secs() < 3600
    }
}

/// Performance counters
#[derive(Debug)]
struct PerformanceCounters {
    strategy_switches: AtomicU64,
    evaluations_performed: AtomicU64,
    recommendations_generated: AtomicU64,
    performance_improvements: AtomicU64,
}

/// Trait for pluggable cache strategies (simplified for dynamic dispatch)
pub trait CacheStrategy {
    /// Get strategy identifier
    fn strategy_id(&self) -> &str;
    
    /// Get cache statistics
    fn get_stats(&self) -> PerformanceMetrics;
}

/// Configuration for individual cache strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Strategy-specific parameters
    pub parameters: FxHashMap<String, serde_json::Value>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Resource limits for cache strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: u64,
    /// Maximum disk usage (bytes)
    pub max_disk: u64,
    /// Maximum CPU usage (percentage)
    pub max_cpu: f64,
}

impl Default for WorkloadProfile {
    fn default() -> Self {
        Self {
            request_rate: 100.0,
            working_set_size: 100 * 1024 * 1024, // 100MB
            temporal_locality: 0.7,
            spatial_locality: 0.5,
            read_write_ratio: 0.8, // 80% reads
            target_hit_rate: 0.9,
            latency_sensitivity: 0.8,
        }
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            auto_strategy_switching: true,
            evaluation_interval: Duration::from_secs(300), // 5 minutes
            min_improvement_threshold: 0.05, // 5% improvement
            evaluation_window: 10,
            enable_prediction: true,
        }
    }
}

impl CacheOrchestrator {
    /// Create a new cache orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            workload_profile: RwLock::new(WorkloadProfile::default()),
            strategies: RwLock::new(FxHashMap::default()),
            metrics: RwLock::new(PerformanceMetrics::default()),
            evaluation_history: RwLock::new(Vec::new()),
            config,
            counters: PerformanceCounters {
                strategy_switches: AtomicU64::new(0),
                evaluations_performed: AtomicU64::new(0),
                recommendations_generated: AtomicU64::new(0),
                performance_improvements: AtomicU64::new(0),
            },
        }
    }

    /// Register a cache strategy
    pub fn register_strategy(&self, strategy: Box<dyn CacheStrategy + Send + Sync>) -> Result<()> {
        let strategy_id = strategy.strategy_id().to_string();
        self.strategies.write().insert(strategy_id.clone(), strategy);
        info!("Registered cache strategy: {}", strategy_id);
        Ok(())
    }

    /// Analyze workload and recommend optimal strategy
    pub async fn recommend_strategy(&self) -> Result<StrategyRecommendation> {
        let workload = self.workload_profile.read().clone();
        let recommendation = self.analyze_workload_and_recommend(&workload).await?;
        
        self.counters.recommendations_generated.fetch_add(1, Ordering::Relaxed);
        
        info!("Generated strategy recommendation: {:?}", recommendation.eviction_policy);
        Ok(recommendation)
    }

    /// Update workload profile based on current metrics
    pub async fn update_workload_profile(&self, new_profile: WorkloadProfile) -> Result<()> {
        {
            let mut current_profile = self.workload_profile.write();
            *current_profile = new_profile;
        } // Drop lock before await
        
        // If auto-switching is enabled, evaluate if we need a new strategy
        if self.config.auto_strategy_switching {
            self.evaluate_and_switch_if_needed().await?;
        }
        
        Ok(())
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().clone()
    }

    /// Run continuous optimization
    pub async fn run_optimization_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(self.config.evaluation_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.perform_optimization_cycle().await {
                warn!("Optimization cycle failed: {}", e);
            }
        }
    }

    /// Force strategy evaluation and potential switch
    pub async fn evaluate_and_switch_if_needed(&self) -> Result<()> {
        let recommendation = self.recommend_strategy().await?;
        let _current_metrics = self.get_performance_metrics();
        
        // Check if switching would provide significant improvement
        if recommendation.expected_improvement > self.config.min_improvement_threshold &&
           recommendation.confidence > 0.7 {
            info!("Switching cache strategy due to expected {:.2}% improvement", 
                  recommendation.expected_improvement * 100.0);
            
            self.switch_to_recommended_strategy(recommendation).await?;
            self.counters.strategy_switches.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(())
    }

    /// Get orchestrator statistics
    pub fn get_orchestrator_stats(&self) -> OrchestratorStats {
        OrchestratorStats {
            strategy_switches: self.counters.strategy_switches.load(Ordering::Relaxed),
            evaluations_performed: self.counters.evaluations_performed.load(Ordering::Relaxed),
            recommendations_generated: self.counters.recommendations_generated.load(Ordering::Relaxed),
            performance_improvements: self.counters.performance_improvements.load(Ordering::Relaxed),
            current_metrics: self.get_performance_metrics(),
            workload_profile: self.workload_profile.read().clone(),
        }
    }

    // Private methods

    async fn analyze_workload_and_recommend(&self, workload: &WorkloadProfile) -> Result<StrategyRecommendation> {
        let eviction_policy = self.select_eviction_policy(workload);
        let tier_config = self.configure_tiers(workload);
        let expected_improvement = self.estimate_improvement(workload, &eviction_policy).await?;
        let confidence = self.calculate_confidence(workload);

        Ok(StrategyRecommendation {
            eviction_policy,
            tier_config,
            expected_improvement,
            confidence,
        })
    }

    fn select_eviction_policy(&self, workload: &WorkloadProfile) -> EvictionPolicy {
        // Strategy selection logic based on workload characteristics
        if workload.temporal_locality > 0.8 {
            EvictionPolicy::LRU
        } else if workload.spatial_locality > 0.8 {
            EvictionPolicy::LFU
        } else if workload.latency_sensitivity > 0.9 {
            EvictionPolicy::TTL
        } else if workload.working_set_size > 1_000_000_000 { // > 1GB
            EvictionPolicy::Random // Avoid overhead for very large datasets
        } else {
            EvictionPolicy::Adaptive // Default to adaptive for balanced workloads
        }
    }

    fn configure_tiers(&self, workload: &WorkloadProfile) -> TierConfiguration {
        let mut tier_allocations = FxHashMap::default();
        let mut enabled_tiers = FxHashMap::default();
        let mut promotion_thresholds = FxHashMap::default();

        // Configure based on working set size and latency sensitivity
        if workload.latency_sensitivity > 0.8 {
            // Latency-sensitive: favor L1
            tier_allocations.insert(CacheTier::L1, workload.working_set_size / 2);
            tier_allocations.insert(CacheTier::L2, workload.working_set_size / 3);
            tier_allocations.insert(CacheTier::L3, workload.working_set_size / 6);
            promotion_thresholds.insert(CacheTier::L1, 0.3);
            promotion_thresholds.insert(CacheTier::L2, 0.6);
        } else {
            // Capacity-oriented: balanced approach
            tier_allocations.insert(CacheTier::L1, workload.working_set_size / 4);
            tier_allocations.insert(CacheTier::L2, workload.working_set_size / 2);
            tier_allocations.insert(CacheTier::L3, workload.working_set_size / 4);
            promotion_thresholds.insert(CacheTier::L1, 0.5);
            promotion_thresholds.insert(CacheTier::L2, 0.8);
        }

        // Enable all tiers by default
        enabled_tiers.insert(CacheTier::L1, true);
        enabled_tiers.insert(CacheTier::L2, true);
        enabled_tiers.insert(CacheTier::L3, true);

        TierConfiguration {
            tier_allocations,
            enabled_tiers,
            promotion_thresholds,
        }
    }

    async fn estimate_improvement(&self, _workload: &WorkloadProfile, _policy: &EvictionPolicy) -> Result<f64> {
        // Simplified improvement estimation
        // In practice, this would use ML models or historical data
        let current_metrics = self.get_performance_metrics();
        
        // Estimate based on current hit rate vs target
        let current_hit_rate = current_metrics.hit_rate;
        let target_hit_rate = 0.9;
        
        if current_hit_rate < target_hit_rate {
            Ok((target_hit_rate - current_hit_rate) * 0.5) // Conservative estimate
        } else {
            Ok(0.0)
        }
    }

    fn calculate_confidence(&self, workload: &WorkloadProfile) -> f64 {
        // Calculate confidence based on workload characteristics and historical data
        let evaluation_count = self.evaluation_history.read().len() as f64;
        let base_confidence = 0.5;
        
        // Higher confidence with more evaluation history
        let history_bonus = (evaluation_count / 100.0).min(0.3);
        
        // Higher confidence for well-defined workload characteristics
        let locality_bonus = (workload.temporal_locality + workload.spatial_locality) / 4.0;
        
        (base_confidence + history_bonus + locality_bonus).min(1.0)
    }

    async fn switch_to_recommended_strategy(&self, recommendation: StrategyRecommendation) -> Result<()> {
        // Implementation would switch to the recommended strategy
        // For now, just log the switch
        info!("Switching to strategy: {:?} with expected {:.2}% improvement",
              recommendation.eviction_policy, recommendation.expected_improvement * 100.0);
        Ok(())
    }

    async fn perform_optimization_cycle(&self) -> Result<()> {
        // Collect current metrics
        let current_metrics = self.collect_current_metrics().await?;
        *self.metrics.write() = current_metrics.clone();
        
        // Update workload profile based on metrics
        self.update_workload_from_metrics(&current_metrics).await?;
        
        // Evaluate if strategy change is needed
        if self.config.auto_strategy_switching {
            self.evaluate_and_switch_if_needed().await?;
        }
        
        // Record evaluation
        self.record_evaluation(&current_metrics).await?;
        
        self.counters.evaluations_performed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Completed optimization cycle");
        Ok(())
    }

    async fn collect_current_metrics(&self) -> Result<PerformanceMetrics> {
        // Collect metrics from active strategies
        let strategies = self.strategies.read();
        
        if strategies.is_empty() {
            return Ok(PerformanceMetrics::default());
        }

        // Aggregate metrics from all strategies
        let mut total_hit_rate = 0.0;
        let mut total_latency = Duration::ZERO;
        let mut total_throughput = 0.0;
        let mut count = 0;

        for strategy in strategies.values() {
            let metrics = strategy.get_stats();
            total_hit_rate += metrics.hit_rate;
            total_latency += metrics.avg_latency;
            total_throughput += metrics.throughput;
            count += 1;
        }

        if count > 0 {
            Ok(PerformanceMetrics {
                hit_rate: total_hit_rate / count as f64,
                avg_latency: total_latency / count as u32,
                memory_utilization: 0.0, // Would be calculated from actual usage
                throughput: total_throughput,
                effectiveness_score: total_hit_rate / count as f64, // Simplified
            })
        } else {
            Ok(PerformanceMetrics::default())
        }
    }

    async fn update_workload_from_metrics(&self, metrics: &PerformanceMetrics) -> Result<()> {
        let mut workload = self.workload_profile.write();
        
        // Update workload based on observed metrics
        workload.request_rate = metrics.throughput;
        
        // Adjust target hit rate based on current performance
        if metrics.hit_rate < workload.target_hit_rate {
            workload.latency_sensitivity = (workload.latency_sensitivity * 1.1).min(1.0);
        }
        
        Ok(())
    }

    async fn record_evaluation(&self, metrics: &PerformanceMetrics) -> Result<()> {
        let evaluation = StrategyEvaluation {
            performance: metrics.clone(),
            timestamp: Instant::now(),
        };
        
        let mut history = self.evaluation_history.write();
        history.push(evaluation);
        
        // Keep only recent evaluations
        if history.len() > self.config.evaluation_window {
            history.remove(0);
        }
        
        Ok(())
    }
}

/// Statistics from the cache orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorStats {
    pub strategy_switches: u64,
    pub evaluations_performed: u64,
    pub recommendations_generated: u64,
    pub performance_improvements: u64,
    pub current_metrics: PerformanceMetrics,
    pub workload_profile: WorkloadProfile,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct MockCacheStrategy {
        id: String,
        hit_rate: f64,
    }

    impl CacheStrategy for MockCacheStrategy {
        fn strategy_id(&self) -> &str {
            &self.id
        }

        fn get_stats(&self) -> PerformanceMetrics {
            PerformanceMetrics {
                hit_rate: self.hit_rate,
                avg_latency: Duration::from_millis(10),
                memory_utilization: 0.5,
                throughput: 100.0,
                effectiveness_score: self.hit_rate,
            }
        }
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = OrchestratorConfig::default();
        let orchestrator = CacheOrchestrator::new(config);
        
        let stats = orchestrator.get_orchestrator_stats();
        assert_eq!(stats.strategy_switches, 0);
    }

    #[tokio::test]
    async fn test_strategy_registration() -> Result<()> {
        let config = OrchestratorConfig::default();
        let orchestrator = CacheOrchestrator::new(config);
        
        let mock_strategy = Box::new(MockCacheStrategy {
            id: "test_strategy".to_string(),
            hit_rate: 0.8,
        });
        
        orchestrator.register_strategy(mock_strategy)?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_workload_analysis() -> Result<()> {
        let config = OrchestratorConfig::default();
        let orchestrator = CacheOrchestrator::new(config);
        
        let workload = WorkloadProfile {
            temporal_locality: 0.9, // High temporal locality should suggest LRU
            ..Default::default()
        };
        
        orchestrator.update_workload_profile(workload).await?;
        let recommendation = orchestrator.recommend_strategy().await?;
        
        assert_eq!(recommendation.eviction_policy, EvictionPolicy::LRU);
        
        Ok(())
    }

    #[test]
    fn test_tier_configuration() {
        let config = OrchestratorConfig::default();
        let orchestrator = CacheOrchestrator::new(config);
        
        let workload = WorkloadProfile {
            latency_sensitivity: 0.95, // High latency sensitivity
            working_set_size: 1_000_000,
            ..Default::default()
        };
        
        let tier_config = orchestrator.configure_tiers(&workload);
        
        // Should allocate more to L1 for latency-sensitive workloads
        let l1_allocation = tier_config.tier_allocations.get(&CacheTier::L1).unwrap();
        let l2_allocation = tier_config.tier_allocations.get(&CacheTier::L2).unwrap();
        
        assert!(l1_allocation > l2_allocation);
    }

    #[test]
    fn test_strategy_evaluation() {
        let workload = WorkloadProfile {
            temporal_locality: 0.8,
            spatial_locality: 0.6,
            latency_sensitivity: 0.9,
            ..Default::default()
        };
        
        let performance = PerformanceMetrics {
            hit_rate: 0.85,
            throughput: 1000.0,
            ..Default::default()
        };
        
        let evaluation = StrategyEvaluation {
            performance,
            timestamp: std::time::Instant::now(),
        };
        
        // Test score calculation
        let score = evaluation.score();
        assert!(score > 0.0, "Score should be positive for good performance");
        
        // Test validity check
        assert!(evaluation.is_valid(), "Fresh evaluation should be valid");
    }
}