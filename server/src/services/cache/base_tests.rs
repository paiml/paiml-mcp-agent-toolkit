//! Tests for cache base module

use super::*;
use std::time::Duration;

#[test]
fn test_cache_entry_creation() {
    let value = "test_value".to_string();
    let entry = CacheEntry::new(value.clone(), 100);

    assert_eq!(*entry.value, value);
    assert_eq!(entry.size_bytes, 100);
    assert_eq!(
        entry
            .access_count
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_cache_entry_access() {
    let entry = CacheEntry::new("test".to_string(), 50);

    // Initial access count should be 0
    assert_eq!(
        entry
            .access_count
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Access the entry
    entry.access();
    assert_eq!(
        entry
            .access_count
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // Access again
    entry.access();
    assert_eq!(
        entry
            .access_count
            .load(std::sync::atomic::Ordering::Relaxed),
        2
    );
}

#[test]
fn test_cache_entry_age() {
    let entry = CacheEntry::new("test".to_string(), 50);

    // Age should be very small right after creation
    let age = entry.age();
    assert!(age < Duration::from_secs(1));

    // Test that age calculation works
    std::thread::sleep(Duration::from_millis(10));
    let age2 = entry.age();
    assert!(age2 >= Duration::from_millis(10));
}

#[test]
fn test_cache_stats_new() {
    let stats = CacheStats::new();

    assert_eq!(stats.hits.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(stats.misses.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(
        stats.total_bytes.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats.evictions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_cache_stats_operations() {
    let stats = CacheStats::new();

    // Test hit recording
    stats.record_hit();
    assert_eq!(stats.hits.load(std::sync::atomic::Ordering::Relaxed), 1);

    // Test miss recording
    stats.record_miss();
    assert_eq!(stats.misses.load(std::sync::atomic::Ordering::Relaxed), 1);

    // Test eviction recording
    stats.record_eviction();
    assert_eq!(
        stats.evictions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // Test bytes operations
    stats.add_bytes(100);
    assert_eq!(
        stats.total_bytes.load(std::sync::atomic::Ordering::Relaxed),
        100
    );

    stats.remove_bytes(30);
    assert_eq!(
        stats.total_bytes.load(std::sync::atomic::Ordering::Relaxed),
        70
    );
}

struct TestStrategy;

impl CacheStrategy for TestStrategy {
    type Key = String;
    type Value = String;

    fn cache_key(&self, input: &Self::Key) -> String {
        format!("test_{}", input)
    }

    fn validate(&self, _key: &Self::Key, _value: &Self::Value) -> bool {
        true
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(60))
    }

    fn max_size(&self) -> usize {
        1000
    }
}

#[test]
fn test_cache_strategy_trait() {
    let strategy = TestStrategy;

    assert_eq!(strategy.cache_key(&"key".to_string()), "test_key");
    assert!(strategy.validate(&"key".to_string(), &"value".to_string()));
    assert_eq!(strategy.ttl(), Some(Duration::from_secs(60)));
    assert_eq!(strategy.max_size(), 1000);
}
