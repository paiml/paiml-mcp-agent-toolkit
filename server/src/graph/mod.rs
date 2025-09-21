// Graph analysis module for PMAT
// Following extreme TDD with zero SATD tolerance

pub mod types;
pub mod builder;
pub mod pagerank;
pub mod community;
pub mod centrality;
pub mod structure;
pub mod context_annotator;

#[cfg(feature = "simd")]
pub mod simd_pagerank;

pub mod parallel_louvain;

pub use types::*;
pub use builder::*;
pub use pagerank::*;
pub use community::*;
pub use centrality::*;
pub use structure::*;
pub use context_annotator::*;

#[cfg(test)]
mod tests;