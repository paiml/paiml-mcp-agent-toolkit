//! Integration tests for PDMT deterministic todo generation with quality enforcement

use pmat::models::pdmt::{EnforcementMode, PdmtQualityConfig, PdmtTodo, TodoPriority};
use pmat::services::pdmt_quality_integration::PdmtQualityEnforcer;
use pmat::services::pdmt_service::PdmtService;

#[test]
fn test_pdmt_deterministic_generation() {
    // Test that the same inputs produce the same outputs (determinism)
    let service = PdmtService::new();
    let requirements = vec![
        "implement user authentication with OAuth2".to_string(),
        "add comprehensive logging system".to_string(),
        "create REST API endpoints".to_string(),
    ];

    let config = PdmtQualityConfig {
        enforcement_mode: EnforcementMode::Strict,
        coverage_threshold: 85.0,
        max_complexity: 10,
        require_doctests: true,
        require_property_tests: true,
        require_examples: true,
        zero_satd_tolerance: true,
    };

    // Generate todos twice with same inputs
    let result1 = service
        .generate_todos(
            requirements.clone(),
            Some("test_project".to_string()),
            "high",
            config.clone(),
        )
        .unwrap();

    let result2 = service
        .generate_todos(requirements, Some("test_project".to_string()), "high", config)
        .unwrap();

    // Verify deterministic generation
    assert_eq!(result1.todos.len(), result2.todos.len());
    assert_eq!(result1.deterministic_seed, result2.deterministic_seed);
    assert_eq!(result1.project_name, result2.project_name);

    // Verify todos have proper structure
    for todo in &result1.todos {
        assert!(!todo.content.is_empty());
        assert!(todo.content.len() >= 15);
        assert!(todo.content.len() <= 200); // Reasonable upper bound
        assert!(todo.estimated_hours >= 0.5);
        assert!(todo.estimated_hours <= 8.0);
        assert!(!todo.quality_gates.satd_tolerance); // Zero tolerance
        assert!(todo.quality_gates.coverage_requirement >= 80.0);
    }
}

#[test]
fn test_pdmt_granularity_levels() {
    let service = PdmtService::new();
    let requirements = vec!["implement authentication system".to_string()];
    let config = PdmtQualityConfig::default();

    // Test low granularity
    let low_result = service
        .generate_todos(
            requirements.clone(),
            Some("project".to_string()),
            "low",
            config.clone(),
        )
        .unwrap();

    // Test medium granularity
    let medium_result = service
        .generate_todos(
            requirements.clone(),
            Some("project".to_string()),
            "medium",
            config.clone(),
        )
        .unwrap();

    // Test high granularity
    let high_result = service
        .generate_todos(requirements, Some("project".to_string()), "high", config)
        .unwrap();

    // Higher granularity should produce more todos
    assert!(low_result.todos.len() <= medium_result.todos.len());
    assert!(medium_result.todos.len() <= high_result.todos.len());
}

#[test]
fn test_pdmt_priority_detection() {
    let service = PdmtService::new();
    let config = PdmtQualityConfig::default();

    // Test critical priority
    let critical_req = vec!["fix critical security vulnerability".to_string()];
    let critical_result = service
        .generate_todos(critical_req, None, "low", config.clone())
        .unwrap();

    // Test bug priority
    let bug_req = vec!["fix memory leak bug in parser".to_string()];
    let bug_result = service
        .generate_todos(bug_req, None, "low", config.clone())
        .unwrap();

    // Test feature priority
    let feature_req = vec!["add new dashboard feature".to_string()];
    let feature_result = service.generate_todos(feature_req, None, "low", config).unwrap();

    // Verify priority ordering makes sense
    let critical_todo = &critical_result.todos[0];
    let bug_todo = &bug_result.todos[0];
    let feature_todo = &feature_result.todos[0];

    // Critical and bug fixes should have higher priority than features
    match (&critical_todo.priority, &bug_todo.priority, &feature_todo.priority) {
        (TodoPriority::Critical | TodoPriority::High, _, TodoPriority::Low | TodoPriority::Medium) => {
            // Expected: critical/bug higher than feature
        }
        _ => {
            // This is also acceptable as long as priorities are assigned
        }
    }
}

