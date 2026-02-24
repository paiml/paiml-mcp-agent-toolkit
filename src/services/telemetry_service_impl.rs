impl TelemetryService {
    /// Create a new telemetry service - THE ONE way (Toyota Way principle)
    pub fn new() -> Self {
        let startup_time = Instant::now();
        info!("Initializing PMAT Telemetry Service");

        Self {
            services: Arc::new(DashMap::new()),
            startup_time,
            event_counter: AtomicU64::new(0),
            system_metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
        }
    }

    /// Record operation telemetry data
    #[instrument(skip(self), fields(service = %input.service_name, operation = %input.operation))]
    pub async fn record_operation(&self, input: TelemetryInput) -> Result<TelemetryOutput> {
        let event_id = Uuid::new_v4().to_string();
        let recorded_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        self.update_service_telemetry(&input).await?;
        self.update_system_metrics(&input).await?;

        let event_count = self.event_counter.fetch_add(1, Ordering::Relaxed) + 1;

        log_operation_result(&event_id, &input, event_count);

        trace!(
            event_id = %event_id,
            tags = ?input.tags,
            properties = ?input.properties,
            "Detailed telemetry event recorded"
        );

        Ok(TelemetryOutput {
            event_id,
            recorded_at,
            success: true,
        })
    }

    /// Update service-specific telemetry data
    async fn update_service_telemetry(&self, input: &TelemetryInput) -> Result<()> {
        let mut entry = self
            .services
            .entry(input.service_name.clone())
            .or_insert_with(|| ServiceTelemetryData {
                service_name: input.service_name.clone(),
                ..Default::default()
            });

        let data = entry.value_mut();
        data.total_operations += 1;

        if input.metrics.success {
            data.successful_operations += 1;
        } else {
            data.failed_operations += 1;
        }

        data.total_duration_ms += input.metrics.duration_ms;
        data.avg_duration_ms = data.total_duration_ms / data.total_operations;
        data.total_items_processed += input.metrics.items_processed;

        if let Some(memory_bytes) = input.metrics.memory_bytes {
            data.peak_memory_bytes = data.peak_memory_bytes.max(memory_bytes);
        }

        data.success_rate = data.successful_operations as f64 / data.total_operations as f64;
        data.last_operation_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        *data
            .operation_counts
            .entry(input.operation.clone())
            .or_insert(0) += 1;

        debug!(
            service = %input.service_name,
            total_ops = data.total_operations,
            success_rate = data.success_rate,
            "Service telemetry updated"
        );

        Ok(())
    }

    /// Update system-wide metrics
    async fn update_system_metrics(&self, input: &TelemetryInput) -> Result<()> {
        let mut metrics = self
            .system_metrics
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire system metrics lock"))?;

        let duration = Duration::from_millis(input.metrics.duration_ms);
        metrics.record_request(duration, input.metrics.success);

        Ok(())
    }

    /// Get comprehensive system telemetry data
    #[instrument(skip(self))]
    pub async fn get_system_telemetry(&self) -> Result<SystemTelemetryData> {
        let uptime_seconds = self.startup_time.elapsed().as_secs();
        let startup_time =
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - uptime_seconds;

        let _system_metrics = self
            .system_metrics
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire system metrics lock"))?
            .clone();

        let (services, totals) = aggregate_service_data(&self.services);
        let system_data = build_system_data(totals);

        info!(
            total_operations = system_data.total_operations,
            success_rate = system_data.success_rate,
            uptime_seconds = uptime_seconds,
            "System telemetry aggregated"
        );

        Ok(SystemTelemetryData {
            system_metrics: system_data,
            services,
            startup_time,
            uptime_seconds,
        })
    }

    /// Get telemetry data for a specific service
    pub async fn get_service_telemetry(&self, service_name: &str) -> Option<ServiceTelemetryData> {
        self.services.get(service_name).map(|entry| entry.clone())
    }

    /// Reset all telemetry data (for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.services.clear();
        self.event_counter.store(0, Ordering::Relaxed);
        if let Ok(mut metrics) = self.system_metrics.write() {
            *metrics = ServiceMetrics::default();
        }
    }
}

struct ServiceTotals {
    total_operations: u64,
    total_successful: u64,
    total_duration: u64,
    total_items: u64,
}

fn aggregate_service_data(
    services_map: &Arc<DashMap<String, ServiceTelemetryData>>,
) -> (HashMap<String, ServiceTelemetryData>, ServiceTotals) {
    let mut services = HashMap::new();
    let mut totals = ServiceTotals {
        total_operations: 0,
        total_successful: 0,
        total_duration: 0,
        total_items: 0,
    };

    for entry in services_map.iter() {
        let service_data = entry.value().clone();
        totals.total_operations += service_data.total_operations;
        totals.total_successful += service_data.successful_operations;
        totals.total_duration += service_data.total_duration_ms;
        totals.total_items += service_data.total_items_processed;
        services.insert(entry.key().clone(), service_data);
    }

    (services, totals)
}

fn build_system_data(totals: ServiceTotals) -> ServiceTelemetryData {
    let ServiceTotals {
        total_operations,
        total_successful,
        total_duration,
        total_items,
    } = totals;

    ServiceTelemetryData {
        service_name: "PMAT_SYSTEM".to_string(),
        total_operations,
        successful_operations: total_successful,
        failed_operations: total_operations - total_successful,
        total_duration_ms: total_duration,
        avg_duration_ms: if total_operations > 0 {
            total_duration / total_operations
        } else {
            0
        },
        peak_memory_bytes: 0,
        total_items_processed: total_items,
        success_rate: if total_operations > 0 {
            total_successful as f64 / total_operations as f64
        } else {
            0.0
        },
        last_operation_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        operation_counts: HashMap::new(),
    }
}

fn log_operation_result(event_id: &str, input: &TelemetryInput, event_count: u64) {
    if input.metrics.success {
        info!(
            event_id = %event_id,
            service = %input.service_name,
            operation = %input.operation,
            duration_ms = input.metrics.duration_ms,
            items_processed = input.metrics.items_processed,
            event_count = event_count,
            "Operation completed successfully"
        );
    } else {
        error!(
            event_id = %event_id,
            service = %input.service_name,
            operation = %input.operation,
            duration_ms = input.metrics.duration_ms,
            error = %input.metrics.error_message.as_deref().unwrap_or("Unknown error"),
            event_count = event_count,
            "Operation failed"
        );
    }
}
