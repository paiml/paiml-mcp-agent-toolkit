#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use crate::state::event_store::{EventStore, EventStoreConfig};
use crate::state::snapshot_store::{SnapshotConfig, SnapshotStore};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Types, structs, and error definitions
include!("recovery_types.rs");

// Adaptive snapshot scheduler that learns optimal snapshot intervals
include!("recovery_scheduler.rs");

// State recovery orchestrator with adaptive snapshot scheduling
/// Recovery manager.
pub struct RecoveryManager<S: AgentState> {
    event_store: Arc<EventStore>,
    snapshot_store: Arc<SnapshotStore>,
    snapshot_scheduler: Arc<AdaptiveSnapshotScheduler>,
    _phantom: std::marker::PhantomData<S>,
}

include!("recovery_manager.rs");

// Parallel recovery for partitioned state
include!("recovery_parallel.rs");

// Tests
include!("recovery_tests.rs");
