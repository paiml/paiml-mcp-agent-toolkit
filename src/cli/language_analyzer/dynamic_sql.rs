// SqlAnalyzer trait impl and methods (included from dynamic.rs)

impl LanguageAnalyzer for SqlAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let upper = content.to_uppercase();
        let upper_lines: Vec<&str> = upper.lines().collect();

        for (line_num, uline) in upper_lines.iter().enumerate() {
            let trimmed = uline.trim();
            if let Some(name) = Self::extract_sql_object_name(trimmed) {
                let line_end = Self::find_sql_block_end(&upper_lines, line_num);
                functions.push(FunctionInfo {
                    name,
                    line_start: line_num,
                    line_end,
                });
            }
        }

        // Extract CTEs (WITH name AS (...))
        for (line_num, uline) in upper_lines.iter().enumerate() {
            let trimmed = uline.trim();
            if let Some(rest) = trimmed.strip_prefix("WITH ") {
                for cte_name in Self::extract_cte_names(rest, &upper_lines, line_num) {
                    functions.push(FunctionInfo {
                        name: cte_name.to_lowercase(),
                        line_start: line_num,
                        line_end: line_num,
                    });
                }
            }
        }

        functions
    }

    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics {
        let lines: Vec<&str> = content.lines().collect();
        let end = function.line_end.min(lines.len().saturating_sub(1));
        let func_lines = &lines[function.line_start..=end];

        let mut cyclomatic: u16 = 1;
        let mut cognitive: u16 = 0;
        let mut nesting: u8 = 0;
        let mut max_nesting: u8 = 0;

        for line in func_lines {
            let upper = line.trim().to_uppercase();
            // Control flow keywords
            for kw in &[
                "IF ", "ELSIF ", "ELSEIF ", "WHEN ", "LOOP", "WHILE ", "FOR ",
            ] {
                if upper.starts_with(kw) || upper.contains(&format!(" {kw}")) {
                    cyclomatic += 1;
                    cognitive += 1 + u16::from(nesting);
                }
            }
            if upper.contains(" AND ") || upper.contains(" OR ") {
                cyclomatic += 1;
                cognitive += 1;
            }
            if upper.starts_with("BEGIN")
                || upper.starts_with("LOOP")
                || upper.starts_with("IF ")
            {
                nesting += 1;
                max_nesting = max_nesting.max(nesting);
            }
            if upper.starts_with("END") {
                nesting = nesting.saturating_sub(1);
            }
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

impl SqlAnalyzer {
    /// Extract name from CREATE FUNCTION/VIEW/TRIGGER/PROCEDURE statements
    fn extract_sql_object_name(trimmed_upper: &str) -> Option<String> {
        // Pattern: CREATE [OR REPLACE] (FUNCTION|PROCEDURE|VIEW|TRIGGER) name
        let rest = if let Some(r) = trimmed_upper.strip_prefix("CREATE OR REPLACE ") {
            r
        } else { trimmed_upper.strip_prefix("CREATE ")? };

        let (kind_prefix, after_kind) = if let Some(a) = rest.strip_prefix("FUNCTION ") {
            ("fn:", a)
        } else if let Some(a) = rest.strip_prefix("PROCEDURE ") {
            ("proc:", a)
        } else if let Some(a) = rest.strip_prefix("VIEW ") {
            ("view:", a)
        } else if let Some(a) = rest.strip_prefix("TRIGGER ") {
            ("trigger:", a)
        } else {
            return None;
        };

        let name = after_kind
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .next()
            .unwrap_or("");

        if name.is_empty() {
            return None;
        }
        Some(format!("{kind_prefix}{}", name.to_lowercase()))
    }

    /// Find the end of a SQL block (delimited by ; or $$ or END;)
    fn find_sql_block_end(upper_lines: &[&str], start: usize) -> usize {
        let mut depth: i32 = 0;
        for (i, line) in upper_lines.iter().enumerate().skip(start) {
            let trimmed = line.trim();
            if trimmed.contains("BEGIN") {
                depth += 1;
            }
            if trimmed.starts_with("END") && (trimmed.contains(';') || trimmed == "END") {
                depth -= 1;
                if depth <= 0 {
                    return i;
                }
            }
            if depth == 0 && i > start && trimmed.ends_with(';') {
                return i;
            }
            if trimmed.contains("$$") && i > start {
                return i;
            }
        }
        upper_lines.len().saturating_sub(1)
    }

    /// Extract CTE names from a WITH clause
    fn extract_cte_names(first_rest: &str, _lines: &[&str], _start: usize) -> Vec<String> {
        let mut names = Vec::new();
        // First CTE: WITH name AS
        let name = first_rest
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or("");
        if !name.is_empty() && name != "RECURSIVE" {
            names.push(name.to_string());
        }
        names
    }
}
