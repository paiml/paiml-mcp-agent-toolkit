impl BackpressureController {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            _max_queue_size: max_queue_size,
            current_queue_size: AtomicU64::new(0),
            semaphore: Arc::new(Semaphore::new(max_queue_size)),
            metrics: Arc::new(parking_lot::RwLock::new(BackpressureMetrics::default())),
        }
    }

    pub async fn acquire_permit(&self) -> Result<BackpressurePermit<'_>, BackpressureError> {
        let permit = self
            .semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|_| BackpressureError::QueueFull)?;

        let queue_size = self.current_queue_size.fetch_add(1, Ordering::SeqCst) + 1;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.accepted_count += 1;
        metrics.queue_depth_sum += queue_size;
        metrics.max_queue_depth = metrics.max_queue_depth.max(queue_size);
        metrics.sample_count += 1;

        Ok(BackpressurePermit {
            _permit: permit,
            controller: self,
        })
    }

    pub fn try_acquire_permit(&self) -> Result<BackpressurePermit<'_>, BackpressureError> {
        let permit = self.semaphore.clone().try_acquire_owned().map_err(|_| {
            self.metrics.write().rejected_count += 1;
            BackpressureError::QueueFull
        })?;

        let queue_size = self.current_queue_size.fetch_add(1, Ordering::SeqCst) + 1;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.accepted_count += 1;
        metrics.queue_depth_sum += queue_size;
        metrics.max_queue_depth = metrics.max_queue_depth.max(queue_size);
        metrics.sample_count += 1;

        Ok(BackpressurePermit {
            _permit: permit,
            controller: self,
        })
    }

    fn release(&self) {
        self.current_queue_size.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn get_queue_depth(&self) -> u64 {
        self.current_queue_size.load(Ordering::Relaxed)
    }

    pub fn get_metrics(&self) -> BackpressureMetrics {
        self.metrics.read().clone()
    }

    pub fn get_average_queue_depth(&self) -> f64 {
        let metrics = self.metrics.read();
        if metrics.sample_count > 0 {
            metrics.queue_depth_sum as f64 / metrics.sample_count as f64
        } else {
            0.0
        }
    }
}

impl Drop for BackpressurePermit<'_> {
    fn drop(&mut self) {
        self.controller.release();
    }
}
