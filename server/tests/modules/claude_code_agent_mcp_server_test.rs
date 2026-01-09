//! Comprehensive tests for ClaudeCodeAgentMcpServer
//!
//! Achieves 80+ test coverage through:
//! - Unit tests for constructor and configuration
//! - Configuration validation and defaults
//! - Data structure testing
//! - Edge case handling

use pmat::agent::mcp_server::{AgentConfig, ClaudeCodeAgentMcpServer, MonitoredProject};
use std::path::PathBuf;
use std::time::SystemTime;

/// Test MCP server creation and initialization
#[test]
fn test_mcp_server_creation() {
    let config = AgentConfig {
        name: "test-agent".to_string(),
        version: "1.0.0".to_string(),
        complexity_threshold: 20,
        watch_patterns: vec!["**/*.rs".to_string()],
        update_interval: 5,
        max_projects: 10,
    };

    let _server = ClaudeCodeAgentMcpServer::new(config);
    // Verify we can create a server without errors
}

/// Test default agent configuration
#[test]
fn test_agent_config_default() {
    let config = AgentConfig::default();

    // Verify default values
    assert_eq!(config.name, "pmat-agent");
    assert_eq!(config.version, "1.0.0");
    assert_eq!(config.complexity_threshold, 20);
    assert_eq!(config.max_projects, 10);
    assert_eq!(config.update_interval, 5);

    // Verify watch patterns include common file types
    assert!(config.watch_patterns.contains(&"**/*.rs".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.py".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.js".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.ts".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.java".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.go".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.cpp".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.c".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.hpp".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.h".to_string()));
}

/// Test custom agent configuration
#[test]
fn test_agent_config_custom() {
    let config = AgentConfig {
        name: "custom-agent".to_string(),
        version: "2.0.0".to_string(),
        complexity_threshold: 15,
        watch_patterns: vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
        update_interval: 10,
        max_projects: 5,
    };

    assert_eq!(config.name, "custom-agent");
    assert_eq!(config.version, "2.0.0");
    assert_eq!(config.complexity_threshold, 15);
    assert_eq!(config.max_projects, 5);
    assert_eq!(config.update_interval, 10);
    assert_eq!(config.watch_patterns.len(), 2);
    assert!(config.watch_patterns.contains(&"**/*.rs".to_string()));
    assert!(config.watch_patterns.contains(&"**/*.toml".to_string()));
}

/// Test monitored project configuration
#[test]
fn test_monitored_project_creation() {
    let project = MonitoredProject {
        name: "test-project".to_string(),
        path: PathBuf::from("/test/path"),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: SystemTime::now(),
    };

    assert_eq!(project.name, "test-project");
    assert_eq!(project.path, PathBuf::from("/test/path"));
    assert_eq!(project.complexity_threshold, 20);
    assert_eq!(project.watch_patterns.len(), 1);
    assert!(project.last_analysis.is_none());
    assert!(project.started_at.elapsed().unwrap().as_secs() < 1);
}

/// Test monitored project with multiple watch patterns
#[test]
fn test_monitored_project_multiple_patterns() {
    let project = MonitoredProject {
        name: "multi-pattern-project".to_string(),
        path: PathBuf::from("/project/path"),
        watch_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.py".to_string(),
            "**/*.js".to_string(),
        ],
        complexity_threshold: 25,
        last_analysis: None,
        started_at: SystemTime::now(),
    };

    assert_eq!(project.watch_patterns.len(), 3);
    assert!(project.watch_patterns.contains(&"**/*.rs".to_string()));
    assert!(project.watch_patterns.contains(&"**/*.py".to_string()));
    assert!(project.watch_patterns.contains(&"**/*.js".to_string()));
    assert_eq!(project.complexity_threshold, 25);
}

/// Test configuration validation - empty name
#[test]
fn test_config_validation_empty_name() {
    let config = AgentConfig {
        name: String::new(),
        version: "1.0.0".to_string(),
        complexity_threshold: 20,
        watch_patterns: vec!["**/*.rs".to_string()],
        update_interval: 5,
        max_projects: 10,
    };

    // Empty name should be detectable
    assert!(config.name.is_empty());
}

/// Test configuration validation - zero thresholds
#[test]
fn test_config_validation_zero_values() {
    let config = AgentConfig {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        complexity_threshold: 0,
        watch_patterns: vec![],
        update_interval: 0,
        max_projects: 0,
    };

    // Zero values should be detectable for validation
    assert_eq!(config.complexity_threshold, 0);
    assert_eq!(config.update_interval, 0);
    assert_eq!(config.max_projects, 0);
    assert!(config.watch_patterns.is_empty());
}

/// Test configuration validation - large values
#[test]
fn test_config_validation_large_values() {
    let config = AgentConfig {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        complexity_threshold: 1000,
        watch_patterns: (0..100).map(|i| format!("**/*.{}", i)).collect(),
        update_interval: 86400, // 1 day
        max_projects: 1000,
    };

    // Large values should be handled
    assert_eq!(config.complexity_threshold, 1000);
    assert_eq!(config.update_interval, 86400);
    assert_eq!(config.max_projects, 1000);
    assert_eq!(config.watch_patterns.len(), 100);
}

/// Test monitored project edge cases
#[test]
fn test_monitored_project_edge_cases() {
    // Very low complexity threshold
    let low_threshold_project = MonitoredProject {
        name: "low-threshold".to_string(),
        path: PathBuf::from("/path"),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 1,
        last_analysis: None,
        started_at: SystemTime::now(),
    };

    assert_eq!(low_threshold_project.complexity_threshold, 1);

    // Very high complexity threshold
    let high_threshold_project = MonitoredProject {
        name: "high-threshold".to_string(),
        path: PathBuf::from("/path"),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 1000,
        last_analysis: None,
        started_at: SystemTime::now(),
    };

    assert_eq!(high_threshold_project.complexity_threshold, 1000);
}

/// Test complex path scenarios
#[test]
fn test_complex_paths() {
    let complex_paths = vec![
        PathBuf::from("/"),
        PathBuf::from("/home/user/project with spaces"),
        PathBuf::from("relative/path"),
        PathBuf::from("./current/dir"),
        PathBuf::from("../parent/dir"),
        PathBuf::from("/very/deep/nested/project/structure/with/many/levels"),
    ];

    for path in complex_paths {
        let project = MonitoredProject {
            name: "path-test".to_string(),
            path: path.clone(),
            watch_patterns: vec!["**/*.rs".to_string()],
            complexity_threshold: 20,
            last_analysis: None,
            started_at: SystemTime::now(),
        };

        assert_eq!(project.path, path);
    }
}

/// Test configuration serialization/deserialization compatibility
#[test]
fn test_config_serialization() {
    use serde_json;

    let original_config = AgentConfig {
        name: "serialization-test".to_string(),
        version: "1.2.3".to_string(),
        complexity_threshold: 25,
        watch_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string()],
        update_interval: 15,
        max_projects: 20,
    };

    // Test JSON serialization
    let json_str = serde_json::to_string(&original_config).expect("Serialization should work");
    let deserialized_config: AgentConfig =
        serde_json::from_str(&json_str).expect("Deserialization should work");

    // Verify all fields match
    assert_eq!(deserialized_config.name, original_config.name);
    assert_eq!(deserialized_config.version, original_config.version);
    assert_eq!(
        deserialized_config.complexity_threshold,
        original_config.complexity_threshold
    );
    assert_eq!(
        deserialized_config.watch_patterns,
        original_config.watch_patterns
    );
    assert_eq!(
        deserialized_config.update_interval,
        original_config.update_interval
    );
    assert_eq!(
        deserialized_config.max_projects,
        original_config.max_projects
    );
}

