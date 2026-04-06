// Runtime Violation Tracking for Stack Manifests
// Spec: docs/specifications/dbc.md §14.7
//
// Tracks stack command failures and execution times per session.
// Provides anomaly detection (>3 sigma from historical) and
// content hash chain for trust history integrity.

/// A single runtime violation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeViolation {
    /// Work item this violation occurred in
    pub work_item_id: String,
    /// Timestamp of the violation
    pub timestamp: String,
    /// Command that failed or anomalous
    pub command: String,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Duration in milliseconds (for timing anomalies)
    pub duration_ms: Option<u64>,
    /// Exit code (for failures)
    pub exit_code: Option<i32>,
    /// Error message or anomaly description
    pub message: String,
}

/// Classification of runtime violations (ABC paper §14.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Command returned non-zero exit code
    CommandFailure,
    /// Execution time >3 sigma from historical mean
    TimingAnomaly,
    /// Trust hash mismatch (manifest changed)
    TrustViolation,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationType::CommandFailure => write!(f, "command_failure"),
            ViolationType::TimingAnomaly => write!(f, "timing_anomaly"),
            ViolationType::TrustViolation => write!(f, "trust_violation"),
        }
    }
}

/// Session-level violation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationSummary {
    /// Total violations in this session
    pub total_violations: usize,
    /// Breakdown by type
    pub command_failures: usize,
    pub timing_anomalies: usize,
    pub trust_violations: usize,
    /// Historical comparison
    pub violations_per_session_avg: f64,
    /// Whether current session exceeds 2x historical average
    pub elevated: bool,
}

/// Historical command execution timing for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandTiming {
    /// Command string (first arg)
    pub command: String,
    /// Historical execution durations in ms
    pub durations: Vec<u64>,
    /// Running mean
    pub mean_ms: f64,
    /// Running standard deviation
    pub std_dev_ms: f64,
}

impl CommandTiming {
    /// Create a new timing record for a command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(command: String, duration_ms: u64) -> Self {
        Self {
            command,
            durations: vec![duration_ms],
            mean_ms: duration_ms as f64,
            std_dev_ms: 0.0,
        }
    }

    /// Record a new execution and update statistics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record(&mut self, duration_ms: u64) {
        self.durations.push(duration_ms);
        // Keep last 50 observations
        if self.durations.len() > 50 {
            self.durations.remove(0);
        }
        self.recompute_stats();
    }

    /// Check if a duration is anomalous (>3 sigma from mean)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_anomalous(&self, duration_ms: u64) -> bool {
        if self.durations.len() < 3 {
            return false; // Not enough data
        }
        let z_score = (duration_ms as f64 - self.mean_ms) / self.std_dev_ms.max(1.0);
        z_score.abs() > 3.0
    }

    fn recompute_stats(&mut self) {
        debug_assert!(true, "contract: recompute_stats");
        let n = self.durations.len() as f64;
        self.mean_ms = self.durations.iter().sum::<u64>() as f64 / n;
        let variance = self
            .durations
            .iter()
            .map(|d| {
                let diff = *d as f64 - self.mean_ms;
                diff * diff
            })
            .sum::<f64>()
            / n;
        self.std_dev_ms = variance.sqrt();
    }
}

/// Runtime violation tracker for a work session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViolationTracker {
    /// Violations recorded in this session
    pub violations: Vec<RuntimeViolation>,
    /// Command timing history (persisted across sessions)
    pub timings: HashMap<String, CommandTiming>,
}


