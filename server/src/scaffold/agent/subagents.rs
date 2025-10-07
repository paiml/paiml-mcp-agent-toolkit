//! Claude Code sub-agent templates for PMAT.
//!
//! This module provides specialized sub-agent templates compatible with Claude Code's
//! sub-agent system. Each sub-agent is an AI specialist focused on a specific aspect
//! of code quality analysis.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Available PMAT sub-agents for Claude Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PmatSubAgent {
    /// Complexity analysis specialist (cyclomatic & cognitive complexity).
    ComplexityAnalyst,
    /// Mutation testing expert with ML prediction.
    MutationTester,
    /// Technical debt detector (SATD - TODO, FIXME, HACK comments).
    SATDDetector,
    /// Unused code elimination specialist.
    DeadCodeEliminator,
    /// Generic description detector for documentation enforcement.
    DocumentationEnforcer,

    // Future phases (not implemented in MVP)
    /// Rust-specific quality expert.
    RustQualityExpert,
    /// Python quality expert.
    PythonQualityExpert,
    /// TypeScript/JavaScript quality expert.
    TypeScriptQualityExpert,
    /// WASM deep inspector (bytecode analysis).
    WasmDeepInspector,
    /// Refactoring advisor using pattern learning.
    RefactoringAdvisor,
    /// Test coverage analyst.
    TestCoverageAnalyst,
    /// Quality gate orchestrator.
    QualityGateOrchestrator,
}

impl PmatSubAgent {
    /// Get the display name of the sub-agent.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ComplexityAnalyst => "complexity-analyst",
            Self::MutationTester => "mutation-tester",
            Self::SATDDetector => "satd-detector",
            Self::DeadCodeEliminator => "dead-code-eliminator",
            Self::DocumentationEnforcer => "documentation-enforcer",
            Self::RustQualityExpert => "rust-quality-expert",
            Self::PythonQualityExpert => "python-quality-expert",
            Self::TypeScriptQualityExpert => "typescript-quality-expert",
            Self::WasmDeepInspector => "wasm-deep-inspector",
            Self::RefactoringAdvisor => "refactoring-advisor",
            Self::TestCoverageAnalyst => "test-coverage-analyst",
            Self::QualityGateOrchestrator => "quality-gate-orchestrator",
        }
    }

    /// Get a brief description of the sub-agent.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ComplexityAnalyst =>
                "Expert in cyclomatic and cognitive complexity analysis, suggests refactorings",
            Self::MutationTester =>
                "Mutation testing specialist with ML prediction and test improvement suggestions",
            Self::SATDDetector =>
                "Technical debt identifier tracking TODO, FIXME, and HACK comments",
            Self::DeadCodeEliminator =>
                "Unused code removal specialist identifying safe-to-delete code",
            Self::DocumentationEnforcer =>
                "Generic description detector enforcing documentation quality standards",
            Self::RustQualityExpert =>
                "Rust-specific quality expert covering ownership, lifetimes, and idiomatic patterns",
            Self::PythonQualityExpert =>
                "Python quality expert for type hints, PEP compliance, and best practices",
            Self::TypeScriptQualityExpert =>
                "TypeScript/JavaScript quality expert for type safety and modern patterns",
            Self::WasmDeepInspector =>
                "WebAssembly bytecode analyst for compiler debugging and optimization",
            Self::RefactoringAdvisor =>
                "AI-powered refactoring advisor using historical pattern learning",
            Self::TestCoverageAnalyst =>
                "Test coverage gap identifier suggesting missing test cases",
            Self::QualityGateOrchestrator =>
                "Coordinates multiple quality checks and synthesizes results",
        }
    }

    /// Check if this sub-agent is implemented in the MVP.
    pub fn is_mvp(&self) -> bool {
        matches!(
            self,
            Self::ComplexityAnalyst
                | Self::MutationTester
                | Self::SATDDetector
                | Self::DeadCodeEliminator
                | Self::DocumentationEnforcer
        )
    }

    /// Get the primary MCP tools used by this sub-agent.
    pub fn primary_tools(&self) -> Vec<&'static str> {
        match self {
            Self::ComplexityAnalyst => vec!["analyze_complexity", "analyze_cognitive_complexity"],
            Self::MutationTester => vec!["mutation_test", "mutation_predict", "equivalent_detector"],
            Self::SATDDetector => vec!["analyze_satd", "analyze_context"],
            Self::DeadCodeEliminator => vec!["analyze_dead_code", "analyze_imports"],
            Self::DocumentationEnforcer => vec!["check_generic_docs", "analyze_context"],
            Self::RustQualityExpert => vec!["analyze_complexity", "analyze_borrow_checker"],
            Self::PythonQualityExpert => vec!["analyze_complexity", "analyze_context"],
            Self::TypeScriptQualityExpert => vec!["analyze_complexity", "analyze_context"],
            Self::WasmDeepInspector => vec!["deep_wasm_analyze", "wasm_disassemble"],
            Self::RefactoringAdvisor => vec!["suggest_refactorings", "query_patterns"],
            Self::TestCoverageAnalyst => vec!["analyze_coverage", "suggest_tests"],
            Self::QualityGateOrchestrator => vec!["run_quality_gates", "aggregate_metrics"],
        }
    }

    /// Get all sub-agents (MVP only by default).
    pub fn all_mvp() -> Vec<Self> {
        vec![
            Self::ComplexityAnalyst,
            Self::MutationTester,
            Self::SATDDetector,
            Self::DeadCodeEliminator,
            Self::DocumentationEnforcer,
        ]
    }

    /// Get all sub-agents (including future phases).
    pub fn all() -> Vec<Self> {
        vec![
            Self::ComplexityAnalyst,
            Self::MutationTester,
            Self::SATDDetector,
            Self::DeadCodeEliminator,
            Self::DocumentationEnforcer,
            Self::RustQualityExpert,
            Self::PythonQualityExpert,
            Self::TypeScriptQualityExpert,
            Self::WasmDeepInspector,
            Self::RefactoringAdvisor,
            Self::TestCoverageAnalyst,
            Self::QualityGateOrchestrator,
        ]
    }
}