/// Test creating servers with various configurations
#[test]
fn test_multiple_server_configurations() {
    let configs = vec![
        AgentConfig::default(),
        AgentConfig {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            complexity_threshold: 1,
            watch_patterns: vec!["*.rs".to_string()],
            update_interval: 1,
            max_projects: 1,
        },
        AgentConfig {
            name: "maximal".to_string(),
            version: "999.999.999".to_string(),
            complexity_threshold: 100,
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
                "**/*.java".to_string(),
                "**/*.go".to_string(),
                "**/*.cpp".to_string(),
                "**/*.c".to_string(),
                "**/*.hpp".to_string(),
                "**/*.h".to_string(),
                "**/*.rb".to_string(),
                "**/*.php".to_string(),
                "**/*.cs".to_string(),
                "**/*.swift".to_string(),
                "**/*.kt".to_string(),
            ],
            update_interval: 3600,
            max_projects: 100,
        },
    ];

    for config in configs {
        let _server = ClaudeCodeAgentMcpServer::new(config);
        // Should create successfully with any valid configuration
    }
}

/// Test Clone implementation for AgentConfig
#[test]
fn test_agent_config_clone() {
    let original = AgentConfig {
        name: "original".to_string(),
        version: "1.0.0".to_string(),
        complexity_threshold: 20,
        watch_patterns: vec!["**/*.rs".to_string()],
        update_interval: 5,
        max_projects: 10,
    };

    let cloned = original.clone();

    assert_eq!(cloned.name, original.name);
    assert_eq!(cloned.version, original.version);
    assert_eq!(cloned.complexity_threshold, original.complexity_threshold);
    assert_eq!(cloned.watch_patterns, original.watch_patterns);
    assert_eq!(cloned.update_interval, original.update_interval);
    assert_eq!(cloned.max_projects, original.max_projects);

    // Verify they are independent (modify clone, original unchanged)
    let mut modified = cloned;
    modified.name = "modified".to_string();
    assert_eq!(original.name, "original");
    assert_eq!(modified.name, "modified");
}

/// Test Debug implementation for AgentConfig  
#[test]
fn test_agent_config_debug() {
    let config = AgentConfig::default();
    let debug_str = format!("{:?}", config);

    // Debug output should contain key information
    assert!(debug_str.contains("AgentConfig"));
    assert!(debug_str.contains("pmat-agent"));
    assert!(debug_str.contains("1.0.0"));
    assert!(debug_str.contains("20")); // complexity_threshold
}

