use super::*;
use tempfile::TempDir;

#[test]
fn test_big_o_analyzer_creation() {
    let analyzer = BigOAnalyzer::new();
    // Ensure it creates successfully
    let _ = analyzer;
}

#[test]
fn test_get_loop_keywords() {
    assert_eq!(BigOAnalyzer::get_loop_keywords("rust"), vec!["for", "while", "loop"]);
    assert_eq!(BigOAnalyzer::get_loop_keywords("javascript"), vec!["for", "while", "do"]);
    assert_eq!(BigOAnalyzer::get_loop_keywords("typescript"), vec!["for", "while", "do"]);
    assert_eq!(BigOAnalyzer::get_loop_keywords("python"), vec!["for", "while"]);
    assert_eq!(BigOAnalyzer::get_loop_keywords("unknown"), vec!["for", "while"]);
}

#[test]
fn test_detect_recursive_call() {
    assert!(BigOAnalyzer::detect_recursive_call("    fibonacci(n-1) + fibonacci(n-2)", "fibonacci"));
    assert!(BigOAnalyzer::detect_recursive_call("return factorial(n-1) * n", "factorial"));
    assert!(!BigOAnalyzer::detect_recursive_call("fn fibonacci() {", "fibonacci"));
    assert!(!BigOAnalyzer::detect_recursive_call("function fibonacci() {", "fibonacci"));
    assert!(!BigOAnalyzer::detect_recursive_call("let result = other_function()", "fibonacci"));
}

#[test]
fn test_detect_sorting_operation() {
    assert!(BigOAnalyzer::detect_sorting_operation("    vec.sort()"));
    assert!(BigOAnalyzer::detect_sorting_operation("array.sort((a, b) => a - b)"));
    assert!(BigOAnalyzer::detect_sorting_operation("    sort(data)"));
    assert!(!BigOAnalyzer::detect_sorting_operation("    // sort() is called here"));
    assert!(!BigOAnalyzer::detect_sorting_operation("    resort = true"));
}

#[test]
fn test_detect_binary_search() {
    assert!(BigOAnalyzer::detect_binary_search("    vec.binary_search(&target)"));
    assert!(BigOAnalyzer::detect_binary_search("    binarySearch(array, target)"));
    assert!(!BigOAnalyzer::detect_binary_search("    // binary_search is efficient"));
    assert!(!BigOAnalyzer::detect_binary_search("    my_binary_search_impl()"));
}

#[test]
fn test_calculate_loop_depth() {
    let code_simple = vec![
        "for i in 0..n {",
        "    println!(i);",
        "}",
    ];
    assert_eq!(BigOAnalyzer::calculate_loop_depth(&code_simple, &["for", "while"]), 1);
    
    let code_nested = vec![
        "for i in 0..n {",
        "    for j in 0..m {",
        "        println!(i, j);",
        "    }",
        "}",
    ];
    assert_eq!(BigOAnalyzer::calculate_loop_depth(&code_nested, &["for", "while"]), 2);
    
    let code_triple = vec![
        "for i in 0..n {",
        "    for j in 0..m {",
        "        for k in 0..p {",
        "            process(i, j, k);",
        "        }",
        "    }",
        "}",
    ];
    assert_eq!(BigOAnalyzer::calculate_loop_depth(&code_triple, &["for", "while"]), 3);
    
    let code_no_loops = vec![
        "fn constant_time() {",
        "    let x = 5;",
        "    return x + 10;",
        "}",
    ];
    assert_eq!(BigOAnalyzer::calculate_loop_depth(&code_no_loops, &["for", "while"]), 0);
}

#[test]
fn test_determine_time_complexity() {
    // Test constant time
    let constant = BigOAnalyzer::determine_time_complexity(0, false);
    assert_eq!(constant.class, BigOClass::Constant);
    assert_eq!(constant.confidence, 90);
    
    // Test linear time
    let linear = BigOAnalyzer::determine_time_complexity(1, false);
    assert_eq!(linear.class, BigOClass::Linear);
    assert_eq!(linear.confidence, 80);
    
    // Test quadratic time
    let quadratic = BigOAnalyzer::determine_time_complexity(2, false);
    assert_eq!(quadratic.class, BigOClass::Quadratic);
    assert_eq!(quadratic.confidence, 75);
    
    // Test cubic time
    let cubic = BigOAnalyzer::determine_time_complexity(3, false);
    assert_eq!(cubic.class, BigOClass::Polynomial { degree: 3, coefficient: 1 });
    assert_eq!(cubic.confidence, 70);
    
    // Test recursion without loops
    let recursive = BigOAnalyzer::determine_time_complexity(0, true);
    assert_eq!(recursive.class, BigOClass::Unknown);
}

