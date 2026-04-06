#[async_trait::async_trait]
impl Service for TelemetryService {
    type Input = TelemetryInput;
    type Output = TelemetryOutput;
    type Error = anyhow::Error;

    #[instrument(skip(self, input))]
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        span!(Level::INFO, "telemetry_process", service = %input.service_name)
            .in_scope(|| async { self.record_operation(input).await })
            .await
    }

    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError> {
        if input.event_type.is_empty() {
            return Err(ValidationError::MissingField {
                field: "event_type".to_string(),
            });
        }

        if input.service_name.is_empty() {
            return Err(ValidationError::MissingField {
                field: "service_name".to_string(),
            });
        }

        if input.operation.is_empty() {
            return Err(ValidationError::MissingField {
                field: "operation".to_string(),
            });
        }

        Ok(())
    }

    fn metrics(&self) -> ServiceMetrics {
        self.system_metrics
            .read()
            .map(|m| m.clone())
            .unwrap_or_default()
    }
}

// Global telemetry service instance (singleton pattern)
lazy_static::lazy_static! {
    static ref TELEMETRY: Arc<TelemetryService> = Arc::new(TelemetryService::new());
}

/// Get the global telemetry service instance - THE ONE way to access telemetry
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn telemetry() -> Arc<TelemetryService> {
    TELEMETRY.clone()
}

/// Convenience macro for recording operation telemetry
#[macro_export]
macro_rules! record_telemetry {
    ($service:expr, $operation:expr, $duration:expr, $success:expr) => {
        record_telemetry!($service, $operation, $duration, $success, 1, HashMap::new())
    };
    ($service:expr, $operation:expr, $duration:expr, $success:expr, $items:expr) => {
        record_telemetry!(
            $service,
            $operation,
            $duration,
            $success,
            $items,
            HashMap::new()
        )
    };
    ($service:expr, $operation:expr, $duration:expr, $success:expr, $items:expr, $tags:expr) => {{
        let input = $crate::services::telemetry_service::TelemetryInput {
            event_type: "operation".to_string(),
            service_name: $service.to_string(),
            operation: $operation.to_string(),
            metrics: $crate::services::telemetry_service::OperationMetrics {
                duration_ms: $duration.as_millis() as u64,
                items_processed: $items,
                memory_bytes: None,
                cpu_time_ms: None,
                success: $success,
                error_message: None,
            },
            tags: $tags,
            properties: std::collections::HashMap::new(),
        };

        let _ = $crate::services::telemetry_service::telemetry()
            .record_operation(input)
            .await;
    }};
}
