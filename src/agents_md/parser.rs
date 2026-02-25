#![cfg_attr(coverage_nightly, coverage(off))]
//! AGENTS.md Parser Implementation
//!
//! Parses markdown files following the AGENTS.md specification with quality validation.

use super::{
    AgentsMdDocument, Command, DocumentMetadata, Guideline, PathBuf, Priority, QualityRules,
    Section, SectionType,
};
use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Parser as MarkdownParser, Tag, TagEnd};
use regex::Regex;
use std::collections::HashMap;

/// Parser for AGENTS.md documents
pub struct AgentsMdParser {
    /// Validation rules
    validation_rules: ValidationRules,

    /// Command patterns
    command_patterns: Vec<Regex>,
}

/// Validation rules for parsing
#[derive(Debug, Clone)]
pub struct ValidationRules {
    /// Require project overview section
    pub require_overview: bool,

    /// Require testing instructions
    pub require_testing: bool,

    /// Maximum document size in bytes
    pub max_size: usize,

    /// Allowed section types
    pub allowed_sections: Vec<SectionType>,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            require_overview: false,
            require_testing: false,
            max_size: 1024 * 1024, // 1MB
            allowed_sections: vec![
                SectionType::Overview,
                SectionType::DevEnvironment,
                SectionType::Testing,
                SectionType::CodeStyle,
                SectionType::PRGuidelines,
                SectionType::Security,
            ],
        }
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether validation passed
    pub valid: bool,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Line number if applicable
    pub line: Option<usize>,

    /// Section where error occurred
    pub section: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Severity level
    pub severity: WarningSeverity,
}

/// Warning severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
}

/// Mutable state accumulated during markdown parsing
#[derive(Default)]
struct ParseState {
    current_section: Option<Section>,
    current_heading_level: u8,
    in_code_block: bool,
    code_block_content: String,
    code_block_lang: String,
    in_list: bool,
    list_item_content: String,
}

impl Default for AgentsMdParser {
    fn default() -> Self {
        Self::new()
    }
}

// Core parsing methods: new(), with_rules(), parse(), process_event(), event handlers
include!("parser_core.rs");

// Extraction, detection, and validation: validate(), extract_*(), detect_*(), is_command_safe()
include!("parser_extraction.rs");

// Unit tests
include!("parser_tests.rs");
