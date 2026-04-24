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

#[cfg(test)]
mod migrate_handlers_init_tests {
    //! Covers pure-compute helpers in migrate_handlers_init.rs
    //! (72 uncov on broad, 0% cov). The async handle_init / handle_upgrade
    //! require full integration and are skipped.
    use super::*;

    #[test]
    fn test_generate_default_pmat_yaml_is_parseable_yaml() {
        let yaml = generate_default_pmat_yaml();
        let _: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml).unwrap();
    }

    #[test]
    fn test_generate_default_pmat_yaml_contains_expected_sections() {
        let yaml = generate_default_pmat_yaml();
        assert!(yaml.contains("comply:"));
        assert!(yaml.contains("thresholds:"));
        assert!(yaml.contains("coverage:"));
        assert!(yaml.contains("complexity:"));
        assert!(yaml.contains("cb-050"));
        assert!(yaml.contains("cb-060"));
    }

    #[test]
    fn test_generate_default_pmat_yaml_loads_as_pmat_yaml_config() {
        let yaml = generate_default_pmat_yaml();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), yaml).unwrap();
        let cfg =
            crate::models::comply_config::PmatYamlConfig::load_from_path(tmp.path()).unwrap();
        assert!((cfg.comply.thresholds.coverage - 85.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_claude_md_embeds_project_name() {
        let md = generate_claude_md("my-project");
        assert!(md.contains("# Claude Code Configuration for my-project"));
    }

    #[test]
    fn test_generate_claude_md_contains_required_sections() {
        let md = generate_claude_md("x");
        assert!(md.contains("## Code Search Policy"));
        assert!(md.contains("## Quality Standards"));
        assert!(md.contains("## Coverage"));
        assert!(md.contains("## Git Workflow"));
        assert!(md.contains("pmat query"));
        assert!(md.contains("cargo llvm-cov"));
    }

    #[test]
    fn test_generate_claude_md_empty_project_name_still_valid() {
        let md = generate_claude_md("");
        assert!(md.contains("# Claude Code Configuration for "));
        assert!(!md.is_empty());
    }
}
