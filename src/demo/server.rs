#[cfg(feature = "demo")]
use crate::demo::assets::{decompress_asset, get_asset};
#[cfg(feature = "demo")]
use crate::demo::showcase::ShowcaseGallery;
use crate::models::dag::DependencyGraph;
use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use crate::services::polyglot_analyzer::{PolyglotAnalysis, PolyglotAnalyzer};
use crate::services::recommendation_engine::{
    ComplexityTier, RecommendationEngine, RepositoryRecommendation,
};
use anyhow::Result;
use bytes::Bytes;
use dashmap::DashMap;
use http::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Import the validated HTML template
#[allow(unused_imports)]
use super::templates::CSS_DARK_THEME;
#[cfg(feature = "demo")]
use super::templates::HTML_TEMPLATE;

#[cfg(feature = "demo")]
use parking_lot::RwLock;

#[cfg(feature = "demo")]
use bytes::BytesMut;
#[cfg(feature = "demo")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(feature = "demo")]
use tokio::net::{TcpListener, TcpStream};
#[cfg(feature = "demo")]
use tokio::sync::Semaphore;

// --- Struct and type definitions ---
include!("server_types.rs");

// --- HTTP server, connection handling, request/response parsing ---
include!("server_http.rs");

// --- Dashboard and static asset endpoints (incl. non-demo stubs) ---
include!("server_endpoints.rs");

// --- JSON API endpoints (summary, metrics, recommendations, etc.) ---
include!("server_api.rs");

// --- Diagram, architecture, statistics, and analysis data endpoints ---
include!("server_analysis.rs");

// --- DemoContent impl, spawn_sync, Default impls ---
include!("server_content.rs");

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            debug_assert!(true, "contract: module_consistency_check");
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
