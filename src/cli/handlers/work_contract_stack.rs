// Stack Manifests: third-party tool stacks for Design by Contract
// Spec: docs/specifications/dbc.md §2.5

/// Parsed stack manifest from .dbc-stack.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackManifest {
    /// Stack metadata
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    /// Optional base profile to extend
    pub extends: Option<String>,

    /// Claim definitions
    pub require_claims: Vec<StackClaim>,
    pub invariant_claims: Vec<StackClaim>,
    pub ensure_claims: Vec<StackClaim>,

    /// Rescue strategies
    pub rescue_blocks: Vec<StackRescue>,

    /// Default thresholds
    pub thresholds: HashMap<String, f64>,
}

/// A single claim from a stack manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackClaim {
    pub id: String,
    pub description: String,
    pub check: String,
    pub timeout: Option<u64>,
    pub threshold: Option<StackThreshold>,
    pub metric_pattern: Option<String>,
    pub metric_on_no_match: Option<String>,
}

/// Threshold from TOML inline table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackThreshold {
    pub metric: String,
    pub op: String,
    pub value: f64,
}

/// Rescue block from stack manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackRescue {
    pub for_clause: String,
    pub strategy: String,
    pub command: Option<String>,
    pub guidance: Option<String>,
    pub description: Option<String>,
}

/// Trust record for a stack manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTrustRecord {
    pub sha256: String,
    pub trusted_at: String,
    pub trusted_by: String,
    pub commands_reviewed: usize,
}

/// Command restriction violation
#[derive(Debug, Clone, PartialEq)]
pub enum CommandRestriction {
    PipeToShell,
    BacktickSubstitution,
    DollarSubstitution,
    NetworkFetchExecute,
    RedirectToExecutable,
}

impl std::fmt::Display for CommandRestriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandRestriction::PipeToShell => write!(f, "pipe to shell (| sh/bash)"),
            CommandRestriction::BacktickSubstitution => write!(f, "backtick substitution"),
            CommandRestriction::DollarSubstitution => write!(f, "$() command substitution"),
            CommandRestriction::NetworkFetchExecute => write!(f, "network fetch + execute"),
            CommandRestriction::RedirectToExecutable => write!(f, "redirect to executable"),
        }
    }
}

/// Validate a shell command against security restrictions.
/// Returns None if safe, or the restriction violated.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn validate_command(cmd: &str) -> Option<CommandRestriction> {
    // Backtick substitution
    if cmd.contains('`') {
        return Some(CommandRestriction::BacktickSubstitution);
    }

    // $() substitution
    if cmd.contains("$(") {
        return Some(CommandRestriction::DollarSubstitution);
    }

    // Pipe to shell: | sh, | bash, | zsh
    let pipe_shell = regex::Regex::new(r"\|\s*(sh|bash|zsh|dash)\b").ok();
    if let Some(re) = &pipe_shell {
        if re.is_match(cmd) {
            return Some(CommandRestriction::PipeToShell);
        }
    }

    // Network fetch + execute patterns
    let net_exec_patterns = [
        r"(curl|wget)\s+.*\|\s*(sh|bash|python|perl|ruby)",
        r"(curl|wget)\s+-[^\s]*O\s*-\s*.*\|\s*",
    ];
    for pat in &net_exec_patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            if re.is_match(cmd) {
                return Some(CommandRestriction::NetworkFetchExecute);
            }
        }
    }

    // Redirect to executable + chmod +x
    if cmd.contains("chmod +x") && (cmd.contains("curl") || cmd.contains("wget")) {
        return Some(CommandRestriction::RedirectToExecutable);
    }

    None
}

impl StackManifest {
    /// Parse a .dbc-stack.toml file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse(content: &str) -> Result<Self> {
        let table: toml::Table = content.parse().context("Failed to parse .dbc-stack.toml")?;

        let stack = table
            .get("stack")
            .and_then(|v| v.as_table())
            .context("Missing [stack] section")?;

        let name = stack
            .get("name")
            .and_then(|v| v.as_str())
            .context("Missing stack.name")?
            .to_string();
        let version = stack
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();
        let description = stack.get("description").and_then(|v| v.as_str()).map(String::from);
        let extends = stack.get("extends").and_then(|v| v.as_str()).map(String::from);

        let require_claims = Self::parse_claims(&table, "require")?;
        let invariant_claims = Self::parse_claims(&table, "invariant")?;
        let ensure_claims = Self::parse_claims(&table, "ensure")?;
        let rescue_blocks = Self::parse_rescues(&table)?;

