//! Coverage tests split for file health compliance (CB-040)
mod coverage_tests {
    use super::*;
    use crate::ast::core::{Location, ProofAnnotation};

    include!("coverage_tests_part1.rs");
    include!("coverage_tests_part2.rs");
    include!("coverage_tests_part3.rs");
    include!("coverage_tests_part4.rs");
}
