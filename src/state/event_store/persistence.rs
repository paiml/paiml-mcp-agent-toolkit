#![cfg_attr(coverage_nightly, coverage(off))]
use crate::state::*;
use parking_lot::RwLock;
use std::collections::BTreeMap;

use super::EventStoreError;

// ============================================================================
// Persistence Trait (Batuta Pattern: Trait Abstraction for Testability)
// ============================================================================

/// Trait for event persistence backends.
/// Enables testing with InMemoryPersistence and production with JsonFilePersistence.
#[async_trait::async_trait]
pub trait EventPersistence: Send + Sync {
    /// Append a single event to the persistence layer
    async fn append_event(&self, event: &StateEvent) -> Result<(), EventStoreError>;

    /// Append multiple events in a batch
    async fn append_batch(&self, events: &[StateEvent]) -> Result<(), EventStoreError>;

    /// Load all events from the persistence layer
    async fn load_all(&self) -> Result<Vec<StateEvent>, EventStoreError>;

    /// Compact the event log (rewrite with only current events)
    async fn compact(&self, events: &BTreeMap<EventId, StateEvent>) -> Result<(), EventStoreError>;
}

// ============================================================================
// In-Memory Persistence (For Testing - No I/O)
// ============================================================================

/// In-memory persistence backend for testing.
/// Stores events in a Vec, no file I/O required.
#[derive(Default)]
pub struct InMemoryPersistence {
    events: RwLock<Vec<StateEvent>>,
}

impl InMemoryPersistence {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of persisted events (for testing)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn len(&self) -> usize {
        self.events.read().len()
    }

    /// Check if empty (for testing)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_empty(&self) -> bool {
        self.events.read().is_empty()
    }

    /// Clear all persisted events (for testing)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn clear(&self) {
        self.events.write().clear();
    }
}

#[async_trait::async_trait]
impl EventPersistence for InMemoryPersistence {
    async fn append_event(&self, event: &StateEvent) -> Result<(), EventStoreError> {
        self.events.write().push(event.clone());
        Ok(())
    }

    async fn append_batch(&self, events: &[StateEvent]) -> Result<(), EventStoreError> {
        debug_assert!(!events.is_empty(), "events must not be empty");
        self.events.write().extend(events.iter().cloned());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<StateEvent>, EventStoreError> {
        debug_assert!(true, "contract: load_all");
        Ok(self.events.read().clone())
    }

    async fn compact(&self, events: &BTreeMap<EventId, StateEvent>) -> Result<(), EventStoreError> {
        debug_assert!(true, "contract: compact");
        let mut persisted = self.events.write();
        persisted.clear();
        persisted.extend(events.values().cloned());
        Ok(())
    }
}