        let thresholds = table
            .get("thresholds")
            .and_then(|v| v.as_table())
            .map(|t| {
                t.iter()
                    .filter_map(|(k, v)| v.as_float().map(|f| (k.clone(), f)))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            name,
            version,
            description,
            extends,
            require_claims,
            invariant_claims,
            ensure_claims,
            rescue_blocks,
            thresholds,
        })
    }

    fn parse_claims(table: &toml::Table, kind: &str) -> Result<Vec<StackClaim>> {
        let arr = match table.get(kind) {
            Some(toml::Value::Array(a)) => a,
            _ => return Ok(vec![]),
        };

        arr.iter()
            .map(|entry| {
                let t = entry.as_table().context("Claim must be a table")?;
                let id = t.get("id").and_then(|v| v.as_str()).context("Missing claim id")?.to_string();
                let description = t.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let check = t.get("check").and_then(|v| v.as_str()).context("Missing claim check command")?.to_string();
                let timeout = t.get("timeout").and_then(|v| v.as_integer()).map(|v| v as u64);
                let metric_pattern = t.get("metric_pattern").and_then(|v| v.as_str()).map(String::from);
                let metric_on_no_match = t.get("metric_on_no_match").and_then(|v| v.as_str()).map(String::from);

                let threshold = t.get("threshold").and_then(|v| v.as_table()).map(|th| {
                    StackThreshold {
                        metric: th.get("metric").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        op: th.get("op").and_then(|v| v.as_str()).unwrap_or("Gte").to_string(),
                        value: th.get("value").and_then(|v| v.as_float()).unwrap_or(0.0),
                    }
                });

                Ok(StackClaim { id, description, check, timeout, threshold, metric_pattern, metric_on_no_match })
            })
            .collect()
    }

    fn parse_rescues(table: &toml::Table) -> Result<Vec<StackRescue>> {
        let arr = match table.get("rescue") {
            Some(toml::Value::Array(a)) => a,
            _ => return Ok(vec![]),
        };

        arr.iter()
            .map(|entry| {
                let t = entry.as_table().context("Rescue must be a table")?;
                Ok(StackRescue {
                    for_clause: t.get("for_clause").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    strategy: t.get("strategy").and_then(|v| v.as_str()).unwrap_or("manual").to_string(),
                    command: t.get("command").and_then(|v| v.as_str()).map(String::from),
                    guidance: t.get("guidance").and_then(|v| v.as_str()).map(String::from),
                    description: t.get("description").and_then(|v| v.as_str()).map(String::from),
                })
            })
            .collect()
    }

    /// Validate all commands against security restrictions
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_commands(&self) -> Vec<(String, CommandRestriction)> {
        let mut violations = Vec::new();
        let all_claims = self.require_claims.iter()
            .chain(self.invariant_claims.iter())
            .chain(self.ensure_claims.iter());

        for claim in all_claims {
            if let Some(restriction) = validate_command(&claim.check) {
                violations.push((claim.id.clone(), restriction));
            }
        }

        for rescue in &self.rescue_blocks {
            if let Some(cmd) = &rescue.command {
                if let Some(restriction) = validate_command(cmd) {
                    violations.push((rescue.for_clause.clone(), restriction));
                }
            }
        }

        violations
    }

    /// Convert stack claims to ContractClause instances
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_contract_clauses(&self) -> Vec<ContractClause> {
        let mut clauses = Vec::new();
        let stack_name = self.name.clone();

        for claim in &self.require_claims {
            clauses.push(stack_claim_to_clause(claim, ClauseKind::Require, &stack_name));
        }
        for claim in &self.invariant_claims {
            clauses.push(stack_claim_to_clause(claim, ClauseKind::Invariant, &stack_name));
        }
        for claim in &self.ensure_claims {
            clauses.push(stack_claim_to_clause(claim, ClauseKind::Ensure, &stack_name));
        }

        clauses
    }

    /// Resolve the `extends` field into base profile claims.
    ///
    /// If `extends` is "rust", "universal", or "pmat", the base profile's
    /// claims are prepended before the stack's own claims. Stack claims
    /// with the same ID as a base claim override the base.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn resolve_base_claims(&self, config: &DbcConfig) -> Vec<ContractClause> {
        let base_claims = match self.extends.as_deref() {
            Some("universal") => claims_for_profile(&ContractProfile::Universal, config),
            Some("rust") => claims_for_profile(&ContractProfile::Rust, config),
            Some("pmat") => claims_for_profile(&ContractProfile::Pmat, config),
            _ => Vec::new(),
        };

        let stack_claims = self.to_contract_clauses();

        // Stack claims override base claims with the same ID
        let stack_ids: std::collections::HashSet<&str> =
            stack_claims.iter().map(|c| c.id.as_str()).collect();
        let mut merged: Vec<ContractClause> = base_claims
            .into_iter()
            .filter(|c| !stack_ids.contains(c.id.as_str()))
            .collect();
        merged.extend(stack_claims);
        merged
    }

    /// Total claim count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn claim_count(&self) -> usize {
        self.require_claims.len() + self.invariant_claims.len() + self.ensure_claims.len()
    }

    /// Get content hash for TOFU trust model
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

