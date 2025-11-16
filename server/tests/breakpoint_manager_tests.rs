// TRACE-002: Breakpoint Management System
// RED Phase Tests - Sprint 71
//
// Tests for advanced breakpoint management with validation,
// conditional breakpoints, and hit count tracking

use serde_json::json;

// RED Phase Test 1: Set and Remove Breakpoints
#[test]
fn test_set_breakpoint_in_rust_file() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    let result = mgr.set_breakpoint(bp.clone());
    assert!(result.is_ok(), "Setting valid breakpoint should succeed");
    assert_eq!(mgr.count(), 1, "Should have 1 breakpoint");
    assert!(
        mgr.has_breakpoint(&bp.source, bp.line),
        "Should find breakpoint at line 10"
    );
}

// RED Phase Test 2: Remove Breakpoint
#[test]
fn test_remove_breakpoint() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp.clone()).unwrap();
    assert_eq!(mgr.count(), 1);

    let result = mgr.remove_breakpoint(&bp.source, bp.line);
    assert!(
        result.is_ok(),
        "Removing existing breakpoint should succeed"
    );
    assert_eq!(mgr.count(), 0, "Should have 0 breakpoints after removal");
    assert!(
        !mgr.has_breakpoint(&bp.source, bp.line),
        "Should not find breakpoint after removal"
    );
}

// RED Phase Test 3: Multiple Breakpoints in Same File
#[test]
fn test_multiple_breakpoints_same_file() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp1 = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    let bp2 = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 20,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp1.clone()).unwrap();
    mgr.set_breakpoint(bp2.clone()).unwrap();

    assert_eq!(mgr.count(), 2, "Should have 2 breakpoints");
    assert!(mgr.has_breakpoint(&bp1.source, bp1.line));
    assert!(mgr.has_breakpoint(&bp2.source, bp2.line));
}

// RED Phase Test 4: Breakpoints in Multiple Files
#[test]
fn test_breakpoints_multiple_files() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp1 = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    let bp2 = pmat::services::dap::Breakpoint {
        source: "src/lib.rs".to_string(),
        line: 15,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp1.clone()).unwrap();
    mgr.set_breakpoint(bp2.clone()).unwrap();

    assert_eq!(mgr.count(), 2);
    assert_eq!(mgr.breakpoints_in_file(&bp1.source).len(), 1);
    assert_eq!(mgr.breakpoints_in_file(&bp2.source).len(), 1);
}

// RED Phase Test 5: Clear All Breakpoints
#[test]
fn test_clear_all_breakpoints() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    mgr.set_breakpoint(pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    })
    .unwrap();

    mgr.set_breakpoint(pmat::services::dap::Breakpoint {
        source: "src/lib.rs".to_string(),
        line: 15,
        column: None,
        condition: None,
    })
    .unwrap();

    assert_eq!(mgr.count(), 2);

    mgr.clear_all();
    assert_eq!(mgr.count(), 0, "Should have 0 breakpoints after clear");
}

// RED Phase Test 6: Clear Breakpoints in Specific File
#[test]
fn test_clear_file_breakpoints() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    mgr.set_breakpoint(pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    })
    .unwrap();

    mgr.set_breakpoint(pmat::services::dap::Breakpoint {
        source: "src/lib.rs".to_string(),
        line: 15,
        column: None,
        condition: None,
    })
    .unwrap();

    mgr.clear_file("src/main.rs");

    assert_eq!(
        mgr.count(),
        1,
        "Should have 1 breakpoint after clearing main.rs"
    );
    assert_eq!(mgr.breakpoints_in_file("src/lib.rs").len(), 1);
    assert_eq!(mgr.breakpoints_in_file("src/main.rs").len(), 0);
}

// RED Phase Test 7: Conditional Breakpoint - Simple Expression
#[test]
fn test_conditional_breakpoint_simple() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: Some("x > 5".to_string()),
    };

    mgr.set_breakpoint(bp.clone()).unwrap();

    // Get breakpoint and check condition
    let retrieved = mgr.get_breakpoint(&bp.source, bp.line);
    assert!(retrieved.is_some(), "Should find breakpoint");
    assert_eq!(retrieved.unwrap().condition, Some("x > 5".to_string()));
}

// RED Phase Test 8: Conditional Breakpoint - Evaluation
#[test]
fn test_conditional_breakpoint_evaluation() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: Some("count == 3".to_string()),
    };

    mgr.set_breakpoint(bp.clone()).unwrap();

    // Simulate hitting breakpoint with variables
    let variables = json!({
        "count": 3
    });

    let should_break = mgr.should_break(&bp.source, bp.line, Some(&variables));
    assert!(should_break, "Should break when condition is true");

    // Different value
    let variables2 = json!({
        "count": 5
    });

    let should_not_break = mgr.should_break(&bp.source, bp.line, Some(&variables2));
    assert!(
        !should_not_break,
        "Should not break when condition is false"
    );
}

