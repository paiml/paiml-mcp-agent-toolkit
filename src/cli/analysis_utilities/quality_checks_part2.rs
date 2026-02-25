// Quality check functions part 2 - split for file health (CB-040)
//
// Security checks and duplicate code detection
include!("quality_checks_part2_security_duplicates.rs");

// Coverage checking and documentation section validation
include!("quality_checks_part2_coverage_sections.rs");

// Provability analysis and quality gate output formatting
include!("quality_checks_part2_provability.rs");
