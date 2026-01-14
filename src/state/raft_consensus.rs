use super::*;
use async_raft::raft::{Entry, EntryPayload, MembershipConfig};
use async_raft::{AppData, AppDataResponse, Config as RaftConfig, Raft, RaftMetrics, RaftNetwork, RaftStorage};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

// Raft consensus for critical state synchronization
pub type NodeId = u64;

// Forward declarations for the Raft instance type
// We'll define this after the concrete types are declared
pub type RaftInstance<S> = Raft<ClientRequest, ClientResponse, ConsensusNetwork, ConsensusStorage<S>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequest {
    pub id: Uuid,
    pub operation: StateOperation,
}

// Implement AppData marker trait
impl AppData for ClientRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
}

// Implement AppDataResponse marker trait
impl AppDataResponse for ClientResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateOperation {
    Apply(StateEvent),
    Snapshot(Vec<u8>),
    Query(String),
}

// Raft storage implementation
pub struct ConsensusStorage<S: AgentState> {
    node_id: NodeId,
    log: Arc<RwLock<BTreeMap<u64, Entry<ClientRequest>>>>,
    state_machine: Arc<RwLock<S>>,
    current_term: Arc<RwLock<u64>>,
    voted_for: Arc<RwLock<Option<NodeId>>>,
    membership: Arc<RwLock<MembershipConfig>>,
    snapshot: Arc<RwLock<Option<RaftSnapshot>>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct RaftSnapshot {
    index: u64,
    term: u64,
    membership: MembershipConfig,
    state: Vec<u8>,
}

impl<S: AgentState> ConsensusStorage<S> {
    pub fn new(node_id: NodeId, initial_state: S) -> Self {
        Self {
            node_id,
            log: Arc::new(RwLock::new(BTreeMap::new())),
            state_machine: Arc::new(RwLock::new(initial_state)),
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            membership: Arc::new(RwLock::new(MembershipConfig::new_initial(node_id))),
            snapshot: Arc::new(RwLock::new(None)),
        }
    }

    async fn apply_entry(&self, entry: &Entry<ClientRequest>) -> ClientResponse {
        match &entry.payload {
            EntryPayload::Normal(request) => {
                match &request.operation {
                    StateOperation::Apply(event) => {
                        let mut state = self.state_machine.write();
                        state.apply_event(event);
                        ClientResponse {
                            success: true,
                            result: Some(serde_json::json!({"applied": true})),
                        }
                    }
                    StateOperation::Snapshot(data) => {
                        if let Ok(new_state) = bincode::deserialize::<S>(data) {
                            *self.state_machine.write() = new_state;
                            ClientResponse {
                                success: true,
                                result: Some(serde_json::json!({"snapshot_applied": true})),
                            }
                        } else {
                            ClientResponse {
                                success: false,
                                result: Some(
                                    serde_json::json!({"error": "Failed to deserialize snapshot"}),
                                ),
                            }
                        }
                    }
                    StateOperation::Query(query) => {
                        // Read-only query
                        let state = self.state_machine.read();
                        ClientResponse {
                            success: true,
                            result: Some(serde_json::json!({
                                "query": query,
                                "last_event_id": state.last_event_id(),
                            })),
                        }
                    }
                }
            }
            EntryPayload::ConfigChange(membership) => {
                *self.membership.write() = membership.clone();
                ClientResponse {
                    success: true,
                    result: Some(serde_json::json!({"membership_updated": true})),
                }
            }
            _ => ClientResponse {
                success: false,
                result: None,
            },
        }
    }
}

#[async_trait]
impl<S: AgentState> RaftStorage<ClientRequest, ClientResponse> for ConsensusStorage<S> {
    type Snapshot = Vec<u8>;
    type ShutdownError = std::io::Error;

    async fn get_membership_config(&self) -> Result<MembershipConfig, std::io::Error> {
        Ok(self.membership.read().clone())
    }

