/// Comply subcommands for PMAT compliance checking and migration
#[derive(Debug, Clone, Subcommand)]
pub enum ComplyCommands {
    /// Check project compliance with current PMAT version
    #[command(visible_aliases = &["status"])]
    Check {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Treat warnings as errors (exit 2 on a warnings-only report)
        ///
        /// Exit codes for `comply check` are a tri-state, and `--strict`
        /// changes only the third (#945):
        ///
        /// 0 = no failures, and with --strict no warnings either.
        ///
        /// 1 = one or more checks Failed (with or without --strict).
        ///
        /// 2 = --strict only: 0 failures but 1+ warnings. The report still says
        /// COMPLIANT, and without --strict the same run exits 0.
        ///
        /// Code 2 therefore means "compliant, but not clean". A CI job that
        /// wants the old "fail only on failures" behaviour should drop
        /// --strict rather than special-case the code.
        #[arg(long)]
        strict: bool,

        /// Show only failures (breaking changes/incompatibilities)
        #[arg(long)]
        failures_only: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: ComplyOutputFormat,

        /// Additional project paths to include in cross-stack health checks
        #[arg(long, value_name = "PATH")]
        include_project: Vec<PathBuf>,
    },

    /// Migrate project to latest PMAT standards
    Migrate {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Target PMAT version (defaults to current binary version)
        #[arg(long)]
        version: Option<String>,

        /// Dry run (show what would be migrated without changing files)
        #[arg(long)]
        dry_run: bool,

        /// Skip backup creation (NOT RECOMMENDED)
        #[arg(long)]
        no_backup: bool,

        /// Force migration even if breaking changes detected
        #[arg(long)]
        force: bool,
    },

    /// Upgrade project to a specific quality enforcement style (e.g., Popperian)
    Upgrade {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Target style (e.g., "popperian")
        #[arg(short = 't', long = "target", default_value = "popperian")]
        target: String,

        /// Dry run (show what would be upgraded)
        #[arg(long)]
        dry_run: bool,
    },

    /// Show changelog since project's PMAT version
    Diff {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Start version for changelog (defaults to project version)
        #[arg(long)]
        from: Option<String>,

        /// End version for changelog (defaults to current binary)
        #[arg(long)]
        to: Option<String>,

        /// Show only breaking changes
        #[arg(long)]
        breaking_only: bool,
    },

    /// Update hooks and configs to latest versions
    Update {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Update only hooks
        #[arg(long)]
        hooks: bool,

        /// Update only configs
        #[arg(long)]
        config: bool,

        /// Dry run (show what would be updated)
        #[arg(long)]
        dry_run: bool,
    },

    /// Initialize .pmat/project.toml with current version
    Init {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Force overwrite existing project.toml
        #[arg(long)]
        force: bool,
    },

    /// Install git hooks for mandatory work tracking (W-006)
    /// Blocks commits without active tickets per master-plan-pmat-work-system.md
    #[command(visible_aliases = &["install", "hooks"])]
    Enforce {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Remove all PMAT hooks (disable enforcement)
        #[arg(long)]
        disable: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: ComplyOutputFormat,
    },

