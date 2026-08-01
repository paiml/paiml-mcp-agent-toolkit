// Detection helpers for BigOAnalyzer
// This file is include!()'d into big_o_analyzer.rs scope.
// NO use imports or #! inner attributes allowed.

impl BigOAnalyzer {
    fn get_loop_keywords(language: &str) -> Vec<&'static str> {
        match language {
            "rust" => vec!["for", "while", "loop"],
            "javascript" | "typescript" => vec!["for", "while", "do"],
            "python" => vec!["for", "while"],
            _ => vec!["for", "while"],
        }
    }

    /// True when `line` calls `function_name` somewhere other than its own
    /// definition.
    ///
    /// #661: the old test was `contains(name) && !starts_with("fn")`. Because
    /// the analyzed text began at the function's *name*, its first line always
    /// contained the name and never started with "fn" — so every function
    /// looked recursive, `determine_time_complexity` returned `unknown()` with
    /// confidence 0, the averaged confidence fell to 45, and the whole function
    /// was dropped below the default threshold of 50. That is why `analyze
    /// big-o` reported "Total Functions Analyzed: 0" on files whose functions
    /// contain no loops.
    fn detect_recursive_call(line: &str, function_name: &str) -> bool {
        if function_name.is_empty() {
            return false;
        }
        let needle = format!("{function_name}(");
        let mut from = 0usize;
        while let Some(rel) = line.get(from..).and_then(|tail| tail.find(&needle)) {
            let at = from + rel;
            let before = line.get(..at).unwrap_or_default().trim_end();
            if !Self::is_definition_keyword_suffix(before) {
                return true;
            }
            from = at + needle.len();
        }
        false
    }

    /// Does `before` end with the keyword that introduces a definition of the
    /// name that follows it (so the occurrence is the definition, not a call)?
    fn is_definition_keyword_suffix(before: &str) -> bool {
        ["fn", "def", "func", "function", "fun"]
            .iter()
            .any(|kw| before.ends_with(kw))
    }

    fn detect_sorting_operation(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains(".sort(") || trimmed.contains("sort(")
    }

    fn detect_binary_search(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains("binary_search") || trimmed.contains("binarySearch")
    }

    fn detect_rust_iterator_patterns(function_body: &str) -> bool {
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

    /// `trimmed` begins with `keyword` as a whole word.
    fn starts_with_keyword(trimmed: &str, keyword: &str) -> bool {
        trimmed.strip_prefix(keyword).is_some_and(|rest| {
            rest.is_empty()
                || !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '!')
        })
    }

    fn calculate_loop_depth(lines: &[&str], loop_keywords: &[&str]) -> usize {
        let mut loop_depth = 0;
        let mut max_loop_depth = 0;

        for line in lines {
            let trimmed = line.trim();

            // Track loop depth
            for keyword in loop_keywords {
                // Word boundary required: `format!(..)` starts with "for" but
                // is not a loop, and counting it reported loop-free functions
                // as O(n).
                if Self::starts_with_keyword(trimmed, keyword) {
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
