#![cfg_attr(coverage_nightly, coverage(off))]

use std::collections::{BTreeMap, HashSet};

use super::grade::{MetricCategory, PenaltyAttribution};

/// Penalty tracker.
///
/// Keyed by issue id in a BTreeMap so `get_attributions()` returns a
/// deterministic order — penalties land in serialized TDG scores (and
/// baselines), where HashMap iteration order made output byte-unstable.
pub struct PenaltyTracker {
    applied: BTreeMap<String, PenaltyAttribution>,
}

impl Default for PenaltyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PenaltyTracker {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            applied: BTreeMap::new(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Apply the operation.
    pub fn apply(
        &mut self,
        issue_id: String,
        category: MetricCategory,
        amount: f32,
        issue: String,
    ) -> Option<f32> {
        if self.applied.contains_key(&issue_id) {
            return None;
        }

        self.applied.insert(
            issue_id,
            PenaltyAttribution {
                source_metric: category,
                amount,
                applied_to: HashSet::from([category]),
                issue,
            },
        );

        Some(amount)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get attributions.
    pub fn get_attributions(&self) -> Vec<PenaltyAttribution> {
        self.applied.values().cloned().collect()
    }
}
