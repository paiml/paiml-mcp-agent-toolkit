// Quality gate formatting functions - split for file health (CB-040)
//
// Formatters: JSON, human-readable, JUnit XML output
include!("quality_checks_part3_formatters.rs");

// Reports: summary, detailed, markdown output
include!("quality_checks_part3_reports.rs");

// Tests for all quality_checks_part3 formatters and reports
include!("quality_checks_part3_tests.rs");
