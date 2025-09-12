//! HTTP Acceptance Test Helpers
//!
//! Helper modules and utilities for HTTP acceptance testing framework.
//! Provides HTTP client, validators, and common testing utilities.

pub mod http_test_client;

/// Re-export main components for convenience
pub use http_test_client::{HttpResponse, HttpTestClient, HttpTestResult, HttpValidators};
