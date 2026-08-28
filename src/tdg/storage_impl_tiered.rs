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
        // Issue #1050 P8. `pmat tdg <repo>` left `?? .pmat/` in that repo's git
        // status, holding `tdg-cold.db` and `tdg-warm.db`. An analysis tool must
        // not dirty the tree it is analysing, and the ignore rule travels with
        // the directory rather than being something each project has to add and
        // then keep in step with pmat's filenames.
        let _ = crate::utils::pmat_cache_dir::ensure_cache_dir(db_path.as_ref());

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

    /// Warm entries above which the compression ratio is left unmeasured.
    ///
    /// Measuring it means reading the stored values back, and the backend
    /// iterator materialises the whole tier; doing that to a 100 MB warm store
    /// just to print one percentage is not a trade worth making. Past this
    /// point `get_statistics` reports the ratio as unmeasured (0.0) rather than
    /// guessing at it.
    const COMPRESSION_SAMPLE_MAX_ENTRIES: usize = 4096;

    /// The warm tier's real compressed:uncompressed ratio, or `None` when there
    /// is nothing to measure.
    ///
    /// `get_statistics` used to hand back the literal `compression_ratio: 0.33,
    /// // Default compression ratio`, which every renderer printed as
    /// "Compression ratio: 33.0%" — including over a store holding zero
    /// entries, and identically for an empty directory and a 4260-file repo.
    /// (The 0.33000001311302185 seen in JSON is only that literal's f32
    /// round-trip.) `store()` writes warm values through
    /// `lz4_flex::compress_prepend_size`, which prefixes each blob with its
    /// uncompressed length as a little-endian u32, so the true ratio can be
    /// read straight back out of the bytes that are actually stored.
    fn measure_warm_compression_ratio(&self, warm_entries: usize) -> Option<f32> {
        if warm_entries == 0 || warm_entries > Self::COMPRESSION_SAMPLE_MAX_ENTRIES {
            return None;
        }

        let mut compressed_total: u64 = 0;
        let mut raw_total: u64 = 0;
        for item in self.warm_backend.iter().ok()? {
            let Ok((_, value)) = item else { continue };
            let Some(header) = value.get(..4) else {
                continue;
            };
            raw_total += u64::from(u32::from_le_bytes([
                header[0], header[1], header[2], header[3],
            ]));
            compressed_total += value.len() as u64;
        }

        if raw_total == 0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        Some(compressed_total as f32 / raw_total as f32)
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

        // Ask for the key the backends actually write. This read used to be
        // `.get("entry_count")`, a key no backend has ever inserted, so it fell
        // through to unwrap_or(0) and every tier count was zero regardless of
        // what was stored — see STAT_KEY_ENTRIES.
        let warm_entries = warm_stats
            .get(crate::tdg::storage_backend::STAT_KEY_ENTRIES)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let cold_entries = cold_stats
            .get(crate::tdg::storage_backend::STAT_KEY_ENTRIES)
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
            // 0.0 means "not measured"; see measure_warm_compression_ratio.
            compression_ratio: self
                .measure_warm_compression_ratio(warm_entries)
                .unwrap_or(0.0),
            // The backends name themselves. Both of these were the literal
            // "sled" — a backend that was deleted from this tree entirely — so
            // diagnostics announced "Warm (sled backend)" over libsql files.
            warm_backend: self.warm_backend.backend_name().to_string(),
            cold_backend: self.cold_backend.backend_name().to_string(),
            backend_stats,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tiered_statistics_tests {
    use super::*;

    fn record_for(content: &[u8]) -> FullTdgRecord {
        FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("test.rs"),
                content_hash: blake3::hash(content),
                size_bytes: content.len() as u64,
                modified_time: SystemTime::now(),
            },
            score: TdgScore::default(),
            components: ComponentScores::default(),
            semantic_sig: SemanticSignature {
                ast_structure_hash: 1,
                identifier_pattern: String::new(),
                control_flow_pattern: String::new(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "test".to_string(),
                analysis_duration_ms: 1,
                language_confidence: 1.0,
                analysis_timestamp: SystemTime::now(),
                cache_hit: false,
            },
            git_context: None,
        }
    }

    /// The reported tier counts must come from the backend the same report
    /// embeds. They were read under the key "entry_count", which no backend
    /// writes, so warm/cold/total were always 0 while `backend_stats` in the
    /// very same payload carried the real count.
    #[tokio::test]
    async fn test_warm_entries_match_the_backend_they_describe() {
        let storage = TieredStore::in_memory();
        storage.store(record_for(b"fn a() {}")).await.unwrap();
        storage.store(record_for(b"fn b() {}")).await.unwrap();

        let stats = storage.get_statistics();
        let backend_entries: usize = stats.backend_stats["warm"]
            [crate::tdg::storage_backend::STAT_KEY_ENTRIES]
            .parse()
            .unwrap();

        assert_eq!(backend_entries, 2, "two records were stored");
        assert_eq!(
            stats.warm_entries, backend_entries,
            "warm_entries must equal the backend count reported beside it"
        );
        assert_eq!(stats.total_entries, stats.hot_entries + backend_entries);
    }

    /// The ratio was the literal 0.33 for every store, empty or not.
    #[tokio::test]
    async fn test_compression_ratio_is_measured_not_a_constant() {
        let empty = TieredStore::in_memory();
        assert_eq!(
            empty.get_statistics().compression_ratio,
            0.0,
            "nothing is stored, so there is no ratio to report"
        );

        let storage = TieredStore::in_memory();
        storage.store(record_for(b"fn a() {}")).await.unwrap();
        let ratio = storage.get_statistics().compression_ratio;
        assert!(
            ratio > 0.0 && ratio < 1.0,
            "expected a measured lz4 ratio, got {ratio}"
        );
        assert!(
            (ratio - 0.33).abs() > f32::EPSILON,
            "0.33 is the old hardcoded literal"
        );
    }

    /// Both backend names were the literal "sled" — a backend deleted from the
    /// tree — so diagnostics announced "Warm (sled backend)" over libsql files.
    #[test]
    fn test_backend_names_come_from_the_backends() {
        let stats = TieredStore::in_memory().get_statistics();
        assert_eq!(stats.warm_backend, "in-memory");
        assert_eq!(stats.cold_backend, "in-memory");
    }
}
