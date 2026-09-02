use crate::mcp_pmcp::tool_schemas::build_tool_info;
use crate::models::proxy::{ProxyMode, ProxyOperation, ProxyRequest, QualityConfig};
use crate::services::quality_proxy::QualityProxyService;
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info};

/// Input parameters for `quality_check_content` (and its one-release alias
/// `quality_proxy`).
///
/// No `operation`: the tool grades `content` for `file_path` and returns it;
/// it never writes, edits or appends (CRUX-10, #1151 — the write it advertised
/// had never existed, and every caller was writing with its own harness while
/// believing the gate had). `deny_unknown_fields` is what turns a stale
/// `operation: "write"` into `-32602` instead of a silently ignored key.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityProxyInput {
    pub file_path: String,
    pub content: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub quality_config: QualityConfigInput,
}

fn default_mode() -> String {
    "strict".to_string()
}

#[derive(Debug, Deserialize)]
/// Quality config input.
pub struct QualityConfigInput {
    #[serde(default = "default_max_complexity")]
    pub max_complexity: u32,
    #[serde(default = "default_allow_satd")]
    pub allow_satd: bool,
    #[serde(default = "default_require_docs")]
    pub require_docs: bool,
    #[serde(default = "default_auto_format")]
    pub auto_format: bool,
}

impl Default for QualityConfigInput {
    fn default() -> Self {
        Self {
            max_complexity: default_max_complexity(),
            allow_satd: default_allow_satd(),
            require_docs: default_require_docs(),
            auto_format: default_auto_format(),
        }
    }
}

fn default_max_complexity() -> u32 {
    20
}

fn default_allow_satd() -> bool {
    false
}

fn default_require_docs() -> bool {
    true
}

fn default_auto_format() -> bool {
    true
}

/// Tool handler for proxying code changes through quality gates.
///
/// This handler intercepts code operations from AI agents and ensures
/// they meet quality standards before being applied. It supports three
/// enforcement modes: strict (reject low-quality code), advisory (warn
/// but allow), and auto-fix (automatically refactor to meet standards).
///
/// # Example
///
/// ```ignore
/// use pmat::mcp_pmcp::quality_proxy_handler::QualityProxyTool;
/// use pmcp::ToolHandler;
///
/// let tool = QualityProxyTool;
/// assert_eq!(tool.name(), "quality_proxy");
/// ```ignore
///
/// # Strict Mode Example
///
/// ```ignore
/// use pmat::mcp_pmcp::quality_proxy_handler::{QualityProxyTool, QualityProxyInput};
/// use pmcp::ToolHandler;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tool = QualityProxyTool;
///
/// // Intercept a file write operation
/// let input = json!({
///     "operation": "write",
///     "file_path": "src/new_feature.rs",
///     "content": "fn complex_function() { /* code */ }",
///     "mode": "strict",
///     "quality_config": {
///         "max_complexity": 10,
///         "allow_satd": false,
///         "require_docs": true
///     }
/// });
///
/// // Quality proxy validates before allowing write
/// let result = tool.handle(input, Default::default()).await?;
///
/// // Check if operation was approved
/// assert!(result["approved"].is_boolean());
/// if !result["approved"].as_bool().unwrap_or(false) {
///     println!("Quality violations: {}", result["violations"]);
/// }
/// # Ok(())
/// # }
/// ```ignore
///
/// # Auto-Fix Mode Example
///
/// ```ignore
/// use pmat::mcp_pmcp::quality_proxy_handler::QualityProxyTool;
/// use pmcp::ToolHandler;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tool = QualityProxyTool;
///
/// // Enable auto-fix mode
/// let input = json!({
///     "operation": "edit",
///     "file_path": "src/messy_code.rs",
///     "old_content": "fn f(){if(true){return 1;}return 0;}",
///     "new_content": "fn f(){if(true){return 1;}return 0;}//Needs refactoring",
///     "mode": "auto_fix",
///     "quality_config": {
///         "max_complexity": 5,
///         "allow_satd": false,
///         "auto_format": true
///     }
/// });
///
/// // Quality proxy automatically improves the code
/// let result = tool.handle(input, Default::default()).await?;
///
/// // Get the improved version
/// if result["auto_fixed"].as_bool().unwrap_or(false) {
///     let fixed_content = result["fixed_content"].as_str().unwrap();
///     println!("Auto-fixed code: {}", fixed_content);
/// }
/// # Ok(())
/// # }
/// ```ignore
///
/// # Quality Configuration
///
/// ```ignore
/// use pmat::mcp_pmcp::quality_proxy_handler::QualityConfigInput;
///
/// // Strict configuration for production
/// let strict_config = QualityConfigInput {
///     max_complexity: 8,
///     allow_satd: false,
///     require_docs: true,
///     auto_format: true,
/// };
///
/// // Relaxed configuration for prototyping
/// let relaxed_config = QualityConfigInput {
///     max_complexity: 20,
///     allow_satd: false, // Still no TODOs!
///     require_docs: false,
///     auto_format: false,
/// };
/// ```ignore
pub struct QualityProxyTool;

/// The deprecated name, served for exactly one release after the rename.
/// Same schema, same handler, same payload — `quality_proxy` never wrote a
/// file, so the alias asserts nothing the new name does not.
pub struct QualityProxyAliasTool;

/// The live tool's name.
pub const QUALITY_CHECK_CONTENT: &str = "quality_check_content";
/// The name it replaced, kept as an alias for one release.
pub const QUALITY_PROXY_ALIAS: &str = "quality_proxy";

const QUALITY_CHECK_DESCRIPTION: &str = "Grade proposed file content against the project's quality gate (complexity, SATD, docs, lint) and return it with a verdict. Never touches the filesystem: hand the returned content to your own file tool, or let the harness PreToolUse hook gate the change.";

include!("quality_proxy_handler_impl.rs");
include!("quality_proxy_handler_tests.rs");
