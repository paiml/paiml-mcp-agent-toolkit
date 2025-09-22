use super::*;
use super::{AgentMessage, MessageHeader, Priority};
use crate::agents::{AgentError, AgentResponse};
use actix::prelude::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Message metadata for extended functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub content_type: String,
    pub encoding: String,
    pub compressed: bool,
    pub ttl_ms: Option<u64>,
    pub trace_id: Option<String>,
}

// Extensions for AgentMessage
pub trait MessageExtensions {
    fn with_metadata(self, metadata: MessageMetadata) -> MessageWithMetadata;
    fn is_expired(&self) -> bool;
    fn size_bytes(&self) -> usize;
}

pub struct MessageWithMetadata {
    pub message: AgentMessage,
    pub metadata: MessageMetadata,
}

impl MessageExtensions for AgentMessage {
    fn with_metadata(self, metadata: MessageMetadata) -> MessageWithMetadata {
        MessageWithMetadata {
            message: self,
            metadata,
        }
    }

    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let created = self.header.timestamp as u64 * 1000;
        let ttl = self.header.ttl_ms as u64;
        now - created > ttl
    }

    fn size_bytes(&self) -> usize {
        std::mem::size_of::<MessageHeader>() + self.payload.len()
    }
}

// Efficient binary protocol for network transmission
pub struct BinaryProtocol;

impl BinaryProtocol {
    pub fn encode(msg: &AgentMessage) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::with_capacity(1024);

        // Version byte
        buf.put_u8(1);

        // Header
        let header_bytes = bincode::serialize(&msg.header)
            .map_err(|e| ProtocolError::EncodingError(e.to_string()))?;
        buf.put_u32(header_bytes.len() as u32);
        buf.put_slice(&header_bytes);

        // Payload
        buf.put_u32(msg.payload.len() as u32);
        buf.put_slice(&msg.payload);

        // Checksum
        let checksum = crc32fast::hash(&buf[..]);
        buf.put_u32(checksum);

        Ok(buf.freeze())
    }

    pub fn decode(mut data: Bytes) -> Result<AgentMessage, ProtocolError> {
        if data.len() < 5 {
            return Err(ProtocolError::InvalidMessage(
                "Message too short".to_string(),
            ));
        }

        // Version
        let version = data.get_u8();
        if version != 1 {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        // Verify checksum
        let content_len = data.len() - 4;
        let content = data.slice(0..content_len);
        let expected_checksum = data.slice(content_len..).get_u32();
        let actual_checksum = crc32fast::hash(&content[..]);

        if expected_checksum != actual_checksum {
            return Err(ProtocolError::ChecksumMismatch {
                expected: expected_checksum,
                actual: actual_checksum,
            });
        }

        // Decode header
        let header_len = data.get_u32() as usize;
        let header_bytes = data.copy_to_bytes(header_len);
        let header: MessageHeader = bincode::deserialize(&header_bytes)
            .map_err(|e| ProtocolError::DecodingError(e.to_string()))?;

        // Decode payload
        let payload_len = data.get_u32() as usize;
        let payload = data.copy_to_bytes(payload_len);

        Ok(AgentMessage { header, payload })
    }
}

#[derive(Debug, thiserror::Error)]
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
pub struct MessageBatch {
    messages: Vec<AgentMessage>,
    total_size: usize,
    max_size: usize,
}

impl MessageBatch {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            total_size: 0,
            max_size,
        }
    }

    pub fn add(&mut self, msg: AgentMessage) -> Result<(), BatchError> {
        let msg_size = msg.size_bytes();

        if self.total_size + msg_size > self.max_size {
            return Err(BatchError::BatchFull);
        }

        self.total_size += msg_size;
        self.messages.push(msg);
        Ok(())
    }

    pub fn is_full(&self) -> bool {
        self.total_size >= self.max_size
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn size(&self) -> usize {
        self.total_size
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_size = 0;
    }

    pub fn drain(&mut self) -> Vec<AgentMessage> {
        self.total_size = 0;
        std::mem::take(&mut self.messages)
    }

    pub fn encode(&self) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::with_capacity(self.total_size + 100);

        // Batch header
        buf.put_u32(self.messages.len() as u32);
        buf.put_u32(self.total_size as u32);

        // Encode each message
        for msg in &self.messages {
            let encoded = BinaryProtocol::encode(msg)?;
            buf.put_u32(encoded.len() as u32);
            buf.put_slice(&encoded);
        }

        Ok(buf.freeze())
    }

    pub fn decode(mut data: Bytes) -> Result<Vec<AgentMessage>, ProtocolError> {
        if data.len() < 8 {
            return Err(ProtocolError::InvalidMessage(
                "Batch header too short".to_string(),
            ));
        }

        let count = data.get_u32() as usize;
        let _total_size = data.get_u32();

        let mut messages = Vec::with_capacity(count);

        for _ in 0..count {
            let msg_len = data.get_u32() as usize;
            let msg_bytes = data.copy_to_bytes(msg_len);
            let msg = BinaryProtocol::decode(msg_bytes)?;
            messages.push(msg);
        }

        Ok(messages)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Batch is full")]
    BatchFull,
    #[error("Message too large for batch")]
    MessageTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_protocol() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let data = serde_json::json!({"test": "data"});

        let msg = AgentMessage::new(from, to, data).unwrap();

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        let decoded = BinaryProtocol::decode(encoded).unwrap();

        assert_eq!(decoded.header.id, msg.header.id);
        assert_eq!(decoded.header.from, msg.header.from);
        assert_eq!(decoded.header.to, msg.header.to);
    }

    #[test]
    fn test_message_batch() {
        let mut batch = MessageBatch::new(10000);

        for i in 0..5 {
            let from = Uuid::new_v4();
            let to = Uuid::new_v4();
            let data = serde_json::json!({"index": i});
            let msg = AgentMessage::new(from, to, data).unwrap();
            batch.add(msg).unwrap();
        }

        assert_eq!(batch.len(), 5);

        let encoded = batch.encode().unwrap();
        let decoded = MessageBatch::decode(encoded).unwrap();

        assert_eq!(decoded.len(), 5);
    }
}
