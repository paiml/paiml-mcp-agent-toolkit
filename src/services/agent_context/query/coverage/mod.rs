#![cfg_attr(coverage_nightly, coverage(off))]

pub(crate) mod enrichment;
mod loader;
pub(super) mod parsing;
pub(super) mod profdata;
pub(super) mod types;

#[cfg(test)]
mod tests;

pub use enrichment::{
    compute_impact_score, enrich_with_coverage, enrich_with_coverage_diff, format_coverage_summary,
};
pub use loader::{enrich_results_with_coverage, load_workspace_coverage};
pub use parsing::build_coverage_map;
