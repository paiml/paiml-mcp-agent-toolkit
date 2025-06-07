use crate::services::cache::{
    base::{CacheStats, CacheStrategy},
    content_cache::ContentCache,
    persistent::PersistentCache,
    unified::UnifiedCache,
};
use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Adapter to make ContentCache implement UnifiedCache
#[derive(Clone)]
pub struct ContentCacheAdapter<T: CacheStrategy> {
    inner: Arc<RwLock<ContentCache<T>>>,
}

impl<T: CacheStrategy> ContentCacheAdapter<T> {
    pub fn new(cache: ContentCache<T>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(cache)),
        }
    }
}

#[async_trait]
impl<T> UnifiedCache for ContentCacheAdapter<T>
where
    T: CacheStrategy + Send + Sync + Clone + 'static,
    T::Key: Send + Sync + 'static,
    T::Value: Send + Sync + 'static,
{
    type Key = T::Key;
    type Value = T::Value;

    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        self.inner.read().get(key)
    }

    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<()> {
        self.inner.write().put(key, value);
        Ok(())
    }

    async fn remove(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        self.inner.write().remove(key)
    }

    async fn clear(&self) -> Result<()> {
        self.inner.write().clear();
        Ok(())
    }

    fn stats(&self) -> Arc<CacheStats> {
        Arc::new(self.inner.read().stats.clone())
    }

    fn size_bytes(&self) -> usize {
        self.inner.read().stats.memory_usage()
    }

    fn len(&self) -> usize {
        self.inner.read().len()
    }

    async fn evict_if_needed(&self) -> Result<()> {
        // ContentCache doesn't have explicit eviction control
        // LRU eviction happens automatically on insert
        Ok(())
    }
}

/// Adapter to make PersistentCache implement UnifiedCache
#[derive(Clone)]
pub struct PersistentCacheAdapter<T: CacheStrategy> {
    inner: Arc<RwLock<PersistentCache<T>>>,
}

impl<T: CacheStrategy> PersistentCacheAdapter<T> {
    pub fn new(cache: PersistentCache<T>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(cache)),
        }
    }
}

#[async_trait]
impl<T> UnifiedCache for PersistentCacheAdapter<T>
where
    T: CacheStrategy + Send + Sync + Clone + 'static,
    T::Key: Send + Sync + 'static,
    T::Value: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    type Key = T::Key;
    type Value = T::Value;

    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        self.inner.read().get(key)
    }

    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<()> {
        self.inner.write().put(key.clone(), value)?;
        Ok(())
    }

    async fn remove(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        self.inner.write().remove(key)
    }

    async fn clear(&self) -> Result<()> {
        self.inner.write().clear()?;
        Ok(())
    }

    fn stats(&self) -> Arc<CacheStats> {
        Arc::new(self.inner.read().stats.clone())
    }

    fn size_bytes(&self) -> usize {
        self.inner.read().stats.memory_usage()
    }

    fn len(&self) -> usize {
        self.inner.read().len()
    }

    async fn evict_if_needed(&self) -> Result<()> {
        self.inner.write().evict_if_needed();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::cache::strategies::AstCacheStrategy;

    #[tokio::test]
    async fn test_content_cache_adapter() {
        let cache = ContentCache::new(AstCacheStrategy);
        let adapter = ContentCacheAdapter::new(cache);

        // Basic functionality test
        assert_eq!(adapter.len(), 0);
        assert!(adapter.is_empty());
    }
}
