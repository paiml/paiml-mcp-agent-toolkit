//! Unified code intelligence interface
//!
//! Provides a comprehensive analysis interface that combines DAG representation,
//! duplicate detection, dead code analysis, and more into a single API.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::unified_ast::AstDag;
use crate::services::{
    context::analyze_project,
    dag_builder::DagBuilder,
    dead_code_analyzer::{DeadCodeAnalyzer, DeadCodeReport},
    duplicate_detector::CloneReport,
    mermaid_generator::{MermaidGenerator, MermaidOptions},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

include!("code_intelligence_types.rs");
include!("code_intelligence_cache.rs");
include!("code_intelligence_engine.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("code_intelligence_tests.rs");
    include!("code_intelligence_tests_reports.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    include!("code_intelligence_proptests.rs");
}
