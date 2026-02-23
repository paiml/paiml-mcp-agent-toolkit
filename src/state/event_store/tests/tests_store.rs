use crate::state::event_store::store;
use crate::state::event_store::*;
use crate::state::*;
use std::collections::BTreeMap;

#[test]
fn test_event_store_config_default() {
    let config = EventStoreConfig::default();
    assert_eq!(config.max_events_in_memory, 100_000);
    assert_eq!(config.compaction_threshold, 10_000);
    assert!(config.persistence_enabled);
    assert!(!config.sync_writes);
    assert_eq!(config.batch_size, 1000);
}

#[test]
fn test_event_store_config_clone() {
    let config = EventStoreConfig {
        max_events_in_memory: 5000,
        compaction_threshold: 1000,
        persistence_enabled: false,
        sync_writes: true,
        batch_size: 100,
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_events_in_memory, 5000);
    assert_eq!(cloned.compaction_threshold, 1000);
    assert!(!cloned.persistence_enabled);
    assert!(cloned.sync_writes);
}

#[test]
fn test_compaction_result_default() {
    let result = CompactionResult::default();
    assert_eq!(result.events_before, 0);
    assert_eq!(result.events_after, 0);
    assert_eq!(result.bytes_saved, 0);
    assert_eq!(result.duration, std::time::Duration::ZERO);
}

#[test]
fn test_event_store_stats_clone() {
    let stats = EventStoreStats {
        total_events: 100,
        total_partitions: 5,
        next_event_id: 101,
        memory_usage_bytes: 50000,
    };
    let cloned = stats.clone();
    assert_eq!(cloned.total_events, 100);
    assert_eq!(cloned.total_partitions, 5);
    assert_eq!(cloned.next_event_id, 101);
    assert_eq!(cloned.memory_usage_bytes, 50000);
}

#[test]
fn test_event_store_error_display() {
    let persistence_err = EventStoreError::PersistenceError("disk full".to_string());
    assert!(persistence_err.to_string().contains("disk full"));

    let serialization_err = EventStoreError::SerializationError("invalid format".to_string());
    assert!(serialization_err.to_string().contains("invalid format"));

    let corrupted_err = EventStoreError::CorruptedData("checksum mismatch".to_string());
    assert!(corrupted_err.to_string().contains("checksum mismatch"));

    let not_found_err = EventStoreError::EventNotFound(42);
    assert!(not_found_err.to_string().contains("42"));
}

#[test]
fn test_estimate_memory_usage() {
    let events: BTreeMap<EventId, StateEvent> = BTreeMap::new();
    let usage = store::estimate_memory_usage_for_test(&events);
    assert_eq!(usage, 0);

    let mut events_map: BTreeMap<EventId, StateEvent> = BTreeMap::new();
    events_map.insert(
        1,
        StateEvent::new("p".to_string(), "e".to_string(), serde_json::json!({})),
    );
    let usage2 = store::estimate_memory_usage_for_test(&events_map);
    assert!(usage2 > 0);
}

#[tokio::test]
async fn test_event_append_and_retrieve() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    let event = StateEvent::new(
        "partition1".to_string(),
        "test_event".to_string(),
        serde_json::json!({"data": "test"}),
    );

    let id = store.append(event.clone()).await.expect("internal error");
    assert_eq!(id, 1);

    let retrieved = store.get_event(id).expect("internal error");
    assert_eq!(retrieved.partition_key, "partition1");
    assert_eq!(retrieved.event_type, "test_event");
}

#[tokio::test]
async fn test_batch_append() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    let events = vec![
        StateEvent::new("p1".to_string(), "e1".to_string(), serde_json::json!({})),
        StateEvent::new("p1".to_string(), "e2".to_string(), serde_json::json!({})),
        StateEvent::new("p2".to_string(), "e3".to_string(), serde_json::json!({})),
    ];

    let ids = store.append_batch(events).await.expect("internal error");
    assert_eq!(ids, vec![1, 2, 3]);

    let p1_events = store.get_partition_events("p1", None);
    assert_eq!(p1_events.len(), 2);

    let p2_events = store.get_partition_events("p2", None);
    assert_eq!(p2_events.len(), 1);
}

#[tokio::test]
async fn test_get_events_since() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    for i in 0..10 {
        let event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({"index": i}),
        );
        store.append(event).await.expect("internal error");
    }

    let events = store.get_events_since(5, Some(3));
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type, "event_5");
    assert_eq!(events[1].event_type, "event_6");
    assert_eq!(events[2].event_type, "event_7");
}

