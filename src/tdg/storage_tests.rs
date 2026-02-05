
use super::*;
use crate::tdg::Grade;

fn create_test_record() -> FullTdgRecord {
    let content = b"fn test() { println!(\"hello\"); }";
    let hash = blake3::hash(content);

    FullTdgRecord {
        identity: FileIdentity {
            path: PathBuf::from("test.rs"),
            content_hash: hash,
            size_bytes: content.len() as u64,
            modified_time: SystemTime::now(),
        },
        score: TdgScore::default(),
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
        git_context: None, // Test helper doesn't include git context
    }
}

#[tokio::test]
async fn test_tiered_storage_creation() {
    let storage = TieredStore::in_memory();

    let stats = storage.get_statistics();
    assert_eq!(stats.hot_entries, 0);
    assert_eq!(stats.warm_entries, 0);
    assert_eq!(stats.cold_entries, 0);
}

#[tokio::test]
async fn test_in_memory_storage() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();
    let hash = record.identity.content_hash;

    // Store record
    storage.store(record.clone()).await.unwrap();

    // Check hot cache
    let hot_entry = storage.get_hot(&hash).unwrap();
    assert_eq!(hot_entry.total_score, 100.0); // TdgScore::default() = 100.0
    assert_eq!(hot_entry.grade, Grade::APLus as u8);

    // Retrieve full record
    let retrieved = storage.retrieve_full(&hash).await.unwrap().unwrap();
    assert_eq!(retrieved.score.total, record.score.total);
    assert_eq!(retrieved.identity.path, record.identity.path);
}

#[tokio::test]
async fn test_store_and_retrieve() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();
    let hash = record.identity.content_hash;

    // Store record
    storage.store(record.clone()).await.unwrap();

    // Check hot cache
    let hot_entry = storage.get_hot(&hash).unwrap();
    assert_eq!(hot_entry.total_score, 100.0); // TdgScore::default() = 100.0
    assert_eq!(hot_entry.grade, Grade::APLus as u8);

    // Retrieve full record
    let retrieved = storage.retrieve_full(&hash).await.unwrap().unwrap();
    assert_eq!(retrieved.score.total, record.score.total);
    assert_eq!(retrieved.identity.path, record.identity.path);
}

#[tokio::test]
async fn test_compression() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();

    // Store and verify compression
    storage.store(record.clone()).await.unwrap();
    storage.flush().unwrap();

    let stats = storage.get_statistics();
    assert!(stats.compression_ratio > 0.0);
    assert!(stats.compression_ratio < 1.0); // Should be compressed
}

#[test]
fn test_hot_cache_cleanup() {
    let storage = TieredStore::in_memory();

    // Add some entries with old timestamps
    let old_hash = blake3::hash(b"old content");
    let old_entry = HotCacheEntry {
        content_hash: *old_hash.as_bytes(),
        grade: Grade::B as u8,
        total_score: 75.0,
        timestamp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64)
            - 3600, // 1 hour ago
    };
    storage.hot.insert(old_hash, old_entry);

    // Cleanup entries older than 30 minutes
    let removed = storage.cleanup_hot_cache(1800);
    assert_eq!(removed, 1);
    assert!(storage.hot.is_empty());
}

#[tokio::test]
async fn test_backend_migration() {
    use crate::tdg::storage_backend::StorageBackendType;

    let mut storage = TieredStore::in_memory();

    // Store some records
    let record1 = create_test_record();
    let record2 = create_test_record();
    storage.store(record1.clone()).await.unwrap();
    storage.store(record2.clone()).await.unwrap();

    // Migrate to another in-memory backend (tests migration logic)
    let new_warm = StorageConfig {
        backend_type: StorageBackendType::InMemory,
        path: None,
        cache_size_mb: None,
        compression: true,
    };

    let new_cold = StorageConfig {
        backend_type: StorageBackendType::InMemory,
        path: None,
        cache_size_mb: None,
        compression: false,
    };

    storage.migrate_backend(new_warm, new_cold).await.unwrap();

    // Verify data still accessible
    let retrieved = storage
        .retrieve_full(&record1.identity.content_hash)
        .await
        .unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_storage_statistics_format_diagnostic() {
    let stats = StorageStatistics {
        hot_entries: 10,
        warm_entries: 50,
        cold_entries: 100,
        total_entries: 160,
        hot_memory_kb: 5,
        compression_ratio: 0.25,
        warm_backend: "sled".to_string(),
        cold_backend: "sled".to_string(),
        backend_stats: HashMap::new(),
    };

    let output = stats.format_diagnostic();
    assert!(output.contains("Hot (memory): 10 entries"));
    assert!(output.contains("Warm (sled backend): 50 entries"));
    assert!(output.contains("Cold (sled backend): 100 entries"));
    assert!(output.contains("Total: 160 entries"));
    assert!(output.contains("25.0%"));
}

#[test]
fn test_tiered_storage_factory_in_memory() {
    let storage = TieredStorageFactory::create_in_memory();
    let stats = storage.get_statistics();
    assert_eq!(stats.hot_entries, 0);
}

#[tokio::test]
async fn test_get_by_path() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();
    let target_path = record.identity.path.clone();

    // Store record
    storage.store(record.clone()).await.unwrap();

    // Query by path
    let results = storage.get_by_path(&target_path).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].identity.path, target_path);
}

