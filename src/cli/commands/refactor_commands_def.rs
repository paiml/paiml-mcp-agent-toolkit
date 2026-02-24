/// Refactor subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum RefactorCommands {
    /// Run refactor server mode for batch processing
    Serve {
        /// Refactor mode (batch or interactive)
        #[arg(long, value_enum, default_value = "batch")]
        refactor_mode: RefactorMode,

        /// JSON configuration file for batch mode
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Project directory to refactor
        #[arg(short = 'p', long, default_value = ".")]
        project: PathBuf,

        /// Number of parallel workers
        #[arg(long, default_value = "4")]
        parallel: usize,

        /// Memory limit in MB
        #[arg(long, default_value = "512")]
        memory_limit: usize,

        /// Files per batch
        #[arg(long, default_value = "10")]
        batch_size: usize,

        /// Priority sorting expression (e.g., "complexity * `defect_probability`")
        #[arg(long)]
        priority: Option<String>,

        /// Checkpoint directory for resuming
        #[arg(long)]
        checkpoint_dir: Option<PathBuf>,

        /// Resume from previous checkpoint
        #[arg(long)]
        resume: bool,

        /// Auto-commit with message template
        #[arg(long)]
        auto_commit: Option<String>,

        /// Maximum runtime in seconds
        #[arg(long)]
        max_runtime: Option<u64>,
    },

    /// Run interactive refactoring mode
    Interactive {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Explanation level for operations
        #[arg(long, value_enum, default_value = "detailed")]
        explain: ExplainLevel,

        /// Checkpoint file for state persistence
        #[arg(long, default_value = "refactor_state.json")]
        checkpoint: PathBuf,

        /// Target complexity threshold
        #[arg(long, default_value = "20")]
        target_complexity: u16,

        /// Maximum steps to execute
        #[arg(long)]
        steps: Option<u32>,

        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Show current refactoring status
    Status {
        /// Checkpoint file to read state from
        #[arg(long, default_value = "refactor_state.json")]
        checkpoint: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: RefactorOutputFormat,
    },

    /// Resume refactoring from checkpoint
    Resume {
        /// Checkpoint file to resume from
        #[arg(long, default_value = "refactor_state.json")]
        checkpoint: PathBuf,

        /// Maximum steps to execute
        #[arg(long, default_value = "10")]
        steps: u32,

        /// Override explanation level
        #[arg(long, value_enum)]
        explain: Option<ExplainLevel>,
    },

    /// AI-powered automated refactoring to achieve RIGID extreme quality standards
    Auto {
        /// Project path to refactor
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Single file mode - refactor one file at a time
        #[arg(long)]
        single_file_mode: bool,

        /// Specific file to refactor (implies single file mode)
        #[arg(long)]
        file: Option<PathBuf>,

        /// Maximum iterations to run
        #[arg(long, default_value = "100")]
        max_iterations: u32,

        /// Quality profile to enforce
        #[arg(long, value_enum, default_value = "extreme")]
        quality_profile: QualityProfile,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "detailed")]
        format: RefactorAutoOutputFormat,

        /// Dry run mode (don't write files)
        #[arg(long)]
        dry_run: bool,

        /// Skip compilation check
        #[arg(long)]
        skip_compilation: bool,

        /// Skip test execution
        #[arg(long)]
        skip_tests: bool,

        /// Output checkpoint file
        #[arg(long)]
        checkpoint: Option<PathBuf>,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Patterns to exclude from refactoring (e.g., "tests/**", "benches/**")
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Patterns to include for refactoring (overrides exclude)
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Path to .refactorignore file
        #[arg(long)]
        ignore_file: Option<PathBuf>,

        /// Specific test file to fix (automatically includes related source files)
        #[arg(long, short = 't')]
        test: Option<PathBuf>,

        /// Test name pattern to fix (e.g., "`test_mixed_language_project_context`")
        #[arg(long)]
        test_name: Option<String>,

        /// GitHub issue URL to guide the refactoring process
        #[arg(long)]
        github_issue: Option<String>,

        /// Bug report markdown file path to analyze and fix
        #[arg(long)]
        bug_report_path: Option<PathBuf>,
    },

    /// AI-assisted documentation cleanup and refactoring
    Docs {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Include docs directory
        #[arg(long, default_value_t = true)]
        include_docs: bool,

        /// Include root directory
        #[arg(long, default_value_t = true)]
        include_root: bool,

        /// Additional directories to scan
        #[arg(long, value_delimiter = ',')]
        additional_dirs: Vec<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: RefactorDocsOutputFormat,

        /// Dry run - show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Patterns to identify temporary files (e.g., "fix-*.sh", "*_TEMP.md")
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "fix-*,test-*,temp-*,tmp-*,*_TEMP*,*_TMP*,FAST_*,FIX_*,ZERO_DEFECTS_*"
        )]
        temp_patterns: Vec<String>,

        /// Patterns to identify outdated status files
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "*_STATUS.md,*_PROGRESS.md,*_COMPLETE.md,final_verification.md,overnight-*.md"
        )]
        status_patterns: Vec<String>,

        /// Patterns to identify build artifacts
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "*.mmd,optimization_state.json,complexity_report.json,satd_report.json"
        )]
        artifact_patterns: Vec<String>,

        /// Custom patterns to include in cleanup
        #[arg(long, value_delimiter = ',')]
        custom_patterns: Vec<String>,

        /// Minimum age in days before considering a file for cleanup
        #[arg(long, default_value_t = 0)]
        min_age_days: u32,

        /// Maximum file size in MB to consider (larger files are skipped)
        #[arg(long, default_value_t = 10)]
        max_size_mb: u64,

        /// Include subdirectories recursively
        #[arg(long, default_value_t = true)]
        recursive: bool,

        /// Preserve files matching these patterns (overrides other patterns)
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "README.md,LICENSE*,CHANGELOG*,CONTRIBUTING*"
        )]
        preserve_patterns: Vec<String>,

        /// Output file path for the report
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Auto-remove files without confirmation (use with caution)
        #[arg(long)]
        auto_remove: bool,

        /// Create backup before removing files
        #[arg(long)]
        backup: bool,

        /// Backup directory path
        #[arg(long, default_value = ".refactor-docs-backup")]
        backup_dir: PathBuf,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,
    },
}
