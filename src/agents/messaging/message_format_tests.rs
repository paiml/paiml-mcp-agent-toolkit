#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_protocol {
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
        let encoded = BinaryProtocol::encode(&msg).unwrap();

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
}