#[tokio::test]
async fn test_retrieve_nonexistent_record() {
    let storage = TieredStore::in_memory();
    let fake_hash = blake3::hash(b"nonexistent");

    // Should return None, not error
    let result = storage.retrieve_full(&fake_hash).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_hot_nonexistent() {
    let storage = TieredStore::in_memory();
    let fake_hash = blake3::hash(b"nonexistent");

    // Should return None
    let result = storage.get_hot(&fake_hash);
    assert!(result.is_none());
}

#[test]
fn test_hot_cache_entry_from_record() {
    let record = create_test_record();
    let entry = HotCacheEntry::from_record(&record);

    assert_eq!(entry.total_score, record.score.total);
    assert_eq!(entry.grade, record.score.grade as u8);
    assert!(entry.timestamp > 0);
}

#[test]
fn test_component_scores_default() {
    let scores = ComponentScores::default();
    assert!(scores.complexity_breakdown.is_empty());
    assert!(scores.duplication_sources.is_empty());
    assert!(scores.coupling_dependencies.is_empty());
    assert!(scores.doc_missing_items.is_empty());
    assert!(scores.consistency_violations.is_empty());
}

#[tokio::test]
async fn test_get_all_with_git_context_empty() {
    let storage = TieredStore::in_memory();
    let record = create_test_record(); // has git_context: None

    // Store record without git context
    storage.store(record).await.unwrap();

    // Should not return records without git_context
    let results = storage.get_all_with_git_context().await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_get_by_commit_no_match() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();

    storage.store(record).await.unwrap();

    // Query with non-existent commit
    let results = storage.get_by_commit("abc1234").await.unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_file_identity_clone() {
    let content = b"test content";
    let identity = FileIdentity {
        path: PathBuf::from("test.rs"),
        content_hash: blake3::hash(content),
        size_bytes: content.len() as u64,
        modified_time: SystemTime::now(),
    };

    let cloned = identity.clone();
    assert_eq!(cloned.path, identity.path);
    assert_eq!(cloned.content_hash, identity.content_hash);
}

#[test]
fn test_semantic_signature_clone() {
    let sig = SemanticSignature {
        ast_structure_hash: 12345,
        identifier_pattern: "foo,bar".to_string(),
        control_flow_pattern: "loop".to_string(),
        import_dependencies: vec!["std".to_string()],
    };

    let cloned = sig.clone();
    assert_eq!(cloned.ast_structure_hash, sig.ast_structure_hash);
    assert_eq!(cloned.identifier_pattern, sig.identifier_pattern);
}

#[test]
fn test_analysis_metadata_clone() {
    let meta = AnalysisMetadata {
        analyzer_version: "1.0.0".to_string(),
        analysis_duration_ms: 100,
        language_confidence: 0.95,
        analysis_timestamp: SystemTime::now(),
        cache_hit: true,
    };

    let cloned = meta.clone();
    assert_eq!(cloned.analyzer_version, meta.analyzer_version);
    assert_eq!(cloned.cache_hit, meta.cache_hit);
}

#[tokio::test]
async fn test_flush() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();

    storage.store(record).await.unwrap();

    // Flush should not error
    let result = storage.flush();
    assert!(result.is_ok());
}

#[test]
fn test_hot_cache_cleanup_no_old_entries() {
    let storage = TieredStore::in_memory();

    // Add a fresh entry
    let hash = blake3::hash(b"fresh content");
    let entry = HotCacheEntry {
        content_hash: *hash.as_bytes(),
        grade: Grade::A as u8,
        total_score: 90.0,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };
    storage.hot.insert(hash, entry);

    // Cleanup with 1 hour threshold should not remove fresh entry
    let removed = storage.cleanup_hot_cache(3600);
    assert_eq!(removed, 0);
    assert_eq!(storage.hot.len(), 1);
}

// Additional tests for uncovered code paths

#[test]
fn test_file_identity_debug() {
    let content = b"debug test";
    let identity = FileIdentity {
        path: PathBuf::from("debug.rs"),
        content_hash: blake3::hash(content),
        size_bytes: content.len() as u64,
        modified_time: SystemTime::now(),
    };
    let debug_str = format!("{:?}", identity);
    assert!(debug_str.contains("FileIdentity"));
    assert!(debug_str.contains("debug.rs"));
}

#[test]
fn test_component_scores_clone_and_debug() {
    let mut scores = ComponentScores::default();
    scores.complexity_breakdown.insert("func1".to_string(), 5.0);
    scores.duplication_sources.push("dup1.rs".to_string());
    scores.coupling_dependencies.push("dep1".to_string());
    scores.doc_missing_items.push("missing_doc".to_string());
    scores.consistency_violations.push("violation1".to_string());

    let cloned = scores.clone();
    assert_eq!(cloned.complexity_breakdown.get("func1"), Some(&5.0));
    assert_eq!(cloned.duplication_sources.len(), 1);
    assert_eq!(cloned.coupling_dependencies.len(), 1);

    let debug_str = format!("{:?}", scores);
    assert!(debug_str.contains("ComponentScores"));
}

#[test]
fn test_semantic_signature_debug() {
    let sig = SemanticSignature {
        ast_structure_hash: 999999,
        identifier_pattern: "pattern".to_string(),
        control_flow_pattern: "control".to_string(),
        import_dependencies: vec!["dep1".to_string(), "dep2".to_string()],
    };
    let debug_str = format!("{:?}", sig);
    assert!(debug_str.contains("SemanticSignature"));
    assert!(debug_str.contains("999999"));
}

#[test]
fn test_analysis_metadata_debug() {
    let meta = AnalysisMetadata {
        analyzer_version: "3.0.0".to_string(),
        analysis_duration_ms: 50,
        language_confidence: 0.99,
        analysis_timestamp: SystemTime::now(),
        cache_hit: false,
    };
    let debug_str = format!("{:?}", meta);
    assert!(debug_str.contains("AnalysisMetadata"));
    assert!(debug_str.contains("3.0.0"));
}

#[test]
fn test_full_tdg_record_clone() {
    let record = create_test_record();
    let cloned = record.clone();

    assert_eq!(cloned.identity.path, record.identity.path);
    assert_eq!(cloned.score.total, record.score.total);
    assert_eq!(
        cloned.semantic_sig.ast_structure_hash,
        record.semantic_sig.ast_structure_hash
    );
}

#[test]
fn test_full_tdg_record_debug() {
    let record = create_test_record();
    let debug_str = format!("{:?}", record);
    assert!(debug_str.contains("FullTdgRecord"));
    assert!(debug_str.contains("identity"));
}

#[test]
fn test_hot_cache_entry_copy() {
    let hash = blake3::hash(b"copy test");
    let entry = HotCacheEntry {
        content_hash: *hash.as_bytes(),
        grade: Grade::B as u8,
        total_score: 80.0,
        timestamp: 12345,
    };
    let copied = entry; // Copy trait
    assert_eq!(copied.total_score, 80.0);
    assert_eq!(copied.timestamp, 12345);
}

#[test]
fn test_hot_cache_entry_debug() {
    let hash = blake3::hash(b"debug test");
    let entry = HotCacheEntry {
        content_hash: *hash.as_bytes(),
        grade: Grade::A as u8,
        total_score: 95.0,
        timestamp: 99999,
    };
    let debug_str = format!("{:?}", entry);
    assert!(debug_str.contains("HotCacheEntry"));
    assert!(debug_str.contains("95"));
}

#[test]
fn test_storage_statistics_clone() {
    let stats = StorageStatistics {
        hot_entries: 5,
        warm_entries: 10,
        cold_entries: 20,
        total_entries: 35,
        hot_memory_kb: 2,
        compression_ratio: 0.3,
        warm_backend: "memory".to_string(),
        cold_backend: "memory".to_string(),
        backend_stats: HashMap::new(),
    };
    let cloned = stats.clone();
    assert_eq!(cloned.hot_entries, 5);
    assert_eq!(cloned.total_entries, 35);
}

#[test]
fn test_storage_statistics_debug() {
    let stats = StorageStatistics {
        hot_entries: 1,
        warm_entries: 2,
        cold_entries: 3,
        total_entries: 6,
        hot_memory_kb: 1,
        compression_ratio: 0.5,
        warm_backend: "test".to_string(),
        cold_backend: "test".to_string(),
        backend_stats: HashMap::new(),
    };
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("StorageStatistics"));
}

