// Message extensions implementation
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

// Binary protocol encode/decode implementation
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
