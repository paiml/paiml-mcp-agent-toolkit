#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::mcp::{McpRequest, McpResponse};
use crate::TemplateServerTrait;
use serde_json::json;
use std::sync::Arc;

pub async fn handle_initialize<T: TemplateServerTrait>(
    _server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    // Extract protocol version from params if provided
    let protocol_version = request
        .params
        .as_ref()
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or("2024-11-05");

    // Return initialization response with server info
    McpResponse::success(
        request.id,
        json!({
            "protocolVersion": protocol_version,
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {},
            },
            "serverInfo": {
                "name": "pmat",
                "version": env!("CARGO_PKG_VERSION"),
                "vendor": "Pragmatic AI Labs (paiml.com)",
                "author": "Pragmatic AI Labs",
                "description": "Professional project scaffolding toolkit that generates Makefiles, README.md files, and .gitignore files for Rust, Deno, and Python projects. Created by Pragmatic AI Labs to streamline project setup with best practices.",
                "capabilities": [
                    "Generate individual project files (Makefile, README.md, .gitignore)",
                    "Scaffold complete projects with all files at once",
                    "Support for Rust CLI/library projects",
                    "Support for Deno/TypeScript applications",
                    "Support for Python UV projects",
                    "Smart subdirectory creation for organized project structure"
                ],
                "supportedTemplates": ["makefile", "readme", "gitignore"],
                "supportedToolchains": ["rust", "deno", "python-uv"],
                "examples": [
                    "Create a new Rust CLI project: scaffold_project with toolchain='rust'",
                    "Generate just a Makefile: generate_template with resource_uri='template://makefile/rust/cli'",
                    "Search for Python templates: search_templates with query='python'"
                ]
            }
        }),
    )
}

// Core scaffolding tool definitions (get_server_info, generate_template, etc.)
include!("initialize_tools_core.rs");

// Standard analysis tool definitions (churn, complexity, DAG, context, dead code, deep context)
include!("initialize_tools_standard.rs");

// Vectorized SIMD-accelerated tool definitions
include!("initialize_tools_vectorized.rs");

// Reporting, SATD, and lint hotspot tool definitions
include!("initialize_tools_reporting.rs");

// Assembler function that combines all analysis tools
include!("initialize_tools_analysis.rs");

pub async fn handle_tools_list<T: TemplateServerTrait>(
    _server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    // Build the tools list from core and analysis definitions
    let mut tools = core_tool_definitions();
    tools.extend(analysis_tool_definitions());

    McpResponse::success(
        request.id,
        json!({
            "tools": tools
        }),
    )
}

// Tests for initialize and tools_list handlers
include!("initialize_tests.rs");