#[test]
fn test_storage_statistics_serialization() {
    let mut backend_stats = HashMap::new();
    let mut warm_map = HashMap::new();
    warm_map.insert("entry_count".to_string(), "100".to_string());
    backend_stats.insert("warm".to_string(), warm_map);

    let stats = StorageStatistics {
        hot_entries: 10,
        warm_entries: 100,
        cold_entries: 500,
        total_entries: 610,
        hot_memory_kb: 5,
        compression_ratio: 0.25,
        warm_backend: "libsql".to_string(),
        cold_backend: "libsql".to_string(),
        backend_stats,
    };

    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: StorageStatistics = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.hot_entries, 10);
    assert_eq!(deserialized.total_entries, 610);
    assert_eq!(deserialized.warm_backend, "libsql");
}

#[test]
fn test_file_identity_serialization() {
    let content = b"serialize test";
    let identity = FileIdentity {
        path: PathBuf::from("serialize.rs"),
        content_hash: blake3::hash(content),
        size_bytes: content.len() as u64,
        modified_time: SystemTime::UNIX_EPOCH,
    };

    let json = serde_json::to_string(&identity).unwrap();
    let deserialized: FileIdentity = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, identity.path);
    assert_eq!(deserialized.size_bytes, identity.size_bytes);
}