#[tokio::test]
async fn test_memory_limit_enforcement() {
    let config = EventStoreConfig {
        max_events_in_memory: 5,
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    for i in 0..10 {
        let event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({"index": i}),
        );
        store.append(event).await.expect("internal error");
    }

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 5); // Only 5 events in memory
    assert_eq!(stats.next_event_id, 11); // But ID counter continues
}

#[tokio::test]
async fn test_get_latest_event_id() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    // Initially should be 0 (1 - 1)
    let initial_id = store.get_latest_event_id();
    assert_eq!(initial_id, 0);

    // Add some events
    for i in 0..5 {
        let event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({}),
        );
        store.append(event).await.expect("internal error");
    }

    let latest_id = store.get_latest_event_id();
    assert_eq!(latest_id, 5);
}

#[tokio::test]
async fn test_get_partition_events_with_since() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    for i in 0..5 {
        let event = StateEvent::new(
            "my_partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({"index": i}),
        );
        store.append(event).await.expect("internal error");
    }

    // Get events since ID 2
    let events = store.get_partition_events("my_partition", Some(2));
    assert_eq!(events.len(), 3); // IDs 3, 4, 5
}

#[tokio::test]
async fn test_get_partition_events_nonexistent_partition() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    let events = store.get_partition_events("nonexistent", None);
    assert!(events.is_empty());
}

#[tokio::test]
async fn test_get_event_nonexistent() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    let event = store.get_event(999);
    assert!(event.is_none());
}

#[tokio::test]
async fn test_get_events_since_no_limit() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    for i in 0..5 {
        let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
        store.append(event).await.expect("internal error");
    }

    // Get all events since ID 2 with no limit
    let events = store.get_events_since(2, None);
    assert_eq!(events.len(), 3); // IDs 3, 4, 5
}

#[tokio::test]
async fn test_compact_no_persistence() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    let result = store.compact().await.expect("internal error");
    assert_eq!(result.events_before, 0);
    assert_eq!(result.events_after, 0);
}

#[tokio::test]
async fn test_batch_append_memory_limit() {
    let config = EventStoreConfig {
        max_events_in_memory: 3,
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    let events: Vec<StateEvent> = (0..10)
        .map(|i| StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({})))
        .collect();

    let ids = store.append_batch(events).await.expect("internal error");
    assert_eq!(ids.len(), 10);

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 3); // Only 3 events retained
}

#[tokio::test]
async fn test_get_statistics() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store = EventStore::new(config).await.expect("internal error");

    // Add events to different partitions
    store
        .append(StateEvent::new(
            "p1".to_string(),
            "e1".to_string(),
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    store
        .append(StateEvent::new(
            "p1".to_string(),
            "e2".to_string(),
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    store
        .append(StateEvent::new(
            "p2".to_string(),
            "e3".to_string(),
            serde_json::json!({}),
        ))
        .await
        .unwrap();

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 3);
    assert_eq!(stats.total_partitions, 2);
    assert_eq!(stats.next_event_id, 4);
    assert!(stats.memory_usage_bytes > 0);
}

#[test]
fn test_event_store_error_debug() {
    let err = EventStoreError::PersistenceError("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("PersistenceError"));
}

#[test]
fn test_compaction_result_debug() {
    let result = CompactionResult {
        events_before: 100,
        events_after: 50,
        bytes_saved: 1024,
        duration: std::time::Duration::from_secs(1),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("events_before"));
    assert!(debug_str.contains("100"));
}

#[test]
fn test_event_store_stats_debug() {
    let stats = EventStoreStats {
        total_events: 100,
        total_partitions: 5,
        next_event_id: 101,
        memory_usage_bytes: 50000,
    };
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("total_events"));
    assert!(debug_str.contains("100"));
}

#[test]
fn test_compaction_result_defaults() {
    let result = CompactionResult::default();
    assert_eq!(result.events_before, 0);
    assert_eq!(result.events_after, 0);
    assert_eq!(result.bytes_saved, 0);
}

#[test]
fn test_event_store_error_all_variants() {
    let ser_err = EventStoreError::SerializationError("invalid json".to_string());
    let persist_err = EventStoreError::PersistenceError("connection lost".to_string());
    let corrupt_err = EventStoreError::CorruptedData("bad format".to_string());
    let not_found_err = EventStoreError::EventNotFound(42);

    assert!(format!("{:?}", ser_err).contains("SerializationError"));
    assert!(format!("{:?}", persist_err).contains("PersistenceError"));
    assert!(format!("{:?}", corrupt_err).contains("CorruptedData"));
    assert!(format!("{:?}", not_found_err).contains("EventNotFound"));
}
