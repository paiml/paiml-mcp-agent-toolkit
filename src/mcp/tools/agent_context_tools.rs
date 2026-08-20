#![cfg_attr(coverage_nightly, coverage(off))]
// MCP Agent Context Tools
// PMAT-470: RAG-powered semantic code search for agents
//
// These tools expose the agent context index via MCP protocol,
// enabling AI agents to search code with quality-aware filtering.

use crate::services::agent_context::{AgentContextIndex, QueryOptions};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Why a `pmat_*` tool failed: the caller's input, or ours.
///
/// These tools used to fail with a bare `String`, and the pmcp adapter guessed
/// the origin from a hand-maintained list of message PREFIXES
/// (`CLIENT_FAULT_PREFIXES`). Everything unlisted defaulted to
/// `-32603 Internal error`, so `{limit: 9999}` — a value the schema documents a
/// `maximum: 100` for — came back as `-32603 Internal error: Limit exceeds
/// maximum of 100` and sent the caller debugging the server. JSON-RPC reserves
/// -32602 Invalid params for exactly that case.
///
/// Carrying the variant instead of re-deriving it from prose means a new
/// validation message cannot silently join the internal-error bucket: the
/// author has to choose, and the compiler makes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// A caller-supplied value failed a bound the schema documents.
    /// JSON-RPC `-32602 Invalid params`.
    InvalidParams(String),
    /// A failure of ours: index build, IO, corrupt state.
    /// JSON-RPC `-32603 Internal error`.
    Internal(String),
}

impl ToolError {
    /// Build an `InvalidParams` from anything string-like.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidParams(message.into())
    }

    /// The human-readable message, whichever variant this is.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidParams(m) | Self::Internal(m) => m,
        }
    }

    /// Whether this is the caller's fault (⇒ -32602) rather than ours.
    #[must_use]
    pub fn is_invalid_params(&self) -> bool {
        matches!(self, Self::InvalidParams(_))
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ToolError {}

/// The index layer reports failures as `String`. Those are OURS — a failed
/// build, a corrupt file, an IO error — so `?` on them keeps meaning
/// "internal". Caller mistakes are constructed explicitly via
/// [`ToolError::invalid`], never by this conversion.
impl From<String> for ToolError {
    fn from(message: String) -> Self {
        Self::Internal(message)
    }
}

impl From<&str> for ToolError {
    fn from(message: &str) -> Self {
        Self::Internal(message.to_string())
    }
}

/// MCP Tool trait (same as semantic_search_tools)
#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value, ToolError>;
}

// ============================================================================
// Struct Definitions
// ============================================================================

/// Manages the agent context index lifecycle
pub struct IndexManager {
    index: RwLock<Option<AgentContextIndex>>,
    project_path: PathBuf,
}

/// Search functions by natural language query with quality filtering
pub struct QueryCodeTool {
    manager: Arc<IndexManager>,
}

/// Get details for a specific function by ID
pub struct GetFunctionTool {
    manager: Arc<IndexManager>,
}

/// Find functions similar to a reference function
pub struct FindSimilarTool {
    manager: Arc<IndexManager>,
}

/// Get index statistics and health
pub struct IndexStatsTool {
    manager: Arc<IndexManager>,
}

// ============================================================================
// Argument reading
// ============================================================================
//
// Every optional argument on these tools used to be read as
// `params[key].as_T().unwrap_or(default)`. That collapses THREE different
// caller intents into one: "I said nothing" (use the default), "I said
// something the schema forbids" (`limit: -1`, `limit: 2.5`, `limit: "10"`), and
// "I said something of the wrong type entirely". Only the first deserves the
// default; the other two are the caller's mistake and JSON-RPC reserves
// `-32602 Invalid params` for exactly that.
//
// The upper bounds (`limit > 100`, `min_similarity` outside 0.0..=1.0) were
// already enforced — which is what made the hole hard to see: `{"limit": 9999}`
// was refused while `{"limit": -1}` came back as a perfectly ordinary
// 10-result page. A bound enforced at one end only is not a bound, it is a
// coincidence.
//
// Absent means absent: a missing key — or an explicit `null`, which is how
// every other pmat tool's `#[serde(default)] Option<T>` spells "unset" — is the
// only thing that selects the default.

/// The value the caller supplied for `key`, or `None` if they supplied nothing.
fn supplied<'a>(params: &'a Value, key: &str) -> Option<&'a Value> {
    match params.get(key) {
        None | Some(Value::Null) => None,
        Some(value) => Some(value),
    }
}

