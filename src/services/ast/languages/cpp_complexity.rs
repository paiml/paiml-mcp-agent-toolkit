/// C++ complexity analyzer for extracting C++-specific metrics (complexity ≤10)
#[cfg(feature = "cpp-ast")]
pub struct CppComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "cpp-ast")]
impl Default for CppComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of a C++ source line for complexity analysis
struct LineClassification {
    is_decision_point: bool,
    is_nesting_construct: bool,
    is_template: bool,
    opens_brace: bool,
    closes_brace: bool,
}

fn classify_line(trimmed: &str) -> LineClassification {
    let is_if_stmt = trimmed.starts_with("if ") || trimmed.starts_with("if(");
    let is_else_if = trimmed.starts_with("else if") || trimmed.contains("} else if");
    let is_switch = trimmed.starts_with("switch ");
    let is_case = trimmed.starts_with("case ") || trimmed.starts_with("default:");
    let is_loop = trimmed.starts_with("while ")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("do ");
    let is_try = trimmed.starts_with("try ");
    let is_catch = trimmed.starts_with("catch ");
    let is_goto = trimmed.starts_with("goto ");
    let is_ternary = trimmed.contains(" ? ") && trimmed.contains(" : ");
    let is_range_for = trimmed.contains("for (") && trimmed.contains(" : ");
    let is_lambda = trimmed.contains("[")
        && trimmed.contains("]")
        && (trimmed.contains("(") || trimmed.contains("mutable"));
    let is_template = trimmed.contains("<") && trimmed.contains(">");

    LineClassification {
        is_decision_point: is_if_stmt
            || is_else_if
            || is_switch
            || is_case
            || is_loop
            || is_catch
            || is_goto
            || is_ternary
            || is_range_for,
        is_nesting_construct: is_if_stmt
            || is_else_if
            || is_switch
            || is_loop
            || is_try
            || is_lambda,
        is_template,
        opens_brace: trimmed.contains("{") && !trimmed.contains("}"),
        closes_brace: trimmed.contains("}"),
    }
}

fn is_skippable_line(trimmed: &str, in_comment: &mut bool) -> bool {
    if trimmed.starts_with('#') {
        return true;
    }
    if trimmed.contains("/*") {
        *in_comment = true;
    }
    if *in_comment {
        if trimmed.contains("*/") {
            *in_comment = false;
        }
        return true;
    }
    trimmed.starts_with("//")
}

fn is_function_start(trimmed: &str) -> bool {
    trimmed.contains("{") && (trimmed.contains("(") || trimmed.contains(")"))
}

impl CppComplexityAnalyzer {
    /// Creates a new C++ complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of C++ source code
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 0;

        let mut nesting_depth = 0;
        let mut in_comment = false;
        let mut in_function = false;

        for line in source.lines() {
            let trimmed = line.trim();

            if is_skippable_line(trimmed, &mut in_comment) {
                continue;
            }

            if !in_function && is_function_start(trimmed) {
                in_function = true;
            }

            if in_function {
                let cl = classify_line(trimmed);
                self.update_complexity(&cl, &mut nesting_depth);
                if cl.closes_brace {
                    nesting_depth = nesting_depth.saturating_sub(1);
                    if nesting_depth == 0 {
                        in_function = false;
                    }
                }
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }

    fn update_complexity(&mut self, cl: &LineClassification, nesting_depth: &mut u32) {
        if cl.is_decision_point {
            self.cyclomatic_complexity += 1;
        }
        if cl.is_nesting_construct {
            self.cognitive_complexity += 1 + *nesting_depth;
            *nesting_depth += 1;
        } else if cl.is_template {
            self.cognitive_complexity += 1;
        }
        if cl.opens_brace {
            *nesting_depth += 1;
        }
    }
}
