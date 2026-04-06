// Contract Scoring: 5-dimension quality scoring for pmat work contracts
// Spec: docs/specifications/dbc.md §13.4-13.5
//
// Adapted from provable-contracts scoring (spec_depth, falsification, kani, lean, binding)
// to pmat work contract dimensions (spec_depth, falsification, invariant_health,
// subcontracting, traceability).

/// 5-dimension contract quality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractScore {
    /// How comprehensive are the contract clauses? (weight: 0.20)
    pub spec_depth: f64,
    /// What fraction of claims have been verified? (weight: 0.25)
    pub falsification_coverage: f64,
    /// Invariant pass rate across checkpoints (weight: 0.25)
    pub invariant_health: f64,
    /// Monotonic postcondition strengthening score (weight: 0.10)
    pub subcontracting: f64,
    /// Coverage of require/ensure/invariant triad (weight: 0.20)
    pub traceability: f64,
    /// Weighted total score (0.0 to 1.0)
    pub total: f64,
    /// Letter grade (A-F)
    pub grade: ScoreGrade,
}

/// Scoring weights (sum = 1.0)
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub spec_depth: f64,
    pub falsification: f64,
    pub invariant_health: f64,
    pub subcontracting: f64,
    pub traceability: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            spec_depth: 0.20,
            falsification: 0.25,
            invariant_health: 0.25,
            subcontracting: 0.10,
            traceability: 0.20,
        }
    }
}

/// Letter grades for contract scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreGrade {
    A,
    B,
    C,
    D,
    F,
}

impl ScoreGrade {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_score(score: f64) -> Self {
        if score >= 0.90 {
            ScoreGrade::A
        } else if score >= 0.75 {
            ScoreGrade::B
        } else if score >= 0.60 {
            ScoreGrade::C
        } else if score >= 0.40 {
            ScoreGrade::D
        } else {
            ScoreGrade::F
        }
    }
}

impl std::fmt::Display for ScoreGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScoreGrade::A => write!(f, "A"),
            ScoreGrade::B => write!(f, "B"),
            ScoreGrade::C => write!(f, "C"),
            ScoreGrade::D => write!(f, "D"),
            ScoreGrade::F => write!(f, "F"),
        }
    }
}

/// Compute the 5-dimension contract score for a work contract.
///
/// Each dimension is scored 0.0..1.0, then weighted and summed.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn score_contract(contract: &WorkContract, project_path: &Path) -> ContractScore {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let weights = ScoringWeights::default();

    let spec_depth = compute_spec_depth(contract);
    let falsification_coverage = compute_falsification_coverage(contract);
    let invariant_health = compute_invariant_health(contract, project_path);
    let subcontracting = compute_subcontracting_score(contract);
    let traceability = compute_traceability(contract);

    let total = weights.spec_depth * spec_depth
        + weights.falsification * falsification_coverage
        + weights.invariant_health * invariant_health
        + weights.subcontracting * subcontracting
        + weights.traceability * traceability;

    // All dimensions must be in [0.0, 1.0]
    debug_assert!((0.0..=1.0).contains(&spec_depth), "spec_depth out of range: {}", spec_depth);
    debug_assert!((0.0..=1.0).contains(&falsification_coverage), "falsification out of range: {}", falsification_coverage);
    debug_assert!((0.0..=1.0).contains(&invariant_health), "invariant_health out of range: {}", invariant_health);
    debug_assert!((0.0..=1.0).contains(&subcontracting), "subcontracting out of range: {}", subcontracting);
    debug_assert!((0.0..=1.0).contains(&traceability), "traceability out of range: {}", traceability);
    debug_assert!((0.0..=1.0).contains(&total), "total score out of range: {}", total);

    let grade = ScoreGrade::from_score(total);

    ContractScore {
        spec_depth,
        falsification_coverage,
        invariant_health,
        subcontracting,
        traceability,
        total,
        grade,
    }
}

/// Spec depth: ratio of defined clauses to expected minimum.
///
/// A well-specified contract has at least 3 require, 5 ensure, 3 invariant clauses.
fn compute_spec_depth(contract: &WorkContract) -> f64 {
    if !contract.is_dbc() {
        // v4.0 flat contracts: score based on claim count vs expected 22
        let ratio = contract.claims.len() as f64 / 22.0;
        return ratio.min(1.0);
    }

    let require_score = (contract.require.len() as f64 / 3.0).min(1.0);
    let ensure_score = (contract.ensure.len() as f64 / 5.0).min(1.0);
    let invariant_score = (contract.invariant.len() as f64 / 3.0).min(1.0);

    (require_score + ensure_score + invariant_score) / 3.0
}

