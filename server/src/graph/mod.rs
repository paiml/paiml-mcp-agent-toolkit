// Graph analysis module for PMAT
// Following extreme TDD with zero SATD tolerance

pub mod aprender_adapter;
pub mod builder;
pub mod centrality;
pub mod community;
pub mod context_annotator;
pub mod pagerank;
// pub mod storage;  // Phase 7.1: WIP - requires trueno-db v0.3.1 full integration
pub mod structure;
pub mod symbol_table;
pub mod types;

// #[cfg(feature = "simd")]
// pub mod simd_pagerank;

pub mod parallel_louvain;

pub use builder::*;
pub use centrality::*;
pub use community::*;
pub use context_annotator::*;
pub use pagerank::*;
pub use structure::*;
pub use types::*;

#[cfg(test)]
mod tests;