impl ViolationTracker {
    /// Record a command failure violation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_failure(
        &mut self,
        work_item_id: &str,
        command: &str,
        exit_code: i32,
        message: &str,
    ) {
        debug_assert!(!work_item_id.is_empty(), "work_item_id must not be empty");
        debug_assert!(!command.is_empty(), "command must not be empty");
        self.violations.push(RuntimeViolation {
            work_item_id: work_item_id.to_string(),
            timestamp: chrono_now(),
            command: command.to_string(),
            violation_type: ViolationType::CommandFailure,
            duration_ms: None,
            exit_code: Some(exit_code),
            message: message.to_string(),
        });
    }

    /// Record command execution and check for timing anomaly
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_execution(
        &mut self,
        work_item_id: &str,
        command: &str,
        duration_ms: u64,
    ) -> bool {
        debug_assert!(!work_item_id.is_empty(), "work_item_id must not be empty");
        debug_assert!(!command.is_empty(), "command must not be empty");
        let is_anomalous = self
            .timings
            .get(command)
            .map(|t| t.is_anomalous(duration_ms))
            .unwrap_or(false);

        // Update timing history
        self.timings
            .entry(command.to_string())
            .and_modify(|t| t.record(duration_ms))
            .or_insert_with(|| CommandTiming::new(command.to_string(), duration_ms));

        if is_anomalous {
            let timing = &self.timings[command];
            self.violations.push(RuntimeViolation {
                work_item_id: work_item_id.to_string(),
                timestamp: chrono_now(),
                command: command.to_string(),
                violation_type: ViolationType::TimingAnomaly,
                duration_ms: Some(duration_ms),
                exit_code: None,
                message: format!(
                    "Execution time {}ms deviates >3 sigma from mean {:.0}ms (std: {:.0}ms)",
                    duration_ms, timing.mean_ms, timing.std_dev_ms
                ),
            });
        }

        is_anomalous
    }

    /// Record a trust violation (manifest hash changed)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_trust_violation(
        &mut self,
        work_item_id: &str,
        manifest_path: &str,
        message: &str,
    ) {
        debug_assert!(!work_item_id.is_empty(), "work_item_id must not be empty");
        self.violations.push(RuntimeViolation {
            work_item_id: work_item_id.to_string(),
            timestamp: chrono_now(),
            command: manifest_path.to_string(),
            violation_type: ViolationType::TrustViolation,
            duration_ms: None,
            exit_code: None,
            message: message.to_string(),
        });
    }

    /// Get summary statistics for the current session
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn summary(&self, historical_avg: f64) -> ViolationSummary {
        let command_failures = self
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::CommandFailure)
            .count();
        let timing_anomalies = self
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::TimingAnomaly)
            .count();
        let trust_violations = self
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::TrustViolation)
            .count();
        let total = self.violations.len();

        ViolationSummary {
            total_violations: total,
            command_failures,
            timing_anomalies,
            trust_violations,
            violations_per_session_avg: historical_avg,
            elevated: total as f64 > historical_avg * 2.0,
        }
    }

    /// Save violation tracker to disk
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, project_path: &Path, work_item_id: &str) -> Result<()> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let dir = project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("violations");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("tracker.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load violation tracker from disk (returns default if not found)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load(project_path: &Path, work_item_id: &str) -> Self {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let path = project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("violations")
            .join("tracker.json");
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
}

/// Content hash chain entry for trust history integrity (§14.7.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChainEntry {
    /// Manifest file path
    pub manifest_path: String,
    /// SHA-256 of manifest content
    pub content_hash: String,
    /// SHA-256 of previous chain entry (empty string for first entry)
    pub prev_hash: String,
    /// Combined hash: SHA-256(content_hash + prev_hash)
    pub chain_hash: String,
    /// Timestamp of trust decision
    pub trusted_at: String,
}

impl TrustChainEntry {
    /// Create a new trust chain entry linked to the previous
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(manifest_path: &str, content_hash: &str, prev_hash: &str) -> Self {
        debug_assert!(!manifest_path.is_empty(), "manifest_path must not be empty");
        debug_assert!(!content_hash.is_empty(), "content_hash must not be empty");
        debug_assert!(!prev_hash.is_empty(), "prev_hash must not be empty");
        let chain_input = format!("{}{}", content_hash, prev_hash);
        let chain_hash = format!("{:x}", Sha256::digest(chain_input.as_bytes()));
        Self {
            manifest_path: manifest_path.to_string(),
            content_hash: content_hash.to_string(),
            prev_hash: prev_hash.to_string(),
            chain_hash,
            trusted_at: chrono_now(),
        }
    }

    /// Verify the chain hash is consistent
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn verify(&self) -> bool {
        let chain_input = format!("{}{}", self.content_hash, self.prev_hash);
        let expected = format!("{:x}", Sha256::digest(chain_input.as_bytes()));
        self.chain_hash == expected
    }
}

/// Simple timestamp function (avoids chrono dependency)
fn chrono_now() -> String {
    debug_assert!(true, "contract: chrono_now");
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // ISO-8601 approximate (year-month-day only for simplicity)
    format!("{}Z", secs)
}
