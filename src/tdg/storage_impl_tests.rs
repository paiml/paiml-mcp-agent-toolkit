#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod storage_impl_tests {
    use super::*;
    use crate::tdg::Grade;
    use crate::tdg::language_simple::Language;

    // ============================================================================
    // FileIdentity Tests
    // ============================================================================

    fn create_test_file_identity() -> FileIdentity {
        FileIdentity {
            path: PathBuf::from("src/test.rs"),
            content_hash: blake3::hash(b"test content"),
            size_bytes: 1024,
            modified_time: SystemTime::now(),
        }
    }

    #[test]
    fn test_file_identity_creation() {
        let identity = create_test_file_identity();
        assert_eq!(identity.path, PathBuf::from("src/test.rs"));
        assert_eq!(identity.size_bytes, 1024);
    }

    #[test]
    fn test_file_identity_clone() {
        let identity = create_test_file_identity();
        let cloned = identity.clone();
        assert_eq!(identity.path, cloned.path);
        assert_eq!(identity.size_bytes, cloned.size_bytes);
        assert_eq!(identity.content_hash, cloned.content_hash);
    }

    #[test]
    fn test_file_identity_serialization() {
        let identity = create_test_file_identity();
        let json = serde_json::to_string(&identity).unwrap();
        let deserialized: FileIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(identity.path, deserialized.path);
        assert_eq!(identity.size_bytes, deserialized.size_bytes);
    }

    // ============================================================================
    // ComponentScores Tests
    // ============================================================================

    #[test]
    fn test_component_scores_default() {
        let scores = ComponentScores::default();
        assert!(scores.complexity_breakdown.is_empty());
        assert!(scores.duplication_sources.is_empty());
        assert!(scores.coupling_dependencies.is_empty());
        assert!(scores.doc_missing_items.is_empty());
        assert!(scores.consistency_violations.is_empty());
    }

    #[test]
    fn test_component_scores_with_data() {
        let mut complexity = HashMap::new();
        complexity.insert("function_a".to_string(), 15.0);
        complexity.insert("function_b".to_string(), 8.5);

        let scores = ComponentScores {
            complexity_breakdown: complexity,
            duplication_sources: vec!["file_a.rs".to_string()],
            coupling_dependencies: vec!["mod_x".to_string(), "mod_y".to_string()],
            doc_missing_items: vec!["function_c".to_string()],
            consistency_violations: vec![],
        };

        assert_eq!(scores.complexity_breakdown.len(), 2);
        assert_eq!(scores.duplication_sources.len(), 1);
        assert_eq!(scores.coupling_dependencies.len(), 2);
        assert_eq!(scores.doc_missing_items.len(), 1);
    }

    #[test]
    fn test_component_scores_serialization() {
        let scores = ComponentScores::default();
        let json = serde_json::to_string(&scores).unwrap();
        let deserialized: ComponentScores = serde_json::from_str(&json).unwrap();
        assert!(deserialized.complexity_breakdown.is_empty());
    }

    // ============================================================================
    // SemanticSignature Tests
    // ============================================================================

    fn create_test_semantic_signature() -> SemanticSignature {
        SemanticSignature {
            ast_structure_hash: 12345678,
            identifier_pattern: "snake_case".to_string(),
            control_flow_pattern: "linear".to_string(),
            import_dependencies: vec!["std".to_string(), "tokio".to_string()],
        }
    }

    #[test]
    fn test_semantic_signature_creation() {
        let sig = create_test_semantic_signature();
        assert_eq!(sig.ast_structure_hash, 12345678);
        assert_eq!(sig.identifier_pattern, "snake_case");
        assert_eq!(sig.control_flow_pattern, "linear");
        assert_eq!(sig.import_dependencies.len(), 2);
    }

    #[test]
    fn test_semantic_signature_clone() {
        let sig = create_test_semantic_signature();
        let cloned = sig.clone();
        assert_eq!(sig.ast_structure_hash, cloned.ast_structure_hash);
        assert_eq!(sig.identifier_pattern, cloned.identifier_pattern);
    }

    #[test]
    fn test_semantic_signature_serialization() {
        let sig = create_test_semantic_signature();
        let json = serde_json::to_string(&sig).unwrap();
        let deserialized: SemanticSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(sig.ast_structure_hash, deserialized.ast_structure_hash);
    }

    // ============================================================================
    // AnalysisMetadata Tests
    // ============================================================================

    fn create_test_analysis_metadata() -> AnalysisMetadata {
        AnalysisMetadata {
            analyzer_version: "2.0.0".to_string(),
            analysis_duration_ms: 150,
            language_confidence: 0.95,
            analysis_timestamp: SystemTime::now(),
            cache_hit: false,
        }
    }

    #[test]
    fn test_analysis_metadata_creation() {
        let metadata = create_test_analysis_metadata();
        assert_eq!(metadata.analyzer_version, "2.0.0");
        assert_eq!(metadata.analysis_duration_ms, 150);
        assert!((metadata.language_confidence - 0.95).abs() < 0.001);
        assert!(!metadata.cache_hit);
    }

    #[test]
    fn test_analysis_metadata_with_cache_hit() {
        let metadata = AnalysisMetadata {
            analyzer_version: "2.0.0".to_string(),
            analysis_duration_ms: 5,
            language_confidence: 1.0,
            analysis_timestamp: SystemTime::now(),
            cache_hit: true,
        };
        assert!(metadata.cache_hit);
        assert_eq!(metadata.analysis_duration_ms, 5);
    }

    #[test]
    fn test_analysis_metadata_serialization() {
        let metadata = create_test_analysis_metadata();
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: AnalysisMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.analyzer_version, deserialized.analyzer_version);
        assert_eq!(metadata.analysis_duration_ms, deserialized.analysis_duration_ms);
    }

    // ============================================================================
    // HotCacheEntry Tests
    // ============================================================================

    fn create_test_full_tdg_record() -> FullTdgRecord {
        FullTdgRecord {
            identity: create_test_file_identity(),
            score: TdgScore {
                structural_complexity: 5.0,
                semantic_complexity: 3.0,
                duplication_ratio: 0.1,
                coupling_score: 2.0,
                doc_coverage: 0.8,
                consistency_score: 0.9,
                entropy_score: 0.5,
                total: 82.5,
                grade: Grade::B,
                confidence: 0.95,
                language: Language::Rust,
                file_path: Some(PathBuf::from("src/test.rs")),
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            components: ComponentScores::default(),
            semantic_sig: create_test_semantic_signature(),
            metadata: create_test_analysis_metadata(),
            git_context: None,
        }
    }

    #[test]
    fn test_hot_cache_entry_from_record() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);

        assert_eq!(entry.grade, Grade::B as u8);
        assert!((entry.total_score - 82.5).abs() < 0.001);
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_hot_cache_entry_hash_bytes() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);

        // Hash bytes should be 32 bytes (Blake3)
        assert_eq!(entry.content_hash.len(), 32);
    }

    #[test]
    fn test_hot_cache_entry_copy() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);
        let copied = entry; // Should work since HotCacheEntry is Copy

        assert_eq!(entry.grade, copied.grade);
        assert_eq!(entry.total_score, copied.total_score);
    }

    // ============================================================================
    // FullTdgRecord Tests
    // ============================================================================

    #[test]
    fn test_full_tdg_record_creation() {
        let record = create_test_full_tdg_record();
        assert_eq!(record.score.grade, Grade::B);
        assert!((record.score.total - 82.5).abs() < 0.001);
        assert!(record.git_context.is_none());
    }

    #[test]
    fn test_full_tdg_record_clone() {
        let record = create_test_full_tdg_record();
        let cloned = record.clone();
        assert_eq!(record.score.grade, cloned.score.grade);
        assert_eq!(record.identity.path, cloned.identity.path);
    }

    #[test]
    fn test_full_tdg_record_serialization() {
        let record = create_test_full_tdg_record();
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: FullTdgRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record.score.grade, deserialized.score.grade);
    }

    // ============================================================================
    // TieredStore Tests
    // ============================================================================

    #[test]
    fn test_tiered_store_in_memory() {
        let store = TieredStore::in_memory();
        let stats = store.get_statistics();
        assert_eq!(stats.hot_entries, 0);
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_tiered_store_store_and_get_hot() {
        let store = TieredStore::in_memory();
        let record = create_test_full_tdg_record();
        let hash = record.identity.content_hash;

        store.store(record).await.unwrap();

        let hot_entry = store.get_hot(&hash);
        assert!(hot_entry.is_some());

        let entry = hot_entry.unwrap();
        assert_eq!(entry.grade, Grade::B as u8);
    }

    #[tokio::test]
    async fn test_tiered_store_retrieve_full() {
        let store = TieredStore::in_memory();
        let record = create_test_full_tdg_record();
        let hash = record.identity.content_hash;

        store.store(record.clone()).await.unwrap();

        let retrieved = store.retrieve_full(&hash).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_record = retrieved.unwrap();
        assert_eq!(retrieved_record.score.grade, record.score.grade);
    }

    #[tokio::test]
    async fn test_tiered_store_retrieve_nonexistent() {
        let store = TieredStore::in_memory();
        let hash = blake3::hash(b"nonexistent");

        let retrieved = store.retrieve_full(&hash).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_tiered_store_cleanup_hot_cache() {
        let store = TieredStore::in_memory();

        // Add some entries directly to hot cache
        let hash1 = blake3::hash(b"test1");
        let hash2 = blake3::hash(b"test2");

        // Entry with old timestamp
        let old_entry = HotCacheEntry {
            content_hash: [0u8; 32],
            grade: 0,
            total_score: 50.0,
            timestamp: 0, // Very old
        };

        // Entry with current timestamp
        let new_entry = HotCacheEntry {
            content_hash: [0u8; 32],
            grade: 0,
            total_score: 50.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        store.hot.insert(hash1, old_entry);
        store.hot.insert(hash2, new_entry);

        // Cleanup entries older than 10 seconds
        let removed = store.cleanup_hot_cache(10);

        // Old entry should be removed
        assert_eq!(removed, 1);
        assert!(store.hot.get(&hash1).is_none());
        assert!(store.hot.get(&hash2).is_some());
    }

    #[test]
    fn test_tiered_store_flush() {
        let store = TieredStore::in_memory();
        let result = store.flush();
        assert!(result.is_ok());
    }

    // ============================================================================
    // StorageStatistics Tests
    // ============================================================================

    #[test]
    fn test_storage_statistics_creation() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 500,
            cold_entries: 2000,
            total_entries: 2600,
            hot_memory_kb: 50,
            compression_ratio: 0.33,
            warm_backend: "sled".to_string(),
            cold_backend: "sled".to_string(),
            backend_stats: HashMap::new(),
        };

        assert_eq!(stats.hot_entries, 100);
        assert_eq!(stats.warm_entries, 500);
        assert_eq!(stats.cold_entries, 2000);
        assert_eq!(stats.total_entries, 2600);
    }

    #[test]
    fn test_storage_statistics_format_diagnostic() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 500,
            cold_entries: 2000,
            total_entries: 2600,
            hot_memory_kb: 50,
            compression_ratio: 0.33,
            warm_backend: "sled".to_string(),
            cold_backend: "sled".to_string(),
            backend_stats: HashMap::new(),
        };

        let diagnostic = stats.format_diagnostic();
        assert!(diagnostic.contains("Hot (memory): 100 entries"));
        assert!(diagnostic.contains("Warm (sled backend): 500 entries"));
        assert!(diagnostic.contains("Cold (sled backend): 2000 entries"));
        assert!(diagnostic.contains("Total: 2600 entries"));
        assert!(diagnostic.contains("Compression ratio: 33.0%"));
    }

    #[test]
    fn test_storage_statistics_serialization() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 500,
            cold_entries: 2000,
            total_entries: 2600,
            hot_memory_kb: 50,
            compression_ratio: 0.33,
            warm_backend: "sled".to_string(),
            cold_backend: "sled".to_string(),
            backend_stats: HashMap::new(),
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: StorageStatistics = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.hot_entries, deserialized.hot_entries);
        assert_eq!(stats.total_entries, deserialized.total_entries);
    }

    // ============================================================================
    // TieredStorageFactory Tests
    // ============================================================================

    #[test]
    fn test_tiered_storage_factory_in_memory() {
        let store = TieredStorageFactory::create_in_memory();
        let stats = store.get_statistics();
        assert_eq!(stats.hot_entries, 0);
    }

    // ============================================================================
    // Debug Trait Tests
    // ============================================================================

    #[test]
    fn test_file_identity_debug() {
        let identity = create_test_file_identity();
        let debug = format!("{:?}", identity);
        assert!(debug.contains("FileIdentity"));
        assert!(debug.contains("src/test.rs"));
    }

    #[test]
    fn test_component_scores_debug() {
        let scores = ComponentScores::default();
        let debug = format!("{:?}", scores);
        assert!(debug.contains("ComponentScores"));
    }

    #[test]
    fn test_semantic_signature_debug() {
        let sig = create_test_semantic_signature();
        let debug = format!("{:?}", sig);
        assert!(debug.contains("SemanticSignature"));
        assert!(debug.contains("snake_case"));
    }

    #[test]
    fn test_analysis_metadata_debug() {
        let metadata = create_test_analysis_metadata();
        let debug = format!("{:?}", metadata);
        assert!(debug.contains("AnalysisMetadata"));
        assert!(debug.contains("2.0.0"));
    }

    #[test]
    fn test_hot_cache_entry_debug() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);
        let debug = format!("{:?}", entry);
        assert!(debug.contains("HotCacheEntry"));
    }
}
