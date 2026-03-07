// Design by Contract types for pmat work (Meyer triad)
// Spec: docs/specifications/dbc.md

/// Contract clause — a single obligation in the Meyer triad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractClause {
    /// Unique identifier, e.g. "require.compiles", "ensure.coverage"
    pub id: String,

    /// Which part of the Meyer triad this belongs to
    pub kind: ClauseKind,

    /// Human-readable description of the obligation
    pub description: String,

    /// Reuse existing falsification method for evaluation
    pub falsification_method: FalsificationMethod,

    /// Optional numeric/boolean threshold for pass/fail
    pub threshold: Option<ClauseThreshold>,

    /// Jidoka: stop-the-line if violated?
    pub blocking: bool,

    /// How this clause was generated
    pub source: ClauseSource,
}

/// Meyer triad classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClauseKind {
    /// Client obligation — checked at work start
    Require,
    /// Supplier guarantee — checked at work complete
    Ensure,
    /// Always-true property — checked at every checkpoint
    Invariant,
}

impl std::fmt::Display for ClauseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClauseKind::Require => write!(f, "require"),
            ClauseKind::Ensure => write!(f, "ensure"),
            ClauseKind::Invariant => write!(f, "invariant"),
        }
    }
}

/// How a clause was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClauseSource {
    /// Auto-generated from project analysis
    Default,
    /// Inherited from prior iteration (subcontracting)
    Inherited { from_iteration: u32 },
    /// User-specified via config
    Manual,
    /// From a third-party stack manifest
    Stack { manifest_name: String },
}

/// Threshold types for contract clauses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClauseThreshold {
    /// Numeric threshold: metric op value (e.g., coverage >= 95.0)
    Numeric {
        metric: String,
        op: ThresholdOp,
        value: f64,
    },
    /// Boolean threshold: expected true/false
    Boolean { expected: bool },
    /// Delta threshold: relative to baseline (e.g., coverage delta >= 0.0)
    Delta {
        metric: String,
        op: ThresholdOp,
        value: f64,
    },
}

/// Comparison operators for thresholds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdOp {
    /// Greater than or equal
    Gte,
    /// Less than or equal
    Lte,
    /// Equal
    Eq,
    /// Greater than
    Gt,
    /// Less than
    Lt,
}

impl std::fmt::Display for ThresholdOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThresholdOp::Gte => write!(f, ">="),
            ThresholdOp::Lte => write!(f, "<="),
            ThresholdOp::Eq => write!(f, "=="),
            ThresholdOp::Gt => write!(f, ">"),
            ThresholdOp::Lt => write!(f, "<"),
        }
    }
}

/// Result of comparing two thresholds (for subcontracting validation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdComparison {
    /// Child threshold is stricter than parent
    Strengthened,
    /// Child threshold is weaker than parent
    Weakened,
    /// Child threshold is equivalent to parent
    Equal,
    /// Thresholds are incompatible (different types or operators)
    Incompatible,
}

/// Compare parent and child thresholds for subcontracting validation.
///
/// Returns whether the child threshold is strengthened, weakened, equal,
/// or incompatible relative to the parent. Used by `validate_subcontracting()`
/// and claim ID conflict resolution in profile composition.
pub fn compare_thresholds(
    parent: &Option<ClauseThreshold>,
    child: &Option<ClauseThreshold>,
) -> ThresholdComparison {
    match (parent, child) {
        (None, None) => ThresholdComparison::Equal,
        (None, Some(_)) => ThresholdComparison::Strengthened,
        (Some(_), None) => ThresholdComparison::Weakened,
        (Some(p), Some(c)) => compare_threshold_values(p, c),
    }
}

