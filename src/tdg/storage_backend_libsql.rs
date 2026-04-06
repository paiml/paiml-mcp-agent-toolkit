/// Libsql database backend (modern SQLite-compatible)
/// Note: Uses rusqlite for synchronous API (libsql is async-first)
pub struct LibsqlBackend {
    db: Arc<parking_lot::Mutex<rusqlite::Connection>>,
    path: std::path::PathBuf,
}

impl LibsqlBackend {
    pub fn new(path: &Path) -> Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Create database directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open database with rusqlite (libsql-compatible)
        let conn = rusqlite::Connection::open(path)?;

        // Enable WAL mode for concurrent access (#161)
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        // Create table for key-value storage
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tdg_storage (
                key BLOB PRIMARY KEY,
                value BLOB NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            db: Arc::new(parking_lot::Mutex::new(conn)),
            path: path.to_path_buf(),
        })
    }

    pub fn new_temporary() -> Result<Self> {
        // Use in-memory database for ephemeral storage
        let conn = rusqlite::Connection::open_in_memory()?;

        // Create table for key-value storage
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tdg_storage (
                key BLOB PRIMARY KEY,
                value BLOB NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            db: Arc::new(parking_lot::Mutex::new(conn)),
            path: std::path::PathBuf::from(":memory:"),
        })
    }
}

impl StorageBackend for LibsqlBackend {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        let db = self.db.lock();
        db.execute(
            "INSERT OR REPLACE INTO tdg_storage (key, value) VALUES (?, ?)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        let db = self.db.lock();
        let mut stmt = db.prepare_cached("SELECT value FROM tdg_storage WHERE key = ?")?;

        let result = stmt.query_row([key], |row| row.get::<_, Vec<u8>>(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        let db = self.db.lock();
        db.execute("DELETE FROM tdg_storage WHERE key = ?", [key])?;
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        let db = self.db.lock();
        let mut stmt = db.prepare_cached("SELECT 1 FROM tdg_storage WHERE key = ? LIMIT 1")?;
        let result = stmt.query_row([key], |_row| Ok(()));

        match result {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn iter(&self) -> Result<StorageIterator<'_>> {
        debug_assert!(self.path.exists(), "self.path must exist");
        // For iteration, we need to collect all data first since we can't hold
        // the lock across iterator lifetime
        let db = self.db.lock();
        let mut stmt = db.prepare("SELECT key, value FROM tdg_storage")?;
        let rows = stmt.query_map([], |row| {
            let key: Vec<u8> = row.get(0)?;
            let value: Vec<u8> = row.get(1)?;
            Ok((key, value))
        })?;

        let items: Vec<Result<KeyValuePair>> = rows.map(|r| r.map_err(Into::into)).collect();

        Ok(Box::new(items.into_iter()))
    }

    fn size_on_disk(&self) -> Result<u64> {
        debug_assert!(self.path.exists(), "self.path must exist");
        if self.path.to_str() == Some(":memory:") {
            // In-memory database - estimate from row count
            let db = self.db.lock();
            let total_bytes: Option<i64> = db.query_row(
                "SELECT SUM(LENGTH(key) + LENGTH(value)) FROM tdg_storage",
                [],
                |row| row.get(0),
            )?;

            Ok(total_bytes.unwrap_or(0) as u64)
        } else {
            // File-based database
            match std::fs::metadata(&self.path) {
                Ok(metadata) => Ok(metadata.len()),
                Err(_) => Ok(0),
            }
        }
    }

    fn flush(&self) -> Result<()> {
        debug_assert!(self.path.exists(), "self.path must exist");
        // SQLite auto-commits by default, but we can execute a checkpoint for WAL mode
        let db = self.db.lock();
        // Try WAL checkpoint, ignore error if not in WAL mode
        let _ = db.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        debug_assert!(self.path.exists(), "self.path must exist");
        let db = self.db.lock();
        db.execute("DELETE FROM tdg_storage", [])?;
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        debug_assert!(self.path.exists(), "self.path must exist");
        "libsql"
    }

    fn get_stats(&self) -> HashMap<String, String> {
        debug_assert!(self.path.exists(), "self.path must exist");
        let mut stats = HashMap::new();

        let db = self.db.lock();

        // Get row count
        if let Ok(count) =
            db.query_row::<i64, _, _>("SELECT COUNT(*) FROM tdg_storage", [], |row| row.get(0))
        {
            stats.insert("entries".to_string(), count.to_string());
        }

        // Get database size
        if let Ok(size) = self.size_on_disk() {
            stats.insert("size_bytes".to_string(), size.to_string());
        }

        // Add database path
        stats.insert("path".to_string(), self.path.display().to_string());

        // Get page count (SQLite specific)
        if let Ok(pages) = db.query_row::<i64, _, _>("PRAGMA page_count", [], |row| row.get(0)) {
            stats.insert("page_count".to_string(), pages.to_string());
        }

        stats
    }
}
