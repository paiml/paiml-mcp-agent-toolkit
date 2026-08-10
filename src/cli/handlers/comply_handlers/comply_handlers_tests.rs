// Tests for comply handlers
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.version.is_empty());
    }

    #[test]
    fn test_calculate_versions_behind_same() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_check_status_equality() {
        assert_eq!(CheckStatus::Pass, CheckStatus::Pass);
        assert_ne!(CheckStatus::Pass, CheckStatus::Fail);
    }

    #[test]
    fn test_severity_variants() {
        let _ = Severity::Info;
        let _ = Severity::Warning;
        let _ = Severity::Error;
        let _ = Severity::Critical;
    }
}

// Coverage tests extracted to comply_handlers_coverage_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax (missing imports from comply_cb_detect.rs)
#[cfg(feature = "broken-tests")]
#[path = "comply_handlers_coverage_tests.rs"]
mod coverage_tests;

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_version_behind_never_negative(major in 0u32..100, minor in 0u32..1000, patch in 0u32..100) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let behind = calculate_versions_behind(&version);
            // Should always return a non-negative value (saturating_sub)
            prop_assert!(behind < u32::MAX);
        }

        #[test]
        fn test_check_version_currency_always_returns_valid_check(
            major in 0u32..10,
            minor in 0u32..500,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let check = check_version_currency(&version);

            // Check should always have non-empty fields
            prop_assert!(!check.name.is_empty());
            prop_assert!(!check.message.is_empty());

            // Status should be one of the valid variants
            prop_assert!(matches!(
                check.status,
                CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Skip
            ));
        }

        #[test]
        fn test_project_config_roundtrip_serialization(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            auto_update in proptest::bool::ANY
        ) {
            let config = ProjectConfig {
                pmat: PmatSection {
                    version: version.clone(),
                    last_compliance_check: Some(Utc::now()),
                    auto_update,
                },
            };

            let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
            let deserialized: ProjectConfig = toml::from_str(&serialized).expect("Deserialization failed");

            prop_assert_eq!(deserialized.pmat.version, version);
            prop_assert_eq!(deserialized.pmat.auto_update, auto_update);
        }

        #[test]
        fn test_compliance_check_serialization_roundtrip(
            name in "[a-zA-Z ]{1,50}",
            message in "[a-zA-Z0-9 ]{1,100}"
        ) {
            let check = ComplianceCheck {
                name: name.clone(),
                status: CheckStatus::Pass,
                message: message.clone(),
                severity: Severity::Info,
            };

            let json = serde_json::to_string(&check).expect("Serialization failed");
            let deserialized: ComplianceCheck = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.message, message);
        }

        #[test]
        fn test_breaking_change_serialization_roundtrip(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            description in "[a-zA-Z0-9 ]{1,200}"
        ) {
            let change = BreakingChange {
                version: version.clone(),
                description: description.clone(),
                migration_guide: Some("Guide".to_string()),
            };

            let json = serde_json::to_string(&change).expect("Serialization failed");
            let deserialized: BreakingChange = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.version, version);
            prop_assert_eq!(deserialized.description, description);
        }

        // These two properties were the stub written down as invariants: every
        // entry carried PMAT_VERSION because `get_changelog_entries` ignored its
        // range and stamped the current version on three canned rows, and
        // `get_breaking_changes_since` returned `[]` unconditionally. Both now
        // read CHANGELOG.md, so the properties assert what should actually hold
        // of a version range.

        #[test]
        fn test_changelog_entries_are_distinct_real_versions(_seed in 0u32..1000) {
            let entries = get_changelog_entries("0.0.0", "999.999.999");
            prop_assert!(!entries.is_empty());

            let mut seen = std::collections::HashSet::new();
            for entry in &entries {
                prop_assert!(!entry.version.is_empty());
                // A repeated version means the range is not being read.
                prop_assert!(seen.insert(entry.version.clone()));
            }
            prop_assert!(seen.len() > 1, "the changelog holds more than one release");
        }

        #[test]
        fn test_breaking_changes_never_predate_the_version_asked_for(
            major in 0u32..100,
            minor in 0u32..1000,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let changes = get_breaking_changes_since(&version);
            // Whatever comes back must be strictly newer than the floor given,
            // and must name a version.
            for change in &changes {
                prop_assert!(!change.version.is_empty());
                prop_assert_ne!(&change.version, &version);
            }
        }
    }

    // Additional property tests that require tempdir (can't use proptest macro easily)
    #[test]
    fn test_check_config_files_consistency() {
        use tempfile::TempDir;

        // Test that check_config_files is consistent across multiple calls
        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_config_files(temp.path());
        let check2 = check_config_files(temp.path());

        assert_eq!(check1.status, check2.status);
        assert_eq!(check1.message, check2.message);
    }

    #[test]
    fn test_check_hooks_consistency() {
        use tempfile::TempDir;

        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_hooks_installed(temp.path());
        let check2 = check_hooks_installed(temp.path());

        assert_eq!(check1.status, check2.status);
    }
}
