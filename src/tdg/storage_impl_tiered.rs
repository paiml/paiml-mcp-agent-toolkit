/// Tiered storage system with Hot/Warm/Cold tiers using flexible backends
pub struct TieredStore {
    /// Hot cache - recent files (in-memory)
    hot: Arc<DashMap<Blake3Hash, HotCacheEntry>>,
    /// Warm storage - compressed recent records (backend-agnostic)
    warm_backend: Box<dyn StorageBackend>,
    /// Cold storage - full historical records (backend-agnostic)
    cold_backend: Box<dyn StorageBackend>,
    /// Archival configuration
    archive_after_days: u32,
}

impl TieredStore {
    /// Create new tiered storage instance with default Libsql backend
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let warm_config = StorageConfig {
            backend_type: crate::tdg::storage_backend::StorageBackendType::Libsql,
            path: Some(db_path.as_ref().join(".pmat/tdg-warm.db")),
            cache_size_mb: Some(128),
            compression: true,
        };

        let cold_config = StorageConfig {
            backend_type: crate::tdg::storage_backend::StorageBackendType::Libsql,
            path: Some(db_path.as_ref().join(".pmat/tdg-cold.db")),
            cache_size_mb: Some(64),
            compression: false, // Cold storage doesn't need additional compression
        };

        Self::with_config(warm_config, cold_config)
    }

    /// Create tiered storage with specific backend configurations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_config(warm_config: StorageConfig, cold_config: StorageConfig) -> Result<Self> {
        let warm_backend = StorageBackendFactory::create_from_config(&warm_config)?;
        let cold_backend = StorageBackendFactory::create_from_config(&cold_config)?;

        Ok(Self {
            hot: Arc::new(DashMap::new()),
            warm_backend,
            cold_backend,
            archive_after_days: 30,
        })
    }

    /// Create in-memory tiered storage for testing
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn in_memory() -> Self {
        Self {
            hot: Arc::new(DashMap::new()),
            warm_backend: StorageBackendFactory::create_in_memory(),
            cold_backend: StorageBackendFactory::create_in_memory(),
            archive_after_days: 30,
        }
    }

    /// Store a complete TDG record in all tiers
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn store(&self, record: FullTdgRecord) -> Result<()> {
        let hash = record.identity.content_hash;

        // Hot cache entry (immediate access)
        let hot_entry = HotCacheEntry::from_record(&record);
        self.hot.insert(hash, hot_entry);

        // Warm storage - compress with LZ4 for space efficiency
        // NOTE: Using serde_json instead of bincode due to incompatibility with FullTdgRecord
        // (see git commit 46968e5f - bincode causes "unexpected end of file" errors)
        let serialized = serde_json::to_vec(&record)?;
        let compressed = compress_prepend_size(&serialized);
        self.warm_backend.put(hash.as_bytes(), &compressed)?;

        // Schedule cold archival if record is old enough
        if self.should_archive(&record) {
            self.archive_to_cold(record).await?;
        }

        Ok(())
    }

    /// Retrieve hot cache entry (fastest access)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_hot(&self, hash: &Blake3Hash) -> Option<HotCacheEntry> {
        self.hot.get(hash).map(|entry| *entry.value())
    }

    /// Retrieve full record from any tier
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn retrieve_full(&self, hash: &Blake3Hash) -> Result<Option<FullTdgRecord>> {
        // Check warm storage first (compressed but fast)
        if let Some(compressed) = self.warm_backend.get(hash.as_bytes())? {
            let decompressed = decompress_size_prepended(&compressed)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            return Ok(Some(serde_json::from_slice(&decompressed)?));
        }

        // Check cold storage (full historical records)
        if let Some(archived) = self.cold_backend.get(hash.as_bytes())? {
            // NOTE: Using serde_json instead of bincode (see store() method)
            return Ok(Some(serde_json::from_slice(&archived)?));
        }

        Ok(None)
    }

    /// Check if record should be archived to cold storage
    fn should_archive(&self, record: &FullTdgRecord) -> bool {
        let age_days = record
            .metadata
            .analysis_timestamp
            .elapsed()
            .unwrap_or_default()
            .as_secs()
            / (24 * 60 * 60);

        age_days > u64::from(self.archive_after_days)
    }

    /// Archive record to cold storage and remove from warm
    async fn archive_to_cold(&self, record: FullTdgRecord) -> Result<()> {
        let hash = record.identity.content_hash;

        // Store in cold storage (uncompressed for long-term access)
        // NOTE: Using serde_json instead of bincode (see store() method)
        let serialized = serde_json::to_vec(&record)?;
        self.cold_backend.put(hash.as_bytes(), &serialized)?;

        // OLAP-Compatible Pattern (Issue #79, P0-4):
        // Remove from warm storage to save space (data lifecycle management)
        // This is NOT an OLTP update - we're moving data between storage tiers
        // The record remains immutable; we're just changing its storage location
        self.warm_backend.delete(hash.as_bytes())?;

        Ok(())
    }

    /// Clean up expired hot cache entries
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn cleanup_hot_cache(&self, max_age_seconds: u64) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut removed = 0;
        self.hot.retain(|_, entry| {
            let age = now - entry.timestamp;
            if age > max_age_seconds as i64 {
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }

    /// Migrate between storage backends
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn migrate_backend(
        &mut self,
        new_warm_config: StorageConfig,
        new_cold_config: StorageConfig,
    ) -> Result<()> {
        // Create new backends
        let new_warm = StorageBackendFactory::create_from_config(&new_warm_config)?;
        let new_cold = StorageBackendFactory::create_from_config(&new_cold_config)?;

        // Migrate warm storage
        if let Ok(iter) = self.warm_backend.iter() {
            for result in iter {
                let (key, value) = result?;
                new_warm.put(&key, &value)?;
            }
        }

        // Migrate cold storage
        if let Ok(iter) = self.cold_backend.iter() {
            for result in iter {
                let (key, value) = result?;
                new_cold.put(&key, &value)?;
            }
        }

        // Swap backends
        self.warm_backend = new_warm;
        self.cold_backend = new_cold;

        Ok(())
    }

    /// Flush all pending writes
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn flush(&self) -> Result<()> {
        self.warm_backend.flush()?;
        self.cold_backend.flush()?;
        Ok(())
    }

    /// Get storage statistics for monitoring and dogfooding
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_statistics(&self) -> StorageStatistics {
        let hot_entries = self.hot.len();
        let hot_memory_kb = (hot_entries * std::mem::size_of::<HotCacheEntry>()) / 1024;

        // Get backend statistics (if available)
        let warm_stats = self.warm_backend.get_stats();
        let cold_stats = self.cold_backend.get_stats();

        let warm_entries = warm_stats
            .get("entry_count")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let cold_entries = cold_stats
            .get("entry_count")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let total_entries = hot_entries + warm_entries + cold_entries;

        let mut backend_stats = HashMap::new();
        backend_stats.insert("warm".to_string(), warm_stats);
        backend_stats.insert("cold".to_string(), cold_stats);

        StorageStatistics {
            hot_entries,
            warm_entries,
            cold_entries,
            total_entries,
            hot_memory_kb,
            compression_ratio: 0.33,          // Default compression ratio
            warm_backend: "sled".to_string(), // Default backend type
            cold_backend: "sled".to_string(), // Default backend type
            backend_stats,
        }
    }
}
