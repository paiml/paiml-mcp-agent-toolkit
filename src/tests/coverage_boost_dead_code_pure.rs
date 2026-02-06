#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for dead_code_analyzer pure functions
//! Target: Extract and test pure functions from dead_code_analyzer.rs

use crate::services::dead_code_analyzer::{
    calculate_dead_percentage, classify_dead_functions_pure, collect_functions_from_context,
    compute_reachability, detect_function_calls_in_lines,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// compute_reachability tests
// ============================================================================

#[test]
fn test_compute_reachability_empty_graph() {
    let entry_points = HashSet::new();
    let function_calls = HashMap::new();

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert!(reachable.is_empty());
}

#[test]
fn test_compute_reachability_single_entry_no_calls() {
    let mut entry_points = HashSet::new();
    entry_points.insert("main".to_string());
    let function_calls = HashMap::new();

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert_eq!(reachable.len(), 1);
    assert!(reachable.contains("main"));
}

#[test]
fn test_compute_reachability_linear_chain() {
    let mut entry_points = HashSet::new();
    entry_points.insert("main".to_string());

    let mut function_calls = HashMap::new();
    function_calls.insert("main".to_string(), {
        let mut s = HashSet::new();
        s.insert("foo".to_string());
        s
    });
    function_calls.insert("foo".to_string(), {
        let mut s = HashSet::new();
        s.insert("bar".to_string());
        s
    });
    function_calls.insert("bar".to_string(), {
        let mut s = HashSet::new();
        s.insert("baz".to_string());
        s
    });

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert_eq!(reachable.len(), 4);
    assert!(reachable.contains("main"));
    assert!(reachable.contains("foo"));
    assert!(reachable.contains("bar"));
    assert!(reachable.contains("baz"));
}

#[test]
fn test_compute_reachability_with_cycle() {
    let mut entry_points = HashSet::new();
    entry_points.insert("main".to_string());

    let mut function_calls = HashMap::new();
    function_calls.insert("main".to_string(), {
        let mut s = HashSet::new();
        s.insert("a".to_string());
        s
    });
    function_calls.insert("a".to_string(), {
        let mut s = HashSet::new();
        s.insert("b".to_string());
        s
    });
    function_calls.insert("b".to_string(), {
        let mut s = HashSet::new();
        s.insert("a".to_string()); // cycle back
        s
    });

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert_eq!(reachable.len(), 3);
    assert!(reachable.contains("main"));
    assert!(reachable.contains("a"));
    assert!(reachable.contains("b"));
}

#[test]
fn test_compute_reachability_unreachable_functions() {
    let mut entry_points = HashSet::new();
    entry_points.insert("main".to_string());

    let mut function_calls = HashMap::new();
    function_calls.insert("main".to_string(), {
        let mut s = HashSet::new();
        s.insert("used".to_string());
        s
    });
    // "unused" and "orphan" are not called by anyone reachable
    function_calls.insert("unused".to_string(), {
        let mut s = HashSet::new();
        s.insert("orphan".to_string());
        s
    });

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains("main"));
    assert!(reachable.contains("used"));
    assert!(!reachable.contains("unused"));
    assert!(!reachable.contains("orphan"));
}

#[test]
fn test_compute_reachability_multiple_entry_points() {
    let mut entry_points = HashSet::new();
    entry_points.insert("main".to_string());
    entry_points.insert("test_entry".to_string());

    let mut function_calls = HashMap::new();
    function_calls.insert("main".to_string(), {
        let mut s = HashSet::new();
        s.insert("helper1".to_string());
        s
    });
    function_calls.insert("test_entry".to_string(), {
        let mut s = HashSet::new();
        s.insert("helper2".to_string());
        s
    });

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert_eq!(reachable.len(), 4);
    assert!(reachable.contains("main"));
    assert!(reachable.contains("test_entry"));
    assert!(reachable.contains("helper1"));
    assert!(reachable.contains("helper2"));
}

#[test]
fn test_compute_reachability_diamond_pattern() {
    let mut entry_points = HashSet::new();
    entry_points.insert("main".to_string());

    let mut function_calls = HashMap::new();
    function_calls.insert("main".to_string(), {
        let mut s = HashSet::new();
        s.insert("left".to_string());
        s.insert("right".to_string());
        s
    });
    function_calls.insert("left".to_string(), {
        let mut s = HashSet::new();
        s.insert("bottom".to_string());
        s
    });
    function_calls.insert("right".to_string(), {
        let mut s = HashSet::new();
        s.insert("bottom".to_string());
        s
    });

    let reachable = compute_reachability(&entry_points, &function_calls);
    assert_eq!(reachable.len(), 4);
    assert!(reachable.contains("main"));
    assert!(reachable.contains("left"));
    assert!(reachable.contains("right"));
    assert!(reachable.contains("bottom"));
}

