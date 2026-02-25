// Coverage tests for comply handlers - part 2
// Split for file health compliance (CB-040)
//
// Section 1: check_compute_brick, check_cargo_lock, check_msrv,
//            check_ci_configured, check_paiml_deps_workspace tests
// Section 2: get_breaking_changes_since, get_changelog_entries,
//            load_or_create_project_config, update_last_check_timestamp,
//            migrate_project_version, migrate_gitignore,
//            update_project_config, print_compliance_text tests

include!("coverage_part2_checks.rs");
include!("coverage_part2_config.rs");
