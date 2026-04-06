//! JSON Schema definitions for MCP tool parameters

use super::handler::SimpleMcpHandler;
use serde_json::{json, Value};

impl SimpleMcpHandler {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_complexity_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "max_cyclomatic": {"type": "integer"},
                "max_cognitive": {"type": "integer"},
                "max_halstead": {"type": "number"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_satd_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "severity": {"type": "string", "enum": ["low", "medium", "high", "critical"]},
                "critical_only": {"type": "boolean"},
                "strict": {"type": "boolean"},
                "fail_on_violation": {"type": "boolean"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_dead_code_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "include_unreachable": {"type": "boolean"},
                "min_dead_lines": {"type": "integer"},
                "max_percentage": {"type": "number"},
                "fail_on_violation": {"type": "boolean"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_tdg_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "threshold": {"type": "number"},
                "include_components": {"type": "boolean"},
                "critical_only": {"type": "boolean"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_lint_hotspot_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "file": {"type": "string"},
                "max_density": {"type": "number"},
                "min_confidence": {"type": "number"},
                "enforce": {"type": "boolean"},
                "dry_run": {"type": "boolean"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_quality_gate_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "profile": {"type": "string", "enum": ["standard", "strict", "extreme", "toyota"]},
                "file": {"type": "string"},
                "fail_on_violation": {"type": "boolean"},
                "verbose": {"type": "boolean"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_entropy_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "top_files": {"type": "integer"},
                "include_tests": {"type": "boolean"},
                "timeout": {"type": "integer"},
                "min_severity": {"type": "string", "enum": ["low", "medium", "high"]},
                "top_violations": {"type": "integer"},
                "file": {"type": "string"}
            },
            "required": ["path"]
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_refactor_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file": {"type": "string"},
                "format": {"type": "string", "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]},
                "output": {"type": "string"},
                "target_complexity": {"type": "integer"},
                "dry_run": {"type": "boolean"},
                "timeout": {"type": "integer"}
            },
            "required": ["file"]
        })
    }
}
