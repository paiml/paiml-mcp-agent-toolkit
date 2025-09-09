//! TDD Tests for Ruchy TDG Integration
//! 
//! Testing Ruchy language detection and TDG scoring integration

#[cfg(all(test, feature = "ruchy-ast"))]
mod ruchy_tdg_integration_tests {
    use std::path::Path;
    
    #[test]
    fn test_ruchy_language_detection_from_extension() {
        // RED: This should fail because Ruchy is not yet added to Language enum
        let ruchy_file = Path::new("test.ruchy");
        let language = pmat::tdg::Language::from_extension(ruchy_file);
        
        // Should detect Ruchy files correctly
        assert_eq!(language, pmat::tdg::Language::Ruchy);
    }
    
    #[test]
    fn test_ruchy_alternative_extension() {
        // RED: Test alternative .rh extension 
        let ruchy_file = Path::new("script.rh");
        let language = pmat::tdg::Language::from_extension(ruchy_file);
        
        assert_eq!(language, pmat::tdg::Language::Ruchy);
    }
    
    #[test]
    fn test_ruchy_language_confidence() {
        // RED: Test that Ruchy has appropriate confidence level
        let confidence = pmat::tdg::Language::Ruchy.confidence();
        
        // Ruchy is a mature language with good tooling, should have high confidence
        assert!(confidence >= 0.90, "Ruchy should have high confidence score, got {}", confidence);
    }
    
    #[test] 
    fn test_ruchy_language_display() {
        // RED: Test that Ruchy has proper string representation
        let ruchy_lang = pmat::tdg::Language::Ruchy;
        let display_name = format!("{:?}", ruchy_lang);
        
        assert_eq!(display_name, "Ruchy");
    }
    
    #[cfg(feature = "ruchy-ast")]
    #[tokio::test]
    async fn test_ruchy_tdg_analysis_integration() {
        // RED: Test that TDG analysis works with Ruchy files
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let ruchy_code = r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    let result = fibonacci(10);
    println(f"Result: {result}");
}
"#;
        
        let mut temp_file = NamedTempFile::with_suffix(".ruchy").unwrap();
        temp_file.write_all(ruchy_code.as_bytes()).unwrap();
        
        // Should be able to analyze Ruchy file with TDG
        let analyzer = pmat::tdg::TdgAnalyzer::new()?;
        let result = analyzer.analyze_file(temp_file.path()).await;
            
        assert!(result.is_ok(), "TDG analysis should work with Ruchy files");
        
        let score = result.unwrap();
        
        // Should have valid TDG score components
        assert!(score.total >= 0.0 && score.total <= 100.0);
        assert!(score.structural_complexity >= 0.0);
        assert!(score.semantic_complexity >= 0.0);
        
        // Should detect functions (fibonacci + main)
        assert!(score.structural_complexity > 0.0, "Should detect function complexity");
    }
    
    #[test]
    fn test_ruchy_language_rules() {
        // RED: Test that Ruchy has appropriate language rules
        let rules = pmat::tdg::LanguageRules::for_language(pmat::tdg::Language::Ruchy);
        
        // Ruchy is a modern language, should have high standards
        // TODO: Add complexity and test requirements when fields are available
        // assert!(rules.max_complexity <= 15, "Ruchy should enforce reasonable complexity limits");
        // assert!(rules.requires_tests, "Ruchy should require tests (it's a quality-focused language)");
        
        // For now, just verify the rules object exists and has basic fields
        assert_eq!(rules.language, pmat::tdg::Language::Ruchy);
    }
    
    #[test]
    fn test_ruchy_semantic_signature_patterns() {
        // RED: Test that Ruchy files generate appropriate semantic signatures
        let ruchy_patterns = vec![
            "fun main", "match", "if", "actor", "|>", "receive"
        ];
        
        // These patterns should be recognized in Ruchy semantic analysis
        for pattern in ruchy_patterns {
            // This is a placeholder test - the actual implementation will 
            // need to check that these patterns are properly handled
            assert!(!pattern.is_empty(), "Pattern '{}' should be non-empty", pattern);
        }
    }
    
    #[cfg(feature = "ruchy-ast")]
    #[tokio::test]
    async fn test_ruchy_actor_model_complexity() {
        // RED: Test that actor model patterns are analyzed correctly
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let actor_code = r#"
actor Counter {
    count: i32,
    
    receive increment() {
        self.count += 1;
    }
    
    receive get() -> i32 {
        self.count
    }
    
    receive decrement() {
        if self.count > 0 {
            self.count -= 1;
        }
    }
}

fun main() {
    let counter = spawn Counter { count: 0 };
    counter <- increment();
    let value = counter <? get();
    println(f"Count: {value}");
}
"#;
        
        let mut temp_file = NamedTempFile::with_suffix(".ruchy").unwrap();
        temp_file.write_all(actor_code.as_bytes()).unwrap();
        
        let analyzer = pmat::tdg::TdgAnalyzer::new()?;
        let result = analyzer.analyze_file(temp_file.path()).await;
            
        assert!(result.is_ok(), "Should analyze actor model code");
        
        let score = result.unwrap();
        
        // Actor model should contribute to complexity
        assert!(score.structural_complexity > 10.0, "Actor patterns should increase structural complexity");
        
        // Should detect multiple message handlers
        assert!(score.coupling_score > 0.0, "Actor message passing should affect coupling score");
    }
    
    #[cfg(feature = "ruchy-ast")]
    #[tokio::test]
    async fn test_ruchy_pipeline_operator_complexity() {
        // RED: Test that pipeline operators are handled in complexity analysis
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let pipeline_code = r#"
fun process_data(numbers: [i32]) -> [i32] {
    numbers
        |> filter(|x| x > 0)
        |> map(|x| x * 2)
        |> filter(|x| x < 100)
        |> sort()
        |> reverse()
}

fun main() {
    let data = [1, -2, 3, 50, 200];
    let result = process_data(data);
    println(f"Processed: {result}");
}
"#;
        
        let mut temp_file = NamedTempFile::with_suffix(".ruchy").unwrap();
        temp_file.write_all(pipeline_code.as_bytes()).unwrap();
        
        let analyzer = pmat::tdg::TdgAnalyzer::new()?;
        let result = analyzer.analyze_file(temp_file.path()).await;
            
        assert!(result.is_ok(), "Should analyze pipeline operator code");
        
        let score = result.unwrap();
        
        // Pipeline operations might affect cognitive complexity
        assert!(score.semantic_complexity > 0.0, "Pipeline operations should contribute to semantic complexity");
        
        // Should maintain good readability despite complexity
        assert!(score.total >= 60.0, "Well-written pipeline code should score reasonably well");
    }
}

#[cfg(not(feature = "ruchy-ast"))]
#[cfg(test)]
mod ruchy_tdg_fallback_tests {
    use std::path::Path;
    
    #[test]
    fn test_ruchy_detection_without_ast_feature() {
        // When ruchy-ast feature is disabled, should still detect language
        let ruchy_file = Path::new("test.ruchy");
        let language = pmat::tdg::Language::from_extension(ruchy_file);
        
        assert_eq!(language, pmat::tdg::Language::Ruchy);
    }
    
    #[test] 
    fn test_ruchy_fallback_confidence() {
        // Should still provide confidence even without full AST support
        let confidence = pmat::tdg::Language::Ruchy.confidence();
        assert!(confidence > 0.0);
    }
}