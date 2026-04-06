// Detection helpers for BigOAnalyzer
// This file is include!()'d into big_o_analyzer.rs scope.
// NO use imports or #! inner attributes allowed.

impl BigOAnalyzer {
    fn get_loop_keywords(language: &str) -> Vec<&'static str> {
        debug_assert!(!language.is_empty(), "language must not be empty");
        match language {
            "rust" => vec!["for", "while", "loop"],
            "javascript" | "typescript" => vec!["for", "while", "do"],
            "python" => vec!["for", "while"],
            _ => vec!["for", "while"],
        }
    }

    fn detect_recursive_call(line: &str, function_name: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        debug_assert!(!function_name.is_empty(), "function_name must not be empty");
        let trimmed = line.trim();
        trimmed.contains(function_name)
            && !trimmed.starts_with("fn")
            && !trimmed.starts_with("function")
    }

    fn detect_sorting_operation(line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let trimmed = line.trim();
        trimmed.contains(".sort(") || trimmed.contains("sort(")
    }

    fn detect_binary_search(line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let trimmed = line.trim();
        trimmed.contains("binary_search") || trimmed.contains("binarySearch")
    }

    fn detect_rust_iterator_patterns(function_body: &str) -> bool {
        debug_assert!(!function_body.is_empty(), "function_body must not be empty");
        let linear_patterns = [
            ".iter()",
            ".into_iter()",
            ".sum()",
            ".collect()",
            ".map(",
            ".filter(",
            ".fold(",
            ".reduce(",
            ".for_each(",
            ".find(",
            ".any(",
            ".all(",
        ];

        linear_patterns
            .iter()
            .any(|pattern| function_body.contains(pattern))
    }

    fn calculate_loop_depth(lines: &[&str], loop_keywords: &[&str]) -> usize {
        debug_assert!(!lines.is_empty(), "lines must not be empty");
        let mut loop_depth = 0;
        let mut max_loop_depth = 0;

        for line in lines {
            let trimmed = line.trim();

            // Track loop depth
            for keyword in loop_keywords {
                if trimmed.starts_with(keyword) {
                    loop_depth += 1;
                    max_loop_depth = max_loop_depth.max(loop_depth);
                }
            }

            if trimmed.contains('}') && loop_depth > 0 {
                loop_depth -= 1;
            }
        }

        max_loop_depth
    }

    fn determine_time_complexity(max_loop_depth: usize, has_recursion: bool) -> ComplexityBound {
        debug_assert!(true, "contract: determine_time_complexity");
        if has_recursion && max_loop_depth == 0 {
            return ComplexityBound::unknown();
        }

        match max_loop_depth {
            0 => ComplexityBound::constant().with_confidence(90),
            1 => ComplexityBound::linear().with_confidence(80),
            2 => ComplexityBound::quadratic().with_confidence(75),
            3 => ComplexityBound::polynomial(3, 1).with_confidence(70),
            n => ComplexityBound::polynomial(n as u32, 1).with_confidence(60),
        }
    }

    fn detect_space_complexity(function_body: &str) -> (ComplexityBound, bool) {
        debug_assert!(!function_body.is_empty(), "function_body must not be empty");
        let space_indicators = [
            "Vec::new",
            "vec!",
            "HashMap::new",
            "HashSet::new",
            "BTreeMap::new",
            "[]",
        ];

        let has_allocation = space_indicators
            .iter()
            .any(|indicator| function_body.contains(indicator));

        if has_allocation {
            (ComplexityBound::linear().with_confidence(70), true)
        } else {
            (ComplexityBound::constant().with_confidence(90), false)
        }
    }
}
