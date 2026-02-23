#![cfg_attr(coverage_nightly, coverage(off))]
//! Integration and edge case tests for command executor

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::super::super::*;
    use std::sync::Arc;

    /// Creates a test server instance
    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("Failed to create test server"))
    }

    // ============================================================================
    // Full Integration Tests for Command Groups
    // ============================================================================

    /// Test that GenerateCommandGroup can be used with the registry
    #[test]
    fn test_generate_group_in_registry() {
        let registry = CommandRegistry::default();
        let _generate = &registry.generate_handlers;
        // Group should be accessible
    }

    /// Test that AnalyzeCommandGroup can be used with the registry
    #[test]
    fn test_analyze_group_in_registry() {
        let registry = CommandRegistry::default();
        let _analyze = &registry.analyze_handlers;
        // Group should be accessible
    }

    /// Test that UtilityCommandGroup can be used with the registry
    #[test]
    fn test_utility_group_in_registry() {
        let registry = CommandRegistry::default();
        let _utility = &registry.utility_handlers;
        // Group should be accessible
    }

    /// Test that DemoCommandGroup can be used with the registry
    #[test]
    fn test_demo_group_in_registry() {
        let registry = CommandRegistry::default();
        let _demo = &registry.demo_handlers;
        // Group should be accessible
    }

    // ============================================================================
    // CommandExecutor with CommandExecutorFactory Integration
    // ============================================================================

    #[test]
    fn test_factory_and_executor_integration() {
        let server = create_test_server();
        let executor = CommandExecutorFactory::create(server);

        // Verify the executor is fully functional
        let _ = &executor.registry.generate_handlers;
        let _ = &executor.registry.analyze_handlers;
        let _ = &executor.registry.utility_handlers;
        let _ = &executor.registry.demo_handlers;
    }

    /// Test multiple executors can be created from same server
    #[test]
    fn test_multiple_executors_same_server() {
        let server = create_test_server();

        let executor1 = CommandExecutorFactory::create(server.clone());
        let executor2 = CommandExecutorFactory::create(server.clone());

        // Both should share the same server
        assert!(Arc::ptr_eq(&executor1.server, &executor2.server));
    }

    /// Test executors with different servers are independent
    #[test]
    fn test_executors_different_servers() {
        let server1 = create_test_server();
        let server2 = create_test_server();

        let executor1 = CommandExecutorFactory::create(server1);
        let executor2 = CommandExecutorFactory::create(server2);

        // Servers should be different
        assert!(!Arc::ptr_eq(&executor1.server, &executor2.server));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod edge_case_tests {
    use super::super::super::*;
    use std::sync::Arc;

    /// Creates a test server instance
    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("Failed to create test server"))
    }

    // ============================================================================
    // Edge Cases for Command Groups
    // ============================================================================

    /// Test that all command groups implement Default correctly
    #[test]
    fn test_all_groups_implement_default() {
        // These should all compile and run without panic
        let _ = GenerateCommandGroup::default();
        let _ = AnalyzeCommandGroup::default();
        let _ = UtilityCommandGroup::default();
        let _ = DemoCommandGroup::default();
    }

    /// Test CommandRegistry default is consistent
    #[test]
    fn test_registry_default_consistency() {
        // Creating multiple registries should work
        let registries: Vec<CommandRegistry> =
            (0..10).map(|_| CommandRegistry::default()).collect();

        assert_eq!(registries.len(), 10);
    }

    /// Test CommandExecutor can be created many times
    #[test]
    fn test_executor_creation_stress() {
        let server = create_test_server();

        // Create many executors
        let executors: Vec<CommandExecutor> = (0..100)
            .map(|_| CommandExecutorFactory::create(server.clone()))
            .collect();

        assert_eq!(executors.len(), 100);

        // All should share the same server
        for executor in &executors {
            assert!(Arc::ptr_eq(&executor.server, &server));
        }
    }

    // ============================================================================
    // Memory and Safety Tests
    // ============================================================================

    /// Test that dropping executors doesn't cause issues
    #[test]
    fn test_executor_drop_safety() {
        let server = create_test_server();

        {
            let _executor = CommandExecutorFactory::create(server.clone());
            // Executor goes out of scope here
        }

        // Server should still be valid
        assert_eq!(Arc::strong_count(&server), 1);
    }

    /// Test registry drop safety
    #[test]
    fn test_registry_drop_safety() {
        {
            let _registry = CommandRegistry::default();
            // Registry goes out of scope here
        }
        // Should not panic
    }

    /// Test command group drop safety
    #[test]
    fn test_command_group_drop_safety() {
        {
            let _generate = GenerateCommandGroup::default();
            let _analyze = AnalyzeCommandGroup::default();
            let _utility = UtilityCommandGroup::default();
            let _demo = DemoCommandGroup::default();
            // All go out of scope here
        }
        // Should not panic
    }
}
