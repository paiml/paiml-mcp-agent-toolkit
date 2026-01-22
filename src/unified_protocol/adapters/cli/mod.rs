// CLI adapter - split for file health (CB-040)

use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use serde_json::{json, Value};
use tracing::debug;

use crate::cli::commands::{QddCommands, ScaffoldCommands};
use crate::cli::{
    AnalyzeCommands, Commands, ComplexityOutputFormat, ContextFormat, DagType, OutputFormat,
};
use crate::models::churn::ChurnOutputFormat;
use crate::unified_protocol::{
    CliContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

use super::cli_helpers::{
    big_o_format_to_string, churn_format_to_string, complexity_format_to_string,
    dag_type_to_string, dead_code_format_to_string, deep_context_cache_strategy_to_string,
    deep_context_dag_type_to_string, deep_context_format_to_string, format_to_string,
    graph_metric_type_to_string, graph_metrics_format_to_string,
    incremental_coverage_format_to_string, name_similarity_format_to_string,
    proof_annotation_format_to_string, property_type_filter_to_string,
    provability_format_to_string, satd_format_to_string, satd_severity_to_string,
    symbol_table_format_to_string, symbol_type_filter_to_string, tdg_format_to_string,
    verification_method_filter_to_string, AnalyzeCommandCategory, CliOutput, CommandCategory,
};

/// CLI adapter that converts command line arguments to unified requests
pub struct CliAdapter;

impl CliAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// Include decode core methods (impl CliAdapter block)
include!("decode_core.rs");

// Include dispatch methods (impl CliAdapter block)
include!("dispatch_methods.rs");

// Include decode analyze methods (impl CliAdapter block)
include!("decode_analyze.rs");

// Include protocol adapter impl and CliInput struct
include!("protocol_impl.rs");

// Include CliInput impl and additional CliAdapter impl
include!("cli_input_impl.rs");

#[cfg(test)]
#[path = "../cli_tests.rs"]
mod tests;
