// ScalaAnalyzer trait impl and methods (included from dynamic.rs)

impl LanguageAnalyzer for ScalaAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(name) = Self::extract_scala_name(trimmed) {
                let line_end = if trimmed.contains('{') {
                    find_brace_balanced_end(&lines, line_num, false)
                } else {
                    // Single-expression def: find next blank or next def
                    Self::find_expression_end(&lines, line_num)
                };
                functions.push(FunctionInfo {
                    name,
                    line_start: line_num,
                    line_end,
                });
            }
        }
        functions
    }

    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let lines: Vec<&str> = content.lines().collect();
        let end = function.line_end.min(lines.len().saturating_sub(1));
        let func_lines = &lines[function.line_start..=end];

        let mut cyclomatic: u16 = 1;
        let mut cognitive: u16 = 0;
        let mut nesting: u8 = 0;
        let mut max_nesting: u8 = 0;

        for line in func_lines {
            let trimmed = line.trim();
            if trimmed.starts_with("if ")
                || trimmed.starts_with("if(")
                || trimmed.contains(" if ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("for(")
                || trimmed.contains("catch ")
            {
                cyclomatic += 1;
                cognitive += 1 + u16::from(nesting);
            }
            if trimmed.contains(" && ") || trimmed.contains(" || ") {
                cyclomatic += 1;
                cognitive += 1;
            }
            nesting += trimmed.matches('{').count() as u8;
            nesting = nesting.saturating_sub(trimmed.matches('}').count() as u8);
            max_nesting = max_nesting.max(nesting);
        }

        ComplexityMetrics {
            cyclomatic: cyclomatic.min(255),
            cognitive: cognitive.min(255),
            nesting_max: max_nesting,
            lines: func_lines.len() as u16,
            halstead: None,
        }
    }
}

impl ScalaAnalyzer {
    fn extract_scala_name(trimmed: &str) -> Option<String> {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        // Match: def name, class name, object name, trait name
        let prefixes = [
            "def ",
            "override def ",
            "private def ",
            "protected def ",
            "class ",
            "case class ",
            "abstract class ",
            "object ",
            "trait ",
        ];
        for prefix in &prefixes {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = rest
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn find_expression_end(lines: &[&str], start: usize) -> usize {
        for i in (start + 1)..lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty()
                || trimmed.starts_with("def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("object ")
                || trimmed.starts_with("trait ")
                || trimmed.starts_with("val ")
                || trimmed.starts_with("var ")
                || trimmed.starts_with("}")
            {
                return i.saturating_sub(1).max(start);
            }
        }
        lines.len().saturating_sub(1)
    }
}
