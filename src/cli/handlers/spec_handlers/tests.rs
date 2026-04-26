//! Tests split for file health compliance (CB-040)
//! This file is included as `mod tests` from spec_handlers/mod.rs via
//! `#[path = "tests.rs"]`, so its top-level items ARE the test module's
//! contents. No inner `mod tests` wrapper (would double-nest `super::`).

include!("tests_part1.rs");
include!("tests_part2.rs");
