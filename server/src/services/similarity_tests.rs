//! Comprehensive TDD tests for code similarity detection
//!
//! These tests are written BEFORE implementation following TDD principles.
//! They define the expected behavior of our similarity detection system.

#[cfg(test)]
mod exact_duplicate_tests {
    use super::super::similarity::*;

    #[test]
    fn test_detect_exact_duplicate_functions() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = r#"
            fn calculate_sum(a: i32, b: i32) -> i32 {
                let result = a + b;
                println!("Sum: {}", result);
                result
            }
        "#;
        
        let code2 = r#"
            fn calculate_sum(a: i32, b: i32) -> i32 {
                let result = a + b;
                println!("Sum: {}", result);
                result
            }
        "#;
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let duplicates = detector.detect_exact_duplicates(&files);
        
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].locations.len(), 2);
        assert_eq!(duplicates[0].similarity, 1.0);
    }

    #[test]
    fn test_ignore_whitespace_differences() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = "fn add(a:i32,b:i32)->i32{a+b}";
        let code2 = "fn add(a: i32, b: i32) -> i32 { a + b }";
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let duplicates = detector.detect_exact_duplicates(&files);
        
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].similarity, 1.0);
    }

    #[test]
    fn test_detect_multiple_duplicates() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code_a = "fn process() { validate(); transform(); save(); }";
        let code_b = "fn handle() { check(); convert(); store(); }";
        
        let files = vec![
            ("file1.rs".into(), format!("{}\n{}", code_a, code_b)),
            ("file2.rs".into(), code_a.to_string()),
            ("file3.rs".into(), code_b.to_string()),
            ("file4.rs".into(), code_a.to_string()),
        ];
        
        let duplicates = detector.detect_exact_duplicates(&files);
        
        // Should find code_a in 3 places and code_b in 2 places
        assert!(duplicates.len() >= 2);
    }

    #[test]
    fn test_minimum_size_threshold() {
        let mut config = SimilarityConfig::default();
        config.min_lines = 5;
        let detector = SimilarityDetector::new(config);
        
        let small_dup = "fn tiny() { 42 }";
        let large_dup = r#"
            fn process_data(input: &str) -> Result<String> {
                let parsed = parse(input)?;
                let validated = validate(parsed)?;
                let transformed = transform(validated);
                Ok(serialize(transformed))
            }
        "#;
        
        let files = vec![
            ("file1.rs".into(), format!("{}\n{}", small_dup, large_dup)),
            ("file2.rs".into(), format!("{}\n{}", small_dup, large_dup)),
        ];
        
        let duplicates = detector.detect_exact_duplicates(&files);
        
        // Should only detect the large duplicate, not the small one
        assert_eq!(duplicates.len(), 1);
        assert!(duplicates[0].lines >= 5);
    }
}

#[cfg(test)]
mod structural_similarity_tests {
    use super::super::similarity::*;

    #[test]
    fn test_detect_renamed_variables() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = r#"
            fn calculate(x: i32, y: i32) -> i32 {
                let temp = x * 2;
                temp + y
            }
        "#;
        
        let code2 = r#"
            fn calculate(a: i32, b: i32) -> i32 {
                let result = a * 2;
                result + b
            }
        "#;
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let similarities = detector.detect_structural_similarity(&files, 0.9);
        
        assert_eq!(similarities.len(), 1);
        assert!(similarities[0].similarity > 0.9);
        assert_eq!(similarities[0].clone_type, CloneType::Type2);
    }

    #[test]
    fn test_detect_renamed_functions() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = "fn process_user(id: u64) { fetch(id); update(id); notify(id); }";
        let code2 = "fn handle_customer(key: u64) { fetch(key); update(key); notify(key); }";
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let similarities = detector.detect_structural_similarity(&files, 0.85);
        
        assert_eq!(similarities.len(), 1);
        assert!(similarities[0].similarity > 0.85);
    }

    #[test]
    fn test_detect_reordered_statements() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = r#"
            fn init() {
                setup_logger();
                load_config();
                connect_db();
            }
        "#;
        
        let code2 = r#"
            fn init() {
                load_config();
                setup_logger();
                connect_db();
            }
        "#;
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let similarities = detector.detect_structural_similarity(&files, 0.8);
        
        assert_eq!(similarities.len(), 1);
        assert!(similarities[0].similarity > 0.8);
        assert_eq!(similarities[0].clone_type, CloneType::Type3);
    }
}

