// AgentMessage constructor and builder methods

impl AgentMessage {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(from: Uuid, to: Uuid, payload: impl Serialize) -> Result<Self, rmp_serde::encode::Error> {
        let payload_bytes = rmp_serde::to_vec(&payload)?;

        Ok(Self {
            header: MessageHeader {
                id: Uuid::new_v4(),
                from,
                to,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("internal error")
                    .as_nanos() as u64,
                correlation_id: None,
                priority: Priority::Normal,
                ttl_ms: 5000, // 5 second default TTL
            },
            payload: Bytes::from(payload_bytes),
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.header.priority = priority;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With correlation.
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.header.correlation_id = Some(correlation_id);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With ttl.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.header.ttl_ms = ttl.as_millis() as u32;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("internal error")
            .as_nanos() as u64;

        let expiry = self.header.timestamp + (self.header.ttl_ms as u64 * 1_000_000);
        now > expiry
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Deserialize payload.
    pub fn deserialize_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, rmp_serde::decode::Error> {
        rmp_serde::from_slice(&self.payload)
    }
}
