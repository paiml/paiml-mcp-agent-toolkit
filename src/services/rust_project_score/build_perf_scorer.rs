#![cfg_attr(coverage_nightly, coverage(off))]
//! BuildPerfScorer - Build Performance Category (15 points)
//! Split for file health compliance (CB-040)

include!("build_perf_impl.rs");

#[cfg(test)]
#[path = "build_perf_tests.rs"]
mod tests;
