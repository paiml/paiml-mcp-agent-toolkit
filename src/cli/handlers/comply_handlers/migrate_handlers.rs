// Migration, enforce, report, init, and upgrade handlers for comply subcommands.
//
// This file is include!()'d into comply_handlers/mod.rs scope,
// where it has access to check_handlers items (via pub use check_handlers::*).
//
// Split into submodules for file health (CB-040):
// - migrate_handlers_migration.rs: handle_migrate, handle_diff, handle_update
// - migrate_handlers_init.rs: handle_init, handle_upgrade, scaffold generators
// - migrate_handlers_enforce.rs: handle_enforce, handle_report, hook helpers

// Migration, diff, and update commands
include!("migrate_handlers_migration.rs");

// Project initialization, upgrade, and scaffold generation
include!("migrate_handlers_init.rs");

// Enforcement hooks and compliance reporting
include!("migrate_handlers_enforce.rs");
