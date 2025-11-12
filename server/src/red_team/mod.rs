// Red Team Mode: Automated hallucination detection for software repositories
//
// Based on specification: docs/specifications/red-team-mode-spec.md v1.1
// Implements detection of false claims in commit messages, documentation, and code comments

pub mod claim_extractor;

pub use claim_extractor::{Claim, ClaimCategory, ClaimExtractor};
