#![cfg_attr(coverage_nightly, coverage(off))]
// Zero-copy message passing protocol
pub mod backpressure;
pub mod circuit_breaker;
pub mod message_format;
pub mod pubsub;
pub mod request_response;

use actix::prelude::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<crate::agents::AgentResponse, crate::agents::AgentError>")]
/// Agent message.
pub struct AgentMessage {
    pub header: MessageHeader,
    pub payload: Bytes, // Zero-copy payload
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Message header.
pub struct MessageHeader {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub timestamp: u64, // Unix timestamp in nanos
    pub correlation_id: Option<Uuid>,
    pub priority: Priority,
    pub ttl_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Priority level for priority.
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

// Custom Serialize/Deserialize for AgentMessage due to Bytes field
include!("messaging_serde.rs");

// AgentMessage constructor and builder methods
include!("messaging_agent.rs");

// Message router with priority queue and RouterError
include!("messaging_router.rs");

// Unit tests (basic tests)
include!("messaging_tests.rs");

// Coverage tests (serialization, builder, router, error)
include!("messaging_coverage_tests.rs");
