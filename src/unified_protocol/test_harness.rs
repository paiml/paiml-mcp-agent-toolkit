#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::{debug, info};

use super::adapters::{CliAdapter, HttpAdapter, McpAdapter};
use super::service::{
    AnalysisService, DefaultAnalysisService, DefaultTemplateService, TemplateService,
    UnifiedService,
};
use super::{Protocol, UnifiedRequest};

/// Test harness for validating protocol equivalence across all supported protocols
pub struct TestHarness {
    service: UnifiedService,
    mcp_adapter: McpAdapter,
    http_adapter: HttpAdapter,
    cli_adapter: CliAdapter,
    test_results: Arc<Mutex<TestResults>>,
}

/// Results tracking for test execution
#[derive(Debug, Default, Clone)]
pub struct TestResults {
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub protocol_failures: HashMap<Protocol, Vec<String>>,
    pub equivalence_failures: Vec<EquivalenceFailure>,
}

/// Details about a protocol equivalence failure
#[derive(Debug, Clone)]
pub struct EquivalenceFailure {
    pub test_name: String,
    pub protocols: (Protocol, Protocol),
    pub expected: Value,
    pub actual: Value,
    pub difference: String,
}

type TestFunction = Box<
    dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TestError>> + Send>>
        + Send
        + Sync,
>;

/// Results from running the complete test suite
#[derive(Debug)]
pub struct TestSuiteResults {
    pub passed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub summary: Option<TestResults>,
}

impl TestSuiteResults {
    fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
            summary: None,
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.passed.len() + self.failed.len();
        if total == 0 {
            0.0
        } else {
            self.passed.len() as f64 / total as f64
        }
    }

    pub fn is_successful(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Errors that can occur during testing
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Protocol failure: {0}")]
    ProtocolFailure(String),

    #[error("Unexpected success: {0}")]
    UnexpectedSuccess(String),

    #[error("Inconsistent behavior: {0}")]
    InconsistentBehavior(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Service error: {0}")]
    ServiceError(String),

    #[error("Response error: {0}")]
    ResponseError(String),

    #[error("HTTP error {0}: {1}")]
    HttpError(StatusCode, String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

include!("test_harness_methods.rs");
include!("test_harness_tests.rs");