/// Read an integer argument bounded by `min..=max`, rejecting anything else.
fn bounded_integer(
    params: &Value,
    key: &str,
    min: u64,
    max: u64,
) -> Result<Option<u64>, ToolError> {
    let Some(value) = supplied(params, key) else {
        return Ok(None);
    };
    value
        .as_u64()
        .filter(|n| (min..=max).contains(n))
        .map(Some)
        .ok_or_else(|| {
            ToolError::invalid(format!(
                "Invalid {key}: {value}. Expected an integer between {min} and {max}"
            ))
        })
}

/// Read a number argument bounded by `min..=max`, rejecting anything else.
fn bounded_number(params: &Value, key: &str, min: f64, max: f64) -> Result<Option<f64>, ToolError> {
    let Some(value) = supplied(params, key) else {
        return Ok(None);
    };
    value
        .as_f64()
        .filter(|n| n.is_finite() && (min..=max).contains(n))
        .map(Some)
        .ok_or_else(|| {
            ToolError::invalid(format!(
                "Invalid {key}: {value}. Expected a number between {min} and {max}"
            ))
        })
}

/// Read a boolean argument, rejecting `"true"`, `1` and friends.
fn boolean(params: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    let Some(value) = supplied(params, key) else {
        return Ok(None);
    };
    value.as_bool().map(Some).ok_or_else(|| {
        ToolError::invalid(format!(
            "Invalid {key}: {value}. Expected a boolean (true or false)"
        ))
    })
}

/// Read a string argument, rejecting a non-string of any kind.
fn string<'a>(params: &'a Value, key: &str) -> Result<Option<&'a str>, ToolError> {
    let Some(value) = supplied(params, key) else {
        return Ok(None);
    };
    value
        .as_str()
        .map(Some)
        .ok_or_else(|| ToolError::invalid(format!("Invalid {key}: {value}. Expected a string")))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse function ID in format "file_path::function_name"
fn parse_function_id(function_id: &str) -> Result<(String, String), ToolError> {
    // Find the last "::" separator
    if let Some(pos) = function_id.rfind("::") {
        let file_path = function_id.get(..pos).unwrap_or_default();
        let function_name = function_id.get(pos + 2..).unwrap_or_default();
        if file_path.is_empty() || function_name.is_empty() {
            return Err(ToolError::invalid(format!(
                "Invalid function_id format. Expected 'file_path::function_name', got: {}",
                function_id
            )));
        }
        Ok((file_path.to_string(), function_name.to_string()))
    } else {
        Err(ToolError::invalid(format!(
            "Invalid function_id format. Expected 'file_path::function_name', got: {}",
            function_id
        )))
    }
}

/// The `min_grade` values this tool accepts, in the order the schema lists them.
///
/// Sourced from [`crate::tdg::Grade::all`] rather than a second hand-written
/// table: the five-letter tables that used to be copied around are exactly what
/// made `A-`/`B+` silently unmatchable (see
/// `services::agent_context::query::grades`).
fn min_grade_enum() -> Vec<String> {
    crate::tdg::Grade::all()
        .iter()
        .map(std::string::ToString::to_string)
        .collect()
}

/// Validate a caller-supplied `min_grade` against the schema's own enum.
///
/// An out-of-enum grade used to sail straight through to the filter, where it
/// matched nothing: `min_grade: "Z"` and `min_grade: ""` both returned
/// `total: 0` — indistinguishable from "your query has no matches". A typo is
/// not an empty result set.
///
/// Matching is deliberately CASE-INSENSITIVE and accepts the `+`/`-` modifier
/// grades, because [`crate::tdg::Grade`]'s own parser (and therefore the CLI's
/// `--min-grade`) does. The schema is written to say the same thing; making the
/// tool stricter than the CLI would have traded a silent-zero for a
/// surface-to-surface contradiction.
fn validate_min_grade(raw: &str) -> Result<String, ToolError> {
    if crate::services::agent_context::query::grades::parse_grade(raw).is_some() {
        return Ok(raw.to_string());
    }
    Err(ToolError::invalid(format!(
        "Invalid min_grade: {raw:?}. Expected one of: {} (case-insensitive)",
        min_grade_enum().join(", ")
    )))
}

// ============================================================================
// Implementations (split into include files)
// ============================================================================

include!("agent_context_tools_index_manager.rs");
include!("agent_context_tools_query_tool.rs");
include!("agent_context_tools_lookup_tools.rs");
include!("agent_context_tools_tests.rs");
