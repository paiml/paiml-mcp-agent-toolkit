#![cfg_attr(coverage_nightly, coverage(off))]

mod json_persistence;
mod persistence;
mod store;

pub use json_persistence::JsonFilePersistence;
pub use persistence::{EventPersistence, InMemoryPersistence};
pub use store::{CompactionResult, EventStore, EventStoreConfig, EventStoreError, EventStoreStats};

#[cfg(test)]
mod tests;
