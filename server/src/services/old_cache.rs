use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

pub async fn get_metadata<T: Clone>(
    cache: &Arc<RwLock<LruCache<String, Arc<T>>>>,
    key: &str,
) -> Option<Arc<T>> {
    let mut cache_guard = cache.write().await;
    if let Some(value) = cache_guard.get(key) {
        debug!("Cache hit for metadata: {}", key);
        Some(Arc::clone(value))
    } else {
        debug!("Cache miss for metadata: {}", key);
        None
    }
}

pub async fn put_metadata<T>(
    cache: &Arc<RwLock<LruCache<String, Arc<T>>>>,
    key: String,
    value: Arc<T>,
) {
    let mut cache_guard = cache.write().await;
    cache_guard.put(key.clone(), value);
    debug!("Cached metadata: {}", key);
}

pub async fn get_content(
    cache: &Arc<RwLock<LruCache<String, Arc<str>>>>,
    key: &str,
) -> Option<Arc<str>> {
    let mut cache_guard = cache.write().await;
    if let Some(value) = cache_guard.get(key) {
        debug!("Cache hit for content: {}", key);
        Some(Arc::clone(value))
    } else {
        debug!("Cache miss for content: {}", key);
        None
    }
}

pub async fn put_content(
    cache: &Arc<RwLock<LruCache<String, Arc<str>>>>,
    key: String,
    value: Arc<str>,
) {
    let mut cache_guard = cache.write().await;
    cache_guard.put(key.clone(), value);
    debug!("Cached content: {}", key);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_old_cache_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
