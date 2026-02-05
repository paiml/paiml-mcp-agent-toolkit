// Tests extracted to tests.rs for file health compliance (CB-040)
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // Deleted estimate_cognitive_complexity - using proper AST analysis instead
    use std::io::Write;
    use tempfile::TempDir;

    include!("tests_core_part1.rs");
    include!("tests_core_part2.rs");
    include!("tests_core_part3.rs");
    include!("tests_core_part4.rs");
}
