#![cfg_attr(coverage_nightly, coverage(off))]
use super::{AgentMessage, MessageHeader};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Message metadata for extended functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata for message.
pub struct MessageMetadata {
    pub content_type: String,
    pub encoding: String,
    pub compressed: bool,
    pub ttl_ms: Option<u64>,
    pub trace_id: Option<String>,
}

// Extensions for AgentMessage
/// Trait defining Message extensions behavior.
pub trait MessageExtensions {
    fn with_metadata(self, metadata: MessageMetadata) -> MessageWithMetadata;
    fn is_expired(&self) -> bool;
    fn size_bytes(&self) -> usize;
}

/// Metadata for message with.
pub struct MessageWithMetadata {
    pub message: AgentMessage,
    pub metadata: MessageMetadata,
}

// Efficient binary protocol for network transmission
/// Binary protocol.
pub struct BinaryProtocol;

#[derive(Debug, thiserror::Error)]
/// Error variants for protocol operations.
pub enum ProtocolError {
    #[error("Encoding error: {0}")]
    EncodingError(String),
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: u32, actual: u32 },
}

// Message batching for efficiency
/// Message batch.
pub struct MessageBatch {
    messages: Vec<AgentMessage>,
    total_size: usize,
    max_size: usize,
}

#[derive(Debug, thiserror::Error)]
/// Error variants for batch operations.
pub enum BatchError {
    #[error("Batch is full")]
    BatchFull,
    #[error("Message too large for batch")]
    MessageTooLarge,
}

// --- Implementation includes ---
include!("message_format_protocol.rs");
include!("message_format_batch.rs");

// --- Test includes ---
include!("message_format_tests.rs");
include!("message_format_tests_batch.rs");