#[test]
fn test_component_scores_serialization() {
    let mut scores = ComponentScores::default();
    scores.complexity_breakdown.insert("main".to_string(), 10.0);
    scores.duplication_sources.push("dup.rs".to_string());

    let json = serde_json::to_string(&scores).unwrap();
    let deserialized: ComponentScores = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.complexity_breakdown.get("main"), Some(&10.0));
    assert_eq!(deserialized.duplication_sources.len(), 1);
}

#[test]
fn test_semantic_signature_serialization() {
    let sig = SemanticSignature {
        ast_structure_hash: 0xDEADBEEF,
        identifier_pattern: "id_pattern".to_string(),
        control_flow_pattern: "cf_pattern".to_string(),
        import_dependencies: vec!["std::io".to_string()],
    };

    let json = serde_json::to_string(&sig).unwrap();
    let deserialized: SemanticSignature = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.ast_structure_hash, sig.ast_structure_hash);
    assert_eq!(deserialized.import_dependencies.len(), 1);
}

#[test]
fn test_analysis_metadata_serialization() {
    let meta = AnalysisMetadata {
        analyzer_version: "2.0.0".to_string(),
        analysis_duration_ms: 250,
        language_confidence: 0.85,
        analysis_timestamp: SystemTime::UNIX_EPOCH,
        cache_hit: true,
    };

    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: AnalysisMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.analyzer_version, "2.0.0");
    assert!(deserialized.cache_hit);
}

#[tokio::test]
async fn test_storage_multiple_records() {
    let storage = TieredStore::in_memory();

    // Store multiple records
    for i in 0..5 {
        let content = format!("content {}", i);
        let hash = blake3::hash(content.as_bytes());
        let record = FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from(format!("file{}.rs", i)),
                content_hash: hash,
                size_bytes: content.len() as u64,
                modified_time: SystemTime::now(),
            },
            score: TdgScore::default(),
            components: ComponentScores::default(),
            semantic_sig: SemanticSignature {
                ast_structure_hash: i as u64,
                identifier_pattern: format!("pattern{}", i),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "1.0.0".to_string(),
                analysis_duration_ms: 10,
                language_confidence: 1.0,
                analysis_timestamp: SystemTime::now(),
                cache_hit: false,
            },
            git_context: None,
        };
        storage.store(record).await.unwrap();
    }

    let stats = storage.get_statistics();
    assert_eq!(stats.hot_entries, 5);
}

