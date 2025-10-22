//! Integration tests for TDG storage backend flexibility

use pmat::tdg::{
    Grade, InMemoryBackend, Language, StorageBackend, StorageBackendFactory,
    StorageBackendType, StorageConfig, TdgScore, TieredStore,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(feature = "sled-backend")]
use pmat::tdg::SledBackend;

#[test]
fn test_in_memory_backend_basic_operations() {
    let backend = InMemoryBackend::new();

    // Test basic put/get
    let key = b"test_key";
    let value = b"test_value";

    backend.put(key, value).unwrap();
    assert!(backend.contains(key).unwrap());

    let retrieved = backend.get(key).unwrap().unwrap();
    assert_eq!(retrieved, value);

    // Test delete
    backend.delete(key).unwrap();
    assert!(!backend.contains(key).unwrap());
}

#[cfg(feature = "sled-backend")]
#[ignore]
#[test]
fn test_sled_backend_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let backend = SledBackend::new(temp_dir.path()).unwrap();

    // Test persistence
    let key = b"persist_key";
    let value = b"persist_value";

    backend.put(key, value).unwrap();
    backend.flush().unwrap();

    // Reopen the same database
    let backend2 = SledBackend::new(temp_dir.path()).unwrap();
    let retrieved = backend2.get(key).unwrap().unwrap();
    assert_eq!(retrieved, value);
}

#[test]
fn test_backend_factory_creation() {
    // Test in-memory creation
    let backend = StorageBackendFactory::create_in_memory();
    assert_eq!(backend.backend_name(), "in-memory");

    // Test config-based creation
    let config = StorageConfig {
        backend_type: StorageBackendType::InMemory,
        path: None,
        cache_size_mb: None,
        compression: false,
    };
    let backend = StorageBackendFactory::create_from_config(&config).unwrap();
    assert_eq!(backend.backend_name(), "in-memory");
}

#[cfg(feature = "sled-backend")]
#[test]
fn test_sled_backend_factory_creation() {
    // Test temporary sled creation
    let backend = StorageBackendFactory::create_sled_temporary().unwrap();
    assert_eq!(backend.backend_name(), "sled");
}

#[tokio::test]
async fn test_tiered_storage_with_backends() {
    // Test with in-memory backend
    let storage = TieredStore::in_memory();

    // Create a test record
    use pmat::tdg::{
        AnalysisMetadata, ComponentScores, FileIdentity, FullTdgRecord, SemanticSignature,
    };
    use std::collections::HashMap;
    use std::time::SystemTime;

    let content = b"fn test() { println!(\"hello\"); }";
    let hash = blake3::hash(content);

    let record = FullTdgRecord {
        identity: FileIdentity {
            path: PathBuf::from("test.rs"),
            content_hash: hash,
            size_bytes: content.len() as u64,
            modified_time: SystemTime::now(),
        },
        score: TdgScore {
            structural_complexity: 20.0,
            semantic_complexity: 18.0,
            duplication_ratio: 19.0,
            coupling_score: 14.0,
            doc_coverage: 9.0,
            consistency_score: 8.0,
            entropy_score: 16.0,
            total: 88.0,
            grade: Grade::AMinus,
            confidence: 0.95,
            language: Language::Rust,
            file_path: Some(PathBuf::from("test.rs")),
            penalties_applied: Vec::new(),
        },
        components: ComponentScores {
            complexity_breakdown: HashMap::new(),
            duplication_sources: Vec::new(),
            coupling_dependencies: Vec::new(),
            doc_missing_items: Vec::new(),
            consistency_violations: Vec::new(),
        },
        semantic_sig: SemanticSignature {
            ast_structure_hash: 123456789,
            identifier_pattern: "test,println".to_string(),
            control_flow_pattern: "function_call".to_string(),
            import_dependencies: Vec::new(),
        },
        metadata: AnalysisMetadata {
            analyzer_version: "2.38.0".to_string(),
            analysis_duration_ms: 5,
            language_confidence: 1.0,
            analysis_timestamp: SystemTime::now(),
            cache_hit: false,
        },
    };

    // Store and retrieve
    storage.store(record.clone()).await.unwrap();

    // Check hot cache
    let hot_entry = storage.get_hot(&hash).unwrap();
    assert_eq!(hot_entry.total_score, 88.0);

    // Retrieve full record
    let retrieved = storage.retrieve_full(&hash).await.unwrap().unwrap();
    assert_eq!(retrieved.score.total, record.score.total);
}

#[cfg(feature = "sled-backend")]
#[tokio::test]
async fn test_storage_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let storage = TieredStore::new(temp_dir.path()).unwrap();

    let stats = storage.get_statistics();
    assert_eq!(stats.hot_entries, 0);
    assert_eq!(stats.warm_entries, 0);
    assert_eq!(stats.cold_entries, 0);
    assert_eq!(stats.warm_backend, "sled");
    assert_eq!(stats.cold_backend, "sled");
}

#[test]
fn test_backend_iteration() {
    let backend = InMemoryBackend::new();

    // Add multiple entries
    for i in 0..10 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        backend.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    // Test iteration
    let mut count = 0;
    for result in backend.iter().unwrap() {
        let (_key, _value) = result.unwrap();
        count += 1;
    }
    assert_eq!(count, 10);
}

#[test]
fn test_backend_clear() {
    let backend = InMemoryBackend::new();

    // Add entries
    backend.put(b"key1", b"value1").unwrap();
    backend.put(b"key2", b"value2").unwrap();

    assert!(backend.contains(b"key1").unwrap());

    // Clear all
    backend.clear().unwrap();

    assert!(!backend.contains(b"key1").unwrap());
    assert!(!backend.contains(b"key2").unwrap());
}

#[cfg(feature = "rocksdb-backend")]
#[ignore]
#[test]
fn test_rocksdb_backend() {
    use pmat::tdg::storage_backend::RocksDbBackend;

    let temp_dir = TempDir::new().unwrap();
    let backend = RocksDbBackend::new(temp_dir.path()).unwrap();

    // Test basic operations
    let key = b"rocks_key";
    let value = b"rocks_value";

    backend.put(key, value).unwrap();
    assert!(backend.contains(key).unwrap());

    let retrieved = backend.get(key).unwrap().unwrap();
    assert_eq!(retrieved, value);

    assert_eq!(backend.backend_name(), "rocksdb");
}

#[test]
fn test_storage_backend_type_display() {
    assert_eq!(format!("{}", StorageBackendType::Sled), "sled");
    assert_eq!(format!("{}", StorageBackendType::Libsql), "libsql");
    assert_eq!(format!("{}", StorageBackendType::InMemory), "in-memory");
    assert_eq!(format!("{}", StorageBackendType::RocksDb), "rocksdb");
}

#[test]
fn test_libsql_backend() {
    use pmat::tdg::storage_backend::LibsqlBackend;

    let temp_dir = TempDir::new().unwrap();
    let backend = LibsqlBackend::new(temp_dir.path().join("test.db").as_path()).unwrap();

    // Test basic operations
    let key = b"libsql_key";
    let value = b"libsql_value";

    backend.put(key, value).unwrap();
    assert!(backend.contains(key).unwrap());

    let retrieved = backend.get(key).unwrap().unwrap();
    assert_eq!(retrieved, value);

    assert_eq!(backend.backend_name(), "libsql");
}
