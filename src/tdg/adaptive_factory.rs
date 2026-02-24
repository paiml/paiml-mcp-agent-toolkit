/// Factory for creating adaptive threshold managers
pub struct AdaptiveThresholdFactory;

impl AdaptiveThresholdFactory {
    /// Create manager with default configuration
    #[must_use]
    pub fn create_default() -> AdaptiveThresholdManager {
        AdaptiveThresholdManager::new(AdaptiveConfig::default())
    }

    /// Create manager optimized for development (fast feedback)
    #[must_use]
    pub fn create_dev_optimized() -> AdaptiveThresholdManager {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 50, // Faster target for dev
            sample_window_size: 20,      // Smaller window for quicker adaptation
            adjustment_sensitivity: 0.2, // More aggressive adjustments
            ..Default::default()
        };
        AdaptiveThresholdManager::new(config)
    }

    /// Create manager optimized for production (stable)
    #[must_use]
    pub fn create_prod_optimized() -> AdaptiveThresholdManager {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 200, // More conservative target
            sample_window_size: 100,      // Larger window for stability
            adjustment_sensitivity: 0.05, // Conservative adjustments
            ..Default::default()
        };
        AdaptiveThresholdManager::new(config)
    }
}
