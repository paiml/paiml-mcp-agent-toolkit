/// Composite complexity score for ranking files
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositeComplexityScore {
    pub cyclomatic_max: u32,
    pub cognitive_avg: f64,
    pub halstead_effort: f64,
    pub function_count: usize,
    pub total_score: f64,
}

impl Default for CompositeComplexityScore {
    fn default() -> Self {
        Self {
            cyclomatic_max: 0,
            cognitive_avg: 0.0,
            halstead_effort: 0.0,
            function_count: 0,
            total_score: 0.0,
        }
    }
}

impl PartialEq for CompositeComplexityScore {
    fn eq(&self, other: &Self) -> bool {
        (self.total_score - other.total_score).abs() < f64::EPSILON
    }
}

impl PartialOrd for CompositeComplexityScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.total_score.partial_cmp(&other.total_score)
    }
}

/// Churn score for ranking files by change frequency
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChurnScore {
    pub commit_count: usize,
    pub unique_authors: usize,
    pub lines_changed: usize,
    pub recency_weight: f64,
    pub score: f64,
}

impl Default for ChurnScore {
    fn default() -> Self {
        Self {
            commit_count: 0,
            unique_authors: 0,
            lines_changed: 0,
            recency_weight: 0.0,
            score: 0.0,
        }
    }
}

impl PartialEq for ChurnScore {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < f64::EPSILON
    }
}

impl PartialOrd for ChurnScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/// Duplication score for ranking files by code duplication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicationScore {
    pub exact_clones: usize,
    pub renamed_clones: usize,
    pub gapped_clones: usize,
    pub semantic_clones: usize,
    pub duplication_ratio: f64,
    pub score: f64,
}

impl Default for DuplicationScore {
    fn default() -> Self {
        Self {
            exact_clones: 0,
            renamed_clones: 0,
            gapped_clones: 0,
            semantic_clones: 0,
            duplication_ratio: 0.0,
            score: 0.0,
        }
    }
}

impl PartialEq for DuplicationScore {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < f64::EPSILON
    }
}

impl PartialOrd for DuplicationScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/// SIMD-optimized ranking for large datasets
/// Ranks files by their scores using vectorized operations
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::ranking::rank_files_vectorized;
///
/// let scores = vec![0.8, 0.2, 0.9, 0.1];
/// let ranked = rank_files_vectorized(&scores, 2);
///
/// assert_eq!(ranked.len(), 2);
/// assert_eq!(ranked[0], 2); // Index of highest score (0.9)
/// assert_eq!(ranked[1], 0); // Index of second highest (0.8)
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn rank_files_vectorized(scores: &[f32], limit: usize) -> Vec<usize> {
    debug_assert!(limit > 0, "limit must be positive");
    let mut indices: Vec<usize> = (0..scores.len()).collect();

    // For large datasets, use parallel sorting
    if scores.len() > 1024 {
        indices.par_sort_unstable_by(|&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap_or(Ordering::Equal)
        });
    } else {
        // Standard sort for smaller datasets
        indices.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap_or(Ordering::Equal));
    }

    indices.truncate(limit);
    indices
}

