#![cfg_attr(coverage_nightly, coverage(off))]
//! Agent template definitions and implementations.

use super::context::AgentContext;
use super::features::QualityLevel;
use super::generator::{GeneratedFiles, TemplateGenerator};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Available agent templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentTemplate {
    /// Deterministic calculator agent.
    DeterministicCalculator,
    /// State machine workflow agent.
    StateMachineWorkflow,
    /// Hybrid analyzer with deterministic core and probabilistic wrapper.
    HybridAnalyzer,
    /// MCP tool server agent.
    MCPToolServer,
    /// Custom agent template from a path.
    CustomAgent(PathBuf),
}

/// MCP server template generator.
pub struct MCPServerTemplate {
    /// Template name.
    name: String,
    /// Template description.
    description: String,
}

impl Default for MCPServerTemplate {
    fn default() -> Self {
        Self {
            name: "mcp-server".to_string(),
            description: "MCP tool server with async handlers and resource management".to_string(),
        }
    }
}

/// State machine template generator.
pub struct StateMachineTemplate {
    /// Template name.
    name: String,
    /// Template description.
    description: String,
}

impl Default for StateMachineTemplate {
    fn default() -> Self {
        Self {
            name: "state-machine".to_string(),
            description: "State machine agent with transitions and invariants".to_string(),
        }
    }
}

// MCP server template generation: Cargo.toml, main.rs, server, tools, transport,
// agent core/handlers, quality module, tests, config, and README generators.
include!("templates_mcp_generation.rs");