fn compare_threshold_values(parent: &ClauseThreshold, child: &ClauseThreshold) -> ThresholdComparison {
    match (parent, child) {
        (
            ClauseThreshold::Numeric {
                op: p_op,
                value: p_val,
                ..
            },
            ClauseThreshold::Numeric {
                op: c_op,
                value: c_val,
                ..
            },
        ) => compare_numeric(*p_op, *p_val, *c_op, *c_val),

        (
            ClauseThreshold::Delta {
                op: p_op,
                value: p_val,
                ..
            },
            ClauseThreshold::Delta {
                op: c_op,
                value: c_val,
                ..
            },
        ) => compare_numeric(*p_op, *p_val, *c_op, *c_val),

        (
            ClauseThreshold::Boolean { expected: p },
            ClauseThreshold::Boolean { expected: c },
        ) => match (p, c) {
            (true, true) | (false, false) => ThresholdComparison::Equal,
            (false, true) => ThresholdComparison::Strengthened,
            (true, false) => ThresholdComparison::Weakened,
        },

        // Different types are incompatible
        _ => ThresholdComparison::Incompatible,
    }
}

fn compare_numeric(
    p_op: ThresholdOp,
    p_val: f64,
    c_op: ThresholdOp,
    c_val: f64,
) -> ThresholdComparison {
    if p_op != c_op {
        return ThresholdComparison::Incompatible;
    }
    if p_op == ThresholdOp::Eq {
        return if (c_val - p_val).abs() < f64::EPSILON {
            ThresholdComparison::Equal
        } else {
            ThresholdComparison::Incompatible
        };
    }
    // For Gte/Gt: higher child = strengthened. For Lte/Lt: lower child = strengthened.
    let higher_is_stronger = matches!(p_op, ThresholdOp::Gte | ThresholdOp::Gt);
    compare_ordered(c_val, p_val, higher_is_stronger)
}

fn compare_ordered(child: f64, parent: f64, higher_is_stronger: bool) -> ThresholdComparison {
    #[allow(clippy::float_cmp)]
    if child == parent {
        return ThresholdComparison::Equal;
    }
    let child_higher = child > parent;
    if child_higher == higher_is_stronger {
        ThresholdComparison::Strengthened
    } else {
        ThresholdComparison::Weakened
    }
}

/// Record of an explicitly excluded claim (via --without)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludedClaim {
    /// Claim ID that was excluded
    pub id: String,
    /// Why it was excluded
    pub reason: String,
    /// CLI flag used
    pub flag: String,
}

/// Contract quality metric: active_claims / applicable_claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractQuality {
    /// Number of active claims (not excluded)
    pub active_claims: usize,
    /// Number of applicable claims for the profile
    pub applicable_claims: usize,
    /// Quality score (0.0 to 1.0)
    pub score: f64,
    /// Quality rating
    pub rating: String,
}

impl ContractQuality {
    pub fn calculate(active: usize, applicable: usize) -> Self {
        let score = if applicable == 0 {
            0.0
        } else {
            active as f64 / applicable as f64
        };
        let rating = Self::rate(score);
        Self { active_claims: active, applicable_claims: applicable, score, rating }
    }

    fn rate(score: f64) -> String {
        if score >= 1.0 { "Full" }
        else if score >= 0.8 { "Strong" }
        else if score >= 0.5 { "Partial" }
        else { "Weak" }
        .to_string()
    }
}

/// Subcontracting violation when child weakens parent postconditions
#[derive(Debug, Clone)]
pub enum SubcontractingViolation {
    /// A postcondition from the parent was dropped
    PostconditionDropped { clause: String },
    /// A postcondition from the parent was weakened
    PostconditionWeakened {
        clause: String,
        parent_threshold: Option<ClauseThreshold>,
        child_threshold: Option<ClauseThreshold>,
    },
    /// Thresholds are incompatible (cannot compare)
    IncompatibleThresholds {
        clause: String,
        parent_threshold: Option<ClauseThreshold>,
        child_threshold: Option<ClauseThreshold>,
    },
}

impl std::fmt::Display for SubcontractingViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubcontractingViolation::PostconditionDropped { clause } => {
                write!(f, "postcondition dropped: {clause}")
            }
            SubcontractingViolation::PostconditionWeakened { clause, .. } => {
                write!(f, "postcondition weakened: {clause}")
            }
            SubcontractingViolation::IncompatibleThresholds { clause, .. } => {
                write!(f, "incompatible thresholds: {clause}")
            }
        }
    }
}

