#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_entropy_for_repetitive_code() {
        let calculator = EntropyCalculator::new();
        let repetitive = "aaaaaaaaaa";
        let entropy = calculator.calculate(repetitive);
        assert!(entropy < 1.0);
    }

    #[test]
    fn test_high_entropy_for_diverse_code() {
        let calculator = EntropyCalculator::new();
        let diverse = "fn calculate_prime(n: u64) -> bool { if n <= 1 { false } else { true } }";
        let entropy = calculator.calculate(diverse);
        assert!(entropy > 3.0);
    }

    #[test]
    fn test_token_entropy() {
        let calculator = EntropyCalculator::new();
        let code = "fn foo() { let x = 1; let y = 2; }";
        let token_entropy = calculator.calculate_token_entropy(code);
        assert!(token_entropy > 0.0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // EntropyCalculator tests
    #[test]
    fn test_entropy_calculator_default() {
        let calculator = EntropyCalculator::default();
        let _ = calculator;
    }

    #[test]
    fn test_entropy_calculator_new() {
        let calculator = EntropyCalculator::new();
        let _ = calculator;
    }

    #[test]
    fn test_calculate_empty_string() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("");
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_calculate_single_char() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("a");
        assert_eq!(entropy, 0.0); // Single char = 0 entropy
    }

    #[test]
    fn test_calculate_two_chars_equal() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("ab");
        // Two different characters with equal probability = 1 bit
        assert!((entropy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_two_chars_unequal() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("aab");
        // a appears twice, b once -> less than 1 bit
        assert!(entropy > 0.0 && entropy < 1.5);
    }

    #[test]
    fn test_calculate_uniform_distribution() {
        let calculator = EntropyCalculator::new();
        // All unique characters
        let entropy = calculator.calculate("abcdefgh");
        // 8 unique chars = 3 bits of entropy
        assert!((entropy - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_with_spaces() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("hello world");
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_calculate_with_newlines() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("line1\nline2\nline3");
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_calculate_with_unicode() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate("hello\u{4e16}\u{754c}");
        assert!(entropy > 0.0);
    }

    // Token entropy tests
    #[test]
    fn test_calculate_token_entropy_empty() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate_token_entropy("");
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_calculate_token_entropy_single_token() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate_token_entropy("hello");
        assert_eq!(entropy, 0.0); // Single token = 0 entropy
    }

    #[test]
    fn test_calculate_token_entropy_multiple_tokens() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate_token_entropy("fn foo bar baz");
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_calculate_token_entropy_repeated_tokens() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate_token_entropy("let let let let");
        assert_eq!(entropy, 0.0); // All same tokens
    }

    #[test]
    fn test_calculate_token_entropy_with_delimiters() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate_token_entropy("fn foo() { let x = 1; }");
        // Should split on delimiters and have positive entropy
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_calculate_token_entropy_with_punctuation() {
        let calculator = EntropyCalculator::new();
        let entropy = calculator.calculate_token_entropy("a.b.c[0].d");
        assert!(entropy > 0.0);
    }

    // Tokenize tests
    #[test]
    fn test_calculate_ast_diversity_empty_file() {
        let calculator = EntropyCalculator::new();
        let empty_file: syn::File = syn::parse_str("").unwrap();
        let diversity = calculator.calculate_ast_diversity(&empty_file);
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_tokenize_empty() {
        let calculator = EntropyCalculator::new();
        let tokens = calculator.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_simple() {
        let calculator = EntropyCalculator::new();
        let tokens = calculator.tokenize("fn foo");
        assert_eq!(tokens, vec!["fn", "foo"]);
    }

    #[test]
    fn test_tokenize_with_delimiters() {
        let calculator = EntropyCalculator::new();
        let tokens = calculator.tokenize("foo()");
        assert!(tokens.contains(&"foo".to_string()));
        assert!(tokens.contains(&"(".to_string()));
        assert!(tokens.contains(&")".to_string()));
    }

    #[test]
    fn test_tokenize_with_braces() {
        let calculator = EntropyCalculator::new();
        let tokens = calculator.tokenize("{ x }");
        assert!(tokens.contains(&"{".to_string()));
        assert!(tokens.contains(&"x".to_string()));
        assert!(tokens.contains(&"}".to_string()));
    }

    #[test]
    fn test_tokenize_with_semicolon() {
        let calculator = EntropyCalculator::new();
        let tokens = calculator.tokenize("let x;");
        assert!(tokens.contains(&"let".to_string()));
        assert!(tokens.contains(&"x".to_string()));
        assert!(tokens.contains(&";".to_string()));
    }

    #[test]
    fn test_tokenize_preserves_identifiers() {
        let calculator = EntropyCalculator::new();
        let tokens = calculator.tokenize("foo_bar_baz");
        assert_eq!(tokens, vec!["foo_bar_baz"]);
    }

    // AST diversity tests
    #[test]
    fn test_calculate_ast_diversity_empty() {
        let calculator = EntropyCalculator::new();
        let ast = syn::parse_file("").unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_single_fn() {
        let calculator = EntropyCalculator::new();
        let ast = syn::parse_file("fn foo() {}").unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        // Single item type = 0 entropy
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_mixed_items() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            fn foo() {}
            struct Bar {}
            enum Baz { A, B }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        // Three different item types
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_impl() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            struct Foo {}
            impl Foo {
                fn bar(&self) {}
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_trait() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            trait Foo {
                fn bar(&self);
            }
            struct Baz {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_mod() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            mod inner {
                fn foo() {}
            }
            fn bar() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_use() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            use std::io;
            fn foo() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_type_alias() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            type MyInt = i32;
            fn foo() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_const() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            const FOO: i32 = 42;
            fn bar() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_with_static() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            static FOO: i32 = 42;
            fn bar() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_ast_diversity_multiple_same_type() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            fn foo() {}
            fn bar() {}
            fn baz() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let diversity = calculator.calculate_ast_diversity(&ast);
        // All same type = 0 entropy
        assert_eq!(diversity, 0.0);
    }

    // Edge cases
    #[test]
    fn test_calculate_very_long_string() {
        let calculator = EntropyCalculator::new();
        let long_string: String = (0..1000).map(|i| (b'a' + (i % 26) as u8) as char).collect();
        let entropy = calculator.calculate(&long_string);
        // 26 characters used uniformly
        assert!(entropy > 4.0);
    }

    #[test]
    fn test_calculate_binary_like() {
        let calculator = EntropyCalculator::new();
        let binary = "0101010101";
        let entropy = calculator.calculate(binary);
        // Two characters, equal frequency = 1 bit
        assert!((entropy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_token_entropy_realistic_code() {
        let calculator = EntropyCalculator::new();
        let code = r#"
            fn calculate_sum(items: Vec<i32>) -> i32 {
                let mut sum = 0;
                for item in items {
                    sum += item;
                }
                sum
            }
        "#;
        let entropy = calculator.calculate_token_entropy(code);
        assert!(entropy > 0.0);
    }
}
