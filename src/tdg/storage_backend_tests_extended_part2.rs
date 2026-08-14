    // ============ StorageBackendFactory Tests ============

    #[test]
    fn test_factory_create_in_memory() {
        let backend = StorageBackendFactory::create_in_memory();
        assert_eq!(backend.backend_name(), "in-memory");
    }

    #[test]
    fn test_factory_create_default() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("default.db");
        let backend = StorageBackendFactory::create_default(&db_path).unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    #[test]
    fn test_factory_create_libsql() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backend = StorageBackendFactory::create_libsql(&db_path).unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    #[test]
    fn test_factory_create_libsql_temporary() {
        let backend = StorageBackendFactory::create_libsql_temporary().unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    #[test]
    fn test_factory_create_from_config_inmemory() {
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
    fn test_factory_create_from_config_libsql_no_path() {
        let config = StorageConfig {
            backend_type: StorageBackendType::Libsql,
            path: None,
            cache_size_mb: None,
            compression: false,
        };
        let backend = StorageBackendFactory::create_from_config(&config).unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    #[test]
    fn test_factory_create_from_config_libsql_with_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            backend_type: StorageBackendType::Libsql,
            path: Some(temp_dir.path().join("config.db")),
            cache_size_mb: Some(64),
            compression: true,
        };
        let backend = StorageBackendFactory::create_from_config(&config).unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    // NOTE: Sled and RocksDB tests removed - backends no longer supported

    // ============ Iteration Tests ============

    #[test]
    fn test_in_memory_backend_iter_empty() {
        let backend = InMemoryBackend::new();
        let iter = backend.iter().unwrap();
        let results: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_in_memory_backend_iter_multiple() {
        let backend = InMemoryBackend::new();
        for i in 0..10 {
            backend
                .put(
                    format!("key{}", i).as_bytes(),
                    format!("value{}", i).as_bytes(),
                )
                .unwrap();
        }

        let iter = backend.iter().unwrap();
        let results: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_libsql_backend_iter_empty() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        let iter = backend.iter().unwrap();
        let results: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_libsql_backend_iter_multiple() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        for i in 0..10 {
            backend
                .put(
                    format!("key{}", i).as_bytes(),
                    format!("value{}", i).as_bytes(),
                )
                .unwrap();
        }

        let iter = backend.iter().unwrap();
        let results: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(results.len(), 10);
    }

    // ============ Stats Tests ============

    #[test]
    fn test_in_memory_backend_get_stats() {
        let backend = InMemoryBackend::new();
        backend.put(b"key1", b"value1").unwrap();
        backend.put(b"key2", b"value2").unwrap();

        let stats = backend.get_stats();
        assert_eq!(stats.get("entries").unwrap(), "2");
        assert!(stats.contains_key("memory_bytes"));
    }

    #[test]
    #[ignore] // Slow test - takes >60s
    fn test_libsql_backend_get_stats() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        backend.put(b"key1", b"value1").unwrap();

        let stats = backend.get_stats();
        assert!(stats.contains_key("entries"));
        assert!(stats.contains_key("path"));
    }

    #[test]
    fn test_libsql_backend_get_stats_full() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("stats_test.db");
        let backend = LibsqlBackend::new(&db_path).unwrap();

        backend.put(b"key1", b"value1").unwrap();
        backend.put(b"key2", b"value2").unwrap();
        backend.flush().unwrap();

        let stats = backend.get_stats();
        assert_eq!(stats.get("entries").unwrap(), "2");
        assert!(stats.contains_key("size_bytes"));
        assert!(stats.contains_key("path"));
        assert!(stats.contains_key("page_count"));
    }

    #[test]
    fn test_in_memory_backend_large_values() {
        let backend = InMemoryBackend::new();
        let large_key = vec![0u8; 1024];
        let large_value = vec![1u8; 10240];

        backend.put(&large_key, &large_value).unwrap();
        let retrieved = backend.get(&large_key).unwrap().unwrap();
        assert_eq!(retrieved.len(), 10240);
    }

    #[test]
    fn test_libsql_backend_overwrite() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        backend.put(b"key", b"value1").unwrap();
        assert_eq!(backend.get(b"key").unwrap().unwrap(), b"value1");

        backend.put(b"key", b"value2").unwrap();
        assert_eq!(backend.get(b"key").unwrap().unwrap(), b"value2");
    }

    #[test]
    fn test_libsql_backend_binary_data() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        let binary_key = vec![0u8, 255, 128, 64];
        let binary_value = vec![255u8, 0, 1, 254];

        backend.put(&binary_key, &binary_value).unwrap();
        let retrieved = backend.get(&binary_key).unwrap().unwrap();
        assert_eq!(retrieved, binary_value);
    }

    #[test]
    fn test_storage_trait_via_box_dyn() {
        // Test using trait object
        let backend: Box<dyn StorageBackend> = Box::new(InMemoryBackend::new());

        backend.put(b"key", b"value").unwrap();
        assert!(backend.contains(b"key").unwrap());
        assert_eq!(backend.backend_name(), "in-memory");

        let retrieved = backend.get(b"key").unwrap().unwrap();
        assert_eq!(retrieved, b"value");
    }

    #[test]
    fn test_storage_backend_with_empty_key_value() {
        let backend = InMemoryBackend::new();

        // Empty key
        backend.put(b"", b"value").unwrap();
        assert!(backend.contains(b"").unwrap());
        assert_eq!(backend.get(b"").unwrap().unwrap(), b"value");

        // Empty value
        backend.put(b"key", b"").unwrap();
        assert!(backend.contains(b"key").unwrap());
        assert_eq!(backend.get(b"key").unwrap().unwrap(), b"");
    }

    #[test]
    fn test_libsql_backend_with_empty_key_value() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        // Empty key
        backend.put(b"", b"value").unwrap();
        assert!(backend.contains(b"").unwrap());
        assert_eq!(backend.get(b"").unwrap().unwrap(), b"value");

        // Empty value
        backend.put(b"key", b"").unwrap();
        assert!(backend.contains(b"key").unwrap());
        assert_eq!(backend.get(b"key").unwrap().unwrap(), b"");
    }

    #[test]
    fn test_in_memory_backend_concurrent_access() {
        use std::thread;

        let backend = Arc::new(InMemoryBackend::new());
        let mut handles = vec![];

        for i in 0..10 {
            let backend_clone = Arc::clone(&backend);
            let handle = thread::spawn(move || {
                let key = format!("key{}", i);
                let value = format!("value{}", i);
                backend_clone.put(key.as_bytes(), value.as_bytes()).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all keys were written
        for i in 0..10 {
            let key = format!("key{}", i);
            assert!(backend.contains(key.as_bytes()).unwrap());
        }
    }

    #[test]
    fn test_key_value_pair_type_alias() {
        let pair: KeyValuePair = (vec![1, 2, 3], vec![4, 5, 6]);
        assert_eq!(pair.0, vec![1, 2, 3]);
        assert_eq!(pair.1, vec![4, 5, 6]);
    }

    #[test]
    fn test_libsql_backend_delete_nonexistent() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        // Should not error
        assert!(backend.delete(b"nonexistent").is_ok());
    }

    #[test]
    fn test_libsql_backend_flush() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        backend.put(b"key", b"value").unwrap();
        // Flush should not error
        assert!(backend.flush().is_ok());
    }

    #[test]
    fn test_libsql_backend_file_not_found_size() {
        // Create backend with file that gets deleted
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("to_delete.db");
        let backend = LibsqlBackend::new(&db_path).unwrap();

        // Delete the temp directory to simulate file not found
        drop(temp_dir);

        // A database that cannot be measured must report that, not a size of 0
        // bytes — "0" is a legitimate reading for a healthy empty db, so
        // conflating the two hides the failure. (This test previously accepted
        // either answer and asserted nothing.)
        let err = backend
            .size_on_disk()
            .expect_err("a deleted database file must not be measured as 0 bytes");
        assert!(
            err.to_string().contains("to_delete.db"),
            "error must identify the database it could not measure, got: {err}"
        );
    }

    #[test]
    fn test_storage_backend_type_all_variants() {
        let variants = [StorageBackendType::Libsql, StorageBackendType::InMemory];

        for variant in variants {
            let display = format!("{}", variant);
            assert!(!display.is_empty());

            let debug = format!("{:?}", variant);
            assert!(!debug.is_empty());

            let cloned = variant;
            assert_eq!(cloned, variant);
        }
    }

    #[test]
    fn test_storage_config_with_all_fields() {
        let config = StorageConfig {
            backend_type: StorageBackendType::Libsql,
            path: Some(std::path::PathBuf::from("/var/lib/tdg/data.db")),
            cache_size_mb: Some(512),
            compression: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: StorageConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.backend_type, config.backend_type);
        assert_eq!(deserialized.path, config.path);
        assert_eq!(deserialized.cache_size_mb, config.cache_size_mb);
        assert_eq!(deserialized.compression, config.compression);
    }

    #[test]
    fn test_in_memory_iter_with_modifications() {
        let backend = InMemoryBackend::new();

        // Add entries
        for i in 0..5 {
            backend
                .put(format!("key{}", i).as_bytes(), b"value")
                .unwrap();
        }

        // Get iterator count
        let count1 = backend.iter().unwrap().count();
        assert_eq!(count1, 5);

        // Delete some entries
        backend.delete(b"key0").unwrap();
        backend.delete(b"key2").unwrap();

        // Get new iterator count
        let count2 = backend.iter().unwrap().count();
        assert_eq!(count2, 3);
    }

    #[test]
    fn test_libsql_iter_order_independence() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        backend.put(b"z", b"last").unwrap();
        backend.put(b"a", b"first").unwrap();
        backend.put(b"m", b"middle").unwrap();

        let iter = backend.iter().unwrap();
        let results: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(results.len(), 3);

        // All three keys should be present
        let keys: Vec<_> = results.iter().map(|(k, _)| k.clone()).collect();
        assert!(keys.contains(&b"a".to_vec()));
        assert!(keys.contains(&b"m".to_vec()));
        assert!(keys.contains(&b"z".to_vec()));
    }
