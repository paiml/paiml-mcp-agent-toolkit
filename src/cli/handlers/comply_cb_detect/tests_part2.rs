#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;

// =============================================================================
// Tests for CB-1000 MLOps Model Quality
// =============================================================================

mod cb1000_model_tests {
    use super::*;
    use tempfile::TempDir;

    include!("tests_part2_cb1000_mlops.rs");
}

// =============================================================================
// Tests for CB-800 Scala Best Practices
// =============================================================================

mod cb800_scala_tests {
    use super::*;
    use tempfile::TempDir;

    include!("tests_part2_cb800_scala.rs");
}

// =============================================================================
// Tests for CB-513 through CB-518: Rust Best Practices (Extended)
// =============================================================================

#[cfg(test)]
mod cb513_to_cb518_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("tests_part2_cb513_cb518_rust.rs");
}

// =============================================================================
// Tests for CB-519 through CB-522: Aprender Bug Pattern Detection (Part 1)
// =============================================================================

#[cfg(test)]
mod cb519_to_cb522_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("tests_part2_cb519_cb522_aprender.rs");
}

// =============================================================================
// Tests for CB-523 through CB-527: Aprender Bug Pattern Detection (Part 2)
// =============================================================================

#[cfg(test)]
mod cb523_to_cb527_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("tests_part2_cb523_cb527_aprender.rs");
}