/// Falsification coverage: fraction of claims with verification results.
fn compute_falsification_coverage(contract: &WorkContract) -> f64 {
    if contract.claims.is_empty() {
        return 0.0;
    }

    let verified = contract
        .claims
        .iter()
        .filter(|c| c.result.is_some())
        .count();
    verified as f64 / contract.claims.len() as f64
}

/// Invariant health: pass rate across checkpoint history.
///
/// Loads all checkpoints and computes the fraction of invariants that passed.
fn compute_invariant_health(contract: &WorkContract, project_path: &Path) -> f64 {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let checkpoints = CheckpointRecord::load_all(project_path, &contract.work_item_id);
    if checkpoints.is_empty() {
        // No checkpoints yet — neutral score (not penalized)
        return if contract.invariant.is_empty() {
            1.0
        } else {
            0.5
        };
    }

    let total_invariants: usize = checkpoints
        .iter()
        .map(|cp| cp.invariant_results.len())
        .sum();
    if total_invariants == 0 {
        return 1.0;
    }

    let passed: usize = checkpoints
        .iter()
        .flat_map(|cp| &cp.invariant_results)
        .filter(|r| r.passed)
        .count();

    passed as f64 / total_invariants as f64
}

/// Subcontracting score: 1.0 if postconditions are monotonically non-weakening.
fn compute_subcontracting_score(contract: &WorkContract) -> f64 {
    if contract.iteration <= 1 || contract.inherited_postconditions.is_empty() {
        return 1.0; // First iteration or no inherited — full score
    }

    match validate_subcontracting(&contract.inherited_postconditions, &contract.ensure) {
        Ok(()) => 1.0,
        Err(_) => 0.0,
    }
}

/// Traceability: coverage of the require/ensure/invariant triad.
///
/// Full traceability = all three triad legs are non-empty.
fn compute_traceability(contract: &WorkContract) -> f64 {
    if !contract.is_dbc() {
        // v4.0: traceability is binary — claims exist or not
        return if contract.claims.is_empty() { 0.0 } else { 0.8 };
    }

    let mut score: f64 = 0.0;
    if !contract.require.is_empty() {
        score += 1.0 / 3.0;
    }
    if !contract.ensure.is_empty() {
        score += 1.0 / 3.0;
    }
    if !contract.invariant.is_empty() {
        score += 1.0 / 3.0;
    }

    // Bonus for exclusion transparency (up to 1.0 total)
    if !contract.excluded_claims.is_empty() && score > 0.0 {
        score = (score + 0.05).min(1.0);
    }

    score
}

// === Drift Detection (DBC spec §13.5, §14.3) ===

/// Drift metrics for a contract — measures staleness and divergence.
///
/// Based on ABC drift bounds theorem (arXiv:2602.22302):
/// D* = alpha / gamma, where alpha = drift rate, gamma = recovery rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMetrics {
    /// Hours since last checkpoint
    pub hours_since_checkpoint: f64,
    /// Hours since contract creation
    pub hours_since_creation: f64,
    /// Estimated drift rate alpha (0.0..1.0)
    pub drift_rate: f64,
    /// Recovery rate gamma from checkpoint frequency
    pub recovery_rate: f64,
    /// Bounded drift D* = alpha / gamma (lower is better)
    pub bounded_drift: f64,
    /// Whether the contract is considered stale
    pub is_stale: bool,
}

/// Compute drift metrics for a contract.
///
/// Staleness threshold: 24 hours without a checkpoint.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn compute_drift_metrics(contract: &WorkContract, project_path: &Path) -> DriftMetrics {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let now = chrono::Utc::now();
    let hours_since_creation = (now - contract.created_at).num_minutes() as f64 / 60.0;

    let checkpoints = CheckpointRecord::load_all(project_path, &contract.work_item_id);
    let hours_since_checkpoint = if let Some(last) = checkpoints.last() {
        (now - last.timestamp).num_minutes() as f64 / 60.0
    } else {
        hours_since_creation
    };

    // Drift rate: increases with time since last checkpoint
    // alpha = min(1.0, hours_since_checkpoint / 48.0)
    let drift_rate = (hours_since_checkpoint / 48.0).min(1.0);

    // Recovery rate: based on checkpoint frequency
    // gamma = checkpoints_per_24h (capped at 1.0)
    let recovery_rate = if hours_since_creation > 0.0 {
        let cps_per_day = (checkpoints.len() as f64 / hours_since_creation) * 24.0;
        cps_per_day.min(1.0).max(0.01) // floor at 0.01 to avoid div-by-zero
    } else {
        0.01
    };

    // ABC theorem: D* = alpha / gamma
    let bounded_drift = (drift_rate / recovery_rate).min(1.0);
    let is_stale = hours_since_checkpoint > 24.0;

    DriftMetrics {
        hours_since_checkpoint,
        hours_since_creation,
        drift_rate,
        recovery_rate,
        bounded_drift,
        is_stale,
    }
}

