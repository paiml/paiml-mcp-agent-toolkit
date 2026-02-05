use std::collections::HashMap;

pub struct EntropyCalculator;

impl Default for EntropyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(&self, source: &str) -> f64 {
        if source.is_empty() {
            return 0.0;
        }

        let mut char_counts = HashMap::new();
        let total = source.len() as f64;

        // Count character frequencies
        for ch in source.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        // Calculate Shannon entropy
        let mut entropy = 0.0;
        for count in char_counts.values() {
            let probability = *count as f64 / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    pub fn calculate_token_entropy(&self, source: &str) -> f64 {
        // Tokenize source code and calculate entropy based on tokens
        let tokens = self.tokenize(source);
        if tokens.is_empty() {
            return 0.0;
        }

        let mut token_counts = HashMap::new();
        let total = tokens.len() as f64;

        for token in &tokens {
            *token_counts.entry(token.as_str()).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in token_counts.values() {
            let probability = *count as f64 / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    fn tokenize(&self, source: &str) -> Vec<String> {
        // Simple tokenization based on whitespace and common delimiters
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in source.chars() {
            if ch.is_whitespace() || "{}[](),;:.".contains(ch) {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                if !ch.is_whitespace() {
                    tokens.push(ch.to_string());
                }
            } else {
                current_token.push(ch);
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    pub fn calculate_ast_diversity(&self, ast: &syn::File) -> f64 {
        // Calculate diversity based on AST node types
        let mut node_types = HashMap::new();
        let mut total = 0;

        // Count different types of syntax nodes
        for item in &ast.items {
            let node_type = match item {
                syn::Item::Fn(_) => "function",
                syn::Item::Struct(_) => "struct",
                syn::Item::Enum(_) => "enum",
                syn::Item::Impl(_) => "impl",
                syn::Item::Trait(_) => "trait",
                syn::Item::Mod(_) => "module",
                syn::Item::Use(_) => "use",
                syn::Item::Type(_) => "type",
                syn::Item::Const(_) => "const",
                syn::Item::Static(_) => "static",
                _ => "other",
            };

            *node_types.entry(node_type).or_insert(0) += 1;
            total += 1;
        }

        if total == 0 {
            return 0.0;
        }

        // Calculate entropy
        let mut entropy = 0.0;
        for count in node_types.values() {
            let probability = *count as f64 / total as f64;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        // Scale to make it comparable to character entropy
        entropy * 2.0
    }
}

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
