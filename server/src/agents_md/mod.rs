//! AGENTS.md Integration Module
//! 
//! Implements the AGENTS.md standard for AI agent guidance with PMAT quality enforcement.
//! Provides bidirectional bridge between AGENTS.md and MCP protocols.

pub mod parser;
pub mod discovery;
pub mod executor;
pub mod generator;
pub mod bridge;
pub mod manifest;
pub mod router;
pub mod quality;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Core AGENTS.md document structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentsMdDocument {
    /// Document metadata
    pub metadata: DocumentMetadata,
    
    /// Parsed sections
    pub sections: Vec<Section>,
    
    /// Extracted commands
    pub commands: Vec<Command>,
    
    /// Project guidelines
    pub guidelines: Vec<Guideline>,
    
    /// Quality rules if specified
    pub quality_rules: Option<QualityRules>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentMetadata {
    /// File path
    pub path: PathBuf,
    
    /// Last modified time
    pub modified: std::time::SystemTime,
    
    /// Document version if specified
    pub version: Option<String>,
    
    /// Project name if specified
    pub project: Option<String>,
}

/// Section in AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    /// Section type
    pub section_type: SectionType,
    
    /// Section title
    pub title: String,
    
    /// Section content
    pub content: String,
    
    /// Nested subsections
    pub subsections: Vec<Section>,
}

/// Types of sections in AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SectionType {
    Overview,
    DevEnvironment,
    Testing,
    CodeStyle,
    PRGuidelines,
    Security,
    Custom(String),
}

/// Executable command from AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Command {
    /// Command name/description
    pub name: String,
    
    /// Actual command to execute
    pub command: String,
    
    /// Expected working directory
    pub working_dir: Option<PathBuf>,
    
    /// Environment variables
    pub env: Vec<(String, String)>,
    
    /// Timeout in seconds
    pub timeout: Option<u64>,
    
    /// Whether command is safe to execute
    pub safe: bool,
}

/// Project guideline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guideline {
    /// Guideline category
    pub category: String,
    
    /// Guideline text
    pub text: String,
    
    /// Priority level
    pub priority: Priority,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Quality rules extracted from AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityRules {
    /// Maximum complexity allowed
    pub max_complexity: Option<u32>,
    
    /// Minimum test coverage
    pub min_coverage: Option<f64>,
    
    /// SATD tolerance
    pub satd_allowed: bool,
    
    /// Custom quality checks
    pub custom_checks: Vec<String>,
}

/// Agent capabilities manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Supported operations
    pub operations: Vec<Operation>,
    
    /// Available tools
    pub tools: Vec<Tool>,
    
    /// Quality requirements
    pub quality_requirements: QualityRequirements,
}

/// Supported operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation name
    pub name: String,
    
    /// Description
    pub description: String,
    
    /// Input schema
    pub input_schema: serde_json::Value,
    
    /// Output schema
    pub output_schema: serde_json::Value,
}

/// Available tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    
    /// Tool type
    pub tool_type: ToolType,
    
    /// Configuration
    pub config: serde_json::Value,
}

/// Tool types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Command,
    Analysis,
    Generator,
    Validator,
    Custom(String),
}

/// Quality requirements for agent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Enforce quality gates
    pub enforce_gates: bool,
    
    /// Maximum allowed complexity
    pub max_complexity: u32,
    
    /// Minimum test coverage
    pub min_coverage: f64,
    
    /// Auto-fix issues
    pub auto_fix: bool,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            enforce_gates: true,
            max_complexity: 10,
            min_coverage: 80.0,
            auto_fix: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_requirements_default() {
        let req = QualityRequirements::default();
        assert!(req.enforce_gates);
        assert_eq!(req.max_complexity, 10);
        assert_eq!(req.min_coverage, 80.0);
        assert!(req.auto_fix);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_section_type_equality() {
        assert_eq!(SectionType::Testing, SectionType::Testing);
        assert_ne!(SectionType::Testing, SectionType::Security);
        assert_eq!(
            SectionType::Custom("foo".to_string()),
            SectionType::Custom("foo".to_string())
        );
    }
}