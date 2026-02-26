// Contract Profiles: opt-in tiers for Design by Contract
// Spec: docs/specifications/dbc.md §2

/// Contract profile — determines which claims are generated and enforced
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractProfile {
    /// Any project with git + build/test. 6 claims.
    Universal,
    /// Cargo project (Cargo.toml). 14 claims.
    Rust,
    /// Full batuta stack (.pmat/ dir). 25 claims.
    Pmat,
    /// Third-party stack manifest. Variable claims.
    Stack { manifest_path: PathBuf },
    /// User-defined cherry-picked claims.
    Custom { claim_ids: Vec<String> },
}

impl ContractProfile {
    /// Auto-detect profile from project structure.
    /// Evaluated top-down, first match wins.
    pub fn detect(project_path: &Path) -> Self {
        // Check for explicit config override first
        let config_path = project_path.join(".pmat-work").join("config.toml");
        if config_path.exists() {
            if let Ok(config) = DbcConfig::load_from_path(&config_path) {
                if let Some(profile) = config.profile_override {
                    return profile;
                }
            }
        }

        // Check for stack manifest
        let stack_path = project_path.join(".dbc-stack.toml");
        if stack_path.exists() {
            return ContractProfile::Stack {
                manifest_path: stack_path,
            };
        }

        // Check for pmat index (full batuta stack)
        let pmat_db = project_path.join(".pmat").join("context.db");
        let pmat_idx = project_path.join(".pmat").join("context.idx");
        if pmat_db.exists() || pmat_idx.exists() {
            return ContractProfile::Pmat;
        }

        // Check for Cargo.toml (Rust project)
        if project_path.join("Cargo.toml").exists() {
            return ContractProfile::Rust;
        }

        // Check for git (universal)
        if project_path.join(".git").exists() {
            return ContractProfile::Universal;
        }

        // Fallback to universal (we'll verify git later in toolchain check)
        ContractProfile::Universal
    }

    /// Human-readable profile name
    pub fn name(&self) -> &str {
        match self {
            ContractProfile::Universal => "Universal",
            ContractProfile::Rust => "Rust",
            ContractProfile::Pmat => "Pmat",
            ContractProfile::Stack { .. } => "Stack",
            ContractProfile::Custom { .. } => "Custom",
        }
    }

    /// Required tools for this profile
    pub fn required_tools(&self) -> Vec<RequiredTool> {
        match self {
            ContractProfile::Universal => vec![RequiredTool {
                name: "git".to_string(),
                claim_id: "require.compiles".to_string(),
                install_hint: "Install git from https://git-scm.com".to_string(),
            }],
            ContractProfile::Rust => vec![
                RequiredTool {
                    name: "cargo".to_string(),
                    claim_id: "require.compiles".to_string(),
                    install_hint: "Install Rust from https://rustup.rs".to_string(),
                },
                RequiredTool {
                    name: "cargo-clippy".to_string(),
                    claim_id: "invariant.lint".to_string(),
                    install_hint: "rustup component add clippy".to_string(),
                },
                RequiredTool {
                    name: "cargo-llvm-cov".to_string(),
                    claim_id: "ensure.coverage".to_string(),
                    install_hint: "cargo +nightly install cargo-llvm-cov".to_string(),
                },
                RequiredTool {
                    name: "cargo-audit".to_string(),
                    claim_id: "ensure.supply_chain".to_string(),
                    install_hint: "cargo install cargo-audit".to_string(),
                },
            ],
            ContractProfile::Pmat => vec![
                RequiredTool {
                    name: "cargo".to_string(),
                    claim_id: "require.compiles".to_string(),
                    install_hint: "Install Rust from https://rustup.rs".to_string(),
                },
                RequiredTool {
                    name: "cargo-clippy".to_string(),
                    claim_id: "invariant.lint".to_string(),
                    install_hint: "rustup component add clippy".to_string(),
                },
                RequiredTool {
                    name: "cargo-llvm-cov".to_string(),
                    claim_id: "ensure.coverage".to_string(),
                    install_hint: "cargo +nightly install cargo-llvm-cov".to_string(),
                },
                RequiredTool {
                    name: "cargo-audit".to_string(),
                    claim_id: "ensure.supply_chain".to_string(),
                    install_hint: "cargo install cargo-audit".to_string(),
                },
                RequiredTool {
                    name: "pmat".to_string(),
                    claim_id: "invariant.satd".to_string(),
                    install_hint: "cargo install pmat".to_string(),
                },
            ],
            ContractProfile::Stack { .. } | ContractProfile::Custom { .. } => {
                // Stack/Custom tools are checked separately via manifest
                vec![]
            }
        }
    }
}

