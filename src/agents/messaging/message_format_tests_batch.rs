#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_batch {
    use super::*;
    use uuid::Uuid;

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
