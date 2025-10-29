// DAP (Debug Adapter Protocol) types
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
//
// Types for Debug Adapter Protocol communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DAP Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapRequest {
    pub seq: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub command: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// DAP Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapResponse {
    pub seq: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub request_seq: i64,
    pub success: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl DapResponse {
    /// Create a successful response
    pub fn success(request_seq: i64, seq: i64, command: String, body: Option<serde_json::Value>) -> Self {
        Self {
            seq,
            type_field: "response".to_string(),
            request_seq,
            success: true,
            command,
            message: None,
            body,
        }
    }

    /// Create an error response
    pub fn error(request_seq: i64, seq: i64, command: String, message: String) -> Self {
        Self {
            seq,
            type_field: "response".to_string(),
            request_seq,
            success: false,
            command,
            message: Some(message),
            body: None,
        }
    }
}

/// DAP Event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapEvent {
    pub seq: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// DAP Capabilities structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DapCapabilities {
    pub supports_configuration_done_request: bool,
    pub supports_function_breakpoints: bool,
    pub supports_conditional_breakpoints: bool,
    pub supports_hit_conditional_breakpoints: bool,
    pub supports_evaluate_for_hovers: bool,
    pub supports_step_back: bool,
    pub supports_set_variable: bool,
    pub supports_restart_frame: bool,
    pub supports_goto_targets_request: bool,
    pub supports_step_in_targets_request: bool,
    pub supports_completions_request: bool,
    pub supports_modules_request: bool,
    pub supports_restart_request: bool,
    pub supports_exception_options: bool,
    pub supports_value_formatting_options: bool,
    pub supports_exception_info_request: bool,
    pub supports_terminate_debuggee: bool,
    pub supports_delayed_stack_trace_loading: bool,
    pub supports_loaded_sources_request: bool,
    pub supports_log_points: bool,
    pub supports_terminate_threads_request: bool,
    pub supports_set_expression: bool,
    pub supports_terminate_request: bool,
    pub supports_data_breakpoints: bool,
    pub supports_read_memory_request: bool,
    pub supports_write_memory_request: bool,
    pub supports_disassemble_request: bool,
    pub supports_cancel_request: bool,
    pub supports_breakpoint_locations_request: bool,
    pub supports_clipboard_context: bool,
    pub supports_stepping_granularity: bool,
    pub supports_instruction_breakpoints: bool,
    pub supports_exception_filter_options: bool,
}

/// Breakpoint structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    pub source: String,
    pub line: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Variable structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    pub type_info: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables_reference: Option<i64>,
}

/// Stack frame structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: i64,
    pub name: String,
    pub source: Option<Source>,
    pub line: i64,
    pub column: i64,
}

/// Source structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: Option<String>,
    pub path: Option<String>,
}

/// Scope structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub variables_reference: i64,
    pub expensive: bool,
}

/// Thread structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: i64,
    pub name: String,
}

/// Initialize request arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestArguments {
    pub client_id: Option<String>,
    pub adapter_id: String,
    pub lines_start_at1: Option<bool>,
    pub columns_start_at1: Option<bool>,
    pub path_format: Option<String>,
    pub supports_variable_type: Option<bool>,
    pub supports_variable_paging: Option<bool>,
}

/// Launch request arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestArguments {
    pub program: String,
    pub stop_on_entry: Option<bool>,
    pub cwd: Option<String>,
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// SetBreakpoints request arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBreakpointsArguments {
    pub source: Source,
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
}

/// Source breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakpoint {
    pub line: i64,
    pub column: Option<i64>,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
    pub log_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dap_response_success_serialization() {
        let response = DapResponse::success(
            1,
            2,
            "initialize".to_string(),
            Some(serde_json::json!({"test": "value"})),
        );

        assert_eq!(response.type_field, "response");
        assert_eq!(response.success, true);
        assert_eq!(response.command, "initialize");
        assert!(response.message.is_none());
        assert!(response.body.is_some());
    }

    #[test]
    fn test_dap_response_error_serialization() {
        let response = DapResponse::error(
            1,
            2,
            "unknown".to_string(),
            "Command not supported".to_string(),
        );

        assert_eq!(response.type_field, "response");
        assert_eq!(response.success, false);
        assert_eq!(response.command, "unknown");
        assert_eq!(response.message, Some("Command not supported".to_string()));
        assert!(response.body.is_none());
    }

    #[test]
    fn test_breakpoint_equality() {
        let bp1 = Breakpoint {
            source: "main.rs".to_string(),
            line: 10,
            column: None,
            condition: None,
        };

        let bp2 = Breakpoint {
            source: "main.rs".to_string(),
            line: 10,
            column: None,
            condition: None,
        };

        assert_eq!(bp1, bp2);
    }

    #[test]
    fn test_dap_capabilities_default() {
        let caps = DapCapabilities::default();
        assert!(!caps.supports_configuration_done_request);
        assert!(!caps.supports_conditional_breakpoints);
    }
}
