#![cfg_attr(coverage_nightly, coverage(off))]
/// Storage backend abstraction for flexible persistence options
///
/// ## OLAP Design Pattern (Issue #79, P0-4)
///
/// This storage backend follows OLAP (Online Analytical Processing) principles:
/// - **Append-only writes**: Use `put()` to insert new records
/// - **No single-row updates**: Records are immutable once written
/// - **Batch operations**: Prefer bulk inserts over individual puts
/// - **Read-optimized**: Designed for analytical queries over large datasets
///
/// ### OLAP vs OLTP
///
/// **OLAP (Analytical)**: Columnar storage, append-only, batch inserts
/// - Used for: TDG score storage, analytics, time-series data
/// - Operations: INSERT (append), SELECT (read), bulk DELETE (archival)
///
/// **OLTP (Transactional)**: Row-oriented, UPDATE/DELETE, ACID transactions
/// - Used for: User accounts, shopping carts, real-time updates
/// - Operations: INSERT, UPDATE, DELETE, complex transactions
///
/// ### Why OLAP for TDG Storage?
///
/// 1. **Performance**: Columnar storage is 10-100x faster for analytics
/// 2. **Immutability**: TDG scores are historical facts, never updated
/// 3. **Compression**: Columnar data compresses better (5-10x)
/// 4. **Vectorization**: SIMD operations work best on columnar data
///
/// ### delete() Method - OLAP-Compatible Usage
///
/// The `delete()` method exists for tiered storage management (warm → cold),
/// NOT for updating records. This is an OLAP-compatible pattern:
/// - Data lifecycle management (archive old records to cold storage)
/// - Testing/cleanup (clear all data between test runs)
///
/// **Anti-pattern (OLTP)**: `update_single(id, new_value)` - NEVER DO THIS
/// **Correct pattern (OLAP)**: `put(new_record)` then `delete(old_key)` for archival
///
/// ### Academic References
///
/// - Stonebraker et al. (2005): "C-Store: A Column-oriented DBMS" (VLDB)
/// - Abadi et al. (2013): "The Design and Implementation of Modern Column-Oriented Database Systems"
/// - MonetDB: Vectorized query processing with columnar storage
///
/// Supported backend implementations (this is the complete list — Sled and
/// RocksDB were removed, not deprecated, and no longer exist in the tree):
/// - `LibsqlBackend`: embedded SQLite-compatible store, the default. Named for
///   the libsql dialect it speaks; the driver is `rusqlite`, because libsql's
///   own client is async-first and this trait is synchronous. `libsql` is not a
///   dependency of this crate — see `backend_name()`.
/// - `InMemoryBackend`: fast testing and development backend
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Type alias for key-value pair
pub type KeyValuePair = (Vec<u8>, Vec<u8>);

/// Type alias for storage iterator
pub type StorageIterator<'a> = Box<dyn Iterator<Item = Result<KeyValuePair>> + 'a>;

/// Trait for storage backend implementations
///
/// ## OLAP Usage Guidelines
///
/// - **put()**: Append-only writes (insert new records)
/// - **get()**: Read operations (retrieve records)
/// - **delete()**: ONLY for tiered storage management (warm → cold archival)
/// - **clear()**: ONLY for testing/cleanup
///
/// ⚠️  **NEVER use delete() to update records** - use put() with a new key instead
pub trait StorageBackend: Send + Sync {
    /// Store a key-value pair (append-only operation)
    ///
    /// OLAP pattern: Insert new records, never update existing ones
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Retrieve a value by key (read operation)
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Delete a key-value pair
    ///
    /// ⚠️ **OLAP Usage Only**: Use this ONLY for:
    /// - Tiered storage management (moving data from warm → cold storage)
    /// - Testing/cleanup (clear all data between test runs)
    ///
    /// **NEVER use delete() to update records** - this violates OLAP principles
    fn delete(&self, key: &[u8]) -> Result<()>;

    /// Check if a key exists
    fn contains(&self, key: &[u8]) -> Result<bool>;

    /// Iterate over all key-value pairs
    fn iter(&self) -> Result<StorageIterator<'_>>;

    /// Get approximate size in bytes
    fn size_on_disk(&self) -> Result<u64>;

    /// Flush any pending writes
    fn flush(&self) -> Result<()>;

    /// Clear all data
    fn clear(&self) -> Result<()>;

    /// Get backend name for diagnostics
    fn backend_name(&self) -> &'static str;

    /// Get backend-specific statistics
    fn get_stats(&self) -> HashMap<String, String>;
}

// NOTE: Sled backend removed - unmaintained, replaced by LibsqlBackend (default)
// See: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/XX

include!("storage_backend_libsql.rs");

include!("storage_backend_inmemory.rs");

