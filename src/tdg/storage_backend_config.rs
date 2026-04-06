// NOTE: RocksDB backend removed - C++ dependency removed in favor of pure-Rust trueno-db
// For high-performance storage, use TruenoDbBackend (async, SIMD-accelerated)
// See: https://crates.io/crates/trueno-db

/// Storage backend factory for creating appropriate backends
pub struct StorageBackendFactory;

impl StorageBackendFactory {
    /// Create default backend (libsql)
    pub fn create_default(path: &Path) -> Result<Box<dyn StorageBackend>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Ok(Box::new(LibsqlBackend::new(path)?))
    }

    /// Create in-memory backend for testing
    #[must_use]
    pub fn create_in_memory() -> Box<dyn StorageBackend> {
        Box::new(InMemoryBackend::new())
    }

    /// Create libsql backend
    pub fn create_libsql(path: &Path) -> Result<Box<dyn StorageBackend>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Ok(Box::new(LibsqlBackend::new(path)?))
    }

    /// Create temporary libsql backend
    pub fn create_libsql_temporary() -> Result<Box<dyn StorageBackend>> {
        Ok(Box::new(LibsqlBackend::new_temporary()?))
    }

    /// Create backend from configuration
    pub fn create_from_config(config: &StorageConfig) -> Result<Box<dyn StorageBackend>> {
        match config.backend_type {
            StorageBackendType::Libsql => {
                if let Some(path) = &config.path {
                    Self::create_libsql(path)
                } else {
                    Self::create_libsql_temporary()
                }
            }
            StorageBackendType::InMemory => Ok(Self::create_in_memory()),
        }
    }
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend_type: StorageBackendType,
    pub path: Option<std::path::PathBuf>,
    pub cache_size_mb: Option<u32>,
    pub compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend_type: StorageBackendType::Libsql,
            path: None,
            cache_size_mb: Some(128),
            compression: true,
        }
    }
}

/// Available storage backend types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StorageBackendType {
    /// Modern SQLite-compatible backend (default, recommended)
    Libsql,
    /// In-memory backend for testing
    InMemory,
}

impl std::fmt::Display for StorageBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageBackendType::Libsql => write!(f, "libsql"),
            StorageBackendType::InMemory => write!(f, "in-memory"),
        }
    }
}
