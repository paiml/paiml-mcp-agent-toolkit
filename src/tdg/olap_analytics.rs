//! OLAP Analytics Backend for TDG Scores (Phase 5, Task 5.2)
//!
//! Provides high-performance analytics queries using trueno-db for OLAP workloads.
//!
//! # Design Pattern: Hybrid OLTP/OLAP
//!
//! - **OLTP Storage** (existing): Libsql/SQLite for transactional updates
//! - **OLAP Analytics** (this module): trueno-db for fast analytics queries
//! - **Sync Strategy**: Periodic batch load from OLTP -> OLAP
//!
//! # Performance Targets
//!
//! - Top-K queries: 5-28x faster than heap-based (specification lines 960-962)
//! - Aggregations: 2.78-33x faster via SIMD/GPU (specification line 1015)
//!
//! # Academic References
//!
//! - Stonebraker et al. (2005): "C-Store: A Column-oriented DBMS" (VLDB)
//! - Abadi et al. (2013): "The Design and Implementation of Modern Column-Oriented Database Systems"
//! - Funke et al. (2018): "GPU paging for out-of-core workloads" (SIGMOD)

use crate::tdg::{Language, TdgScore};
use anyhow::Result;

/// Aggregation operations supported by OLAP backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggOp {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

/// Trait for OLAP analytics on TDG scores
///
/// Designed for read-heavy analytical workloads with batch writes.
///
/// # OLAP Usage Pattern
///
/// ```rust,ignore
/// // 1. Batch load data from OLTP store
/// let scores: Vec<TdgScore> = oltp_store.load_all()?;
/// olap_analytics.store_batch(&scores).await?;
///
/// // 2. Run analytics queries (fast!)
/// let top_10_complex = olap_analytics.query_top_k(10, "structural_complexity").await?;
/// let avg_tdg = olap_analytics.aggregate(AggOp::Avg, "total").await?;
/// ```
///
/// # Implementation Notes
///
/// - **Append-only**: Use `store_batch()` for bulk inserts
/// - **No updates**: TDG scores are immutable facts (OLAP principle)
/// - **Columnar storage**: Optimized for analytical queries
#[async_trait::async_trait]
pub trait OlapAnalytics: Send + Sync {
    /// Store a batch of TDG scores (append-only operation)
    ///
    /// # Arguments
    ///
    /// * `scores` - Slice of TDG scores to insert
    ///
    /// # Returns
    ///
    /// Number of records inserted
    ///
    /// # Performance
    ///
    /// Batch inserts are 10-100x faster than individual inserts in columnar databases.
    async fn store_batch(&self, scores: &[TdgScore]) -> Result<usize>;

    /// Query Top-K scores by a specific metric
    ///
    /// # Arguments
    ///
    /// * `k` - Number of top results to return
    /// * `order_by` - Field name to order by (e.g., "total", "structural_complexity")
    ///
    /// # Returns
    ///
    /// Vec of K highest-scoring records in descending order
    ///
    /// # Performance
    ///
    /// - SIMD: 5x faster than heap (450ms vs 2.3s for 1M files)
    /// - GPU: 28.75x faster (80ms vs 2.3s)
    async fn query_top_k(&self, k: usize, order_by: &str) -> Result<Vec<TdgScore>>;

    /// Compute aggregation over a specific metric
    ///
    /// # Arguments
    ///
    /// * `operation` - Aggregation operation (SUM, AVG, MIN, MAX, COUNT)
    /// * `column` - Field name to aggregate (e.g., "total", "doc_coverage")
    ///
    /// # Returns
    ///
    /// Computed aggregation result
    ///
    /// # Performance
    ///
    /// SIMD aggregations are 2.78-33x faster than scalar implementations.
    async fn aggregate(&self, operation: AggOp, column: &str) -> Result<f64>;

    /// Query scores filtered by language
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language filter
    /// * `limit` - Maximum number of results (optional)
    ///
    /// # Returns
    ///
    /// Vec of TDG scores matching the language filter
    async fn query_by_language(
        &self,
        language: Language,
        limit: Option<usize>,
    ) -> Result<Vec<TdgScore>>;

    /// Get total number of records in analytics store
    async fn count(&self) -> Result<usize>;

    /// Clear all analytics data (for testing/cleanup)
    async fn clear(&self) -> Result<()>;
}

/// trueno-db OLAP analytics backend
///
/// Uses Arrow columnar format for SIMD/GPU-accelerated analytics.
///
/// # Example
///
/// ```rust,ignore
/// use pmat::tdg::olap_analytics::{TruenoOlapAnalytics, AggOp};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let analytics = TruenoOlapAnalytics::new("/tmp/tdg_analytics.db").await?;
///
///     // Load batch of scores
///     analytics.store_batch(&scores).await?;
///
///     // Run analytics
///     let top_10 = analytics.query_top_k(10, "total").await?;
///     let avg_score = analytics.aggregate(AggOp::Avg, "total").await?;
///
///     Ok(())
/// }
/// ```
#[cfg(feature = "analytics-simd")]
pub struct TruenoOlapAnalytics {
    storage: std::sync::Mutex<trueno_db::storage::StorageEngine>,
    query_engine: trueno_db::query::QueryEngine,
    executor: trueno_db::query::QueryExecutor,
}

#[cfg(feature = "analytics-simd")]
impl std::fmt::Debug for TruenoOlapAnalytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TruenoOlapAnalytics")
            .field("storage", &"<Mutex<StorageEngine>>")
            .field("query_engine", &"<QueryEngine>")
            .field("executor", &"<QueryExecutor>")
            .finish()
    }
}

// TruenoOlapAnalytics constructor and Arrow conversion methods
#[cfg(feature = "analytics-simd")]
include!("olap_analytics_trueno.rs");

// OlapAnalytics trait implementation for TruenoOlapAnalytics
#[cfg(feature = "analytics-simd")]
include!("olap_analytics_trait_impl.rs");

// Unit tests
include!("olap_analytics_tests.rs");