#[test]
fn test_detect_space_complexity() {
    let (space, has_alloc) = BigOAnalyzer::detect_space_complexity("let v = Vec::new();");
    assert_eq!(space.class, BigOClass::Linear);
    assert!(has_alloc);
    
    let (space, has_alloc) = BigOAnalyzer::detect_space_complexity("let v = vec![1, 2, 3];");
    assert_eq!(space.class, BigOClass::Linear);
    assert!(has_alloc);
    
    let (space, has_alloc) = BigOAnalyzer::detect_space_complexity("let map = HashMap::new();");
    assert_eq!(space.class, BigOClass::Linear);
    assert!(has_alloc);
    
    let (space, has_alloc) = BigOAnalyzer::detect_space_complexity("let x = 5; let y = x + 10;");
    assert_eq!(space.class, BigOClass::Constant);
    assert!(!has_alloc);
}

#[test]
fn test_analysis_config_creation() {
    let config = BigOAnalysisConfig {
        project_path: PathBuf::from("/test/path"),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["test_*.rs".to_string()],
        confidence_threshold: 70,
        analyze_space_complexity: true,
    };
    
    assert_eq!(config.project_path, PathBuf::from("/test/path"));
    assert_eq!(config.include_patterns.len(), 1);
    assert_eq!(config.exclude_patterns.len(), 1);
    assert_eq!(config.confidence_threshold, 70);
    assert!(config.analyze_space_complexity);
}

#[test]
fn test_complexity_distribution_default() {
    let dist = ComplexityDistribution {
        constant: 0,
        logarithmic: 0,
        linear: 0,
        linearithmic: 0,
        quadratic: 0,
        cubic: 0,
        exponential: 0,
        unknown: 0,
    };
    
    assert_eq!(dist.constant, 0);
    assert_eq!(dist.linear, 0);
    assert_eq!(dist.quadratic, 0);
}

#[test]
fn test_function_complexity_creation() {
    let func = FunctionComplexity {
        file_path: PathBuf::from("src/main.rs"),
        function_name: "process_data".to_string(),
        line_number: 42,
        time_complexity: ComplexityBound::linear(),
        space_complexity: ComplexityBound::constant(),
        confidence: 85,
        notes: vec!["Pattern: nested loops".to_string()],
    };
    
    assert_eq!(func.function_name, "process_data");
    assert_eq!(func.line_number, 42);
    assert_eq!(func.confidence, 85);
    assert_eq!(func.notes.len(), 1);
}

#[test]
fn test_pattern_match_creation() {
    let pattern = PatternMatch {
        pattern_name: "nested_loops".to_string(),
        occurrences: 5,
        typical_complexity: BigOClass::Quadratic,
    };
    
    assert_eq!(pattern.pattern_name, "nested_loops");
    assert_eq!(pattern.occurrences, 5);
    assert_eq!(pattern.typical_complexity, BigOClass::Quadratic);
}

#[tokio::test]
async fn test_analyze_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let analyzer = BigOAnalyzer::new();
    
    let config = BigOAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        confidence_threshold: 70,
        analyze_space_complexity: true,
    };
    
    let result = analyzer.analyze(config).await.unwrap();
    
    assert_eq!(result.analyzed_functions, 0);
    assert!(result.high_complexity_functions.is_empty());
    assert!(result.pattern_matches.is_empty());
}

#[test]
fn test_serialization() {
    let report = BigOAnalysisReport {
        analyzed_functions: 10,
        complexity_distribution: ComplexityDistribution {
            constant: 5,
            logarithmic: 0,
            linear: 3,
            linearithmic: 0,
            quadratic: 2,
            cubic: 0,
            exponential: 0,
            unknown: 0,
        },
        high_complexity_functions: vec![],
        pattern_matches: vec![],
        recommendations: vec!["Consider optimizing quadratic algorithms".to_string()],
    };
    
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: BigOAnalysisReport = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.analyzed_functions, 10);
    assert_eq!(deserialized.complexity_distribution.constant, 5);
    assert_eq!(deserialized.recommendations.len(), 1);
}