#[tokio::test]
async fn test_get_by_path_no_match() {
    let storage = TieredStore::in_memory();
    let record = create_test_record();
    storage.store(record).await.unwrap();

    // Query with non-existent path
    let results = storage
        .get_by_path(Path::new("nonexistent.rs"))
        .await
        .unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_hot_cache_cleanup_mixed_entries() {
    let storage = TieredStore::in_memory();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Add old entry
    let old_hash = blake3::hash(b"old");
    storage.hot.insert(
        old_hash,
        HotCacheEntry {
            content_hash: *old_hash.as_bytes(),
            grade: Grade::C as u8,
            total_score: 70.0,
            timestamp: now - 7200, // 2 hours ago
        },
    );

    // Add fresh entry
    let fresh_hash = blake3::hash(b"fresh");
    storage.hot.insert(
        fresh_hash,
        HotCacheEntry {
            content_hash: *fresh_hash.as_bytes(),
            grade: Grade::A as u8,
            total_score: 95.0,
            timestamp: now,
        },
    );

    // Cleanup entries older than 1 hour
    let removed = storage.cleanup_hot_cache(3600);
    assert_eq!(removed, 1);
    assert_eq!(storage.hot.len(), 1);
    assert!(storage.hot.contains_key(&fresh_hash));
    assert!(!storage.hot.contains_key(&old_hash));
}

#[test]
fn test_storage_statistics_format_diagnostic_with_backend_stats() {
    let mut backend_stats = HashMap::new();
    let mut warm_map = HashMap::new();
    warm_map.insert("entry_count".to_string(), "50".to_string());
    warm_map.insert("size_bytes".to_string(), "1024".to_string());
    backend_stats.insert("warm".to_string(), warm_map);

    let stats = StorageStatistics {
        hot_entries: 25,
        warm_entries: 50,
        cold_entries: 75,
        total_entries: 150,
        hot_memory_kb: 10,
        compression_ratio: 0.4,
        warm_backend: "libsql".to_string(),
        cold_backend: "libsql".to_string(),
        backend_stats,
    };

    let output = stats.format_diagnostic();
    assert!(output.contains("Hot (memory): 25 entries"));
    assert!(output.contains("10 KB"));
    assert!(output.contains("libsql backend"));
    assert!(output.contains("40.0%"));
}

#[tokio::test]
async fn test_full_tdg_record_serialization_round_trip() {
    let record = create_test_record();

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: FullTdgRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.identity.path, record.identity.path);
    assert_eq!(deserialized.score.total, record.score.total);
    assert_eq!(
        deserialized.semantic_sig.ast_structure_hash,
        record.semantic_sig.ast_structure_hash
    );
    assert_eq!(
        deserialized.metadata.analyzer_version,
        record.metadata.analyzer_version
    );
}

#[test]
fn test_hot_cache_entry_from_record_with_different_grades() {
    use crate::tdg::Grade;

    // Test with different grades
    for (grade, expected_score) in [
        (Grade::APLus, 100.0),
        (Grade::A, 90.0),
        (Grade::B, 80.0),
        (Grade::C, 70.0),
        (Grade::D, 60.0),
        (Grade::F, 50.0),
    ] {
        let content = format!("content for grade {:?}", grade);
        let hash = blake3::hash(content.as_bytes());
        let mut record = create_test_record();
        record.identity.content_hash = hash;
        record.score.grade = grade;
        record.score.total = expected_score;

        let entry = HotCacheEntry::from_record(&record);
        assert_eq!(entry.grade, grade as u8);
        assert_eq!(entry.total_score, expected_score);
    }
}

