    #[test]
    fn test_cosine_similarity() {
        // Identical vectors
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);

        // Orthogonal vectors
        let v3 = vec![1.0, 0.0, 0.0];
        let v4 = vec![0.0, 1.0, 0.0];
        let sim2 = TursoVectorDB::cosine_similarity(&v3, &v4);
        assert!((sim2 - 0.0).abs() < 0.001);

        // Opposite vectors
        let v5 = vec![1.0, 0.0, 0.0];
        let v6 = vec![-1.0, 0.0, 0.0];
        let sim3 = TursoVectorDB::cosine_similarity(&v5, &v6);
        assert!((sim3 + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_serialization() {
        let embedding = vec![0.1, 0.2, 0.3];
        let json = serde_json::to_string(&embedding).expect("internal error");
        let deserialized: Vec<f32> = serde_json::from_str(&json).expect("internal error");
        assert_eq!(embedding, deserialized);
    }

    // ============ EmbeddingEntry Tests ============

    #[test]
    fn test_embedding_entry_creation() {
        let entry = EmbeddingEntry {
            file_path: "src/main.rs".to_string(),
            chunk_name: "main".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 10,
            content_checksum: "abc123".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            model: "text-embedding-3-small".to_string(),
        };
        assert_eq!(entry.file_path, "src/main.rs");
        assert_eq!(entry.chunk_name, "main");
        assert_eq!(entry.embedding.len(), 3);
    }

    #[test]
    fn test_embedding_entry_clone() {
        let entry = EmbeddingEntry {
            file_path: "test.rs".to_string(),
            chunk_name: "test_fn".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 5,
            end_line: 15,
            content_checksum: "def456".to_string(),
            embedding: vec![0.5, 0.6],
            model: "model-v1".to_string(),
        };
        let cloned = entry.clone();
        assert_eq!(cloned.file_path, entry.file_path);
        assert_eq!(cloned.embedding, entry.embedding);
    }

    #[test]
    fn test_embedding_entry_debug() {
        let entry = EmbeddingEntry {
            file_path: "a.rs".to_string(),
            chunk_name: "b".to_string(),
            chunk_type: "c".to_string(),
            language: "d".to_string(),
            start_line: 0,
            end_line: 0,
            content_checksum: "e".to_string(),
            embedding: vec![],
            model: "f".to_string(),
        };
        let debug = format!("{:?}", entry);
        assert!(debug.contains("EmbeddingEntry"));
    }

    // ============ SearchResult Tests ============

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            id: 42,
            file_path: "src/lib.rs".to_string(),
            chunk_name: "process".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 100,
            end_line: 150,
            similarity: 0.95,
            embedding: vec![0.1, 0.2],
        };
        assert_eq!(result.id, 42);
        assert_eq!(result.similarity, 0.95);
    }

    #[test]
    fn test_search_result_clone() {
        let result = SearchResult {
            id: 1,
            file_path: "test.rs".to_string(),
            chunk_name: "test".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 5,
            similarity: 0.8,
            embedding: vec![0.3],
        };
        let cloned = result.clone();
        assert_eq!(cloned.id, result.id);
        assert_eq!(cloned.similarity, result.similarity);
    }

    #[test]
    fn test_search_result_debug() {
        let result = SearchResult {
            id: 0,
            file_path: "".to_string(),
            chunk_name: "".to_string(),
            chunk_type: "".to_string(),
            language: "".to_string(),
            start_line: 0,
            end_line: 0,
            similarity: 0.0,
            embedding: vec![],
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("SearchResult"));
    }

    // ============ Cosine Similarity Edge Cases ============

    #[test]
    fn test_cosine_similarity_empty_vectors() {
        let v1: Vec<f32> = vec![];
        let v2: Vec<f32> = vec![];
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!(sim.is_nan() || sim == 0.0);
    }

    #[test]
    fn test_cosine_similarity_single_element() {
        let v1 = vec![1.0];
        let v2 = vec![1.0];
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_large_vectors() {
        let v1: Vec<f32> = (0..1536).map(|i| (i as f32) * 0.001).collect();
        let v2: Vec<f32> = (0..1536).map(|i| (i as f32) * 0.001).collect();
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_magnitudes() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![2.0, 4.0, 6.0]; // Same direction, 2x magnitude
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_negative_values() {
        let v1 = vec![-1.0, -2.0, -3.0];
        let v2 = vec![-1.0, -2.0, -3.0];
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);
    }

    // ============ Database Operations Tests (In-Memory) ============

    #[tokio::test]
    async fn test_new_local_in_memory() {
        let result = TursoVectorDB::new_local(":memory:").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insert_and_retrieve() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        let entry = EmbeddingEntry {
            file_path: "src/main.rs".to_string(),
            chunk_name: "main".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 10,
            content_checksum: "checksum123".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            model: "test-model".to_string(),
        };

        let id = db.insert(&entry).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_persistence_round_trip() {
        // #568: entries saved by one TursoVectorDB load into a fresh one opened
        // on the same path (this is what makes `embed sync` -> `semantic search`
        // work across separate CLI processes).
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("embeddings.db");
        let path_str = db_path.to_string_lossy().to_string();

        let make = |name: &str, file: &str| EmbeddingEntry {
            file_path: file.to_string(),
            chunk_name: name.to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 10,
            content_checksum: format!("sum_{name}"),
            embedding: vec![0.1, 0.2, 0.3, 0.4],
            model: "aprender-tfidf-local".to_string(),
        };

        {
            let db = TursoVectorDB::new_local(path_str.as_str()).await.unwrap();
            db.insert(&make("alpha", "src/a.rs")).await.unwrap();
            db.insert(&make("beta", "src/b.rs")).await.unwrap();
            db.save().await.unwrap();
        }
        assert!(db_path.exists(), "save() must write the db file");

        // A fresh instance rebuilds the store from the persisted entries.
        let reloaded = TursoVectorDB::new_local(path_str.as_str()).await.unwrap();
        let stats = reloaded.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 2, "reloaded store must have both entries");
        assert_eq!(stats.unique_files, 2);
    }

    #[tokio::test]
    async fn test_get_entry() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        let entry = EmbeddingEntry {
            file_path: "test.rs".to_string(),
            chunk_name: "test_fn".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 5,
            end_line: 20,
            content_checksum: "abc".to_string(),
            embedding: vec![0.5, 0.6, 0.7],
            model: "model".to_string(),
        };

        db.insert(&entry).await.unwrap();
        let result = db.get_entry("test.rs", "test_fn").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_similarity_search() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        // Insert some entries
        for i in 0..5 {
            let entry = EmbeddingEntry {
                file_path: format!("file{}.rs", i),
                chunk_name: format!("func{}", i),
                chunk_type: "function".to_string(),
                language: "rust".to_string(),
                start_line: i,
                end_line: i + 10,
                content_checksum: format!("checksum{}", i),
                embedding: vec![i as f32 * 0.1, 0.5, 0.5],
                model: "test".to_string(),
            };
            db.insert(&entry).await.unwrap();
        }

        let query = vec![0.2, 0.5, 0.5];
        let results = db.similarity_search(&query, 3).await.unwrap();
        assert!(results.len() <= 3);
    }

    #[tokio::test]
    async fn test_delete_file_entries() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        let entry = EmbeddingEntry {
            file_path: "to_delete.rs".to_string(),
            chunk_name: "func".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: 0,
            end_line: 5,
            content_checksum: "del".to_string(),
            embedding: vec![0.1],
            model: "m".to_string(),
        };

        db.insert(&entry).await.unwrap();
        let deleted = db.delete_file_entries("to_delete.rs").await.unwrap();
        assert_eq!(deleted, 1);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        // Insert entries
        for i in 0..3 {
            let entry = EmbeddingEntry {
                file_path: format!("file{}.rs", i),
                chunk_name: "func".to_string(),
                chunk_type: "function".to_string(),
                language: "rust".to_string(),
                start_line: 0,
                end_line: 1,
                content_checksum: format!("cs{}", i),
                embedding: vec![0.1],
                model: "m".to_string(),
            };
            db.insert(&entry).await.unwrap();
        }

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.unique_files, 3);
    }

    #[tokio::test]
    async fn test_query_by_language() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        // Insert Rust entries
        for i in 0..3 {
            let entry = EmbeddingEntry {
                file_path: format!("file{}.rs", i),
                chunk_name: format!("func{}", i),
                chunk_type: "function".to_string(),
                language: "rust".to_string(),
                start_line: 0,
                end_line: 10,
                content_checksum: format!("cs{}", i),
                embedding: vec![0.1, 0.2, 0.3],
                model: "test".to_string(),
            };
            db.insert(&entry).await.unwrap();
        }

        // Insert Python entry
        let py_entry = EmbeddingEntry {
            file_path: "script.py".to_string(),
            chunk_name: "main".to_string(),
            chunk_type: "function".to_string(),
            language: "python".to_string(),
            start_line: 0,
            end_line: 5,
            content_checksum: "py_cs".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            model: "test".to_string(),
        };
        db.insert(&py_entry).await.unwrap();

        let rust_results = db.query_by_language("rust").await.unwrap();
        assert_eq!(rust_results.len(), 3);

        let python_results = db.query_by_language("python").await.unwrap();
        assert_eq!(python_results.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_insert() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();

        let entries: Vec<EmbeddingEntry> = (0..5)
            .map(|i| EmbeddingEntry {
                file_path: format!("batch{}.rs", i),
                chunk_name: format!("func{}", i),
                chunk_type: "function".to_string(),
                language: "rust".to_string(),
                start_line: i,
                end_line: i + 10,
                content_checksum: format!("batch_cs{}", i),
                embedding: vec![i as f32 * 0.1, 0.5],
                model: "test".to_string(),
            })
            .collect();

        let ids = db.batch_insert(&entries).await.unwrap();
        assert_eq!(ids.len(), 5);

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 5);
    }