    async fn get_initial_state(&self) -> Result<async_raft::storage::InitialState, std::io::Error> {
        let membership = self.membership.read().clone();
        let mut last_log_index = 0;
        let mut last_log_term = 0;

        if let Some(last_entry) = self.log.read().iter().rev().next() {
            last_log_index = *last_entry.0;
            last_log_term = last_entry.1.term;
        }

        let last_applied_log = if let Some(snapshot) = &*self.snapshot.read() {
            snapshot.index
        } else {
            last_log_index
        };

        Ok(async_raft::storage::InitialState {
            last_log_index,
            last_log_term,
            last_applied_log,
            hard_state: async_raft::storage::HardState {
                current_term: *self.current_term.read(),
                voted_for: *self.voted_for.read(),
            },
            membership,
        })
    }

    async fn save_hard_state(
        &self,
        hs: &async_raft::storage::HardState,
    ) -> Result<(), std::io::Error> {
        *self.current_term.write() = hs.current_term;
        *self.voted_for.write() = hs.voted_for;
        Ok(())
    }

    async fn get_log_entries(
        &self,
        start: u64,
        stop: u64,
    ) -> Result<Vec<Entry<ClientRequest>>, std::io::Error> {
        let log = self.log.read();
        let entries: Vec<_> = log
            .range(start..stop)
            .map(|(_, entry)| entry.clone())
            .collect();
        Ok(entries)
    }

    async fn delete_logs_from(&self, start: u64, stop: Option<u64>) -> Result<(), std::io::Error> {
        let mut log = self.log.write();
        let keys_to_remove: Vec<_> = if let Some(stop) = stop {
            log.range(start..stop).map(|(k, _)| *k).collect()
        } else {
            log.range(start..).map(|(k, _)| *k).collect()
        };

        for key in keys_to_remove {
            log.remove(&key);
        }
        Ok(())
    }

    async fn append_entry_to_log(
        &self,
        entry: &Entry<ClientRequest>,
    ) -> Result<(), std::io::Error> {
        self.log.write().insert(entry.index, entry.clone());
        Ok(())
    }

    async fn replicate_to_log(
        &self,
        entries: &[Entry<ClientRequest>],
    ) -> Result<(), std::io::Error> {
        let mut log = self.log.write();
        for entry in entries {
            log.insert(entry.index, entry.clone());
        }
        Ok(())
    }

    async fn apply_entry_to_state_machine(
        &self,
        index: &u64,
        data: &ClientRequest,
    ) -> Result<ClientResponse, std::io::Error> {
        let log = self.log.read();
        if let Some(entry) = log.get(index) {
            Ok(self.apply_entry(entry).await)
        } else {
            Ok(ClientResponse {
                success: false,
                result: Some(serde_json::json!({"error": "Entry not found"})),
            })
        }
    }

    async fn replicate_to_state_machine(
        &self,
        entries: &[(&u64, &ClientRequest)],
    ) -> Result<(), std::io::Error> {
        for (index, _data) in entries {
            if let Some(entry) = self.log.read().get(index) {
                self.apply_entry(entry).await;
            }
        }
        Ok(())
    }