// State machine template generation: Cargo.toml, main.rs, state definitions,
// transitions, invariants, and test generators.
include!("templates_state_machine_generation.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::agent::generator::FileContent;

    fn mk_ctx(name: &str, quality: QualityLevel) -> AgentContext {
        AgentContext {
            name: name.to_string(),
            template_type: AgentTemplate::MCPToolServer,
            features: Default::default(),
            quality_level: quality,
            deterministic_core: None,
            probabilistic_wrapper: None,
        }
    }

    fn file_contents<'a>(files: &'a GeneratedFiles, path: &str) -> &'a str {
        files
            .files
            .get(&PathBuf::from(path))
            .expect("file exists")
            .as_str()
            .expect("text content")
    }

    // --- MCPServerTemplate ---

    #[test]
    fn test_mcp_template_default_fields() {
        let t = MCPServerTemplate::default();
        assert_eq!(t.name(), "mcp-server");
        assert!(
            t.description().contains("MCP tool server"),
            "got {:?}",
            t.description()
        );
    }

    #[test]
    fn test_mcp_template_generate_standard_quality_skips_quality_files() {
        // QualityLevel::Standard → if-branch in templates_mcp_generation.rs:28 is FALSE
        // → NO quality/ files emitted + main.rs omits `mod quality;`.
        let t = MCPServerTemplate::default();
        let files = t
            .generate(&mk_ctx("test_agent", QualityLevel::Standard))
            .unwrap();

        let expected = [
            "Cargo.toml",
            "src/main.rs",
            "src/mcp/mod.rs",
            "src/mcp/server.rs",
            "src/mcp/tools.rs",
            "src/mcp/transport.rs",
            "src/agent/mod.rs",
            "src/agent/core.rs",
            "src/agent/handlers.rs",
            "tests/integration.rs",
            "tests/deterministic.rs",
            ".pmat/agent.toml",
            ".pmat/quality-gates.toml",
            "README.md",
        ];
        for f in &expected {
            assert!(files.contains_file(&PathBuf::from(f)), "missing {f}");
        }
        assert!(!files.contains_file(&PathBuf::from("src/quality/mod.rs")));
        assert!(!files.contains_file(&PathBuf::from("src/quality/invariants.rs")));
        assert!(!files.contains_file(&PathBuf::from("src/quality/validators.rs")));

        let main_rs = file_contents(&files, "src/main.rs");
        assert!(
            !main_rs.contains("mod quality;"),
            "Standard must omit `mod quality;`, got:\n{main_rs}"
        );
    }

    #[test]
    fn test_mcp_template_generate_strict_quality_includes_quality_files() {
        let t = MCPServerTemplate::default();
        let files = t
            .generate(&mk_ctx("test_agent", QualityLevel::Strict))
            .unwrap();
        assert!(files.contains_file(&PathBuf::from("src/quality/mod.rs")));
        assert!(files.contains_file(&PathBuf::from("src/quality/invariants.rs")));
        assert!(files.contains_file(&PathBuf::from("src/quality/validators.rs")));

        let main_rs = file_contents(&files, "src/main.rs");
        assert!(
            main_rs.contains("mod quality;"),
            "non-Standard must include `mod quality;`"
        );
    }

    #[test]
    fn test_mcp_template_generate_extreme_quality_includes_quality_files() {
        let t = MCPServerTemplate::default();
        let files = t
            .generate(&mk_ctx("extreme_agent", QualityLevel::Extreme))
            .unwrap();
        assert!(files.contains_file(&PathBuf::from("src/quality/mod.rs")));

        let main_rs = file_contents(&files, "src/main.rs");
        assert!(main_rs.contains("mod quality;"));
    }

    #[test]
    fn test_mcp_template_context_name_flows_into_generated_files() {
        let t = MCPServerTemplate::default();
        let files = t
            .generate(&mk_ctx("my_unique_agent_xyz", QualityLevel::Standard))
            .unwrap();

        assert!(file_contents(&files, "Cargo.toml").contains("my_unique_agent_xyz"));
        assert!(file_contents(&files, "src/main.rs").contains("my_unique_agent_xyz"));
        assert!(file_contents(&files, "src/mcp/server.rs").contains("my_unique_agent_xyz"));
        assert!(file_contents(&files, "src/agent/core.rs").contains("my_unique_agent_xyz"));
        assert!(file_contents(&files, ".pmat/agent.toml").contains("my_unique_agent_xyz"));
        assert!(file_contents(&files, "README.md").contains("my_unique_agent_xyz"));
    }

    #[test]
    fn test_mcp_template_readme_reflects_quality_level() {
        // The generate_readme match-arms (Standard/Strict/Extreme) at the bottom of
        // templates_mcp_generation.rs must flow through into the README body.
        let t = MCPServerTemplate::default();
        let stdr = t.generate(&mk_ctx("a", QualityLevel::Standard)).unwrap();
        let strict = t.generate(&mk_ctx("a", QualityLevel::Strict)).unwrap();
        let ext = t.generate(&mk_ctx("a", QualityLevel::Extreme)).unwrap();
        assert!(file_contents(&stdr, "README.md").contains("standard"));
        assert!(file_contents(&strict, "README.md").contains("strict"));
        assert!(file_contents(&ext, "README.md").contains("Toyota Way extreme"));
    }

    #[test]
    fn test_mcp_template_validate_context_empty_name_errors() {
        let t = MCPServerTemplate::default();
        let ctx = mk_ctx("", QualityLevel::Standard);
        let err = t.validate_context(&ctx).expect_err("empty name must error");
        assert!(
            err.to_string().contains("name"),
            "error should mention name, got: {err}"
        );
    }

    #[test]
    fn test_mcp_template_validate_context_non_empty_name_ok() {
        let t = MCPServerTemplate::default();
        let ctx = mk_ctx("ok_name", QualityLevel::Standard);
        assert!(t.validate_context(&ctx).is_ok());
    }

    #[test]
    fn test_mcp_template_generated_files_are_text_content() {
        let t = MCPServerTemplate::default();
        let files = t.generate(&mk_ctx("x", QualityLevel::Standard)).unwrap();
        for (path, content) in &files.files {
            assert!(
                matches!(content, FileContent::Text(_)),
                "expected Text content for {path:?}, got {content:?}"
            );
        }
    }

    #[test]
    fn test_mcp_template_cargo_toml_has_pmcp_dep() {
        // Pin a key dependency so mutations on the Cargo.toml generator are caught.
        let t = MCPServerTemplate::default();
        let files = t.generate(&mk_ctx("x", QualityLevel::Standard)).unwrap();
        let cargo = file_contents(&files, "Cargo.toml");
        assert!(cargo.contains("pmcp ="), "Cargo.toml missing pmcp dep");
        assert!(cargo.contains("tokio"));
        assert!(cargo.contains("proptest"));
    }

    #[test]
    fn test_mcp_template_quality_gates_reflect_level_numbers() {
        // Strict and Extreme have different max_complexity / min_line_coverage values;
        // assert the Cargo-toml-ish output reflects that (not identical).
        let t = MCPServerTemplate::default();
        let strict = t.generate(&mk_ctx("x", QualityLevel::Strict)).unwrap();
        let ext = t.generate(&mk_ctx("x", QualityLevel::Extreme)).unwrap();
        assert_ne!(
            file_contents(&strict, ".pmat/quality-gates.toml"),
            file_contents(&ext, ".pmat/quality-gates.toml"),
            "Strict and Extreme must produce different quality-gates.toml"
        );
    }

    // --- StateMachineTemplate ---

    #[test]
    fn test_state_machine_template_default_fields() {
        let t = StateMachineTemplate::default();
        assert_eq!(t.name(), "state-machine");
        assert!(t.description().contains("State machine"));
    }

    fn mk_sm_ctx(name: &str) -> AgentContext {
        AgentContext {
            name: name.to_string(),
            template_type: AgentTemplate::StateMachineWorkflow,
            features: Default::default(),
            quality_level: QualityLevel::Standard,
            deterministic_core: None,
            probabilistic_wrapper: None,
        }
    }

    #[test]
    fn test_state_machine_template_generate_all_files() {
        let t = StateMachineTemplate::default();
        let files = t.generate(&mk_sm_ctx("test_sm")).unwrap();
        let expected = [
            "Cargo.toml",
            "src/main.rs",
            "src/agent/mod.rs",
            "src/agent/state.rs",
            "src/agent/transitions.rs",
            "src/agent/invariants.rs",
            "tests/state_transitions.rs",
            "tests/invariants.rs",
        ];
        for f in &expected {
            assert!(files.contains_file(&PathBuf::from(f)), "missing {f}");
        }
        assert_eq!(
            files.file_count(),
            expected.len(),
            "state machine template must produce exactly these files"
        );
    }

    #[test]
    fn test_state_machine_template_context_name_flows() {
        let t = StateMachineTemplate::default();
        let files = t.generate(&mk_sm_ctx("xform_fsm")).unwrap();
        assert!(file_contents(&files, "Cargo.toml").contains("xform_fsm"));
        assert!(file_contents(&files, "src/main.rs").contains("xform_fsm"));
        assert!(file_contents(&files, "src/agent/state.rs").contains("xform_fsm"));
    }

    #[test]
    fn test_state_machine_template_state_enum_variants_present() {
        // generate_state_definitions hardcodes enum variants — kill mutations that
        // would rename/remove them.
        let t = StateMachineTemplate::default();
        let files = t.generate(&mk_sm_ctx("x")).unwrap();
        let state_rs = file_contents(&files, "src/agent/state.rs");
        for variant in &["Initial", "Processing", "Complete", "Error(String)"] {
            assert!(state_rs.contains(variant), "missing variant {variant}");
        }
        for event in &["Start", "Process", "Finish", "Fail(String)"] {
            assert!(state_rs.contains(event), "missing event {event}");
        }
    }

    #[test]
    fn test_state_machine_template_validate_context_empty_name_errors() {
        let t = StateMachineTemplate::default();
        let err = t
            .validate_context(&mk_sm_ctx(""))
            .expect_err("empty name must error");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_state_machine_template_validate_context_non_empty_ok() {
        let t = StateMachineTemplate::default();
        assert!(t.validate_context(&mk_sm_ctx("ok")).is_ok());
    }

    // --- AgentTemplate enum round-trip (serde coverage) ---

    #[test]
    fn test_agent_template_enum_serde_round_trip() {
        // Each variant must survive JSON round-trip. Kills any mutation that breaks
        // the derived Serialize/Deserialize on the enum.
        let variants = vec![
            AgentTemplate::DeterministicCalculator,
            AgentTemplate::StateMachineWorkflow,
            AgentTemplate::HybridAnalyzer,
            AgentTemplate::MCPToolServer,
            AgentTemplate::CustomAgent(PathBuf::from("/tmp/custom")),
        ];
        for v in variants {
            let json = serde_json::to_string(&v).expect("serialize");
            let _: AgentTemplate = serde_json::from_str(&json).expect("round-trip");
        }
    }
}
