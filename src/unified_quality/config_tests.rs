// Tests for config module
// Included from config.rs - no `use` imports or `#!` inner attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_unified_config_default() {
        let config = UnifiedConfig::default();
        assert_eq!(config.mode, QualityMode::Observe);
        assert!(config.auto_progress);
        assert!(config.monitoring.enabled);
        assert!(!config.automation.enabled);
    }

    #[test]
    fn test_unified_config_default_values() {
        let config = UnifiedConfig::default();
        assert_eq!(config.progress_after_days, 30);
        assert!(!config.research.property_synthesis);
        assert!(!config.research.formal_verification);
        assert!(!config.research.ml_suggestions);
        assert!(!config.research.autonomous_mode);
    }

    #[test]
    fn test_should_progress() {
        let config = UnifiedConfig::default();
        assert!(!config.should_progress(15));
        assert!(config.should_progress(30));
        assert!(config.should_progress(45));
    }

    #[test]
    fn test_should_progress_disabled() {
        let config = UnifiedConfig {
            auto_progress: false,
            ..Default::default()
        };
        assert!(!config.should_progress(30));
        assert!(!config.should_progress(100));
    }

    #[test]
    fn test_next_mode() {
        let mut config = UnifiedConfig {
            mode: QualityMode::Observe,
            ..UnifiedConfig::default()
        };
        assert_eq!(config.next_mode(), Some(QualityMode::Advise));

        config.mode = QualityMode::Extreme;
        assert_eq!(config.next_mode(), None);
    }

    #[test]
    fn test_next_mode_all_transitions() {
        let mut config = UnifiedConfig {
            mode: QualityMode::Observe,
            ..Default::default()
        };
        assert_eq!(config.next_mode(), Some(QualityMode::Advise));

        config.mode = QualityMode::Advise;
        assert_eq!(config.next_mode(), Some(QualityMode::Guide));

        config.mode = QualityMode::Guide;
        assert_eq!(config.next_mode(), Some(QualityMode::Enforce));

        config.mode = QualityMode::Enforce;
        assert_eq!(config.next_mode(), Some(QualityMode::Extreme));

        config.mode = QualityMode::Extreme;
        assert_eq!(config.next_mode(), None);
    }

    #[test]
    fn test_default_path() {
        let path = UnifiedConfig::default_path();
        assert_eq!(path, PathBuf::from(".pmat/config.toml"));
    }

    #[test]
    fn test_budget_config_default() {
        let budget = BudgetConfig::default();
        assert_eq!(budget.complexity_points, 100);
        assert_eq!(budget.satd_allowance, 20);
        assert!((budget.coverage_floor - 70.0).abs() < f64::EPSILON);
        assert!((budget.regeneration_daily - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_config_clone() {
        let budget = BudgetConfig::default();
        let cloned = budget.clone();
        assert_eq!(cloned.complexity_points, budget.complexity_points);
        assert_eq!(cloned.satd_allowance, budget.satd_allowance);
    }

    #[test]
    fn test_budget_config_debug() {
        let budget = BudgetConfig::default();
        let debug_str = format!("{:?}", budget);
        assert!(debug_str.contains("BudgetConfig"));
        assert!(debug_str.contains("complexity_points"));
    }

    #[test]
    fn test_budget_config_serialization() {
        let budget = BudgetConfig::default();
        let json = serde_json::to_string(&budget).unwrap();
        assert!(json.contains("complexity_points"));

        let deserialized: BudgetConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.complexity_points, 100);
    }

    #[test]
    fn test_monitoring_config_default() {
        let monitoring = MonitoringConfig::default();
        assert!(monitoring.enabled);
        assert!(monitoring.incremental);
        assert!(monitoring.cache_ast);
        assert_eq!(monitoring.dashboard_port, 8080);
        assert_eq!(monitoring.update_interval, 5);
        assert!(!monitoring.watch_patterns.is_empty());
    }

    #[test]
    fn test_monitoring_config_watch_patterns() {
        let monitoring = MonitoringConfig::default();
        assert!(monitoring.watch_patterns.contains(&"**/*.rs".to_string()));
        assert!(monitoring.watch_patterns.contains(&"**/*.py".to_string()));
        assert!(monitoring.watch_patterns.contains(&"**/*.js".to_string()));
        assert!(monitoring.watch_patterns.contains(&"**/*.ts".to_string()));
    }

    #[test]
    fn test_monitoring_config_clone() {
        let monitoring = MonitoringConfig::default();
        let cloned = monitoring.clone();
        assert_eq!(cloned.enabled, monitoring.enabled);
        assert_eq!(cloned.dashboard_port, monitoring.dashboard_port);
    }

    #[test]
    fn test_monitoring_config_debug() {
        let monitoring = MonitoringConfig::default();
        let debug_str = format!("{:?}", monitoring);
        assert!(debug_str.contains("MonitoringConfig"));
        assert!(debug_str.contains("dashboard_port"));
    }

    #[test]
    fn test_monitoring_config_serialization() {
        let monitoring = MonitoringConfig::default();
        let json = serde_json::to_string(&monitoring).unwrap();
        assert!(json.contains("dashboard_port"));

        let deserialized: MonitoringConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.dashboard_port, 8080);
    }

    #[test]
    fn test_automation_config_default() {
        let automation = AutomationConfig::default();
        assert!(!automation.enabled);
        assert!(automation.require_review);
        assert!(automation.safe_only);
        assert!(!automation.create_branches);
    }

    #[test]
    fn test_automation_config_clone() {
        let automation = AutomationConfig::default();
        let cloned = automation.clone();
        assert_eq!(cloned.enabled, automation.enabled);
        assert_eq!(cloned.require_review, automation.require_review);
    }

    #[test]
    fn test_automation_config_debug() {
        let automation = AutomationConfig::default();
        let debug_str = format!("{:?}", automation);
        assert!(debug_str.contains("AutomationConfig"));
        assert!(debug_str.contains("require_review"));
    }

    #[test]
    fn test_automation_config_serialization() {
        let automation = AutomationConfig::default();
        let json = serde_json::to_string(&automation).unwrap();
        assert!(json.contains("require_review"));

        let deserialized: AutomationConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.require_review);
    }

    #[test]
    fn test_research_config_default() {
        let research = ResearchConfig::default();
        assert!(!research.property_synthesis);
        assert!(!research.formal_verification);
        assert!(!research.ml_suggestions);
        assert!(!research.autonomous_mode);
    }

    #[test]
    fn test_research_config_clone() {
        let research = ResearchConfig::default();
        let cloned = research.clone();
        assert_eq!(cloned.property_synthesis, research.property_synthesis);
        assert_eq!(cloned.autonomous_mode, research.autonomous_mode);
    }

    #[test]
    fn test_research_config_debug() {
        let research = ResearchConfig::default();
        let debug_str = format!("{:?}", research);
        assert!(debug_str.contains("ResearchConfig"));
        assert!(debug_str.contains("property_synthesis"));
    }

    #[test]
    fn test_research_config_serialization() {
        let research = ResearchConfig::default();
        let json = serde_json::to_string(&research).unwrap();
        assert!(json.contains("property_synthesis"));

        let deserialized: ResearchConfig = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.property_synthesis);
    }

    #[test]
    fn test_unified_config_clone() {
        let config = UnifiedConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.mode, config.mode);
        assert_eq!(cloned.auto_progress, config.auto_progress);
        assert_eq!(cloned.progress_after_days, config.progress_after_days);
    }

    #[test]
    fn test_unified_config_debug() {
        let config = UnifiedConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("UnifiedConfig"));
        assert!(debug_str.contains("mode"));
    }

    #[test]
    fn test_unified_config_serialization() {
        let config = UnifiedConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("mode"));
        assert!(json.contains("auto_progress"));

        let deserialized: UnifiedConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mode, QualityMode::Observe);
    }

    #[test]
    fn test_unified_config_to_file_and_from_file() {
        let config = UnifiedConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        config.to_file(&path).unwrap();
        let loaded = UnifiedConfig::from_file(&path).unwrap();

        assert_eq!(loaded.mode, config.mode);
        assert_eq!(loaded.auto_progress, config.auto_progress);
        assert_eq!(loaded.progress_after_days, config.progress_after_days);
    }

    #[test]
    fn test_unified_config_from_file_invalid() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        std::fs::write(&path, "invalid toml {").unwrap();

        let result = UnifiedConfig::from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_unified_config_from_file_not_found() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result = UnifiedConfig::from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_adoption_playbook() {
        let playbook = AdoptionPlaybook::default();
        assert_eq!(playbook.current_week, 1);
        assert_eq!(playbook.phases.len(), 4);
        assert_eq!(playbook.phases[0].name, "Baseline Establishment");
    }

    #[test]
    fn test_adoption_playbook_phases() {
        let playbook = AdoptionPlaybook::default();

        // Phase 1
        assert_eq!(playbook.phases[0].weeks, (1, 2));
        assert!(!playbook.phases[0].activities.is_empty());
        assert!(!playbook.phases[0].success_criteria.is_empty());

        // Phase 2
        assert_eq!(playbook.phases[1].weeks, (3, 4));
        assert_eq!(playbook.phases[1].name, "Team Education");

        // Phase 3
        assert_eq!(playbook.phases[2].weeks, (5, 8));
        assert_eq!(playbook.phases[2].name, "Assisted Improvement");

        // Phase 4
        assert_eq!(playbook.phases[3].weeks, (9, 12));
        assert_eq!(playbook.phases[3].name, "Gradual Enforcement");
    }

    #[test]
    fn test_adoption_playbook_clone() {
        let playbook = AdoptionPlaybook::default();
        let cloned = playbook.clone();
        assert_eq!(cloned.current_week, playbook.current_week);
        assert_eq!(cloned.phases.len(), playbook.phases.len());
    }

    #[test]
    fn test_adoption_playbook_debug() {
        let playbook = AdoptionPlaybook::default();
        let debug_str = format!("{:?}", playbook);
        assert!(debug_str.contains("AdoptionPlaybook"));
        assert!(debug_str.contains("current_week"));
    }

    #[test]
    fn test_adoption_playbook_serialization() {
        let playbook = AdoptionPlaybook::default();
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(json.contains("current_week"));
        assert!(json.contains("phases"));

        let deserialized: AdoptionPlaybook = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_week, 1);
        assert_eq!(deserialized.phases.len(), 4);
    }

    #[test]
    fn test_adoption_phase_clone() {
        let phase = AdoptionPhase {
            weeks: (1, 2),
            name: "Test Phase".to_string(),
            activities: vec!["Activity 1".to_string()],
            success_criteria: vec!["Criterion 1".to_string()],
        };
        let cloned = phase.clone();
        assert_eq!(cloned.weeks, phase.weeks);
        assert_eq!(cloned.name, phase.name);
    }

    #[test]
    fn test_adoption_phase_debug() {
        let phase = AdoptionPhase {
            weeks: (1, 2),
            name: "Test Phase".to_string(),
            activities: vec![],
            success_criteria: vec![],
        };
        let debug_str = format!("{:?}", phase);
        assert!(debug_str.contains("AdoptionPhase"));
        assert!(debug_str.contains("Test Phase"));
    }

    #[test]
    fn test_adoption_phase_serialization() {
        let phase = AdoptionPhase {
            weeks: (5, 8),
            name: "Test Phase".to_string(),
            activities: vec!["Do something".to_string()],
            success_criteria: vec!["Check something".to_string()],
        };
        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains("Test Phase"));

        let deserialized: AdoptionPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.weeks, (5, 8));
        assert_eq!(deserialized.name, "Test Phase");
    }
}
