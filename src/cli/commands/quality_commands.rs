#![cfg_attr(coverage_nightly, coverage(off))]
// Quality commands (QDD, Enforce) - extracted for file health (CB-040)

use crate::cli::{EnforceOutputFormat, QualityProfile};
use clap::Subcommand;
use std::path::PathBuf;

/// Quality-Driven Development (QDD) subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum QddCommands {
    /// Create high-quality code from specification
    Create {
        /// Type of code to create
        #[arg(long, value_enum, default_value = "function")]
        code_type: QddCodeType,

        /// Name of the code element (function, module, etc.)
        #[arg(long)]
        name: String,

        /// Purpose/description of the code
        #[arg(long)]
        purpose: String,

        /// Quality profile to use
        #[arg(long, value_enum, default_value = "standard")]
        profile: QddQualityProfile,

        /// Input parameters as type:name pairs
        #[arg(long, value_parser = parse_parameter)]
        input: Vec<(String, String)>,

        /// Output type
        #[arg(long, default_value = "()")]
        output: String,

        /// Output file path
        #[arg(short, long)]
        output_file: Option<PathBuf>,
    },

    /// Refactor existing code to meet quality standards  
    Refactor {
        /// File to refactor
        #[arg(short, long)]
        file: PathBuf,

        /// Specific function to refactor (optional)
        #[arg(long)]
        function: Option<String>,

        /// Quality profile to target
        #[arg(long, value_enum, default_value = "standard")]
        profile: QddQualityProfile,

        /// Maximum complexity allowed
        #[arg(long)]
        max_complexity: Option<u32>,

        /// Minimum test coverage required (%)
        #[arg(long)]
        min_coverage: Option<u32>,

        /// Output file path (default: overwrite original)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Dry run - show what would be changed
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate code against quality standards
    Validate {
        /// File or directory to validate
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Quality profile to validate against
        #[arg(long, value_enum, default_value = "standard")]
        profile: QddQualityProfile,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: QddOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Fail on any violations
        #[arg(long)]
        strict: bool,
    },
}

/// QDD code types
#[derive(clap::ValueEnum, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum QddCodeType {
    Function,
    Module,
    Service,
    Test,
}

/// QDD quality profiles  
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum QddQualityProfile {
    Extreme,
    Standard,
    Relaxed,
}

/// QDD output formats
#[derive(clap::ValueEnum, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum QddOutputFormat {
    Summary,
    Detailed,
    Json,
    Markdown,
}

/// Parse parameter as type:name
fn parse_parameter(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("Parameter must be in format type:name".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Enforce subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum EnforceCommands {
    /// Enforce extreme quality standards
    Extreme {
        /// Project path to enforce quality on
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Single file mode - enforce on one file at a time
        #[arg(long)]
        single_file_mode: bool,

        /// Specific file to enforce (implies single file mode)
        #[arg(long)]
        file: Option<PathBuf>,

        /// Dry run - show what would be changed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Quality profile to use
        #[arg(long, value_enum, default_value = "extreme")]
        profile: QualityProfile,

        /// Show progress during enforcement
        ///
        /// This was `default_value_t = true` with no `--no-` counterpart, so
        /// passing `--show-progress` set a flag that was already set and the
        /// output was byte-identical either way. A flag that cannot be off is
        /// not a flag.
        #[arg(long)]
        show_progress: bool,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: EnforceOutputFormat,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Maximum iterations before giving up
        #[arg(long, default_value_t = 100)]
        max_iterations: u32,

        /// Target improvement percentage
        #[arg(long)]
        target_improvement: Option<f32>,

        /// Maximum time in seconds
        #[arg(long)]
        max_time: Option<u64>,

        /// Apply suggestions automatically
        #[arg(long)]
        apply_suggestions: bool,

        /// Validate only (no changes)
        #[arg(long)]
        validate_only: bool,

        /// List all violations and exit
        #[arg(long)]
        list_violations: bool,

        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,

        /// CI mode (exit with error on violations)
        #[arg(long)]
        ci_mode: bool,

        /// Include pattern
        #[arg(long)]
        include: Option<String>,

        /// Exclude pattern
        #[arg(long)]
        exclude: Option<String>,

        /// Cache directory
        #[arg(long)]
        cache_dir: Option<PathBuf>,

        /// Clear cache before starting
        #[arg(long)]
        clear_cache: bool,
    },
}

#[cfg(test)]
mod quality_commands_tests {
    use super::*;

    #[test]
    fn test_parse_parameter_valid() {
        let (ty, name) = parse_parameter("String:user_id").unwrap();
        assert_eq!(ty, "String");
        assert_eq!(name, "user_id");
    }

    #[test]
    fn test_parse_parameter_no_colon() {
        let err = parse_parameter("no_colon").unwrap_err();
        assert!(err.contains("type:name"));
    }

    #[test]
    fn test_parse_parameter_too_many_colons() {
        let err = parse_parameter("a:b:c").unwrap_err();
        assert!(err.contains("type:name"));
    }

    #[test]
    fn test_parse_parameter_empty_parts() {
        let (ty, name) = parse_parameter(":").unwrap();
        assert_eq!(ty, "");
        assert_eq!(name, "");
    }
}