#[cfg(test)]
mod semantic_similarity_tests {
    use super::super::similarity::*;

    #[test]
    fn test_detect_semantic_equivalence() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = r#"
            fn is_even(n: i32) -> bool {
                n % 2 == 0
            }
        "#;
        
        let code2 = r#"
            fn is_even(num: i32) -> bool {
                if num % 2 == 0 {
                    true
                } else {
                    false
                }
            }
        "#;
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let similarities = detector.detect_semantic_similarity(&files, 0.7);
        
        assert_eq!(similarities.len(), 1);
        assert!(similarities[0].similarity > 0.7);
        assert_eq!(similarities[0].clone_type, CloneType::Type4);
    }

    #[test]
    fn test_detect_loop_equivalence() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code1 = r#"
            fn sum_array(arr: &[i32]) -> i32 {
                let mut sum = 0;
                for val in arr {
                    sum += val;
                }
                sum
            }
        "#;
        
        let code2 = r#"
            fn sum_array(arr: &[i32]) -> i32 {
                arr.iter().sum()
            }
        "#;
        
        let files = vec![
            ("file1.rs".into(), code1.to_string()),
            ("file2.rs".into(), code2.to_string()),
        ];
        
        let similarities = detector.detect_semantic_similarity(&files, 0.6);
        
        assert_eq!(similarities.len(), 1);
        assert!(similarities[0].similarity > 0.6);
    }
}

#[cfg(test)]
mod entropy_analysis_tests {
    use super::super::similarity::*;

    #[test]
    fn test_calculate_shannon_entropy() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        // Repetitive code should have low entropy
        let repetitive = r#"
            if x == 1 { return 1; }
            if x == 2 { return 2; }
            if x == 3 { return 3; }
            if x == 4 { return 4; }
        "#;
        
        // Complex code should have high entropy
        let complex = r#"
            match x {
                Pattern::A(a) => process_a(a)?,
                Pattern::B { field1, field2 } => handle_b(field1, field2),
                Pattern::C(Some(val)) if val > 0 => transform(val),
                _ => default_handler(),
            }
        "#;
        
        let entropy_low = detector.calculate_entropy(repetitive);
        let entropy_high = detector.calculate_entropy(complex);
        
        assert!(entropy_low < entropy_high);
        assert!(entropy_low < 2.0);  // Low entropy threshold
        assert!(entropy_high > 3.0); // High entropy threshold
    }

    #[test]
    fn test_identify_refactoring_opportunities() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code = r#"
            fn process_order(order: Order) -> Result<()> {
                if order.status == "pending" {
                    validate_order(&order)?;
                    calculate_total(&order);
                    update_inventory(&order)?;
                    send_notification(&order);
                }
                Ok(())
            }
            
            fn process_return(return_req: Return) -> Result<()> {
                if return_req.status == "pending" {
                    validate_return(&return_req)?;
                    calculate_refund(&return_req);
                    update_inventory(&return_req)?;
                    send_notification(&return_req);
                }
                Ok(())
            }
        "#;
        
        let files = vec![("file.rs".into(), code.to_string())];
        
        let report = detector.analyze_entropy(&files);
        let opportunities = detector.find_refactoring_opportunities(&files);
        
        assert!(!opportunities.is_empty());
        assert!(opportunities[0].suggestion.contains("Extract common pattern"));
    }

    #[test]
    fn test_detect_copy_paste_patterns() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        let code = r#"
            fn validate_email(email: &str) -> bool {
                if email.is_empty() {
                    log::error("Email is empty");
                    return false;
                }
                if !email.contains('@') {
                    log::error("Email missing @");
                    return false;
                }
                true
            }
            
            fn validate_phone(phone: &str) -> bool {
                if phone.is_empty() {
                    log::error("Phone is empty");
                    return false;
                }
                if phone.len() < 10 {
                    log::error("Phone too short");
                    return false;
                }
                true
            }
        "#;
        
        let files = vec![("validators.rs".into(), code.to_string())];
        
        let report = detector.analyze_entropy(&files);
        
        assert!(!report.low_entropy_patterns.is_empty());
        assert!(report.recommendations.iter()
            .any(|r| r.contains("validation pattern")));
    }
}

