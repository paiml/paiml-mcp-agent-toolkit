use std::collections::HashMap;

pub struct EntropyCalculator;

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
