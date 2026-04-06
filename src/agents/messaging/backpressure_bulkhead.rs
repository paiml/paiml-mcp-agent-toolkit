impl Clone for Bulkhead {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            max_concurrent: self.max_concurrent,
            semaphore: self.semaphore.clone(),
            active_count: self.active_count.clone(),
            rejected_count: self.rejected_count.clone(),
        }
    }
}

impl Bulkhead {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            name,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_count: Arc::new(AtomicU32::new(0)),
            rejected_count: Arc::new(AtomicU64::new(0)),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute<F, T>(&self, operation: F) -> Result<T, BackpressureError>
    where
        F: std::future::Future<Output = T>,
    {
        let permit = self.semaphore.clone().try_acquire_owned().map_err(|_| {
            self.rejected_count.fetch_add(1, Ordering::Relaxed);
            BackpressureError::QueueFull
        })?;

        self.active_count.fetch_add(1, Ordering::Relaxed);

        let result = operation.await;

        drop(permit);
        self.active_count.fetch_sub(1, Ordering::Relaxed);

        Ok(result)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_metrics(&self) -> BulkheadMetrics {
        BulkheadMetrics {
            name: self.name.clone(),
            max_concurrent: self.max_concurrent,
            active_count: self.active_count.load(Ordering::Relaxed),
            rejected_count: self.rejected_count.load(Ordering::Relaxed),
        }
    }
}
