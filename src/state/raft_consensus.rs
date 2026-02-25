#![cfg_attr(coverage_nightly, coverage(off))]
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

// Raft network implementation for inter-node communication
pub struct ConsensusNetwork {
    node_id: NodeId,
    peers: Arc<RwLock<BTreeMap<NodeId, SocketAddr>>>,
}

// Consensus manager coordinating Raft operations
pub struct ConsensusManager<S: AgentState> {
    node_id: NodeId,
    raft: Arc<RaftInstance<S>>,
    storage: Arc<ConsensusStorage<S>>,
    network: Arc<ConsensusNetwork>,
    metrics_rx: mpsc::UnboundedReceiver<RaftMetrics>,
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

// --- Implementation files ---
include!("raft_consensus_storage.rs");
include!("raft_consensus_network.rs");
include!("raft_consensus_tests.rs");