    /// Generate compliance report (W-009)
    Report {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Include the debt tickets under .pmat-tickets/ in the report
        ///
        /// It used to be read inside the `-f text` arm alone, and there it only
        /// printed "(Work history not yet implemented)" — so with the DEFAULT
        /// markdown format, and with json/sarif, the flag changed nothing. Every
        /// format carries the ticket listing now.
        #[arg(long)]
        include_history: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "markdown")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// CB-2100: generate the comply enforcement ledger.
    ///
    /// One row per CB rule: severity, whether a required status check actually
    /// reaches it, which invocation carries it, and the file:line it is defined
    /// at. Rules whose status cannot be established are written UNREACHABLE —
    /// that is a finding, not a blank.
    ///
    /// Exits non-zero when the committed ledger has drifted, so the same
    /// command serves as the check and as the generator.
    Ledger {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Write the ledger to docs/status/comply-enforcement-ledger.md
        #[arg(long)]
        write: bool,

        /// Write output to this file instead (implies --write)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// CB-2102: check the ratchet baselines, or lower them.
    ///
    /// Without `--lower` this is the same judgement CB-2102 makes inside
    /// `pmat comply check`, exiting non-zero when any ratcheted metric has got
    /// worse than the value the repository last agreed to.
    ///
    /// With `--lower` it is the scheduled pass: every baseline the tree has
    /// beaten is rewritten to the measured value, and the entry's
    /// `justification` — which no longer justifies anything — is removed. It
    /// can never raise a baseline: the new numbers come from
    /// `kernel::next_baseline`, which is monotone non-increasing.
    Ratchet {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Rewrite every baseline the tree has beaten.
        #[arg(long)]
        lower: bool,
    },

    /// Layer 2 (Genchi Genbutsu): Evidence-based review checklist (COMPLY-045)
    /// Generates a reviewer checklist with reproducibility, hypothesis, and trace evidence.
    Review {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "markdown")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Layer 3 (Governance): Generate audit artifact with sovereign trail (COMPLY-045)
    /// Requires clean git state. Produces UNSIGNED compliance evidence pinned to the HEAD commit.
    //
    // The help used to promise "signed compliance evidence". `AuditArtifact`
    // carries no signature, digest or attestation field of any kind — nothing
    // in the artifact is verifiable after the fact beyond the git SHA it names,
    // so the word "signed" was a claim the command never backed.
    Audit {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "json")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Generate file health baseline for ratchet enforcement
    /// Scans source files, calculates health metrics, saves to .pmat/file-health-baseline.json
    Baseline {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Preview what would be generated without writing files
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate .pmat/binding-index.json, O(1) caches, and contracts/work/*.yaml
    /// Enables CB-1350 differential obligation verification at commit time.
    #[command(visible_aliases = &["refresh", "rb"])]
    RefreshBindings {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },

    /// Override verification level ratchet for a binding (escape hatch for CB-1330).
    /// Records signed entry in .pmat-metrics/ratchet-overrides.jsonl, expires in 14 days.
    #[command(visible_aliases = &["override"])]
    RatchetOverride {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Binding/function name to override
        #[arg(long)]
        binding: String,

        /// Current (higher) level being overridden from
        #[arg(long)]
        from: String,

        /// Target (lower) level being overridden to
        #[arg(long)]
        to: String,

        /// Reason for the override
        #[arg(long)]
        reason: String,

        /// Associated work item ID
        #[arg(long = "work-item")]
        work_item: Option<String>,
    },

    /// Validate non-code asset layout contracts (README, Dockerfile, SVG, etc.)
    /// Runs CB-1320..1326 checks on a specific asset or all assets.
    #[command(visible_aliases = &["av"])]
    AssetValidate {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Specific asset to validate (readme, dockerfile, svg, changelog, badges, book, forjar)
        #[arg(long)]
        asset: Option<String>,
    },

    /// Cross-crate duplication detection (CC-001 through CC-005)
    /// Detects copy-paste duplication, API divergence, and churn correlation across workspace crates.
    #[command(visible_aliases = &["cc", "xc"])]
    CrossCrate {
        /// Workspace root path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Explicit crate paths (comma-separated)
        #[arg(long = "crates", value_delimiter = ',', num_args = 1..)]
        crates: Option<Vec<PathBuf>>,

        /// Minimum similarity threshold for clone detection (0.0-1.0)
        #[arg(long = "similarity-threshold", default_value = "0.80")]
        similarity_threshold: f64,

        /// Window in days for churn correlation (CC-004)
        #[arg(long = "churn-window-days", default_value = "7")]
        churn_window_days: u32,

        /// Comma-separated list of rules to run (e.g., "cc001,cc002")
        #[arg(long = "rules")]
        rules: Option<String>,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Exit with error code 1 if findings detected
        #[arg(long)]
        strict: bool,

        /// Save current finding counts as ratchet baseline
        #[arg(long = "save-baseline")]
        save_baseline: bool,
    },
}

/// Comply output formats
#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum ComplyOutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
    /// SARIF format for GitHub Code Scanning (delegates contract checks to pv lint)
    Sarif,
}

/// Project diagnostics output formats (lltop Tab 8)
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum, PartialEq, Eq)]
pub enum ProjectDiagOutputFormat {
    /// Human-readable summary with status icons
    #[default]
    Summary,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
    /// Andon-style visualization (Toyota Way)
    Andon,
}

/// Output format for perfection score (master-plan-pmat-work-system.md)
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum PerfectionScoreOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}
