// MessageBatch implementation
impl MessageBatch {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            total_size: 0,
            max_size,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add(&mut self, msg: AgentMessage) -> Result<(), BatchError> {
        let msg_size = msg.size_bytes();

        if self.total_size + msg_size > self.max_size {
            return Err(BatchError::BatchFull);
        }

        self.total_size += msg_size;
        self.messages.push(msg);
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_full(&self) -> bool {
        self.total_size >= self.max_size
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn size(&self) -> usize {
        self.total_size
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_size = 0;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn drain(&mut self) -> Vec<AgentMessage> {
        self.total_size = 0;
        std::mem::take(&mut self.messages)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