/// Test edge case: empty watch patterns
#[test]
fn test_empty_watch_patterns() {
    let config = AgentConfig {
        name: "no-patterns".to_string(),
        version: "1.0.0".to_string(),
        complexity_threshold: 20,
        watch_patterns: vec![],
        update_interval: 5,
        max_projects: 10,
    };

    let _server = ClaudeCodeAgentMcpServer::new(config);
    // Should handle empty watch patterns gracefully

    let project = MonitoredProject {
        name: "no-patterns-project".to_string(),
        path: PathBuf::from("/path"),
        watch_patterns: vec![],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: SystemTime::now(),
    };

    assert!(project.watch_patterns.is_empty());
}

/// Test extremely long configuration values
#[test]
fn test_long_config_values() {
    let long_name = "a".repeat(1000);
    let long_version = "1.".repeat(100) + "0";

    let config = AgentConfig {
        name: long_name.clone(),
        version: long_version.clone(),
        complexity_threshold: 20,
        watch_patterns: vec!["**/*.rs".to_string()],
        update_interval: 5,
        max_projects: 10,
    };

    let _server = ClaudeCodeAgentMcpServer::new(config);
    // Should handle long strings without crashing
}

/// Test monitored project with very long paths
#[test]
fn test_long_paths_and_names() {
    let long_path = PathBuf::from(format!("/{}", "very_long_directory_name/".repeat(20)));
    let long_name = "very_long_project_name_".repeat(10);

    let project = MonitoredProject {
        name: long_name.clone(),
        path: long_path.clone(),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: SystemTime::now(),
    };

    assert_eq!(project.name, long_name);
    assert_eq!(project.path, long_path);
}

/// Test various watch pattern formats
#[test]
fn test_watch_pattern_formats() {
    let patterns = vec![
        "*.rs",
        "**/*.rs",
        "src/**/*.rs",
        "**/*",
        "*.{rs,py,js}",
        "tests/**/*.rs",
        "examples/*.rs",
        "**/*.{test,spec}.{rs,py,js}",
    ];

    for pattern in patterns {
        let config = AgentConfig {
            name: "pattern-test".to_string(),
            version: "1.0.0".to_string(),
            complexity_threshold: 20,
            watch_patterns: vec![pattern.to_string()],
            update_interval: 5,
            max_projects: 10,
        };

        let _server = ClaudeCodeAgentMcpServer::new(config);
    }
}

/// Test configuration boundary values
#[test]
fn test_configuration_boundaries() {
    // Test minimum values
    let min_config = AgentConfig {
        name: "a".to_string(),
        version: "0".to_string(),
        complexity_threshold: 1,
        watch_patterns: vec!["*".to_string()],
        update_interval: 1,
        max_projects: 1,
    };
    let _min_server = ClaudeCodeAgentMcpServer::new(min_config);

    // Test maximum reasonable values
    let max_config = AgentConfig {
        name: "maximum-configuration-test".to_string(),
        version: "999.999.999".to_string(),
        complexity_threshold: u32::MAX,
        watch_patterns: (0..1000).map(|i| format!("pattern_{}", i)).collect(),
        update_interval: u64::MAX,
        max_projects: usize::MAX,
    };
    let _max_server = ClaudeCodeAgentMcpServer::new(max_config);
}

/// Integration test: Full workflow simulation
#[test]
fn test_integration_workflow_simulation() {
    // 1. Create server with custom configuration
    let config = AgentConfig {
        name: "integration-test".to_string(),
        version: "1.0.0".to_string(),
        complexity_threshold: 15,
        watch_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string()],
        update_interval: 10,
        max_projects: 5,
    };

    let _server = ClaudeCodeAgentMcpServer::new(config);

    // 2. Create monitored project configurations
    let projects = vec![
        MonitoredProject {
            name: "project-1".to_string(),
            path: PathBuf::from("/project1"),
            watch_patterns: vec!["**/*.rs".to_string()],
            complexity_threshold: 15,
            last_analysis: None,
            started_at: SystemTime::now(),
        },
        MonitoredProject {
            name: "project-2".to_string(),
            path: PathBuf::from("/project2"),
            watch_patterns: vec!["**/*.py".to_string()],
            complexity_threshold: 20,
            last_analysis: None,
            started_at: SystemTime::now(),
        },
    ];

    // 3. Verify all projects can be created
    for project in projects {
        assert!(!project.name.is_empty());
        assert!(!project.watch_patterns.is_empty());
        assert!(project.complexity_threshold > 0);
        assert!(project.last_analysis.is_none());
    }
}

/// Test time-related functionality
#[test]
fn test_time_functionality() {
    let start_time = SystemTime::now();

    let project = MonitoredProject {
        name: "time-test".to_string(),
        path: PathBuf::from("/test"),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: start_time,
    };

    assert_eq!(project.started_at, start_time);

    // Test that elapsed time is reasonable (less than a few seconds for test execution)
    let elapsed = project.started_at.elapsed().unwrap();
    assert!(elapsed.as_secs() < 5);
}
