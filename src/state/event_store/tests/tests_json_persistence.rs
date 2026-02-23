use crate::state::event_store::*;
use crate::state::*;
use std::collections::BTreeMap;

#[tokio::test]
async fn test_json_file_persistence_new() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_events.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap()).await;
    assert!(persistence.is_ok());
}

#[tokio::test]
async fn test_json_file_persistence_append_event() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_events.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

    let event = StateEvent::new(
        "test_partition".to_string(),
        "test_type".to_string(),
        serde_json::json!({"key": "value"}),
    );

    let result = persistence.append_event(&event).await;
    assert!(result.is_ok());

    // Verify file was written
    let metadata = std::fs::metadata(&file_path).unwrap();
    assert!(metadata.len() > 0);
}

#[tokio::test]
async fn test_json_file_persistence_append_batch() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_batch.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

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

    // Load and verify
    let loaded = persistence.load_all().await.unwrap();
    assert_eq!(loaded.len(), 5);
}

#[tokio::test]
async fn test_json_file_persistence_load_all() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_load.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

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
async fn test_json_file_persistence_load_empty_file() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

    let loaded = persistence.load_all().await.unwrap();
    assert!(loaded.is_empty());
}

#[tokio::test]
async fn test_json_file_persistence_compact() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("compact.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

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

    // Compact
    let result = persistence.compact(&events_map).await;
    assert!(result.is_ok());

    // Load and verify
    let loaded = persistence.load_all().await.unwrap();
    assert_eq!(loaded.len(), 5);
}

#[tokio::test]
async fn test_json_file_serialize_deserialize_roundtrip() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("roundtrip.log");
    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

    // Create event with complex JSON data (this failed with bincode!)
    let event = StateEvent::new(
        "test_partition".to_string(),
        "complex_event".to_string(),
        serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"},
                "number": 42,
                "boolean": true,
                "null": null
            }
        }),
    );

    persistence.append_event(&event).await.unwrap();
    let loaded = persistence.load_all().await.unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].event_type, "complex_event");
    assert_eq!(loaded[0].data["nested"]["array"][0], 1);
    assert_eq!(loaded[0].data["nested"]["object"]["key"], "value");
}

#[tokio::test]
async fn test_json_file_persistence_serialize_event() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("serialize.log");
    let _persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();

    let event = StateEvent::new(
        "test".to_string(),
        "serialize_test".to_string(),
        serde_json::json!({"nested": {"key": "value"}}),
    );

    // Test serialization
    let serialized = JsonFilePersistence::serialize_event(&event).unwrap();
    assert!(serialized.contains("serialize_test"));
    assert!(serialized.contains('\t')); // Tab separator
    assert!(serialized.ends_with('\n')); // Newline at end
}

#[tokio::test]
async fn test_json_file_persistence_handles_corrupt_line() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("corrupt.log");

    // Write a corrupt line directly to the file
    std::fs::write(&file_path, "corrupt line without tab\n").unwrap();

    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();
    let result = persistence.load_all().await;

    // Should return an error for corrupt lines
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("CorruptedData"));
    }
}

#[tokio::test]
async fn test_json_file_persistence_handles_checksum_mismatch() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("bad_checksum.log");

    // Write a line with invalid checksum
    std::fs::write(&file_path, "{\"event_type\":\"test\"}\t999999999\n").unwrap();

    let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
        .await
        .unwrap();
    let result = persistence.load_all().await;

    // Should return an error for checksum mismatch
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("CorruptedData"));
    }
}
