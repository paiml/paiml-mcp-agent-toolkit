use crate::state::event_store::*;
use crate::state::*;
use std::collections::BTreeMap;

#[test]
fn test_in_memory_persistence_new() {
    let persistence = InMemoryPersistence::new();
    assert!(persistence.is_empty());
    assert_eq!(persistence.len(), 0);
}

#[tokio::test]
async fn test_in_memory_persistence_append_event() {
    let persistence = InMemoryPersistence::new();

    let event = StateEvent::new(
        "test_partition".to_string(),
        "test_type".to_string(),
        serde_json::json!({"key": "value"}),
    );

    let result = persistence.append_event(&event).await;
    assert!(result.is_ok());
    assert_eq!(persistence.len(), 1);
}

#[tokio::test]
async fn test_in_memory_persistence_append_batch() {
    let persistence = InMemoryPersistence::new();

    let events: Vec<StateEvent> = (0..5)
        .map(|i| {
            StateEvent::new(
                format!("partition_{}", i),
                format!("type_{}", i),
                serde_json::json!({"index": i}),
            )
        })
        .collect();

    let result = persistence.append_batch(&events).await;
    assert!(result.is_ok());
    assert_eq!(persistence.len(), 5);
}

#[tokio::test]
async fn test_in_memory_persistence_load_all() {
    let persistence = InMemoryPersistence::new();

    // Append multiple events
    for i in 0..3 {
        let event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({"data": i}),
        );
        persistence.append_event(&event).await.unwrap();
    }

    // Load all events
    let loaded = persistence.load_all().await.unwrap();
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[0].event_type, "event_0");
    assert_eq!(loaded[1].event_type, "event_1");
    assert_eq!(loaded[2].event_type, "event_2");
}

#[tokio::test]
async fn test_in_memory_persistence_compact() {
    let persistence = InMemoryPersistence::new();

    // Add some events
    let mut events_map = BTreeMap::new();
    for i in 1..=5 {
        let mut event = StateEvent::new(
            "partition".to_string(),
            format!("event_{}", i),
            serde_json::json!({}),
        );
        event.id = i;
        events_map.insert(i, event.clone());
        persistence.append_event(&event).await.unwrap();
    }

    // Compact (should replace all events with provided map)
    let result = persistence.compact(&events_map).await;
    assert!(result.is_ok());

    // Load and verify
    let loaded = persistence.load_all().await.unwrap();
    assert_eq!(loaded.len(), 5);
}

#[tokio::test]
async fn test_in_memory_persistence_clear() {
    let persistence = InMemoryPersistence::new();

    for i in 0..3 {
        let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
        persistence.append_event(&event).await.unwrap();
    }

    assert_eq!(persistence.len(), 3);
    persistence.clear();
    assert!(persistence.is_empty());
}
