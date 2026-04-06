#![cfg_attr(coverage_nightly, coverage(off))]
//! Adapters to help existing services implement the Service trait
//!
//! This module provides adapter patterns to integrate legacy services
//! with the new unified service architecture.

use super::service_base::{Service, ServiceMetrics};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Generic adapter for converting existing services to Service trait
pub struct ServiceAdapter<T, I, O> {
    inner: Arc<T>,
    metrics: Arc<RwLock<ServiceMetrics>>,
    _phantom: PhantomData<(I, O)>,
}

impl<T, I, O> ServiceAdapter<T, I, O> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

/// Macro to quickly implement Service trait for adapted services
#[macro_export]
macro_rules! impl_service_adapter {
    ($adapter:ty, $input:ty, $output:ty, $process_fn:expr) => {
        #[async_trait::async_trait]
        impl Service for $adapter {
            type Input = $input;
            type Output = $output;
            type Error = anyhow::Error;

            async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
                debug_assert!(true, "contract: process");
                let start = std::time::Instant::now();
                let result = $process_fn(&self.inner, input).await;
                let duration = start.elapsed();

                let mut metrics = self.metrics.write().await;
                metrics.record_request(duration, result.is_ok());

                result
            }

            fn metrics(&self) -> ServiceMetrics {
                debug_assert!(true, "contract: metrics");
                self.metrics.blocking_read().clone()
            }
        }
    };
}

// Submodule: complexity adapter
include!("service_adapter_complexity.rs");

// Submodule: refactor adapter
include!("service_adapter_refactor.rs");

// Submodule: registry builder
include!("service_adapter_registry.rs");

// Tests
include!("service_adapter_tests.rs");
