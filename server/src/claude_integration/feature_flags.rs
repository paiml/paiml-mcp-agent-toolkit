// Feature flags for progressive rollout of Claude integration
// Implements canary deployments and kill switch

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Rollout strategy for feature flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolloutStrategy {
    /// Feature is disabled
    Disabled,
    /// Feature is enabled for allowlisted items only
    Allowlist,
    /// Feature is enabled for a percentage (0-100)
    Percentage(u32),
    /// Feature is fully enabled
    FullRollout,
}

/// Feature flags for Claude integration
pub struct FeatureFlags {
    /// Master kill switch
    enabled: AtomicBool,

    /// Rollout percentage (0-100)
    rollout_percentage: AtomicU32,

    /// Allowlist for specific identifiers
    allowlist: Arc<parking_lot::RwLock<HashSet<String>>>,

    /// Maximum latency before auto-rollback (milliseconds)
    max_latency_ms: AtomicU32,

    /// Current rollout strategy
    strategy: Arc<parking_lot::RwLock<RolloutStrategy>>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureFlags {
    /// Create new feature flags with conservative defaults
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false), // Default: disabled
            rollout_percentage: AtomicU32::new(0),
            allowlist: Arc::new(parking_lot::RwLock::new(HashSet::new())),
            max_latency_ms: AtomicU32::new(5000), // 5 second default
            strategy: Arc::new(parking_lot::RwLock::new(RolloutStrategy::Disabled)),
        }
    }

    /// Check if feature should be enabled for a given identifier
    pub fn should_use_claude(&self, identifier: &str) -> bool {
        // Kill switch check
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }

        let strategy = *self.strategy.read();

        match strategy {
            RolloutStrategy::Disabled => false,
            RolloutStrategy::Allowlist => self.allowlist.read().contains(identifier),
            RolloutStrategy::Percentage(pct) => {
                // Consistent hashing for stable routing
                let hash = self.hash_identifier(identifier);
                (hash % 100) < pct as u64
            }
            RolloutStrategy::FullRollout => true,
        }
    }

    /// Enable the feature (master switch)
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    /// Disable the feature (kill switch)
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    /// Set rollout strategy
    pub fn set_strategy(&self, strategy: RolloutStrategy) {
        *self.strategy.write() = strategy;

        // Update related settings
        match strategy {
            RolloutStrategy::Disabled => {
                self.rollout_percentage.store(0, Ordering::Release);
            }
            RolloutStrategy::Percentage(pct) => {
                self.rollout_percentage.store(pct, Ordering::Release);
            }
            RolloutStrategy::FullRollout => {
                self.rollout_percentage.store(100, Ordering::Release);
            }
            _ => {}
        }
    }

    /// Add identifier to allowlist
    pub fn add_to_allowlist(&self, identifier: impl Into<String>) {
        self.allowlist.write().insert(identifier.into());
    }

    /// Remove identifier from allowlist
    pub fn remove_from_allowlist(&self, identifier: &str) {
        self.allowlist.write().remove(identifier);
    }

    /// Set maximum latency threshold
    pub fn set_max_latency(&self, latency_ms: u32) {
        self.max_latency_ms.store(latency_ms, Ordering::Release);
    }

    /// Automatically rollback if latency exceeds threshold
    pub fn auto_rollback_on_degradation(&self, current_latency_ms: u32) -> bool {
        let max = self.max_latency_ms.load(Ordering::Relaxed);

        if current_latency_ms > max {
            tracing::warn!(
                "Performance degradation detected: {}ms > {}ms, disabling Claude integration",
                current_latency_ms,
                max
            );
            self.disable();
            true
        } else {
            false
        }
    }

    /// Get current rollout percentage
    pub fn get_percentage(&self) -> u32 {
        self.rollout_percentage.load(Ordering::Relaxed)
    }

    /// Get current strategy
    pub fn get_strategy(&self) -> RolloutStrategy {
        *self.strategy.read()
    }

    /// Check if enabled (master switch)
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Hash identifier for consistent routing
    fn hash_identifier(&self, identifier: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        identifier.hash(&mut hasher);
        hasher.finish()
    }

    /// Get allowlist size
    pub fn allowlist_size(&self) -> usize {
        self.allowlist.read().len()
    }
}

