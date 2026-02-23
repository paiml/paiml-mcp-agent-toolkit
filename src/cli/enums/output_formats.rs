#![cfg_attr(coverage_nightly, coverage(off))]
//! General output format enums for CLI commands

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Output format for general commands
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
        }
    }
}

/// Output format for query command (PMAT-470: RAG agent context)
#[derive(Clone, Debug, ValueEnum, PartialEq, Default)]
pub enum QueryOutputFormat {
    /// Plain text output
    #[default]
    Text,
    /// JSON output for scripting
    Json,
    /// Markdown output for documentation
    Markdown,
}

impl fmt::Display for QueryOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryOutputFormat::Text => write!(f, "text"),
            QueryOutputFormat::Json => write!(f, "json"),
            QueryOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Enforce output format  
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum EnforceOutputFormat {
    /// Summary output
    Summary,
    /// JSON output
    Json,
    /// Progress output
    Progress,
    /// SARIF format
    Sarif,
}

impl fmt::Display for EnforceOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnforceOutputFormat::Summary => write!(f, "summary"),
            EnforceOutputFormat::Json => write!(f, "json"),
            EnforceOutputFormat::Progress => write!(f, "progress"),
            EnforceOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Refactor output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum RefactorOutputFormat {
    Json,
    Table,
    Summary,
}

impl fmt::Display for RefactorOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorOutputFormat::Json => write!(f, "json"),
            RefactorOutputFormat::Table => write!(f, "table"),
            RefactorOutputFormat::Summary => write!(f, "summary"),
        }
    }
}

/// Prompt output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum PromptOutputFormat {
    Yaml,
    Json,
    Text,
}

impl fmt::Display for PromptOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptOutputFormat::Yaml => write!(f, "yaml"),
            PromptOutputFormat::Json => write!(f, "json"),
            PromptOutputFormat::Text => write!(f, "text"),
        }
    }
}

/// Refactor auto output format
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum RefactorAutoOutputFormat {
    /// Concise summary of progress
    Summary,
    /// Detailed progress information
    Detailed,
    /// JSON format for automation
    Json,
}

impl fmt::Display for RefactorAutoOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorAutoOutputFormat::Summary => write!(f, "summary"),
            RefactorAutoOutputFormat::Detailed => write!(f, "detailed"),
            RefactorAutoOutputFormat::Json => write!(f, "json"),
        }
    }
}

/// Refactor docs output format
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum RefactorDocsOutputFormat {
    /// Summary of found issues
    Summary,
    /// Detailed list with explanations
    Detailed,
    /// JSON format for automation
    Json,
    /// Interactive mode for confirmation
    Interactive,
}

impl fmt::Display for RefactorDocsOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorDocsOutputFormat::Summary => write!(f, "summary"),
            RefactorDocsOutputFormat::Detailed => write!(f, "detailed"),
            RefactorDocsOutputFormat::Json => write!(f, "json"),
            RefactorDocsOutputFormat::Interactive => write!(f, "interactive"),
        }
    }
}

/// Context format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ContextFormat {
    Markdown,
    Json,
    Sarif,
    #[value(name = "llm-optimized")]
    LlmOptimized,
}

impl fmt::Display for ContextFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextFormat::Markdown => write!(f, "markdown"),
            ContextFormat::Json => write!(f, "json"),
            ContextFormat::Sarif => write!(f, "sarif"),
            ContextFormat::LlmOptimized => write!(f, "llm-optimized"),
        }
    }
}

/// TDG output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum TdgOutputFormat {
    Table,
    Json,
    Markdown,
    Sarif,
}

impl fmt::Display for TdgOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TdgOutputFormat::Table => write!(f, "table"),
            TdgOutputFormat::Json => write!(f, "json"),
            TdgOutputFormat::Markdown => write!(f, "markdown"),
            TdgOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Makefile output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum MakefileOutputFormat {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
    /// GCC-style output for editor integration
    Gcc,
    /// SARIF format for CI/CD integration
    Sarif,
}

impl fmt::Display for MakefileOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MakefileOutputFormat::Human => write!(f, "human"),
            MakefileOutputFormat::Json => write!(f, "json"),
            MakefileOutputFormat::Gcc => write!(f, "gcc"),
            MakefileOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Lint hotspot output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum LintHotspotOutputFormat {
    /// Summary output
    Summary,
    /// Detailed output
    Detailed,
    /// JSON output
    Json,
    /// Enforcement JSON output
    #[value(name = "enforcement-json")]
    EnforcementJson,
    /// SARIF format for CI/CD integration
    Sarif,
}

impl fmt::Display for LintHotspotOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LintHotspotOutputFormat::Summary => write!(f, "summary"),
            LintHotspotOutputFormat::Detailed => write!(f, "detailed"),
            LintHotspotOutputFormat::Json => write!(f, "json"),
            LintHotspotOutputFormat::EnforcementJson => write!(f, "enforcement-json"),
            LintHotspotOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Report output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ReportOutputFormat {
    /// JSON format (default)
    Json,
    /// CSV format for spreadsheet analysis
    Csv,
    /// Markdown format with tables and visualizations
    Markdown,
    /// Plain text format
    Text,
    /// HTML format (legacy)
    Html,
    /// PDF format (legacy)
    Pdf,
    /// Dashboard format (legacy)
    Dashboard,
}

impl ReportOutputFormat {
    /// Get the string representation of the report output format
    fn as_str(&self) -> &'static str {
        match self {
            ReportOutputFormat::Json => "json",
            ReportOutputFormat::Csv => "csv",
            ReportOutputFormat::Markdown => "markdown",
            ReportOutputFormat::Text => "text",
            ReportOutputFormat::Html => "html",
            ReportOutputFormat::Pdf => "pdf",
            ReportOutputFormat::Dashboard => "dashboard",
        }
    }
}

impl fmt::Display for ReportOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Output format for repository health score
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum RepoScoreOutputFormat {
    /// Text format with colored output (default)
    Text,
    /// JSON format for programmatic use
    Json,
    /// Markdown format with tables
    Markdown,
    /// YAML format
    Yaml,
}

impl RepoScoreOutputFormat {
    /// Get the string representation
    fn as_str(&self) -> &'static str {
        match self {
            RepoScoreOutputFormat::Text => "text",
            RepoScoreOutputFormat::Json => "json",
            RepoScoreOutputFormat::Markdown => "markdown",
            RepoScoreOutputFormat::Yaml => "yaml",
        }
    }
}

impl fmt::Display for RepoScoreOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Output format for Five Whys debug analysis
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DebugOutputFormat {
    Text,
    Json,
    Markdown,
}

impl fmt::Display for DebugOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugOutputFormat::Text => write!(f, "text"),
            DebugOutputFormat::Json => write!(f, "json"),
            DebugOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}
