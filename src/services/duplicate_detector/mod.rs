#![cfg_attr(coverage_nightly, coverage(off))]
//! High-performance duplicate code detection using LSH and `MinHash`
//!
//! This module implements the duplicate code detection system as specified
//! in the dupe-code-redux-spec.md using locality-sensitive hashing (LSH),
//! `MinHash` signatures, and cross-language normalization.

mod engine;
pub(crate) mod keywords;
mod lsh;
mod minhash;
mod tokenizer;
pub mod types;

pub use engine::DuplicateDetectionEngine;
pub use lsh::LshIndex;
pub use minhash::MinHashGenerator;
pub use tokenizer::UniversalFeatureExtractor;
pub use types::{
    CloneGroup, CloneInstance, CloneReport, CloneSummary, CloneType, CodeFragment,
    DuplicateDetectionConfig, DuplicationHotspot, FragmentId, Language, MinHashSignature, Token,
    TokenKind,
};

// Tests extracted to duplicate_detector_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "../duplicate_detector_tests.rs"]
mod tests;
