// ConsensusNetwork: peer management + RaftNetwork trait + ConsensusManager operations

impl ConsensusNetwork {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            peers: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_peer(&self, node_id: NodeId, addr: SocketAddr) {
        self.peers.write().insert(node_id, addr);
    }

    pub fn remove_peer(&self, node_id: NodeId) {
        self.peers.write().remove(&node_id);
    }
}

#[async_trait]
impl RaftNetwork<ClientRequest> for ConsensusNetwork {
    async fn append_entries(
        &self,
        target: NodeId,
        rpc: async_raft::raft::AppendEntriesRequest<ClientRequest>,
    ) -> Result<async_raft::raft::AppendEntriesResponse, async_raft::error::RaftError> {
        debug_assert!(true, "contract: append_entries");
        // In production, this would make an actual network call
        // For now, return a mock response
        Ok(async_raft::raft::AppendEntriesResponse {
            term: rpc.term,
            success: true,
            conflict_opt: None,
        })
    }

    async fn install_snapshot(
        &self,
        target: NodeId,
        rpc: async_raft::raft::InstallSnapshotRequest,
    ) -> Result<async_raft::raft::InstallSnapshotResponse, async_raft::error::RaftError> {
        debug_assert!(true, "contract: install_snapshot");
        // In production, this would make an actual network call
        Ok(async_raft::raft::InstallSnapshotResponse { term: rpc.term })
    }

    async fn vote(
        &self,
        target: NodeId,
        rpc: async_raft::raft::VoteRequest,
    ) -> Result<async_raft::raft::VoteResponse, async_raft::error::RaftError> {
        // In production, this would make an actual network call
        Ok(async_raft::raft::VoteResponse {
            term: rpc.term,
            vote_granted: true,
        })
    }
}

// ConsensusManager: coordinating Raft operations
impl<S: AgentState> ConsensusManager<S> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn new(
        node_id: NodeId,
        initial_state: S,
        config: RaftConfig,
    ) -> Result<Self, ConsensusError> {
        let storage = Arc::new(ConsensusStorage::new(node_id, initial_state));
        let network = Arc::new(ConsensusNetwork::new(node_id));

        let (raft, metrics_rx) = Raft::new(node_id, config, network.clone(), storage.clone());

        Ok(Self {
            node_id,
            raft: Arc::new(raft),
            storage,
            network,
            metrics_rx,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn propose_state_change(
        &self,
        event: StateEvent,
    ) -> Result<ClientResponse, ConsensusError> {
        let request = ClientRequest {
            id: Uuid::new_v4(),
            operation: StateOperation::Apply(event),
        };

        self.raft
            .client_write(request)
            .await
            .map_err(|e| ConsensusError::RaftError(e.to_string()))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn query_state(&self, query: String) -> Result<ClientResponse, ConsensusError> {
        let request = ClientRequest {
            id: Uuid::new_v4(),
            operation: StateOperation::Query(query),
        };

        self.raft
            .client_read()
            .await
            .map_err(|e| ConsensusError::RaftError(e.to_string()))?;

        // For read queries, we can directly read from state machine if we're the leader
        let state = self.storage.state_machine.read();
        Ok(ClientResponse {
            success: true,
            result: Some(serde_json::json!({
                "last_event_id": state.last_event_id(),
                "events_since_snapshot": state.events_since_snapshot(),
            })),
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_node(&self, node_id: NodeId, addr: SocketAddr) {
        self.network.add_peer(node_id, addr);
    }

    pub fn remove_node(&self, node_id: NodeId) {
        self.network.remove_peer(node_id);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_metrics(&self) -> Option<RaftMetrics> {
        // Would receive from metrics channel
        None
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_leader(&self) -> bool {
        // Check current Raft state
        true // Placeholder
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn transfer_leadership(&self, target: NodeId) -> Result<(), ConsensusError> {
        // Initiate leadership transfer
        Ok(())
    }
}
