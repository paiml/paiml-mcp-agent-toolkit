#![cfg_attr(coverage_nightly, coverage(off))]
//! Infra Score module — CI/CD infrastructure quality scoring (0-100).
//!
//! Specification: docs/specifications/components/infra-score.md (in infra repo)
//!
//! 5 dimensions, 100 points total:
//! - Workflow Architecture (25 pts)
//! - Build Reliability (25 pts)
//! - Quality Pipeline (20 pts)
//! - Deployment & Release (15 pts)
//! - Supply Chain Security (15 pts)
//!
//! Hard cutoff: Score <90 = auto-fail.

pub mod aggregator;
pub mod models;
pub mod scorers;

pub use aggregator::InfraScoreAggregator;
pub use models::*;
