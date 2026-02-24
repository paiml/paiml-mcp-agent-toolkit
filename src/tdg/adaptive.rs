use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

include!("adaptive_types.rs");
include!("adaptive_manager.rs");
include!("adaptive_factory.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("adaptive_tests_core.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    include!("adaptive_tests_property.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod extended_coverage_tests {
    use super::*;

    include!("adaptive_tests_extended.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    include!("adaptive_tests_edge.rs");
}
