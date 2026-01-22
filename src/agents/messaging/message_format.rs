use super::{AgentMessage, MessageHeader};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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
            .expect("internal error")
            .as_millis() as u64;
        let created = self.header.timestamp * 1000;
        let ttl = u64::from(self.header.ttl_ms);
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

    pub fn decode(data: Bytes) -> Result<AgentMessage, ProtocolError> {
        if data.len() < 5 {
            return Err(ProtocolError::InvalidMessage(
                "Message too short".to_string(),
            ));
        }

        // Calculate checksum first (before consuming any bytes)
        let content_len = data.len() - 4;
        let content = data.slice(0..content_len);
        let mut checksum_bytes = data.slice(content_len..);
        let expected_checksum = checksum_bytes.get_u32();
        let actual_checksum = crc32fast::hash(&content[..]);

        // Now parse the content
        let mut data = content;

        // Version
        let version = data.get_u8();
        if version != 1 {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

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

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
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
    use uuid::Uuid;

    #[test]
    fn test_binary_protocol() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let data = serde_json::json!({"test": "data"});

        let msg = AgentMessage::new(from, to, data).expect("internal error");

        let encoded = BinaryProtocol::encode(&msg).expect("internal error");
        let decoded = BinaryProtocol::decode(encoded).expect("internal error");

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
            let msg = AgentMessage::new(from, to, data).expect("internal error");
            batch.add(msg).expect("internal error");
        }

        assert_eq!(batch.len(), 5);

        let encoded = batch.encode().expect("internal error");
        let decoded = MessageBatch::decode(encoded).expect("internal error");

        assert_eq!(decoded.len(), 5);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::agents::messaging::Priority;
    use uuid::Uuid;

    // MessageMetadata tests
    #[test]
    fn test_message_metadata_debug() {
        let metadata = MessageMetadata {
            content_type: "application/json".to_string(),
            encoding: "utf-8".to_string(),
            compressed: false,
            ttl_ms: Some(5000),
            trace_id: Some("trace-123".to_string()),
        };

        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("MessageMetadata"));
        assert!(debug_str.contains("content_type"));
        assert!(debug_str.contains("encoding"));
        assert!(debug_str.contains("compressed"));
    }

    #[test]
    fn test_message_metadata_clone() {
        let metadata = MessageMetadata {
            content_type: "text/plain".to_string(),
            encoding: "ascii".to_string(),
            compressed: true,
            ttl_ms: None,
            trace_id: None,
        };

        let cloned = metadata.clone();

        assert_eq!(cloned.content_type, metadata.content_type);
        assert_eq!(cloned.encoding, metadata.encoding);
        assert_eq!(cloned.compressed, metadata.compressed);
        assert_eq!(cloned.ttl_ms, metadata.ttl_ms);
        assert_eq!(cloned.trace_id, metadata.trace_id);
    }

    #[test]
    fn test_message_metadata_serialize_deserialize() {
        let metadata = MessageMetadata {
            content_type: "application/octet-stream".to_string(),
            encoding: "binary".to_string(),
            compressed: true,
            ttl_ms: Some(10000),
            trace_id: Some("trace-abc-123".to_string()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: MessageMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content_type, metadata.content_type);
        assert_eq!(deserialized.encoding, metadata.encoding);
        assert_eq!(deserialized.compressed, metadata.compressed);
        assert_eq!(deserialized.ttl_ms, metadata.ttl_ms);
        assert_eq!(deserialized.trace_id, metadata.trace_id);
    }

    #[test]
    fn test_message_metadata_with_none_values() {
        let metadata = MessageMetadata {
            content_type: "text/html".to_string(),
            encoding: "utf-8".to_string(),
            compressed: false,
            ttl_ms: None,
            trace_id: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: MessageMetadata = serde_json::from_str(&json).unwrap();

        assert!(deserialized.ttl_ms.is_none());
        assert!(deserialized.trace_id.is_none());
    }

    // MessageExtensions trait tests
    #[test]
    fn test_message_extensions_with_metadata() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();

        let metadata = MessageMetadata {
            content_type: "text/plain".to_string(),
            encoding: "utf-8".to_string(),
            compressed: false,
            ttl_ms: Some(3000),
            trace_id: Some("trace-001".to_string()),
        };

        let with_metadata = msg.with_metadata(metadata.clone());

        assert_eq!(with_metadata.metadata.content_type, "text/plain");
        assert_eq!(with_metadata.metadata.encoding, "utf-8");
        assert!(!with_metadata.metadata.compressed);
        assert_eq!(with_metadata.metadata.ttl_ms, Some(3000));
        assert_eq!(
            with_metadata.metadata.trace_id,
            Some("trace-001".to_string())
        );
    }

    #[test]
    fn test_message_extensions_size_bytes() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "hello world").unwrap();
        let size = msg.size_bytes();

        // Size should be header size + payload length
        assert!(size > 0);
        assert!(size >= std::mem::size_of::<super::MessageHeader>());
    }

    #[test]
    fn test_message_extensions_size_bytes_empty_payload() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "").unwrap();
        let size = msg.size_bytes();

        // Even empty payload should have header size
        assert!(size >= std::mem::size_of::<super::MessageHeader>());
    }

    // BinaryProtocol tests
    #[test]
    fn test_binary_protocol_encode_decode_roundtrip() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let msg = AgentMessage::new(from, to, "test payload").unwrap();

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        let decoded = BinaryProtocol::decode(encoded).unwrap();

        assert_eq!(decoded.header.id, msg.header.id);
        assert_eq!(decoded.header.from, msg.header.from);
        assert_eq!(decoded.header.to, msg.header.to);
        assert_eq!(decoded.header.priority, msg.header.priority);
        assert_eq!(decoded.header.ttl_ms, msg.header.ttl_ms);
        assert_eq!(decoded.payload, msg.payload);
    }

    #[test]
    fn test_binary_protocol_encode_with_all_priorities() {
        for priority in [
            Priority::Critical,
            Priority::High,
            Priority::Normal,
            Priority::Low,
        ] {
            let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
                .unwrap()
                .with_priority(priority);

            let encoded = BinaryProtocol::encode(&msg).unwrap();
            let decoded = BinaryProtocol::decode(encoded).unwrap();

            assert_eq!(decoded.header.priority, priority);
        }
    }

    #[test]
    fn test_binary_protocol_decode_message_too_short() {
        let data = Bytes::from(vec![1, 2, 3]); // Only 3 bytes, too short

        let result = BinaryProtocol::decode(data);
        assert!(result.is_err());

        if let Err(ProtocolError::InvalidMessage(msg)) = result {
            assert!(msg.contains("too short"));
        } else {
            panic!("Expected InvalidMessage error");
        }
    }

    #[test]
    fn test_binary_protocol_decode_unsupported_version() {
        // Create valid-looking message but with wrong version
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        let mut encoded = BinaryProtocol::encode(&msg).unwrap();

        // Modify the encoded bytes to have wrong version
        let mut bytes = encoded.to_vec();
        bytes[0] = 99; // Set version to 99

        // Recalculate checksum for the modified content
        let content_len = bytes.len() - 4;
        let content = &bytes[0..content_len];
        let new_checksum = crc32fast::hash(content);
        bytes[content_len..].copy_from_slice(&new_checksum.to_be_bytes());

        let result = BinaryProtocol::decode(Bytes::from(bytes));
        assert!(result.is_err());

        if let Err(ProtocolError::UnsupportedVersion(v)) = result {
            assert_eq!(v, 99);
        } else {
            panic!("Expected UnsupportedVersion error");
        }
    }

    #[test]
    fn test_binary_protocol_decode_checksum_mismatch() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        let encoded = BinaryProtocol::encode(&msg).unwrap();

        // Corrupt the payload without updating checksum
        let mut bytes = encoded.to_vec();
        if bytes.len() > 10 {
            bytes[10] ^= 0xFF; // Flip bits in the middle
        }

        let result = BinaryProtocol::decode(Bytes::from(bytes));
        assert!(result.is_err());

        match result {
            Err(ProtocolError::ChecksumMismatch { expected, actual }) => {
                assert_ne!(expected, actual);
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    // ProtocolError tests
    #[test]
    fn test_protocol_error_encoding_error() {
        let err = ProtocolError::EncodingError("encoding failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Encoding error"));
        assert!(display.contains("encoding failed"));

        let debug = format!("{:?}", err);
        assert!(debug.contains("EncodingError"));
    }

    #[test]
    fn test_protocol_error_decoding_error() {
        let err = ProtocolError::DecodingError("decoding failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Decoding error"));
        assert!(display.contains("decoding failed"));
    }

    #[test]
    fn test_protocol_error_invalid_message() {
        let err = ProtocolError::InvalidMessage("bad format".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid message"));
        assert!(display.contains("bad format"));
    }

    #[test]
    fn test_protocol_error_unsupported_version() {
        let err = ProtocolError::UnsupportedVersion(42);
        let display = format!("{}", err);
        assert!(display.contains("Unsupported"));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_protocol_error_checksum_mismatch() {
        let err = ProtocolError::ChecksumMismatch {
            expected: 12345,
            actual: 67890,
        };
        let display = format!("{}", err);
        assert!(display.contains("Checksum mismatch"));
        assert!(display.contains("12345"));
        assert!(display.contains("67890"));
    }

    // MessageBatch tests
    #[test]
    fn test_message_batch_new() {
        let batch = MessageBatch::new(1024);

        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
        assert_eq!(batch.size(), 0);
        assert!(!batch.is_full());
    }

    #[test]
    fn test_message_batch_add() {
        let mut batch = MessageBatch::new(10000);

        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        let result = batch.add(msg);

        assert!(result.is_ok());
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
        assert!(batch.size() > 0);
    }

    #[test]
    fn test_message_batch_add_until_full() {
        let mut batch = MessageBatch::new(500); // Small batch size

        // Add messages until batch is full
        let mut added = 0;
        for _ in 0..100 {
            let msg =
                AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test data payload").unwrap();
            match batch.add(msg) {
                Ok(()) => added += 1,
                Err(BatchError::BatchFull) => break,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        assert!(added > 0);
        assert!(batch.is_full() || batch.len() > 0);
    }

    #[test]
    fn test_message_batch_add_returns_error_when_full() {
        let mut batch = MessageBatch::new(100); // Very small batch

        // Fill the batch
        let large_payload = "x".repeat(80);
        let msg1 = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), &large_payload).unwrap();
        let _ = batch.add(msg1);

        // Try to add another - should fail
        let msg2 = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), &large_payload).unwrap();
        let result = batch.add(msg2);

        assert!(matches!(result, Err(BatchError::BatchFull)));
    }

    #[test]
    fn test_message_batch_is_empty() {
        let mut batch = MessageBatch::new(1000);
        assert!(batch.is_empty());

        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        batch.add(msg).unwrap();
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_message_batch_len() {
        let mut batch = MessageBatch::new(10000);

        for i in 0..5 {
            let msg =
                AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), format!("msg {}", i)).unwrap();
            batch.add(msg).unwrap();
            assert_eq!(batch.len(), i + 1);
        }
    }

    #[test]
    fn test_message_batch_size() {
        let mut batch = MessageBatch::new(10000);

        let initial_size = batch.size();
        assert_eq!(initial_size, 0);

        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test payload").unwrap();
        batch.add(msg).unwrap();

        assert!(batch.size() > initial_size);
    }

    #[test]
    fn test_message_batch_clear() {
        let mut batch = MessageBatch::new(10000);

        // Add some messages
        for _ in 0..3 {
            let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
            batch.add(msg).unwrap();
        }

        assert_eq!(batch.len(), 3);
        assert!(batch.size() > 0);

        // Clear the batch
        batch.clear();

        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
        assert_eq!(batch.size(), 0);
    }

    #[test]
    fn test_message_batch_drain() {
        let mut batch = MessageBatch::new(10000);

        // Add some messages
        for i in 0..3 {
            let msg =
                AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), format!("msg {}", i)).unwrap();
            batch.add(msg).unwrap();
        }

        assert_eq!(batch.len(), 3);

        // Drain the batch
        let messages = batch.drain();

        assert_eq!(messages.len(), 3);
        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
        assert_eq!(batch.size(), 0);
    }

    #[test]
    fn test_message_batch_encode_decode_roundtrip() {
        let mut batch = MessageBatch::new(10000);

        // Add messages
        for i in 0..5 {
            let msg = AgentMessage::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                serde_json::json!({"index": i}),
            )
            .unwrap();
            batch.add(msg).unwrap();
        }

        let encoded = batch.encode().unwrap();
        let decoded = MessageBatch::decode(encoded).unwrap();

        assert_eq!(decoded.len(), 5);
    }

    #[test]
    fn test_message_batch_decode_empty() {
        let batch = MessageBatch::new(10000);
        let encoded = batch.encode().unwrap();
        let decoded = MessageBatch::decode(encoded).unwrap();

        assert!(decoded.is_empty());
    }

    #[test]
    fn test_message_batch_decode_header_too_short() {
        let data = Bytes::from(vec![0, 0, 0]); // Only 3 bytes, need at least 8

        let result = MessageBatch::decode(data);
        assert!(result.is_err());

        if let Err(ProtocolError::InvalidMessage(msg)) = result {
            assert!(msg.contains("too short"));
        } else {
            panic!("Expected InvalidMessage error");
        }
    }

    // BatchError tests
    #[test]
    fn test_batch_error_batch_full() {
        let err = BatchError::BatchFull;
        let display = format!("{}", err);
        assert!(display.contains("full"));

        let debug = format!("{:?}", err);
        assert!(debug.contains("BatchFull"));
    }

    #[test]
    fn test_batch_error_message_too_large() {
        let err = BatchError::MessageTooLarge;
        let display = format!("{}", err);
        assert!(display.contains("too large"));

        let debug = format!("{:?}", err);
        assert!(debug.contains("MessageTooLarge"));
    }

    // MessageWithMetadata tests
    #[test]
    fn test_message_with_metadata_struct() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        let metadata = MessageMetadata {
            content_type: "application/json".to_string(),
            encoding: "utf-8".to_string(),
            compressed: true,
            ttl_ms: Some(5000),
            trace_id: Some("trace-id".to_string()),
        };

        let mwm = MessageWithMetadata {
            message: msg.clone(),
            metadata: metadata.clone(),
        };

        assert_eq!(mwm.message.header.id, msg.header.id);
        assert_eq!(mwm.metadata.content_type, "application/json");
        assert_eq!(mwm.metadata.encoding, "utf-8");
        assert!(mwm.metadata.compressed);
    }

    // Edge case tests
    #[test]
    fn test_binary_protocol_with_large_payload() {
        let large_payload = "x".repeat(10000);
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), &large_payload).unwrap();

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        let decoded = BinaryProtocol::decode(encoded).unwrap();

        assert_eq!(decoded.header.id, msg.header.id);
        assert_eq!(decoded.payload, msg.payload);
    }

    #[test]
    fn test_binary_protocol_with_empty_payload() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "").unwrap();

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        let decoded = BinaryProtocol::decode(encoded).unwrap();

        assert_eq!(decoded.header.id, msg.header.id);
    }

    #[test]
    fn test_binary_protocol_with_special_characters() {
        let special = "Hello\x00World\n\t\r\u{1F600}";
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), special).unwrap();

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        let decoded = BinaryProtocol::decode(encoded).unwrap();

        assert_eq!(decoded.header.id, msg.header.id);
    }

    #[test]
    fn test_binary_protocol_with_correlation_id() {
        let correlation_id = Uuid::new_v4();
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_correlation(correlation_id);

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        let decoded = BinaryProtocol::decode(encoded).unwrap();

        assert_eq!(decoded.header.correlation_id, Some(correlation_id));
    }

    #[test]
    fn test_message_batch_single_message() {
        let mut batch = MessageBatch::new(10000);
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "single").unwrap();
        batch.add(msg).unwrap();

        let encoded = batch.encode().unwrap();
        let decoded = MessageBatch::decode(encoded).unwrap();

        assert_eq!(decoded.len(), 1);
    }

    #[test]
    fn test_message_batch_reuse_after_drain() {
        let mut batch = MessageBatch::new(10000);

        // First round
        for i in 0..3 {
            let msg =
                AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), format!("msg {}", i)).unwrap();
            batch.add(msg).unwrap();
        }

        let _ = batch.drain();
        assert!(batch.is_empty());

        // Second round - should work
        for i in 0..2 {
            let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), format!("new msg {}", i))
                .unwrap();
            batch.add(msg).unwrap();
        }

        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_message_batch_reuse_after_clear() {
        let mut batch = MessageBatch::new(10000);

        // First round
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "first").unwrap();
        batch.add(msg).unwrap();

        batch.clear();
        assert!(batch.is_empty());

        // Second round
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "second").unwrap();
        batch.add(msg).unwrap();

        assert_eq!(batch.len(), 1);
    }
}
