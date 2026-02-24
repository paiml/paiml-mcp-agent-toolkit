    // ============ InMemoryBackend Tests ============

    #[test]
    fn test_in_memory_backend_default() {
        let backend = InMemoryBackend::default();
        assert_eq!(backend.backend_name(), "in-memory");
    }

    #[test]
    fn test_in_memory_backend_flush() {
        let backend = InMemoryBackend::new();
        // Flush should be a no-op but should not error
        assert!(backend.flush().is_ok());
    }

    #[test]
    fn test_in_memory_backend_size_on_disk() {
        let backend = InMemoryBackend::new();

        // Empty backend should have size 0
        assert_eq!(backend.size_on_disk().unwrap(), 0);

        // Add some data
        backend.put(b"key1", b"value1").unwrap();
        backend.put(b"key2", b"value2").unwrap();

        // Size should be sum of key + value lengths
        let expected_size = "key1".len() + "value1".len() + "key2".len() + "value2".len();
        assert_eq!(backend.size_on_disk().unwrap(), expected_size as u64);
    }

    #[test]
    fn test_in_memory_backend_get_nonexistent() {
        let backend = InMemoryBackend::new();
        let result = backend.get(b"nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_in_memory_backend_delete_nonexistent() {
        let backend = InMemoryBackend::new();
        // Deleting nonexistent key should not error
        assert!(backend.delete(b"nonexistent").is_ok());
    }

    #[test]
    fn test_in_memory_backend_overwrite() {
        let backend = InMemoryBackend::new();
        backend.put(b"key", b"value1").unwrap();
        backend.put(b"key", b"value2").unwrap();

        let retrieved = backend.get(b"key").unwrap().unwrap();
        assert_eq!(retrieved, b"value2");
    }

    #[test]
    fn test_in_memory_backend_binary_data() {
        let backend = InMemoryBackend::new();
        let binary_key = vec![0u8, 1, 2, 255, 254];
        let binary_value = vec![255u8, 0, 128, 64, 32];

        backend.put(&binary_key, &binary_value).unwrap();
        let retrieved = backend.get(&binary_key).unwrap().unwrap();
        assert_eq!(retrieved, binary_value);
    }

    // ============ LibsqlBackend Tests ============

    #[test]
    fn test_libsql_backend_temporary() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    #[test]
    fn test_libsql_backend_get_nonexistent() {
        let backend = LibsqlBackend::new_temporary().unwrap();
        let result = backend.get(b"nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_libsql_backend_contains() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        assert!(!backend.contains(b"key").unwrap());
        backend.put(b"key", b"value").unwrap();
        assert!(backend.contains(b"key").unwrap());
    }

    #[test]
    fn test_libsql_backend_delete() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        backend.put(b"key", b"value").unwrap();
        assert!(backend.contains(b"key").unwrap());

        backend.delete(b"key").unwrap();
        assert!(!backend.contains(b"key").unwrap());
    }

    #[test]
    #[ignore] // Slow test - takes >60s
    fn test_libsql_backend_clear() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        for i in 0..5 {
            backend
                .put(format!("key{}", i).as_bytes(), b"value")
                .unwrap();
        }

        backend.clear().unwrap();

        let stats = backend.get_stats();
        assert_eq!(stats.get("entries").unwrap(), "0");
    }

    #[test]
    fn test_libsql_backend_size_on_disk_memory() {
        let backend = LibsqlBackend::new_temporary().unwrap();

        // Empty in-memory database
        let size_empty = backend.size_on_disk().unwrap();
        assert_eq!(size_empty, 0);

        // Add data
        backend.put(b"key", b"value").unwrap();
        let size_with_data = backend.size_on_disk().unwrap();
        assert!(size_with_data > 0);
    }

    #[test]
    fn test_libsql_backend_size_on_disk_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backend = LibsqlBackend::new(&db_path).unwrap();

        backend.put(b"key", b"value").unwrap();
        backend.flush().unwrap();

        let size = backend.size_on_disk().unwrap();
        // File-based database should have positive size
        assert!(size > 0);
    }

    // ============ StorageConfig Tests ============

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.backend_type, StorageBackendType::Libsql);
        assert!(config.path.is_none());
        assert_eq!(config.cache_size_mb, Some(128));
        assert!(config.compression);
    }

    #[test]
    fn test_storage_config_custom() {
        let config = StorageConfig {
            backend_type: StorageBackendType::InMemory,
            path: Some(std::path::PathBuf::from("/tmp/test.db")),
            cache_size_mb: Some(256),
            compression: false,
        };
        assert_eq!(config.backend_type, StorageBackendType::InMemory);
        assert!(config.path.is_some());
        assert_eq!(config.cache_size_mb, Some(256));
        assert!(!config.compression);
    }

    #[test]
    fn test_storage_config_serialization() {
        let config = StorageConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("\"backend_type\""));
        assert!(serialized.contains("Libsql"));
    }

    #[test]
    fn test_storage_config_deserialization() {
        let json =
            r#"{"backend_type":"InMemory","path":null,"cache_size_mb":64,"compression":true}"#;
        let config: StorageConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.backend_type, StorageBackendType::InMemory);
        assert!(config.path.is_none());
        assert_eq!(config.cache_size_mb, Some(64));
    }

    #[test]
    fn test_storage_config_debug() {
        let config = StorageConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("StorageConfig"));
    }

    #[test]
    fn test_storage_config_clone() {
        let config = StorageConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.backend_type, config.backend_type);
    }

    // ============ StorageBackendType Tests ============

    #[test]
    fn test_storage_backend_type_display_libsql() {
        assert_eq!(format!("{}", StorageBackendType::Libsql), "libsql");
    }

    #[test]
    fn test_storage_backend_type_display_inmemory() {
        assert_eq!(format!("{}", StorageBackendType::InMemory), "in-memory");
    }

    #[test]
    fn test_storage_backend_type_equality() {
        assert_eq!(StorageBackendType::Libsql, StorageBackendType::Libsql);
        assert_ne!(StorageBackendType::Libsql, StorageBackendType::InMemory);
    }

    #[test]
    fn test_storage_backend_type_serialization() {
        let backend_type = StorageBackendType::Libsql;
        let serialized = serde_json::to_string(&backend_type).unwrap();
        assert_eq!(serialized, "\"Libsql\"");
    }

    #[test]
    fn test_storage_backend_type_deserialization() {
        let deserialized: StorageBackendType = serde_json::from_str("\"InMemory\"").unwrap();
        assert_eq!(deserialized, StorageBackendType::InMemory);
    }

    #[test]
    fn test_storage_backend_type_debug() {
        let debug = format!("{:?}", StorageBackendType::Libsql);
        assert!(debug.contains("Libsql"));
    }

    #[test]
    fn test_storage_backend_type_copy() {
        let backend_type = StorageBackendType::Libsql;
        let copied = backend_type;
        assert_eq!(copied, StorageBackendType::Libsql);
    }