impl std::str::FromStr for PmatSubAgent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "complexity-analyst" => Ok(Self::ComplexityAnalyst),
            "mutation-tester" => Ok(Self::MutationTester),
            "satd-detector" => Ok(Self::SATDDetector),
            "dead-code-eliminator" => Ok(Self::DeadCodeEliminator),
            "documentation-enforcer" => Ok(Self::DocumentationEnforcer),
            "rust-quality-expert" => Ok(Self::RustQualityExpert),
            "python-quality-expert" => Ok(Self::PythonQualityExpert),
            "typescript-quality-expert" => Ok(Self::TypeScriptQualityExpert),
            "wasm-deep-inspector" => Ok(Self::WasmDeepInspector),
            "refactoring-advisor" => Ok(Self::RefactoringAdvisor),
            "test-coverage-analyst" => Ok(Self::TestCoverageAnalyst),
            "quality-gate-orchestrator" => Ok(Self::QualityGateOrchestrator),
            _ => bail!("Unknown sub-agent: {}", s),
        }
    }
}

impl std::fmt::Display for PmatSubAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Sub-agent generator for Claude Code.
pub struct SubAgentGenerator {
    template_dir: PathBuf,
}

impl SubAgentGenerator {
    /// Create a new sub-agent generator.
    pub fn new() -> Self {
        Self {
            template_dir: PathBuf::from("server/src/scaffold/agent/subagent_templates"),
        }
    }

    /// Create a generator with custom template directory.
    pub fn with_template_dir(template_dir: PathBuf) -> Self {
        Self { template_dir }
    }

    /// Generate a sub-agent definition in markdown format.
    pub fn generate_subagent(&self, agent: PmatSubAgent) -> Result<String> {
        if !agent.is_mvp() {
            bail!(
                "Sub-agent {} is not yet implemented (future phase)",
                agent.name()
            );
        }

        match agent {
            PmatSubAgent::ComplexityAnalyst => self.generate_complexity_analyst(),
            PmatSubAgent::MutationTester => self.generate_mutation_tester(),
            PmatSubAgent::SATDDetector => self.generate_satd_detector(),
            PmatSubAgent::DeadCodeEliminator => self.generate_dead_code_eliminator(),
            PmatSubAgent::DocumentationEnforcer => self.generate_documentation_enforcer(),
            _ => bail!("Sub-agent {} not implemented in MVP", agent.name()),
        }
    }