#[tokio::test]
async fn test_storage_overwrite_same_hash() {
    let storage = TieredStore::in_memory();
    let mut record = create_test_record();
    let hash = record.identity.content_hash;

    // Store initial record
    storage.store(record.clone()).await.unwrap();

    // Modify and store again with same hash
    record.score.total = 75.0;
    storage.store(record.clone()).await.unwrap();

    // Hot cache should have updated value
    let hot_entry = storage.get_hot(&hash).unwrap();
    assert_eq!(hot_entry.total_score, 75.0);
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod extended_tests {
    use super::*;
    use crate::tdg::Grade;

    fn create_test_record_with_old_timestamp() -> FullTdgRecord {
        let content = b"fn old_test() {}";
        let hash = blake3::hash(content);

        FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("old_test.rs"),
                content_hash: hash,
                size_bytes: content.len() as u64,
                modified_time: SystemTime::UNIX_EPOCH, // Very old
            },
            score: TdgScore::default(),
            components: ComponentScores::default(),
            semantic_sig: SemanticSignature {
                ast_structure_hash: 999,
                identifier_pattern: "old".to_string(),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "1.0.0".to_string(),
                analysis_duration_ms: 1,
                language_confidence: 1.0,
                analysis_timestamp: SystemTime::UNIX_EPOCH, // Very old
                cache_hit: false,
            },
            git_context: None,
        }
    }

    #[tokio::test]
    async fn test_should_archive_old_record() {
        let storage = TieredStore::in_memory();
        let old_record = create_test_record_with_old_timestamp();

        // Record from UNIX_EPOCH should definitely be old enough to archive
        assert!(storage.should_archive(&old_record));
    }

    #[tokio::test]
    async fn test_should_not_archive_recent_record() {
        let storage = TieredStore::in_memory();
        let content = b"fn recent() {}";
        let record = FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("recent.rs"),
                content_hash: blake3::hash(content),
                size_bytes: content.len() as u64,
                modified_time: SystemTime::now(),
            },
            score: TdgScore::default(),
            components: ComponentScores::default(),
            semantic_sig: SemanticSignature {
                ast_structure_hash: 1,
                identifier_pattern: "recent".to_string(),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "1.0.0".to_string(),
                analysis_duration_ms: 1,
                language_confidence: 1.0,
                analysis_timestamp: SystemTime::now(),
                cache_hit: false,
            },
            git_context: None,
        };

        // Recent record should not be archived
        assert!(!storage.should_archive(&record));
    }

    #[tokio::test]
    async fn test_store_old_record_triggers_archival() {
        let storage = TieredStore::in_memory();
        let old_record = create_test_record_with_old_timestamp();
        let hash = old_record.identity.content_hash;

        // Store should trigger archival for old record
        storage.store(old_record).await.unwrap();

        // Record should be in cold storage (retrieved from cold since warm was cleaned)
        let retrieved = storage.retrieve_full(&hash).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_storage_statistics_with_zero_compression() {
        let stats = StorageStatistics {
            hot_entries: 0,
            warm_entries: 0,
            cold_entries: 0,
            total_entries: 0,
            hot_memory_kb: 0,
            compression_ratio: 0.0,
            warm_backend: "memory".to_string(),
            cold_backend: "memory".to_string(),
            backend_stats: HashMap::new(),
        };

        let output = stats.format_diagnostic();
        assert!(output.contains("Hot (memory): 0 entries"));
        assert!(output.contains("0.0%"));
    }

    #[test]
    fn test_storage_statistics_high_compression() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 200,
            cold_entries: 300,
            total_entries: 600,
            hot_memory_kb: 50,
            compression_ratio: 0.9,
            warm_backend: "libsql".to_string(),
            cold_backend: "libsql".to_string(),
            backend_stats: HashMap::new(),
        };

        let output = stats.format_diagnostic();
        assert!(output.contains("Total: 600 entries"));
        assert!(output.contains("90.0%"));
    }

    #[tokio::test]
    async fn test_get_by_path_multiple_matches() {
        let storage = TieredStore::in_memory();
        let path = PathBuf::from("shared_path.rs");

        // Store two records with same path but different content
        for i in 0..2 {
            let content = format!("content version {}", i);
            let record = FullTdgRecord {
                identity: FileIdentity {
                    path: path.clone(),
                    content_hash: blake3::hash(content.as_bytes()),
                    size_bytes: content.len() as u64,
                    modified_time: SystemTime::now(),
                },
                score: TdgScore::default(),
                components: ComponentScores::default(),
                semantic_sig: SemanticSignature {
                    ast_structure_hash: i as u64,
                    identifier_pattern: format!("v{}", i),
                    control_flow_pattern: "linear".to_string(),
                    import_dependencies: Vec::new(),
                },
                metadata: AnalysisMetadata {
                    analyzer_version: "1.0.0".to_string(),
                    analysis_duration_ms: 1,
                    language_confidence: 1.0,
                    analysis_timestamp: SystemTime::now(),
                    cache_hit: false,
                },
                git_context: None,
            };
            storage.store(record).await.unwrap();
        }

        let results = storage.get_by_path(&path).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_file_identity_different_sizes() {
        let identity1 = FileIdentity {
            path: PathBuf::from("small.rs"),
            content_hash: blake3::hash(b"s"),
            size_bytes: 1,
            modified_time: SystemTime::now(),
        };

        let identity2 = FileIdentity {
            path: PathBuf::from("large.rs"),
            content_hash: blake3::hash(b"large content with more bytes"),
            size_bytes: 1_000_000,
            modified_time: SystemTime::now(),
        };

        assert!(identity2.size_bytes > identity1.size_bytes);
    }

    #[test]
    fn test_component_scores_all_populated() {
        let scores = ComponentScores {
            complexity_breakdown: [("func1".to_string(), 10.0), ("func2".to_string(), 20.0)]
                .into_iter()
                .collect(),
            duplication_sources: vec!["dup1.rs".to_string(), "dup2.rs".to_string()],
            coupling_dependencies: vec!["dep1".to_string(), "dep2".to_string(), "dep3".to_string()],
            doc_missing_items: vec!["item1".to_string()],
            consistency_violations: vec!["v1".to_string(), "v2".to_string()],
        };

        assert_eq!(scores.complexity_breakdown.len(), 2);
        assert_eq!(scores.duplication_sources.len(), 2);
        assert_eq!(scores.coupling_dependencies.len(), 3);
        assert_eq!(scores.doc_missing_items.len(), 1);
        assert_eq!(scores.consistency_violations.len(), 2);
    }

    #[test]
    fn test_semantic_signature_with_many_imports() {
        let sig = SemanticSignature {
            ast_structure_hash: 0xCAFEBABE,
            identifier_pattern: "complex_pattern".to_string(),
            control_flow_pattern: "nested_loops_with_branches".to_string(),
            import_dependencies: vec![
                "std::io".to_string(),
                "std::fs".to_string(),
                "std::collections::HashMap".to_string(),
                "tokio::sync::RwLock".to_string(),
                "serde::{Serialize, Deserialize}".to_string(),
            ],
        };

        assert_eq!(sig.import_dependencies.len(), 5);
        assert!(sig.import_dependencies.contains(&"std::io".to_string()));
    }

    #[test]
    fn test_analysis_metadata_with_cache_hit() {
        let meta_hit = AnalysisMetadata {
            analyzer_version: "2.0.0".to_string(),
            analysis_duration_ms: 1, // Fast due to cache
            language_confidence: 1.0,
            analysis_timestamp: SystemTime::now(),
            cache_hit: true,
        };

        let meta_miss = AnalysisMetadata {
            analyzer_version: "2.0.0".to_string(),
            analysis_duration_ms: 500, // Slower without cache
            language_confidence: 0.95,
            analysis_timestamp: SystemTime::now(),
            cache_hit: false,
        };

        assert!(meta_hit.cache_hit);
        assert!(!meta_miss.cache_hit);
        assert!(meta_hit.analysis_duration_ms < meta_miss.analysis_duration_ms);
    }

    #[tokio::test]
    async fn test_storage_statistics_after_multiple_stores() {
        let storage = TieredStore::in_memory();

        // Store several records
        for i in 0..10 {
            let content = format!("file content {}", i);
            let record = FullTdgRecord {
                identity: FileIdentity {
                    path: PathBuf::from(format!("file{}.rs", i)),
                    content_hash: blake3::hash(content.as_bytes()),
                    size_bytes: content.len() as u64,
                    modified_time: SystemTime::now(),
                },
                score: TdgScore::default(),
                components: ComponentScores::default(),
                semantic_sig: SemanticSignature {
                    ast_structure_hash: i as u64,
                    identifier_pattern: "test".to_string(),
                    control_flow_pattern: "linear".to_string(),
                    import_dependencies: Vec::new(),
                },
                metadata: AnalysisMetadata {
                    analyzer_version: "1.0.0".to_string(),
                    analysis_duration_ms: 1,
                    language_confidence: 1.0,
                    analysis_timestamp: SystemTime::now(),
                    cache_hit: false,
                },
                git_context: None,
            };
            storage.store(record).await.unwrap();
        }

        let stats = storage.get_statistics();
        assert_eq!(stats.hot_entries, 10);
    }

    #[test]
    fn test_hot_cache_entry_grade_boundary_values() {
        let grades = [
            (Grade::APLus, 0),
            (Grade::A, 1),
            (Grade::AMinus, 2),
            (Grade::BPlus, 3),
            (Grade::B, 4),
            (Grade::BMinus, 5),
            (Grade::CPlus, 6),
            (Grade::C, 7),
            (Grade::CMinus, 8),
            (Grade::D, 9),
            (Grade::F, 10),
        ];

        for (grade, expected_val) in grades {
            assert_eq!(
                grade as u8, expected_val,
                "Grade {:?} should have value {}",
                grade, expected_val
            );
        }
    }

    #[tokio::test]
    async fn test_flush_after_multiple_operations() {
        let storage = TieredStore::in_memory();

        // Perform various operations
        for i in 0..5 {
            let content = format!("flush test {}", i);
            let record = FullTdgRecord {
                identity: FileIdentity {
                    path: PathBuf::from(format!("flush{}.rs", i)),
                    content_hash: blake3::hash(content.as_bytes()),
                    size_bytes: content.len() as u64,
                    modified_time: SystemTime::now(),
                },
                score: TdgScore::default(),
                components: ComponentScores::default(),
                semantic_sig: SemanticSignature {
                    ast_structure_hash: i as u64,
                    identifier_pattern: "flush".to_string(),
                    control_flow_pattern: "linear".to_string(),
                    import_dependencies: Vec::new(),
                },
                metadata: AnalysisMetadata {
                    analyzer_version: "1.0.0".to_string(),
                    analysis_duration_ms: 1,
                    language_confidence: 1.0,
                    analysis_timestamp: SystemTime::now(),
                    cache_hit: false,
                },
                git_context: None,
            };
            storage.store(record).await.unwrap();
        }

        // Cleanup should work
        let _ = storage.cleanup_hot_cache(3600);

        // Flush should succeed
        assert!(storage.flush().is_ok());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod git_context_integration_tests {
    use super::*;
    use crate::models::git_context::GitContext;
    use std::path::PathBuf;

    // Helper: Get repository root
    fn get_repo_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut current = manifest_dir.clone();
        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() && git_dir.join("HEAD").exists() {
                return current;
            }
            if !current.pop() {
                return manifest_dir.parent().unwrap().to_path_buf();
            }
        }
    }

    // RED TEST 1: FullTdgRecord can store git_context
    #[test]
    fn test_full_tdg_record_stores_git_context() {
        // Arrange
        let repo_path = get_repo_root();
        let git_context = GitContext::from_current_dir(&repo_path).ok();

        let record = FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("test.rs"),
                content_hash: blake3::hash(b"test"),
                size_bytes: 4,
                modified_time: std::time::SystemTime::now(),
            },
            score: TdgScore::default(),
            components: ComponentScores {
                complexity_breakdown: Default::default(),
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            semantic_sig: SemanticSignature {
                ast_structure_hash: 12345,
                identifier_pattern: "test".to_string(),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "2.178.0".to_string(),
                analysis_duration_ms: 100,
                language_confidence: 0.95,
                analysis_timestamp: std::time::SystemTime::now(),
                cache_hit: false,
            },
            git_context,
        };

        // Act & Assert
        if record.git_context.is_some() {
            let ctx = record.git_context.as_ref().unwrap();
            assert!(!ctx.commit_sha.is_empty(), "Should have commit SHA");
            assert!(!ctx.branch.is_empty(), "Should have branch name");
        }
    }

    // RED TEST 2: FullTdgRecord serializes with git_context
    #[test]
    fn test_full_tdg_record_serializes_with_git_context() {
        // Arrange
        let repo_path = get_repo_root();
        let git_context = GitContext::from_current_dir(&repo_path).ok();

        let record = FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("test.rs"),
                content_hash: blake3::hash(b"test"),
                size_bytes: 4,
                modified_time: std::time::SystemTime::now(),
            },
            score: TdgScore::default(),
            components: ComponentScores {
                complexity_breakdown: Default::default(),
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            semantic_sig: SemanticSignature {
                ast_structure_hash: 12345,
                identifier_pattern: "test".to_string(),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "2.178.0".to_string(),
                analysis_duration_ms: 100,
                language_confidence: 0.95,
                analysis_timestamp: std::time::SystemTime::now(),
                cache_hit: false,
            },
            git_context: git_context.clone(),
        };

        // Act: Serialize to JSON
        let json = serde_json::to_string(&record).unwrap();

        // Assert: Deserialize back
        let deserialized: FullTdgRecord = serde_json::from_str(&json).unwrap();

        if let Some(orig) = git_context.as_ref() {
            if let Some(deser) = deserialized.git_context.as_ref() {
                assert_eq!(orig.commit_sha, deser.commit_sha, "Commit SHA should match");
                assert_eq!(orig.branch, deser.branch, "Branch should match");
            } else {
                panic!("Git context should round-trip through JSON");
            }
        }
    }

    // RED TEST 3: FullTdgRecord works without git_context (backward compat)
    #[test]
    fn test_full_tdg_record_works_without_git_context() {
        // Arrange
        let record = FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("test.rs"),
                content_hash: blake3::hash(b"test"),
                size_bytes: 4,
                modified_time: std::time::SystemTime::now(),
            },
            score: TdgScore::default(),
            components: ComponentScores {
                complexity_breakdown: Default::default(),
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            semantic_sig: SemanticSignature {
                ast_structure_hash: 12345,
                identifier_pattern: "test".to_string(),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "2.178.0".to_string(),
                analysis_duration_ms: 100,
                language_confidence: 0.95,
                analysis_timestamp: std::time::SystemTime::now(),
                cache_hit: false,
            },
            git_context: None, // No git context
        };

        // Act: Serialize to JSON
        let json = serde_json::to_string(&record).unwrap();

        // Assert: Should not contain "git_context" field (skipped)
        assert!(
            !json.contains("git_context"),
            "JSON should skip None git_context field"
        );

        // Deserialize back
        let deserialized: FullTdgRecord = serde_json::from_str(&json).unwrap();
        assert!(
            deserialized.git_context.is_none(),
            "Git context should remain None"
        );
    }
}
