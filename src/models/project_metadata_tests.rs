#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // === ProjectMetadata Tests ===

    #[test]
    fn test_new_project_metadata() {
        let metadata = ProjectMetadata::new("2.205.0");
        assert_eq!(metadata.pmat.version, "2.205.0");
        assert_eq!(metadata.pmat.schema_version, "1.0");
        assert!(metadata.pmat.last_compliance_check.is_some());
        assert!(metadata.compliance.breaking_changes_accepted.is_empty());
    }

    #[test]
    fn test_new_project_metadata_from_string() {
        let version = String::from("1.0.0");
        let metadata = ProjectMetadata::new(version);
        assert_eq!(metadata.pmat.version, "1.0.0");
    }

    #[test]
    fn test_new_project_metadata_from_str() {
        let metadata = ProjectMetadata::new("0.5.0-beta");
        assert_eq!(metadata.pmat.version, "0.5.0-beta");
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = ProjectMetadata::new("2.205.0");

        metadata.save(temp_dir.path()).unwrap();
        assert!(ProjectMetadata::exists(temp_dir.path()));

        let loaded = ProjectMetadata::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.pmat.version, metadata.pmat.version);
        assert_eq!(loaded.pmat.schema_version, metadata.pmat.schema_version);
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("deep").join("nested").join("project");

        let metadata = ProjectMetadata::new("1.0.0");
        metadata.save(&nested_path).unwrap();

        assert!(ProjectMetadata::exists(&nested_path));
    }

    #[test]
    fn test_load_nonexistent_file_fails() {
        let temp_dir = TempDir::new().unwrap();
        let result = ProjectMetadata::load(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_malformed_toml_fails() {
        let temp_dir = TempDir::new().unwrap();
        let pmat_dir = temp_dir.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).unwrap();
        std::fs::write(pmat_dir.join("project.toml"), "not = [valid toml\n").unwrap();

        let result = ProjectMetadata::load(temp_dir.path());
        assert!(result.is_err(), "malformed toml must return parse error");
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("Failed to parse") || err_msg.contains("project.toml"),
            "error must name the file: {err_msg}"
        );
    }

    #[test]
    fn test_exists_false_for_new_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!ProjectMetadata::exists(temp_dir.path()));
    }

    #[test]
    fn test_record_migration() {
        let mut metadata = ProjectMetadata::new("2.150.0");
        metadata.record_migration("2.150.0".to_string(), "2.205.0".to_string(), true);

        assert_eq!(metadata.pmat.version, "2.205.0");
        assert_eq!(metadata.compliance.migration_history.len(), 1);
        assert!(metadata.compliance.last_migration.is_some());

        let migration = &metadata.compliance.migration_history[0];
        assert_eq!(migration.from_version, "2.150.0");
        assert_eq!(migration.to_version, "2.205.0");
        assert!(migration.success);
    }

    #[test]
    fn test_record_migration_failed() {
        let mut metadata = ProjectMetadata::new("2.150.0");
        metadata.record_migration("2.150.0".to_string(), "2.205.0".to_string(), false);

        // Version should NOT change on failed migration
        assert_eq!(metadata.pmat.version, "2.150.0");
        assert_eq!(metadata.compliance.migration_history.len(), 1);
        // last_migration should NOT be set on failure
        assert!(metadata.compliance.last_migration.is_none());

        let migration = &metadata.compliance.migration_history[0];
        assert!(!migration.success);
    }

    #[test]
    fn test_record_multiple_migrations() {
        let mut metadata = ProjectMetadata::new("1.0.0");
        metadata.record_migration("1.0.0".to_string(), "1.5.0".to_string(), true);
        metadata.record_migration("1.5.0".to_string(), "2.0.0".to_string(), true);
        metadata.record_migration("2.0.0".to_string(), "2.5.0".to_string(), true);

        assert_eq!(metadata.pmat.version, "2.5.0");
        assert_eq!(metadata.compliance.migration_history.len(), 3);
    }

    #[test]
    fn test_accept_breaking_change() {
        let mut metadata = ProjectMetadata::new("2.150.0");
        metadata.accept_breaking_change("2.180.0".to_string());
        metadata.accept_breaking_change("2.195.0".to_string());

        assert!(metadata.is_breaking_change_accepted("2.180.0"));
        assert!(metadata.is_breaking_change_accepted("2.195.0"));
        assert!(!metadata.is_breaking_change_accepted("2.200.0"));
        assert_eq!(metadata.compliance.breaking_changes_accepted.len(), 2);
    }

    #[test]
    fn test_accept_breaking_change_duplicate() {
        let mut metadata = ProjectMetadata::new("2.150.0");
        metadata.accept_breaking_change("2.180.0".to_string());
        metadata.accept_breaking_change("2.180.0".to_string()); // Duplicate

        // Should only have one entry
        assert_eq!(metadata.compliance.breaking_changes_accepted.len(), 1);
    }

    #[test]
    fn test_update_compliance_check() {
        let mut metadata = ProjectMetadata::new("2.205.0");
        let first_check = metadata.pmat.last_compliance_check.clone();

        // Wait a tiny bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        metadata.update_compliance_check();
        let second_check = metadata.pmat.last_compliance_check.clone();

        assert_ne!(first_check, second_check);
    }

    #[test]
    fn test_get_path() {
        let path = Path::new("/tmp/test-project");
        let expected = path.join(".pmat").join("project.toml");
        assert_eq!(ProjectMetadata::get_path(path), expected);
    }

    #[test]
    fn test_get_path_relative() {
        let path = Path::new("relative/path");
        let result = ProjectMetadata::get_path(path);
        assert!(result.ends_with("project.toml"));
        assert!(result.to_string_lossy().contains(".pmat"));
    }

    #[test]
    fn test_serialization() {
        let metadata = ProjectMetadata::new("2.205.0");
        let toml = toml::to_string_pretty(&metadata).unwrap();

        assert!(toml.contains("[pmat]"));
        assert!(toml.contains("version = \"2.205.0\""));
        assert!(toml.contains("schema_version = \"1.0\""));
        assert!(toml.contains("[compliance]"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let metadata = ProjectMetadata::new("2.205.0");
        let toml_str = toml::to_string_pretty(&metadata).unwrap();
        let parsed: ProjectMetadata = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.pmat.version, metadata.pmat.version);
        assert_eq!(parsed.pmat.schema_version, metadata.pmat.schema_version);
    }

    #[test]
    fn test_metadata_with_migrations_serialization() {
        let mut metadata = ProjectMetadata::new("1.0.0");
        metadata.record_migration("1.0.0".to_string(), "2.0.0".to_string(), true);
        metadata.accept_breaking_change("1.5.0".to_string());

        let toml_str = toml::to_string_pretty(&metadata).unwrap();
        let parsed: ProjectMetadata = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.pmat.version, "2.0.0");
        assert_eq!(parsed.compliance.migration_history.len(), 1);
        assert_eq!(parsed.compliance.breaking_changes_accepted.len(), 1);
    }

    // === PmatMetadata Tests ===

    #[test]
    fn test_pmat_metadata_creation() {
        let pmat = PmatMetadata {
            version: "1.0.0".to_string(),
            last_compliance_check: Some("2024-01-01T00:00:00Z".to_string()),
            schema_version: "1.0".to_string(),
        };

        assert_eq!(pmat.version, "1.0.0");
        assert!(pmat.last_compliance_check.is_some());
        assert_eq!(pmat.schema_version, "1.0");
    }

    #[test]
    fn test_pmat_metadata_without_compliance_check() {
        let pmat = PmatMetadata {
            version: "1.0.0".to_string(),
            last_compliance_check: None,
            schema_version: "1.0".to_string(),
        };

        assert!(pmat.last_compliance_check.is_none());
    }

    #[test]
    fn test_pmat_metadata_serialization_skips_none() {
        let pmat = PmatMetadata {
            version: "1.0.0".to_string(),
            last_compliance_check: None,
            schema_version: "1.0".to_string(),
        };

        let toml = toml::to_string(&pmat).unwrap();
        assert!(!toml.contains("last_compliance_check"));
    }

    // === ComplianceMetadata Tests ===

    #[test]
    fn test_compliance_metadata_default() {
        let compliance = ComplianceMetadata::default();
        assert!(compliance.breaking_changes_accepted.is_empty());
        assert!(compliance.last_migration.is_none());
        assert!(compliance.migration_history.is_empty());
    }

    #[test]
    fn test_compliance_metadata_with_data() {
        let compliance = ComplianceMetadata {
            breaking_changes_accepted: vec!["1.0.0".to_string(), "2.0.0".to_string()],
            last_migration: Some("2024-01-01T00:00:00Z".to_string()),
            migration_history: vec![MigrationRecord {
                from_version: "1.0.0".to_string(),
                to_version: "2.0.0".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                success: true,
            }],
        };

        assert_eq!(compliance.breaking_changes_accepted.len(), 2);
        assert!(compliance.last_migration.is_some());
        assert_eq!(compliance.migration_history.len(), 1);
    }

    #[test]
    fn test_compliance_metadata_serialization_skips_empty() {
        let compliance = ComplianceMetadata::default();
        let toml = toml::to_string(&compliance).unwrap();

        assert!(!toml.contains("breaking_changes_accepted"));
        assert!(!toml.contains("migration_history"));
    }

    // === MigrationRecord Tests ===

    #[test]
    fn test_migration_record_creation() {
        let record = MigrationRecord {
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            success: true,
        };

        assert_eq!(record.from_version, "1.0.0");
        assert_eq!(record.to_version, "2.0.0");
        assert!(record.success);
    }

    #[test]
    fn test_migration_record_failed() {
        let record = MigrationRecord {
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            success: false,
        };

        assert!(!record.success);
    }

    #[test]
    fn test_migration_record_serialization() {
        let record = MigrationRecord {
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            success: true,
        };

        let toml = toml::to_string(&record).unwrap();
        assert!(toml.contains("from_version = \"1.0.0\""));
        assert!(toml.contains("to_version = \"2.0.0\""));
        assert!(toml.contains("success = true"));
    }

    // === Clone Tests ===

    #[test]
    fn test_project_metadata_clone() {
        let metadata = ProjectMetadata::new("1.0.0");
        let cloned = metadata.clone();
        assert_eq!(cloned.pmat.version, metadata.pmat.version);
    }

    #[test]
    fn test_migration_record_clone() {
        let record = MigrationRecord {
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            success: true,
        };
        let cloned = record.clone();
        assert_eq!(cloned.from_version, record.from_version);
    }

    // === Debug Tests ===

    #[test]
    fn test_project_metadata_debug() {
        let metadata = ProjectMetadata::new("1.0.0");
        let debug = format!("{:?}", metadata);
        assert!(debug.contains("ProjectMetadata"));
        assert!(debug.contains("1.0.0"));
    }

    #[test]
    fn test_migration_record_debug() {
        let record = MigrationRecord {
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            success: true,
        };
        let debug = format!("{:?}", record);
        assert!(debug.contains("MigrationRecord"));
    }

    // === PartialEq Tests ===

    #[test]
    fn test_project_metadata_equality() {
        let m1 = ProjectMetadata {
            pmat: PmatMetadata {
                version: "1.0.0".to_string(),
                last_compliance_check: None,
                schema_version: "1.0".to_string(),
            },
            compliance: ComplianceMetadata::default(),
        };

        let m2 = ProjectMetadata {
            pmat: PmatMetadata {
                version: "1.0.0".to_string(),
                last_compliance_check: None,
                schema_version: "1.0".to_string(),
            },
            compliance: ComplianceMetadata::default(),
        };

        assert_eq!(m1, m2);
    }

    #[test]
    fn test_project_metadata_inequality() {
        let m1 = ProjectMetadata::new("1.0.0");
        let m2 = ProjectMetadata::new("2.0.0");
        assert_ne!(m1.pmat.version, m2.pmat.version);
    }

    #[test]
    fn test_default_schema_version() {
        assert_eq!(default_schema_version(), "1.0");
    }
}