    async fn do_log_compaction(&self) -> Result<Vec<u8>, std::io::Error> {
        let state = self.state_machine.read();
        let snapshot_data = bincode::serialize(&*state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let last_log_index = self
            .log
            .read()
            .iter()
            .rev()
            .next()
            .map(|(idx, _)| *idx)
            .unwrap_or(0);

        let snapshot = RaftSnapshot {
            index: last_log_index,
            term: *self.current_term.read(),
            membership: self.membership.read().clone(),
            state: snapshot_data.clone(),
        };

        *self.snapshot.write() = Some(snapshot);

        Ok(snapshot_data)
    }

    async fn create_snapshot(
        &self,
    ) -> Result<
        (
            async_raft::storage::CurrentSnapshotData<Self::Snapshot>,
            MembershipConfig,
        ),
        std::io::Error,
    > {
        let snapshot_bytes = self.do_log_compaction().await?;
        let last_applied_log = self
            .log
            .read()
            .iter()
            .rev()
            .next()
            .map(|(idx, _)| *idx)
            .unwrap_or(0);

        let snapshot_data = async_raft::storage::CurrentSnapshotData {
            index: last_applied_log,
            term: *self.current_term.read(),
            membership: self.membership.read().clone(),
            snapshot: snapshot_bytes,
        };

        Ok((snapshot_data, self.membership.read().clone()))
    }

    async fn finalize_snapshot_installation(
        &self,
        index: u64,
        term: u64,
        delete_through: Option<u64>,
        id: String,
        snapshot: Self::Snapshot,
    ) -> Result<(), std::io::Error> {
        // Apply snapshot to state machine
        if let Ok(new_state) = bincode::deserialize::<S>(&snapshot) {
            *self.state_machine.write() = new_state;
        }

        // Delete old log entries
        if let Some(through) = delete_through {
            self.delete_logs_from(0, Some(through + 1)).await?;
        }

        // Save snapshot metadata
        let snapshot = RaftSnapshot {
            index,
            term,
            membership: self.membership.read().clone(),
            state: snapshot,
        };
        *self.snapshot.write() = Some(snapshot);

        Ok(())
    }

    async fn get_current_snapshot(
        &self,
    ) -> Result<Option<async_raft::storage::CurrentSnapshotData<Vec<u8>>>, std::io::Error> {
        if let Some(snapshot) = &*self.snapshot.read() {
            Ok(Some(async_raft::storage::CurrentSnapshotData {
                index: snapshot.index,
                term: snapshot.term,
                membership: snapshot.membership.clone(),
                snapshot: snapshot.state.clone(),
            }))
        } else {
            Ok(None)
        }
    }
}

// Raft network implementation for inter-node communication
pub struct ConsensusNetwork {
    node_id: NodeId,
    peers: Arc<RwLock<BTreeMap<NodeId, SocketAddr>>>,
}

impl ConsensusNetwork {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            peers: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

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

// Consensus manager coordinating Raft operations
pub struct ConsensusManager<S: AgentState> {
    node_id: NodeId,
    raft: Arc<RaftInstance<S>>,
    storage: Arc<ConsensusStorage<S>>,
    network: Arc<ConsensusNetwork>,
    metrics_rx: mpsc::UnboundedReceiver<RaftMetrics>,
}

impl<S: AgentState> ConsensusManager<S> {
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

    pub fn add_node(&self, node_id: NodeId, addr: SocketAddr) {
        self.network.add_peer(node_id, addr);
    }

    pub fn remove_node(&self, node_id: NodeId) {
        self.network.remove_peer(node_id);
    }

    pub async fn get_metrics(&self) -> Option<RaftMetrics> {
        // Would receive from metrics channel
        None
    }

    pub fn is_leader(&self) -> bool {
        // Check current Raft state
        true // Placeholder
    }

    pub async fn transfer_leadership(&self, target: NodeId) -> Result<(), ConsensusError> {
        // Initiate leadership transfer
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Raft error: {0}")]
    RaftError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Not leader")]
    NotLeader,
    #[error("Consensus timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_consensus_storage() {
        let storage = ConsensusStorage::<ExampleState>::new(1, ExampleState::default());

        let entry = Entry {
            term: 1,
            index: 1,
            payload: EntryPayload::Normal(ClientRequest {
                id: Uuid::new_v4(),
                operation: StateOperation::Query("test".to_string()),
            }),
        };

        storage.append_entry_to_log(&entry).await.unwrap();

        let entries = storage.get_log_entries(1, 2).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[actix_rt::test]
    async fn test_consensus_manager() {
        let config = RaftConfig {
            heartbeat_interval: 200,
            election_timeout_min: 300,
            election_timeout_max: 600,
            ..Default::default()
        };

        let manager = ConsensusManager::<ExampleState>::new(1, ExampleState::default(), config)
            .await
            .unwrap();

        assert_eq!(manager.node_id, 1);
    }

    #[test]
    fn test_network_peer_management() {
        let network = ConsensusNetwork::new(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        network.add_peer(2, addr);
        assert_eq!(network.peers.read().len(), 1);

        network.remove_peer(2);
        assert_eq!(network.peers.read().len(), 0);
    }
}