// ============================================================================
// detect_function_calls_in_lines tests
// ============================================================================

#[test]
fn test_detect_calls_empty_lines() {
    let lines: Vec<&str> = vec![];
    let all_functions = HashMap::new();

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);
    assert!(calls.is_empty());
}

#[test]
fn test_detect_calls_no_functions() {
    let lines = vec!["let x = 5;", "println!(\"hello\");"];
    let all_functions = HashMap::new();

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);
    assert!(calls.is_empty());
}

#[test]
fn test_detect_calls_simple_call() {
    let lines = vec![
        "fn main() {",
        "    helper();",
        "}",
        "fn helper() {",
        "    println!(\"hi\");",
        "}",
    ];

    let mut all_functions = HashMap::new();
    all_functions.insert("test.rs::main".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert("test.rs::helper".to_string(), ("test.rs".to_string(), 4));

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);

    // main calls helper
    assert!(calls.contains_key("test.rs::main"));
    let main_calls = calls.get("test.rs::main").unwrap();
    assert!(main_calls.contains("test.rs::helper"));
}

#[test]
fn test_detect_calls_ignores_function_definitions() {
    let lines = vec!["fn main() {", "    fn helper() {}", // nested definition, not a call
                     "}"];

    let mut all_functions = HashMap::new();
    all_functions.insert("test.rs::main".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert("test.rs::helper".to_string(), ("test.rs".to_string(), 2));

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);

    // Should NOT detect "fn helper" as a call
    if let Some(main_calls) = calls.get("test.rs::main") {
        assert!(
            !main_calls.contains("test.rs::helper"),
            "Should not count function definitions as calls"
        );
    }
}

#[test]
fn test_detect_calls_multiple_calls_same_line() {
    let lines = vec!["fn main() {", "    foo(); bar();", "}"];

    let mut all_functions = HashMap::new();
    all_functions.insert("test.rs::main".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert("test.rs::foo".to_string(), ("test.rs".to_string(), 10));
    all_functions.insert("test.rs::bar".to_string(), ("test.rs".to_string(), 20));

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);

    let main_calls = calls.get("test.rs::main").unwrap();
    assert!(main_calls.contains("test.rs::foo"));
    assert!(main_calls.contains("test.rs::bar"));
}

#[test]
fn test_detect_calls_ignores_self_calls() {
    // The function should not count self-recursion as a separate callee
    let lines = vec!["fn recursive() {", "    recursive();", "}"];

    let mut all_functions = HashMap::new();
    all_functions.insert(
        "test.rs::recursive".to_string(),
        ("test.rs".to_string(), 1),
    );

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);

    // Self-call should be ignored
    if let Some(rec_calls) = calls.get("test.rs::recursive") {
        assert!(
            !rec_calls.contains("test.rs::recursive"),
            "Self-calls should be ignored"
        );
    }
}

#[test]
fn test_detect_calls_cross_file() {
    let lines = vec!["fn main() {", "    other_file_fn();", "}"];

    let mut all_functions = HashMap::new();
    all_functions.insert("test.rs::main".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert(
        "other.rs::other_file_fn".to_string(),
        ("other.rs".to_string(), 1),
    );

    let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);

    let main_calls = calls.get("test.rs::main").unwrap();
    assert!(main_calls.contains("other.rs::other_file_fn"));
}

// ============================================================================
// classify_dead_functions_pure tests
// ============================================================================

#[test]
fn test_classify_dead_empty() {
    let all_functions = HashMap::new();
    let reachable = HashSet::new();

    let dead = classify_dead_functions_pure(&all_functions, &reachable);
    assert!(dead.is_empty());
}

