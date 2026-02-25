#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> CiCdLearningConfig {
        CiCdLearningConfig {
            data_dir: PathBuf::from("/tmp/pmat_test/training_data"),
            model_dir: PathBuf::from("/tmp/pmat_test/models"),
            min_samples_for_training: 5,
            max_training_samples: 100,
            auto_train: false, // Disable for tests
            versioning_enabled: true,
        }
    }

    fn create_test_metadata() -> CiCdMetadata {
        CiCdMetadata {
            system: "github".to_string(),
            repository: "test/repo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            build_id: "12345".to_string(),
        }
    }

    fn create_test_mutant() -> Mutant {
        Mutant {
            id: "test_1".to_string(),
            original_file: PathBuf::from("test.rs"),
            mutated_source: "fn test() { }".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 10,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "test_hash".to_string(),
            status: MutantStatus::Pending,
        }
    }

    #[test]
    fn test_ci_cd_config_default() {
        let config = CiCdLearningConfig::default();
        assert_eq!(config.min_samples_for_training, 50);
        assert_eq!(config.max_training_samples, 10000);
        assert!(config.auto_train);
        assert!(config.versioning_enabled);
    }

    #[test]
    fn test_ci_cd_learning_manager_creation() {
        let config = create_test_config();
        let manager = CiCdLearningManager::new(config);
        assert!(manager.current_version().is_none());
    }

    #[test]
    fn test_training_batch_creation() {
        let _metadata = create_test_metadata();
        let mutant = create_test_mutant();

        let results = [MutationResult {
            mutant,
            status: MutantStatus::Killed,
            test_failures: vec!["test1".to_string()],
            execution_time_ms: 100,
            error_message: None,
        }];

        let samples: Vec<TrainingData> = results
            .iter()
            .map(|r| TrainingData {
                mutant: r.mutant.clone(),
                was_killed: r.status == MutantStatus::Killed,
                test_failures: r.test_failures.clone(),
                execution_time_ms: r.execution_time_ms,
            })
            .collect();

        assert_eq!(samples.len(), 1);
        assert!(samples[0].was_killed);
    }

    #[test]
    fn test_model_version_increment() {
        let config = create_test_config();
        let mut manager = CiCdLearningManager::new(config);

        assert_eq!(manager.get_next_version(), 1);

        manager.current_version = Some(ModelVersion {
            version: 5,
            trained_at: Utc::now(),
            sample_count: 100,
            accuracy: 0.85,
            file_path: PathBuf::from("/tmp/model_v5.bin"),
            metadata: None,
        });

        assert_eq!(manager.get_next_version(), 6);
    }

    #[tokio::test]
    async fn test_collect_training_data() {
        let config = create_test_config();
        let mut manager = CiCdLearningManager::new(config);
        let metadata = create_test_metadata();
        let mutant = create_test_mutant();

        let results = vec![
            MutationResult {
                mutant: mutant.clone(),
                status: MutantStatus::Killed,
                test_failures: vec!["test1".to_string()],
                execution_time_ms: 100,
                error_message: None,
            },
            MutationResult {
                mutant,
                status: MutantStatus::Survived,
                test_failures: vec![],
                execution_time_ms: 50,
                error_message: None,
            },
        ];

        let batch = manager
            .collect_training_data(&results, metadata)
            .await
            .unwrap();

        assert_eq!(batch.samples.len(), 2);
        assert!(batch.samples[0].was_killed);
        assert!(!batch.samples[1].was_killed);
    }
}