// RED Phase Test 9: Unconditional Breakpoint Always Breaks
#[test]
fn test_unconditional_breakpoint_always_breaks() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp.clone()).unwrap();

    // Should always break regardless of variables
    let should_break1 = mgr.should_break(&bp.source, bp.line, None);
    assert!(
        should_break1,
        "Unconditional breakpoint should always break (no vars)"
    );

    let variables = json!({"x": 10});
    let should_break2 = mgr.should_break(&bp.source, bp.line, Some(&variables));
    assert!(
        should_break2,
        "Unconditional breakpoint should always break (with vars)"
    );
}

// RED Phase Test 10: Hit Count Tracking
#[test]
fn test_hit_count_tracking() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp.clone()).unwrap();

    // Initial hit count should be 0
    assert_eq!(mgr.get_hit_count(&bp.source, bp.line), 0);

    // Simulate hitting breakpoint
    mgr.record_hit(&bp.source, bp.line);
    assert_eq!(mgr.get_hit_count(&bp.source, bp.line), 1);

    mgr.record_hit(&bp.source, bp.line);
    assert_eq!(mgr.get_hit_count(&bp.source, bp.line), 2);

    mgr.record_hit(&bp.source, bp.line);
    assert_eq!(mgr.get_hit_count(&bp.source, bp.line), 3);
}

// RED Phase Test 11: Hit Count Reset on Removal
#[test]
fn test_hit_count_reset_on_removal() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp.clone()).unwrap();
    mgr.record_hit(&bp.source, bp.line);
    mgr.record_hit(&bp.source, bp.line);
    assert_eq!(mgr.get_hit_count(&bp.source, bp.line), 2);

    // Remove and re-add
    mgr.remove_breakpoint(&bp.source, bp.line).unwrap();
    mgr.set_breakpoint(bp.clone()).unwrap();

    // Hit count should be reset
    assert_eq!(mgr.get_hit_count(&bp.source, bp.line), 0);
}

// RED Phase Test 12: Concurrent Breakpoint Access
#[test]
fn test_concurrent_breakpoint_access() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let mgr = Arc::new(Mutex::new(pmat::services::dap::BreakpointManager::new()));
    let mut handles = vec![];

    // Spawn 10 threads setting breakpoints
    for i in 0..10 {
        let mgr_clone = Arc::clone(&mgr);
        let handle = thread::spawn(move || {
            let mut mgr = mgr_clone.lock().unwrap();
            let bp = pmat::services::dap::Breakpoint {
                source: "src/main.rs".to_string(),
                line: 10 + i as i64,
                column: None,
                condition: None,
            };
            mgr.set_breakpoint(bp).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let mgr = mgr.lock().unwrap();
    assert_eq!(
        mgr.count(),
        10,
        "Should have 10 breakpoints from concurrent access"
    );
}

// RED Phase Test 13: Get All Breakpoints
#[test]
fn test_get_all_breakpoints() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    mgr.set_breakpoint(pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    })
    .unwrap();

    mgr.set_breakpoint(pmat::services::dap::Breakpoint {
        source: "src/lib.rs".to_string(),
        line: 15,
        column: None,
        condition: Some("x > 0".to_string()),
    })
    .unwrap();

    let all_bps = mgr.all_breakpoints();
    assert_eq!(all_bps.len(), 2, "Should return all breakpoints");
}

// RED Phase Test 14: Duplicate Breakpoint Handling
#[test]
fn test_duplicate_breakpoint_handling() {
    let mut mgr = pmat::services::dap::BreakpointManager::new();

    let bp = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp.clone()).unwrap();
    assert_eq!(mgr.count(), 1);

    // Setting same breakpoint again should update, not duplicate
    let bp_updated = pmat::services::dap::Breakpoint {
        source: "src/main.rs".to_string(),
        line: 10,
        column: None,
        condition: Some("x > 5".to_string()),
    };

    mgr.set_breakpoint(bp_updated.clone()).unwrap();
    assert_eq!(
        mgr.count(),
        1,
        "Should still have 1 breakpoint (updated, not duplicated)"
    );

    // Check condition was updated
    let retrieved = mgr.get_breakpoint(&bp.source, bp.line);
    assert_eq!(retrieved.unwrap().condition, Some("x > 5".to_string()));
}

// RED Phase Test 15: Performance - Set 1000 Breakpoints
#[test]
fn test_performance_many_breakpoints() {
    use std::time::Instant;

    let mut mgr = pmat::services::dap::BreakpointManager::new();
    let start = Instant::now();

    // Set 1000 breakpoints
    for i in 0..1000 {
        let bp = pmat::services::dap::Breakpoint {
            source: format!("src/file_{}.rs", i / 100),
            line: i,
            column: None,
            condition: None,
        };
        mgr.set_breakpoint(bp).unwrap();
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "Setting 1000 breakpoints should take less than 100ms, took {:?}",
        duration
    );
    assert_eq!(mgr.count(), 1000);
}