/// A tool required by a contract profile
#[derive(Debug, Clone)]
pub struct RequiredTool {
    /// Tool binary name (looked up in PATH)
    pub name: String,
    /// Which claim needs this tool
    pub claim_id: String,
    /// How to install the tool
    pub install_hint: String,
}

impl RequiredTool {
    /// Check if this tool is available on the system
    pub fn is_available(&self) -> bool {
        which_tool(&self.name)
    }
}

/// A tool that was required but not found
#[derive(Debug, Clone)]
pub struct MissingTool {
    /// Tool name
    pub name: String,
    /// Claim that needs it
    pub claim_id: String,
    /// Install instructions
    pub install_hint: String,
}

/// Check if a tool binary exists in PATH
fn which_tool(name: &str) -> bool {
    // Handle special cases
    match name {
        "cargo-clippy" => {
            std::process::Command::new("cargo")
                .args(["clippy", "--version"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        "cargo-llvm-cov" => {
            std::process::Command::new("cargo")
                .args(["llvm-cov", "--version"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        "cargo-audit" => {
            std::process::Command::new("cargo")
                .args(["audit", "--version"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        _ => {
            std::process::Command::new("which")
                .arg(name)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    }
}

/// Check toolchain requirements for a profile. Returns list of missing tools.
pub fn check_toolchain(profile: &ContractProfile, _project_path: &Path) -> Vec<MissingTool> {
    profile
        .required_tools()
        .into_iter()
        .filter(|tool| !tool.is_available())
        .map(|tool| MissingTool {
            name: tool.name,
            claim_id: tool.claim_id,
            install_hint: tool.install_hint,
        })
        .collect()
}

/// DbC configuration from .pmat-work/config.toml
#[derive(Debug, Clone, Default)]
pub struct DbcConfig {
    /// Override the auto-detected profile
    pub profile_override: Option<ContractProfile>,
    /// Custom threshold overrides
    pub thresholds: DbcThresholdOverrides,
    /// Whether rescue protocol is enabled
    pub rescue_enabled: Option<bool>,
    /// Checkpoint configuration
    pub pre_commit_hook: bool,
}

/// Threshold overrides from config
#[derive(Debug, Clone, Default)]
pub struct DbcThresholdOverrides {
    pub coverage_pct: Option<f64>,
    pub max_complexity: Option<u32>,
    pub max_file_lines: Option<usize>,
}

impl DbcConfig {
    /// Load DbC config from .pmat-work/config.toml
    pub fn load(project_path: &Path) -> Self {
        let config_path = project_path.join(".pmat-work").join("config.toml");
        Self::load_from_path(&config_path).unwrap_or_default()
    }

    /// Load from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        Self::parse_toml(&content)
    }

    /// Parse TOML content into DbcConfig
    fn parse_toml(content: &str) -> Result<Self> {
        let table: toml::Table = content
            .parse()
            .context("Failed to parse config.toml")?;

        let mut config = DbcConfig::default();

        if let Some(dbc) = table.get("dbc").and_then(|v| v.as_table()) {
            // Profile override
            if let Some(profile_str) = dbc.get("profile").and_then(|v| v.as_str()) {
                config.profile_override = match profile_str {
                    "universal" => Some(ContractProfile::Universal),
                    "rust" => Some(ContractProfile::Rust),
                    "pmat" => Some(ContractProfile::Pmat),
                    "custom" => {
                        let claims = dbc
                            .get("claims")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        Some(ContractProfile::Custom { claim_ids: claims })
                    }
                    _ => None,
                };
            }

            // Threshold overrides
            if let Some(thresholds) = dbc.get("thresholds").and_then(|v| v.as_table()) {
                if let Some(cov) = thresholds.get("coverage_pct").and_then(|v| v.as_float()) {
                    config.thresholds.coverage_pct = Some(cov);
                }
                if let Some(cx) = thresholds.get("max_complexity").and_then(|v| v.as_integer()) {
                    config.thresholds.max_complexity = Some(cx as u32);
                }
                if let Some(fl) = thresholds.get("max_file_lines").and_then(|v| v.as_integer()) {
                    config.thresholds.max_file_lines = Some(fl as usize);
                }
            }

            // Rescue config
            if let Some(rescue) = dbc.get("rescue").and_then(|v| v.as_table()) {
                if let Some(enabled) = rescue.get("enabled").and_then(|v| v.as_bool()) {
                    config.rescue_enabled = Some(enabled);
                }
            }

            // Checkpoint config
            if let Some(checkpoints) = dbc.get("checkpoints").and_then(|v| v.as_table()) {
                if let Some(hook) = checkpoints.get("pre_commit_hook").and_then(|v| v.as_bool()) {
                    config.pre_commit_hook = hook;
                }
            }
        }

        Ok(config)
    }
}

/// Generate claims for a given profile
pub fn claims_for_profile(
    profile: &ContractProfile,
    config: &DbcConfig,
) -> Vec<ContractClause> {
    let max_complexity = config.thresholds.max_complexity.unwrap_or(20) as f64;
    let max_file_lines = config.thresholds.max_file_lines.unwrap_or(500) as f64;
    let coverage_pct = config.thresholds.coverage_pct.unwrap_or(95.0);

    match profile {
        ContractProfile::Universal => universal_claims(max_file_lines),
        ContractProfile::Rust => rust_claims(max_complexity, max_file_lines, coverage_pct),
        ContractProfile::Pmat => pmat_claims(max_complexity, max_file_lines, coverage_pct),
        ContractProfile::Custom { claim_ids } => {
            // Generate all Pmat claims, then filter to requested IDs
            let all = pmat_claims(max_complexity, max_file_lines, coverage_pct);
            all.into_iter()
                .filter(|c| claim_ids.contains(&c.id))
                .collect()
        }
        ContractProfile::Stack { .. } => {
            // Stack claims come from manifest parsing (Phase 2)
            // For now, return universal as base
            universal_claims(max_file_lines)
        }
    }
}

/// Classify a flat list of clauses into the Meyer triad
pub fn classify_claims(
    clauses: &[ContractClause],
) -> (Vec<ContractClause>, Vec<ContractClause>, Vec<ContractClause>) {
    let mut require = Vec::new();
    let mut ensure = Vec::new();
    let mut invariant = Vec::new();

    for clause in clauses {
        match clause.kind {
            ClauseKind::Require => require.push(clause.clone()),
            ClauseKind::Ensure => ensure.push(clause.clone()),
            ClauseKind::Invariant => invariant.push(clause.clone()),
        }
    }

    (require, ensure, invariant)
}

/// Apply explicit exclusions (--without). Returns (active, excluded).
pub fn apply_exclusions(
    clauses: Vec<ContractClause>,
    without: &[String],
) -> (Vec<ContractClause>, Vec<ExcludedClaim>) {
    let (excluded_clauses, active): (Vec<_>, Vec<_>) =
        clauses.into_iter().partition(|c| without.contains(&c.id));

    let excluded = excluded_clauses
        .into_iter()
        .map(|c| ExcludedClaim {
            flag: format!("--without {}", c.id),
            id: c.id,
            reason: "developer_excluded".to_string(),
        })
        .collect();

    (active, excluded)
}

// === Claim generators by profile ===

fn universal_claims(max_file_lines: f64) -> Vec<ContractClause> {
    vec![
        // Require (2)
        ContractClause {
            id: "require.compiles".to_string(),
            kind: ClauseKind::Require,
            description: "Project builds successfully".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "require.tests_exist".to_string(),
            kind: ClauseKind::Require,
            description: "At least one test exists".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        // Invariant (2)
        ContractClause {
            id: "invariant.compiles".to_string(),
            kind: ClauseKind::Invariant,
            description: "Project compiles at every checkpoint".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.file_size".to_string(),
            kind: ClauseKind::Invariant,
            description: format!("No file exceeds {} lines", max_file_lines as usize),
            falsification_method: FalsificationMethod::FileSizeRegression,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "max_file_lines".to_string(),
                op: ThresholdOp::Lte,
                value: max_file_lines,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        // Ensure (2)
        ContractClause {
            id: "ensure.tests_pass".to_string(),
            kind: ClauseKind::Ensure,
            description: "All tests pass".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.git_sync".to_string(),
            kind: ClauseKind::Ensure,
            description: "All changes pushed to remote".to_string(),
            falsification_method: FalsificationMethod::GitHubSync,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
    ]
}

fn rust_claims(max_complexity: f64, max_file_lines: f64, coverage_pct: f64) -> Vec<ContractClause> {
    let mut claims = universal_claims(max_file_lines);

    // Additional Require: none beyond universal

    // Additional Invariant (2 more = 4 total)
    claims.extend([
        ContractClause {
            id: "invariant.lint".to_string(),
            kind: ClauseKind::Invariant,
            description: "cargo clippy passes".to_string(),
            falsification_method: FalsificationMethod::LintPass,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.complexity".to_string(),
            kind: ClauseKind::Invariant,
            description: format!("No function exceeds complexity {}", max_complexity as u32),
            falsification_method: FalsificationMethod::ComplexityRegression,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "max_complexity".to_string(),
                op: ThresholdOp::Lte,
                value: max_complexity,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    // Additional Ensure (6 more = 8 total)
    claims.extend([
        ContractClause {
            id: "ensure.no_regression".to_string(),
            kind: ClauseKind::Ensure,
            description: "No previously-passing test now fails".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: format!("Coverage >= {}%", coverage_pct),
            falsification_method: FalsificationMethod::AbsoluteCoverage,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "coverage_pct".to_string(),
                op: ThresholdOp::Gte,
                value: coverage_pct,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.differential_coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: "Changed lines must be covered".to_string(),
            falsification_method: FalsificationMethod::DifferentialCoverage,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.supply_chain".to_string(),
            kind: ClauseKind::Ensure,
            description: "No vulnerable dependencies".to_string(),
            falsification_method: FalsificationMethod::SupplyChainIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.examples_compile".to_string(),
            kind: ClauseKind::Ensure,
            description: "All examples compile and run".to_string(),
            falsification_method: FalsificationMethod::ExamplesCompile,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.tdg_regression".to_string(),
            kind: ClauseKind::Ensure,
            description: "TDG score >= baseline".to_string(),
            falsification_method: FalsificationMethod::TdgRegression,
            threshold: Some(ClauseThreshold::Delta {
                metric: "tdg_score".to_string(),
                op: ThresholdOp::Gte,
                value: 0.0,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    claims
}

fn pmat_claims(max_complexity: f64, max_file_lines: f64, coverage_pct: f64) -> Vec<ContractClause> {
    let mut claims = rust_claims(max_complexity, max_file_lines, coverage_pct);

    // Additional Require (2 more = 4 total)
    claims.extend([
        ContractClause {
            id: "require.manifest_integrity".to_string(),
            kind: ClauseKind::Require,
            description: "FileManifest anti-gaming check".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "require.meta_falsification".to_string(),
            kind: ClauseKind::Require,
            description: "Falsifier self-check active".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    // Additional Invariant (3 more = 7 total)
    claims.extend([
        ContractClause {
            id: "invariant.satd".to_string(),
            kind: ClauseKind::Invariant,
            description: "No new SATD markers".to_string(),
            falsification_method: FalsificationMethod::SatdDetection,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.dead_code".to_string(),
            kind: ClauseKind::Invariant,
            description: "No new dead code introduced".to_string(),
            falsification_method: FalsificationMethod::DeadCodeDetection,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.fix_chain".to_string(),
            kind: ClauseKind::Invariant,
            description: "Fix-after-fix chains within limit".to_string(),
            falsification_method: FalsificationMethod::FixChainLimit,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "fix_chain_count".to_string(),
                op: ThresholdOp::Lte,
                value: 3.0,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    // Additional Ensure (6 more = 14 total)
    claims.extend([
        ContractClause {
            id: "ensure.coverage_gaming".to_string(),
            kind: ClauseKind::Ensure,
            description: "No coverage exclusion gaming".to_string(),
            falsification_method: FalsificationMethod::CoverageGaming,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.spec_quality".to_string(),
            kind: ClauseKind::Ensure,
            description: "Spec score meets threshold".to_string(),
            falsification_method: FalsificationMethod::SpecQuality,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "spec_score".to_string(),
                op: ThresholdOp::Gte,
                value: 95.0,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.book_validation".to_string(),
            kind: ClauseKind::Ensure,
            description: "pmat-book validation passes".to_string(),
            falsification_method: FalsificationMethod::BookValidation,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.per_file_coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: format!("All files >= {}% coverage", coverage_pct),
            falsification_method: FalsificationMethod::PerFileCoverage,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "per_file_coverage_pct".to_string(),
                op: ThresholdOp::Gte,
                value: coverage_pct,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.variant_coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: "All match arm variants tested".to_string(),
            falsification_method: FalsificationMethod::VariantCoverage,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.cross_crate_parity".to_string(),
            kind: ClauseKind::Ensure,
            description: "Cross-crate integration tests pass".to_string(),
            falsification_method: FalsificationMethod::CrossCrateParity,
            threshold: None,
            blocking: false, // Not blocking by default
            source: ClauseSource::Default,
        },
    ]);

    claims
}
