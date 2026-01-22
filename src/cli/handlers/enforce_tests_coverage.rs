mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    include!("enforce_coverage_part1.rs");
    include!("enforce_coverage_part2.rs");
    include!("enforce_coverage_part3.rs");
    include!("enforce_coverage_part4.rs");
}