// === Trend Tracking (DBC spec §13.6) ===

/// Point-in-time quality snapshot for trend tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrendSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Contract score at this point
    pub score: f64,
    /// Grade at this point
    pub grade: ScoreGrade,
    /// Number of active claims
    pub active_claims: usize,
    /// Number of verified claims
    pub verified_claims: usize,
    /// Drift bound at this point
    pub bounded_drift: f64,
    /// Git SHA at snapshot time
    pub git_sha: String,
}

/// Trend analysis result with drift detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    /// All snapshots, sorted chronologically
    pub snapshots: Vec<QualityTrendSnapshot>,
    /// Rolling average score (7-snapshot window)
    pub rolling_average: f64,
    /// Score delta from rolling average
    pub delta_from_average: f64,
    /// Whether drift is detected (>5% drop from rolling average)
    pub drift_detected: bool,
    /// Trend direction
    pub direction: TrendDirection,
}

/// Trend direction indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::Improving => write!(f, "improving"),
            TrendDirection::Stable => write!(f, "stable"),
            TrendDirection::Declining => write!(f, "declining"),
        }
    }
}

/// Record a quality trend snapshot to .pmat-work/{item-id}/trend/
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn record_trend_snapshot(
    contract: &WorkContract,
    score: &ContractScore,
    drift: &DriftMetrics,
    git_sha: &str,
    project_path: &Path,
) -> Result<PathBuf> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let verified = contract
        .claims
        .iter()
        .filter(|c| c.result.is_some())
        .count();

    let snapshot = QualityTrendSnapshot {
        timestamp: chrono::Utc::now(),
        score: score.total,
        grade: score.grade,
        active_claims: contract.claims.len(),
        verified_claims: verified,
        bounded_drift: drift.bounded_drift,
        git_sha: git_sha.to_string(),
    };

    let trend_dir = project_path
        .join(".pmat-work")
        .join(&contract.work_item_id)
        .join("trend");
    std::fs::create_dir_all(&trend_dir)?;

    let filename = format!(
        "snapshot-{}.json",
        snapshot.timestamp.format("%Y%m%dT%H%M%S")
    );
    let path = trend_dir.join(filename);
    let json = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(&path, json)?;

    Ok(path)
}

/// Load and analyze quality trend for a work item.
///
/// Uses a 7-snapshot rolling window. Drift is detected when the current
/// score drops more than 5% below the rolling average.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn load_quality_trend(project_path: &Path, work_item_id: &str) -> QualityTrend {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let trend_dir = project_path
        .join(".pmat-work")
        .join(work_item_id)
        .join("trend");

    let mut snapshots = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&trend_dir) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(snap) = serde_json::from_str::<QualityTrendSnapshot>(&content) {
                    snapshots.push(snap);
                }
            }
        }
    }
    snapshots.sort_by_key(|s| s.timestamp);

    analyze_trend(snapshots)
}

/// Analyze a list of snapshots into a QualityTrend.
fn analyze_trend(snapshots: Vec<QualityTrendSnapshot>) -> QualityTrend {
    if snapshots.is_empty() {
        return QualityTrend {
            snapshots,
            rolling_average: 0.0,
            delta_from_average: 0.0,
            drift_detected: false,
            direction: TrendDirection::Stable,
        };
    }

    // Rolling average over last 7 snapshots
    let window_size = 7.min(snapshots.len());
    let window_start = snapshots.len() - window_size;
    let rolling_sum: f64 = snapshots[window_start..].iter().map(|s| s.score).sum();
    let rolling_average = rolling_sum / window_size as f64;

    let current_score = snapshots.last().map(|s| s.score).unwrap_or(0.0);
    let delta_from_average = current_score - rolling_average;

    // Drift: >5% drop from rolling average
    let drift_detected = delta_from_average < -0.05;

    // Direction based on last 2 snapshots
    let direction = if snapshots.len() < 2 {
        TrendDirection::Stable
    } else {
        let prev = snapshots[snapshots.len() - 2].score;
        let diff = current_score - prev;
        if diff > 0.02 {
            TrendDirection::Improving
        } else if diff < -0.02 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    };

    QualityTrend {
        snapshots,
        rolling_average,
        delta_from_average,
        drift_detected,
        direction,
    }
}