/// Checkpoint record — captures invariant evaluation at a point in time (DbC §8.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRecord {
    /// Unique checkpoint ID
    pub checkpoint_id: String,
    /// Work item this checkpoint belongs to
    pub work_item_id: String,
    /// Timestamp of the checkpoint
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Git SHA at checkpoint time
    pub git_sha: String,
    /// Contract iteration number
    pub iteration: u32,
    /// Results of each invariant evaluation
    pub invariant_results: Vec<InvariantResult>,
    /// Whether all invariants held
    pub all_invariants_hold: bool,
    /// Drift bound D* at checkpoint time (DBC spec §13.5)
    #[serde(default)]
    pub drift_bound: Option<f64>,
}

/// Result of evaluating a single invariant clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantResult {
    /// Clause ID that was evaluated
    pub clause_id: String,
    /// Whether the invariant held
    pub passed: bool,
    /// Human-readable explanation
    pub explanation: String,
}

impl CheckpointRecord {
    /// Create a new checkpoint record from invariant evaluation results
    pub fn new(
        work_item_id: String,
        git_sha: String,
        iteration: u32,
        invariant_results: Vec<InvariantResult>,
    ) -> Self {
        let all_hold = invariant_results.iter().all(|r| r.passed);
        Self {
            checkpoint_id: uuid::Uuid::new_v4().to_string(),
            work_item_id,
            timestamp: chrono::Utc::now(),
            git_sha,
            iteration,
            invariant_results,
            all_invariants_hold: all_hold,
            drift_bound: None,
        }
    }

    /// Save checkpoint to .pmat-work/{item-id}/checkpoints/
    pub fn save(&self, project_path: &Path) -> Result<PathBuf> {
        let checkpoint_dir = project_path
            .join(".pmat-work")
            .join(&self.work_item_id)
            .join("checkpoints");
        std::fs::create_dir_all(&checkpoint_dir)?;

        let filename = format!("checkpoint-{}.json", self.checkpoint_id);
        let checkpoint_path = checkpoint_dir.join(filename);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&checkpoint_path, &json)?;
        Ok(checkpoint_path)
    }

    /// Load all checkpoints for a work item, sorted by timestamp
    pub fn load_all(project_path: &Path, work_item_id: &str) -> Vec<Self> {
        let checkpoint_dir = project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("checkpoints");

        let mut records = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&checkpoint_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(record) = serde_json::from_str::<CheckpointRecord>(&content) {
                        records.push(record);
                    }
                }
            }
        }
        records.sort_by_key(|r| r.timestamp);
        records
    }
}

/// Validate that a child contract does not weaken parent postconditions.
/// Returns Ok(()) if subcontracting rules hold, or the first violation found.
pub fn validate_subcontracting(
    parent_ensure: &[ContractClause],
    child_ensure: &[ContractClause],
) -> Result<(), SubcontractingViolation> {
    for parent_clause in parent_ensure {
        let child_clause = child_ensure.iter().find(|c| c.id == parent_clause.id);
        match child_clause {
            None => {
                return Err(SubcontractingViolation::PostconditionDropped {
                    clause: parent_clause.id.clone(),
                });
            }
            Some(child) => {
                match compare_thresholds(&parent_clause.threshold, &child.threshold) {
                    ThresholdComparison::Weakened => {
                        return Err(SubcontractingViolation::PostconditionWeakened {
                            clause: parent_clause.id.clone(),
                            parent_threshold: parent_clause.threshold.clone(),
                            child_threshold: child.threshold.clone(),
                        });
                    }
                    ThresholdComparison::Incompatible => {
                        return Err(SubcontractingViolation::IncompatibleThresholds {
                            clause: parent_clause.id.clone(),
                            parent_threshold: parent_clause.threshold.clone(),
                            child_threshold: child.threshold.clone(),
                        });
                    }
                    ThresholdComparison::Strengthened | ThresholdComparison::Equal => {}
                }
            }
        }
    }
    Ok(())
}