fn stack_claim_to_clause(claim: &StackClaim, kind: ClauseKind, stack_name: &str) -> ContractClause {
    let threshold = claim.threshold.as_ref().map(|t| {
        let op = match t.op.as_str() {
            "Gte" | ">=" => ThresholdOp::Gte,
            "Lte" | "<=" => ThresholdOp::Lte,
            "Gt" | ">" => ThresholdOp::Gt,
            "Lt" | "<" => ThresholdOp::Lt,
            "Eq" | "==" => ThresholdOp::Eq,
            _ => ThresholdOp::Gte,
        };
        ClauseThreshold::Numeric {
            metric: t.metric.clone(),
            op,
            value: t.value,
        }
    });

    ContractClause {
        id: claim.id.clone(),
        kind,
        description: claim.description.clone(),
        falsification_method: FalsificationMethod::MetaFalsification, // Stack claims use shell checks
        threshold,
        blocking: true,
        source: ClauseSource::Stack {
            manifest_name: stack_name.to_string(),
        },
    }
}

/// Load trust records from .pmat-work/trusted-stacks.json
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn load_trust_records(project_path: &Path) -> HashMap<String, StackTrustRecord> {
    let trust_path = project_path.join(".pmat-work").join("trusted-stacks.json");
    if let Ok(content) = std::fs::read_to_string(&trust_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

/// Save trust records to .pmat-work/trusted-stacks.json
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn save_trust_records(project_path: &Path, records: &HashMap<String, StackTrustRecord>) -> Result<()> {
    let trust_dir = project_path.join(".pmat-work");
    std::fs::create_dir_all(&trust_dir)?;
    let trust_path = trust_dir.join("trusted-stacks.json");
    let json = serde_json::to_string_pretty(records)?;
    std::fs::write(&trust_path, json)?;
    Ok(())
}

/// Check if a manifest is trusted (content hash matches)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn is_manifest_trusted(project_path: &Path, manifest_path: &str, content: &str) -> bool {
    let records = load_trust_records(project_path);
    if let Some(record) = records.get(manifest_path) {
        record.sha256 == StackManifest::content_hash(content)
    } else {
        false
    }
}

/// Execute a stack claim's check command (no sh -c, uses Command::new with arg splitting)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_stack_check(cmd: &str, project_path: &Path, timeout_secs: u64) -> Result<(bool, String)> {
    // Split on && for sequential commands
    let parts: Vec<&str> = cmd.split("&&").collect();

    let mut combined_output = String::new();
    for part in parts {
        let trimmed = part.trim();
        let args: Vec<&str> = trimmed.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        let result = std::process::Command::new(args[0])
            .args(&args[1..])
            .current_dir(project_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                combined_output.push_str(&stdout);
                combined_output.push_str(&stderr);
                if !output.status.success() {
                    return Ok((false, combined_output));
                }
            }
            Err(e) => {
                return Ok((false, format!("Command '{}' failed to execute: {}", args[0], e)));
            }
        }
    }

    let _ = timeout_secs; // Timeout handling would use tokio::time::timeout in async context
    Ok((true, combined_output))
}

/// Extract a metric value from command output using a regex pattern
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_metric(output: &str, pattern: &str) -> Result<Option<f64>> {
    let re = regex::Regex::new(pattern).context("Invalid metric_pattern regex")?;
    if let Some(captures) = re.captures(output) {
        if let Some(m) = captures.get(1) {
            let val: f64 = m.as_str().parse().context("Captured value is not a number")?;
            return Ok(Some(val));
        }
    }
    Ok(None)
}