#[cfg(test)]
mod winnowing_algorithm_tests {
    use super::super::similarity::*;

    #[test]
    fn test_winnowing_fingerprinting() {
        let winnower = Winnowing::new(40, 15); // window=40, k=15
        
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "The quick brown fox leaps over the lazy dog"; // "jumps" -> "leaps"
        
        let fingerprint1 = winnower.fingerprint(text1);
        let fingerprint2 = winnower.fingerprint(text2);
        
        let similarity = winnower.similarity(&fingerprint1, &fingerprint2);
        
        // Should be highly similar but not identical
        assert!(similarity > 0.8);
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_winnowing_substring_guarantee() {
        let winnower = Winnowing::new(40, 15);
        
        let text = "This is a long text with a guaranteed substring match in the middle of it";
        let substring = "guaranteed substring match";
        
        let text_fingerprint = winnower.fingerprint(text);
        let sub_fingerprint = winnower.fingerprint(substring);
        
        // Winnowing guarantees finding matching substrings
        let matches = winnower.find_matches(&text_fingerprint, &sub_fingerprint);
        
        assert!(!matches.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::super::similarity::*;

    #[test]
    fn test_comprehensive_analysis() {
        let config = SimilarityConfig {
            min_lines: 3,
            min_tokens: 20,
            similarity_threshold: 0.7,
            enable_entropy: true,
            enable_ast: true,
            enable_semantic: true,
            window_size: 40,
            k_gram_size: 15,
        };
        
        let detector = SimilarityDetector::new(config);
        
        let files = vec![
            ("auth.rs".into(), include_str!("../../tests/fixtures/auth_duplicate.rs").to_string()),
            ("user.rs".into(), include_str!("../../tests/fixtures/user_similar.rs").to_string()),
            ("admin.rs".into(), include_str!("../../tests/fixtures/admin_semantic.rs").to_string()),
        ];
        
        let report = detector.comprehensive_analysis(&files);
        
        assert!(report.exact_duplicates.len() > 0);
        assert!(report.structural_similarities.len() > 0);
        assert!(report.semantic_similarities.len() > 0);
        assert!(report.entropy_analysis.is_some());
        assert!(report.refactoring_opportunities.len() > 0);
        
        // Verify metrics
        assert!(report.metrics.duplication_percentage > 0.0);
        assert!(report.metrics.average_entropy > 0.0);
        assert_eq!(report.metrics.total_clones,
            report.exact_duplicates.len() +
            report.structural_similarities.len() +
            report.semantic_similarities.len());
    }

    #[test]
    fn test_performance_100k_loc() {
        use std::time::Instant;
        
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        
        // Generate 100K lines of code
        let mut large_file = String::new();
        for i in 0..10000 {
            large_file.push_str(&format!(
                "fn function_{}(x: i32) -> i32 {{ x * {} + {} }}\n",
                i, i % 100, i % 50
            ));
        }
        
        let files = vec![("large.rs".into(), large_file)];
        
        let start = Instant::now();
        let _report = detector.comprehensive_analysis(&files);
        let elapsed = start.elapsed();
        
        // Should process 100K LOC in less than 5 seconds
        assert!(elapsed.as_secs() < 5);
    }
}