    /// Export sub-agent to a file compatible with Claude Code.
    pub fn export_for_claude_code(
        &self,
        agent: PmatSubAgent,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        let content = self.generate_subagent(agent)?;
        let filename = format!("{}.md", agent.name());
        let output_path = output_dir.join(filename);

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)?;

        // Write the sub-agent definition
        std::fs::write(&output_path, content)?;

        Ok(output_path)
    }

    /// Export all MVP sub-agents.
    pub fn export_all_mvp(&self, output_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for agent in PmatSubAgent::all_mvp() {
            let path = self.export_for_claude_code(agent, output_dir)?;
            paths.push(path);
        }
        Ok(paths)
    }

    /// Get MCP tool mapping for all sub-agents.
    pub fn get_tool_mapping() -> HashMap<PmatSubAgent, Vec<&'static str>> {
        let mut mapping = HashMap::new();
        for agent in PmatSubAgent::all() {
            mapping.insert(agent, agent.primary_tools());
        }
        mapping
    }

    // Template generation methods (to be implemented)
    fn generate_complexity_analyst(&self) -> Result<String> {
        Ok(include_str!("subagent_templates/complexity_analyst.md.tmpl").to_string())
    }

    fn generate_mutation_tester(&self) -> Result<String> {
        Ok(include_str!("subagent_templates/mutation_tester.md.tmpl").to_string())
    }

    fn generate_satd_detector(&self) -> Result<String> {
        Ok(include_str!("subagent_templates/satd_detector.md.tmpl").to_string())
    }

    fn generate_dead_code_eliminator(&self) -> Result<String> {
        Ok(include_str!("subagent_templates/dead_code_eliminator.md.tmpl").to_string())
    }

    fn generate_documentation_enforcer(&self) -> Result<String> {
        Ok(include_str!("subagent_templates/documentation_enforcer.md.tmpl").to_string())
    }
}

impl Default for SubAgentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_name() {
        assert_eq!(PmatSubAgent::ComplexityAnalyst.name(), "complexity-analyst");
        assert_eq!(PmatSubAgent::MutationTester.name(), "mutation-tester");
    }

    #[test]
    fn test_subagent_from_str() {
        let agent: PmatSubAgent = "complexity-analyst".parse().unwrap();
        assert_eq!(agent, PmatSubAgent::ComplexityAnalyst);

        let agent: PmatSubAgent = "mutation-tester".parse().unwrap();
        assert_eq!(agent, PmatSubAgent::MutationTester);
    }

    #[test]
    fn test_subagent_from_str_invalid() {
        let result: Result<PmatSubAgent> = "invalid-agent".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_all_mvp_agents() {
        let agents = PmatSubAgent::all_mvp();
        assert_eq!(agents.len(), 5);
        assert!(agents.iter().all(|a| a.is_mvp()));
    }

    #[test]
    fn test_primary_tools() {
        let tools = PmatSubAgent::ComplexityAnalyst.primary_tools();
        assert!(tools.contains(&"analyze_complexity"));

        let tools = PmatSubAgent::MutationTester.primary_tools();
        assert!(tools.contains(&"mutation_test"));
        assert!(tools.contains(&"mutation_predict"));
    }

    #[test]
    fn test_tool_mapping() {
        let mapping = SubAgentGenerator::get_tool_mapping();
        assert!(!mapping.is_empty());

        let complexity_tools = mapping.get(&PmatSubAgent::ComplexityAnalyst).unwrap();
        assert!(complexity_tools.contains(&"analyze_complexity"));
    }

    #[test]
    fn test_generator_creation() {
        let gen = SubAgentGenerator::new();
        assert!(gen.template_dir.to_string_lossy().contains("subagent_templates"));
    }

    #[test]
    fn test_non_mvp_agent_generates_error() {
        let gen = SubAgentGenerator::new();
        let result = gen.generate_subagent(PmatSubAgent::RustQualityExpert);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not yet implemented"));
    }
}
