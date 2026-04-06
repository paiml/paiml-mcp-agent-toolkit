#![cfg_attr(coverage_nightly, coverage(off))]
//! EnhancedParser implementation with incremental parsing and caching

use super::types::{CacheStats, CachedSyntax};
use super::visitor::ComplexityVisitor;
use crate::unified_quality::metrics::Metrics;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use syn::{visit::Visit, File};

/// Enhanced parser using syn for accurate Rust analysis
pub struct EnhancedParser {
    /// Cached ASTs with metadata
    cache: Arc<dashmap::DashMap<PathBuf, CachedSyntax>>,
}

impl Default for EnhancedParser {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedParser {
    /// Create a new enhanced parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Parse file with incremental updates
    pub fn parse_incremental(&mut self, path: &PathBuf, content: &str) -> Result<Metrics> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content_hash = self.calculate_hash(content);

        // Check cache for existing result
        if let Some(cached) = self.cache.get(path) {
            if cached.content_hash == content_hash {
                // Content unchanged, return cached metrics
                if let Some(ref metrics) = cached.metrics {
                    return Ok(metrics.clone());
                }
            }
        }

        // Parse and analyze
        self.parse_and_analyze(path, content)
    }

    /// Parse and analyze Rust code
    fn parse_and_analyze(&mut self, path: &PathBuf, content: &str) -> Result<Metrics> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Parse using syn
        let syntax: File =
            syn::parse_str(content).map_err(|e| anyhow!("Failed to parse Rust code: {e}"))?;

        // Calculate metrics using visitor pattern
        let mut visitor = ComplexityVisitor::new(content.to_string());
        visitor.visit_file(&syntax);

        let metrics = Metrics {
            complexity: visitor.complexity,
            cognitive: visitor.cognitive,
            satd_count: visitor.satd_count,
            coverage: 0.8, // Placeholder - would integrate with coverage tools
            lines: content.lines().count() as u32,
            functions: visitor.function_count,
            timestamp: SystemTime::now(),
        };

        // Cache the result (without storing syn::File directly to avoid Send issues)
        self.cache.insert(
            path.clone(),
            CachedSyntax {
                syntax_str: format!("{syntax:#?}"), // Debug representation
                content: content.to_string(),
                last_modified: SystemTime::now(),
                content_hash: self.calculate_hash(content),
                metrics: Some(metrics.clone()),
            },
        );

        Ok(metrics)
    }

    /// Calculate content hash for caching
    pub(crate) fn calculate_hash(&self, content: &str) -> u64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cached metrics if available
    #[must_use]
    pub fn get_cached_metrics(&self, path: &PathBuf) -> Option<Metrics> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.cache.get(path)?.metrics.clone()
    }

    /// Clear cache for a file
    pub fn clear_cache(&self, path: &PathBuf) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.cache.remove(path);
    }

    /// Clear entire cache
    pub fn clear_all_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    #[must_use]
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            memory_usage_estimate: self.cache.len() * 2048, // Rough estimate
        }
    }
}
