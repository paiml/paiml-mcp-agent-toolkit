/// In-memory backend for testing and development
pub struct InMemoryBackend {
    data: Arc<DashMap<Vec<u8>, Vec<u8>>>,
}

impl InMemoryBackend {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryBackend {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        self.data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        Ok(self.data.get(key).map(|v| v.clone()))
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        self.data.remove(key);
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        Ok(self.data.contains_key(key))
    }

    fn iter(&self) -> Result<StorageIterator<'_>> {
        let entries: Vec<_> = self
            .data
            .iter()
            .map(|entry| Ok((entry.key().clone(), entry.value().clone())))
            .collect();
        Ok(Box::new(entries.into_iter()))
    }

    fn size_on_disk(&self) -> Result<u64> {
        let size: usize = self
            .data
            .iter()
            .map(|entry| entry.key().len() + entry.value().len())
            .sum();
        Ok(size as u64)
    }

    fn flush(&self) -> Result<()> {
        // No-op for in-memory backend
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        self.data.clear();
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "in-memory"
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("entries".to_string(), self.data.len().to_string());
        let size: usize = self
            .data
            .iter()
            .map(|entry| entry.key().len() + entry.value().len())
            .sum();
        stats.insert("memory_bytes".to_string(), size.to_string());
        stats
    }
}
