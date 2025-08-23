use pmat::services::polyglot_analyzer::{PolyglotAnalyzer, LanguageInfo, PolyglotAnalysis};

#[tokio::test]
async fn test_polyglot_analyzer_initialization() {
    let analyzer = PolyglotAnalyzer::new();
    
    // Test that the analyzer initializes correctly by using public API
    let test_insights = analyzer.generate_polyglot_insights(&PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.5,
    });
    
    // Should generate some insights even for empty analysis
    assert!(!test_insights.is_empty(), "Should generate insights");
}

#[tokio::test]
async fn test_polyglot_analyzer_insights() {
    let analyzer = PolyglotAnalyzer::new();
    
    // Create a mock polyglot analysis
    let analysis = PolyglotAnalysis {
        languages: vec![
            pmat::services::polyglot_analyzer::LanguageStats {
                language: "rust".to_string(),
                file_count: 10,
                line_count: 1000,
                complexity_score: 5.5,
                test_coverage: 0.8,
                primary_frameworks: vec!["Tokio".to_string(), "Serde".to_string()],
            },
            pmat::services::polyglot_analyzer::LanguageStats {
                language: "python".to_string(),
                file_count: 5,
                line_count: 500,
                complexity_score: 3.2,
                test_coverage: 0.7,
                primary_frameworks: vec!["Django".to_string()],
            },
            pmat::services::polyglot_analyzer::LanguageStats {
                language: "javascript".to_string(),
                file_count: 8,
                line_count: 800,
                complexity_score: 4.1,
                test_coverage: 0.6,
                primary_frameworks: vec!["React".to_string()],
            },
        ],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(pmat::services::polyglot_analyzer::ArchitecturePattern::Mixed),
        integration_points: vec![],
        recommendation_score: 0.85,
    };
    
    // Generate insights
    let insights = analyzer.generate_polyglot_insights(&analysis);
    
    // Verify insights are generated
    assert!(!insights.is_empty());
    assert!(insights.iter().any(|i| i.contains("polyglot project")));
    assert!(insights.iter().any(|i| i.contains("Primary language: rust")));
    assert!(insights.iter().any(|i| i.contains("Architecture pattern: Mixed")));
    assert!(insights.iter().any(|i| i.contains("recommendation score: 0.85")));
}

#[test]
fn test_language_info_creation() {
    let lang_info = LanguageInfo {
        name: "rust".to_string(),
        file_count: 25,
        line_count: 2500,
        frameworks: vec!["Tokio".to_string(), "Serde".to_string()],
    };
    
    assert_eq!(lang_info.name, "rust");
    assert_eq!(lang_info.file_count, 25);
    assert_eq!(lang_info.line_count, 2500);
    assert_eq!(lang_info.frameworks.len(), 2);
}

#[test] 
fn test_polyglot_analysis_serialization() {
    use serde_json;
    
    let analysis = PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(pmat::services::polyglot_analyzer::ArchitecturePattern::Monolithic),
        integration_points: vec![],
        recommendation_score: 0.5,
    };
    
    // Test that the analysis can be serialized to JSON
    let json_result = serde_json::to_string(&analysis);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("Monolithic"));
    assert!(json_str.contains("0.5"));
}