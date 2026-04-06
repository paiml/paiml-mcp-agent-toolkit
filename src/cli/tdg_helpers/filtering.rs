#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG hotspot filtering logic

use crate::models::tdg::TDGHotspot;

/// Filter TDG hotspots based on criteria
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn filter_tdg_hotspots(
    mut hotspots: Vec<TDGHotspot>,
    threshold: f64,
    top: usize,
    critical_only: bool,
) -> Vec<TDGHotspot> {
    // Apply threshold filter
    if threshold > 0.0 {
        hotspots.retain(|h| h.tdg_score >= threshold);
    }

    // Apply critical filter
    if critical_only {
        hotspots.retain(|h| h.tdg_score > 2.5);
    }

    // Apply top limit
    if top > 0 && hotspots.len() > top {
        hotspots.truncate(top);
    }

    hotspots
}
