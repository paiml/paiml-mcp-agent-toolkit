// OlapAnalytics trait implementation for TruenoOlapAnalytics.
// Provides store_batch, query_top_k, aggregate, query_by_language, count, and clear.

#[async_trait::async_trait]
impl OlapAnalytics for TruenoOlapAnalytics {
    async fn store_batch(&self, scores: &[TdgScore]) -> Result<usize> {
        if scores.is_empty() {
            return Ok(0);
        }

        // Convert to Arrow format
        let batch = self.scores_to_arrow(scores)?;

        // Append to trueno-db (OLAP append-only pattern)
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;
        storage.append_batch(batch)?;

        Ok(scores.len())
    }

    async fn query_top_k(&self, k: usize, order_by: &str) -> Result<Vec<TdgScore>> {
        // Use trueno-db SQL Top-K optimization (ORDER BY + LIMIT)
        let query = format!(
            "SELECT * FROM tdg_scores ORDER BY {} DESC LIMIT {}",
            order_by, k
        );

        let storage = self
            .storage
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

        // trueno-db doesn't need table name in storage (single table model)
        // So we parse but ignore table name
        let plan = self.query_engine.parse(&query)?;
        let result_batch = self.executor.execute(&plan, &storage)?;

        self.arrow_to_scores(result_batch)
    }

    async fn aggregate(&self, operation: AggOp, column: &str) -> Result<f64> {
        // Use trueno-db SIMD/GPU-accelerated aggregation
        let op_str = match operation {
            AggOp::Sum => "SUM",
            AggOp::Avg => "AVG",
            AggOp::Min => "MIN",
            AggOp::Max => "MAX",
            AggOp::Count => "COUNT",
        };
        let query = format!("SELECT {}({}) FROM tdg_scores", op_str, column);

        let storage = self
            .storage
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

        let plan = self.query_engine.parse(&query)?;
        let result_batch = self.executor.execute(&plan, &storage)?;

        // Extract scalar result from RecordBatch
        if result_batch.num_rows() == 0 {
            return Ok(0.0);
        }

        let column = result_batch.column(0);
        let value = if let Some(float_array) =
            column.as_any().downcast_ref::<arrow::array::Float32Array>()
        {
            float_array.value(0) as f64
        } else if let Some(float_array) =
            column.as_any().downcast_ref::<arrow::array::Float64Array>()
        {
            float_array.value(0)
        } else if let Some(int_array) = column.as_any().downcast_ref::<arrow::array::Int64Array>() {
            int_array.value(0) as f64
        } else {
            return Err(anyhow::anyhow!("Unexpected result type for aggregation"));
        };

        Ok(value)
    }

    async fn query_by_language(
        &self,
        language: Language,
        limit: Option<usize>,
    ) -> Result<Vec<TdgScore>> {
        // Use SQL WHERE filtering
        let lang_str = format!("{:?}", language);
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let query = format!(
            "SELECT * FROM tdg_scores WHERE language = '{}'{}",
            lang_str, limit_clause
        );

        let storage = self
            .storage
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

        let plan = self.query_engine.parse(&query)?;
        let result_batch = self.executor.execute(&plan, &storage)?;

        self.arrow_to_scores(result_batch)
    }

    async fn count(&self) -> Result<usize> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

        // Sum rows across all batches
        let total_rows: usize = storage.batches().iter().map(|b| b.num_rows()).sum();
        Ok(total_rows)
    }

    async fn clear(&self) -> Result<()> {
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

        // Clear all batches (OLAP: recreate storage)
        *storage = trueno_db::storage::StorageEngine::new(vec![]);
        Ok(())
    }
}
