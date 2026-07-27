#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::project_meta::{BuildInfo, ProjectOverview};
use crate::services::deep_context::DeepContext;
use std::fmt::Write;

include!("formatting_helpers_formatters.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    include!("formatting_helpers_tests.rs");
}
