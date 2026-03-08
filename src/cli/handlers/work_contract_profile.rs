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
        ContractProfile::Stack { manifest_path } => {
            // §2.6: Load manifest, resolve extends field, merge base + stack claims
            match std::fs::read_to_string(manifest_path) {
                Ok(content) => match StackManifest::parse(&content) {
                    Ok(manifest) => manifest.resolve_base_claims(config),
                    Err(_) => universal_claims(max_file_lines),
                },
                Err(_) => universal_claims(max_file_lines),
            }
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