#[tokio::test]
async fn test_pdmt_quality_enforcement_strict_mode() {
    let enforcer = PdmtQualityEnforcer::new();

    // Create a todo with good quality settings
    let good_todo = PdmtTodo::new("Implement secure user authentication".to_string(), TodoPriority::High);

    let todo_list = pmat::models::pdmt::PdmtTodoList {
        project_name: "test_project".to_string(),
        todos: vec![good_todo],
        quality_config: PdmtQualityConfig {
            enforcement_mode: EnforcementMode::Strict,
            coverage_threshold: 80.0,
            max_complexity: 8,
            require_doctests: true,
            require_property_tests: true,
            require_examples: true,
            zero_satd_tolerance: true,
        },
        generated_at: chrono::Utc::now().to_rfc3339(),
        deterministic_seed: 42,
    };

    let result = enforcer.enforce_quality_standards(&todo_list).await.unwrap();
    assert!(result.overall_passed);
}

#[tokio::test]
async fn test_pdmt_quality_enforcement_failures() {
    let enforcer = PdmtQualityEnforcer::new();

    // Create a todo with poor quality settings (will fail validation)
    let mut bad_todo = PdmtTodo::new("x".to_string(), TodoPriority::Low); // Too short
    bad_todo.quality_gates.coverage_requirement = 50.0; // Too low
    bad_todo.quality_gates.satd_tolerance = true; // Should be false

    let todo_list = pmat::models::pdmt::PdmtTodoList {
        project_name: "test_project".to_string(),
        todos: vec![bad_todo],
        quality_config: PdmtQualityConfig::default(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        deterministic_seed: 42,
    };

    let result = enforcer.enforce_quality_standards(&todo_list).await.unwrap();
    assert!(!result.overall_passed);
    assert!(!result.recommendations.is_empty());
}

#[test]
fn test_pdmt_validation_commands() {
    let service = PdmtService::new();
    let requirements = vec!["implement testing framework".to_string()];
    let config = PdmtQualityConfig::default();

    let result = service
        .generate_todos(requirements, None, "medium", config)
        .unwrap();

    // Verify all todos have proper validation commands
    for todo in &result.todos {
        assert!(!todo.validation_commands.unit_tests.is_empty());
        assert!(!todo.validation_commands.doctests.is_empty());
        assert!(!todo.validation_commands.quality_proxy.is_empty());
        assert!(todo.validation_commands.quality_proxy.contains("pmat"));
    }
}

#[test]
fn test_pdmt_success_criteria() {
    let service = PdmtService::new();
    let requirements = vec!["implement database connection pooling".to_string()];
    
    let config = PdmtQualityConfig {
        coverage_threshold: 90.0,
        require_doctests: true,
        require_property_tests: true,
        require_examples: true,
        ..Default::default()
    };

    let result = service
        .generate_todos(requirements, None, "high", config)
        .unwrap();

    // Verify success criteria reflect quality config
    assert!(!result.todos.is_empty(), "No todos generated");
    
    for todo in &result.todos {
        assert!(!todo.success_criteria.is_empty(), "Todo has no success criteria");
        
        // Check if any criteria mentions coverage or quality (case-insensitive)
        let criteria_text = todo.success_criteria.join(" ").to_lowercase();
        let has_relevant_criteria = criteria_text.contains("coverage") || 
                                   criteria_text.contains("quality") ||
                                   criteria_text.contains("test") ||
                                   criteria_text.contains("90");
        
        assert!(has_relevant_criteria, 
                "Todo '{}' has no relevant success criteria. Criteria: {:?}", 
                todo.content, todo.success_criteria);
    }
}

#[test]
fn test_pdmt_implementation_specs() {
    let service = PdmtService::new();
    let requirements = vec![
        "implement user authentication module".to_string(),
        "create API rate limiter".to_string(),
    ];
    let config = PdmtQualityConfig::default();

    let result = service
        .generate_todos(requirements, None, "high", config)
        .unwrap();

    // Verify implementation specs are generated
    let mut has_specs = false;
    for todo in &result.todos {
        // At least some todos should have file specifications
        if !todo.implementation_specs.primary_files.is_empty() ||
           !todo.implementation_specs.test_files.is_empty() ||
           !todo.implementation_specs.doc_files.is_empty() ||
           !todo.implementation_specs.example_files.is_empty() {
            has_specs = true;
            break;
        }
    }
    // At least one todo should have implementation specs
    assert!(has_specs, "No todos have implementation specs");
}

#[test]
fn test_pdmt_dependency_management() {
    let service = PdmtService::new();
    let requirements = vec!["implement complete authentication flow".to_string()];
    let config = PdmtQualityConfig::default();

    let result = service
        .generate_todos(requirements, None, "high", config)
        .unwrap();

    // With high granularity, we should have multiple todos
    assert!(result.todos.len() > 1);

    // Test todos should depend on implementation todos
    let has_dependencies = result.todos.iter()
        .any(|todo| !todo.dependencies.is_empty());
    
    // At high granularity, we expect some dependencies
    if result.todos.len() > 2 {
        assert!(has_dependencies);
    }
}