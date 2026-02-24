#![cfg_attr(coverage_nightly, coverage(off))]
//! Telemetry & Logging System per SPECIFICATION.md Section 35
//!
//! This module provides comprehensive observability infrastructure for PMAT,
//! implementing structured logging, metrics collection, and performance monitoring
//! following the Toyota Way principle of ONE implementation for telemetry.

use crate::services::service_base::{Service, ServiceMetrics, ValidationError};
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, instrument, span, trace, Level};
use uuid::Uuid;

// Types: structs, enums, and TelemetryService struct + Default impl
include!("telemetry_service_types.rs");

// Core impl block: new(), record_operation(), update_*, get_*, reset()
// Also includes free helper functions: aggregate_service_data, build_system_data, log_operation_result
include!("telemetry_service_impl.rs");

// Service trait impl, global singleton, telemetry() accessor, record_telemetry! macro
include!("telemetry_service_trait.rs");

// Unit tests and property tests
include!("telemetry_service_tests.rs");
