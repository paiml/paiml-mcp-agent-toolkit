// ComplexityAnalyzer impl: function analysis, Halstead metrics
// Free functions: tokenize, is_operator, is_operand, count_lines, maintainability index

impl ComplexityAnalyzer {
    pub fn analyze_functions(&self, ast: &File) -> Vec<FunctionMetrics> {
        let mut functions = Vec::new();

        for item in &ast.items {
            if let Item::Fn(func) = item {
                let name = func.sig.ident.to_string();
                let complexity = self.calculate_function_complexity(func);
                let lines = count_lines(&func.block);
                let parameters = func.sig.inputs.len();

                functions.push(FunctionMetrics {
                    name,
                    complexity,
                    lines,
                    parameters,
                });
            }
        }

        functions
    }

    fn calculate_function_complexity(&self, func: &syn::ItemFn) -> u32 {
        // Use the existing method from ComplexityAnalyzer
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![syn::Item::Fn(func.clone())],
        };
        self.calculate_cyclomatic(&file)
    }

    pub fn calculate_halstead_metrics(&self, code: &str) -> HalsteadMetrics {
        debug_assert!(!code.is_empty(), "code must not be empty");
        let mut operators = HashMap::new();
        let mut operands = HashMap::new();

        // Tokenize and classify
        for token in tokenize(code) {
            if is_operator(&token) {
                *operators.entry(token).or_insert(0) += 1;
            } else if is_operand(&token) {
                *operands.entry(token).or_insert(0) += 1;
            }
        }

        let n1 = operators.len();
        let n2 = operands.len();
        let n1_total: usize = operators.values().sum();
        let n2_total: usize = operands.values().sum();

        let vocabulary = n1 + n2;
        let length = n1_total + n2_total;
        let volume = if vocabulary > 0 {
            length as f64 * (vocabulary as f64).log2()
        } else {
            0.0
        };

        let difficulty = if n2 > 0 && n2_total > 0 {
            (n1 as f64 / 2.0) * (n2_total as f64 / n2 as f64)
        } else {
            0.0
        };

        let effort = difficulty * volume;

        HalsteadMetrics {
            vocabulary,
            length,
            volume,
            difficulty,
            effort,
        }
    }
}

fn count_lines(block: &syn::Block) -> usize {
    block.stmts.len()
}

fn tokenize(code: &str) -> Vec<String> {
    debug_assert!(!code.is_empty(), "code must not be empty");
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in code.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            if !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn is_operator(token: &str) -> bool {
    debug_assert!(!token.is_empty(), "token must not be empty");
    matches!(
        token,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "="
            | "=="
            | "!="
            | "<"
            | ">"
            | "<="
            | ">="
            | "&&"
            | "||"
            | "!"
            | "&"
            | "|"
            | "^"
            | "<<"
            | ">>"
            | "+="
            | "-="
            | "*="
            | "/="
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
    )
}

fn is_operand(token: &str) -> bool {
    debug_assert!(!token.is_empty(), "token must not be empty");
    !is_operator(token) && !token.chars().all(|c| c.is_ascii_punctuation())
}

// Maintainability Index calculation
pub fn calculate_maintainability_index(
    halstead_volume: f64,
    cyclomatic_complexity: u32,
    lines_of_code: usize,
) -> f64 {
    // Contract: calculate_maintainability_index returns a bounded score
    let mi = 171.0
        - 5.2 * halstead_volume.ln()
        - 0.23 * cyclomatic_complexity as f64
        - 16.2 * (lines_of_code as f64).ln();

    mi.max(0.0).min(100.0)
}
