// === AgentConfig Tests ===

#[test]
fn test_agent_config_custom() {
    let config = AgentConfig {
        name: "custom-agent".to_string(),
        version: "2.0.0".to_string(),
        complexity_threshold: 15,
        watch_patterns: vec!["**/*.go".to_string()],
        update_interval: 10,
        max_projects: 5,
    };

    assert_eq!(config.name, "custom-agent");
    assert_eq!(config.version, "2.0.0");
    assert_eq!(config.complexity_threshold, 15);
    assert_eq!(config.watch_patterns.len(), 1);
    assert_eq!(config.update_interval, 10);
    assert_eq!(config.max_projects, 5);
}

#[test]
fn test_agent_config_default_watch_patterns() {
    let config = AgentConfig::default();

    assert!(config.watch_patterns.contains(&"**/*.rs".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.py".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.js".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.ts".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.java".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.go".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.cpp".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.c".to_string()));
}

#[test]
fn test_agent_config_serialization() {
    let config = AgentConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.name, deserialized.name);
    assert_eq!(config.version, deserialized.version);
    assert_eq!(
        config.complexity_threshold,
        deserialized.complexity_threshold
    );
}

#[test]
fn test_agent_config_debug() {
    let config = AgentConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("pmat-agent"));
    assert!(debug_str.contains("1.0.0"));
}

#[test]
fn test_agent_config_hpp_pattern() {
    let config = AgentConfig::default();
    assert!(config.watch_patterns.contains(&"**/*.hpp".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.h".to_string()));
}

// === MonitoredProject Tests ===

#[test]
fn test_monitored_project_with_analysis() {
    let analysis = ProjectAnalysisResult {
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        quality_score: 0.85,
        files_analyzed: 10,
        functions_analyzed: 50,
        avg_complexity: 8.5,
        hotspot_functions: 3,
        satd_issues: 2,
        quality_gate_status: "PASSED".to_string(),
        recommendations: vec!["Reduce complexity".to_string()],
    };

    let project = MonitoredProject {
        path: PathBuf::from("/test/project"),
        name: "test_project".to_string(),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: Some(analysis),
        started_at: std::time::SystemTime::now(),
    };

    assert!(project.last_analysis.is_some());
    let analysis = project.last_analysis.unwrap();
    assert_eq!(analysis.quality_score, 0.85);
    assert_eq!(analysis.files_analyzed, 10);
}

#[test]
fn test_monitored_project_clone() {
    let project = MonitoredProject {
        path: PathBuf::from("/test/project"),
        name: "cloned_project".to_string(),
        watch_patterns: vec!["**/*.py".to_string()],
        complexity_threshold: 25,
        last_analysis: None,
        started_at: std::time::SystemTime::now(),
    };

    let cloned = project.clone();
    assert_eq!(cloned.name, project.name);
    assert_eq!(cloned.complexity_threshold, project.complexity_threshold);
}

#[test]
fn test_monitored_project_debug() {
    let project = MonitoredProject {
        path: PathBuf::from("/test/project"),
        name: "debug_test".to_string(),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: std::time::SystemTime::now(),
    };

    let debug_str = format!("{:?}", project);
    assert!(debug_str.contains("debug_test"));
}

#[test]
fn test_monitored_project_with_multiple_watch_patterns() {
    let project = MonitoredProject {
        path: PathBuf::from("/test/polyglot"),
        name: "polyglot_project".to_string(),
        watch_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.py".to_string(),
            "**/*.js".to_string(),
            "**/*.ts".to_string(),
        ],
        complexity_threshold: 15,
        last_analysis: None,
        started_at: std::time::SystemTime::now(),
    };

    assert_eq!(project.watch_patterns.len(), 4);
    assert!(project.watch_patterns.contains(&"**/*.py".to_string()));
}

// === ProjectAnalysisResult Tests ===

#[test]
fn test_project_analysis_result_creation() {
    let result = ProjectAnalysisResult {
        timestamp: chrono::Utc::now().to_rfc3339(),
        quality_score: 0.92,
        files_analyzed: 25,
        functions_analyzed: 100,
        avg_complexity: 6.5,
        hotspot_functions: 2,
        satd_issues: 1,
        quality_gate_status: "PASSED".to_string(),
        recommendations: vec![
            "Good code quality".to_string(),
            "Consider adding more tests".to_string(),
        ],
    };

    assert_eq!(result.quality_score, 0.92);
    assert_eq!(result.files_analyzed, 25);
    assert_eq!(result.recommendations.len(), 2);
}

#[test]
fn test_project_analysis_result_serialization() {
    let result = ProjectAnalysisResult {
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        quality_score: 0.75,
        files_analyzed: 15,
        functions_analyzed: 60,
        avg_complexity: 10.0,
        hotspot_functions: 5,
        satd_issues: 3,
        quality_gate_status: "FAILED".to_string(),
        recommendations: vec!["Reduce complexity".to_string()],
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("quality_score"));
    assert!(json.contains("0.75"));

    let deserialized: ProjectAnalysisResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.quality_score, 0.75);
}