include!("storage_backend_config.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_in_memory_backend() {
        let backend = InMemoryBackend::new();

        // Test basic operations
        let key = b"test_key";
        let value = b"test_value";

        backend.put(key, value).unwrap();
        assert!(backend.contains(key).unwrap());

        let retrieved = backend.get(key).unwrap().unwrap();
        assert_eq!(retrieved, value);

        backend.delete(key).unwrap();
        assert!(!backend.contains(key).unwrap());
    }

    // NOTE: test_sled_backend removed - Sled backend removed from codebase

    #[test]
    fn test_libsql_backend() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LibsqlBackend::new(temp_dir.path().join("test.db").as_path()).unwrap();

        // Test basic operations
        let key = b"libsql_key";
        let value = b"libsql_value";

        backend.put(key, value).unwrap();
        backend.flush().unwrap();

        let retrieved = backend.get(key).unwrap().unwrap();
        assert_eq!(retrieved, value);

        // Test iteration
        let mut count = 0;
        for result in backend.iter().unwrap() {
            let (k, v) = result.unwrap();
            if k == key.to_vec() {
                assert_eq!(v, value);
                count += 1;
            }
        }
        assert_eq!(count, 1);

        // Test stats
        let stats = backend.get_stats();
        assert!(stats.contains_key("entries"));
        assert_eq!(stats.get("entries").unwrap(), "1");
    }

    /// Characterization test (passes before and after the `flush()` rewrite):
    /// pins that flush really does TRUNCATE-checkpoint the WAL.
    ///
    /// It exists because the old `let _ = db.execute("PRAGMA wal_checkpoint…")`
    /// got the right result for the wrong reason: `Connection::execute` rejects
    /// row-returning statements, so it always handed back an
    /// `ExecuteReturnedResults` that the `let _` discarded — the checkpoint
    /// landed only because rusqlite steps the statement before noticing the
    /// rows. Nothing verified the outcome, so any change to that incidental
    /// ordering would have turned flush into a silent no-op that still returned
    /// `Ok(())`. This test makes the outcome observable.
    #[test]
    fn flush_actually_checkpoints_the_wal() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("flush.db");
        let backend = LibsqlBackend::new(db_path.as_path()).unwrap();

        // Enough data to be visible, but well under SQLite's 1000-page
        // auto-checkpoint threshold so only an explicit flush can move it.
        for i in 0..200 {
            backend
                .put(format!("key_{i}").as_bytes(), &[7u8; 512])
                .unwrap();
        }

        let wal_path = db_path.with_file_name("flush.db-wal");
        let wal_before = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
        assert!(
            wal_before > 0,
            "precondition: writes should be sitting in the WAL before flush"
        );

        backend.flush().unwrap();

        let wal_after = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
        assert_eq!(
            wal_after, 0,
            "flush() must TRUNCATE-checkpoint the WAL; {wal_after} bytes still pending"
        );
        assert!(
            std::fs::metadata(&db_path).unwrap().len() > 0,
            "checkpointed data must be in the main database file"
        );
    }

    /// Regression: a failed `stat` was reported as a real size of 0 bytes,
    /// indistinguishable from a legitimately empty database.
    #[test]
    fn size_on_disk_fails_loudly_when_the_db_file_is_gone() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("vanishing.db");
        let backend = LibsqlBackend::new(db_path.as_path()).unwrap();
        backend.put(b"k", b"v").unwrap();

        std::fs::remove_file(&db_path).unwrap();

        let err = backend
            .size_on_disk()
            .expect_err("an unreadable db file must not be reported as size 0");
        let msg = err.to_string();
        assert!(
            msg.contains("vanishing.db"),
            "error must identify the database it could not measure, got: {msg}"
        );
    }

    #[test]
    fn test_backend_factory() {
        // Test in-memory creation
        let backend = StorageBackendFactory::create_in_memory();
        assert_eq!(backend.backend_name(), "in-memory");

        // Test temporary libsql creation
        let backend = StorageBackendFactory::create_libsql_temporary().unwrap();
        assert_eq!(backend.backend_name(), "libsql");

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
    fn test_storage_iterator_type_alias() {
        let backend = InMemoryBackend::new();

        // Add test data
        backend.put(b"key1", b"value1").unwrap();
        backend.put(b"key2", b"value2").unwrap();

        // Get iterator using the type alias
        let iter: StorageIterator = backend.iter().unwrap();

        // Collect results
        let results: Vec<KeyValuePair> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(results.len(), 2);

        // Verify the KeyValuePair type alias works
        for pair in results {
            let (key, value): KeyValuePair = pair;
            assert!(!key.is_empty());
            assert!(!value.is_empty());
        }
    }

    #[test]
    fn test_backend_clear() {
        let backend = InMemoryBackend::new();

        // Add multiple entries
        for i in 0..10 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            backend.put(key.as_bytes(), value.as_bytes()).unwrap();
        }

        // Verify entries exist
        assert!(backend.contains(b"key_5").unwrap());

        // Clear all data
        backend.clear().unwrap();

        // Verify all entries are gone
        assert!(!backend.contains(b"key_5").unwrap());

        let stats = backend.get_stats();
        assert_eq!(stats.get("entries").unwrap(), "0");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod extended_tests {
    use super::*;
    use tempfile::TempDir;

    include!("storage_backend_tests_extended_part1.rs");

    include!("storage_backend_tests_extended_part2.rs");
}
