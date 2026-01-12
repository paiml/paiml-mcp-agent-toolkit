//! Integration tests for TDG storage backend flexibility

use pmat::tdg::{
    Grade, InMemoryBackend, Language, StorageBackend, StorageBackendFactory, StorageBackendType,
    StorageConfig, TdgScore, TieredStore,
};
use std::path::PathBuf;
use tempfile::TempDir;

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

#[test]
fn test_lz4_compression_works() {
    // EXTREME TDD: RED test - Verify lz4 compression/decompression works independently
    use lz4_flex::{compress_prepend_size, decompress_size_prepended};

    let original = b"fn test() { println!(\"hello\"); }";
    let compressed = compress_prepend_size(original);
    let decompressed = decompress_size_prepended(&compressed).expect("lz4 decompression failed");

    assert_eq!(original.as_slice(), decompressed.as_slice());
}

#[test]
fn test_bincode_blake3hash_serialization() {
    // EXTREME TDD: RED test - Check if Blake3Hash serializes properly with bincode
    let content = b"test data";
    let hash = blake3::hash(content);

    let serialized = bincode::serialize(&hash).expect("Failed to serialize Blake3Hash");
    println!("DEBUG: Blake3Hash serialized to {} bytes", serialized.len());

    let deserialized: blake3::Hash =
        bincode::deserialize(&serialized).expect("Failed to deserialize Blake3Hash");
    assert_eq!(hash, deserialized);
}

#[test]
fn test_bincode_systemtime_serialization() {
    // EXTREME TDD: RED test - Check if SystemTime serializes properly with bincode
    use std::time::SystemTime;

    let now = SystemTime::now();
    let serialized = bincode::serialize(&now).expect("Failed to serialize SystemTime");
    println!("DEBUG: SystemTime serialized to {} bytes", serialized.len());

    let deserialized: SystemTime =
        bincode::deserialize(&serialized).expect("Failed to deserialize SystemTime");
    // SystemTime may not implement Eq, so just check it doesn't panic
    println!(
        "DEBUG: SystemTime deserialized successfully: {:?}",
        deserialized
    );
}

#[test]
fn test_bincode_tdgscore_serialization() {
    // EXTREME TDD: RED test - Check if TdgScore serializes properly with bincode
    use pmat::tdg::{Grade, Language, TdgScore};
    use std::path::PathBuf;

    let score = TdgScore {
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
        critical_defects_count: 0,
        has_critical_defects: false,
    };

    let serialized = bincode::serialize(&score).expect("Failed to serialize TdgScore");
    println!("DEBUG: TdgScore serialized to {} bytes", serialized.len());

    let deserialized: TdgScore =
        bincode::deserialize(&serialized).expect("Failed to deserialize TdgScore");
    assert_eq!(deserialized.total, score.total);
    println!("DEBUG: TdgScore deserialized successfully");
}

#[test]
fn test_bincode_fileidentity_serialization() {
    // EXTREME TDD: RED test - Check if FileIdentity serializes properly with bincode
    use pmat::tdg::FileIdentity;
    use std::path::PathBuf;
    use std::time::SystemTime;

    let content = b"test";
    let hash = blake3::hash(content);

    let identity = FileIdentity {
        path: PathBuf::from("test.rs"),
        content_hash: hash,
        size_bytes: content.len() as u64,
        modified_time: SystemTime::now(),
    };

    let serialized = bincode::serialize(&identity).expect("Failed to serialize FileIdentity");
    println!(
        "DEBUG: FileIdentity serialized to {} bytes",
        serialized.len()
    );

    let deserialized: FileIdentity =
        bincode::deserialize(&serialized).expect("Failed to deserialize FileIdentity");
    assert_eq!(deserialized.size_bytes, identity.size_bytes);
    println!("DEBUG: FileIdentity deserialized successfully");
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
            critical_defects_count: 0,
            has_critical_defects: false,
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
        git_context: None,
    };

    // Store and retrieve
    storage.store(record.clone()).await.unwrap();

    // Check hot cache
    let hot_entry = storage.get_hot(&hash).unwrap();
    assert_eq!(hot_entry.total_score, 88.0);

    // Retrieve full record from storage
    let retrieved = storage.retrieve_full(&hash).await.unwrap().unwrap();
    assert_eq!(retrieved.score.total, record.score.total);
    assert_eq!(retrieved.identity.path, record.identity.path);
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

#[test]
fn test_storage_backend_type_display() {
    assert_eq!(format!("{}", StorageBackendType::Libsql), "libsql");
    assert_eq!(format!("{}", StorageBackendType::InMemory), "in-memory");
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