#[test]
fn test_classify_dead_all_reachable() {
    let mut all_functions = HashMap::new();
    all_functions.insert("main".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert("helper".to_string(), ("test.rs".to_string(), 10));

    let mut reachable = HashSet::new();
    reachable.insert("main".to_string());
    reachable.insert("helper".to_string());

    let dead = classify_dead_functions_pure(&all_functions, &reachable);
    assert!(dead.is_empty());
}

#[test]
fn test_classify_dead_some_unreachable() {
    let mut all_functions = HashMap::new();
    all_functions.insert("main".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert("used".to_string(), ("test.rs".to_string(), 10));
    all_functions.insert("unused".to_string(), ("test.rs".to_string(), 20));

    let mut reachable = HashSet::new();
    reachable.insert("main".to_string());
    reachable.insert("used".to_string());

    let dead = classify_dead_functions_pure(&all_functions, &reachable);
    assert_eq!(dead.len(), 1);

    let (name, file, line) = &dead[0];
    assert_eq!(name, "unused");
    assert_eq!(file, "test.rs");
    assert_eq!(*line, 20);
}

#[test]
fn test_classify_dead_all_dead() {
    let mut all_functions = HashMap::new();
    all_functions.insert("orphan1".to_string(), ("test.rs".to_string(), 1));
    all_functions.insert("orphan2".to_string(), ("test.rs".to_string(), 10));

    let reachable = HashSet::new();

    let dead = classify_dead_functions_pure(&all_functions, &reachable);
    assert_eq!(dead.len(), 2);
}

#[test]
fn test_classify_dead_extracts_function_name_from_qualified() {
    let mut all_functions = HashMap::new();
    all_functions.insert(
        "src/lib.rs::module::submod::my_function".to_string(),
        ("src/lib.rs".to_string(), 42),
    );

    let reachable = HashSet::new();

    let dead = classify_dead_functions_pure(&all_functions, &reachable);
    assert_eq!(dead.len(), 1);

    let (name, _, _) = &dead[0];
    assert_eq!(name, "my_function");
}

// ============================================================================
// collect_functions_from_context tests
// ============================================================================

#[test]
fn test_collect_functions_empty_context() {
    let files: Vec<crate::services::context::FileContext> = vec![];

    let (all_functions, entry_points) = collect_functions_from_context(&files);
    assert!(all_functions.is_empty());
    assert!(entry_points.is_empty());
}

#[test]
fn test_collect_functions_with_main() {
    use crate::services::context::{AstItem, FileContext};

    let files = vec![FileContext {
        path: "main.rs".to_string(),
        language: "rust".to_string(),
        items: vec![AstItem::Function {
            name: "main".to_string(),
            visibility: "private".to_string(),
            is_async: false,
            line: 1,
        }],
        complexity_metrics: None,
    }];

    let (all_functions, entry_points) = collect_functions_from_context(&files);

    assert_eq!(all_functions.len(), 1);
    assert!(all_functions.contains_key("main.rs::main"));

    assert_eq!(entry_points.len(), 1);
    assert!(entry_points.contains("main.rs::main"));
}

#[test]
fn test_collect_functions_pub_is_entry_point() {
    use crate::services::context::{AstItem, FileContext};

    let files = vec![FileContext {
        path: "lib.rs".to_string(),
        language: "rust".to_string(),
        items: vec![
            AstItem::Function {
                name: "pub api".to_string(),
                visibility: "public".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Function {
                name: "private_helper".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 10,
            },
        ],
        complexity_metrics: None,
    }];

    let (all_functions, entry_points) = collect_functions_from_context(&files);

    assert_eq!(all_functions.len(), 2);

    // "pub api" starts with "pub " so it should be an entry point
    assert!(entry_points.contains("lib.rs::pub api"));
    assert!(!entry_points.contains("lib.rs::private_helper"));
}

#[test]
fn test_collect_functions_multiple_files() {
    use crate::services::context::{AstItem, FileContext};

    let files = vec![
        FileContext {
            path: "main.rs".to_string(),
            language: "rust".to_string(),
            items: vec![AstItem::Function {
                name: "main".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 1,
            }],
            complexity_metrics: None,
        },
        FileContext {
            path: "lib.rs".to_string(),
            language: "rust".to_string(),
            items: vec![
                AstItem::Function {
                    name: "helper".to_string(),
                    visibility: "private".to_string(),
                    is_async: false,
                    line: 5,
                },
                AstItem::Function {
                    name: "another".to_string(),
                    visibility: "private".to_string(),
                    is_async: false,
                    line: 15,
                },
            ],
            complexity_metrics: None,
        },
    ];

    let (all_functions, entry_points) = collect_functions_from_context(&files);

    assert_eq!(all_functions.len(), 3);
    assert!(all_functions.contains_key("main.rs::main"));
    assert!(all_functions.contains_key("lib.rs::helper"));
    assert!(all_functions.contains_key("lib.rs::another"));

    // Only main is entry point
    assert_eq!(entry_points.len(), 1);
    assert!(entry_points.contains("main.rs::main"));
}

#[test]
fn test_collect_functions_ignores_non_functions() {
    use crate::services::context::{AstItem, FileContext};

    let files = vec![FileContext {
        path: "lib.rs".to_string(),
        language: "rust".to_string(),
        items: vec![
            AstItem::Function {
                name: "my_func".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Struct {
                name: "MyStruct".to_string(),
                visibility: "public".to_string(),
                fields_count: 2,
                derives: vec![],
                line: 10,
            },
            AstItem::Enum {
                name: "MyEnum".to_string(),
                visibility: "public".to_string(),
                variants_count: 3,
                line: 20,
            },
        ],
        complexity_metrics: None,
    }];

    let (all_functions, _) = collect_functions_from_context(&files);

    assert_eq!(all_functions.len(), 1);
    assert!(all_functions.contains_key("lib.rs::my_func"));
}

// ============================================================================
// calculate_dead_percentage tests
// ============================================================================

#[test]
fn test_calculate_dead_percentage_zero_total() {
    let result = calculate_dead_percentage(0, 0);
    assert_eq!(result, 0.0);
}

#[test]
fn test_calculate_dead_percentage_zero_dead() {
    let result = calculate_dead_percentage(100, 0);
    assert_eq!(result, 0.0);
}

#[test]
fn test_calculate_dead_percentage_all_dead() {
    let result = calculate_dead_percentage(100, 100);
    assert!((result - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_calculate_dead_percentage_half_dead() {
    let result = calculate_dead_percentage(100, 50);
    assert!((result - 50.0).abs() < f32::EPSILON);
}

#[test]
fn test_calculate_dead_percentage_quarter_dead() {
    let result = calculate_dead_percentage(100, 25);
    assert!((result - 25.0).abs() < f32::EPSILON);
}

#[test]
fn test_calculate_dead_percentage_fractional() {
    let result = calculate_dead_percentage(3, 1);
    assert!((result - 33.333336).abs() < 0.001);
}

// ============================================================================
// Integration tests: combine pure functions
// ============================================================================

#[test]
fn test_integration_full_analysis_flow() {
    use crate::services::context::{AstItem, FileContext};

    // Setup: file with main calling helper, and unused function
    let files = vec![FileContext {
        path: "test.rs".to_string(),
        language: "rust".to_string(),
        items: vec![
            AstItem::Function {
                name: "main".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Function {
                name: "helper".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 5,
            },
            AstItem::Function {
                name: "unused".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 10,
            },
        ],
        complexity_metrics: None,
    }];

    // Step 1: Collect functions
    let (all_functions, entry_points) = collect_functions_from_context(&files);
    assert_eq!(all_functions.len(), 3);
    assert!(entry_points.contains("test.rs::main"));

    // Step 2: Detect calls (simulated source)
    let source_lines = vec![
        "fn main() {",
        "    helper();",
        "}",
        "",
        "fn helper() {",
        "    println!(\"hi\");",
        "}",
        "",
        "",
        "fn unused() {}",
    ];
    let function_calls = detect_function_calls_in_lines("test.rs", &source_lines, &all_functions);

    // Step 3: Compute reachability
    let reachable = compute_reachability(&entry_points, &function_calls);
    assert!(reachable.contains("test.rs::main"));
    assert!(reachable.contains("test.rs::helper"));
    assert!(!reachable.contains("test.rs::unused"));

    // Step 4: Classify dead functions
    let dead = classify_dead_functions_pure(&all_functions, &reachable);
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].0, "unused");

    // Step 5: Calculate percentage
    let percentage = calculate_dead_percentage(all_functions.len(), dead.len());
    assert!((percentage - 33.333336).abs() < 0.001);
}

#[test]
fn test_integration_no_dead_code() {
    use crate::services::context::{AstItem, FileContext};

    let files = vec![FileContext {
        path: "test.rs".to_string(),
        language: "rust".to_string(),
        items: vec![
            AstItem::Function {
                name: "main".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Function {
                name: "helper".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 5,
            },
        ],
        complexity_metrics: None,
    }];

    let (all_functions, entry_points) = collect_functions_from_context(&files);

    // main calls helper
    let source_lines = vec!["fn main() {", "    helper();", "}", "fn helper() {}"];
    let function_calls = detect_function_calls_in_lines("test.rs", &source_lines, &all_functions);
    let reachable = compute_reachability(&entry_points, &function_calls);
    let dead = classify_dead_functions_pure(&all_functions, &reachable);

    assert!(dead.is_empty());
    assert_eq!(calculate_dead_percentage(all_functions.len(), dead.len()), 0.0);
}

#[test]
fn test_integration_all_dead_no_entry_points() {
    use crate::services::context::{AstItem, FileContext};

    let files = vec![FileContext {
        path: "lib.rs".to_string(),
        language: "rust".to_string(),
        items: vec![
            AstItem::Function {
                name: "orphan1".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Function {
                name: "orphan2".to_string(),
                visibility: "private".to_string(),
                is_async: false,
                line: 5,
            },
        ],
        complexity_metrics: None,
    }];

    let (all_functions, entry_points) = collect_functions_from_context(&files);
    assert!(entry_points.is_empty()); // No main, no pub

    let function_calls = HashMap::new();
    let reachable = compute_reachability(&entry_points, &function_calls);
    let dead = classify_dead_functions_pure(&all_functions, &reachable);

    assert_eq!(dead.len(), 2);
    assert_eq!(calculate_dead_percentage(2, 2), 100.0);
}
