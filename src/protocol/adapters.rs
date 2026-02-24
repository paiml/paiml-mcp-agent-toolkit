//! Protocol adapter implementations for MCP, HTTP, and CLI

#![cfg_attr(coverage_nightly, coverage(off))]
use super::{
    ComplexityParams, DeadCodeParams, Deserialize, HttpRequest, JsonRpcRequest, JsonRpcResponse,
    Operation, ProtocolAdapter, ProtocolError, QualityGateParams, RequestContext, SatdParams,
    Serialize, UnifiedRequest, UnifiedResponse, Value,
};
use async_trait::async_trait;
use std::collections::HashMap;

// --- Types: HttpResponse, CliRequest, CliResponse ---
include!("adapters_types.rs");

// --- Helper functions: HTTP parsing, routing ---
include!("adapters_helpers.rs");

// --- MCP Adapter ---
include!("adapters_mcp.rs");

// --- HTTP Adapter ---
include!("adapters_http.rs");

// --- CLI Adapter ---
include!("adapters_cli.rs");

// --- Tests: MCP + HTTP adapter tests, property tests ---
include!("adapters_tests_protocol.rs");

// --- Tests: CLI adapter tests, helper function tests, serialization tests ---
include!("adapters_tests_cli.rs");
