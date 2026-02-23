use crate::state::event_store::*;
use crate::state::*;
use std::sync::Arc;

// ===== EventStore with InMemoryPersistence Tests =====

#[tokio::test]
async fn test_event_store_with_in_memory_persistence() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig {
        persistence_enabled: true,
        ..Default::default()
    };
    let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

    // Add events
    for i in 0..3 {
        let event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({"index": i}),
        );
        store.append(event).await.unwrap();
    }

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 3);
    assert_eq!(persistence.len(), 3);
}

#[tokio::test]
async fn test_event_store_batch_with_in_memory_persistence() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig {
        persistence_enabled: true,
        ..Default::default()
    };
    let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

    let events: Vec<StateEvent> = (0..5)
        .map(|i| StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({})))
        .collect();

    let ids = store.append_batch(events).await.unwrap();
    assert_eq!(ids.len(), 5);
    assert_eq!(persistence.len(), 5);
}

#[tokio::test]
async fn test_event_store_compact_with_in_memory_persistence() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig {
        persistence_enabled: true,
        ..Default::default()
    };
    let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

    // Add events
    for i in 0..10 {
        let event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({}),
        );
        store.append(event).await.unwrap();
    }

    // Compact
    let result = store.compact().await.unwrap();
    assert_eq!(result.events_before, 10);
    assert_eq!(result.events_after, 10);
}

#[tokio::test]
async fn test_event_store_recovery_with_in_memory_persistence() {
    let persistence = Arc::new(InMemoryPersistence::new());

    // First store - create events
    {
        let config = EventStoreConfig {
            persistence_enabled: true,
            ..Default::default()
        };
        let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

        for i in 0..5 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.unwrap();
        }
    }

    // Second store - recover events (same persistence instance)
    {
        let config = EventStoreConfig {
            persistence_enabled: true,
            ..Default::default()
        };
        let mut store = EventStore::new_with_persistence(config, Some(persistence.clone()));
        store.recover().await.unwrap();

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.next_event_id, 6); // Should continue from 6

        // Verify events can be retrieved
        let event = store.get_event(1);
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, "event_0");
    }
}

// ===== Additional Comprehensive Tests =====

#[tokio::test]
async fn test_event_store_no_persistence_backend() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store: EventStore<InMemoryPersistence> = EventStore::new_with_persistence(config, None);

    let event = StateEvent::new(
        "partition".to_string(),
        "event_type".to_string(),
        serde_json::json!({}),
    );

    let id = store.append(event).await.unwrap();
    assert_eq!(id, 1);

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 1);
}

#[tokio::test]
async fn test_event_store_get_events_since() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig::default();
    let store = EventStore::new_with_persistence(config, Some(persistence));

    // Add 5 events
    for i in 0..5 {
        let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
        store.append(event).await.unwrap();
    }

    // Get events since ID 2 (should get events 3, 4, 5)
    let events = store.get_events_since(2, None);
    assert_eq!(events.len(), 3);

    // Get events since ID 2 with limit
    let events_limited = store.get_events_since(2, Some(2));
    assert_eq!(events_limited.len(), 2);
}

#[tokio::test]
async fn test_event_store_get_partition_events() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig::default();
    let store = EventStore::new_with_persistence(config, Some(persistence));

    // Add events to different partitions
    for i in 0..5 {
        let event = StateEvent::new(
            format!("partition_{}", i % 2),
            format!("e{}", i),
            serde_json::json!({}),
        );
        store.append(event).await.unwrap();
    }

    // Get events for partition_0
    let events_p0 = store.get_partition_events("partition_0", None);
    assert_eq!(events_p0.len(), 3); // indices 0, 2, 4

    // Get events for partition_1
    let events_p1 = store.get_partition_events("partition_1", None);
    assert_eq!(events_p1.len(), 2); // indices 1, 3

    // Get events for non-existent partition
    let events_none = store.get_partition_events("nonexistent", None);
    assert!(events_none.is_empty());

    // Get events with since filter
    let events_since = store.get_partition_events("partition_0", Some(2));
    assert_eq!(events_since.len(), 2); // only events with id > 2
}

#[tokio::test]
async fn test_event_store_get_event() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig::default();
    let store = EventStore::new_with_persistence(config, Some(persistence));

    let event = StateEvent::new(
        "p".to_string(),
        "test_event".to_string(),
        serde_json::json!({"data": 42}),
    );
    let id = store.append(event).await.unwrap();

    // Get existing event
    let retrieved = store.get_event(id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().event_type, "test_event");

    // Get non-existent event
    let missing = store.get_event(999);
    assert!(missing.is_none());
}

#[tokio::test]
async fn test_event_store_get_latest_event_id() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig::default();
    let store = EventStore::new_with_persistence(config, Some(persistence));

    // Initially 0 (1 - 1)
    assert_eq!(store.get_latest_event_id(), 0);

    // After adding events
    for _ in 0..5 {
        let event = StateEvent::new("p".to_string(), "e".to_string(), serde_json::json!({}));
        store.append(event).await.unwrap();
    }

    assert_eq!(store.get_latest_event_id(), 5);
}

#[tokio::test]
async fn test_event_store_memory_limit_enforcement() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig {
        max_events_in_memory: 5,
        ..Default::default()
    };
    let store = EventStore::new_with_persistence(config, Some(persistence));

    // Add more events than the limit
    for i in 0..10 {
        let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
        store.append(event).await.unwrap();
    }

    // Stats should show only max_events_in_memory events
    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 5);

    // Oldest events should be removed (events 1-5 removed, 6-10 remain)
    assert!(store.get_event(1).is_none());
    assert!(store.get_event(6).is_some());
}

#[tokio::test]
async fn test_event_store_batch_memory_limit_enforcement() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig {
        max_events_in_memory: 3,
        ..Default::default()
    };
    let store = EventStore::new_with_persistence(config, Some(persistence));

    // Add batch that exceeds limit
    let events: Vec<StateEvent> = (0..5)
        .map(|i| StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({})))
        .collect();

    store.append_batch(events).await.unwrap();

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 3);
}

#[tokio::test]
async fn test_event_store_compact_without_persistence() {
    let config = EventStoreConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let store: EventStore<InMemoryPersistence> = EventStore::new_with_persistence(config, None);

    // Compact should return default result
    let result = store.compact().await.unwrap();
    assert_eq!(result.events_before, 0);
    assert_eq!(result.events_after, 0);
}

#[tokio::test]
async fn test_event_store_statistics_multiple_partitions() {
    let persistence = Arc::new(InMemoryPersistence::new());
    let config = EventStoreConfig::default();
    let store = EventStore::new_with_persistence(config, Some(persistence));

    // Add events to multiple partitions
    for i in 0..10 {
        let event = StateEvent::new(
            format!("partition_{}", i % 3),
            format!("e{}", i),
            serde_json::json!({}),
        );
        store.append(event).await.unwrap();
    }

    let stats = store.get_statistics();
    assert_eq!(stats.total_events, 10);
    assert_eq!(stats.total_partitions, 3);
    assert_eq!(stats.next_event_id, 11);
    assert!(stats.memory_usage_bytes > 0);
}