/// Builder for feature flags configuration
pub struct FeatureFlagsBuilder {
    enabled: bool,
    strategy: RolloutStrategy,
    allowlist: HashSet<String>,
    max_latency_ms: u32,
}

impl Default for FeatureFlagsBuilder {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: RolloutStrategy::Disabled,
            allowlist: HashSet::new(),
            max_latency_ms: 5000,
        }
    }
}

impl FeatureFlagsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn strategy(mut self, strategy: RolloutStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn add_to_allowlist(mut self, identifier: impl Into<String>) -> Self {
        self.allowlist.insert(identifier.into());
        self
    }

    pub fn max_latency(mut self, latency_ms: u32) -> Self {
        self.max_latency_ms = latency_ms;
        self
    }

    pub fn build(self) -> FeatureFlags {
        let flags = FeatureFlags::new();

        if self.enabled {
            flags.enable();
        }

        flags.set_strategy(self.strategy);
        flags.set_max_latency(self.max_latency_ms);

        for id in self.allowlist {
            flags.add_to_allowlist(id);
        }

        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_default_disabled() {
        let flags = FeatureFlags::new();
        assert!(!flags.is_enabled());
        assert!(!flags.should_use_claude("test"));
    }

    #[test]
    fn test_enable_disable() {
        let flags = FeatureFlags::new();

        flags.enable();
        assert!(flags.is_enabled());

        flags.disable();
        assert!(!flags.is_enabled());
    }

    #[test]
    fn test_allowlist_strategy() {
        let flags = FeatureFlags::new();
        flags.enable();
        flags.set_strategy(RolloutStrategy::Allowlist);

        // Not in allowlist
        assert!(!flags.should_use_claude("user1"));

        // Add to allowlist
        flags.add_to_allowlist("user1");
        assert!(flags.should_use_claude("user1"));

        // Remove from allowlist
        flags.remove_from_allowlist("user1");
        assert!(!flags.should_use_claude("user1"));
    }

    #[test]
    fn test_percentage_strategy() {
        let flags = FeatureFlags::new();
        flags.enable();
        flags.set_strategy(RolloutStrategy::Percentage(50));

        // Test consistent hashing
        let id1 = "test_user_1";
        let result1 = flags.should_use_claude(id1);
        let result2 = flags.should_use_claude(id1);
        assert_eq!(result1, result2); // Same ID always gets same result

        // Test that approximately 50% are enabled
        let mut enabled_count = 0;
        for i in 0..1000 {
            if flags.should_use_claude(&format!("user_{}", i)) {
                enabled_count += 1;
            }
        }

        // Should be roughly 50% (allow 10% variance)
        assert!(
            enabled_count > 450 && enabled_count < 550,
            "Expected ~500, got {}",
            enabled_count
        );
    }

    #[test]
    fn test_full_rollout_strategy() {
        let flags = FeatureFlags::new();
        flags.enable();
        flags.set_strategy(RolloutStrategy::FullRollout);

        assert!(flags.should_use_claude("any_user"));
        assert!(flags.should_use_claude("another_user"));
    }

    #[test]
    fn test_auto_rollback() {
        let flags = FeatureFlags::new();
        flags.enable();
        flags.set_max_latency(1000);

        // Low latency - no rollback
        assert!(!flags.auto_rollback_on_degradation(500));
        assert!(flags.is_enabled());

        // High latency - triggers rollback
        assert!(flags.auto_rollback_on_degradation(2000));
        assert!(!flags.is_enabled());
    }

    #[test]
    fn test_builder() {
        let flags = FeatureFlagsBuilder::new()
            .enabled(true)
            .strategy(RolloutStrategy::Percentage(25))
            .add_to_allowlist("special_user")
            .max_latency(3000)
            .build();

        assert!(flags.is_enabled());
        assert_eq!(flags.get_percentage(), 25);
        assert_eq!(flags.allowlist_size(), 1);
    }

    #[test]
    fn test_kill_switch_overrides_all() {
        let flags = FeatureFlags::new();
        flags.set_strategy(RolloutStrategy::FullRollout);
        flags.add_to_allowlist("user1");

        // Even with full rollout, disabled switch prevents usage
        flags.disable();
        assert!(!flags.should_use_claude("user1"));
        assert!(!flags.should_use_claude("any_user"));
    }
}