#[test]
fn test_project_analysis_result_clone() {
    let result = ProjectAnalysisResult {
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        quality_score: 0.90,
        files_analyzed: 10,
        functions_analyzed: 50,
        avg_complexity: 8.0,
        hotspot_functions: 2,
        satd_issues: 1,
        quality_gate_status: "PASSED".to_string(),
        recommendations: vec!["Test recommendation".to_string()],
    };

    let cloned = result.clone();
    assert_eq!(cloned.quality_score, result.quality_score);
    assert_eq!(cloned.files_analyzed, result.files_analyzed);
    assert_eq!(cloned.recommendations.len(), result.recommendations.len());
}

#[test]
fn test_project_analysis_result_debug() {
    let result = ProjectAnalysisResult {
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        quality_score: 0.85,
        files_analyzed: 10,
        functions_analyzed: 50,
        avg_complexity: 8.5,
        hotspot_functions: 3,
        satd_issues: 2,
        quality_gate_status: "PASSED".to_string(),
        recommendations: vec![],
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("quality_score"));
}

#[test]
fn test_project_analysis_result_all_fields() {
    let result = ProjectAnalysisResult {
        timestamp: "2024-06-15T12:30:45Z".to_string(),
        quality_score: 0.88,
        files_analyzed: 42,
        functions_analyzed: 156,
        avg_complexity: 12.5,
        hotspot_functions: 7,
        satd_issues: 4,
        quality_gate_status: "PASSED_WITH_WARNINGS".to_string(),
        recommendations: vec![
            "Refactor complex functions".to_string(),
            "Add missing documentation".to_string(),
            "Reduce TODO count".to_string(),
        ],
    };

    assert_eq!(result.timestamp, "2024-06-15T12:30:45Z");
    assert_eq!(result.quality_score, 0.88);
    assert_eq!(result.files_analyzed, 42);
    assert_eq!(result.functions_analyzed, 156);
    assert_eq!(result.avg_complexity, 12.5);
    assert_eq!(result.hotspot_functions, 7);
    assert_eq!(result.satd_issues, 4);
    assert_eq!(result.quality_gate_status, "PASSED_WITH_WARNINGS");
    assert_eq!(result.recommendations.len(), 3);
}

// === QualityMonitorCommand Tests ===

#[test]
fn test_quality_monitor_command_start_monitoring() {
    let project = MonitoredProject {
        path: PathBuf::from("/test/project"),
        name: "test".to_string(),
        watch_patterns: vec![],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: std::time::SystemTime::now(),
    };

    let command = QualityMonitorCommand::StartMonitoring {
        project_path: PathBuf::from("/test/project"),
        config: Box::new(project),
    };

    match command {
        QualityMonitorCommand::StartMonitoring {
            project_path,
            config,
        } => {
            assert_eq!(project_path, PathBuf::from("/test/project"));
            assert_eq!(config.name, "test");
        }
        _ => panic!("Wrong command type"),
    }
}

#[test]
fn test_quality_monitor_command_stop_monitoring() {
    let command = QualityMonitorCommand::StopMonitoring {
        project_id: "test_project".to_string(),
    };

    match command {
        QualityMonitorCommand::StopMonitoring { project_id } => {
            assert_eq!(project_id, "test_project");
        }
        _ => panic!("Wrong command type"),
    }
}

#[test]
fn test_quality_monitor_command_shutdown() {
    let command = QualityMonitorCommand::Shutdown;

    matches!(command, QualityMonitorCommand::Shutdown);
}

#[test]
fn test_quality_monitor_command_get_status() {
    let (tx, _rx) = oneshot::channel();
    let command = QualityMonitorCommand::GetStatus {
        project_id: "test_project".to_string(),
        response_tx: Box::new(tx),
    };

    match command {
        QualityMonitorCommand::GetStatus {
            project_id,
            response_tx: _,
        } => {
            assert_eq!(project_id, "test_project");
        }
        _ => panic!("Wrong command type"),
    }
}

#[test]
fn test_quality_monitor_command_debug() {
    let command = QualityMonitorCommand::Shutdown;
    let debug_str = format!("{:?}", command);
    assert!(debug_str.contains("Shutdown"));
}
